use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::Data;
use syn::DataStruct;
use syn::DataEnum;

struct FieldInfo {
    ident: syn::Ident,
    ty: syn::Type,
}

struct VariantInfo {
    ident: syn::Ident,
    ty: syn::Fields,
}

#[proc_macro_derive(Reflect)]
pub fn derive_reflect(token_stream: TokenStream) -> TokenStream {
    let item: syn::DeriveInput = parse_macro_input!(token_stream);
    let ident = item.ident;
    let (value, _type) = match item.data {
        Data::Struct(info) => {
            derive_data_struct(ident.clone(), info)
        }

        Data::Enum(info) => {
            derive_data_enum(ident.clone(), info)
        }
        Data::Union(_info) => {
            unimplemented!()
        }
    };

    let output = quote!(
        impl reflect::Reflect for #ident {
            fn to_value(&self) ->  reflect::Value {
                #value
            }
            
            fn to_type() -> reflect::Type {
                #_type
            }
        }
    );

    TokenStream::from(output)
}

fn derive_data_struct(ident_struct: syn::Ident, info_struct: DataStruct) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let ident_struct_string = ident_struct.to_string();
    let mut fields = Vec::new();
    for field in info_struct.fields {
        let ident = field.ident.unwrap();
        let ty = desugar_type(field.ty);
        fields.push(
            FieldInfo {
                ident: ident,
                ty: ty,
            }
        );
    }

    let mut fields_type = Vec::new();
    for field_info in fields.iter() {
        let ident_string = field_info.ident.to_string();
        let ty = desugar_type(field_info.ty.clone());
        fields_type.push(
            quote! {
                reflect::StructField {
                    name: #ident_string,
                    field_type: reflect::TypeGet::from::<#ty>()
                }
            }
        );
    }

    let mut fields_values = Vec::new();
    for (field_info, token_type) in fields.iter().zip(fields_type.clone()) {
        let field_name = field_info.ident.clone();
        fields_values.push(
            quote! {
                reflect::StructFieldValue {
                    info: #token_type,
                    value: self.#field_name.to_value()
                }
            }
        );
    }

    let struct_type = quote!(
        reflect::StructType {
            id: std::any::TypeId::of::<#ident_struct>(),
            name: #ident_struct_string,
            fields: vec![#(#fields_type),*],
        }
    );

    (
        quote!(
            reflect::Value::Struct(
                reflect::StructValue {
                    info: #struct_type,
                    fields: vec![#(#fields_values),*]
                }
            )
        )
        ,
        quote!(
            reflect::Type::Struct(
                reflect::StructType {
                    id: std::any::TypeId::of::<#ident_struct>(),
                    name: #ident_struct_string,
                    fields: vec![#(#fields_type),*],
                }
            )
        )
    )
}

fn derive_data_enum(ident_enum: syn::Ident, info_enum: DataEnum) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let ident_string = ident_enum.to_string();
    let mut variants_info = Vec::new();
    for variant in info_enum.variants {
        let ident = variant.ident;
        let ty = variant.fields;
        variants_info.push(VariantInfo {
            ident: ident,
            ty: ty,
        });
    }
    
    let mut variants_type = Vec::new();
    for variant in variants_info.iter() {
        let ty;
        let ident_string = variant.ident.to_string();
        match &variant.ty {
            syn::Fields::Named(_fields) => {
                unimplemented!("NamedFields")
            }
            syn::Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    let _type = fields.unnamed.first().unwrap().ty.clone();
                    let _type = desugar_type(_type);
                    ty = Some(quote! { reflect::TypeGet::from::<#_type>()});
                } else {
                    let mut types = Vec::new();
                    for field in fields.unnamed.iter() {
                        let _type = field.ty.clone();
                        let _type = desugar_type(_type);
                        types.push( quote! { reflect::TypeGet::from::<#_type>()} );
                    }
                    ty = Some(quote! {
                        reflect::TypeGet::from_type(
                            reflect::Type::Tuple(
                                vec![#(#types),*]
                            )
                        )
                    });
                }
            }
            syn::Fields::Unit => { ty = Some(quote! {reflect::TypeGet::from_type(reflect::Type::Unit)} ); }
        }

        let ty = ty.unwrap();

        let variant_type = quote! {
            reflect::EnumVariant {
                variant_name: #ident_string,
                variant_type: #ty,
            }
        };
        variants_type.push(variant_type);
    }

    
    let mut variants_values_potential = Vec::new();
    for (variant_info, token_type) in variants_info.iter().zip(variants_type.iter()) {
        let variant_name = variant_info.ident.clone();
        match &variant_info.ty {
            syn::Fields::Unit => {
                variants_values_potential.push(
                    quote!(
                        if let #ident_enum::#variant_name = self {
                            reflect::EnumVariantValue {
                                info: #token_type,
                                value: reflect::Value::Unit
                            }
                        }
                    )
                );
            }
            syn::Fields::Unnamed(_fields) => {
                let values: Vec<proc_macro2::Ident> = (0..variant_info.ty.len()).map(|i| { quote::format_ident!("value{}", i) } ).collect();
                variants_values_potential.push(
                    quote!(
                        if let #ident_enum::#variant_name(#(#values),*) = self {
                            reflect::EnumVariantValue {
                                info: #token_type,
                                value: (#(#values),*).to_value()
                            }
                        }
                    )
                );
                /*
                if variant_info.ty.len() == 1 {
                    variants_values_potential.push(
                        quote!(
                            if let #ident_enum::#variant_name(value) = self {
                                reflect::EnumVariantValue {
                                    info: #token_type,
                                    value: value.to_value()
                                }
                            }
                        )
                    );
                } else {
                    variants_values_potential.push(
                        quote!(
                            if let #ident_enum::#variant_name(value1, value2) = self {
                                reflect::EnumVariantValue {
                                    info: #token_type,
                                    value: (value1, value2).to_value()
                                }
                            }
                        )
                    );
                }
                */
            }
            _ => {
                unimplemented!()
            }
        }
    }

    let enum_type = quote!(
        reflect::EnumType {
            id: std::any::TypeId::of::<#ident_enum>(),
            enum_name: #ident_string,
            variants: vec![#(#variants_type),*],
        }
    );

    (   
        
        quote!(
            reflect::Value::Enum(Box::new(
                reflect::EnumValue {
                    info: #enum_type,
                    variant: #(#variants_values_potential) else * else { panic!() },
                }
            ))
        )
        ,
        quote!(
            reflect::Type::Enum(Box::new(
                reflect::EnumType {
                    id: std::any::TypeId::of::<#ident_enum>(),
                    enum_name: #ident_string,
                    variants: vec![#(#variants_type),*],
                }
            ))
        )
    )
}


fn desugar_type(ty: syn::Type) -> syn::Type {
    match ty {
        /*
        syn::Type::Reference(reference) => {
            *reference.elem
        }
        */
        _ => { ty }
    }
}