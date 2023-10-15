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

#[macro_export]
macro_rules! mut_g {
    ( $e:expr ) => {
        $e.write().unwrap()
    };
}

#[macro_export]
macro_rules! get_g {
    ( $e:expr ) => {
        $e.read().unwrap()
    };
}
