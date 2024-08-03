#![allow(unknown_lints, clippy::type_complexity, clippy::match_on_vec_items)]
use std::{collections::BTreeMap, mem, path::Path, rc::Rc, str};

use quote::{format_ident, quote};
use syn::{
    parse2, parse_str, punctuated::Punctuated, spanned::Spanned, visit_mut::VisitMut, ExprArray,
    ExprBinary, ExprBlock, ExprCall, ExprCast, ExprClosure, ExprField, ExprGroup, ExprIf,
    ExprIndex, ExprLoop, ExprMacro, ExprMatch, ExprMethodCall, ExprParen, ExprPath, ExprRange,
    ExprReference, ExprRepeat, ExprTuple, ExprUnary, ExprUnsafe, PathSegment, Token,
};

use v_eval::{eval, Value};
use v_htmlescape::escape;

use yarte_helpers::config::Config;
use yarte_parser::{
    source_map::Span, AtHelperKind, ErrorMessage, Helper, Node, Parsed, Partial, PartialBlock,
    SExpr, SNode, SVExpr, Ws,
};

#[macro_use]
mod macros;
mod error;
mod hir;
mod imports;
mod scope;
mod serialize;
mod validator;
mod visit_derive;
mod visit_partial;
mod visits;

use self::{
    error::{GError, GResult, MiddleError},
    scope::Scope,
    visit_partial::visit_partial,
};
pub use self::{
    hir::*,
    imports::resolve_imports,
    serialize::{serialize, serialize_resolved},
    visit_derive::{visit_derive, Print, Struct},
};

#[derive(Copy, Clone, Debug)]
pub struct HIROptions {
    pub is_text: bool,
    pub resolve_to_self: bool,
    pub parent: &'static str,
}

impl Default for HIROptions {
    fn default() -> Self {
        Self {
            resolve_to_self: true,
            is_text: false,
            parent: "yarte",
        }
    }
}

pub fn generate(
    c: &Config,
    s: &Struct,
    parsed: Parsed,
    opt: HIROptions,
) -> Result<Vec<HIR>, Vec<ErrorMessage<GError>>> {
    LoweringContext::new(c, s, parsed, opt).build()
}

#[derive(Clone, Debug, PartialEq)]
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
/// TODO: refactor for only left booleans on the stack at recursion
struct LoweringContext<'a> {
    // as state
    pub(self) opt: HIROptions,
    // Copiable
    pub(self) c: &'a Config,
    /// ast of DeriveInput
    // Copiable
    pub(self) s: &'a Struct<'a>,
    /// Scope stack
    // TODO:
    pub(self) scp: Scope,
    /// On State stack
    // TODO:
    pub(self) on: Vec<On>,
    /// On partial scope
    // TODO: remove in favor of anything
    pub(self) partial: Option<(BTreeMap<String, syn::Expr>, usize)>,
    // TODO: remove LoweringContext in favor of reference to state
    block: Vec<(Ws, &'a [SNode<'a>], LoweringContext<'a>)>,
    /// current file path
    on_path: Rc<Path>,
    /// buffer for writable
    // UnAlloc init
    buf_w: Vec<Writable<'a>>,
    // buffer for error builders
    // UnAlloc init
    buf_err: Vec<(GError, proc_macro2::Span)>,
    /// Errors buffer
    // UnAlloc init
    errors: Vec<ErrorMessage<GError>>,
    /// path - nodes
    // Copiable
    ctx: Parsed<'a>,
    /// Last parent conditional
    spans: Vec<Span>,
    /// whitespace buffer adapted from [`askama`](https://github.com/djc/askama)
    // Copiable
    next_ws: Option<&'a str>,
    /// whitespace flag adapted from [`askama`](https://github.com/djc/askama)
    // Copiable
    skip_ws: bool,
    // Copiable
    recursion: usize,
}

// TODO: remove
impl<'a> Clone for LoweringContext<'a> {
    fn clone(&self) -> Self {
        Self {
            opt: self.opt,
            c: self.c,
            s: self.s,
            scp: self.scp.clone(),
            on: self.on.to_vec(),
            spans: self.spans.to_vec(),
            partial: self.partial.clone(),
            block: self.block.clone(),
            buf_w: vec![],
            buf_err: vec![],
            errors: vec![],
            ctx: self.ctx,
            on_path: self.on_path.clone(),
            recursion: self.recursion,
            next_ws: self.next_ws,
            skip_ws: self.skip_ws,
        }
    }
}

