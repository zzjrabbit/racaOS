use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
    let name = format!("{}", attr);

    let main_fn = parse_macro_input!(item as ItemFn);
    let main_fn_name = &main_fn.sig.ident;

    quote!(
        #[used]
        #[unsafe(no_mangle)]
        #[doc(hidden)]
        pub static _MODULE_INFO: mostd::ModuleInfo = mostd::ModuleInfo {
            name: #name,
        };

        #[unsafe(no_mangle)]
        pub extern "C" fn _module_init_() {
            let _: () = #main_fn_name();
        }

        #[expect(unused)]
        #main_fn
    )
    .into()
}
