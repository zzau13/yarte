// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The HTML5 tree builder.
//!
// TODO: remove https://html.spec.whatwg.org/#optional-tags
// TODO: Whitespace control
// TODO: Never two text nodes one behind the other
// TODO: svg
// TODO: math
// TODO: coverage
// TODO: Use the spec html5 as possible
// TODO: Coverage with html5lib-test

#![allow(clippy::unnested_or_patterns)]
#![allow(
    clippy::cognitive_complexity,
    clippy::redundant_static_lifetimes,
    clippy::suspicious_else_formatting,
    clippy::unused_unit,
    clippy::wrong_self_convention,
)]

use std::{
    borrow::Cow::{Borrowed, Owned},
    collections::VecDeque,
    fmt,
    iter::{Enumerate, Rev},
    slice,
};

use log::{debug, log_enabled, Level};
use mac::{_tt_as_expr_hack, matches};

use markup5ever::{tendril::StrTendril, Namespace};

use crate::{
    interface::{
        create_element, AppendNode, AppendText, Attribute, ExpandedName, NodeOrText, QualName,
        TreeSink, YName,
    },
    tokenizer::{
        self, states::RawKind, Doctype, EndTag, StartTag, Tag, TokenSink, TokenSinkResult,
    },
    utils::{is_ascii_whitespace, to_escaped_string},
};

pub use self::PushFlag::*;
use self::{tag_sets::*, types::*};

#[macro_use]
mod tag_sets;

mod data;
mod types;

include!(concat!(env!("OUT_DIR"), "/rules.rs"));

/// The HTML tree builder.
pub struct TreeBuilder<Handle, Sink> {
    /// Consumer of tree modifications.
    pub sink: Sink,

    /// Insertion mode.
    mode: InsertionMode,

    /// Original insertion mode, used by Text and InTableText modes.
    orig_mode: Option<InsertionMode>,

    /// The document node, which is created by the sink.
    doc_handle: Handle,

    /// Stack of open elements, most recently added at end.
    open_elems: Vec<Handle>,

    /// List of active formatting elements.
    active_formatting: Vec<FormatEntry<Handle>>,

    //§ END
    /// The context element for the fragment parsing algorithm.
    context_elem: Option<Handle>,
}

thread_local! {
    static YARTE_TAG: QualName = QualName::new(None, ns!(html), y_name!("marquee"));
}

pub fn is_marquee(name: &QualName) -> bool {
    YARTE_TAG.with(|x| *x.local == *name.local)
}

pub fn get_marquee() -> QualName {
    YARTE_TAG.with(|x| x.clone())
}

