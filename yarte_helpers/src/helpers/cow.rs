use std::borrow::Cow;

pub trait AsCow {
    fn as_cow(&self) -> Cow<'_, str>;
}

pub trait AsCowA {
    fn __as_cow_it(&self) -> Cow<'_, str>;
}

impl<T: AsCow> AsCowA for T {
    #[inline]
    fn __as_cow_it(&self) -> Cow<'_, str> {
        self.as_cow()
    }
}

macro_rules! impl_cow {
    ($($t:ty)*) => {
        $(
        impl AsCow for $t {
            #[inline]
            fn as_cow(&self) -> Cow<'_, str> {
                Cow::Owned(self.to_string())
            }
        }
        )*
    };
}

impl_cow! {
    u8 u16 u32 u64 usize
    i8 i16 i32 i64 isize
    char bool
}

impl AsCow for str {
    #[inline]
    fn as_cow(&self) -> Cow<'_, str> {
        Cow::Borrowed(self)
    }
}

impl AsCow for String {
    #[inline]
    fn as_cow(&self) -> Cow<'_, str> {
        Cow::Borrowed(self)
    }
}
