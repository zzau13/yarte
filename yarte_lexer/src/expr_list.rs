use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Result, Token};

pub(super) struct ExprList {
    list: Punctuated<Expr, Token![,]>,
}

impl Parse for ExprList {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ExprList {
            list: input.parse_terminated(Expr::parse)?,
        })
    }
}

impl From<ExprList> for Vec<crate::Expr> {
    fn from(list: ExprList) -> Vec<crate::Expr> {
        list.list
            .into_pairs()
            .map(|p| crate::Expr(p.into_value()))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use syn::parse_str;

    #[test]
    fn test() {
        let src = "bar, foo = \"bar,\"\n, fuu = 1  , goo = true,    ";
        let expected = vec![
            parse_str::<crate::Expr>("bar").unwrap(),
            parse_str::<crate::Expr>("foo=\"bar,\"").unwrap(),
            parse_str::<crate::Expr>("fuu=1").unwrap(),
            parse_str::<crate::Expr>("goo=true").unwrap(),
        ];

        let res: Vec<crate::Expr> = parse_str::<ExprList>(src).unwrap().into();

        assert_eq!(expected, res);

        let src = "bar, foo = \"bar,\"\n, fuu = 1  , goo = true";
        let res: Vec<crate::Expr> = parse_str::<ExprList>(src).unwrap().into();

        assert_eq!(expected, res);
    }
}
