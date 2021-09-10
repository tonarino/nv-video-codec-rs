macro_rules! builder_field_setter {
    ($name:ident : $type:ty) => {
        pub fn $name(self, $name: $type) -> Self {
            Self { $name, ..self }
        }
    };
}

// TODO(efyang) : have tryfrom?
macro_rules! ffi_enum {
    (
        $(#[$m:meta])*
        $v:vis enum $name:ident = $ffi_name:ident
        cvt_err: $cvt_err_name:ident
        {
            $($field:ident = $ffi_field:ident)*
        }
    ) => {
        $(#[$m])*
        $v enum $name {
            $($field),*,
        }

        impl Into<$ffi_name> for $name {
            fn into(self) -> $ffi_name {
                match self {
                    $(Self::$field => $ffi_name::$ffi_field),*,
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $cvt_err_name {
            UnknownVariant(String),
        }

        impl std::convert::TryFrom<$ffi_name> for $name {
            type Error = $cvt_err_name;
            fn try_from(raw: $ffi_name) -> Result<Self, Self::Error> {
                 match raw {
                    $($ffi_name::$ffi_field => Ok(Self::$field)),*,
                    _ => Err($cvt_err_name::UnknownVariant(raw.0.to_string())),
                }
            }
        }
    };
}