impl<Handle, Sink> TreeBuilder<Handle, Sink>
where
    Handle: Clone,
    Sink: TreeSink<Handle = Handle>,
{
    /// Create a new tree builder which sends tree modifications to a particular `TreeSink`.
    ///
    /// The tree builder is also a `TokenSink`.
    pub fn new(mut sink: Sink) -> TreeBuilder<Handle, Sink> {
        let doc_handle = sink.get_document();
        TreeBuilder {
            sink,
            mode: Initial,
            orig_mode: None,
            doc_handle,
            open_elems: vec![],
            active_formatting: vec![],
            context_elem: None,
        }
    }

    /// Create a new tree builder which sends tree modifications to a particular `TreeSink`.
    /// This is for parsing fragments.
    ///
    /// The tree builder is also a `TokenSink`.
    pub fn new_for_fragment(mut sink: Sink, context_elem: Handle) -> TreeBuilder<Handle, Sink> {
        let doc_handle = sink.get_document();
        let mut tb = TreeBuilder {
            sink,
            mode: InHtml,
            orig_mode: None,
            doc_handle,
            open_elems: vec![],
            active_formatting: vec![],
            context_elem: Some(context_elem),
        };

        tb.create_root(vec![]);

        tb
    }

    fn debug_step(&self, mode: InsertionMode, token: &Token) {
        if log_enabled!(Level::Debug) {
            debug!(
                "processing {} in insertion mode {:?}",
                to_escaped_string(token),
                mode
            );
        }
    }

    fn process_to_completion(&mut self, mut token: Token) -> TokenSinkResult<Handle> {
        // Queue of additional tokens yet to be processed.
        // This stays empty in the common case where we don't split whitespace.
        let mut more_tokens = VecDeque::new();

        loop {
            let should_have_acknowledged_self_closing_flag = matches!(
                token,
                TagToken(Tag {
                    self_closing: true,
                    kind: StartTag,
                    ..
                })
            );
            let result = if self.is_foreign(&token) {
                self.step_foreign(token.clone())
            } else {
                let mode = self.mode;

                self.step(mode, token.clone())
            };
            match result {
                Done => {
                    if should_have_acknowledged_self_closing_flag {
                        self.sink
                            .parse_error(Borrowed("Unacknowledged self-closing tag"));
                    }
                    token = unwrap_or_return!(
                        more_tokens.pop_front(),
                        tokenizer::TokenSinkResult::Continue
                    );
                }
                DoneAckSelfClosing => {
                    token = unwrap_or_return!(
                        more_tokens.pop_front(),
                        tokenizer::TokenSinkResult::Continue
                    );
                }
                Reprocess(m, t) => {
                    self.mode = m;
                    token = t;
                }
                ReprocessForeign(t) => {
                    token = t;
                }
                SplitWhitespace(mut buf) => {
                    let p = buf.pop_front_char_run(is_ascii_whitespace);
                    let (first, is_ws) = unwrap_or_return!(p, tokenizer::TokenSinkResult::Continue);
                    let status = if is_ws { Whitespace } else { NotWhitespace };
                    token = CharacterTokens(status, first);

                    if buf.len32() > 0 {
                        more_tokens.push_back(CharacterTokens(NotSplit, buf));
                    }
                }
                Script(node) => {
                    assert!(more_tokens.is_empty());
                    return tokenizer::TokenSinkResult::Script(node);
                }
                ToPlaintext => {
                    assert!(more_tokens.is_empty());
                    return tokenizer::TokenSinkResult::Plaintext;
                }
                ToRawData(k) => {
                    assert!(more_tokens.is_empty());
                    return tokenizer::TokenSinkResult::RawData(k);
                }
            }
        }
    }

    /// Are we parsing a HTML fragment?
    pub fn is_fragment(&self) -> bool {
        self.context_elem.is_some()
    }

    fn appropriate_place_for_insertion(&mut self, override_target: Option<Handle>) -> Handle {
        override_target.unwrap_or_else(|| self.current_node().clone())
    }

    fn insert_at(&mut self, insertion_point: Handle, child: NodeOrText<Handle>) {
        self.sink.append(&insertion_point, child)
    }
}

impl<Handle, Sink> TokenSink for TreeBuilder<Handle, Sink>
where
    Handle: Clone,
    Sink: TreeSink<Handle = Handle>,
{
    type Handle = Handle;

    fn process_token(
        &mut self,
        token: tokenizer::Token,
        _line_number: u64,
    ) -> TokenSinkResult<Handle> {
        // Handle `ParseError` and `DoctypeToken`; convert everything else to the local `Token` type.
        let token = match token {
            tokenizer::ParseError(e) => {
                self.sink.parse_error(e);
                return tokenizer::TokenSinkResult::Continue;
            }

            tokenizer::DoctypeToken(dt) => {
                if self.mode == Initial {
                    if data::doctype_error(&dt) {
                        self.sink
                            .parse_error(Owned(format!("Bad DOCTYPE: {:?}", dt)));
                    }
                    let Doctype {
                        name,
                        public_id,
                        system_id,
                        ..
                    } = dt;
                    self.sink.append_doctype_to_document(
                        name.unwrap_or_default(),
                        public_id.unwrap_or_default(),
                        system_id.unwrap_or_default(),
                    );

                    self.mode = BeforeHtml;
                    return tokenizer::TokenSinkResult::Continue;
                } else {
                    self.sink
                        .parse_error(Owned(format!("DOCTYPE in insertion mode {:?}", self.mode)));
                    return tokenizer::TokenSinkResult::Continue;
                }
            }

            tokenizer::TagToken(x) => TagToken(x),
            tokenizer::CommentToken(x) => CommentToken(x),
            tokenizer::NullCharacterToken => NullCharacterToken,
            tokenizer::EOFToken => EOFToken,

            tokenizer::CharacterTokens(x) => {
                if x.is_empty() {
                    return tokenizer::TokenSinkResult::Continue;
                }
                CharacterTokens(NotSplit, x)
            }
        };

        self.process_to_completion(token)
    }

    fn end(&mut self) {
        for elem in self.open_elems.drain(..).rev() {
            self.sink.pop(&elem);
        }
    }

    fn adjusted_current_node_present_but_not_in_html_namespace(&self) -> bool {
        !self.open_elems.is_empty()
            && self.sink.elem_name(self.adjusted_current_node()).ns != &ns!(html)
    }
}

