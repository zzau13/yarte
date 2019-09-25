use std::{
    collections::{BTreeMap, HashMap},
    fmt::{self, Write},
    mem,
    path::PathBuf,
};

use mime_guess::from_ext;
use syn::parse_str;
use syn::visit::Visit;
use v_eval::{ctx_as_ref, eval};
use v_htmlescape::escape;
use yarte_config::Config;

use crate::parser::{Helper, Node, Ws};

mod identifier;
mod scope;
mod validator;
mod visit_derive;
mod visit_each;
mod visit_partial;
mod visits;

use self::scope::Scope;
use self::visit_derive::Struct;
use self::visit_each::find_loop_var;
use self::visit_partial::visit_partial;

pub(crate) use self::visit_derive::{visit_derive, Print};

pub(crate) fn generate(c: &Config, s: &Struct, ctx: Context) -> String {
    Generator::new(c, s, ctx).build()
}

pub(crate) trait EWrite: fmt::Write {
    fn write(&mut self, s: &dyn fmt::Display) {
        write!(self, "{}", s).unwrap()
    }

    fn writeln(&mut self, s: &dyn fmt::Display) {
        writeln!(self, "{}", s).unwrap()
    }
}

impl EWrite for String {}

pub(self) type Context<'a> = &'a BTreeMap<&'a PathBuf, Vec<Node<'a>>>;

#[derive(Debug, PartialEq)]
pub(self) enum On {
    Each(usize),
    With(usize),
}

#[derive(Debug)]
enum Writable<'a> {
    Lit(&'a str),
    LitP(String),
    Expr(String, bool),
}

pub(self) struct Generator<'a> {
    pub(self) c: &'a Config<'a>,
    // ast of DeriveInput
    pub(self) s: &'a Struct<'a>,
    // buffer for tokens
    pub(self) buf_t: String,
    // Scope stack
    pub(self) scp: Scope,
    // On State stack
    pub(self) on: Vec<On>,
    // On partial scope
    pub(self) partial: Option<(HashMap<String, syn::Expr>, usize)>,
    // buffer for writable
    buf_w: Vec<Writable<'a>>,
    // path - nodes
    ctx: Context<'a>,
    // current file path
    on_path: PathBuf,
    // whitespace flag and buffer based on https://github.com/djc/askama
    next_ws: Option<&'a str>,
    skip_ws: bool,
}

