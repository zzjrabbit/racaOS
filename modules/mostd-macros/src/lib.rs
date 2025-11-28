use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let main_fn = parse_macro_input!(item as ItemFn);
    let main_fn_name = &main_fn.sig.ident;

    quote!(
        #[unsafe(no_mangle)]
        pub extern "C" fn init() {
            let _: () = #main_fn_name();
        }

        #[expect(unused)]
        #main_fn
    )
    .into()
}
