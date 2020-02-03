#![allow(clippy::cognitive_complexity)]

use std::{mem, path::PathBuf};

use syn::visit::Visit;

use yarte_config::Config;
use yarte_parser::{Helper, Node, Partial, PartialBlock, SNode};

use super::{is_super, Context, Generator};
use crate::Struct;

pub(super) fn find_loop_var(g: &Generator, nodes: &[SNode]) -> bool {
    FindEach::from(g).find(nodes)
}

// Find {{ index }} {{ index0 }} {{ first }} {{ _index_[0-9] }}
#[derive(Clone)]
pub struct FindEach<'a> {
    loop_var: bool,
    s: &'a Struct<'a>,
    c: &'a Config<'a>,
    ctx: Context<'a>,
    on_path: PathBuf,
    block: Vec<(&'a [SNode<'a>], FindEach<'a>)>,
    on_: usize,
    recursion: usize,
}

impl<'a> From<&Generator<'a>> for FindEach<'a> {
    fn from(g: &Generator<'a>) -> FindEach<'a> {
        FindEach {
            loop_var: false,
            c: g.c,
            s: g.s,
            ctx: g.ctx,
            on_path: g.on_path.clone(),
            block: g.block.iter().map(|(_, x, g)| (*x, g.into())).collect(),
            on_: 0,
            recursion: g.recursion,
        }
    }
}

impl<'a> FindEach<'a> {
    pub fn find(&mut self, nodes: &'a [SNode]) -> bool {
        for n in nodes {
            match n.t() {
                Node::Local(expr) => self.visit_local(expr.t()),
                Node::Expr(_, expr) | Node::Safe(_, expr) | Node::RExpr(_, expr) => {
                    self.visit_expr(expr.t())
                }
                Node::Helper(h) => {
                    let h: &Helper = &*h;
                    match h {
                        Helper::If((_, first, block), else_if, els) => {
                            self.visit_expr(first.t());
                            if self.loop_var {
                                break;
                            }
                            self.find(block);
                            for (_, e, b) in else_if {
                                if self.loop_var {
                                    break;
                                }

                                self.visit_expr(e.t());
                                if self.loop_var {
                                    break;
                                }

                                self.find(b);
                            }
                            if self.loop_var {
                                break;
                            }
                            if let Some((_, els)) = els {
                                self.find(els);
                            }
                        }
                        Helper::With(_, e, b) => {
                            self.visit_expr(e.t());
                            if self.loop_var {
                                break;
                            }
                            self.on_ += 1;
                            self.find(b);
                            self.on_ -= 1;
                        }
                        Helper::Unless(_, expr, block) => {
                            self.visit_expr(expr.t());
                            if self.loop_var {
                                break;
                            }
                            self.find(block);
                        }
                        Helper::Each(_, expr, block) => {
                            self.visit_expr(expr.t());
                            if self.loop_var {
                                break;
                            }
                            self.on_ += 1;
                            self.find(block);
                            self.on_ -= 1;
                        }
                        Helper::Defined(..) => unimplemented!(),
                    }
                }
                Node::Partial(Partial(_, path, expr)) => {
                    self.recursion += 1;
                    if self.s.recursion_limit <= self.recursion {
                        // TODO: to error message
                        panic!("Recursion limit")
                    }

                    let p = self.c.resolve_partial(&self.on_path, path.t());
                    let nodes = self.ctx.get(&p).unwrap();
                    let expr = expr.t();
                    if !expr.is_empty() {
                        let at = if let syn::Expr::Assign(_) = expr[0] {
                            0
                        } else {
                            1
                        };
                        for e in &expr[at..] {
                            self.visit_expr(e);
                            if self.loop_var {
                                break;
                            }
                        }
                        if at == 1 {
                            break;
                        }
                    }

                    let parent = mem::replace(&mut self.on_path, p);

                    self.find(nodes);

                    self.on_path = parent;
                    self.recursion -= 1;
                }
                Node::PartialBlock(PartialBlock(_, path, expr, block)) => {
                    self.recursion += 1;
                    if self.s.recursion_limit <= self.recursion {
                        // TODO: to error message
                        panic!("Recursion limit")
                    }

                    let p = self.c.resolve_partial(&self.on_path, path.t());
                    let nodes = self.ctx.get(&p).unwrap();
                    let expr = expr.t();
                    if !expr.is_empty() {
                        let at = if let syn::Expr::Assign(_) = expr[0] {
                            0
                        } else {
                            1
                        };
                        for e in &expr[at..] {
                            self.visit_expr(e);
                            if self.loop_var {
                                break;
                            }
                        }
                        if at == 1 {
                            break;
                        }
                    }

                    let parent = mem::replace(&mut self.on_path, p);
                    self.block.push((block, self.clone()));
                    self.find(nodes);
                    self.on_path = parent;
                    self.block.pop();
                    self.recursion -= 1;
                }
                Node::Block(_) => {
                    if let Some((block, mut old)) = self.block.pop() {
                        old.find(block);
                        self.loop_var |= old.loop_var;
                        self.block.push((block, old));
                    } else {
                        // TODO: to error message
                        panic!("Use inside partial block");
                    }
                }
                Node::Raw(..) | Node::Lit(..) | Node::Comment(_) => (),
            }
            if self.loop_var {
                break;
            }
        }
        self.loop_var
    }
}

impl<'a> Visit<'a> for FindEach<'a> {
    fn visit_expr_path(&mut self, i: &'a syn::ExprPath) {
        macro_rules! search {
            ($ident:expr) => {
                match $ident.as_ref() {
                    "index" | "index0" | "first" => self.loop_var = true,
                    ident => {
                        let ident = ident.as_bytes();
                        if 7 < ident.len()
                            && &ident[0..7] == b"_index_"
                            && ident[7].is_ascii_digit()
                        {
                            self.loop_var = true;
                        }
                    }
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
