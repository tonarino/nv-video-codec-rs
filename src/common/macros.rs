macro_rules! builder_field_setter {
    ($name:ident : $type:ty) => {
        pub fn $name(self, $name: $type) -> Self {
            Self { $name, ..self }
        }
    };
}

macro_rules! builder_field_setter_opt {
    ($name:ident : $type:ty) => {
        pub fn $name(self, $name: $type) -> Self {
            Self { $name: Some($name), ..self }
        }
    };
}

macro_rules! define_opaque_pointer_type {
    ($name:ident) => {
        #[repr(C)]
        struct $name {
            _data: [u8; 0],
            _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
        }
    };
}

// TODO(efyang) : have tryfrom?
macro_rules! ffi_enum {
    (
        $(#[$m:meta])*
        $v:vis enum $name:ident = $ffi_name:ident
        {
            $($field:ident = $ffi_field:ident)*
        }
    ) => {
        $(#[$m])*
        $v enum $name {
            $($field),*,
        }

        impl Into<$ffi_name::Type> for $name {
            fn into(self) -> $ffi_name::Type {
                match self {
                    $(Self::$field => $ffi_name::$ffi_field),*,
                }
            }
        }

        impl From<$ffi_name::Type> for $name {
            fn from(raw: $ffi_name::Type) -> Self {
                match raw {
                    $($ffi_name::$ffi_field => Self::$field),*,
                    _ => panic!("Encountered unknown variant of type {}: {}", stringify!($name), raw),
                }
            }
        }
    };
}
