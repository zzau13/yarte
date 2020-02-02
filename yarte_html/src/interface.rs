// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains functionality for managing the DOM, including adding/removing nodes.
//!
//! It can be used by a parser to create the DOM graph structure in memory.
use std::{borrow::Cow, fmt, ops::Deref};

use markup5ever::{
    local_name, namespace_url, ns, tendril::StrTendril, LocalName, Namespace, Prefix,
};

pub use self::NodeOrText::{AppendNode, AppendText};

/// Bypass LocalName composing with expressions
#[derive(Eq, PartialOrd, Ord, Clone)]
pub enum YName {
    Local(LocalName),
    Expr(StrTendril),
}

impl PartialEq for YName {
    fn eq(&self, other: &Self) -> bool {
        use YName::*;
        match (self, other) {
            (Local(a), Local(b)) => a == b,
            (Expr(_), Expr(_)) => true,
            _ => false,
        }
    }
}

impl Deref for YName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            YName::Local(l) => &**l,
            YName::Expr(l) => &*l,
        }
    }
}

#[macro_export]
macro_rules! y_name {
    ($local:tt) => {
        YName::Local(local_name!($local))
    };
    (e $local:tt) => {
        YName::Expr(StrTrendril::from($local))
    };
}

impl fmt::Debug for YName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YName::Local(local) => fmt::Display::fmt(local, f),
            YName::Expr(local) => fmt::Display::fmt(local, f),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Attribute {
    /// The name of the attribute (e.g. the `class` in `<div class="test">`)
    pub name: QualName,
    /// The value of the attribute (e.g. the `"test"` in `<div class="test">`)
    pub value: StrTendril,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct QualName {
    pub prefix: Option<Prefix>,
    pub ns: Namespace,
    pub local: YName,
}

impl QualName {
    #[inline]
    pub fn new(prefix: Option<Prefix>, ns: Namespace, local: YName) -> QualName {
        QualName { prefix, ns, local }
    }

    #[inline]
    pub fn expanded(&self) -> ExpandedName {
        ExpandedName {
            ns: &self.ns,
            local: &self.local,
        }
    }
}

#[derive(Copy, Clone, Eq)]
pub struct ExpandedName<'a> {
    pub ns: &'a Namespace,
    pub local: &'a YName,
}

impl<'a, 'b> PartialEq<ExpandedName<'a>> for ExpandedName<'b> {
    fn eq(&self, other: &ExpandedName<'a>) -> bool {
        self.ns == other.ns && self.local == other.local
    }
}

impl<'a> fmt::Debug for ExpandedName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.ns.is_empty() {
            write!(f, "{:?}", self.local)
        } else {
            write!(f, "{{{}}}:{:?}", self.ns, self.local)
        }
    }
}

#[macro_export]
macro_rules! expanded_name {
    ("", $local:tt) => {
        $crate::interface::ExpandedName {
            ns: &ns!(),
            local: &$crate::interface::YName::Local(local_name!($local)),
        }
    };
    ($ns: ident $local:tt) => {
        $crate::interface::ExpandedName {
            ns: &ns!($ns),
            local: &$crate::interface::YName::Local(local_name!($local)),
        }
    };
}

/// Something which can be inserted into the DOM.
///
/// Adjacent sibling text nodes are merged into a single node, so
/// the sink may not want to allocate a `Handle` for each.
pub enum NodeOrText<Handle> {
    AppendNode(Handle),
    AppendText(StrTendril),
}

/// Whether to interrupt further parsing of the current input until
/// the next explicit resumption of the tokenizer, or continue without
/// any interruption.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum NextParserState {
    /// Stop further parsing.
    Suspend,
    /// Continue without interruptions.
    Continue,
}

/// Special properties of an element, useful for tagging elements with this information.
#[derive(Default)]
pub struct ElementFlags {
    /// A document fragment should be created, associated with the element,
    /// and returned in TreeSink::get_template_contents.
    ///
    /// See [template-contents in the whatwg spec][whatwg template-contents].
    ///
    /// [whatwg template-contents]: https://html.spec.whatwg.org/multipage/#template-contents
    pub template: bool,

    /// This boolean should be recorded with the element and returned
    /// in TreeSink::is_mathml_annotation_xml_integration_point
    ///
    /// See [html-integration-point in the whatwg spec][whatwg integration-point].
    ///
    /// [whatwg integration-point]: https://html.spec.whatwg.org/multipage/#html-integration-point
    pub mathml_annotation_xml_integration_point: bool,

