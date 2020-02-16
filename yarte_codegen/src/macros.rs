// PDA current state get/set
// Double mutable ref in self
#[macro_export]
macro_rules! last {
    ($_self:ident) => {
        $_self.stack.last().expect("one state in stack")
    };
}

#[macro_export]
macro_rules! last_mut {
    ($_self:ident) => {
        $_self.stack.last_mut().expect("one state in stack")
    };
}
