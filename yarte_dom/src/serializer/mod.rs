// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{
    default::Default,
    io::{self, Write},
};

use html5ever::serialize::serialize as html_serialize;
use markup5ever::serialize::Serialize;

mod tree_element;

pub use self::tree_element::TreeElement;

pub fn serialize<Wr, T>(writer: Wr, node: &T) -> io::Result<()>
where
    Wr: Write,
    T: Serialize,
{
    html_serialize(writer, node, Default::default())
}
