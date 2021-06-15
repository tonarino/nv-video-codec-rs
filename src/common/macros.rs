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
    }
}