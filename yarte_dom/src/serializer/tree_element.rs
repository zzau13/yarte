use std::io;

use markup5ever::{
    serialize::{Serialize, Serializer, TraversalScope},
    QualName,
};

use crate::parser::{ParseAttribute, MARK};

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

impl Serialize for TreeElement {
    fn serialize<S>(&self, serializer: &mut S, _traversal_scope: TraversalScope) -> io::Result<()>
    where
        S: Serializer,
    {
        use TreeElement::*;
        match self {
            Node {
                children,
                name,
                attrs,
            } => {
                serializer.start_elem(
                    name.clone(),
                    attrs.iter().map(|x| (&x.name, x.value.as_str())),
                )?;
                for child in children {
                    child.serialize(serializer, TraversalScope::IncludeNode)?;
                }
                serializer.end_elem(name.clone())
            }
            Text(s) => serializer.write_text(s),
            DocType => serializer.write_doctype("html"),
            Mark(s) => serializer.write_comment(&format!("{}{}", MARK, s)),
        }
    }
}
