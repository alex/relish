use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Meta, parse_macro_input};

#[proc_macro_derive(Relish, attributes(relish))]
pub fn derive_relish(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    derive_relish_expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn derive_relish_expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;

    match &input.data {
        Data::Struct(data_struct) => impl_relish_struct(name, data_struct),
        Data::Enum(data_enum) => impl_relish_enum(name, data_enum),
        Data::Union(_) => Err(syn::Error::new_spanned(
            name,
            "Union types are not supported",
        )),
    }
}

fn impl_relish_struct(
    name: &syn::Ident,
    data: &syn::DataStruct,
) -> syn::Result<proc_macro2::TokenStream> {
    let fields = match &data.fields {
        Fields::Named(fields) => &fields.named,
        _ => {
            return Err(syn::Error::new_spanned(
                name,
                "Only named fields are supported",
            ));
        }
    };

    let mut field_info = Vec::new();
    let mut skipped_fields = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        let mut field_id = None;
        let mut skip = false;

        for attr in &field.attrs {
            if attr.path().is_ident("relish")
                && let Meta::List(meta_list) = &attr.meta
            {
                let tokens = &meta_list.tokens;
                let tokens_str = tokens.to_string();

                if let Some(id_str) = tokens_str.strip_prefix("field_id =") {
                    let id_str = id_str.trim();
                    let id: u8 = id_str.parse().map_err(|_| {
                        syn::Error::new_spanned(attr, "field_id must be a valid u8")
                    })?;
                    field_id = Some(id);
                } else if tokens_str == "skip" {
                    skip = true;
                }
            }
        }

        if !skip && field_id.is_none() {
            return Err(syn::Error::new_spanned(
                field_name,
                format!(
                    "Field '{field_name}' must have either #[relish(field_id = ...)] or #[relish(skip)]",
                ),
            ));
        }

        if skip {
            skipped_fields.push((field_name.clone(), field_ty.clone()));
        } else {
            field_info.push((field_name.clone(), field_ty.clone(), field_id.unwrap()));
        }
    }

    field_info.sort_by_key(|(_, _, id)| *id);

    for window in field_info.windows(2) {
        if window[0].2 == window[1].2 {
            return Err(syn::Error::new_spanned(
                name,
                format!("Duplicate field_id: {}", window[0].2),
            ));
        }
    }

    let parse_field_reads: Vec<_> = field_info
        .iter()
        .map(|(name, ty, id)| {
            quote! {
                let #name = parser.read_value_for_field_id::<#ty>(#id)?;
            }
        })
        .collect();

    let field_names_write: Vec<_> = field_info.iter().map(|(name, _, _)| name.clone()).collect();
    let field_types_write: Vec<_> = field_info.iter().map(|(_, ty, _)| ty.clone()).collect();
    let field_ids_write: Vec<_> = field_info.iter().map(|(_, _, id)| *id).collect();

    let field_names_len: Vec<_> = field_info.iter().map(|(name, _, _)| name.clone()).collect();

    let field_from_option = field_info.iter().map(|(name, _ty, _)| {
        quote! {
            #name: relish::FieldValue::from_option(#name)?
        }
    });

    let skipped_field_init = skipped_fields.iter().map(|(name, _ty)| {
        quote! {
            #name: Default::default()
        }
    });

    let expanded = quote! {
        impl relish::Relish for #name {
            const TYPE: relish::TypeId = relish::TypeId::Struct;

            fn parse_value(data: &mut relish::BytesRef) -> relish::ParseResult<Self> {
                let mut parser = relish::StructParser::new(data);
                #(#parse_field_reads)*
                parser.finish()?;

                Ok(Self {
                    #(#field_from_option,)*
                    #(#skipped_field_init),*
                })
            }

            fn write_value(&self, buffer: &mut Vec<u8>) -> relish::WriteResult<()> {
                let mut content_len = 0;
                #(
                    if let Some(value) = relish::FieldValue::as_relish(&self.#field_names_write) {
                        content_len += 1 + 1;
                        content_len += value.value_length();
                    }
                )*

                relish::write_tagged_varint_length(buffer, content_len)?;

                #(
                    if let Some(value) = relish::FieldValue::as_relish(&self.#field_names_write) {
                        buffer.push(#field_ids_write);
                        buffer.push(<#field_types_write as relish::FieldValue>::T::TYPE as u8);
                        value.write_value(buffer)?;
                    }
                )*

                Ok(())
            }

            fn value_length(&self) -> usize {
                let mut content_size = 0;
                #(
                    if let Some(value) = relish::FieldValue::as_relish(&self.#field_names_len) {
                        content_size += 1;
                        content_size += 1 + value.value_length();
                    }
                )*
                relish::tagged_varint_length_size(content_size) + content_size
            }
        }
    };

    Ok(expanded)
}

