use std::io::{self, Write};

use log::warn;

use markup5ever::{local_name, namespace_url, ns};

use yarte_parser::trim;

use crate::interface::{QualName, YName};

pub type AttrRef<'a> = (&'a QualName, &'a str);

#[derive(Default)]
pub struct ElemInfo {
    html_name: Option<YName>,
    ignore_children: bool,
}

enum Ws {
    Skip,
    C,
}

#[derive(Copy, Clone, Default)]
pub struct SerializerOpt {
    pub wasm: bool,
}

pub struct HtmlSerializer<Wr: Write> {
    pub writer: Wr,
    stack: Vec<ElemInfo>,
    skip_ws: Option<Ws>,
    next_ws: Option<String>,
    opts: SerializerOpt,
}

fn tagname(name: &QualName) -> YName {
    match name.ns {
        ns!(html) | ns!(mathml) | ns!(svg) => (),
        ref ns => {
            // FIXME(#122)
            warn!("node with weird namespace {:?}", ns);
        }
    }

    name.local.clone()
}

// TODO: optional tag
impl<Wr: Write> HtmlSerializer<Wr> {
    pub fn new(writer: Wr, opts: SerializerOpt) -> Self {
        HtmlSerializer {
            writer,
            stack: vec![ElemInfo {
                html_name: None,
                ignore_children: false,
            }],
            next_ws: None,
            skip_ws: None,
            opts,
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
            if self.opts.wasm && name.local.to_string().starts_with("on") {
                continue;
            }
            self.writer.write_all(b" ")?;

            match name.ns {
                ns!() => (),
                ns!(xml) => self.writer.write_all(b"xml:")?,
                ns!(xmlns) => {
                    if name.local != y_name!("xmlns") {
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
                y_name!("area")
                | y_name!("base")
                | y_name!("basefont")
                | y_name!("bgsound")
                | y_name!("br")
                | y_name!("col")
                | y_name!("embed")
                | y_name!("frame")
                | y_name!("hr")
                | y_name!("img")
                | y_name!("input")
                | y_name!("keygen")
                | y_name!("link")
                | y_name!("meta")
                | y_name!("param")
                | y_name!("source")
                | y_name!("track")
                | y_name!("wbr") => true,
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
            Some(y_name!("style"))
            | Some(y_name!("script"))
            | Some(y_name!("xmp"))
            | Some(y_name!("iframe"))
            | Some(y_name!("noembed"))
            | Some(y_name!("noframes"))
            | Some(y_name!("plaintext")) => false,

            _ => true,
        };

        let v = match self.parent().html_name {
            Some(y_name!("pre")) | Some(y_name!("listing")) => {
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

    pub fn write_doctype(&mut self, name: &str) -> io::Result<()> {
        assert!(self.next_ws.is_none(), "text before doctype");
        self.writer.write_all(b"<!DOCTYPE ")?;
        self.writer.write_all(name.as_bytes())?;
        self.writer.write_all(b">")
    }

    pub fn end(&mut self, parent: Option<&QualName>) -> io::Result<()> {
        if let Some(name) = parent {
            self.tag_whitespace(name)?;
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
            y_name!("a")
            | y_name!("abbr")
            | y_name!("b")
            | y_name!("bdi")
            | y_name!("bdo")
            | y_name!("br")
            | y_name!("cite")
            | y_name!("code")
            | y_name!("data")
            | y_name!("del")
            | y_name!("dfn")
            | y_name!("em")
            | y_name!("i")
            | y_name!("input")
            | y_name!("ins")
            | y_name!("kbd")
            | y_name!("mark")
            | y_name!("q")
            | y_name!("rp")
            | y_name!("rt")
            | y_name!("ruby")
            | y_name!("s")
            | y_name!("samp")
            | y_name!("small")
            | y_name!("span")
            | y_name!("strong")
            | y_name!("sub")
            | y_name!("sup")
            | y_name!("time")
            | y_name!("u")
            | y_name!("var")
            | y_name!("wbr") => {
                if let Some(text) = self.next_ws.take() {
                    match self.skip_ws {
                        Some(Ws::C) | None if !text.is_empty() => self.writer.write_all(b" ")?,
                        _ => (),
                    }
                }
                self.skip_ws = Some(Ws::C);
            }
            YName::Expr(_) => {
                if let Some(text) = self.next_ws.take() {
                    match self.skip_ws {
                        None if !text.is_empty() => self.writer.write_all(text.as_bytes())?,
                        Some(Ws::C) => self.writer.write_all(b" ")?,
                        _ => (),
                    }
                }
                self.skip_ws = None;
            }
            _ => {
                self.next_ws = None;
                self.skip_ws = Some(Ws::Skip);
            }
        }

        Ok(())
    }
}
