// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io::{self, Write};

use log::warn;
pub use markup5ever::serialize::AttrRef;
use markup5ever::{local_name, namespace_url, ns, LocalName, QualName};

use yarte_parser::trim;

#[derive(Default)]
pub struct ElemInfo {
    html_name: Option<LocalName>,
    ignore_children: bool,
}

enum Ws {
    Skip,
    C,
}

pub struct HtmlSerializer<Wr: Write> {
    pub writer: Wr,
    stack: Vec<ElemInfo>,
    skip_ws: Option<Ws>,
    next_ws: Option<String>,
}

fn tagname(name: &QualName) -> LocalName {
    match name.ns {
        ns!(html) | ns!(mathml) | ns!(svg) => (),
        ref ns => {
            // FIXME(#122)
            warn!("node with weird namespace {:?}", ns);
        }
    }

    name.local.clone()
}

impl<Wr: Write> HtmlSerializer<Wr> {
    pub fn new(writer: Wr) -> Self {
        HtmlSerializer {
            writer,
            stack: vec![ElemInfo {
                html_name: None,
                ignore_children: false,
            }],
            next_ws: None,
            skip_ws: None,
        }
    }

    fn parent(&mut self) -> &mut ElemInfo {
        if self.stack.is_empty() {
            panic!("no parent ElemInfo")
        }
        self.stack.last_mut().unwrap()
    }

    fn write_escaped(&mut self, text: &str, attr_mode: bool) -> io::Result<()> {
        for c in text.chars() {
            match c {
                '&' => self.writer.write_all(b"&amp;"),
                '\u{00A0}' => self.writer.write_all(b"&nbsp;"),
                '"' if attr_mode => self.writer.write_all(b"&quot;"),
                '<' if !attr_mode => self.writer.write_all(b"&lt;"),
                '>' if !attr_mode => self.writer.write_all(b"&gt;"),
                c => self.writer.write_fmt(format_args!("{}", c)),
            }?;
        }
        Ok(())
    }

    pub fn start_elem<'a, AttrIter>(&mut self, name: QualName, attrs: AttrIter) -> io::Result<()>
    where
        AttrIter: Iterator<Item = AttrRef<'a>>,
    {
        let html_name = match name.ns {
            ns!(html) => Some(name.local.clone()),
            _ => None,
        };

        if self.parent().ignore_children {
            self.stack.push(ElemInfo {
                html_name,
                ignore_children: true,
            });
            return Ok(());
        }

        self.tag_whitespace(&name)?;

        self.writer.write_all(b"<")?;
        self.writer.write_all(tagname(&name).as_bytes())?;
        for (name, value) in attrs {
            self.writer.write_all(b" ")?;

            match name.ns {
                ns!() => (),
                ns!(xml) => self.writer.write_all(b"xml:")?,
                ns!(xmlns) => {
                    if name.local != local_name!("xmlns") {
                        self.writer.write_all(b"xmlns:")?;
                    }
                }
                ns!(xlink) => self.writer.write_all(b"xlink:")?,
                ref ns => {
                    // FIXME(#122)
                    warn!("attr with weird namespace {:?}", ns);
                    self.writer.write_all(b"unknown_namespace:")?;
                }
            }

            self.writer.write_all(name.local.as_bytes())?;
            if !value.is_empty() {
                self.writer.write_all(b"=\"")?;
                self.write_escaped(value, true)?;
                self.writer.write_all(b"\"")?;
            }
        }
        self.writer.write_all(b">")?;

        let ignore_children = name.ns == ns!(html)
            && match name.local {
                local_name!("area")
                | local_name!("base")
                | local_name!("basefont")
                | local_name!("bgsound")
                | local_name!("br")
                | local_name!("col")
                | local_name!("embed")
                | local_name!("frame")
                | local_name!("hr")
                | local_name!("img")
                | local_name!("input")
                | local_name!("keygen")
                | local_name!("link")
                | local_name!("meta")
                | local_name!("param")
                | local_name!("source")
                | local_name!("track")
                | local_name!("wbr") => true,
                _ => false,
            };

        self.stack.push(ElemInfo {
            html_name,
            ignore_children,
        });

        Ok(())
    }

