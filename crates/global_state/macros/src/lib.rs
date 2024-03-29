use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(GlobalState)]
pub fn derive_global_state(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let mut quotes = vec![];
    if let Data::Enum(enm) = input.data {
        for variant in enm.variants.iter() {
            let variant_ident = &variant.ident;
            quotes.push(quote! {
                app.add_exit_system(#ident::#variant_ident, global_state::global_cleanup);
                app.add_exit_system(#ident::#variant_ident, global_state::reset_state_time::<#ident>);
                app.add_system_set(
                    iyes_loopless::prelude::ConditionSet::new()
                        .run_in_state(#ident::#variant_ident)
                        .with_system(global_state::update_state_time::<#ident>)
                        .into()
                );
            });
        }
    }

    TokenStream::from(quote! {
        impl GlobalState for #ident {
            fn init_global_state(app: &mut bevy::app::App) {
                app.add_state(#ident::default());
                app.init_resource::<global_state::StateTime<#ident>>();
                #(#quotes)*
            }
        }
    })
}

#[proc_macro_derive(TransientState)]
pub fn derive_transient_state(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let mut quotes = vec![];
    if let Data::Enum(enm) = input.data {
        for variant in enm.variants.iter() {
            let variant_ident = &variant.ident;
            quotes.push(quote! {
                app.add_exit_system(#ident::#variant_ident, global_state::cleanup_transient);
                app.add_exit_system(#ident::#variant_ident, global_state::reset_state_time::<#ident>);
                app.add_system_set(
                    iyes_loopless::prelude::ConditionSet::new()
                        .run_in_state(#ident::#variant_ident)
                        .with_system(global_state::update_state_time::<#ident>)
                        .into()
                );
            });
        }
    }

    TokenStream::from(quote! {
        impl TransientState for #ident {
            fn init_transient_state(app: &mut bevy::app::App) {
                app.add_state(#ident::default());
                app.init_resource::<global_state::StateTime<#ident>>();
                #(#quotes)*
            }
        }
    })
}
