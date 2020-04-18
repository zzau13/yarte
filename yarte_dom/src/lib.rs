extern crate quote_impersonated as quote;
extern crate syn_impersonated as syn;

#[macro_use]
#[cfg(feature = "wasm-app")]
pub mod dom;
pub mod dom_fmt;
mod serialize;
mod sink;

pub use self::dom_fmt::DOMFmt;

#[cfg(test)]
mod test {
    use crate::{
        serialize::serialize,
        sink::{parse_document, parse_fragment},
    };

    #[test]
    fn test_div() {
        let src = "<div attr=\"some\" \t class=\"any\"    \n>Hi!<br   /></div><div \
                   some7Na=\"hola\">hi</div>";
        let expected =
            "<div attr=\"some\" class=\"any\">Hi!<br></div><div some7na=\"hola\">hi</div>";

        let a = parse_fragment(src).unwrap();
        let mut writer = Vec::new();
        serialize(&mut writer, a.into(), Default::default()).expect("some serialize node");

        let html = String::from_utf8(writer).expect("");

        assert_eq!(expected, html);
    }

    #[test]
    fn test_table() {
        let src = "<table>yarteHashHTMLExpressionsATTT0x00000000</table>";
        let expected = "<table>yarteHashHTMLExpressionsATTT0x00000000</table>";

        let a = parse_fragment(src).unwrap();
        let mut writer = Vec::new();
        serialize(&mut writer, a.into(), Default::default()).expect("some serialize node");

        let html = String::from_utf8(writer).expect("");

        assert_eq!(expected, html);
    }

    #[test]
    fn test_attributes() {
        let src = "<div class=\"yarteHashHTMLExpressionsATTT0x00000000\"></div>";
        let expected = "<div class=\"yarteHashHTMLExpressionsATTT0x00000000\"></div>";

        let a = parse_fragment(src).unwrap();
        let mut writer = Vec::new();
        serialize(&mut writer, a.into(), Default::default()).expect("some serialize node");

        let html = String::from_utf8(writer).expect("");

        assert_eq!(expected, html);
    }

    #[test]
    fn test_document_err() {
        let src = "<div class=\"yarteHashHTMLExpressionsATTT0x00000000\"></div>";

        assert!(parse_document(src).is_err());
    }

    #[test]
    fn test_document_ok() {
        let src = "<html><body><div \
                   class=\"yarteHashHTMLExpressionsATTT0x00000000\"></div></body></html>";
        let expected = "<html><body><div \
                        class=\"yarteHashHTMLExpressionsATTT0x00000000\"></div></body></\
                        html>";

        let a = parse_document(src).unwrap();
        let mut writer = Vec::new();
        serialize(&mut writer, a.into(), Default::default()).expect("some serialize node");

        let html = String::from_utf8(writer).expect("");

        assert_eq!(expected, html);
    }

    #[test]
    fn test_document_ok_doctype() {
        let src = "<!DOCTYPE html><html><body><div \
                   class=\"yarteHashHTMLExpressionsATTT0x00000000\"></div></body></html>";
        let expected = "<!DOCTYPE html><html><body><div \
                        class=\"yarteHashHTMLExpressionsATTT0x00000000\"></div></body></\
                        html>";

        let a = parse_document(src).unwrap();
        let mut writer = Vec::new();
        serialize(&mut writer, a.into(), Default::default()).expect("some serialize node");

        let html = String::from_utf8(writer).expect("");

        assert_eq!(expected, html);
    }

    #[test]
    fn test_document_ok_table() {
        let src = "<html><body><table>yarteHashHTMLExpressionsATTT0x00000000</table></\
                   body></html>";
        let expected = "<html><body><table>yarteHashHTMLExpressionsATTT0x00000000</table></\
                        body></html>";

        let a = parse_document(src).unwrap();
        let mut writer = Vec::new();
        serialize(&mut writer, a.into(), Default::default()).expect("some serialize node");

        let html = String::from_utf8(writer).expect("");

        assert_eq!(expected, html);
    }

    #[test]
    fn test_document_ok_head() {
        let src = "<html><head><title>yartehashhtmlexpressionsattt0x00000000</title></\
                   head><body><div attr=\"some\" \t class=\"any\"    \n>Hi!<br   /></div><div \
                   some7Na=\"hola\">hi</div></body></html>";
        let expected = "<html><head><title>yartehashhtmlexpressionsattt0x00000000</title></\
                        head><body><div attr=\"some\" class=\"any\">Hi!<br></div><div \
                        some7na=\"hola\">hi</div></body></html>";

        let a = parse_document(src).unwrap();
        let mut writer = Vec::new();
        serialize(&mut writer, a.into(), Default::default()).expect("some serialize node");

        let html = String::from_utf8(writer).expect("");

        assert_eq!(expected, html);
    }
}
