use std::io::{self, Write};

use markup5ever::QualName;

use crate::{
    serializer::{HtmlSerializer, Position},
    sink::{ParseAttribute, ParseElement, ParseNodeId, Sink, MARK},
    tree_builder::YARTE_TAG,
};

pub fn serialize<Wr>(writer: Wr, node: &Tree) -> io::Result<()>
where
    Wr: Write,
{
    let mut ser = HtmlSerializer::new(writer);
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
        match sink.nodes.get(child).expect("Child") {
            Text(s) => {
                if let Some(TreeElement::Text(last)) = tree.last_mut() {
                    last.push_str(s);
                } else {
                    tree.push(TreeElement::Text(s.clone()))
                }
            }
            Node {
                name,
                attrs,
                children,
                ..
            } => tree.push(TreeElement::Node {
                name: name.clone(),
                attrs: attrs.to_vec(),
                children: get_children(children, sink),
            }),
            Mark(s) => tree.push(TreeElement::Mark(s.clone())),
            _ => panic!("Expect document in root"),
        }
    }

    tree
}

impl Tree {
    pub fn serialize<W: Write>(&self, serializer: &mut HtmlSerializer<W>) -> io::Result<()> {
        _serialize(&self.nodes, serializer)
    }
}

fn _serialize<W: Write>(
    nodes: &[TreeElement],
    serializer: &mut HtmlSerializer<W>,
) -> io::Result<()> {
    use TreeElement::*;
    for (pos, node) in nodes.iter().enumerate().map(|(i, x)| {
        (
            match i {
                0 => Position::Head,
                a if a + 1 == nodes.len() => Position::Tail,
                _ => Position::Middle,
            },
            x,
        )
    }) {
        match node {
            Node {
                children,
                name,
                attrs,
            } => {
                serializer.start_elem(
                    name.clone(),
                    attrs.iter().map(|x| (&x.name, x.value.as_str())),
                    pos,
                )?;
                _serialize(children, serializer)?;
                serializer.end_elem(name.clone(), pos)?
            }
            Text(s) => serializer.write_text(s)?,
            DocType => serializer.write_doctype("html")?,
            Mark(s) => serializer.write_comment(&format!("{}{}", MARK, s))?,
        }
    }
    serializer.end()
}
