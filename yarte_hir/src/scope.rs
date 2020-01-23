use std::ops::Index;

use quote::{format_ident, quote};

#[derive(Debug)]
pub(super) struct Scope {
    scope: Vec<syn::Expr>,
    level: Vec<usize>,
    pub(super) count: usize,
}

macro_rules! pop {
    ($_self:ident) => {{
        let last = $_self.level.pop().expect("someone scope");
        $_self.scope.drain(last..)
    }};
}

impl Scope {
    pub(super) fn new(root: syn::Expr, count: usize) -> Scope {
        Scope {
            scope: vec![root],
            level: vec![],
            count,
        }
    }

    #[inline]
    pub(super) fn len(&self) -> usize {
        self.level.len() + 1
    }

    #[inline]
    pub(super) fn is_empty(&self) -> bool {
        self.scope.is_empty()
    }

    pub(super) fn pop(&mut self) {
        pop!(self);
    }

    pub(super) fn pops(&mut self) -> Vec<syn::Expr> {
        pop!(self).collect()
    }

    pub(super) fn push_ident(&mut self, ident: &str) -> syn::Ident {
        let ident = format_ident!("{}__{}", ident, format!("{:#010x?}", self.count));
        self.scope
            .push(syn::parse2(quote!(#ident)).expect("Correct expression"));
        self.count += 1;
        ident
    }

    pub(super) fn push_scope(&mut self, t: Vec<syn::Expr>) {
        self.level.push(self.scope.len());
        self.extend(t);
    }

    #[inline]
    pub(super) fn extend(&mut self, t: Vec<syn::Expr>) {
        self.scope.extend(t);
    }

    pub(super) fn get(&self, n: usize) -> Option<&[syn::Expr]> {
        if n == 0 {
            Some(
                self.level
                    .first()
                    .map_or(&self.scope, |j| &self.scope[..*j]),
            )
        } else {
            self.level
                .get(n)
                .map_or(self.level.get(n - 1).map(|j| &self.scope[*j..]), |j| {
                    Some(&self.scope[self.level[n - 1]..*j])
                })
        }
    }

    #[inline]
    pub(super) fn root(&self) -> &syn::Expr {
        debug_assert!(!self.scope.is_empty());
        &self.scope[0]
    }

    pub(super) fn get_by(&self, ident: &str) -> Option<&syn::Expr> {
        self.scope.iter().rev().find(|e| {
            let e = quote!(#e).to_string();
            let e = e.as_bytes();
            let ident = ident.as_bytes();
            e.eq(ident)
                || (ident.len() + 12 == e.len()
                    && ident.eq(&e[..ident.len()])
                    && e[ident.len()..ident.len() + 4].eq(b"__0x")
                    && e[ident.len() + 4..].iter().all(|x| x.is_ascii_hexdigit()))
        })
    }
}

impl Index<usize> for Scope {
    type Output = [syn::Expr];

    fn index(&self, i: usize) -> &Self::Output {
        self.get(i).expect("Scope not found.")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use syn::parse_str;

    #[test]
    fn test_root() {
        let root: syn::Expr = parse_str("self").unwrap();
        let s = Scope::new(root.clone(), 0);
        assert_eq!(s.root(), &root);
    }

    #[test]
    fn test_get() {
        let root: syn::Expr = parse_str("self").unwrap();
        let mut s = Scope::new(root.clone(), 0);
        let scope = vec![parse_str("foo").unwrap()];
        let id = s.len();
        s.push_scope(scope.clone());

        assert_eq!(s.root(), &root);
        assert_eq!(s.get(id).unwrap(), scope.as_slice());
        assert_eq!(&s[id], scope.as_slice());
    }

    #[test]
    fn test_push() {
        let root: syn::Expr = parse_str("self").unwrap();
        let mut s = Scope::new(root.clone(), 0);
        let mut scope = vec![parse_str("foo").unwrap()];
        let id = s.len();
        s.push_scope(scope.clone());
        let ident = "bar";
        let var = s.push_ident(ident);
        let var: syn::Expr = syn::parse2(quote!(#var)).unwrap();
        scope.push(var.clone());

        assert_eq!(s.root(), &root);
        assert_eq!(s.get(id).unwrap(), scope.as_slice());
        assert_eq!(&s[id], scope.as_slice());
        assert_eq!(s.get_by(ident).unwrap(), &var);
    }

    #[test]
    fn test_extend() {
        let root: syn::Expr = parse_str("self").unwrap();
        let mut s = Scope::new(root.clone(), 0);
        let mut scope = vec![parse_str("foo").unwrap()];
        let id = s.len();
        s.push_scope(scope.clone());
        let extend = vec![parse_str("bar").unwrap()];
        s.extend(extend.clone());
        scope.extend(extend);

        assert_eq!(s.root(), &root);
        assert_eq!(s.get(id).unwrap(), scope.as_slice());
        assert_eq!(&s[id], scope.as_slice());
    }
}
