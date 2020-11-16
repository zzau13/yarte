use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Result, Token,
};

pub(super) struct ExprPipe {
    list: Punctuated<Expr, Token![=>]>,
}

impl Parse for ExprPipe {
    fn parse(input: ParseStream) -> Result<Self> {
        let list = if input.is_empty() {
            Punctuated::new()
        } else {
            Punctuated::parse_separated_nonempty(input)?
        };
        Ok(ExprPipe { list })
    }
}

impl Into<Vec<crate::Expr>> for ExprPipe {
    fn into(self) -> Vec<crate::Expr> {
        self.list
            .into_pairs()
            .map(|p| crate::Expr(p.into_value()))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use syn::parse_str;

    use super::*;

    #[test]
    fn test() {
        let src = "another_bar.field => match fo { A | B => \"foo\", _ => \"bar\" } => bar => foo = \"bar => \"\n => fuu = 1  => goo = true    ";
        let expected = vec![
            parse_str::<crate::Expr>("another_bar.field").unwrap(),
            parse_str::<crate::Expr>("match fo { A | B => \"foo\", _ => \"bar\" }").unwrap(),
            parse_str::<crate::Expr>("bar").unwrap(),
            parse_str::<crate::Expr>("foo=\"bar => \"").unwrap(),
            parse_str::<crate::Expr>("fuu=1").unwrap(),
            parse_str::<crate::Expr>("goo=true").unwrap(),
        ];

        let res: Vec<crate::Expr> = parse_str::<ExprPipe>(src).unwrap().into();

        assert_eq!(expected, res);

        let src = "bar => foo = \"bar => \"\n => fuu = 1  => goo = true   => ";
        assert!(parse_str::<ExprPipe>(src).is_err());

        let src = "                 \n\t ";
        assert!(parse_str::<ExprPipe>(src).is_ok());
    }
}