    // Prevent construction from outside module
    _private: (),
}

/// A constructor for an element.
///
/// # Examples
///
/// Create an element like `<div class="test-class-name"></div>`:
pub fn create_element<Sink>(sink: &mut Sink, name: QualName, attrs: Vec<Attribute>) -> Sink::Handle
where
    Sink: TreeSink,
{
    let mut flags = ElementFlags::default();
    match name.expanded() {
        expanded_name!(html "template") => flags.template = true,
        expanded_name!(mathml "annotation-xml") => {
            flags.mathml_annotation_xml_integration_point = attrs.iter().any(|attr| {
                attr.name.expanded() == expanded_name!("", "encoding")
                    && (attr.value.eq_ignore_ascii_case("text/html")
                        || attr.value.eq_ignore_ascii_case("application/xhtml+xml"))
            })
        }
        _ => {}
    }
    sink.create_element(name, attrs, flags)
}

/// Methods a parser can use to create the DOM. The DOM provider implements this trait.
///
/// Having this as a trait potentially allows multiple implementations of the DOM to be used with
/// the same parser.
pub trait TreeSink {
    /// `Handle` is a reference to a DOM node.  The tree builder requires
    /// that a `Handle` implements `Clone` to get another reference to
    /// the same node.
    type Handle: Clone;

    /// The overall result of parsing.
    ///
    /// This should default to Self, but default associated types are not stable yet.
    /// [rust-lang/rust#29661](https://github.com/rust-lang/rust/issues/29661)
    type Output;

    /// Consume this sink and return the overall result of parsing.
    ///
    /// TODO:This should default to `fn finish(self) -> Self::Output { self }`,
    /// but default associated types are not stable yet.
    /// [rust-lang/rust#29661](https://github.com/rust-lang/rust/issues/29661)
    fn finish(self) -> Self::Output;

    /// Signal a parse error.
    fn parse_error(&mut self, msg: Cow<'static, str>);

    /// Get a handle to the `Document` node.
    fn get_document(&mut self) -> Self::Handle;

    /// What is the name of this element?
    ///
    /// Should never be called on a non-element node;
    /// feel free to `panic!`.
    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> ExpandedName<'a>;

    /// Create an element.
    ///
    /// When creating a template element (`name.ns.expanded() == expanded_name!(html "template")`),
    /// an associated document fragment called the "template contents" should
    /// also be created. Later calls to self.get_template_contents() with that
    /// given element return it.
    /// See [the template element in the whatwg spec][whatwg template].
    ///
    /// [whatwg template]: https://html.spec.whatwg.org/multipage/#the-template-element
    fn create_element(
        &mut self,
        name: QualName,
        attrs: Vec<Attribute>,
        flags: ElementFlags,
    ) -> Self::Handle;

    /// Append a node as the last child of the given node.  If this would
    /// produce adjacent sibling text nodes, it should concatenate the text
    /// instead.
    ///
    /// The child node will not already have a parent.
    fn append(&mut self, parent: &Self::Handle, child: NodeOrText<Self::Handle>);

    /// Append a `DOCTYPE` element to the `Document` node.
    fn append_doctype_to_document(
        &mut self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    );

    /// Mark a HTML `<script>` as "already started".
    fn mark_script_already_started(&mut self, _node: &Self::Handle) {}

    /// Indicate that a node was popped off the stack of open elements.
    fn pop(&mut self, _node: &Self::Handle) {}

    /// Get a handle to a template's template contents. The tree builder
    /// promises this will never be called with something else than
    /// a template element.
    fn get_template_contents(&mut self, target: &Self::Handle) -> Self::Handle;

    /// Do two handles refer to the same node?
    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool;

    /// Returns true if the adjusted current node is an HTML integration point
    /// and the token is a start tag.
    fn is_mathml_annotation_xml_integration_point(&self, _handle: &Self::Handle) -> bool {
        false
    }

    /// Called whenever the line number changes.
    fn set_current_line(&mut self, _line_number: u64) {}

    /// Indicate that a `script` element is complete.
    fn complete_script(&mut self, _node: &Self::Handle) -> NextParserState {
        NextParserState::Continue
    }
}
