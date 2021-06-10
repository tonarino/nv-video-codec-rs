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
            Self {
                $name: Some($name),
                ..self
            }
        }
    };
}
