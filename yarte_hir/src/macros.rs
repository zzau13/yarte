#[macro_export]
macro_rules! visit_attrs {
    ($_self:ident, $attrs:ident) => {
        for it in $attrs {
            $_self.visit_attribute_mut(it)
        }
    };
}

#[macro_export]
macro_rules! visit_punctuated {
    ($_self:ident, $ele:expr, $method:ident) => {
        for mut el in Punctuated::pairs_mut($ele) {
            $_self.$method(el.value_mut());
        }
    };
}
