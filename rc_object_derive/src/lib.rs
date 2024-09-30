use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_attribute]
pub fn kernel_object(input: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);

    let ident = item.ident;
    let data = match item.data {
        Data::Struct(data) => data,
        _ => panic!("Only structs are supported"),
    };

    let fields: Vec<_> = data
        .fields
        .iter()
        .map(|field| {
            // 对每个字段进行映射
            let ident = &field.ident; // 属性名
            let ty = &field.ty; // 属性的类型
            quote::quote!(
                #ident: #ty
            )
        })
        .collect();

    let input = proc_macro2::TokenStream::from(input);

    quote::quote! {


        struct #ident {
            base: crate::object::KObjectBase,
            #(#fields,)*
        }


        impl crate::object::KernelObject for #ident {
            fn id(&self) -> crate::object::KoID {
               self.base.id
            }

            fn type_name(&self) -> &str {
                stringify!(#ident)
            }

            fn name(&self) -> alloc::string::String {
                self.base.name()
            }

            fn set_name(&self, name: &str) {
                self.base.set_name(name)
            }

            #input
        }

        impl core::fmt::Debug for #ident {
            fn fmt(
                &self,
                f: &mut core::fmt::Formatter<'_>,
            ) -> core::result::Result<(), core::fmt::Error> {
                use crate::object::KernelObject;
                // 输出对象类型、ID 和名称
                f.debug_tuple(&stringify!(#ident))
                    .field(&self.id())
                    .field(&self.name())
                    .finish()
            }
        }
    }
    .into()
}
