use syn::visit::Visit;

use std::{mem, path::PathBuf};

use yarte_config::Config;

use crate::parser::{Helper, Node};

use super::{visits::is_super, Context};

pub(super) fn find_loop_var(c: &Config, ctx: Context, path: PathBuf, nodes: &[Node]) -> bool {
    FindEach::new(c, ctx, path).find(nodes)
}

// Find {{ index }} {{ index0 }} {{ first }} {{ _index_[0-9] }}
struct FindEach<'a> {
    loop_var: bool,
    c: &'a Config<'a>,
    ctx: Context<'a>,
    on_path: PathBuf,
    on_: usize,
}

impl<'a> FindEach<'a> {
    fn new<'n>(c: &'n Config<'n>, ctx: Context<'n>, on_path: PathBuf) -> FindEach<'n> {
        FindEach {
            c,
            ctx,
            on_path,
            loop_var: false,
            on_: 0,
        }
    }

    pub fn find(&mut self, nodes: &'a [Node]) -> bool {
        for n in nodes {
            match n {
                Node::Local(expr) => self.visit_local(expr),
                Node::Expr(_, expr) | Node::Safe(_, expr) => self.visit_expr(expr),
                Node::Helper(h) => {
                    let h: &Helper = &*h;
                    match h {
                        Helper::If((_, first, block), else_if, els) => {
                            self.visit_expr(first);
                            if self.loop_var {
                                break;
                            }
                            self.find(block);
                            for (_, e, b) in else_if {
                                if self.loop_var {
                                    break;
                                }

                                self.visit_expr(e);
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
                            self.visit_expr(e);
                            if self.loop_var {
                                break;
                            }
                            self.on_ += 1;
                            self.find(b);
                            self.on_ -= 1;
                        }
                        Helper::Unless(_, expr, block) => {
                            self.visit_expr(expr);
                            if self.loop_var {
                                break;
                            }
                            self.find(block);
                        }
                        Helper::Each(_, expr, block) => {
                            self.visit_expr(expr);
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
                Node::Partial(_, path, expr) => {
                    let p = self.c.resolve_partial(&self.on_path, path);
                    let nodes = self.ctx.get(&p).unwrap();
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