pub struct ActiveFormattingIter<'a, Handle: 'a> {
    iter: Rev<Enumerate<slice::Iter<'a, FormatEntry<Handle>>>>,
}

impl<'a, Handle> Iterator for ActiveFormattingIter<'a, Handle> {
    type Item = (usize, &'a Handle, &'a Tag);
    fn next(&mut self) -> Option<(usize, &'a Handle, &'a Tag)> {
        match self.iter.next() {
            None => None,
            Some((i, &Element(ref h, ref t))) => Some((i, h, t)),
        }
    }
}

pub enum PushFlag {
    Push,
    NoPush,
}

macro_rules! qualname {
    ("", $local:tt) => {
        QualName {
            prefix: None,
            ns: ns!(),
            local: y_name!($local),
        }
    };
    ($prefix:tt $ns:tt $local:tt) => {
        QualName {
            prefix: Some(namespace_prefix!($prefix)),
            ns: ns!($ns),
            local: y_name!($local),
        }
    };
}

// TODO: Simplify
#[doc(hidden)]
impl<Handle, Sink> TreeBuilder<Handle, Sink>
where
    Handle: Clone,
    Sink: TreeSink<Handle = Handle>,
{
    fn unexpected<T: fmt::Debug>(&mut self, _thing: &T) -> ProcessResult<Handle> {
        self.sink.parse_error(Owned(format!(
            "Unexpected token {} in insertion mode {:?}",
            to_escaped_string(_thing),
            self.mode
        )));
        Done
    }

    /// Iterate over the active formatting elements (with index in the list) from the end
    /// to the last marker, or the beginning if there are no markers.
    fn active_formatting_end_to_marker(&self) -> ActiveFormattingIter<Handle> {
        ActiveFormattingIter {
            iter: self.active_formatting.iter().enumerate().rev(),
        }
    }

    fn position_in_active_formatting(&self, element: &Handle) -> Option<usize> {
        // TODO
        self.active_formatting.iter().position(|n| match n {
            Element(ref handle, _) => self.sink.same_node(handle, element),
        })
    }

    fn stop_parsing(&mut self) -> ProcessResult<Handle> {
        Done
    }

    //§ parsing-elements-that-contain-only-text
    // Switch to `Text` insertion mode, save the old mode, and
    // switch the tokenizer to a raw-data state.
    // The latter only takes effect after the current / next
    // `process_token` of a start tag returns!
    fn to_raw_text_mode(&mut self, k: RawKind) -> ProcessResult<Handle> {
        self.orig_mode = Some(self.mode);
        self.mode = RawText;
        ToRawData(k)
    }

    // The generic raw text / RCDATA parsing algorithm.
    fn parse_raw_data(&mut self, tag: Tag, k: RawKind) -> ProcessResult<Handle> {
        self.insert_element_for(tag);
        self.to_raw_text_mode(k)
    }
    //§ END

    fn current_node(&self) -> &Handle {
        self.open_elems.last().expect("no current element")
    }

    fn adjusted_current_node(&self) -> &Handle {
        if self.open_elems.len() == 1 {
            if let Some(ctx) = self.context_elem.as_ref() {
                return ctx;
            }
        }
        self.current_node()
    }

    fn current_node_in<TagSet>(&self, set: TagSet) -> bool
    where
        TagSet: Fn(ExpandedName) -> bool,
    {
        set(self.sink.elem_name(self.current_node()))
    }

    // Insert at the "appropriate place for inserting a node".
    fn insert_appropriately(&mut self, child: NodeOrText<Handle>, override_target: Option<Handle>) {
        let insertion_point = self.appropriate_place_for_insertion(override_target);
        self.insert_at(insertion_point, child);
    }

    fn adoption_agency(&mut self, subject: YName) {
        // TODO: simplify
        if self.current_node_named(subject.clone())
            && self
                .position_in_active_formatting(self.current_node())
                .is_none()
        {
            self.pop();
            return;
        }

        // 5.
        let (fmt_elem_index, fmt_elem, _) = unwrap_or_return!(
            // We clone the Handle and Tag so they don't cause an immutable borrow of self.
            self.active_formatting_end_to_marker()
                .find(|&(_, _, tag)| tag.name == subject)
                .map(|(i, h, t)| (i, h.clone(), t.clone())),
            {
                self.process_end_tag_in_body(Tag {
                    kind: EndTag,
                    name: subject,
                    self_closing: false,
                    attrs: vec![],
                });
            }
        );

        let fmt_elem_stack_index = unwrap_or_return!(
            self.open_elems
                .iter()
                .rposition(|n| self.sink.same_node(n, &fmt_elem)),
            {
                self.sink
                    .parse_error(Borrowed("Formatting element not open"));
                self.active_formatting.remove(fmt_elem_index);
            }
        );

        // 7.
        if !self.in_scope(default_scope, |n| self.sink.same_node(&n, &fmt_elem)) {
            self.sink
                .parse_error(Borrowed("Formatting element not in scope"));
            return;
        }

        // 8.
        if !self.sink.same_node(self.current_node(), &fmt_elem) {
            self.sink
                .parse_error(Borrowed("Formatting element not current node"));
        }

        // 9.
        self.open_elems.truncate(fmt_elem_stack_index);
        self.active_formatting.remove(fmt_elem_index);
    }

    fn push(&mut self, elem: &Handle) {
        self.open_elems.push(elem.clone());
    }

    fn pop(&mut self) -> Handle {
        let elem = self.open_elems.pop().expect("no current element");
        self.sink.pop(&elem);
        elem
    }

    fn is_marker_or_open(&self, entry: &FormatEntry<Handle>) -> bool {
        // TODO
        match *entry {
            Element(ref node, _) => self
                .open_elems
                .iter()
                .rev()
                .any(|n| self.sink.same_node(&n, &node)),
        }
    }

    /// Reconstruct the active formatting elements.
    fn reconstruct_formatting(&mut self) {
        {
            let last = unwrap_or_return!(self.active_formatting.last(), ());
            if self.is_marker_or_open(last) {
                return;
            }
        }

        let mut entry_index = self.active_formatting.len() - 1;
        loop {
            if entry_index == 0 {
                break;
            }
            entry_index -= 1;
            if self.is_marker_or_open(&self.active_formatting[entry_index]) {
                entry_index += 1;
                break;
            }
        }

        loop {
            // TODO
            let tag = match self.active_formatting[entry_index] {
                Element(_, ref t) => t.clone(),
            };

            // FIXME: Is there a way to avoid cloning the attributes twice here (once on their own,
            // once as part of t.clone() above)?
            let new_element =
                self.insert_element(Push, ns!(html), tag.name.clone(), tag.attrs.clone());
            self.active_formatting[entry_index] = Element(new_element, tag);
            if entry_index == self.active_formatting.len() - 1 {
                break;
            }
            entry_index += 1;
        }
    }

    /// Signal an error depending on the state of the stack of open elements at
    /// the end of the body.
    fn check_body_end(&mut self) {
        declare_tag_set!(body_end_ok =
            "dd" "dt" "li" "optgroup" "option" "p" "rp" "rt" "tbody" "td" "tfoot" "th"
            "thead" "tr" "body" "html");

        for elem in self.open_elems.iter() {
            let error;
            {
                let name = self.sink.elem_name(elem);
                if body_end_ok(name) {
                    continue;
                }
                error = Owned(format!("Unexpected open tag {:?} at end of body", name));
            }
            self.sink.parse_error(error);
            // FIXME: Do we keep checking after finding one bad tag?
            // The spec suggests not.
            return;
        }
    }

    fn in_scope<TagSet, Pred>(&self, scope: TagSet, pred: Pred) -> bool
    where
        TagSet: Fn(ExpandedName) -> bool,
        Pred: Fn(Handle) -> bool,
    {
        for node in self.open_elems.iter().rev() {
            if pred(node.clone()) {
                return true;
            }
            if scope(self.sink.elem_name(node)) {
                return false;
            }
        }

        // supposed to be impossible, because <html> is always in scope

        false
    }

    fn elem_in<TagSet>(&self, elem: &Handle, set: TagSet) -> bool
    where
        TagSet: Fn(ExpandedName) -> bool,
    {
        set(self.sink.elem_name(elem))
    }

    fn html_elem_named(&self, elem: &Handle, name: YName) -> bool {
        let expanded = self.sink.elem_name(elem);
        *expanded.ns == ns!(html) && *expanded.local == name
    }

    fn current_node_named(&self, name: YName) -> bool {
        self.html_elem_named(self.current_node(), name)
    }

    fn in_scope_named<TagSet>(&self, scope: TagSet, name: YName) -> bool
    where
        TagSet: Fn(ExpandedName) -> bool,
    {
        self.in_scope(scope, |elem| self.html_elem_named(&elem, name.clone()))
    }

    //§ closing-elements-that-have-implied-end-tags
    fn generate_implied_end<TagSet>(&mut self, set: TagSet)
    where
        TagSet: Fn(ExpandedName) -> bool,
    {
        loop {
            {
                let elem = unwrap_or_return!(self.open_elems.last(), ());
                let nsname = self.sink.elem_name(elem);
                if !set(nsname) {
                    return;
                }
            }
            self.pop();
        }
    }

    fn generate_implied_end_except(&mut self, except: YName) {
        self.generate_implied_end(|p| {
            if *p.ns == ns!(html) && *p.local == except {
                false
            } else {
                cursory_implied_end(p)
            }
        });
    }
    //§ END

    // Pop elements until an element from the set has been popped.  Returns the
    // number of elements popped.
    fn pop_until<P>(&mut self, pred: P) -> usize
    where
        P: Fn(ExpandedName) -> bool,
    {
        let mut n = 0;
        loop {
            n += 1;
            match self.open_elems.pop() {
                None => break,
                Some(elem) => {
                    if pred(self.sink.elem_name(&elem)) {
                        break;
                    }
                }
            }
        }
        n
    }

    fn pop_until_named(&mut self, name: YName) -> usize {
        self.pop_until(|p| *p.ns == ns!(html) && *p.local == name)
    }

    // Pop elements until one with the specified name has been popped.
    // Signal an error if it was not the first one.
    fn expect_to_close(&mut self, name: YName) {
        if self.pop_until_named(name.clone()) != 1 {
            self.sink.parse_error(Owned(format!(
                "Unexpected open element while closing {:?}",
                name
            )));
        }
    }

    fn close_p_element(&mut self) {
        declare_tag_set!(implied = [cursory_implied_end] - "p");
        self.generate_implied_end(implied);
        self.expect_to_close(y_name!("p"));
    }

    fn append_text(&mut self, text: StrTendril) -> ProcessResult<Handle> {
        self.insert_appropriately(AppendText(text), None);
        Done
    }

    fn append_comment(&mut self, _text: StrTendril) -> ProcessResult<Handle> {
        self.sink
            .parse_error(Borrowed("No use html comment, use yarte comments instead"));
        Done
    }

    //§ creating-and-inserting-nodes
    fn create_root(&mut self, attrs: Vec<Attribute>) {
        let elem = create_element(
            &mut self.sink,
            QualName::new(None, ns!(html), y_name!("html")),
            attrs,
        );
        self.push(&elem);
        self.sink.append(&self.doc_handle, AppendNode(elem));
    }

    fn insert_element(
        &mut self,
        push: PushFlag,
        ns: Namespace,
        name: YName,
        attrs: Vec<Attribute>,
    ) -> Handle {
        // Step 7.
        let qname = QualName::new(None, ns, name);
        let elem = create_element(&mut self.sink, qname, attrs);

        let insertion_point = self.appropriate_place_for_insertion(None);
        self.insert_at(insertion_point, AppendNode(elem.clone()));

        match push {
            Push => self.push(&elem),
            NoPush => (),
        }
        // FIXME: Remove from the stack if we can't append?
        elem
    }

    fn insert_element_for(&mut self, tag: Tag) -> Handle {
        self.insert_element(Push, ns!(html), tag.name, tag.attrs)
    }

    fn insert_and_pop_element_for(&mut self, tag: Tag) -> Handle {
        self.insert_element(NoPush, ns!(html), tag.name, tag.attrs)
    }

    fn insert_phantom(&mut self, name: YName) -> Handle {
        self.insert_element(Push, ns!(html), name, vec![])
    }
    //§ END

    fn create_formatting_element_for(&mut self, tag: Tag) -> Handle {
        // FIXME: This really wants unit tests.
        let mut first_match = None;
        let mut matches = 0usize;
        for (i, _, old_tag) in self.active_formatting_end_to_marker() {
            if tag.equiv_modulo_attr_order(old_tag) {
                first_match = Some(i);
                matches += 1;
            }
        }

        if matches >= 3 {
            self.active_formatting
                .remove(first_match.expect("matches with no index"));
        }

        let elem = self.insert_element(Push, ns!(html), tag.name.clone(), tag.attrs.clone());
        self.active_formatting.push(Element(elem.clone(), tag));
        elem
    }

    fn process_end_tag_in_body(&mut self, tag: Tag) {
        // Look back for a matching open element.
        let mut match_idx = None;
        for (i, elem) in self.open_elems.iter().enumerate().rev() {
            if self.html_elem_named(elem, tag.name.clone()) {
                match_idx = Some(i);
                break;
            }

            if self.elem_in(elem, special_tag) {
                self.sink
                    .parse_error(Borrowed("Found special tag while closing generic tag"));
                return;
            }
        }

        // Can't use unwrap_or_return!() due to rust-lang/rust#16617.
        let match_idx = match match_idx {
            None => {
                // I believe this is impossible, because the root
                // <html> element is in special_tag.
                self.unexpected(&tag);
                return;
            }
            Some(x) => x,
        };

        self.generate_implied_end_except(tag.name.clone());

        if match_idx != self.open_elems.len() - 1 {
            // mis-nested tags
            self.unexpected(&tag);
        }
        self.open_elems.truncate(match_idx);
    }

    //§ tree-construction
    fn is_foreign(&self, token: &Token) -> bool {
        if let EOFToken = *token {
            return false;
        }

        if self.open_elems.is_empty() {
            return false;
        }

        let name = self.sink.elem_name(self.adjusted_current_node());
        if let ns!(html) = *name.ns {
            return false;
        }

        if mathml_text_integration_point(name) {
            match *token {
                CharacterTokens(..) | NullCharacterToken => return false,
                TagToken(Tag {
                    kind: StartTag,
                    ref name,
                    ..
                }) if !matches!(*name, y_name!("mglyph") | y_name!("malignmark")) => {
                    return false;
                }
                _ => (),
            }
        }

        if svg_html_integration_point(name) {
            match *token {
                CharacterTokens(..) | NullCharacterToken => return false,
                TagToken(Tag { kind: StartTag, .. }) => return false,
                _ => (),
            }
        }

        if let expanded_name!(mathml "annotation-xml") = name {
            match *token {
                TagToken(Tag {
                    kind: StartTag,
                    name: y_name!("svg"),
                    ..
                }) => return false,
                CharacterTokens(..) | NullCharacterToken | TagToken(Tag { kind: StartTag, .. }) => {
                    return !self
                        .sink
                        .is_mathml_annotation_xml_integration_point(self.adjusted_current_node());
                }
                _ => {}
            };
        }

        true
    }
    //§ END

    fn enter_foreign(&mut self, mut tag: Tag, ns: Namespace) -> ProcessResult<Handle> {
        match ns {
            ns!(mathml) => self.adjust_mathml_attributes(&mut tag),
            ns!(svg) => self.adjust_svg_attributes(&mut tag),
            _ => (),
        }
        self.adjust_foreign_attributes(&mut tag);

        if tag.self_closing {
            self.insert_element(NoPush, ns, tag.name, tag.attrs);
            DoneAckSelfClosing
        } else {
            self.insert_element(Push, ns, tag.name, tag.attrs);
            Done
        }
    }

    fn adjust_svg_tag_name(&mut self, tag: &mut Tag) {
        let Tag { ref mut name, .. } = *tag;
        match *name {
            y_name!("altglyph") => *name = y_name!("altGlyph"),
            y_name!("altglyphdef") => *name = y_name!("altGlyphDef"),
            y_name!("altglyphitem") => *name = y_name!("altGlyphItem"),
            y_name!("animatecolor") => *name = y_name!("animateColor"),
            y_name!("animatemotion") => *name = y_name!("animateMotion"),
            y_name!("animatetransform") => *name = y_name!("animateTransform"),
            y_name!("clippath") => *name = y_name!("clipPath"),
            y_name!("feblend") => *name = y_name!("feBlend"),
            y_name!("fecolormatrix") => *name = y_name!("feColorMatrix"),
            y_name!("fecomponenttransfer") => *name = y_name!("feComponentTransfer"),
            y_name!("fecomposite") => *name = y_name!("feComposite"),
            y_name!("feconvolvematrix") => *name = y_name!("feConvolveMatrix"),
            y_name!("fediffuselighting") => *name = y_name!("feDiffuseLighting"),
            y_name!("fedisplacementmap") => *name = y_name!("feDisplacementMap"),
            y_name!("fedistantlight") => *name = y_name!("feDistantLight"),
            y_name!("fedropshadow") => *name = y_name!("feDropShadow"),
            y_name!("feflood") => *name = y_name!("feFlood"),
            y_name!("fefunca") => *name = y_name!("feFuncA"),
            y_name!("fefuncb") => *name = y_name!("feFuncB"),
            y_name!("fefuncg") => *name = y_name!("feFuncG"),
            y_name!("fefuncr") => *name = y_name!("feFuncR"),
            y_name!("fegaussianblur") => *name = y_name!("feGaussianBlur"),
            y_name!("feimage") => *name = y_name!("feImage"),
            y_name!("femerge") => *name = y_name!("feMerge"),
            y_name!("femergenode") => *name = y_name!("feMergeNode"),
            y_name!("femorphology") => *name = y_name!("feMorphology"),
            y_name!("feoffset") => *name = y_name!("feOffset"),
            y_name!("fepointlight") => *name = y_name!("fePointLight"),
            y_name!("fespecularlighting") => *name = y_name!("feSpecularLighting"),
            y_name!("fespotlight") => *name = y_name!("feSpotLight"),
            y_name!("fetile") => *name = y_name!("feTile"),
            y_name!("feturbulence") => *name = y_name!("feTurbulence"),
            y_name!("foreignobject") => *name = y_name!("foreignObject"),
            y_name!("glyphref") => *name = y_name!("glyphRef"),
            y_name!("lineargradient") => *name = y_name!("linearGradient"),
            y_name!("radialgradient") => *name = y_name!("radialGradient"),
            y_name!("textpath") => *name = y_name!("textPath"),
            _ => (),
        }
    }

    fn adjust_attributes<F>(&mut self, tag: &mut Tag, mut map: F)
    where
        F: FnMut(YName) -> Option<QualName>,
    {
        for &mut Attribute { ref mut name, .. } in &mut tag.attrs {
            if let Some(replacement) = map(name.local.clone()) {
                *name = replacement;
            }
        }
    }

    fn adjust_svg_attributes(&mut self, tag: &mut Tag) {
        self.adjust_attributes(tag, |k| match k {
            y_name!("attributename") => Some(qualname!("", "attributeName")),
            y_name!("attributetype") => Some(qualname!("", "attributeType")),
            y_name!("basefrequency") => Some(qualname!("", "baseFrequency")),
            y_name!("baseprofile") => Some(qualname!("", "baseProfile")),
            y_name!("calcmode") => Some(qualname!("", "calcMode")),
            y_name!("clippathunits") => Some(qualname!("", "clipPathUnits")),
            y_name!("diffuseconstant") => Some(qualname!("", "diffuseConstant")),
            y_name!("edgemode") => Some(qualname!("", "edgeMode")),
            y_name!("filterunits") => Some(qualname!("", "filterUnits")),
            y_name!("glyphref") => Some(qualname!("", "glyphRef")),
            y_name!("gradienttransform") => Some(qualname!("", "gradientTransform")),
            y_name!("gradientunits") => Some(qualname!("", "gradientUnits")),
            y_name!("kernelmatrix") => Some(qualname!("", "kernelMatrix")),
            y_name!("kernelunitlength") => Some(qualname!("", "kernelUnitLength")),
            y_name!("keypoints") => Some(qualname!("", "keyPoints")),
            y_name!("keysplines") => Some(qualname!("", "keySplines")),
            y_name!("keytimes") => Some(qualname!("", "keyTimes")),
            y_name!("lengthadjust") => Some(qualname!("", "lengthAdjust")),
            y_name!("limitingconeangle") => Some(qualname!("", "limitingConeAngle")),
            y_name!("markerheight") => Some(qualname!("", "markerHeight")),
            y_name!("markerunits") => Some(qualname!("", "markerUnits")),
            y_name!("markerwidth") => Some(qualname!("", "markerWidth")),
            y_name!("maskcontentunits") => Some(qualname!("", "maskContentUnits")),
            y_name!("maskunits") => Some(qualname!("", "maskUnits")),
            y_name!("numoctaves") => Some(qualname!("", "numOctaves")),
            y_name!("pathlength") => Some(qualname!("", "pathLength")),
            y_name!("patterncontentunits") => Some(qualname!("", "patternContentUnits")),
            y_name!("patterntransform") => Some(qualname!("", "patternTransform")),
            y_name!("patternunits") => Some(qualname!("", "patternUnits")),
            y_name!("pointsatx") => Some(qualname!("", "pointsAtX")),
            y_name!("pointsaty") => Some(qualname!("", "pointsAtY")),
            y_name!("pointsatz") => Some(qualname!("", "pointsAtZ")),
            y_name!("preservealpha") => Some(qualname!("", "preserveAlpha")),
            y_name!("preserveaspectratio") => Some(qualname!("", "preserveAspectRatio")),
            y_name!("primitiveunits") => Some(qualname!("", "primitiveUnits")),
            y_name!("refx") => Some(qualname!("", "refX")),
            y_name!("refy") => Some(qualname!("", "refY")),
            y_name!("repeatcount") => Some(qualname!("", "repeatCount")),
            y_name!("repeatdur") => Some(qualname!("", "repeatDur")),
            y_name!("requiredextensions") => Some(qualname!("", "requiredExtensions")),
            y_name!("requiredfeatures") => Some(qualname!("", "requiredFeatures")),
            y_name!("specularconstant") => Some(qualname!("", "specularConstant")),
            y_name!("specularexponent") => Some(qualname!("", "specularExponent")),
            y_name!("spreadmethod") => Some(qualname!("", "spreadMethod")),
            y_name!("startoffset") => Some(qualname!("", "startOffset")),
            y_name!("stddeviation") => Some(qualname!("", "stdDeviation")),
            y_name!("stitchtiles") => Some(qualname!("", "stitchTiles")),
            y_name!("surfacescale") => Some(qualname!("", "surfaceScale")),
            y_name!("systemlanguage") => Some(qualname!("", "systemLanguage")),
            y_name!("tablevalues") => Some(qualname!("", "tableValues")),
            y_name!("targetx") => Some(qualname!("", "targetX")),
            y_name!("targety") => Some(qualname!("", "targetY")),
            y_name!("textlength") => Some(qualname!("", "textLength")),
            y_name!("viewbox") => Some(qualname!("", "viewBox")),
            y_name!("viewtarget") => Some(qualname!("", "viewTarget")),
            y_name!("xchannelselector") => Some(qualname!("", "xChannelSelector")),
            y_name!("ychannelselector") => Some(qualname!("", "yChannelSelector")),
            y_name!("zoomandpan") => Some(qualname!("", "zoomAndPan")),
            _ => None,
        });
    }

    fn adjust_mathml_attributes(&mut self, tag: &mut Tag) {
        self.adjust_attributes(tag, |k| match k {
            y_name!("definitionurl") => Some(qualname!("", "definitionURL")),
            _ => None,
        });
    }

    fn adjust_foreign_attributes(&mut self, tag: &mut Tag) {
        self.adjust_attributes(tag, |k| match k {
            y_name!("xlink:actuate") => Some(qualname!("xlink" xlink "actuate")),
            y_name!("xlink:arcrole") => Some(qualname!("xlink" xlink "arcrole")),
            y_name!("xlink:href") => Some(qualname!("xlink" xlink "href")),
            y_name!("xlink:role") => Some(qualname!("xlink" xlink "role")),
            y_name!("xlink:show") => Some(qualname!("xlink" xlink "show")),
            y_name!("xlink:title") => Some(qualname!("xlink" xlink "title")),
            y_name!("xlink:type") => Some(qualname!("xlink" xlink "type")),
            y_name!("xml:base") => Some(qualname!("xml" xml "base")),
            y_name!("xml:lang") => Some(qualname!("xml" xml "lang")),
            y_name!("xml:space") => Some(qualname!("xml" xml "space")),
            y_name!("xmlns") => Some(qualname!("" xmlns "xmlns")),
            y_name!("xmlns:xlink") => Some(qualname!("xmlns" xmlns "xlink")),
            _ => None,
        });
    }

    fn foreign_start_tag(&mut self, mut tag: Tag) -> ProcessResult<Handle> {
        let current_ns = self.sink.elem_name(self.adjusted_current_node()).ns.clone();
        match current_ns {
            ns!(mathml) => self.adjust_mathml_attributes(&mut tag),
            ns!(svg) => {
                self.adjust_svg_tag_name(&mut tag);
                self.adjust_svg_attributes(&mut tag);
            }
            _ => (),
        }
        self.adjust_foreign_attributes(&mut tag);
        if tag.self_closing {
            // FIXME(#118): <script /> in SVG
            self.insert_element(NoPush, current_ns, tag.name, tag.attrs);
            DoneAckSelfClosing
        } else {
            self.insert_element(Push, current_ns, tag.name, tag.attrs);
            Done
        }
    }

    fn unexpected_start_tag_in_foreign_content(&mut self, tag: Tag) -> ProcessResult<Handle> {
        self.unexpected(&tag);
        if self.is_fragment() {
            self.foreign_start_tag(tag)
        } else {
            self.pop();
            while !self.current_node_in(|n| {
                *n.ns == ns!(html)
                    || mathml_text_integration_point(n)
                    || svg_html_integration_point(n)
            }) {
                self.pop();
            }
            ReprocessForeign(TagToken(tag))
        }
    }
}
