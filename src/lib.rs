extern crate proc_macro;

use convert_case::{Case, Casing};
use lazy_static::lazy_static;
use mdmodels::datamodel::DataModel;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::{BTreeMap, HashMap};
use std::{error::Error, path::Path};
use syn::{parse_macro_input, LitStr};

// Static variables
const FORBIDDEN_NAMES: [&str; 9] = [
    "type", "struct", "enum", "use", "crate", "mod", "fn", "impl", "trait",
];

// Lazy static initialization for type mappings
lazy_static! {
    static ref TYPE_MAPPINGS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("integer", "i32");
        m.insert("float", "f32");
        m.insert("string", "String");
        m.insert("boolean", "bool");
        m
    };
}

/// Procedural macro to generate structs from markdown models
///
/// # Arguments
/// * `input` - A TokenStream representing the input markdown file path
///
/// # Returns
/// A TokenStream containing the generated Rust code for the structs and enums
#[proc_macro]
pub fn parse_mdmodel(input: TokenStream) -> TokenStream {
    // Get the current working directory
    let dir = std::env::var("CARGO_MANIFEST_DIR").map_or_else(
        |_| std::env::current_dir().unwrap(),
        |s| Path::new(&s).to_path_buf(),
    );

    // Parse the input TokenStream as a literal string
    let input = parse_macro_input!(input as LitStr).value();
    let path = dir.join(input);

    // Parse the DataModel from the specified path
    let model = DataModel::from_markdown(&path)
        .unwrap_or_else(|_| panic!("Failed to parse the markdown model at path: {:?}", path));
    let model_name = syn::Ident::new(
        &to_snake(model.name.unwrap_or("model".to_string())),
        proc_macro2::Span::call_site(),
    );
    let mut structs = vec![];

    // Iterate through the objects in the model
    for object in model.objects {
        if is_reserved(&object.name) {
            panic!("Reserved keyword used as object name: {}", object.name);
        }

        let struct_name = syn::Ident::new(&object.name, proc_macro2::Span::call_site());
        let mut fields = vec![];
        let mut getters = vec![];
        let mut setters = vec![];

        // Iterate through the attributes of each object
        for attribute in object.attributes {
            let field_name = syn::Ident::new(&attribute.name, proc_macro2::Span::call_site());
            let field_type = get_data_type(&attribute.dtypes[0])
                .unwrap_or_else(|_| panic!("Unknown data type: {}", attribute.dtypes[0]));
            let wrapped_type = wrap_dtype(attribute.is_array, attribute.required, field_type);
            let builder_attr =
                get_builder_attr(attribute.is_array, attribute.required, &attribute.name);
            let serde_attr = get_serde_attr(attribute.is_array, attribute.required);

            fields.push(quote! {
                #builder_attr
                #serde_attr
                pub #field_name: #wrapped_type
            });

            let getter_name = syn::Ident::new(
                format!("get_{}", attribute.name).as_str(),
                proc_macro2::Span::call_site(),
            );

            let setter_name = syn::Ident::new(
                format!("set_{}", attribute.name).as_str(),
                proc_macro2::Span::call_site(),
            );

            getters.push(quote! {
                pub fn #getter_name(&self) -> &#wrapped_type {
                    &self.#field_name
                }
            });

            setters.push(quote! {
                pub fn #setter_name(&mut self, value: #wrapped_type) -> &mut Self {
                    self.#field_name = value;
                    self
                }
            });
        }

        // Generate the struct definition with pyclass and constructor
        let struct_def = quote! {
            #[derive(Builder, Debug, Clone, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
            pub struct #struct_name {
                #(#fields),*
            }

            impl #struct_name {
                #(#getters)*
                #(#setters)*
            }
        };

        structs.push(struct_def);
    }

    // Iterate through enumerations
    let mut enums = vec![];
    for enum_ in model.enums {
        if is_reserved(&enum_.name) {
            panic!("Reserved keyword used as enum name: {}", enum_.name);
        }
        enums.push(generate_enum(&enum_.mappings, &enum_.name))
    }

    // Combine all generated structs into a single TokenStream
    let expanded = quote! {
        pub mod #model_name {
            use derive_builder::Builder;
            use std::error::Error;

            #(#structs)*
            #(#enums)*
        }
    };

    TokenStream::from(expanded)
}

/// Enumeration for data types
enum DataTypes {
    BaseType(syn::Type),
    ComplexType(syn::Ident),
}

