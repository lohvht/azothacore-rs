#[macro_export]
macro_rules! deref_boilerplate {
    ( $newtype:ty, $deref_type:ty, $field:tt ) => {
        impl ::std::ops::Deref for $newtype {
            type Target = $deref_type;

            fn deref(&self) -> &Self::Target {
                &self.$field
            }
        }

        impl ::std::ops::DerefMut for $newtype {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$field
            }
        }
    };
}