impl<'a> Generator<'a> {
    fn new<'n>(c: &'n Config<'n>, s: &'n Struct<'n>, ctx: Context<'n>) -> Generator<'n> {
        Generator {
            c,
            s,
            ctx,
            buf_t: String::new(),
            buf_w: vec![],
            next_ws: None,
            on: vec![],
            on_path: s.path.clone(),
            partial: None,
            scp: Scope::new("self".to_owned(), 0),
            skip_ws: false,
        }
    }

    fn build(&mut self) -> String {
        let mut buf = String::new();

        let nodes: &[Node] = self.ctx.get(&self.on_path).unwrap();
        let size_hint = self.display(nodes, &mut buf);

        self.template(size_hint, &mut buf);

        if cfg!(feature = "actix-web") {
            self.responder(&mut buf);
        }

        buf
    }

    #[inline]
    fn get_mime(&mut self) -> String {
        let ext = if self.s.wrapped {
            match self.s.path.extension() {
                Some(s) => s.to_str().unwrap(),
                None => "txt",
            }
        } else {
            "html"
        };

        from_ext(ext).first_or_text_plain().to_string()
    }

    fn template(&mut self, size_hint: usize, buf: &mut String) {
        debug_assert_ne!(size_hint, 0);
        self.s.implement_head("::yarte::Template", buf);

        buf.writeln(&"fn mime() -> &'static str {");

        let mut mime = self.get_mime();
        mime.push_str("; charset=utf-8");
        writeln!(buf, "{:?}", mime).unwrap();

        buf.writeln(&"}");
        buf.writeln(&"fn size_hint() -> usize {");
        if cfg!(debug_assertions) {
            buf.writeln(&"// In release, the stream will be optimized");
            buf.writeln(&"// size_hint can been changed");
        }
        buf.writeln(&size_hint);
        buf.writeln(&"}");
        buf.writeln(&"}");
    }

    fn display(&mut self, nodes: &'a [Node], buf: &mut String) -> usize {
        self.s.implement_head("::std::fmt::Display", buf);

        buf.writeln(&"fn fmt(&self, _fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {");

        let last = buf.len();

        self.handle(nodes, buf);
        self.write_buf_writable(buf);
        // heuristic based on https://github.com/lfairy/maud
        let size_hint = 1 + buf.len() - last;

        debug_assert_eq!(self.scp.len(), 1);
        debug_assert_eq!(self.scp.root(), "self");
        debug_assert!(self.on.is_empty());
        debug_assert!(self.buf_t.is_empty());
        debug_assert!(self.buf_w.is_empty());
        debug_assert_eq!(self.on_path, self.s.path);
        debug_assert_eq!(self.next_ws, None);

        buf.writeln(&quote!(Ok(())));

        buf.writeln(&"}");
        buf.writeln(&"}");
        size_hint
    }

    fn responder(&mut self, buf: &mut String) {
        self.s.implement_head("::yarte::aw::Responder", buf);

        buf.writeln(&quote!(
            type Error = ::yarte::aw::Error;
            type Future = ::yarte::aw::FutureResult<::yarte::aw::HttpResponse, ::yarte::aw::Error>;
        ));

        buf.writeln(&quote!(
            #[inline]
            fn respond_to(self, _req: &::yarte::aw::HttpRequest) -> Self::Future
            {
                match self.call() {
                    Ok(val) => {
                        ::yarte::aw::ok(::yarte::aw::HttpResponse::Ok().content_type(Self::mime()).body(val))
                    }
                    Err(_) => {
                        ::yarte::aw::err(::yarte::aw::ErrorInternalServerError("Template parsing error"))
                    }
                }
            }
        ));

        buf.writeln(&"}");
    }

    fn handle(&mut self, nodes: &'a [Node], buf: &mut String) {
        for n in nodes {
            match n {
                Node::Local(expr) => {
                    validator::statement(expr);

                    self.skip_ws();
                    self.write_buf_writable(buf);
                    self.visit_stmt(expr);
                    buf.writeln(&mem::replace(&mut self.buf_t, String::new()));
                }
                Node::Safe(ws, expr) => {
                    let expr: &syn::Expr = &*expr;
                    validator::expression(expr);

                    self.visit_expr(expr);
                    self.handle_ws(*ws);

                    if let Some(..) = self.partial {
                        if let syn::Expr::Path(..) = expr {
                            if let Ok(lit) = syn::parse_str::<syn::Lit>(&self.buf_t) {
                                self.buf_t = String::new();
                                use syn::Lit::*;
                                match lit {
                                    Byte(b) => self.buf_w.push(Writable::LitP(
                                        String::from_utf8(vec![b.value()]).unwrap(),
                                    )),
                                    ByteStr(b) => self.buf_w.push(Writable::LitP(
                                        String::from_utf8(b.value()).unwrap(),
                                    )),
                                    Str(b) => self.buf_w.push(Writable::LitP(b.value())),
                                    Char(b) => {
                                        self.buf_w.push(Writable::LitP(b.value().to_string()))
                                    }
                                    Bool(b) => self.buf_w.push(Writable::LitP(b.value.to_string())),
                                    Float(b) => {
                                        self.buf_w.push(Writable::LitP(b.base10_digits().into()))
                                    }
                                    Int(b) => {
                                        self.buf_w.push(Writable::LitP(b.base10_digits().into()))
                                    }
                                    _ => panic!(
                                        "Not allowed verbatim expression in a template expression"
                                    ),
                                };
                                continue;
                            }
                        }
                    }

                    self.buf_w.push(Writable::Expr(
                        mem::replace(&mut self.buf_t, String::new()),
                        true,
                    ));
                }
                Node::Expr(ws, expr) => {
                    let expr: &syn::Expr = &*expr;
                    validator::expression(expr);

                    self.visit_expr(expr);
                    self.handle_ws(*ws);
                    if let Some(..) = self.partial {
                        if let syn::Expr::Path(..) = expr {
                            if let Ok(lit) = syn::parse_str::<syn::Lit>(&self.buf_t) {
                                self.buf_t = String::new();
                                use syn::Lit::*;
                                match lit {
                                    Byte(b) => {
                                        self.buf_w.push(Writable::LitP(
                                            escape(&String::from_utf8(vec![b.value()]).unwrap())
                                                .to_string(),
                                        ));
                                    }
                                    ByteStr(b) => {
                                        self.buf_w.push(Writable::LitP(
                                            escape(&String::from_utf8(b.value()).unwrap())
                                                .to_string(),
                                        ));
                                    }
                                    Char(b) => {
                                        self.buf_w.push(Writable::LitP(
                                            escape(&b.value().to_string()).to_string(),
                                        ));
                                    }
                                    Str(b) => {
                                        self.buf_w
                                            .push(Writable::LitP(escape(&b.value()).to_string()));
                                    }
                                    Bool(b) => self.buf_w.push(Writable::LitP(b.value.to_string())),
                                    Float(b) => {
                                        self.buf_w.push(Writable::LitP(b.base10_digits().into()))
                                    }
                                    Int(b) => {
                                        self.buf_w.push(Writable::LitP(b.base10_digits().into()))
                                    }
                                    _ => panic!(
                                        "Not allowed verbatim expression in a template expression"
                                    ),
                                }
                                continue;
                            }
                        }
                    }

                    self.buf_w.push(Writable::Expr(
                        mem::replace(&mut self.buf_t, String::new()),
                        false,
                    ))
                }
                Node::Lit(l, lit, r) => self.visit_lit(l, lit, r),
                Node::Helper(h) => self.visit_helper(buf, h),
                Node::Partial(ws, path, expr) => self.visit_partial(buf, *ws, path, expr),
                Node::Comment(c) => {
                    self.skip_ws();
                    if cfg!(debug_assertions) {
                        self.write_buf_writable(buf);
                        for line in c.lines() {
                            buf.write(&"//");
                            buf.writeln(&line.trim_end());
                        }
                    }
                }
                Node::Raw(ws, l, v, r) => {
                    self.handle_ws(ws.0);
                    self.visit_lit(l, v, r);
                    self.handle_ws(ws.1);
                }
            }
        }
    }

    fn visit_lit(&mut self, lws: &'a str, lit: &'a str, rws: &'a str) {
        debug_assert!(self.next_ws.is_none());
        if !lws.is_empty() {
            if self.skip_ws {
                self.skip_ws = false;
            } else if lit.is_empty() {
                debug_assert!(rws.is_empty());
                self.next_ws = Some(lws);
            } else {
                self.buf_w.push(Writable::Lit(lws));
            }
        }

        if !lit.is_empty() {
            self.buf_w.push(Writable::Lit(lit));
        }

        if !rws.is_empty() {
            self.next_ws = Some(rws);
        }
    }

    fn visit_helper(&mut self, buf: &mut String, h: &'a Helper<'a>) {
        use crate::parser::Helper::*;
        match h {
            Each(ws, e, b) => self.visit_each(buf, *ws, e, b),
            If(ifs, elsif, els) => self.visit_if(buf, ifs, elsif, els),
            With(ws, e, b) => self.visit_with(buf, *ws, e, b),
            Unless(ws, e, b) => self.visit_unless(buf, *ws, e, b),
            Defined(..) => unimplemented!(),
        }
    }

    fn visit_unless(
        &mut self,
        buf: &mut String,
        ws: (Ws, Ws),
        cond: &'a syn::Expr,
        nodes: &'a [Node<'a>],
    ) {
        validator::unless(cond);

        self.handle_ws(ws.0);
        if let Some(val) = self.eval_bool(cond) {
            if !val {
                self.scp.push_scope(vec![]);
                self.handle(nodes, buf);
                self.scp.pop();
                self.handle_ws(ws.1);
            }
        } else {
            self.write_buf_writable(buf);

            self.visit_expr(cond);
            writeln!(
                buf,
                "if !({}) {{",
                mem::replace(&mut self.buf_t, String::new())
            )
            .unwrap();

            self.scp.push_scope(vec![]);
            self.handle(nodes, buf);
            self.scp.pop();

            self.handle_ws(ws.1);
            self.write_buf_writable(buf);

            buf.writeln(&"}");
        }
    }

    fn visit_with(
        &mut self,
        buf: &mut String,
        ws: (Ws, Ws),
        args: &'a syn::Expr,
        nodes: &'a [Node<'a>],
    ) {
        validator::scope(args);

        self.handle_ws(ws.0);
        self.visit_expr(args);
        self.on.push(On::With(self.scp.len()));
        self.scp
            .push_scope(vec![mem::replace(&mut self.buf_t, String::new())]);

        self.handle(nodes, buf);

        self.on.pop();
        self.scp.pop();
        self.handle_ws(ws.1);
    }

    fn visit_each(
        &mut self,
        buf: &mut String,
        ws: (Ws, Ws),
        args: &'a syn::Expr,
        nodes: &'a [Node<'a>],
    ) {
        validator::each(args);

        self.handle_ws(ws.0);
        self.write_buf_writable(buf);

        let loop_var = find_loop_var(self.c, self.ctx, self.on_path.clone(), nodes);
        self.visit_expr(args);
        let id = self.scp.len();
        let ctx = if loop_var {
            let ctx = vec![format!("_key_{}", id), format!("_index_{}", id)];
            if let syn::Expr::Range(..) = args {
                writeln!(
                    buf,
                    "for ({}, {}) in ({}).enumerate() {{",
                    ctx[1],
                    ctx[0],
                    &mem::replace(&mut self.buf_t, String::new())
                )
                .unwrap();
            } else {
                writeln!(
                    buf,
                    "for ({}, {}) in (&{}).into_iter().enumerate() {{",
                    ctx[1],
                    ctx[0],
                    &mem::replace(&mut self.buf_t, String::new())
                )
                .unwrap();
            }
            ctx
        } else {
            let ctx = vec![format!("_key_{}", id)];
            if let syn::Expr::Range(..) = args {
                writeln!(
                    buf,
                    "for {} in {} {{",
                    ctx[0],
                    &mem::replace(&mut self.buf_t, String::new())
                )
                .unwrap();
            } else {
                writeln!(
                    buf,
                    "for {} in (&{}).into_iter() {{",
                    ctx[0],
                    &mem::replace(&mut self.buf_t, String::new())
                )
                .unwrap();
            }
            ctx
        };
        self.on.push(On::Each(id));
        self.scp.push_scope(ctx);

        self.handle(nodes, buf);
        self.handle_ws(ws.1);
        self.write_buf_writable(buf);

        self.on.pop();
        self.scp.pop();
        buf.writeln(&"}");
    }

    fn visit_if(
        &mut self,
        buf: &mut String,
        (pws, cond, block): &'a ((Ws, Ws), syn::Expr, Vec<Node>),
        ifs: &'a [(Ws, syn::Expr, Vec<Node<'a>>)],
        els: &'a Option<(Ws, Vec<Node<'a>>)>,
    ) {
        validator::ifs(cond);

        let mut need_else = false;
        let mut last = false;

        self.handle_ws(pws.0);
        if let Some(val) = self.eval_bool(cond) {
            if val {
                self.scp.push_scope(vec![]);
                self.handle(block, buf);
                self.scp.pop();
            }
            last = val
        } else {
            self.write_buf_writable(buf);
            self.scp.push_scope(vec![]);
            self.visit_expr(cond);
            writeln!(
                buf,
                "if {} {{",
                mem::replace(&mut self.buf_t, String::new())
            )
            .unwrap();

            self.handle(block, buf);
            self.scp.pop();
            need_else = true;
        };

        for (ws, cond, block) in ifs {
            validator::ifs(cond);
            if last {
                break;
            }

            self.handle_ws(*ws);
            last = if let Some(val) = self.eval_bool(cond) {
                if need_else {
                    buf.writeln(&'}');
                }

                if val {
                    self.scp.push_scope(vec![]);
                    self.handle(block, buf);
                    self.scp.pop();
                    need_else = false;
                }
                val
            } else {
                self.write_buf_writable(buf);

                self.scp.push_scope(vec![]);
                self.visit_expr(cond);
                if need_else {
                    writeln!(
                        buf,
                        "}} else if {} {{",
                        mem::replace(&mut self.buf_t, String::new())
                    )
                    .unwrap();
                } else {
                    writeln!(
                        buf,
                        "if {} {{",
                        mem::replace(&mut self.buf_t, String::new())
                    )
                    .unwrap();
                }
                self.handle(block, buf);
                self.scp.pop();
                false
            };
        }

        if let Some((ws, els)) = els {
            self.handle_ws(*ws);
            if need_else {
                self.write_buf_writable(buf);
                buf.writeln(&"} else {");
            }

            if !last {
                self.scp.push_scope(vec![]);
                self.handle(els, buf);
                self.scp.pop();
            }
        }

        self.handle_ws(pws.1);
        if need_else {
            self.write_buf_writable(buf);
            buf.writeln(&"}");
        }
    }

    fn visit_partial(&mut self, buf: &mut String, ws: Ws, path: &str, exprs: &'a [syn::Expr]) {
        let p = self.c.resolve_partial(&self.on_path, path);
        let nodes = self.ctx.get(&p).unwrap();

        let p = mem::replace(&mut self.on_path, p);

        self.flush_ws(ws);

        if exprs.is_empty() {
            self.scp.push_scope(vec![]);
            self.handle(nodes, buf);
            self.scp.pop();
        } else {
            let (no_visited, scope) = visit_partial(&exprs);
            let mut cur = HashMap::new();
            for (k, e) in no_visited {
                self.visit_expr(e);
                cur.insert(
                    k,
                    parse_str::<syn::Expr>(&mem::replace(&mut self.buf_t, String::new())).unwrap(),
                );
            }

            if let Some(scope) = scope {
                self.visit_expr(scope);
                let count = self.scp.count;
                let mut parent = mem::replace(
                    &mut self.scp,
                    Scope::new(mem::replace(&mut self.buf_t, String::new()), count),
                );
                let last = mem::replace(&mut self.partial, Some((cur, 0)));
                let on = mem::replace(&mut self.on, vec![]);

                self.handle(nodes, buf);

                parent.count = self.scp.count;
                self.scp = parent;
                self.partial = last;
                self.on = on;
            } else {
                let last = mem::replace(&mut self.partial, Some((cur, self.on.len())));
                self.scp.push_scope(vec![]);

                self.handle(nodes, buf);

                self.scp.pop();
                self.partial = last;
            }
        }

        self.prepare_ws(ws);
        self.on_path = p;
    }

    fn eval_bool(&self, cond: &'a syn::Expr) -> Option<bool> {
        if let Some(val) = if let Some((p, _)) = &self.partial {
            eval(&ctx_as_ref(p), cond)
        } else {
            eval(&ctx_as_ref(&HashMap::new()), cond)
        } {
            use v_eval::Value::Bool;
            if let Bool(cond) = val {
                return Some(cond);
            }
        }

        None
    }

    fn write_buf_writable(&mut self, buf: &mut String) {
        if self.buf_w.is_empty() {
            return;
        }

        let mut buf_lit = String::new();
        if self.buf_w.iter().all(|w| match w {
            Writable::Lit(..) | Writable::LitP(..) => true,
            _ => false,
        }) {
            for s in mem::replace(&mut self.buf_w, vec![]) {
                match s {
                    Writable::Lit(ref s) => buf_lit.write(s),
                    Writable::LitP(ref s) => buf_lit.write(s),
                    _ => unreachable!(),
                }
            }
            writeln!(buf, "_fmt.write_str({:#?})?;", &buf_lit).unwrap();
            return;
        }

        for s in mem::replace(&mut self.buf_w, vec![]) {
            match s {
                Writable::Lit(ref s) => buf_lit.write(s),
                Writable::LitP(ref s) => buf_lit.write(s),
                Writable::Expr(ref s, wrapped) => {
                    if !buf_lit.is_empty() {
                        writeln!(
                            buf,
                            "_fmt.write_str({:#?})?;",
                            &mem::replace(&mut buf_lit, String::new())
                        )
                        .unwrap();
                    }

                    if wrapped || self.s.wrapped {
                        writeln!(buf, "({}).fmt(_fmt)?;", s).unwrap();
                    } else {
                        // wrap
                        writeln!(buf, "::yarte::Render::render(&({}), _fmt)?;", s).unwrap();
                    }
                }
            }
        }

        if !buf_lit.is_empty() {
            writeln!(buf, "_fmt.write_str({:#?})?;", buf_lit).unwrap();
        }
    }

    /* Helper methods for dealing with whitespace nodes */
    fn skip_ws(&mut self) {
        self.next_ws = None;
        self.skip_ws = true;
    }

    // Based on https://github.com/djc/askama
    // Combines `flush_ws()` and `prepare_ws()` to handle both trailing whitespace from the
    // preceding literal and leading whitespace from the succeeding literal.
    fn handle_ws(&mut self, ws: Ws) {
        self.flush_ws(ws);
        self.prepare_ws(ws);
    }

    // If the previous literal left some trailing whitespace in `next_ws` and the
    // prefix whitespace suppressor from the given argument, flush that whitespace.
    // In either case, `next_ws` is reset to `None` (no trailing whitespace).
    fn flush_ws(&mut self, ws: Ws) {
        if self.next_ws.is_some() && !ws.0 {
            let val = self.next_ws.unwrap();
            if !val.is_empty() {
                self.buf_w.push(Writable::Lit(val));
            }
        }
        self.next_ws = None;
    }

    // Sets `skip_ws` to match the suffix whitespace suppressor from the given
    // argument, to determine whether to suppress leading whitespace from the
    // next literal.
    fn prepare_ws(&mut self, ws: Ws) {
        self.skip_ws = ws.1;
    }
}
