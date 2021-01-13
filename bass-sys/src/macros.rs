/// Wraps all calls with debug-level logging of the arguments and return value.
macro_rules! extern_log {
    ($($vis:vis fn $name:ident ( $($arg:ident: $argty:ty),* $(,)? ) $(-> $ty:ty)? ;)*) => {
        $(
            paste::item! {
                #[allow(non_snake_case)]
                mod [< __inner_ffi_ $name >] {
                    #[allow(unused_imports)]
                    use std::os::raw::*;
                    #[allow(unused_imports)]
                    use crate::types::*;
                    extern "C" {
                        pub fn $name($($arg: $argty,)*) $(-> $ty)?;
                    }
                }
            }
            #[allow(non_snake_case)]
            $vis unsafe fn $name ($($arg: $argty,)*) $(-> $ty)? {
                log::trace!("entering {} ({:?})", stringify!($name), ($($arg,)*));
                let result = paste::expr! { self::[< __inner_ffi_ $name >]::$name($($arg,)*) };
                log::trace!("exiting {} => {:?}", stringify!($name), result);
                result
            }
         )*
    };
}
