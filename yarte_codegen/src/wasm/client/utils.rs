use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, punctuated::Punctuated, Field, Ident, Token, Type};

use yarte_dom::dom::{
    Attribute, Each, Element, ExprId, ExprOrText, Expression, IfBlock, IfElse, Node,
};
use yarte_helpers::helpers::calculate_hash;

use super::state::{InsertPath, PathNode, PathStep};

thread_local! {
    static BB_TYPE: Type = parse2(quote!(<Self as Template>::BlackBox)).unwrap();
    static SELF_ID: u64 = calculate_hash(&"self");
}

#[inline]
pub fn get_self_id() -> u64 {
    SELF_ID.with(|x| *x)
}

#[inline]
pub fn is_black_box(ty: &Type) -> bool {
    BB_TYPE.with(|black| ty.eq(&black))
}

#[inline]
pub fn is_inner(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path.is_ident("inner"))
}

#[inline]
pub fn is_state(Field { attrs, ty, .. }: &Field) -> bool {
    !(is_inner(attrs) || is_black_box(ty))
}

pub fn is_on_attr(attr: &Attribute) -> Option<&str> {
    match &attr.name {
        ExprOrText::Text(s) if s.starts_with("on") => Some(s),
        _ => None,
    }
}

pub fn all_children_text<'a, I: Iterator<Item = &'a Node> + Clone>(mut doc: I) -> bool {
    !doc.clone().all(|x| {
        if let Node::Elem(Element::Text(_)) = x {
            true
        } else {
            false
        }
    }) && doc.all(|x| match x {
        Node::Elem(Element::Text(_)) => true,
        Node::Expr(e) => match e {
            Expression::IfElse(_, block) => {
                let IfElse { ifs, if_else, els } = &**block;
                all_if_block_text(ifs)
                    && if_else.iter().all(|x| all_if_block_text(x))
                    && els
                        .as_ref()
                        .map(|x| all_children_text(x.iter()))
                        .unwrap_or(true)
            }
            Expression::Each(_, block) => {
                let Each { body, .. } = &**block;
                all_children_text(body.iter())
            }
            Expression::Local(..) => false,
            _ => true,
        },
        _ => false,
    })
}

#[inline]
pub fn all_if_block_text(IfBlock { block, .. }: &IfBlock) -> bool {
    all_children_text(block.iter())
}

pub fn check_attr_is_text(attr: Attribute) -> bool {
    attr.value.len() == 1
        && match attr.value[0] {
            ExprOrText::Text(..) => true,
            ExprOrText::Expr(..) => false,
        }
}

pub fn get_insert_point<'b, I: Iterator<Item = &'b Node>>(nodes: I) -> Vec<InsertPath> {
    let mut insert = vec![];
    // TODO: inline nodes, expressions, ...
    for e in nodes {
        match e {
            Node::Elem(Element::Node { .. }) => insert.push(InsertPath::Before),
            Node::Expr(Expression::Each(id, _)) | Node::Expr(Expression::IfElse(id, _)) => {
                insert.push(InsertPath::Expr(*id))
            }
            _ => (),
        }
    }

    insert
}

#[inline]
pub fn get_t_root_ident() -> Ident {
    const T_ROOT: &str = "t_root";
    format_ident!("{}", T_ROOT)
}

#[inline]
pub fn get_table_dom_ident(id: ExprId) -> Ident {
    const TABLE_DOM: &str = "__ytable_dom__";
    format_ident!("{}{}", TABLE_DOM, id)
}

#[inline]
pub fn get_table_ident(id: ExprId) -> Ident {
    const TABLE: &str = "__ytable__";
    format_ident!("{}{}", TABLE, id)
}

#[inline]
pub fn get_vdom_ident(id: ExprId) -> Ident {
    const ELEM: &str = "__dom__";
    format_ident!("{}{}", ELEM, id)
}

#[inline]
// TODO: multiple roots
pub fn get_field_root_ident() -> Ident {
    const ROOT: &str = "__root";
    format_ident!("{}", ROOT)
}

#[inline]
pub fn get_component_ty_ident(id: ExprId) -> Ident {
    const TY: &str = "YComponent";
    format_ident!("{}{}", TY, id)
}

#[inline]
pub fn get_node_ident(id: ExprId) -> Ident {
    const NODE: &str = "__ynode__";
    format_ident!("{}{}", NODE, id)
}

#[inline]
pub fn get_body_ident() -> Ident {
    format_ident!("__ybody")
}

pub fn get_number_u8(bits: Vec<bool>) -> u8 {
    let mut n = 0;
    for (i, b) in bits.into_iter().enumerate() {
        if b {
            n += 1 << i as u8
        }
    }
    n
}

pub fn get_number_u16(bits: Vec<bool>) -> u16 {
    let mut n = 0;
    for (i, b) in bits.into_iter().enumerate() {
        if b {
            n += 1 << i as u16
        }
    }
    n
}

pub fn get_number_u32(bits: &[bool]) -> u32 {
    let mut n = 0;
    for (i, b) in bits.iter().enumerate() {
        if *b {
            n += 1 << i as u32
        }
    }
    n
}

pub fn get_split_32(mut bits: &[bool]) -> Punctuated<syn::Expr, Token![,]> {
    let mut buff = Punctuated::new();
    while !bits.is_empty() {
        let (current, next) = bits.split_at(32);
        bits = next;
        let current = get_number_u32(current);
        buff.push(parse2(quote!(#current)).unwrap());
    }

    buff
}

pub fn get_t_root_type(len: usize) -> (TokenStream, usize) {
    match len {
        0..=8 => (quote!(u8), 8),
        9..=16 => (quote!(u16), 16),
        17..=32 => (quote!(u32), 32),
        33..=64 => (quote!(yarte::U64), 64),
        65..=128 => (quote!(yarte::U128), 128),
        129..=256 => (quote!(yarte::U256), 256),
        _ => todo!("more than 256 variables per context"),
    }
}

// TODO: Fix me!!
pub fn get_steps<'b, I: Iterator<Item = &'b PathNode>>(
    mut nodes: I,
    parent: TokenStream,
) -> TokenStream {
    let mut buff = vec![];
    let mut stack = vec![];
    if let Some((ident, path)) = nodes.next() {
        buff.push((parent.clone(), ident.clone(), PathStep(path.iter())));
        stack.push((ident, path))
    }
    for (ident, path) in nodes {
        let mut check = true;
        for (i, last) in stack.iter().rev() {
            if path.starts_with(last) {
                // TODO: assert_ne!(last.len(), path.len());
                buff.push((
                    quote!(#i),
                    ident.clone(),
                    PathStep(path[last.len()..].iter()),
                ));
                check = false;
                break;
            }
        }
        if check {
            buff.push((parent.clone(), ident.clone(), PathStep(path.iter())));
        }
        stack.push((ident, path))
    }

    let mut tokens = TokenStream::new();
    for (p, i, path) in buff.drain(..) {
        tokens.extend(quote!(let #i = #p));
        path.into_tokens(&mut tokens);
        tokens.extend(quote!(;))
    }

    tokens
}

#[inline]
pub fn get_drop<I: Iterator<Item = Ident>>(component: &Ident, roots: I) -> TokenStream {
    let mut tokens = TokenStream::new();
    for root in roots {
        tokens.extend(quote!(self.#root.remove();));
    }
    quote! {
        impl Drop for #component {
            fn drop(&mut self) {
                #tokens
            }
        }
    }
}
