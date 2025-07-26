use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ItemStatic};

#[proc_macro_attribute]
pub fn global_allocator(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let global_allocator_item = parse_macro_input!(item as ItemStatic);
    let global_allocator_name = &global_allocator_item.ident;
    
    quote!(
        #[used]
        #[unsafe(no_mangle)]
        static __ZODIAC_GLOBAL_ALLOCATOR: &'static dyn ::zodiac::mem::Allocator = &#global_allocator_name;

        #global_allocator_item
    )
    .into()
}

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let main_fn = parse_macro_input!(item as ItemFn);
    let main_fn_name = &main_fn.sig.ident;

    quote!(
        #[unsafe(no_mangle)]
        extern "Rust" fn __zodiac_main() -> ! {
            let _: () = #main_fn_name();

            loop {}
        }

        #[expect(unused)]
        #main_fn
    )
    .into()
}

#[proc_macro_attribute]
pub fn panic_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let handler_fn = parse_macro_input!(item as ItemFn);
    let handler_fn_name = &handler_fn.sig.ident;

    quote!(
        #[unsafe(no_mangle)]
        extern "Rust" fn __zodiac_panic_handler(info: &core::panic::PanicInfo) -> ! {
            #handler_fn_name(info);
        }

        #[expect(unused)]
        #handler_fn
    )
    .into()
}
