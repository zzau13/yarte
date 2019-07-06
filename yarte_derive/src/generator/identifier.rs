#[inline]
pub(super) fn is_tuple_index(ident: &[u8]) -> bool {
    1 < ident.len() && ident[0] == b'_' && ident[1..].iter().all(|x| x.is_ascii_digit())
}
