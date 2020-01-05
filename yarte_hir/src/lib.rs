use std::{collections::BTreeMap, mem, path::PathBuf, str};

use quote::quote;
use syn::{
    export::Span, parse_str, punctuated::Punctuated, visit_mut::VisitMut, PathSegment, Token,
};
use v_eval::{eval, Value};
use v_htmlescape::escape;

use yarte_config::Config;
use yarte_helpers::helpers::ErrorMessage;
use yarte_parser::{Helper, Node, Partial, SExpr, SNode, SVExpr, Ws};

mod scope;
mod validator;
mod visit_derive;
mod visit_each;
mod visit_partial;
mod visits;

/// High level intermediate representation after lowering Ast
#[derive(Debug, Clone)]
pub enum HIR {
    Lit(String),
    Expr(Box<syn::Expr>),
    Safe(Box<syn::Expr>),
    Each(Box<Each>),
    IfElse(Box<IfElse>),
    Local(Box<syn::Local>),
}

#[derive(Debug, Clone)]
pub struct IfElse {
    pub ifs: (syn::Expr, Vec<HIR>),
    pub if_else: Vec<(syn::Expr, Vec<HIR>)>,
    pub els: Option<Vec<HIR>>,
}

#[derive(Debug, Clone)]
pub struct Each {
    pub args: syn::Expr,
    pub body: Vec<HIR>,
    pub expr: syn::Expr,
}

pub use self::visit_derive::Struct;
use self::{scope::Scope, visit_each::find_loop_var, visit_partial::visit_partial};

pub use self::visit_derive::{visit_derive, Mode, Print};

pub fn generate(c: &Config, s: &Struct, ctx: Context) -> Result<Vec<HIR>, Vec<ErrorMessage>> {
    Generator::new(c, s, ctx).build()
}

pub type Context<'a> = &'a BTreeMap<&'a PathBuf, Vec<SNode<'a>>>;

#[derive(Debug, PartialEq)]
enum On {
    Each(usize),
    With(usize),
}

#[derive(Debug)]
enum Writable<'a> {
    Lit(&'a str),
    LitP(String),
    Expr(Box<syn::Expr>, bool),
}

/// lowering from `SNode` to `HIR`
/// TODO: Document
struct Generator<'a> {
    pub(self) c: &'a Config<'a>,
    /// ast of DeriveInput
    pub(self) s: &'a Struct<'a>,
    /// Scope stack
    pub(self) scp: Scope,
    /// On State stack
    pub(self) on: Vec<On>,
    /// On partial scope
    pub(self) partial: Option<(BTreeMap<String, syn::Expr>, usize)>,
    /// buffer for writable
    buf_w: Vec<Writable<'a>>,
    /// Errors buffer
    errors: Vec<ErrorMessage>,
    /// path - nodes
    ctx: Context<'a>,
    /// current file path
    on_path: PathBuf,
    /// whitespace buffer adapted from [`askama`](https://github.com/djc/askama)
    next_ws: Option<&'a str>,
    /// whitespace flag adapted from [`askama`](https://github.com/djc/askama)
    skip_ws: bool,
}

