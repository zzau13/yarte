use markup5ever::local_name;
use quote::quote;
use std::{collections::BTreeMap, iter::Filter, mem, slice::Iter};
use syn::{parse2, Expr};
use yarte_dom::dom::{Document, Each, Element, ExprId, Expression, IfBlock, IfElse, Node};

fn find_path(doc: &Document) -> BTreeMap<ExprId, Path> {
    PathBuilder::default().build(doc)
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Step {
    FirstChild,
    NextSibling,
}

#[derive(Clone, Debug)]
enum Parent {
    Head,
    Body,
    Expr(ExprId),
}

#[derive(Clone, Debug)]
enum InsertPoint {
    Append,
    LastBefore(Vec<InsertPath>),
    SetText,
}

#[derive(Clone, Debug)]
enum InsertPath {
    Before,
    Expr(ExprId),
}

#[derive(Clone, Debug)]
struct Path {
    i: InsertPoint,
    p: Parent,
    v: Vec<Step>,
}

#[derive(Default)]
struct PathBuilder {
    steps: Vec<Step>,
    on: Option<Parent>,
    buff: BTreeMap<ExprId, Path>,
}

impl PathBuilder {
    fn build(mut self, doc: &Document) -> BTreeMap<ExprId, Path> {
        assert_eq!(doc.len(), 1);
        match &doc[0] {
            Node::Elem(Element::Node { name, children, .. }) => {
                assert_eq!(local_name!("html"), name.1);
                assert!(children.iter().all(|x| match x {
                    Node::Elem(Element::Node { name, .. }) => match name.1 {
                        local_name!("body") | local_name!("head") => true,
                        _ => false,
                    },
                    Node::Elem(Element::Text(text)) => text.chars().all(|x| x.is_whitespace()),
                    _ => false,
                }));

                let (head, body) = children.iter().fold((None, None), |acc, x| match x {
                    Node::Elem(Element::Node { name, children, .. }) => match name.1 {
                        local_name!("body") => (acc.0, Some(children)),
                        local_name!("head") => (Some(children), acc.1),
                        _ => acc,
                    },
                    _ => acc,
                });
                if let Some(head) = head {
                    self.on = Some(Parent::Head);
                    self.step(head);
                    self.on.take().unwrap();
                }
                if let Some(body) = body {
                    self.on = Some(Parent::Body);
                    self.step(body);
                    self.on.take().unwrap();
                } else {
                    panic!("Need <body> tag")
                }
            }
            _ => panic!("Need html at root"),
        }

        self.buff
    }

    fn step(&mut self, doc: &Document) {
        let mut children = doc.iter().filter(|x| match x {
            Node::Elem(Element::Text(_)) => false,
            _ => true,
        });
        let len = children.clone().fold(0, |acc, _| acc + 1);
        let mut nodes = children.clone().enumerate();
        let mut last = None;
        for (i, node) in nodes {
            match node {
                Node::Elem(Element::Node { .. }) if last.is_none() => {
                    last = Some(Step::FirstChild);
                }
                Node::Elem(Element::Node { .. }) => {
                    last = Some(Step::NextSibling);
                }
                _ => (),
            }

            self.resolve_node(node, last, (i, len), children.clone());
        }

        if last.is_some() {
            let last = self.parent();
            self.steps.drain(last..);
        }
    }

    fn parent(&mut self) -> usize {
        self.steps
            .iter()
            .rposition(|x| match x {
                Step::FirstChild => true,
                _ => false,
            })
            .unwrap_or_default()
    }

    fn do_step(&mut self, body: &Document, id: ExprId) {
        let on = self.on.replace(Parent::Expr(id));
        let steps = mem::take(&mut self.steps);
        self.step(body);
        self.on = on;
        self.steps = steps;
    }

    fn resolve_node<'a, F: Iterator<Item = &'a Node>>(
        &mut self,
        node: &'a Node,
        step: Option<Step>,
        pos: (usize, usize),
        o: F,
    ) {
        match node {
            Node::Elem(Element::Node { children, .. }) => {
                self.steps.push(step.expect("Some step"));
                self.step(children);
            }
            Node::Expr(Expression::Each(id, each)) => {
                let parent = self.parent();
                let Each { body, .. } = &**each;
                self.buff.insert(
                    *id,
                    Path {
                        i: self.insert_point(pos, o),
                        p: self.on.clone().unwrap(),
                        v: self.steps[..parent].to_vec(),
                    },
                );
                self.do_step(body, *id);
            }
            Node::Expr(Expression::Safe(id, _)) | Node::Expr(Expression::Unsafe(id, _)) => {
                self.buff.insert(
                    *id,
                    Path {
                        i: InsertPoint::SetText,
                        p: self.on.clone().unwrap(),
                        v: self.steps.to_vec(),
                    },
                );
            }
            Node::Expr(Expression::IfElse(id, if_else)) => {
                let parent = self.parent();
                let IfElse { ifs, if_else, els } = &**if_else;
                self.buff.insert(
                    *id,
                    Path {
                        i: self.insert_point(pos, o),
                        p: self.on.clone().unwrap(),
                        v: self.steps[..parent].to_vec(),
                    },
                );
                self.if_block(ifs, *id);
                for b in if_else {
                    self.if_block(b, *id);
                }
                if let Some(body) = els {
                    self.do_step(body, *id);
                }
            }
            Node::Elem(Element::Text(_)) => unreachable!(),
            _ => {}
        }
    }

    fn if_block(&mut self, IfBlock { block, .. }: &IfBlock, id: ExprId) {
        self.do_step(block, id);
    }

    fn insert_point<'a, F: Iterator<Item = &'a Node>>(
        &self,
        pos: (usize, usize),
        o: F,
    ) -> InsertPoint {
        if pos.0 + 1 == pos.1 {
            InsertPoint::Append
        } else {
            let mut buff = Vec::with_capacity(pos.1 - 1 - pos.0);
            let o: Vec<&Node> = o.collect();
            for i in o.iter().skip(pos.0 + 1).rev() {
                match i {
                    Node::Elem(Element::Node { .. }) => {
                        buff.push(InsertPath::Before);
                    }
                    Node::Expr(Expression::Each(id, _)) | Node::Expr(Expression::IfElse(id, _)) => {
                        buff.push(InsertPath::Expr(*id));
                    }
                    _ => (),
                }
            }

            InsertPoint::LastBefore(buff)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::BTreeMap;
    use syn::parse_str;
    use yarte_config::{read_config_file, Config};
    use yarte_dom::dom::DOM;
    use yarte_hir::{generate, visit_derive, Context};
    use yarte_parser::{parse, source_map::get_cursor};

    fn test_paths(s: &str) -> BTreeMap<ExprId, Path> {
        let config = read_config_file();
        let config = Config::new(&config);
        let s: syn::DeriveInput = parse_str(s).unwrap();
        let s = visit_derive(&s, &config);

        let mut ctx = BTreeMap::new();
        ctx.insert(&s.path, parse(get_cursor(&s.path, &s.src)));

        let ir = generate(&config, &s, &ctx).unwrap();
        let dom: DOM = ir.into();
        find_path(&dom.doc)
    }
    macro_rules! make_path {
        ($v:ident impl f) => {
            $v.push(Step::FirstChild);
        };
        ($v:ident impl n) => {
            $v.push(Step::NextSibling);
        };
        ($v:ident impl f $($t:tt)*) => {
            $v.push(Step::FirstChild);
            make_path!($v impl $($t)*);
        };
        ($v:ident impl n $($t:tt)*) => {
            $v.push(Step::NextSibling);
            make_path!($v impl $($t)*);
        };
        ($($t:tt)*) => {{
            let mut v = vec![];
            make_path!(v impl $($t)*);
            v
        }};
    }

    #[test]
    fn test_each() {
        let s = r#"
        #[derive(Template)]
        #[template(path = "fortune.hbs")]
        struct FortunesTemplate;
        "#;

        let paths = test_paths(s);
        assert_eq!(paths.len(), 3);
        let each = paths.get(&0).unwrap();
        let id = paths.get(&1).unwrap();
        let msg = paths.get(&2).unwrap();
        assert_eq!(each.v, make_path!(f));
        assert_eq!(id.v, make_path!(f f));
        assert_eq!(msg.v, make_path!(f f n));
    }

    #[test]
    fn test_if() {
        let s = r#"
        #[derive(Template)]
        #[template(path = "article.hbs")]
        struct ArticleTemplate;
        "#;

        let paths = test_paths(s);
        assert_eq!(paths.len(), 1);
        let expr = paths.get(&0).unwrap();
        assert_eq!(expr.v, make_path!(f f n f n));
    }
}