/// Function to get the data type from the type mappings
///
/// # Arguments
/// * `dtype` - A string slice representing the data type
///
/// # Returns
/// A Result containing either a DataTypes enum or an error
fn get_data_type(dtype: &str) -> Result<DataTypes, Box<dyn Error>> {
    match TYPE_MAPPINGS.get(dtype) {
        Some(t) => {
            let field_type: syn::Type = syn::parse_str(t)?;
            Ok(DataTypes::BaseType(field_type))
        }
        None => {
            let field_type: syn::Ident = syn::Ident::new(dtype, proc_macro2::Span::call_site());
            Ok(DataTypes::ComplexType(field_type))
        }
    }
}

/// Function to wrap data types based on their properties (array, required)
///
/// # Arguments
/// * `is_array` - A boolean indicating if the type is an array
/// * `required` - A boolean indicating if the type is required
/// * `dtype` - A DataTypes enum representing the data type
///
/// # Returns
/// A TokenStream representing the wrapped data type
fn wrap_dtype(is_array: bool, required: bool, dtype: DataTypes) -> proc_macro2::TokenStream {
    match dtype {
        DataTypes::BaseType(base_type) => {
            if required && !is_array {
                quote! { #base_type }
            } else if !required && !is_array {
                quote! { Option<#base_type> }
            } else if required && is_array {
                quote! { Vec<#base_type> }
            } else {
                quote! { Option<Vec<#base_type>> }
            }
        }
        DataTypes::ComplexType(complex_type) => {
            if required && !is_array {
                quote! { #complex_type }
            } else if !required && !is_array {
                quote! { Option<#complex_type> }
            } else {
                quote! { Vec<#complex_type> }
            }
        }
    }
}

/// Function to generate builder attributes for struct fields
///
/// # Arguments
/// * `is_array` - A boolean indicating if the field is an array
/// * `required` - A boolean indicating if the field is required
/// * `name` - A string slice representing the field name
///
/// # Returns
/// A TokenStream representing the builder attributes
fn get_builder_attr(is_array: bool, required: bool, name: &str) -> proc_macro2::TokenStream {
    let mut setter_args = vec![];

    if !required {
        setter_args.push(quote! { strip_option });
    }

    if is_array {
        let add_name = syn::Ident::new(&format!("to_{}", name), proc_macro2::Span::call_site());
        setter_args.push(quote! { each(name = #add_name, into) });
    }

    let setter_args = quote! { #(#setter_args),* };

    quote! {
        #[builder(default, setter(into, #setter_args))]
    }
}

/// Function to generate serde attributes for struct fields
///
/// # Arguments
/// * `is_array` - A boolean indicating if the field is an array
/// * `required` - A boolean indicating if the field is required
///
/// # Returns
/// A TokenStream representing the serde attributes
fn get_serde_attr(is_array: bool, required: bool) -> proc_macro2::TokenStream {
    if !required && !is_array {
        quote! { #[serde(skip_serializing_if = "Option::is_none")] }
    } else if is_array {
        quote! { #[serde(default)] }
    } else {
        quote! {}
    }
}

/// Function to generate Rust code for enums
///
/// # Arguments
/// * `mappings` - A reference to a BTreeMap of enum variant mappings
/// * `name` - A string slice representing the enum name
///
/// # Returns
/// A TokenStream containing the generated enum code
fn generate_enum(mappings: &BTreeMap<String, String>, name: &str) -> proc_macro2::TokenStream {
    let enum_name = syn::Ident::new(name, proc_macro2::Span::call_site());
    let mut variants = vec![];
    let mut values = vec![];
    let mut index = 0;

    for (key, value) in mappings {
        let variant_name = syn::Ident::new(&to_camel(key), proc_macro2::Span::call_site());
        let variant_value = syn::LitStr::new(value, proc_macro2::Span::call_site());

        if index == 0 {
            variants.push(quote! {
                #[default]
                #variant_name
            });
            index += 1;
        } else {
            variants.push(quote! {
                #variant_name
            });
        }

        values.push(quote! {
            #enum_name::#variant_name => #variant_value.to_string()
        });
    }

    quote! {
        #[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
        pub enum #enum_name {
            #(#variants),*
        }

        impl std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let s = match self {
                    #(#values),*,
                };
                write!(f, "{}", s)
            }
        }
    }
}

/// Checks if an object or enum name is a reserved keyword
fn is_reserved(name: &str) -> bool {
    FORBIDDEN_NAMES.contains(&name)
}

/// Function to convert a string to snake case
fn to_snake(name: String) -> String {
    name.to_case(Case::Snake)
}

/// Function to convert a string to upper camel case
fn to_camel(name: &str) -> String {
    name.to_case(Case::UpperCamel)
}
