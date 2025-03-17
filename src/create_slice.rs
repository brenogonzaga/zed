#[macro_export]
macro_rules! create_slice {
    (
        enum_name: $enum_name:ident,
        fn_base: $base:ident,
        state: $state_ty:ty,
        initial_state: $initial_state:expr,
        actions: {
            $(
                $action_variant:ident $( ( $($payload:tt)+ ) )?,
            )*
        },
        reducer: $reducer:expr
    ) => {
        paste::paste! {
            #[derive(Clone, Debug)]
            pub enum $enum_name {
                $(
                    $action_variant $( ( $($payload)+ ) )?,
                )*
            }

            // Constante com nome em UPPER_SNAKE_CASE, usando o "nome base"
            pub const [<$base:upper _INITIAL_STATE>]: $state_ty = $initial_state;

            // Função reducer em snake_case, usando o "nome base"
            pub fn [<$base _reducer>](state: &$state_ty, action: &$enum_name) -> $state_ty {
                let mut draft = state.clone();
                match action {
                    $(
                        $enum_name::$action_variant $( ( $($payload)+ ) )? => {
                            ($reducer)(&mut draft, action);
                            draft
                        },
                    )*
                }
            }
        }
    };
}
