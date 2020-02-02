use std::{
    borrow::Cow,
    collections::BTreeMap,
    fmt::{self, Debug, Formatter},
};

use markup5ever::tendril::{StrTendril, TendrilSink};

use yarte_html::{
    driver,
    interface::{
        Attribute as HtmlAttribute, ElementFlags, ExpandedName, NodeOrText as HtmlNodeOrText,
        QualName, TreeSink,
    },
    tree_builder::{get_marquee, is_marquee},
};

pub type ParseNodeId = usize;

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
                &self.qual_name.as_ref().map(|x| (*x.local).to_string()),
            )
            .finish()
    }
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
            ParseElement::Text(s) => f.debug_tuple("Text").field(s).finish(),
            ParseElement::Document(s) => f.debug_tuple("Document").field(s).finish(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Sink {
    count: usize,
    pub nodes: BTreeMap<ParseNodeId, ParseElement>,
    fragment: bool,
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
                            if is_marquee(name) {
                                *parent = Some(p);
                            }
                            Some(())
                        }
                        _ => None,
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
        let node = self.new_parse_node();
        self.fragment = node.id != 0;
        node
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
        _flags: ElementFlags,
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

    fn append(&mut self, p: &Self::Handle, child: HtmlNodeOrText<Self::Handle>) {
        let id = self.append_child(p.id, child);

        match self.nodes.get_mut(&p.id) {
            Some(ParseElement::Document(children)) | Some(ParseElement::Node { children, .. }) => {
                children.push(id);
            }
            _ if p.id == 0 || self.fragment => (),
            _ => panic!("append without parent {:?}, {:?} {:?}", p, id, self.nodes),
        };
    }

    fn append_doctype_to_document(&mut self, _: StrTendril, _: StrTendril, _: StrTendril) {
        if self
            .nodes
            .insert(0, ParseElement::Document(vec![]))
            .is_some()
        {
            panic!("Double Doctype")
        }
    }

    fn get_template_contents(&mut self, target: &Self::Handle) -> Self::Handle {
        target.clone()
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x.id == y.id
    }
}

pub fn parse_document(doc: &str) -> ParseResult<Sink> {
    let parser = driver::parse_document(Sink::default()).from_utf8();

    parser.one(doc.as_bytes())
}

pub fn parse_fragment(doc: &str) -> ParseResult<Sink> {
    let parser = driver::parse_fragment(Sink::default(), get_marquee(), vec![]).from_utf8();
    parser.one(doc.as_bytes()).and_then(|mut a| {
        a.nodes
            .remove(&0)
            .and_then(|_| {
                if let Some(ParseElement::Node { name, .. }) = a.nodes.get_mut(&2) {
                    *name = get_marquee();
                    Some(a)
                } else {
                    None
                }
            })
            .ok_or_else(|| vec![])
    })
}