    pub fn end_elem(&mut self, name: QualName) -> io::Result<()> {
        let info = match self.stack.pop() {
            Some(info) => info,
            _ => panic!("no ElemInfo"),
        };
        if info.ignore_children {
            return Ok(());
        }

        self.tag_whitespace(&name)?;

        self.writer.write_all(b"</")?;
        self.writer.write_all(tagname(&name).as_bytes())?;
        self.writer.write_all(b">")
    }

    pub fn write_text(&mut self, text: &str) -> io::Result<()> {
        assert!(self.next_ws.is_none(), "{:?} at \n{:?}", self.next_ws, text);
        let escape = match self.parent().html_name {
            Some(local_name!("style"))
            | Some(local_name!("script"))
            | Some(local_name!("xmp"))
            | Some(local_name!("iframe"))
            | Some(local_name!("noembed"))
            | Some(local_name!("noframes"))
            | Some(local_name!("plaintext")) => false,

            _ => true,
        };

        let v = match self.parent().html_name {
            Some(local_name!("pre")) | Some(local_name!("listing")) => {
                self.skip_ws = None;
                self.next_ws = None;
                text
            }
            _ => {
                let (l, v, r) = trim(text);

                if !l.is_empty() && v.is_empty() && r.is_empty() {
                    self.next_ws = Some(l.into());
                    v
                } else {
                    match self.skip_ws.take() {
                        Some(Ws::C) if !l.is_empty() => self.writer.write_all(b" ")?,
                        None => self.writer.write_all(l.as_bytes())?,
                        _ => (),
                    }
                    if !r.is_empty() {
                        self.next_ws = Some(r.into());
                    } else {
                        self.next_ws = None;
                    }
                    v
                }
            }
        };

        if escape {
            self.write_escaped(v, false)
        } else {
            self.writer.write_all(v.as_bytes())
        }
    }

    pub fn write_comment(&mut self, text: &str) -> io::Result<()> {
        if let Some(text) = &self.next_ws.take() {
            match self.skip_ws {
                Some(Ws::C) => self.writer.write_all(b" ")?,
                None => self.writer.write_all(text.as_bytes())?,
                Some(Ws::Skip) => (),
            }
        }
        self.skip_ws = None;
        self.writer.write_all(b"<!--")?;
        self.writer.write_all(text.as_bytes())?;
        self.writer.write_all(b"-->")
    }

    pub fn write_doctype(&mut self, name: &str) -> io::Result<()> {
        assert!(self.next_ws.is_none(), "text before doctype");
        self.writer.write_all(b"<!DOCTYPE ")?;
        self.writer.write_all(name.as_bytes())?;
        self.writer.write_all(b">")
    }

    pub fn end(&mut self, parent: Option<QualName>) -> io::Result<()> {
        if let Some(name) = parent {
            self.tag_whitespace(&name)?;
        } else if let Some(text) = &self.next_ws.take() {
            match self.skip_ws {
                Some(Ws::C) => self.writer.write_all(b" ")?,
                None => self.writer.write_all(text.as_bytes())?,
                Some(Ws::Skip) => (),
            }
        }
        Ok(())
    }

    fn tag_whitespace(&mut self, name: &QualName) -> io::Result<()> {
        match name.local {
            local_name!("a")
            | local_name!("abbr")
            | local_name!("b")
            | local_name!("bdi")
            | local_name!("bdo")
            | local_name!("br")
            | local_name!("cite")
            | local_name!("code")
            | local_name!("data")
            | local_name!("del")
            | local_name!("dfn")
            | local_name!("em")
            | local_name!("i")
            | local_name!("input")
            | local_name!("ins")
            | local_name!("kbd")
            | local_name!("mark")
            | local_name!("q")
            | local_name!("rp")
            | local_name!("rt")
            | local_name!("ruby")
            | local_name!("s")
            | local_name!("samp")
            | local_name!("small")
            | local_name!("span")
            | local_name!("strong")
            | local_name!("sub")
            | local_name!("sup")
            | local_name!("time")
            | local_name!("u")
            | local_name!("var")
            | local_name!("wbr") => {
                if let Some(text) = self.next_ws.take() {
                    match self.skip_ws {
                        Some(Ws::C) | None if !text.is_empty() => self.writer.write_all(b" ")?,
                        _ => (),
                    }
                }
                self.skip_ws = Some(Ws::C);
            }
            _ => {
                self.next_ws = None;
                self.skip_ws = Some(Ws::Skip);
            }
        }

        Ok(())
    }
}