fn impl_relish_enum(
    name: &syn::Ident,
    data: &syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut variant_info = Vec::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;

        let mut field_id = None;

        for attr in &variant.attrs {
            if attr.path().is_ident("relish")
                && let Meta::List(meta_list) = &attr.meta
            {
                let tokens = &meta_list.tokens;
                let tokens_str = tokens.to_string();

                if let Some(id_str) = tokens_str.strip_prefix("field_id =") {
                    let id_str = id_str.trim();
                    let id: u8 = id_str.parse().map_err(|_| {
                        syn::Error::new_spanned(attr, "field_id must be a valid u8")
                    })?;
                    field_id = Some(id);
                }
            }
        }

        if field_id.is_none() {
            return Err(syn::Error::new_spanned(
                variant_name,
                format!("Variant '{variant_name}' must have #[relish(field_id = ...)]"),
            ));
        }

        match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let field_ty = &fields.unnamed.first().unwrap().ty;
                variant_info.push((variant_name.clone(), field_ty.clone(), field_id.unwrap()));
            }
            Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    variant_name,
                    "Unit variants are not supported. Each variant must contain exactly one field",
                ));
            }
            Fields::Named(_) => {
                return Err(syn::Error::new_spanned(
                    variant_name,
                    "Named fields are not supported. Each variant must contain exactly one unnamed field",
                ));
            }
            Fields::Unnamed(fields) => {
                return Err(syn::Error::new_spanned(
                    variant_name,
                    format!(
                        "Variant must have exactly one field, found {}",
                        fields.unnamed.len()
                    ),
                ));
            }
        }
    }

    variant_info.sort_by_key(|(_, _, id)| *id);

    for window in variant_info.windows(2) {
        if window[0].2 == window[1].2 {
            return Err(syn::Error::new_spanned(
                name,
                format!("Duplicate field_id: {}", window[0].2),
            ));
        }
    }

    let parse_variants = variant_info.iter().map(|(variant_name, ty, id)| {
        quote! {
            #id => {
                Self::#variant_name(relish::parse_tlv::<#ty>(data)?)
            }
        }
    });

    let write_variants: Vec<_> = variant_info
        .iter()
        .map(|(variant_name, ty, id)| {
            quote! {
                Self::#variant_name(value) => {
                    buffer.push(#id);
                    buffer.push(<#ty as relish::Relish>::TYPE as u8);
                    value.write_value(buffer)?;
                }
            }
        })
        .collect();

    let length_variants: Vec<_> = variant_info
        .iter()
        .map(|(variant_name, _ty, _id)| {
            quote! {
                Self::#variant_name(value) => {
                    1 + 1 + value.value_length()
                }
            }
        })
        .collect();

    let expanded = quote! {
        impl relish::Relish for #name {
            const TYPE: relish::TypeId = relish::TypeId::Enum;

            fn parse_value(data: &mut relish::BytesRef) -> relish::ParseResult<Self> {
                let field_id = relish::read_byte(data)?;
                if field_id & 0x80 != 0 {
                    return Err(relish::ParseError::new(
                        relish::ParseErrorKind::InvalidFieldId(field_id)
                    ));
                }

                let result = match field_id {
                    #(#parse_variants)*
                    _ => return Err(relish::ParseError::new(
                        relish::ParseErrorKind::UnknownVariant(field_id)
                    ))
                };

                if !data.is_empty() {
                    return Err(relish::ParseError::new(
                        relish::ParseErrorKind::ExtraData {
                            bytes_remaining: data.len()
                        }
                    ));
                }

                Ok(result)
            }

            fn write_value(&self, buffer: &mut Vec<u8>) -> relish::WriteResult<()> {
                let content_len = match self {
                    #(#length_variants)*
                };

                relish::write_tagged_varint_length(buffer, content_len)?;

                match self {
                    #(#write_variants)*
                }

                Ok(())
            }

            fn value_length(&self) -> usize {
                let content_size = match self {
                    #(#length_variants)*
                };
                relish::tagged_varint_length_size(content_size) + content_size
            }
        }
    };

    Ok(expanded)
}
