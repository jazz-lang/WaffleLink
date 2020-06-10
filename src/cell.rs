macro_rules! var_size_enum {
    ($name: ident {
        $($rest:tt)*
    }) => {
        #[repr(C)]
        pub struct $name {
            pub tag: u32,
        }
        paste::item!(
            pub trait [<$name ConvEnum>] {
                const VAL_TAG: u32;
                #[allow(non_snake_case)]
                fn [<as_ $name>](&self) -> &$name;
                #[allow(non_snake_case)]
                fn [<as_ $name _mut>](&mut self) -> &mut $name;
            }
            impl $name {
                pub fn downcast_to_ref<T:[<$name ConvEnum>]>(&self) -> &T {
                    unsafe {
                        assert!(self.tag == T::VAL_TAG);
                        std::mem::transmute(self)
                    }
                }
                pub fn downcast_to_mut<T:[<$name ConvEnum>]>(&mut self) -> &mut T {
                    unsafe {
                        assert!(self.tag == T::VAL_TAG);
                        std::mem::transmute(self)
                    }
                }

                pub fn is<T: [<$name ConvEnum>]>(&self) -> bool {
                    self.tag == T::VAL_TAG
                }
            }
        );
        var_size_enum!(@handle $name, 0;$($rest)*);
    };
    (@handle $name: ident,$cnt: expr;$fname: ident {$($field: ident : $t: ty),*} $($rest:tt)*) => {
        #[repr(C)]
        pub struct $fname {
            pub tag: u32,
            $(pub $field: $t),*
        }

        impl $fname {
            pub const TAG: u32 = $cnt;
            pub fn as_main_type(self) -> $name {
                unsafe {
                    let ptr = &self as *const Self;
                    ptr.cast::<$name>().read()
                }
            }
        }
        paste::item!(
        impl [<$name ConvEnum>] for $fname {
            const VAL_TAG: u32 = $cnt;
            #[allow(non_snake_case)]
            fn [<as_ $name>](&self) -> &$name {
                unsafe {
                    std::mem::transmute(self)
                }
            }
            #[allow(non_snake_case)]
            fn [<as_ $name _mut>](&mut self) -> &mut $name {
                unsafe {
                    std::mem::transmute(self)
                }
            }
        }
        );
        impl $name {
            #[allow(non_snake_case)]
            pub fn $fname($($field : $t),*) -> $fname {
                $fname {
                    tag: $cnt,
                    $($field),*
                }
            }
        }
        var_size_enum!(@handle $name,$cnt + 1;$($rest)*);
    };
    (@handle $name: ident,$cnt: expr;) => {};
}
