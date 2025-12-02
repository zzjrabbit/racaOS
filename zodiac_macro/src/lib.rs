use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Data, DataStruct, DataUnion, DeriveInput, Fields, Generics, Ident, ItemFn,
    ItemStatic, parse_macro_input,
};

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

#[proc_macro_derive(Pod)]
pub fn derive_pod(input_token: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input_token as DeriveInput);
    expand_derive_pod(input).into()
}

const ALLOWED_REPRS: [&'static str; 11] = [
    "C", "u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "usize", "isize",
];

fn expand_derive_pod(input: DeriveInput) -> proc_macro2::TokenStream {
    let attrs = input.attrs;
    let ident = input.ident;
    let generics = input.generics;
    match input.data {
        Data::Struct(data_struct) => impl_pod_for_struct(data_struct, generics, ident, attrs),
        Data::Union(data_union) => impl_pod_for_union(data_union, generics, ident, attrs),
        Data::Enum(_) => panic!(
            "Trying to derive `Pod` trait for enum may be unsound. Use `TryFromInt` instead."
        ),
    }
}

fn impl_pod_for_struct(
    data_struct: DataStruct,
    generics: Generics,
    ident: Ident,
    attrs: Vec<Attribute>,
) -> proc_macro2::TokenStream {
    if !has_valid_repr(attrs) {
        panic!("{} has invalid repr to implement Pod", ident.to_string());
    }
    let DataStruct { fields, .. } = data_struct;
    let fields = match fields {
        Fields::Named(fields_named) => fields_named.named,
        Fields::Unnamed(fields_unnamed) => fields_unnamed.unnamed,
        Fields::Unit => panic!("derive pod does not work for struct with unit field"),
    };

    // deal with generics
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let trait_tokens = pod_trait_tokens();

    let pod_where_predicates = fields
        .into_iter()
        .map(|field| {
            let field_ty = field.ty;
            quote! {
                #field_ty: #trait_tokens
            }
        })
        .collect::<Vec<_>>();

    // if where_clause is none, we should add a `where` word manually.
    if where_clause.is_none() {
        quote! {
            #[automatically_derived]
            unsafe impl #impl_generics #trait_tokens for #ident #type_generics where #(#pod_where_predicates),* {}
        }
    } else {
        quote! {
            #[automatically_derived]
            unsafe impl #impl_generics #trait_tokens for #ident #type_generics #where_clause, #(#pod_where_predicates),* {}
        }
    }
}

fn impl_pod_for_union(
    data_union: DataUnion,
    generics: Generics,
    ident: Ident,
    attrs: Vec<Attribute>,
) -> proc_macro2::TokenStream {
    if !has_valid_repr(attrs) {
        panic!("{} has invalid repr to implement Pod", ident.to_string());
    }
    let fields = data_union.fields.named;
    // deal with generics
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let trait_tokens = pod_trait_tokens();
    let pod_where_predicates = fields
        .into_iter()
        .map(|field| {
            let field_ty = field.ty;
            quote! {
                #field_ty: #trait_tokens
            }
        })
        .collect::<Vec<_>>();

    // if where_clause is none, we should add a `where` word manually.
    if where_clause.is_none() {
        quote! {
            #[automatically_derived]
            unsafe impl #impl_generics #trait_tokens for #ident #type_generics where #(#pod_where_predicates),* {}
        }
    } else {
        quote! {
            #[automatically_derived]
            unsafe impl #impl_generics #trait_tokens for #ident #type_generics #where_clause, #(#pod_where_predicates),* {}
        }
    }
}

fn has_valid_repr(attrs: Vec<Attribute>) -> bool {
    for attr in attrs {
        if let Some(ident) = attr.path().get_ident() {
            if "repr" == ident.to_string().as_str() {
                let repr = attr.to_token_stream().to_string();
                let repr = repr.replace("(", "").replace(")", "");
                let reprs = repr
                    .split(",")
                    .map(|one_repr| one_repr.trim())
                    .collect::<Vec<_>>();
                if let Some(_) = ALLOWED_REPRS.iter().position(|allowed_repr| {
                    reprs
                        .iter()
                        .position(|one_repr| one_repr == allowed_repr)
                        .is_some()
                }) {
                    return true;
                }
            }
        }
    }
    false
}

fn pod_trait_tokens() -> proc_macro2::TokenStream {
    let package_name = std::env::var("CARGO_PKG_NAME").unwrap();

    // Only `ostd` and the unit test in `ostd-pod` depend on `Pod` fro `ostd-pod`,
    // other crates depend on `Pod` re-exported from ostd.
    if package_name.as_str() == "ostd" || package_name.as_str() == "ostd-pod" {
        quote! {
            ::ostd_pod::Pod
        }
    } else {
        quote! {
            ::ostd::Pod
        }
    }
}
