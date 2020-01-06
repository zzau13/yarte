// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use html5ever::{tendril::StrTendril, tokenizer::Doctype};

pub fn doctype_error(doctype: &Doctype) -> bool {
    fn opt_tendril_as_slice(x: &Option<StrTendril>) -> Option<&str> {
        match x.as_ref() {
            Some(t) => Some(t),
            None => None,
        }
    }

    let name = opt_tendril_as_slice(&doctype.name);
    let public = opt_tendril_as_slice(&doctype.public_id);
    let system = opt_tendril_as_slice(&doctype.system_id);

    match (name, public, system) {
        (Some("html"), None, None) => false,
        _ => true,
    }
}
