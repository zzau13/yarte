#![allow(unused_variables)]
#![allow(dead_code)]

use std::{
    borrow::Cow,
    collections::BTreeMap,
    fmt::{self, Debug, Formatter},
};

use html5ever::{
    tendril::{StrTendril, TendrilSink},
    tree_builder::{
        Attribute as HtmlAttribute, ElementFlags, NodeOrText as HtmlNodeOrText, QuirksMode,
        TreeBuilderOpts, TreeSink,
    },
    ExpandedName, ParseOpts, QualName,
};

use crate::{driver, serializer::TreeElement, tree_builder::YARTE_TAG};

type ParseNodeId = usize;

#[derive(Clone)]
pub struct ParseNode {
    id: ParseNodeId,
    qual_name: Option<QualName>,
}

impl Debug for ParseNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("ParseNode")
            .field("id", &self.id)
            .field(
                "name",
                &self.qual_name.as_ref().map(|x| x.local.to_string()),
            )
            .finish()
    }
}

#[derive(Debug)]
enum NodeOrText {
    Node(ParseNode),
    Text(String),
}

#[derive(Clone)]
pub struct ParseAttribute {
    pub name: QualName,
    pub value: String,
}

impl Debug for ParseAttribute {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Attr")
            .field("name", &self.name.local.to_string())
            .field("value", &self.value)
            .finish()
    }
}

pub enum ParseElement {
    Mark(String),
    Node {
        name: QualName,
        attrs: Vec<ParseAttribute>,
        children: Vec<ParseNodeId>,
        parent: Option<ParseNodeId>,
    },
    Text(String),
    Document(Vec<ParseNodeId>),
}

