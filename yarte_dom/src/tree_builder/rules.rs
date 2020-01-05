// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// The tree builder rules, as a single, enormous nested match expression.

use crate::tree_builder::{tag_sets::*, types::*};
use html5ever::tokenizer::states::{Plaintext, Rawtext, Rcdata, ScriptData};
use markup5ever::{expanded_name, local_name, namespace_prefix, namespace_url, ns};

use html5ever::tendril::SliceExt;

fn any_not_whitespace(x: &StrTendril) -> bool {
    // FIXME: this might be much faster as a byte scan
    x.chars().any(|c| !is_ascii_whitespace(c))
}

fn current_node<Handle>(open_elems: &[Handle]) -> &Handle {
    open_elems.last().expect("no current element")
}

#[doc(hidden)]
impl<Handle, Sink> TreeBuilder<Handle, Sink>
where
    Handle: Clone,
    Sink: TreeSink<Handle = Handle>,
{
    fn step(&mut self, mode: InsertionMode, token: Token) -> ProcessResult<Handle> {
        self.debug_step(mode, &token);

        match mode {
            Initial => match_token!(token {
                CharacterTokens(NotSplit, text) => SplitWhitespace(text),
                CharacterTokens(Whitespace, _) => Done,
                token => Reprocess(BeforeHtml, token),
            }),
            //§ the-before-html-insertion-mode
            BeforeHtml => match_token!(token {
                CharacterTokens(NotSplit, text) => SplitWhitespace(text),
                CharacterTokens(Whitespace, _) => Done,
                tag @ <html> => {
                    self.create_root(tag.attrs);
                    self.mode = InHtml;
                    Done
                }

                </head> </body> </html> </br> => else,

                tag => self.unexpected(&tag),
            }),
            //§ parsing-main-inbody
            InHtml => match_token!(token {
                token @ CharacterTokens(..) => self.step(Text, token),
                CommentToken(text) => self.append_comment(text),
                NullCharacterToken => self.unexpected(&token),
                EOFToken => {
                    self.stop_parsing()
                }
                </body> => {
                    if self.in_scope_named(default_scope, local_name!("body")) {
                        self.check_body_end();
                        self.mode = AfterBody;
                    } else {
                        self.sink.parse_error(Borrowed("</body> with no <body> in scope"));
                    }
                    Done
                }

                </html> => {
                    if self.in_scope_named(default_scope, local_name!("body")) {
                        self.check_body_end();
                        Reprocess(AfterBody, token)
                    } else {
                        self.sink.parse_error(Borrowed("</html> with no <body> in scope"));
                        Done
                    }
                }

                tag @ <base> <basefont> <bgsound> <link> <meta> => {
                    // FIXME: handle <meta charset=...> and <meta http-equiv="Content-Type">
                    self.insert_and_pop_element_for(tag);
                    DoneAckSelfClosing
                }

                tag @ <area> <br> <embed> <img> <keygen> <wbr> <input> => {
                    self.reconstruct_formatting();
                    self.insert_and_pop_element_for(tag);
                    DoneAckSelfClosing
                }

                // TODO: Check tags and complete
                tag @ <address> <applet> <article> <aside> <blockquote> <body> <caption>
                <center> <col> <colgroup> <dd> <details> <dialog> <dir> <div> <dl>
                <dt> <fieldset> <figcaption> <figure> <footer> <form> <frame> <frameset>
                <head> <header> <hgroup> <li> <main> <marquee> <menu> <nav>
                <object> <ol> <p> <section> <select> <summary>
                <table> <tbody> <td> <tfoot> <th> <thead> <tr> <ul> => {
                    self.insert_element_for(tag);
                    Done
                }

                tag @ <h1> <h2> <h3> <h4> <h5> <h6> => {
                    self.close_p_element_in_button_scope();
                    if self.current_node_in(heading_tag) {
                        self.sink.parse_error(Borrowed("nested heading tags"));
                        self.pop();
                    }
                    self.insert_element_for(tag);
                    Done
                }

                tag @ <pre> <listing> => {
                    self.close_p_element_in_button_scope();
                    self.insert_element_for(tag);
                    self.ignore_lf = true;
                    Done
                }

                tag @ <plaintext> => {
                    self.close_p_element_in_button_scope();
                    self.insert_element_for(tag);
                    ToPlaintext
                }

                tag @ <button> => {
                    if self.in_scope_named(default_scope, local_name!("button")) {
                        self.sink.parse_error(Borrowed("nested buttons"));
                        self.generate_implied_end(cursory_implied_end);
                        self.pop_until_named(local_name!("button"));
                    }
                    self.reconstruct_formatting();
                    self.insert_element_for(tag);
                    Done
                }

                // TODO: Check tags and complete
                tag @ </address> </applet> </article> </aside> </blockquote> </caption>
                </center> </col> </colgroup> </details> </dialog> </dir> </div> </dl>
                </fieldset> </figcaption> </figure> </footer> </form> </frame> </frameset>
                </head> </header> </hgroup> </main> </marquee> </menu> </nav>
                </object> </ol> </section> </select> </summary>
                </table> </tbody> </td> </tfoot> </th> </thead> </tr> </ul> => {
                    if !self.in_scope_named(default_scope, tag.name.clone()) {
                        self.unexpected(&tag);
                    } else {
                        self.generate_implied_end(cursory_implied_end);
                        self.expect_to_close(tag.name);
                    }
                    Done
                }

                </p> => {
                    if !self.in_scope_named(button_scope, local_name!("p")) {
                        self.sink.parse_error(Borrowed("No <p> tag to close"));
                        self.insert_phantom(local_name!("p"));
                    }
                    self.close_p_element();
                    Done
                }

                tag @ </li> </dd> </dt> => {
                    let in_scope = if tag.name == local_name!("li") {
                        self.in_scope_named(list_item_scope, tag.name.clone())
                    } else {
                        self.in_scope_named(default_scope, tag.name.clone())
                    };
                    if in_scope {
                        self.generate_implied_end_except(tag.name.clone());
                        self.expect_to_close(tag.name);
                    } else {
                        self.sink.parse_error(Borrowed("No matching tag to close"));
                    }
                    Done
                }

                tag @ </h1> </h2> </h3> </h4> </h5> </h6> => {
                    if self.in_scope(default_scope, |n| self.elem_in(&n, heading_tag)) {
                        self.generate_implied_end(cursory_implied_end);
                        if !self.current_node_named(tag.name) {
                            self.sink.parse_error(Borrowed("Closing wrong heading tag"));
                        }
                        self.pop_until(heading_tag);
                    } else {
                        self.sink.parse_error(Borrowed("No heading tag to close"));
                    }
                    Done
                }

                tag @ <a> => {
                    self.handle_misnested_a_tags(&tag);
                    self.reconstruct_formatting();
                    self.create_formatting_element_for(tag);
                    Done
                }

                tag @ <b> <big> <code> <em> <font> <i> <s> <small> <strike> <strong> <tt> <u> => {
                    self.reconstruct_formatting();
                    self.create_formatting_element_for(tag);
                    Done
                }

                tag @ <nobr> => {
                    self.reconstruct_formatting();
                    if self.in_scope_named(default_scope, local_name!("nobr")) {
                        self.sink.parse_error(Borrowed("Nested <>obr>"));
                        self.adoption_agency(local_name!("nobr"));
                        self.reconstruct_formatting();
                    }
                    self.create_formatting_element_for(tag);
                    Done
                }

                tag @ </a> </b> </big> </code> </em> </font> </i> </nobr>
                  </s> </small> </strike> </strong> </tt> </u> => {
                    self.adoption_agency(tag.name);
                    Done
                }

                tag @ </br> => {
                    self.unexpected(&tag);
                    self.step(InHtml, TagToken(Tag {
                        kind: StartTag,
                        attrs: vec!(),
                        ..tag
                    }))
                }

                tag @ <param> <source> <track> => {
                    self.insert_and_pop_element_for(tag);
                    DoneAckSelfClosing
                }

                tag @ <hr> => {
                    self.close_p_element_in_button_scope();
                    self.insert_and_pop_element_for(tag);
                    DoneAckSelfClosing
                }

                tag @ <image> => {
                    self.unexpected(&tag);
                    self.step(InHtml, TagToken(Tag {
                        name: local_name!("img"),
                        ..tag
                    }))
                }

                tag @ <textarea> => {
                    self.ignore_lf = true;
                    self.parse_raw_data(tag, Rcdata)
                }

                tag @ <xmp> => {
                    self.close_p_element_in_button_scope();
                    self.reconstruct_formatting();
                    self.parse_raw_data(tag, Rawtext)
                }

                tag @ <iframe> => {
                    self.parse_raw_data(tag, Rawtext)
                }

                tag @ <noembed> => {
                    self.parse_raw_data(tag, Rawtext)
                }

                // <noscript> handled in wildcard case below

                tag @ <rb> <rtc> => {
                    if self.in_scope_named(default_scope, local_name!("ruby")) {
                        self.generate_implied_end(cursory_implied_end);
                    }
                    if !self.current_node_named(local_name!("ruby")) {
                        self.unexpected(&tag);
                    }
                    self.insert_element_for(tag);
                    Done
                }

                tag @ <rp> <rt> => {
                    if self.in_scope_named(default_scope, local_name!("ruby")) {
                        self.generate_implied_end_except(local_name!("rtc"));
                    }
                    if !self.current_node_named(local_name!("rtc")) && !self.current_node_named(local_name!("ruby")) {
                        self.unexpected(&tag);
                    }
                    self.insert_element_for(tag);
                    Done
                }

                tag @ <option> => {
                    if self.current_node_named(local_name!("option")) {
                        self.pop();
                    }
                    self.insert_element_for(tag);
                    Done
                }

                tag @ <optgroup> => {
                    if self.current_node_named(local_name!("option")) {
                        self.pop();
                    }
                    if self.current_node_named(local_name!("optgroup")) {
                        self.pop();
                    }
                    self.insert_element_for(tag);
                    Done
                }

                </optgroup> => {
                    if self.open_elems.len() >= 2
                        && self.current_node_named(local_name!("option"))
                        && self.html_elem_named(&self.open_elems[self.open_elems.len() - 2],
                            local_name!("optgroup")) {
                        self.pop();
                    }
                    if self.current_node_named(local_name!("optgroup")) {
                        self.pop();
                    } else {
                        self.unexpected(&token);
                    }
                    Done
                }

                </option> => {
                    if self.current_node_named(local_name!("option")) {
                        self.pop();
                    } else {
                        self.unexpected(&token);
                    }
                    Done
                }

                tag @ <math> => self.enter_foreign(tag, ns!(mathml)),

                tag @ <svg> => self.enter_foreign(tag, ns!(svg)),

                tag @ <_> => {
                    if self.opts.scripting_enabled && tag.name == local_name!("noscript") {
                        self.parse_raw_data(tag, Rawtext)
                    } else {
                        self.reconstruct_formatting();
                        self.insert_element_for(tag);
                        Done
                    }
                }

                tag @ </_> => {
                    self.process_end_tag_in_body(tag);
                    Done
                }

                // FIXME: This should be unreachable, but match_token requires a
                // catch-all case.
                _ => panic!("impossible case in InHtml mode"),
            }),
            //§ parsing-main-afterbody
            AfterBody => match_token!(token {
                CharacterTokens(NotSplit, text) => SplitWhitespace(text),
                CharacterTokens(Whitespace, _) => self.step(InHtml, token),
                CommentToken(text) => self.append_comment(text),

                <html> => self.step(InHtml, token),

                </html> => {
                    if self.is_fragment() {
                        self.unexpected(&token);
                    } else {
                        self.mode = AfterAfterBody;
                    }
                    Done
                }

                EOFToken => self.stop_parsing(),

                token => {
                    self.unexpected(&token);
                    Reprocess(InHtml, token)
                }
            }),
             //§ the-after-after-body-insertion-mode
            AfterAfterBody => match_token!(token {
                CharacterTokens(NotSplit, text) => SplitWhitespace(text),
                CharacterTokens(Whitespace, _) => self.step(InHtml, token),
                CommentToken(text) => self.append_comment(text),

                <html> => self.step(InHtml, token),

                EOFToken => self.stop_parsing(),

                token => {
                    self.unexpected(&token);
                    Reprocess(InHtml, token)
                }
            }),
            //§ parsing-text
            Text => match_token!(token {
            // TODO: remove whitespace or round inline tags replace by ' '
                CharacterTokens(NotSplit, text) => SplitWhitespace(text),
//                CharacterTokens(Whitespace, text) => self.append_whitespace(text),
                CharacterTokens(_, text) => self.append_text(text),

                CommentToken(text) => self.append_comment(text),

                EOFToken => Reprocess(self.orig_mode.take().unwrap(), token),

                tag => Reprocess(InHtml, tag),
            }),
             //§ parsing-main-incdata
            RawText => match_token!(token {
                CommentToken(text) => self.append_comment(text),
                CharacterTokens(_, text) => self.append_text(text),

                EOFToken => {
                    self.unexpected(&token);
                    if self.current_node_named(local_name!("script")) {
                        let current = current_node(&self.open_elems);
                        self.sink.mark_script_already_started(current);
                    }
                    self.pop();
                    Reprocess(self.orig_mode.take().unwrap(), token)
                }

                tag @ </_> => {
                    let node = self.pop();
                    self.mode = self.orig_mode.take().unwrap();
                    if tag.name == local_name!("script") {
                        return Script(node);
                    }
                    Done
                }

                // The spec doesn't say what to do here.
                // Other tokens are impossible?
                _ => panic!("impossible case in Text mode"),
            }),
            //§ END
        }
    }

    // TODO
    fn step_foreign(&mut self, token: Token) -> ProcessResult<Handle> {
        match_token!(token {
            NullCharacterToken => {
                self.unexpected(&token);
                self.append_text("\u{fffd}".to_tendril())
            }

            CharacterTokens(_, text) => {
                self.append_text(text)
            }

            CommentToken(text) => self.append_comment(text),

            tag @ <b> <big> <blockquote> <body> <br> <center> <code> <dd> <div> <dl>
                <dt> <em> <embed> <h1> <h2> <h3> <h4> <h5> <h6> <head> <hr> <i>
                <img> <li> <listing> <menu> <meta> <nobr> <ol> <p> <pre> <ruby>
                <s> <small> <span> <strong> <strike> <sub> <sup> <table> <tt>
                <u> <ul> <var> => self.unexpected_start_tag_in_foreign_content(tag),

            tag @ <font> => {
                let unexpected = tag.attrs.iter().any(|attr| {
                    matches!(attr.name.expanded(),
                             expanded_name!("", "color") |
                             expanded_name!("", "face") |
                             expanded_name!("", "size"))
                });
                if unexpected {
                    self.unexpected_start_tag_in_foreign_content(tag)
                } else {
                    self.foreign_start_tag(tag)
                }
            }

            tag @ <_> => self.foreign_start_tag(tag),

            // FIXME(#118): </script> in SVG

            tag @ </_> => {
                let mut first = true;
                let mut stack_idx = self.open_elems.len() - 1;
                loop {
                    if stack_idx == 0 {
                        return Done;
                    }

                    let html;
                    let eq;
                    {
                        let node_name = self.sink.elem_name(&self.open_elems[stack_idx]);
                        html = *node_name.ns == ns!(html);
                        eq = node_name.local.eq_ignore_ascii_case(&tag.name);
                    }
                    if !first && html {
                        let mode = self.mode;
                        return self.step(mode, TagToken(tag));
                    }

                    if eq {
                        self.open_elems.truncate(stack_idx);
                        return Done;
                    }

                    if first {
                        self.unexpected(&tag);
                        first = false;
                    }
                    stack_idx -= 1;
                }
            }

            // FIXME: This should be unreachable, but match_token requires a
            // catch-all case.
            _ => panic!("impossible case in foreign content"),
        })
    }
}
