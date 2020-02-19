use std::io::{self, Write};

use yarte_html::{
    interface::QualName,
    serializer::{HtmlSerializer, SerializerOpt},
    tree_builder::is_marquee,
};

use crate::sink::{ParseAttribute, ParseElement, ParseNodeId, Sink};

pub fn serialize<Wr>(writer: Wr, node: Tree, opts: SerializerOpt) -> io::Result<()>
where
    Wr: Write,
{
    let mut ser = HtmlSerializer::new(writer, opts);
    node.serialize(&mut ser)
}

#[derive(Debug)]
pub enum TreeElement {
    Node {
        name: QualName,
        attrs: Vec<ParseAttribute>,
        children: Vec<TreeElement>,
    },
    Text(String),
    DocType,
}

pub struct Tree {
    nodes: Vec<TreeElement>,
}

impl From<Sink> for Tree {
    fn from(mut sink: Sink) -> Tree {
        use ParseElement::*;

        let first = *sink.nodes.keys().next().expect("One node");
        let nodes = match sink.nodes.remove(&first) {
            Some(Document(children)) => {
                let mut tree = vec![TreeElement::DocType];
                tree.extend(get_children(children.into_iter(), &mut sink));
                tree
            }
            Some(Node {
                name,
                attrs,
                children,
                ..
            }) => {
                if is_marquee(&name) {
                    get_children(children.into_iter(), &mut sink)
                } else {
                    vec![TreeElement::Node {
                        name,
                        attrs,
                        children: get_children(children.into_iter(), &mut sink),
                    }]
                }
            }
            Some(Text(s)) => vec![TreeElement::Text(s)],
            None => vec![],
        };

        Tree { nodes }
    }
}

fn get_children<I: Iterator<Item = ParseNodeId>>(children: I, sink: &mut Sink) -> Vec<TreeElement> {
    use ParseElement::*;
    let mut tree = vec![];
    for child in children {
        match sink.nodes.remove(&child).expect("Child") {
            Text(mut s) => {
                if let Some(TreeElement::Text(last)) = tree.last_mut() {
                    last.extend(s.drain(..));
                } else {
                    tree.push(TreeElement::Text(s))
                }
            }
            Node {
                name,
                attrs,
                children,
                ..
            } => tree.push(TreeElement::Node {
                name,
                attrs,
                children: get_children(children.into_iter(), sink),
            }),
            _ => panic!("Expect document in root"),
        }
    }

    tree
}

impl Tree {
    pub fn serialize<W: Write>(self, serializer: &mut HtmlSerializer<W>) -> io::Result<()> {
        _serialize(self.nodes, serializer, None)
    }
}

fn _serialize<W: Write>(
    nodes: Vec<TreeElement>,
    serializer: &mut HtmlSerializer<W>,
    parent: Option<&QualName>,
) -> io::Result<()> {
    use TreeElement::*;
    for node in nodes {
        match node {
            Node {
                children,
                name,
                attrs,
            } => {
                serializer.start_elem(
                    name.clone(),
                    attrs.iter().map(|x| (&x.name, x.value.as_str())),
                )?;
                _serialize(children, serializer, Some(&name))?;
                serializer.end_elem(name)?
            }
            Text(ref s) => serializer.write_text(s)?,
            DocType => serializer.write_doctype("html")?,
        }
    }
    serializer.end(parent)
}
