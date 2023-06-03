#![allow(clippy::cognitive_complexity)]

use std::{mem, path::PathBuf};

use syn::visit::Visit;

use yarte_helpers::config::Config;
use yarte_parser::{Helper, Node, Partial, PartialBlock, SNode};

use super::{is_super, Context, LoweringContext};
use crate::{
    error::{GError, GResult},
    Struct,
};

pub(super) fn find_loop_var(g: &LoweringContext, nodes: &[SNode]) -> GResult<bool> {
    FindEach::from(g).find(nodes)
}

// Find {{ index }} {{ index0 }} {{ first }}
#[derive(Clone)]
pub struct FindEach<'a> {
    loop_var: bool,
    s: &'a Struct<'a>,
    c: &'a Config,
    ctx: Context<'a>,
    on_path: PathBuf,
    block: Vec<(&'a [SNode<'a>], FindEach<'a>)>,
    on_: usize,
    recursion: usize,
    on_error: Option<GError>,
}

impl<'a> From<&LoweringContext<'a>> for FindEach<'a> {
    fn from(g: &LoweringContext<'a>) -> FindEach<'a> {
        FindEach {
            loop_var: false,
            c: g.c,
            s: g.s,
            ctx: g.ctx,
            on_path: g.on_path.clone(),
            block: g.block.iter().map(|(_, x, g)| (*x, g.into())).collect(),
            on_: 0,
            recursion: g.recursion,
            on_error: None,
        }
    }
}

macro_rules! breaks {
    ($_self:ident) => {
        if $_self.loop_var || $_self.on_error.is_some() {
            break;
        }
    };
}

impl<'a> FindEach<'a> {
    // TODO: #39
    pub fn find(&mut self, nodes: &'a [SNode]) -> GResult<bool> {
        macro_rules! partial {
            ($path:ident, $expr:ident) => {{
                self.recursion += 1;
                if self.s.recursion_limit <= self.recursion {
                    self.on_error.replace(GError::RecursionLimit);
                    break;
                }

                let p = self.c.resolve_partial(&self.on_path, $path.t());
                let nodes = self.ctx.get(&p).unwrap();
                let expr = $expr.t();
                if !expr.is_empty() {
                    let at = if let syn::Expr::Assign(_) = *expr[0] {
                        0
                    } else {
                        1
                    };
                    for e in &expr[at..] {
                        self.visit_expr(e);
                        breaks!(self);
                    }
                    if at == 1 {
                        break;
                    }
                }
                (mem::replace(&mut self.on_path, p), nodes)
            }};
        }

        for n in nodes {
            match n.t() {
                Node::Local(expr) => self.visit_local(expr.t()),
                Node::Expr(_, expr) | Node::Safe(_, expr) => self.visit_expr(expr.t()),
                #[cfg(feature = "wasm-app")]
                Node::RExpr(_, expr) => self.visit_expr(expr.t()),
                Node::Helper(h) => {
                    let h: &Helper = h;
                    match h {
                        Helper::If((_, first, block), else_if, els) => {
                            self.visit_expr(first.t());
                            breaks!(self);
                            self.find(block)?;
                            for (_, e, b) in else_if {
                                breaks!(self);

                                self.visit_expr(e.t());
                                breaks!(self);

                                self.find(b)?;
                            }
                            breaks!(self);

                            if let Some((_, els)) = els {
                                self.find(els)?;
                            }
                        }
                        Helper::With(_, e, b) => {
                            self.visit_expr(e.t());
                            breaks!(self);

                            self.on_ += 1;
                            self.find(b)?;
                            self.on_ -= 1;
                        }
                        Helper::Unless(_, expr, block) => {
                            self.visit_expr(expr.t());
                            breaks!(self);

                            self.find(block)?;
                        }
                        Helper::Each(_, expr, block) => {
                            self.visit_expr(expr.t());
                            breaks!(self);

                            self.on_ += 1;
                            self.find(block)?;
                            self.on_ -= 1;
                        }
                        Helper::Defined(..) => {
                            // TODO: #39
                            self.on_error.replace(GError::Unimplemented);
                        }
                    }
                }
                Node::Partial(Partial(_, path, expr)) => {
                    let (parent, nodes) = partial!(path, expr);

                    self.find(nodes)?;

                    self.on_path = parent;
                    self.recursion -= 1;
                }
                Node::PartialBlock(PartialBlock(_, path, expr, block)) => {
                    let (parent, nodes) = partial!(path, expr);

                    self.block.push((block, self.clone()));
                    self.find(nodes)?;
                    self.on_path = parent;
                    self.block.pop();
                    self.recursion -= 1;
                }
                Node::Block(_) => {
                    if let Some((block, mut old)) = self.block.pop() {
                        old.find(block)?;
                        self.loop_var |= old.loop_var;
                        self.block.push((block, old));
                    } else {
                        // TODO: #39
                        self.on_error.replace(GError::PartialBlockNoParent);
                    }
                }
                Node::Raw(..) | Node::Lit(..) | Node::Comment(_) => (),
                #[allow(unreachable_patterns)]
                _ => (),
            }
            breaks!(self);
        }
        if let Some(err) = self.on_error.take() {
            Err(err)
        } else {
            Ok(self.loop_var)
        }
    }
}

impl<'a> Visit<'a> for FindEach<'a> {
    fn visit_expr_path(&mut self, i: &'a syn::ExprPath) {
        macro_rules! search {
            ($ident:expr) => {
                match $ident.as_ref() {
                    "index" | "index0" | "first" => self.loop_var = true,
                    _ => (),
                }
            };
        }

        if !self.loop_var {
            if i.path.segments.len() == 1 {
                search!(i.path.segments[0].ident.to_string());
            } else if 0 < self.on_ {
                if let Some((j, ident)) = is_super(&i.path.segments) {
                    if j == self.on_ {
                        search!(ident);
                    }
                }
            }
        }
    }
}
