use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, parse_macro_input};

#[proc_macro_derive(Id)]
pub fn derive_id(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the struct name
    let struct_name = &input.ident;
    let value_name = syn::Ident::new(&format!("__ID_MACRO_{}", struct_name), struct_name.span());

    // Check that this is a tuple struct with exactly one field
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed,
            Fields::Unnamed(_) => {
                return Error::new_spanned(
                    &input,
                    "Id derive macro only works on tuple structs with exactly one field",
                )
                .to_compile_error()
                .into();
            }
            _ => {
                return Error::new_spanned(&input, "Id derive macro only works on tuple structs")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return Error::new_spanned(&input, "Id derive macro only works on structs")
                .to_compile_error()
                .into();
        }
    };

    // Get the type of the single field
    let field_type = &fields[0].ty;

    // Generate the implementation - only the impl blocks, not the struct again
    let expanded = quote! {
        // Implement Debug, Display, Clone, Copy, Hash, PartialEq, Eq manually
        impl Clone for #struct_name {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl Copy for #struct_name {}

        impl std::hash::Hash for #struct_name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }

        impl PartialEq for #struct_name {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl Eq for #struct_name {}

        impl IntoLua for #struct_name {
            fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
                self.0.into_lua(lua)
            }
        }

        impl FromLua for #struct_name {
            fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
                #field_type::from_lua(value, lua).map(Self)
            }
        }

        #[allow(non_upper_case_globals)]
        static #value_name: std::sync::LazyLock<std::sync::atomic::Atomic<#field_type>> = std::sync::LazyLock::new(Default::default);
        impl #struct_name {
            /// Creates a fresh number id.
            pub fn new() -> Self {
                Self(#value_name.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
            }
        }
    };

    TokenStream::from(expanded)
}