impl<'a> LoweringContext<'a> {
    fn new<'n>(
        c: &'n Config,
        s: &'n Struct<'n>,
        ctx: Parsed<'n>,
        opt: HIROptions,
    ) -> LoweringContext<'n> {
        LoweringContext {
            opt,
            c,
            s,
            ctx,
            // TODO:
            block: vec![],
            buf_w: vec![],
            next_ws: None,
            on: vec![],
            on_path: Rc::clone(&s.path),
            partial: None,
            scp: Scope::new(parse_str("self").expect("parse scope self"), 0),
            skip_ws: false,
            errors: vec![],
            recursion: 0,
            buf_err: vec![],
            spans: vec![],
        }
    }

    fn build(mut self) -> Result<Vec<HIR>, Vec<ErrorMessage<GError>>> {
        let mut buf = vec![];

        let nodes: &[SNode] = &self.ctx.get(&self.on_path).expect("No nodes parsed").1;

        self.handle(nodes, &mut buf);
        self.write_buf_writable(&mut buf);
        debug_assert_eq!(self.scp.len(), 1);
        debug_assert_eq!(self.scp.root(), &parse_str::<syn::Expr>("self").unwrap());
        debug_assert!(self.on.is_empty());
        debug_assert!(self.buf_w.is_empty());
        debug_assert_eq!(self.next_ws, None, "whitespace control");
        // Extreme case
        if buf.is_empty() {
            buf.push(HIR::Lit("".into()));
        }
        assert!((0..buf.len() - 1)
            .all(|i| !matches!((&buf[i], &buf[i + 1]), (HIR::Lit(..), HIR::Lit(..)))));

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
                    let mut expr = (***expr.t()).clone();
                    self.visit_local_mut(&mut expr);
                    buf.push(HIR::Local(Box::new(expr)));
                }
                Node::Safe(ws, sexpr) => {
                    let mut expr = (***sexpr.t()).clone();

                    self.handle_ws(*ws);
                    self.visit_expr_mut(&mut expr);
                    self.write_errors(sexpr.span());

                    if self.read_attributes(&mut expr).is_none()
                        && self.const_eval(&expr, true).is_none()
                    {
                        validator::expression(sexpr, &mut self.errors);
                        self.buf_w.push(Writable::Expr(Box::new(expr), true));
                    }
                }
                Node::Expr(ws, sexpr) => {
                    let mut expr = (***sexpr.t()).clone();

                    self.handle_ws(*ws);
                    self.visit_expr_mut(&mut expr);
                    self.write_errors(sexpr.span());

                    if self.const_eval(&expr, false).is_none() {
                        validator::expression(sexpr, &mut self.errors);
                        self.buf_w.push(Writable::Expr(Box::new(expr), false));
                    }
                }
                Node::Lit(l, lit, r) => self.visit_lit(l, lit.t(), r),
                Node::Helper(h) => {
                    self.spans.push(n.span());
                    self.visit_helper(buf, h);
                    self.spans.pop();
                }
                Node::Partial(Partial(ws, path, expr)) => {
                    if let Err(message) = self.visit_partial(buf, *ws, path.t(), expr, None) {
                        self.errors.push(ErrorMessage {
                            message,
                            span: n.span(),
                        });
                        return;
                    }
                }
                // TODO
                Node::Comment(_) => self.skip_ws(),
                Node::Raw(ws, l, v, r) => {
                    self.handle_ws(ws.0);
                    self.visit_lit(l, v.t(), r);
                    self.handle_ws(ws.1);
                }
                Node::Block(ws) => {
                    if let Some((i_ws, block, mut old)) = self.block.pop() {
                        old.next_ws = self.next_ws.take();
                        old.skip_ws = self.skip_ws;
                        old.scp.count = self.scp.count;
                        old.buf_w.append(&mut self.buf_w);

                        old.handle_ws((ws.0, i_ws.0));

                        old.handle(block, buf);

                        self.errors.append(&mut old.errors);
                        self.buf_w.append(&mut old.buf_w);

                        self.scp.count = old.scp.count;
                        self.next_ws = old.next_ws.take();
                        self.skip_ws = old.skip_ws;

                        self.handle_ws((i_ws.1, ws.1));

                        self.block.push((i_ws, block, old));
                    } else {
                        self.flush_ws(*ws);
                        self.errors.push(ErrorMessage {
                            message: GError::PartialBlockNoParent,
                            span: n.span(),
                        });
                        self.prepare_ws(*ws);
                    }
                }
                Node::PartialBlock(PartialBlock(ws, path, expr, block)) => {
                    if let Err(message) =
                        self.visit_partial(buf, ws.0, path.t(), expr, Some((ws.1, block)))
                    {
                        self.errors.push(ErrorMessage {
                            message,
                            span: n.span(),
                        })
                    }
                }
                Node::Error(err) => {
                    self.skip_ws();
                    if let Some(msg) = self.format_error(err) {
                        self.errors.push(ErrorMessage {
                            message: GError::UserCompileError(msg),
                            span: self.spans.last().copied().unwrap_or_else(|| n.span()),
                        })
                    }
                }
                Node::AtHelper(ws, e, args) => {
                    self.handle_ws(*ws);
                    use AtHelperKind::*;
                    match e {
                        Json => {
                            let mut arg = (*args.t()[0]).clone();
                            self.visit_expr_mut(&mut arg);
                            let expr = parse2(quote!((&(#arg).__as_json()))).unwrap();
                            self.buf_w.push(Writable::Expr(Box::new(expr), false))
                        }
                        JsonPretty => {
                            let mut arg = (*args.t()[0]).clone();
                            self.visit_expr_mut(&mut arg);
                            let expr = parse2(quote!(&(#arg).__as_json_pretty())).unwrap();
                            self.buf_w.push(Writable::Expr(Box::new(expr), false))
                        }
                    }
                }
                #[allow(unreachable_patterns)]
                _ => (),
            }
        }
    }

    // TODO:
    fn format_error(&mut self, err: &SVExpr) -> Option<String> {
        if let Some(first) = err.t().first().map(|x| &**x) {
            if let syn::Expr::Lit(e) = first {
                if let syn::Lit::Str(v) = &e.lit {
                    return Some(v.value());
                }
            }

            self.errors
                .push(MiddleError::new(GError::Internal, first.span(), err.span()).into());

            None
        } else {
            Some(String::new())
        }
    }

    fn visit_lit(&mut self, lws: &'a str, lit: &'a str, rws: &'a str) {
        debug_assert!(self.next_ws.is_none(), "{:?} {:?} ", self.next_ws, lit);
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
        self.spans.push(scond.span());
        let mut cond = (***scond.t()).clone();
        self.handle_ws(ws.0);
        self.visit_expr_mut(&mut cond);
        self.write_errors(scond.span());

        if let Some(val) = self.eval_bool(&cond) {
            if !val {
                self.scp.push_scope(vec![]);
                self.handle(nodes, buf);
                self.scp.pop();
            }
            self.handle_ws(ws.1);
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
                    paren_token: syn::token::Paren::default(),
                    expr: Box::new(cond),
                })),
                attrs: vec![],
                op: syn::UnOp::Not(<Token![!]>::default()),
            });
            buf.push(HIR::IfElse(Box::new(IfElse {
                ifs: (cond, buf_t),
                if_else: vec![],
                els: None,
            })));
        }

        self.spans.pop();
    }

    fn visit_with(&mut self, buf: &mut Vec<HIR>, ws: (Ws, Ws), args: &SExpr, nodes: &'a [SNode]) {
        validator::scope(args, &mut self.errors);

        self.handle_ws(ws.0);
        let mut arg = (***args.t()).clone();
        self.visit_expr_mut(&mut arg);
        self.write_errors(args.span());
        self.on.push(On::With(self.scp.len()));
        self.scp.push_scope(vec![arg]);

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
        self.spans.push(sargs.span());
        // TODO
        let loop_var = true;
        let mut args = (***sargs.t()).clone();
        self.visit_expr_mut(&mut args);
        self.write_errors(sargs.span());

        if let Some(args) = self.eval_iter(&args) {
            self.const_iter(buf, ws, args, nodes, loop_var);
            self.spans.pop();
            return;
        }

        validator::each(sargs, &mut self.errors);

        self.handle_ws(ws.0);
        self.write_buf_writable(buf);

        let id = self.scp.len();
        self.scp.push_scope(vec![]);
        let v = self.scp.push_ident("__key_");
        let (args, expr) = if loop_var {
            let i = self.scp.push_ident("__index_");
            let args = if let syn::Expr::Range(..) = args {
                syn::parse2::<syn::Expr>(quote!(((#args).enumerate()))).unwrap()
            } else {
                syn::parse2::<syn::Expr>(quote!(((&(#args)).__into_citer().enumerate()))).unwrap()
            };
            (args, syn::parse2::<syn::Expr>(quote!((#i, #v))).unwrap())
        } else {
            let args = if let syn::Expr::Range(..) = args {
                args
            } else {
                syn::parse2::<syn::Expr>(quote!(((&(#args)).__into_citer()))).unwrap()
            };
            (args, syn::parse2::<syn::Expr>(quote!(#v)).unwrap())
        };
        self.on.push(On::Each(id));

        let mut body = Vec::new();
        self.handle(nodes, &mut body);
        self.handle_ws(ws.1);
        self.write_buf_writable(&mut body);

        self.on.pop();
        self.scp.pop();
        self.spans.pop();

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
        let mut cond: syn::Expr = (***scond.t()).clone();
        self.visit_expr_mut(&mut cond);
        self.write_errors(scond.span());
        self.handle_ws(pws.0);

        self.spans.push(scond.span());
        let (mut last, mut o_ifs, mut is_handled) = if let Some(val) = self.eval_bool(&cond) {
            if val {
                self.handle(block, buf);
            }
            (val, None, val)
        } else {
            validator::ifs(scond, &mut self.errors);
            self.write_buf_writable(buf);
            let mut body = Vec::new();
            self.handle(block, &mut body);
            (false, Some((cond, body)), false)
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
            let mut cond: syn::Expr = (***scond.t()).clone();
            self.visit_expr_mut(&mut cond);
            self.write_errors(scond.span());

            self.spans.push(scond.span());
            if let Some(val) = self.eval_bool(&cond) {
                if val {
                    if o_ifs.is_some() {
                        let mut body = Vec::new();
                        self.handle(block, &mut body);
                        o_els = Some(body);
                    } else {
                        self.handle(block, buf);
                        is_handled = true;
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

        self.spans.pop();
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

            if is_handled {
                return None;
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

    fn visit_partial(
        &mut self,
        buf: &mut Vec<HIR>,
        a_ws: Ws,
        path: &str,
        exprs: &'a SVExpr,
        block: Option<(Ws, &'a [SNode<'a>])>,
    ) -> GResult<()> {
        self.recursion += 1;
        if self.s.recursion_limit < self.recursion {
            return Err(GError::RecursionLimit);
        }

        // TODO: identifiers
        let p = self.c.resolve_partial(&self.on_path, path);
        let nodes = self.ctx.get(&p).expect("partial parsed").1.as_slice();

        // TODO: to on path stack without duplicates
        let p = mem::replace(&mut self.on_path, p);

        let block = if let Some((ws, block)) = block {
            self.flush_ws((a_ws.0, false));
            // TODO: heritage
            self.block.push(((a_ws.1, ws.0), block, self.clone()));
            Some(ws.1)
        } else {
            self.flush_ws(a_ws);
            None
        };
        if exprs.t().is_empty() {
            self.scp.push_scope(vec![]);
            self.handle(nodes, buf);
            self.scp.pop();
        } else {
            let (no_visited, scope) = visit_partial(exprs, &mut self.errors);
            let mut cur = BTreeMap::new();
            for (k, expr) in no_visited {
                let mut expr = expr.clone();
                self.visit_expr_mut(&mut expr);
                self.write_errors(exprs.span());
                cur.insert(k, expr);
            }

            if let Some(scope) = scope {
                let mut scope = scope.clone();
                self.visit_expr_mut(&mut scope);
                self.write_errors(exprs.span());
                let old = mem::replace(&mut self.opt.resolve_to_self, true);
                let count = self.scp.count;
                // TODO: to heap stack without realloc every block
                let mut parent = mem::replace(&mut self.scp, Scope::new(scope, count));
                let last = mem::replace(&mut self.partial, Some((cur, 0)));

                let on = mem::take(&mut self.on);

                self.handle(nodes, buf);

                parent.count = self.scp.count;
                self.scp = parent;
                self.partial = last;
                self.on = on;
                self.opt.resolve_to_self = old;
            } else {
                // TODO:
                let last = mem::replace(&mut self.partial, Some((cur, self.on.len())));
                self.scp.push_scope(vec![]);

                self.handle(nodes, buf);

                self.scp.pop();
                self.partial = last;
            }
        }
        if let Some(ws) = block {
            self.block.pop();
            self.prepare_ws((false, ws));
        } else {
            self.prepare_ws(a_ws)
        }
        // TODO: identifiers
        self.on_path = p;
        self.recursion -= 1;
        Ok(())
    }

    fn const_eval(&mut self, expr: &syn::Expr, safe: bool) -> Option<()> {
        macro_rules! push_some {
            ($expr:expr) => {{
                let expr = $expr.to_string();
                if !expr.is_empty() {
                    self.buf_w.push(Writable::LitP(expr));
                }
                Some(())
            }};
        }

        self.eval_expr(expr).and_then(|val| match val {
            Value::Int(a) => push_some!(a),
            Value::Float(a) => push_some!(a),
            Value::Bool(a) => push_some!(a),
            Value::Str(a) if safe || self.opt.is_text => push_some!(a),
            Value::Str(a) => push_some!(escape(&a)),
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

    #[inline]
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

    fn read_attributes(&mut self, e: &mut syn::Expr) -> Option<()> {
        use syn::Expr::*;
        match e {
            Array(ExprArray { attrs, .. })
            | Binary(ExprBinary { attrs, .. })
            | Block(ExprBlock { attrs, .. })
            | Call(ExprCall { attrs, .. })
            | Cast(ExprCast { attrs, .. })
            | Closure(ExprClosure { attrs, .. })
            | Field(ExprField { attrs, .. })
            | Group(ExprGroup { attrs, .. })
            | If(ExprIf { attrs, .. })
            | Index(ExprIndex { attrs, .. })
            | Loop(ExprLoop { attrs, .. })
            | Macro(ExprMacro { attrs, .. })
            | Match(ExprMatch { attrs, .. })
            | MethodCall(ExprMethodCall { attrs, .. })
            | Paren(ExprParen { attrs, .. })
            | Path(ExprPath { attrs, .. })
            | Range(ExprRange { attrs, .. })
            | Reference(ExprReference { attrs, .. })
            | Repeat(ExprRepeat { attrs, .. })
            | Tuple(ExprTuple { attrs, .. })
            | Unary(ExprUnary { attrs, .. })
            | Unsafe(ExprUnsafe { attrs, .. }) => {
                if attrs.is_empty() {
                    None
                } else {
                    todo!("read expression attributes")
                }
            }
            _ => None,
        }
    }

    fn resolve_path(
        &self,
        syn::ExprPath { attrs, qself, path }: &syn::ExprPath,
    ) -> GResult<syn::Expr> {
        if qself.is_some() || !attrs.is_empty() {
            return Err(GError::NotAvailable);
        }

        macro_rules! writes {
        ($($t:tt)*) => {
            return syn::parse2(quote!($($t)*)).map_err(|_| GError::Internal)
        };
    }

        macro_rules! index_var {
            ($ident:expr, $j:expr) => {{
                let ident = $ident.as_bytes();
                if is_tuple_index(ident) {
                    let field = syn::Index{ index: u32::from_str_radix(str::from_utf8(&ident[1..]).unwrap(), 10).unwrap(), span: proc_macro2::Span::call_site() };
                    let ident = &self.scp[$j][0];
                    writes!(#ident.#field)
                }
            }};
        }

        macro_rules! each_var {
            ($ident:expr, $j:expr) => {{
                debug_assert!(self.scp.get($j).is_some(), "each var {} {:?}", $j, self.scp);
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
                        let field = format_ident!("{}", ident);
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
                let field = format_ident!("{}", $ident);
                writes!(#ident.#field)
            }};
        }

        macro_rules! self_var {
            ($ident:ident) => {{
                index_var!($ident, 0);
                let field = format_ident!("{}", $ident);
                if self.opt.resolve_to_self {
                    let ident = self.scp.root();
                    writes!(#ident.#field)
                } else {
                    writes!(#field)
                }
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

            // static or constant or struct or enum
            if ident
                .chars()
                .next()
                .map(|x| x.is_uppercase())
                .unwrap_or(false)
            {
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
                Err(GError::SuperWithoutParent)
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
                Err(GError::SuperWithoutParent)
            }
        } else {
            Ok(syn::Expr::Path(syn::ExprPath {
                attrs: vec![],
                qself: None,
                path: path.clone(),
            }))
        }
    }

    fn write_errors(&mut self, span: Span) {
        for (message, range) in mem::take(&mut self.buf_err) {
            self.errors
                .push(MiddleError::new(message, range, span).into())
        }
    }

    fn write_buf_writable(&mut self, buf: &mut Vec<HIR>) {
        if self.buf_w.is_empty() {
            return;
        }

        let mut buf_lit = String::new();
        for s in mem::take(&mut self.buf_w) {
            match s {
                Writable::Lit(s) => buf_lit.push_str(s),
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
