use std::collections::{BTreeSet, HashSet};
use std::slice::Iter;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseBuffer},
    parse2,
    punctuated::Punctuated,
    Field, FieldValue, Ident, Member, Token, Type, VisPublic, Visibility,
};

use indexmap::map::IndexMap;

use yarte_dom::dom::{ExprId, VarId};

use super::utils::{get_field_root_ident, get_t_root_ident};

pub type PathNode = (Ident, Vec<Step>);

// TODO: Expressions in path
// TODO: use HTMLCollection
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Step {
    FirstChild,
    NextSibling,
    Each(usize),
}

// TODO: Expressions in path
// TODO: use HTMLCollection
pub struct PathStep<'a, I: Iterator<Item = &'a Step>>(pub I);

// TODO: to node and unchecked cast to node type
impl<'a, I: Iterator<Item = &'a Step>> PathStep<'a, I> {
    pub fn into_tokens(self, tokens: &mut TokenStream) {
        for i in self.0 {
            use Step::*;
            tokens.extend(match i {
                FirstChild => quote!(.first_element_child().unwrap_throw()),
                NextSibling => quote!(.next_element_sibling().unwrap_throw()),
                Each(_) => todo!("Expressions in path"),
            })
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Parent {
    Head,
    Body,
    Expr(ExprId),
}

impl Default for Parent {
    fn default() -> Self {
        Parent::Head
    }
}

pub struct Len {
    pub base: usize,
    pub expr: Vec<ExprId>,
}

impl From<&[InsertPath]> for Len {
    fn from(i: &[InsertPath]) -> Self {
        use InsertPath::*;
        let mut base = 0;
        let mut expr = vec![];
        for x in i {
            match x {
                Before => {
                    base += 1;
                }
                Expr(i) => {
                    expr.push(*i);
                }
            }
        }

        Len { base, expr }
    }
}

// TODO: Inline elements
#[derive(Clone, Debug)]
pub enum InsertPath {
    Before,
    Expr(ExprId),
}

#[derive(Debug, Clone)]
pub struct BlackBox {
    pub doc: String,
    pub name: Ident,
    pub ty: Type,
}

impl Into<Field> for BlackBox {
    fn into(self) -> Field {
        let BlackBox { name, ty, doc } = self;
        let attr: PAttr = parse2(quote!(#[doc = #doc])).unwrap();
        Field {
            attrs: attr.0,
            vis: Visibility::Public(VisPublic {
                pub_token: <Token![pub]>::default(),
            }),
            ident: Some(name),
            colon_token: Some(<Token![:]>::default()),
            ty,
        }
    }
}

pub struct PAttr(pub Vec<syn::Attribute>);

impl Parse for PAttr {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        Ok(PAttr(input.call(syn::Attribute::parse_outer)?))
    }
}

#[derive(Debug, Default)]
pub struct State {
    pub id: Parent,
    pub bases: HashSet<VarId>,
    /// black box fields
    pub black_box: Vec<BlackBox>,
    /// Intermediate buffers
    pub buff_build: Vec<TokenStream>,
    pub buff_hydrate: Vec<TokenStream>,
    pub buff_new: Vec<TokenStream>,
    pub buff_render: Vec<(BTreeSet<VarId>, TokenStream)>,
    /// Path to nodes in current scope
    pub path_nodes: Vec<PathNode>,
    /// Path to events in current scope
    pub path_events: Vec<PathNode>,
    /// path to nodes
    pub steps: Vec<Step>,
    /// Component ident
    pub component: Option<Ident>,
    ///
    pub parent_id: usize,
    /// Current black box
    pub current_bb: TokenStream,
}

impl State {
    pub fn new(bases: HashSet<VarId>) -> Self {
        State {
            bases,
            ..Default::default()
        }
    }

    pub fn get_black_box_fields(
        &self,
        dom: &Ident,
        on_build: bool,
    ) -> Punctuated<FieldValue, Token![,]> {
        let t_root = get_t_root_ident();
        let root = get_field_root_ident();
        self.black_box
            .iter()
            .fold(<Punctuated<FieldValue, Token![,]>>::new(), |mut acc, x| {
                if x.name == t_root {
                    acc.push(FieldValue {
                        attrs: vec![],
                        member: Member::Named(x.name.clone()),
                        colon_token: Some(<Token![:]>::default()),
                        expr: parse2(quote!(yarte_wasm_app::YNumber::zero())).unwrap(),
                    });
                } else if x.name == root {
                    acc.push(FieldValue {
                        attrs: vec![],
                        member: Member::Named(x.name.clone()),
                        colon_token: Some(<Token![:]>::default()),
                        expr: parse2(quote!(#dom)).unwrap(),
                    });
                } else if on_build && x.name.to_string().starts_with("__closure__") {
                    acc.push(FieldValue {
                        attrs: vec![],
                        member: Member::Named(x.name.clone()),
                        colon_token: Some(<Token![:]>::default()),
                        expr: parse2(quote!(None)).unwrap(),
                    });
                } else {
                    let name = &x.name;
                    acc.push(FieldValue {
                        attrs: vec![],
                        member: Member::Named(x.name.clone()),
                        colon_token: Some(<Token![:]>::default()),
                        expr: parse2(quote!(#name)).unwrap(),
                    });
                }

                acc
            })
    }

    #[inline]
    pub fn get_render_hash(&self) -> IndexMap<Vec<VarId>, TokenStream> {
        self.buff_render.iter().fold(
            IndexMap::new(),
            |mut acc: IndexMap<Vec<VarId>, TokenStream>, (i, x)| {
                acc.entry(i.iter().copied().collect())
                    .and_modify(|old| {
                        old.extend(x.clone());
                    })
                    .or_insert_with(|| x.clone());
                acc
            },
        )
    }

    pub fn get_black_box(&self, name: &Ident) -> TokenStream {
        let fields = self.black_box.iter().cloned().map(Into::into).fold(
            Punctuated::<Field, Token![,]>::new(),
            |mut acc, x| {
                acc.push(x);
                acc
            },
        );

        quote! {
            #[doc = "Internal elements and difference tree"]
            pub struct #name {
                #fields
            }
        }
    }

    pub fn add_t_root(&mut self, base: TokenStream) {
        self.black_box.push(BlackBox {
            doc: "Difference tree".to_string(),
            name: get_t_root_ident(),
            ty: parse2(base).unwrap(),
        });
    }
}

// TODO: check non continuous stack implementation
pub struct Stack<T> {
    data: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new(t: T) -> Self {
        Stack { data: vec![t] }
    }

    pub fn last(&self) -> &T {
        self.data.last().expect("one state in stack")
    }

    pub fn last_mut(&mut self) -> &mut T {
        self.data.last_mut().expect("one state in stack")
    }

    pub fn push(&mut self, t: T) {
        self.data.push(t);
    }

    pub fn pop(&mut self) -> T {
        self.data.pop().expect("one state in stack")
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Stack<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}
