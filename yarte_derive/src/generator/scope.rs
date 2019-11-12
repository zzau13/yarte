use std::ops::Index;

#[derive(Debug)]
pub(super) struct Scope {
    scope: Vec<String>,
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
    pub(super) fn new(root: String, count: usize) -> Scope {
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

    pub(super) fn pops(&mut self) -> Vec<String> {
        pop!(self).collect()
    }

    pub(super) fn push(&mut self, ident: &str) -> &String {
        self.scope.push(format!("{}__{}", ident, self.count));
        self.count += 1;
        &self.scope[self.scope.len() - 1]
    }

    pub(super) fn push_scope(&mut self, t: Vec<String>) {
        self.level.push(self.scope.len());
        self.extend(t);
    }

    #[inline]
    pub(super) fn extend(&mut self, t: Vec<String>) {
        self.scope.extend(t);
    }

    pub(super) fn get(&self, n: usize) -> Option<&[String]> {
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
    pub(super) fn root(&self) -> &String {
        debug_assert!(!self.scope.is_empty());
        &self.scope[0]
    }

    pub(super) fn get_by(&self, ident: &str) -> Option<&String> {
        self.scope.iter().rev().find(|e| {
            let e = e.as_bytes();
            let ident = ident.as_bytes();
            e.eq(ident)
                || (ident.len() + 2 < e.len()
                    && ident.eq(&e[..ident.len()])
                    && e[ident.len()..ident.len() + 2].eq(b"__")
                    && e[ident.len() + 2].is_ascii_digit())
        })
    }
}

impl Index<usize> for Scope {
    type Output = [String];

    fn index(&self, i: usize) -> &Self::Output {
        self.get(i).expect("Scope not found.")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_root() {
        let s = Scope::new("self".to_owned(), 0);
        assert_eq!(s.root(), "self");
    }

    #[test]
    fn test_get() {
        let mut s = Scope::new("self".to_owned(), 0);
        let scope = vec!["foo".to_string()];
        let id = s.len();
        s.push_scope(scope.clone());

        assert_eq!(s.root(), "self");
        assert_eq!(s.get(id).unwrap(), scope.as_slice());
        assert_eq!(&s[id], scope.as_slice());
    }

    #[test]
    fn test_push() {
        let mut s = Scope::new("self".to_owned(), 0);
        let mut scope = vec!["foo".to_string()];
        let id = s.len();
        s.push_scope(scope.clone());
        let ident = "bar";
        let var = s.push(ident).clone();
        scope.push(var.clone());

        assert_eq!(s.root(), "self");
        assert_eq!(s.get(id).unwrap(), scope.as_slice());
        assert_eq!(&s[id], scope.as_slice());
        assert_eq!(s.get_by(ident).unwrap(), &var);
    }

    #[test]
    fn test_extend() {
        let mut s = Scope::new("self".to_owned(), 0);
        let mut scope = vec!["foo".to_string()];
        let id = s.len();
        s.push_scope(scope.clone());
        let extend = vec!["bar".to_string()];
        s.extend(extend.clone());
        scope.extend(extend);

        assert_eq!(s.root(), "self");
        assert_eq!(s.get(id).unwrap(), scope.as_slice());
        assert_eq!(&s[id], scope.as_slice());
    }
}
