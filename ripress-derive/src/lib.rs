use proc_macro::TokenStream;

/// A derive macro for automatically implementing the `FromParams` trait.
///
/// This macro can be applied to structs with named fields to automatically
/// generate an implementation of `FromParams` that extracts route parameters
/// and parses them into the struct fields.
///
/// # Usage
///
/// ```rust,ignore
/// use ripress::req::route_params::FromParams;
/// use ripress_derive::FromParams;
///
/// #[derive(FromParams)]
/// struct UserParams {
///     id: i32,
///     name: String,
/// }
/// ```
///
/// This will generate an implementation that extracts `id` and `name` from
/// the route parameters and parses them into the appropriate types.
#[proc_macro_derive(FromParams)]
pub fn from_params_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens of the type the macro is applied to
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let struct_name = &ast.ident;

    // Only support struct with named fields
    let fields = match ast.data {
        syn::Data::Struct(ref s) => match &s.fields {
            syn::Fields::Named(named) => &named.named,
            _ => {
                return syn::Error::new_spanned(
                    struct_name,
                    "FromParams can only be derived for structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                struct_name,
                "FromParams can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate parsing and assignment for each struct field
    let assigns = fields.iter().filter_map(|f| {
        f.ident.as_ref().map(|ident| {
            let ident_str = ident.to_string();
            quote::quote! {
                let #ident = p[#ident_str].parse()
                    .map_err(|e| format!("Failed to parse field '{}': {}", #ident_str, e))?;
            }
        })
    });

    let field_names = fields.iter().filter_map(|f| {
        f.ident.as_ref().map(|ident| {
            quote::quote! { #ident }
        })
    });

    let expanded = quote::quote! {
        impl ::ripress::req::route_params::FromParams for #struct_name {
            fn from_params(p: &::ripress::req::route_params::RouteParams) -> Result<Self, String> {
                #(#assigns)*
                Ok(Self {
                    #(#field_names,)*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

/// A derive macro for automatically implementing the `FromJson` trait.
///
/// This macro can be applied to structs that implement `Deserialize` from serde
/// to automatically generate an implementation of `FromJson` that extracts JSON
/// from the request body and deserializes it.
///
/// # Usage
///
/// ```rust,ignore
/// use ripress::req::body::json_data::FromJson;
/// use ripress_derive::FromJson;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, FromJson)]
/// struct UserData {
///     name: String,
///     email: String,
/// }
/// ```
///
/// This will generate an implementation that extracts JSON from the request body
/// and deserializes it into the struct. The struct must also derive `Deserialize`
/// from serde.
#[proc_macro_derive(FromJson)]
pub fn from_json_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens of the type the macro is applied to
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let struct_name = &ast.ident;

    let expanded = quote::quote! {
        impl ::ripress::req::body::json_data::FromJson for #struct_name {
            fn from_json(data: &::ripress::req::body::RequestBodyContent) -> Result<Self, String> {
                if let ::ripress::req::body::RequestBodyContent::JSON(json_value) = data {
                    serde_json::from_value::<Self>(json_value.clone())
                        .map_err(|e| format!("Failed to deserialize JSON: {}", e))
                } else {
                    Err("Request body is not JSON".to_string())
                }
            }
        }
    };

    TokenStream::from(expanded)
}
