use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Result, Token,
};

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

impl Into<Vec<Expr>> for ExprList {
    fn into(self) -> Vec<Expr> {
        self.list.into_pairs().map(|p| p.into_value()).collect()
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
            parse_str::<Expr>("bar").unwrap(),
            parse_str::<Expr>("foo=\"bar,\"").unwrap(),
            parse_str::<Expr>("fuu=1").unwrap(),
            parse_str::<Expr>("goo=true").unwrap(),
        ];

        let res: Vec<Expr> = parse_str::<ExprList>(src).unwrap().into();

        assert_eq!(expected, res);

        let src = "bar, foo = \"bar,\"\n, fuu = 1  , goo = true";
        let res: Vec<Expr> = parse_str::<ExprList>(src).unwrap().into();

        assert_eq!(expected, res);
    }
}
