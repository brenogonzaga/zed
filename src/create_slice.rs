#[macro_export]
macro_rules! create_slice {
    (
        enum_name: $enum_name:ident,
        fn_base: $base:ident,
        state: $state_ty:ty,
        initial_state: $initial_state:expr,
        actions: {
            $( $action_variant:ident $( { $($field:ident : $ftype:ty),* $(,)? } )? , )*
        },
        reducer: $reducer:expr
    ) => {
        $crate::paste! {
            #[derive(Clone, Debug)]
            pub enum $enum_name {
                $(
                    $action_variant $( { $($field : $ftype),* } )?,
                )*
            }

            pub const [<$base:upper _INITIAL_STATE>]: $state_ty = $initial_state;

            pub fn [<$base _reducer>](state: &$state_ty, action: &$enum_name) -> $state_ty {
                let mut draft = state.clone();
                match action {
                    $(
                        $enum_name::$action_variant $( { $($field),* } )? => {
                            ($reducer)(&mut draft, action);
                            draft
                        },
                    )*
                }
            }
        }
    };
}
