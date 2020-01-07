use std::{
    default::Default,
    io::{self, Write},
};

use markup5ever::{
    serialize::{Serialize, Serializer, TraversalScope},
    QualName,
};

use crate::{
    serializer,
    sink::{ParseAttribute, ParseElement, ParseNodeId, Sink, MARK},
    tree_builder::YARTE_TAG,
};

pub fn serialize<Wr>(writer: Wr, node: &Tree) -> io::Result<()>
where
    Wr: Write,
{
    serializer::serialize(writer, node, Default::default())
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
    Mark(String),
}

pub struct Tree {
    nodes: Vec<TreeElement>,
}

impl From<Sink> for Tree {
    fn from(sink: Sink) -> Tree {
        use ParseElement::*;

        let nodes = match sink.nodes.values().next() {
            Some(Document(children)) => {
                let mut tree = vec![TreeElement::DocType];
                tree.extend(get_children(children, &sink));
                tree
            }
            Some(Node {
                name,
                attrs,
                children,
                ..
            }) => {
                if name == &*YARTE_TAG {
                    get_children(children, &sink)
                } else {
                    vec![TreeElement::Node {
                        name: name.clone(),
                        attrs: attrs.to_vec(),
                        children: get_children(children, &sink),
                    }]
                }
            }
            Some(Text(s)) => vec![TreeElement::Text(s.clone())],
            Some(Mark(s)) => vec![TreeElement::Mark(s.clone())],
            None => vec![],
        };

        Tree { nodes }
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

impl Serialize for Tree {
    fn serialize<S>(&self, serializer: &mut S, _traversal_scope: TraversalScope) -> io::Result<()>
    where
        S: Serializer,
    {
        _serialize(&self.nodes, serializer)
    }
}

fn _serialize<S: Serializer>(nodes: &[TreeElement], serializer: &mut S) -> io::Result<()> {
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
                _serialize(children, serializer)?;
                serializer.end_elem(name.clone())?
            }
            Text(s) => serializer.write_text(s)?,
            DocType => serializer.write_doctype("html")?,
            Mark(s) => serializer.write_comment(&format!("{}{}", MARK, s))?,
        }
    }
    Ok(())
}