impl<'a> Generator<'a> {
    fn new<'n>(c: &'n Config<'n>, s: &'n Struct<'n>, ctx: Context<'n>) -> Generator<'n> {
        Generator {
            c,
            s,
            ctx,
            buf_w: vec![],
            next_ws: None,
            on: vec![],
            on_path: s.path.clone(),
            partial: None,
            scp: Scope::new(parse_str("self").unwrap(), 0),
            skip_ws: false,
            errors: vec![],
        }
    }

    fn build(mut self) -> Result<Vec<HIR>, Vec<ErrorMessage>> {
        let mut buf = vec![];

        let nodes: &[SNode] = self.ctx.get(&self.on_path).unwrap();

        self.handle(nodes, &mut buf);
        self.write_buf_writable(&mut buf);
        debug_assert_eq!(self.scp.len(), 1);
        debug_assert_eq!(self.scp.root(), &parse_str::<syn::Expr>("self").unwrap());
        debug_assert!(self.on.is_empty());
        debug_assert!(self.buf_w.is_empty());
        debug_assert_eq!(self.on_path, self.s.path);
        debug_assert_eq!(self.next_ws, None);
        // Extreme case
        if buf.is_empty() {
            buf.push(HIR::Lit("".into()));
        }
        assert!((0..buf.len() - 1).all(|i| match (&buf[i], &buf[i + 1]) {
            (HIR::Lit(..), HIR::Lit(..)) => false,
            _ => true,
        }));

        if self.errors.is_empty() {
            Ok(buf)
        } else {
            Err(self.errors)
        }
    }

    fn handle(&mut self, nodes: &'a [SNode], buf: &mut Vec<HIR>) {
        for n in nodes {
            match n.t() {
                Node::Local(expr) => {
                    self.skip_ws();
                    self.write_buf_writable(buf);
                    let mut expr = *expr.t().clone();
                    self.visit_local_mut(&mut expr);
                    buf.push(HIR::Local(Box::new(expr)));
                }
                Node::Safe(ws, sexpr) => {
                    let mut expr = *sexpr.t().clone();

                    self.handle_ws(*ws);
                    self.visit_expr_mut(&mut expr);

                    if self.const_eval(&expr, true).is_none() {
                        validator::expression(sexpr, &mut self.errors);
                        self.buf_w.push(Writable::Expr(Box::new(expr), true));
                    }
                }
                Node::Expr(ws, sexpr) => {
                    let mut expr = *sexpr.t().clone();

                    self.handle_ws(*ws);
                    self.visit_expr_mut(&mut expr);

                    if self.const_eval(&expr, false).is_none() {
                        validator::expression(sexpr, &mut self.errors);
                        self.buf_w.push(Writable::Expr(Box::new(expr), false));
                    }
                }
                Node::Lit(l, lit, r) => self.visit_lit(l, lit.t(), r),
                Node::Helper(h) => self.visit_helper(buf, &h),
                Node::Partial(Partial(ws, path, expr)) => {
                    self.visit_partial(buf, *ws, path.t(), expr)
                }
                // TODO
                Node::Comment(_) => self.skip_ws(),
                Node::Raw(ws, l, v, r) => {
                    self.handle_ws(ws.0);
                    self.visit_lit(l, v.t(), r);
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

    fn visit_helper(&mut self, buf: &mut Vec<HIR>, h: &'a Helper<'a>) {
        use yarte_parser::Helper::*;
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
        buf: &mut Vec<HIR>,
        ws: (Ws, Ws),
        scond: &SExpr,
        nodes: &'a [SNode],
    ) {
        let mut cond = *scond.t().clone();
        self.handle_ws(ws.0);
        self.visit_expr_mut(&mut cond);

        if let Some(val) = self.eval_bool(&cond) {
            if !val {
                self.scp.push_scope(vec![]);
                self.handle(nodes, buf);
                self.scp.pop();
                self.handle_ws(ws.1);
            }
        } else {
            validator::unless(scond, &mut self.errors);

            self.write_buf_writable(buf);
            self.scp.push_scope(vec![]);
            let mut buf_t = vec![];
            self.handle(nodes, &mut buf_t);
            self.scp.pop();

            self.handle_ws(ws.1);
            self.write_buf_writable(&mut buf_t);
            let cond = syn::Expr::Unary(syn::ExprUnary {
                expr: Box::new(syn::Expr::Paren(syn::ExprParen {
                    attrs: vec![],
                    paren_token: syn::token::Paren(Span::call_site()),
                    expr: Box::new(cond),
                })),
                attrs: vec![],
                op: syn::UnOp::Not(Token![!](Span::call_site())),
            });
            buf.push(HIR::IfElse(Box::new(IfElse {
                ifs: (cond, buf_t),
                if_else: vec![],
                els: None,
            })));
        }
    }

    fn visit_with(&mut self, buf: &mut Vec<HIR>, ws: (Ws, Ws), args: &SExpr, nodes: &'a [SNode]) {
        validator::scope(args, &mut self.errors);

        self.handle_ws(ws.0);
        let mut args = *args.t().clone();
        self.visit_expr_mut(&mut args);
        self.on.push(On::With(self.scp.len()));
        self.scp.push_scope(vec![args]);

        self.handle(nodes, buf);

        self.on.pop();
        self.scp.pop();
        self.handle_ws(ws.1);
    }

    fn visit_each(
        &mut self,
        buf: &mut Vec<HIR>,
        ws: (Ws, Ws),
        sargs: &'a SExpr,
        nodes: &'a [SNode<'a>],
    ) {
        let loop_var = find_loop_var(self.c, self.ctx, self.on_path.clone(), nodes);
        let mut args = *sargs.t().clone();
        self.visit_expr_mut(&mut args);

        if let Some(args) = self.eval_iter(&args) {
            self.const_iter(buf, ws, args, nodes, loop_var);
            return;
        }

        validator::each(sargs, &mut self.errors);

        self.handle_ws(ws.0);
        self.write_buf_writable(buf);

        let id = self.scp.len();
        let v = parse_str(&format!("_key_{}", id)).unwrap();
        let (args, expr, ctx) = if loop_var {
            let i = parse_str(&format!("_index_{}", id)).unwrap();
            let args = if let syn::Expr::Range(..) = args {
                syn::parse2::<syn::Expr>(quote!(((#args).enumerate()))).unwrap()
            } else {
                syn::parse2::<syn::Expr>(quote!(((&(#args)).into_iter().enumerate()))).unwrap()
            };
            (
                args,
                syn::parse2::<syn::Expr>(quote!((#i, #v))).unwrap(),
                vec![v, i],
            )
        } else {
            let args = if let syn::Expr::Range(..) = args {
                args
            } else {
                syn::parse2::<syn::Expr>(quote!(((&(#args)).into_iter()))).unwrap()
            };
            (args, syn::parse2::<syn::Expr>(quote!(#v)).unwrap(), vec![v])
        };
        self.on.push(On::Each(id));
        self.scp.push_scope(ctx);

        let mut body = Vec::new();
        self.handle(nodes, &mut body);
        self.handle_ws(ws.1);
        self.write_buf_writable(&mut body);

        self.on.pop();
        self.scp.pop();

        buf.push(HIR::Each(Box::new(Each { args, body, expr })))
    }

    fn visit_if(
        &mut self,
        buf: &mut Vec<HIR>,
        (pws, scond, block): &'a ((Ws, Ws), SExpr, Vec<SNode>),
        ifs: &'a [(Ws, SExpr, Vec<SNode>)],
        els: &'a Option<(Ws, Vec<SNode>)>,
    ) {
        self.scp.push_scope(vec![]);
        let mut cond = *scond.t().clone();
        self.visit_expr_mut(&mut cond);
        self.handle_ws(pws.0);
        let (mut last, mut o_ifs) = if let Some(val) = self.eval_bool(&cond) {
            if val {
                self.handle(block, buf);
            }
            (val, None)
        } else {
            validator::ifs(scond, &mut self.errors);
            self.write_buf_writable(buf);
            let mut body = Vec::new();
            self.handle(block, &mut body);
            (false, Some((cond, body)))
        };
        self.scp.pop();

        let mut if_else = Vec::new();
        let mut o_els = None;
        for (i, (ws, scond, block)) in ifs.iter().enumerate() {
            self.handle_ws(*ws);
            if let Some(body) = o_els.as_mut() {
                self.write_buf_writable(body);
            } else if let Some((_, body)) = if_else.last_mut() {
                self.write_buf_writable(body);
            } else if let Some((_, body)) = o_ifs.as_mut() {
                self.write_buf_writable(body);
            }
            if last {
                break;
            }

            self.scp.push_scope(vec![]);
            let mut cond = *scond.t().clone();
            self.visit_expr_mut(&mut cond);

            if let Some(val) = self.eval_bool(&cond) {
                if val {
                    if o_ifs.is_some() {
                        let mut body = Vec::new();
                        self.handle(block, &mut body);
                        o_els = Some(body);
                    } else {
                        self.handle(block, buf);
                    }
                    last = i + 1 != ifs.len();
                }
            } else {
                validator::ifs(scond, &mut self.errors);

                let mut body = Vec::new();
                self.handle(block, &mut body);
                if o_ifs.is_some() {
                    if_else.push((cond, body));
                } else {
                    o_ifs = Some((cond, body));
                }
            };
            self.scp.pop();
        }

        let mut els = els.as_ref().and_then(|(ws, els)| {
            if last {
                return None;
            }
            self.handle_ws(*ws);
            if let Some(body) = o_els.as_mut() {
                self.write_buf_writable(body);
                return o_els;
            } else if let Some((_, body)) = if_else.last_mut() {
                self.write_buf_writable(body);
            } else if let Some((_, body)) = o_ifs.as_mut() {
                self.write_buf_writable(body);
            }
            if o_ifs.is_some() {
                self.scp.push_scope(vec![]);
                let mut body = Vec::new();
                self.handle(els, &mut body);
                self.scp.pop();
                Some(body)
            } else {
                self.scp.push_scope(vec![]);
                self.handle(els, buf);
                self.scp.pop();
                None
            }
        });

        self.handle_ws(pws.1);
        if let Some(mut ifs) = o_ifs {
            if let Some(body) = els.as_mut() {
                self.write_buf_writable(body);
            } else if let Some((_, body)) = if_else.last_mut() {
                self.write_buf_writable(body);
            } else {
                self.write_buf_writable(&mut ifs.1);
            }
            buf.push(HIR::IfElse(Box::new(IfElse { ifs, if_else, els })))
        }
    }

    fn visit_partial(&mut self, buf: &mut Vec<HIR>, ws: Ws, path: &str, exprs: &'a SVExpr) {
        let p = self.c.resolve_partial(&self.on_path, path);
        let nodes = self.ctx.get(&p).unwrap();

        let p = mem::replace(&mut self.on_path, p);

        self.flush_ws(ws);

        let exprs = exprs.t();
        if exprs.is_empty() {
            self.scp.push_scope(vec![]);
            self.handle(nodes, buf);
            self.scp.pop();
        } else {
            let (no_visited, scope) = visit_partial(&exprs);
            let mut cur = BTreeMap::new();
            for (k, expr) in no_visited {
                let mut expr = expr.clone();
                self.visit_expr_mut(&mut expr);
                cur.insert(k, expr);
            }

            if let Some(scope) = scope {
                let mut scope = scope.clone();
                self.visit_expr_mut(&mut scope);
                let count = self.scp.count;
                let mut parent = mem::replace(&mut self.scp, Scope::new(scope, count));
                let last = mem::replace(&mut self.partial, Some((cur, 0)));
                let on = mem::take(&mut self.on);

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

    fn const_eval(&mut self, expr: &syn::Expr, safe: bool) -> Option<()> {
        macro_rules! push_some {
            ($expr:expr) => {{
                self.buf_w.push(Writable::LitP($expr.to_string()));
                Some(())
            }};
        }

        use Value::*;
        self.eval_expr(expr).and_then(|val| match val {
            Int(a) => push_some!(a),
            Float(a) => push_some!(a),
            Bool(a) => push_some!(a),
            Str(a) if safe || self.s.mode == Mode::Text => push_some!(a),
            Str(a) => push_some!(escape(&a)),
            _ => None,
        })
    }

    fn const_iter(
        &mut self,
        buf: &mut Vec<HIR>,
        ws: (Ws, Ws),
        args: impl IntoIterator<Item = Value>,
        nodes: &'a [SNode<'a>],
        loop_var: bool,
    ) {
        macro_rules! handle {
            ($ctx:expr) => {
                self.prepare_ws(ws.0);
                self.scp.push_scope($ctx);
                self.handle(nodes, buf);
                self.scp.pop();
                self.flush_ws(ws.1);
            };
        }

        let id = self.scp.len();
        self.on.push(On::Each(id));
        self.flush_ws(ws.0);
        if loop_var {
            for (i, v) in args.into_iter().enumerate() {
                handle!(vec![
                    parse_str(&v.to_string()).unwrap(),
                    parse_str(&i.to_string()).unwrap()
                ]);
            }
        } else {
            for v in args.into_iter() {
                handle!(vec![parse_str(&v.to_string()).unwrap()]);
            }
        }

        self.prepare_ws(ws.1);
        self.on.pop();
    }

    fn eval_expr(&self, expr: &syn::Expr) -> Option<Value> {
        eval(&BTreeMap::new(), expr)
    }

    fn eval_bool(&mut self, expr: &syn::Expr) -> Option<bool> {
        self.eval_expr(expr).and_then(|val| match val {
            Value::Bool(cond) => Some(cond),
            _ => None,
        })
    }

    fn eval_iter(&self, expr: &syn::Expr) -> Option<impl IntoIterator<Item = Value>> {
        self.eval_expr(expr).and_then(|val| match val {
            Value::Vec(vector) => Some(vector),
            Value::Range(range) => Some(range.map(Value::Int).collect()),
            Value::Str(s) => Some(s.chars().map(|x| Value::Str(x.to_string())).collect()),
            _ => None,
        })
    }

    fn resolve_path(
        &self,
        syn::ExprPath { attrs, qself, path }: &syn::ExprPath,
    ) -> Result<syn::Expr, ()> {
        if qself.is_some() || !attrs.is_empty() {
            //        panic!("Not available QSelf in a template expression");
            return Err(());
        }

        macro_rules! writes {
        ($($t:tt)*) => {
            return syn::parse2(quote!($($t)*)).map_err(|_| ());
        };
    }

        macro_rules! index_var {
            ($ident:expr, $j:expr) => {{
                let ident = $ident.as_bytes();
                if is_tuple_index(ident) {
                    let field = syn::Index{ index: u32::from_str_radix(str::from_utf8(&ident[1..]).unwrap(), 10).unwrap(), span: Span::call_site() };
                    let ident = &self.scp[$j][0];
                    writes!(#ident.#field)
                }
            }};
        }

        macro_rules! each_var {
            ($ident:expr, $j:expr) => {{
                debug_assert!(self.scp.get($j).is_some(), "{} {:?}", $j, self.scp);
                debug_assert!(!self.scp[$j].is_empty());
                match $ident {
                    "index0" => return Ok(self.scp[$j][1].clone()),
                    "index" => {
                        let ident = &self.scp[$j][1];
                        writes!((#ident + 1))
                    },
                    "first" => {
                        let ident = &self.scp[$j][1];
                        writes!((#ident == 0))
                    },
                    "this" => return Ok(self.scp[$j][0].clone()),
                    ident => {
                        index_var!(ident, $j);
                        let field = syn::Ident::new(ident, Span::call_site());
                        let ident = &self.scp[$j][0];
                        writes!(#ident.#field)
                    },
                }
            }};
        }

        macro_rules! with_var {
            ($ident:expr, $j:expr) => {{
                debug_assert!(self.scp.get($j).is_some());
                debug_assert!(!self.scp[$j].is_empty());
                index_var!($ident, $j);
                let ident = &self.scp[$j][0];
                let field = syn::Ident::new($ident, Span::call_site());
                writes!(#ident.#field)
            }};
        }

        macro_rules! self_var {
            ($ident:ident) => {{
                index_var!($ident, 0);
                let ident = self.scp.root();
                let field = syn::Ident::new($ident, Span::call_site());
                writes!(#ident.#field)
            }};
        }

        macro_rules! partial_var {
            ($ident:ident, $on:expr) => {{
                if let Some((partial, level)) = &self.partial {
                    if *level == $on {
                        if let Some(expr) = partial.get($ident) {
                            return Ok(expr.clone());
                        }
                    }
                }
            }};
        }

        if path.segments.len() == 1 {
            let ident: &str = &path.segments[0].ident.to_string();

            // static or constant
            if ident.chars().all(|x| x.is_ascii_uppercase() || x.eq(&'_')) {
                let ident = &path.segments[0].ident;
                writes!(#ident)
            }

            partial_var!(ident, self.on.len());

            if let Some(ident) = &self.scp.get_by(ident) {
                // in scope
                Ok((*ident).clone())
            } else {
                // out scope
                if ident.eq("self") {
                    return Ok(self.scp.root().clone());
                }

                match self.on.last() {
                    None => self_var!(ident),
                    Some(On::Each(j)) => each_var!(ident, *j),
                    Some(On::With(j)) => with_var!(ident, *j),
                };
            }
        } else if let Some((j, ref ident)) = is_super(&path.segments) {
            if self.on.is_empty() {
                panic!("use super at top");
            } else if self.on.len() == j {
                partial_var!(ident, j);
                self_var!(ident);
            } else if j < self.on.len() {
                partial_var!(ident, j);
                match self.on[self.on.len() - j - 1] {
                    On::Each(j) => each_var!(ident.as_str(), j),
                    On::With(j) => with_var!(ident, j),
                }
            } else {
                panic!("use super without parent")
            }
        } else {
            Ok(syn::Expr::Path(syn::ExprPath {
                attrs: vec![],
                qself: None,
                path: path.clone(),
            }))
        }
    }

    fn write_buf_writable(&mut self, buf: &mut Vec<HIR>) {
        if self.buf_w.is_empty() {
            return;
        }

        let mut buf_lit = String::new();
        for s in mem::take(&mut self.buf_w) {
            match s {
                Writable::Lit(ref s) => buf_lit.push_str(s),
                Writable::LitP(ref s) => buf_lit.push_str(s),
                Writable::Expr(s, wrapped) => {
                    if !buf_lit.is_empty() {
                        buf.push(HIR::Lit(mem::take(&mut buf_lit)));
                    }
                    buf.push(if wrapped { HIR::Safe(s) } else { HIR::Expr(s) })
                }
            }
        }

        if !buf_lit.is_empty() {
            buf.push(HIR::Lit(buf_lit));
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

fn is_super<S>(i: &Punctuated<PathSegment, S>) -> Option<(usize, String)> {
    let idents: Vec<String> = Punctuated::pairs(i)
        .map(|x| x.value().ident.to_string())
        .collect();
    let len = idents.len();
    let ident = idents[len - 1].clone();
    let idents: &[String] = &idents[0..len - 1];

    if idents.iter().all(|x| x.eq("super")) {
        Some((idents.len(), ident))
    } else {
        None
    }
}

#[inline]
fn is_tuple_index(ident: &[u8]) -> bool {
    1 < ident.len() && ident[0] == b'_' && ident[1..].iter().all(|x| x.is_ascii_digit())
}