impl Debug for ParseElement {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseElement::Node {
                name,
                attrs,
                children,
                parent,
            } => f
                .debug_struct("Node")
                .field("name", &name.local.to_string())
                .field("attributes", attrs)
                .field("children", children)
                .field("parent", parent)
                .finish(),
            ParseElement::Mark(s) => f.debug_tuple("Mark").field(s).finish(),
            ParseElement::Text(s) => f.debug_tuple("Text").field(s).finish(),
            ParseElement::Document(s) => f.debug_tuple("Document").field(s).finish(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Sink {
    count: usize,
    nodes: BTreeMap<ParseNodeId, ParseElement>,
    err: Vec<ParseError>,
}

impl Sink {
    fn new_parse_node(&mut self) -> ParseNode {
        let id = self.count;
        self.count += 1;
        ParseNode {
            id,
            qual_name: None,
        }
    }

    fn append_child(
        &mut self,
        p: ParseNodeId,
        child: HtmlNodeOrText<<Self as TreeSink>::Handle>,
    ) -> ParseNodeId {
        match child {
            HtmlNodeOrText::AppendNode(node) => {
                self.nodes
                    .get_mut(&node.id)
                    .and_then(|x| match x {
                        ParseElement::Node { parent, name, .. } => {
                            if name != &*YARTE_TAG {
                                *parent = Some(p);
                            }
                            Some(())
                        }
                        ParseElement::Mark(_) => Some(()),
                        _ => {
                            panic!("Parent {:?} {:?}", node, x);
                            None
                        }
                    })
                    .expect("Get parent");
                node.id
            }
            HtmlNodeOrText::AppendText(text) => {
                let id = self.count;
                self.count += 1;
                self.nodes.insert(id, ParseElement::Text(text.to_string()));
                id
            }
        }
    }
}

#[derive(Debug)]
pub struct ParseError(Cow<'static, str>);

pub type ParseResult<T> = Result<T, Vec<ParseError>>;

pub const MARK: &str = "yarteHashHTMLExpressionsATTT";
pub const HEAD: &str = "<!--yarteHashHTMLExpressionsATTT";
pub const TAIL: &str = "-->";

impl TreeSink for Sink {
    type Handle = ParseNode;
    type Output = ParseResult<Self>;

    fn finish(self) -> Self::Output {
        if self.err.is_empty() {
            Ok(self)
        } else {
            Err(self.err)
        }
    }

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        self.err.push(ParseError(msg))
    }

    fn get_document(&mut self) -> Self::Handle {
        let new_node = self.new_parse_node();
        self.nodes
            .insert(new_node.id, ParseElement::Document(vec![]));
        new_node
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> ExpandedName<'a> {
        target
            .qual_name
            .as_ref()
            .expect("Expected qual name of node!")
            .expanded()
    }

    fn create_element(
        &mut self,
        name: QualName,
        html_attrs: Vec<HtmlAttribute>,
        flags: ElementFlags,
    ) -> Self::Handle {
        let mut new_node = self.new_parse_node();
        new_node.qual_name = Some(name.clone());
        let attrs = html_attrs
            .into_iter()
            .map(|attr| ParseAttribute {
                name: attr.name,
                value: String::from(attr.value),
            })
            .collect();

        self.nodes.insert(
            new_node.id,
            ParseElement::Node {
                name,
                attrs,
                children: vec![],
                parent: None,
            },
        );

        new_node
    }

    fn create_comment(&mut self, text: StrTendril) -> Self::Handle {
        let node = self.new_parse_node();
        if text.as_bytes().starts_with(MARK.as_bytes()) {
            self.nodes.insert(
                node.id,
                ParseElement::Mark(
                    text.to_string()
                        .get(MARK.len()..)
                        .expect("SOME")
                        .to_string(),
                ),
            );
        }

        node
    }

    #[allow(unused_variables)]
    fn create_pi(&mut self, target: StrTendril, data: StrTendril) -> Self::Handle {
        unreachable!()
    }

    fn append(&mut self, p: &Self::Handle, child: HtmlNodeOrText<Self::Handle>) {
        let id = self.append_child(p.id, child);

        match self.nodes.get_mut(&p.id) {
            Some(ParseElement::Document(children)) | Some(ParseElement::Node { children, .. }) => {
                children.push(id);
            }
            _ => unreachable!(),
        };
    }

    fn append_based_on_parent_node(
        &mut self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: HtmlNodeOrText<Self::Handle>,
    ) {
        let parent = match self.nodes.get(&prev_element.id) {
            Some(ParseElement::Node { parent, .. }) => *parent,
            _ => None,
        }
        .expect("Some parent of sibling");
        let id = self.append_child(parent, child);

        match self.nodes.get_mut(&parent) {
            Some(ParseElement::Document(children)) | Some(ParseElement::Node { children, .. }) => {
                children.push(id);
            }
            _ => unreachable!(),
        };
    }

    fn append_doctype_to_document(&mut self, _: StrTendril, _: StrTendril, _: StrTendril) {
        // DO nothing
    }

    fn get_template_contents(&mut self, target: &Self::Handle) -> Self::Handle {
        target.clone()
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x.id == y.id
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        // DO nothing
    }

    fn append_before_sibling(
        &mut self,
        sibling: &Self::Handle,
        new_node: HtmlNodeOrText<Self::Handle>,
    ) {
        let parent = match self.nodes.get(&sibling.id) {
            Some(ParseElement::Node { parent, .. }) => *parent,
            _ => None,
        }
        .expect("Some parent of sibling");

        let id = self.append_child(parent, new_node);
        match self.nodes.get_mut(&parent) {
            Some(ParseElement::Document(children)) | Some(ParseElement::Node { children, .. }) => {
                if let Some(position) = children.iter().position(|x| x == &sibling.id) {
                    let mut head = children.get(..=position).unwrap().to_vec();
                    head.push(id);
                    head.extend_from_slice(children.get(position + 1..).unwrap());
                    *children = head;
                }
            }
            _ => unreachable!(),
        }
    }

    fn add_attrs_if_missing(&mut self, target: &Self::Handle, html_attrs: Vec<HtmlAttribute>) {
        let node = self.nodes.get_mut(&target.id).unwrap();
        let html_attrs: Vec<ParseAttribute> = html_attrs
            .into_iter()
            .map(|attr| ParseAttribute {
                name: attr.name,
                value: String::from(attr.value),
            })
            .collect();

        // TODO: if missing
        match node {
            ParseElement::Node { attrs, .. } => attrs.extend(html_attrs),
            _ => panic!("add attributes if missing in Text or Document"),
        }
    }

    fn remove_from_parent(&mut self, target: &Self::Handle) {
        todo!()
    }

    fn reparent_children(&mut self, node: &Self::Handle, new_parent: &Self::Handle) {
        todo!()
    }
}

impl Into<Vec<TreeElement>> for Sink {
    fn into(self) -> Vec<TreeElement> {
        use ParseElement::*;

        match self.nodes.values().next() {
            Some(Document(children)) => {
                let mut tree = vec![TreeElement::DocType];
                tree.extend(get_children(children, &self));
                tree
            }
            Some(Node {
                name,
                attrs,
                children,
                ..
            }) => {
                if name == &*YARTE_TAG {
                    get_children(children, &self)
                } else {
                    vec![TreeElement::Node {
                        name: name.clone(),
                        attrs: attrs.to_vec(),
                        children: get_children(children, &self),
                    }]
                }
            }
            Some(Text(s)) => vec![TreeElement::Text(s.clone())],
            Some(Mark(s)) => vec![TreeElement::Mark(s.clone())],
            None => vec![],
        }
    }
}

fn get_children(children: &[ParseNodeId], sink: &Sink) -> Vec<TreeElement> {
    use ParseElement::*;
    let mut tree = vec![];
    for child in children {
        tree.push(match sink.nodes.get(child).expect("Child") {
            Text(s) => TreeElement::Text(s.clone()),
            Node {
                name,
                attrs,
                children,
                ..
            } => TreeElement::Node {
                name: name.clone(),
                attrs: attrs.to_vec(),
                children: get_children(children, sink),
            },
            Mark(s) => TreeElement::Mark(s.clone()),
            _ => panic!("Expect document in root"),
        });
    }

    tree
}

pub fn parse_document(doc: &str) -> ParseResult<Sink> {
    let parser = driver::parse_document(
        Sink::default(),
        ParseOpts {
            tree_builder: TreeBuilderOpts {
                exact_errors: cfg!(debug_assertions),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .from_utf8();

    parser.one(doc.as_bytes())
}

pub fn parse_fragment(doc: &str) -> ParseResult<Sink> {
    let parser = driver::parse_fragment(
        Sink::default(),
        ParseOpts {
            tree_builder: TreeBuilderOpts {
                exact_errors: cfg!(debug_assertions),
                ..Default::default()
            },
            ..Default::default()
        },
        YARTE_TAG.clone(),
        vec![],
    )
    .from_utf8();
    parser.one(doc.as_bytes()).and_then(|mut a| {
        a.nodes
            .remove(&0)
            .and_then(|_| a.nodes.remove(&1))
            .and_then(|_| {
                if let Some(ParseElement::Node { name, .. }) = a.nodes.get_mut(&2) {
                    *name = YARTE_TAG.clone();
                    Some(a)
                } else {
                    None
                }
            })
            .ok_or_else(|| vec![])
    })
}
