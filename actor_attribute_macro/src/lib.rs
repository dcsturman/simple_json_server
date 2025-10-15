use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemImpl, ImplItem, ImplItemFn, FnArg, Pat, Type, Visibility};

/// The `#[actor]` attribute macro that implements the Actor trait for a struct.
///
/// This macro should be placed on an `impl` block for a struct. It will:
/// 1. Analyze all public async methods in the impl block
/// 2. Generate message structs for each method's parameters
/// 3. Implement the Actor trait's dispatch method that:
///    - Deserializes JSON messages
///    - Matches method names from the JSON
///    - Calls the appropriate method with deserialized parameters
///    - Serializes and returns the result
///
/// # Example
/// ```rust
/// struct MyActor {
///     state: i32,
/// }
///
/// #[actor]
/// impl MyActor {
///     pub async fn add(&self, a: i32, b: i32) -> i32 {
///         a + b
///     }
///
///     pub async fn get_state(&self) -> i32 {
///         self.state
///     }
/// }
/// ```
///
/// This will generate message structs like:
/// ```rust
/// #[derive(serde::Deserialize)]
/// struct AddMessage {
///     a: i32,
///     b: i32,
/// }
/// ```
///
/// And implement Actor::dispatch to handle JSON like:
/// ```json
/// {"method": "add", "params": {"a": 5, "b": 3}}
/// ```
#[proc_macro_attribute]
pub fn actor(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_impl = parse_macro_input!(input as ItemImpl);

    // Extract the struct type this impl is for
    let struct_type = &input_impl.self_ty;

    // Collect all public async methods
    let mut methods = Vec::new();
    let mut message_structs = Vec::new();
    let mut dispatch_arms = Vec::new();

    for item in &input_impl.items {
        if let ImplItem::Fn(method) = item {
            if is_public_async_method(method) {
                let method_name = &method.sig.ident;
                let method_name_str = method_name.to_string();

                // Extract parameters (excluding &self)
                let params = extract_method_params(method);

                // Generate message struct name
                let message_struct_name = syn::Ident::new(
                    &format!("{}Message", snake_case_to_pascal_case(&method_name_str)),
                    method_name.span()
                );

                // Generate message struct
                if !params.is_empty() {
                    let param_fields: Vec<_> = params.iter().map(|(name, ty)| {
                        quote! { #name: #ty }
                    }).collect();

                    message_structs.push(quote! {
                        #[derive(serde::Deserialize)]
                        struct #message_struct_name {
                            #(#param_fields),*
                        }
                    });
                } else {
                    // For methods with no parameters, create an empty struct
                    message_structs.push(quote! {
                        #[derive(serde::Deserialize)]
                        struct #message_struct_name {}
                    });
                }

                // Generate dispatch arm
                let param_names: Vec<_> = params.iter().map(|(name, _)| name).collect();
                let method_call = if params.is_empty() {
                    quote! { self.#method_name().await }
                } else {
                    quote! { self.#method_name(#(msg_params.#param_names),*).await }
                };

                dispatch_arms.push(quote! {
                    #method_name_str => {
                        let msg_params: #message_struct_name = serde_json::from_value(params)
                            .map_err(|e| format!("Failed to deserialize parameters for {}: {}", #method_name_str, e))?;
                        let result = #method_call;
                        serde_json::to_string(&result)
                            .map_err(|e| format!("Failed to serialize result for {}: {}", #method_name_str, e))
                    }
                });

                methods.push(method);
            }
        }
    }

    // Generate documentation for the Actor implementation
    let doc_string = generate_actor_documentation(&methods, struct_type);

    // Generate the Actor trait implementation
    let actor_impl = quote! {
        #[doc = #doc_string]
        impl crate::Actor for #struct_type {
            fn dispatch(&self, method_name: &str, msg: &str) -> String {
                // Define message structs locally
                #(#message_structs)*

                // Parse the incoming JSON message
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(msg);
                let params = match parsed {
                    Ok(val) => val,
                    Err(e) => return serde_json::to_string(&format!("Failed to parse JSON: {}", e)).unwrap_or_else(|_| "\"JSON parse error\"".to_string()),
                };


                // Create async runtime for calling async methods
                let rt = tokio::runtime::Runtime::new().unwrap();
                let result: Result<String, String> = rt.block_on(async {
                    match method_name {
                        #(#dispatch_arms)*
                        _ => Err(format!("Unknown method: {}", method_name))
                    }
                });

                match result {
                    Ok(json_result) => json_result,
                    Err(error_msg) => serde_json::to_string(&error_msg).unwrap_or_else(|_| "\"Serialization error\"".to_string()),
                }
            }
        }
    };

    // Combine original impl with generated Actor impl
    let expanded = quote! {
        #input_impl

        #actor_impl
    };

    TokenStream::from(expanded)
}

/// Check if a method is public and async
fn is_public_async_method(method: &ImplItemFn) -> bool {
    // Check if method is public
    let is_public = matches!(method.vis, Visibility::Public(_));

    // Check if method is async
    let is_async = method.sig.asyncness.is_some();

    is_public && is_async
}

/// Extract method parameters (excluding &self)
fn extract_method_params(method: &ImplItemFn) -> Vec<(syn::Ident, Type)> {
    let mut params = Vec::new();

    for input in &method.sig.inputs {
        match input {
            FnArg::Receiver(_) => continue, // Skip &self
            FnArg::Typed(pat_type) => {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    params.push((pat_ident.ident.clone(), (*pat_type.ty).clone()));
                }
            }
        }
    }

    params
}

/// Convert snake_case to PascalCase
fn snake_case_to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Generate comprehensive documentation for the Actor implementation
fn generate_actor_documentation(methods: &[&ImplItemFn], struct_type: &syn::Type) -> String {
    let mut doc = String::new();

    // Header
    doc.push_str(&format!("Actor implementation for `{}`.\n\n", quote!(#struct_type)));
    doc.push_str("This implementation provides JSON-based method dispatch for the following methods:\n\n");

    // Method overview table
    doc.push_str("| Method | Parameters | Return Type |\n");
    doc.push_str("|--------|------------|-------------|\n");

    for method in methods {
        let method_name = &method.sig.ident;
        let params = extract_method_params(method);
        let return_type = &method.sig.output;

        let param_str = if params.is_empty() {
            "None".to_string()
        } else {
            params.iter()
                .map(|(name, ty)| format!("`{}`: `{}`", name, quote!(#ty)))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let return_str = match return_type {
            syn::ReturnType::Default => "`()`".to_string(),
            syn::ReturnType::Type(_, ty) => format!("`{}`", quote!(#ty)),
        };

        doc.push_str(&format!("| `{}` | {} | {} |\n", method_name, param_str, return_str));
    }

    // Detailed method documentation
    for method in methods {
        let method_name = &method.sig.ident;
        let method_name_str = method_name.to_string();
        let params = extract_method_params(method);
        let return_type = &method.sig.output;

        doc.push_str("---\n");
        doc.push_str(&format!("# Method `{}`\n\n", method_name));

        // Extract method documentation if available
        if let Some(doc_comment) = extract_method_doc(method) {
            doc.push_str(&format!("{}\n\n", doc_comment));
        }

        // Parameters section
        if params.is_empty() {
            doc.push_str("- **Parameters:** None\n\n");
        } else {
            doc.push_str("- **Parameters:**\n");
            for (name, ty) in &params {
                doc.push_str(&format!("  - `{}`: `{}`\n", name, quote!(#ty)));
            }
            doc.push_str("\n");
        }

        // Return type
        let return_str = match return_type {
            syn::ReturnType::Default => "`()`".to_string(),
            syn::ReturnType::Type(_, ty) => format!("`{}`", quote!(#ty)),
        };
        doc.push_str(&format!("- **Returns:** {}\n\n", return_str));

        // JSON payload example
        doc.push_str("**JSON Payload:**\n");
        doc.push_str("```json\n");
        if params.is_empty() {
            doc.push_str("{}\n");
        } else {
            doc.push_str("{\n");
            for (i, (name, ty)) in params.iter().enumerate() {
                let example_value = generate_example_value(ty);
                let comma = if i == params.len() - 1 { "" } else { "," };
                doc.push_str(&format!("  \"{}\": {}{}\n", name, example_value, comma));
            }
            doc.push_str("}\n");
        }
        doc.push_str("```\n\n");

        // Usage example
        doc.push_str("**Usage Example from Javascript:**\n");
        doc.push_str("```js\n");
        if params.is_empty() {
            doc.push_str(&format!("result = await fetch(\"http://localhost:9000/{}\");\n", method_name_str));
        } else {
            doc.push_str(&format!("result = await fetch(\"http://localhost:9000/{}\", {{ body: JSON.stringify(", method_name_str));
            if params.len() == 1 {
                let (name, ty) = &params[0];
                let example_value = generate_example_value(ty);
                doc.push_str(&format!("{{{}: {}}}", name, example_value));
            } else {
                doc.push_str("{\n");
                for (i, (name, ty)) in params.iter().enumerate() {
                    let example_value = generate_example_value(ty);
                    let comma = if i == params.len() - 1 { "" } else { "," };
                    doc.push_str(&format!("  {}: {}{}\n", name, example_value, comma));
                }
                doc.push_str("}");
            }
            doc.push_str(")});\n");
        }
        doc.push_str("```\n\n");
    }

    doc
}

/// Extract documentation comments from a method
fn extract_method_doc(method: &ImplItemFn) -> Option<String> {
    let mut doc_lines = Vec::new();

    for attr in &method.attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), .. }) = &meta.value {
                    let doc_text = lit_str.value();
                    // Remove leading space that rustdoc adds
                    let trimmed = if doc_text.starts_with(' ') {
                        &doc_text[1..]
                    } else {
                        &doc_text
                    };
                    doc_lines.push(trimmed.to_string());
                }
            }
        }
    }

    if doc_lines.is_empty() {
        None
    } else {
        Some(doc_lines.join("\n"))
    }
}

/// Generate example values for different types
fn generate_example_value(ty: &Type) -> String {
    let type_str = quote!(#ty).to_string();

    match type_str.as_str() {
        "i32" | "i64" | "i8" | "i16" | "isize" => "42".to_string(),
        "u32" | "u64" | "u8" | "u16" | "usize" => "42".to_string(),
        "f32" | "f64" => "3.14".to_string(),
        "bool" => "true".to_string(),
        "String" => "\"example\"".to_string(),
        "char" => "'x'".to_string(),
        s if s.starts_with("Option") => "null".to_string(),
        s if s.starts_with("Vec") => "[]".to_string(),
        s if s.contains("HashMap") || s.contains("BTreeMap") => "{}".to_string(),
        _ => {
            // For custom types, try to provide a reasonable default
            if type_str.contains("String") {
                "\"example\"".to_string()
            } else if type_str.contains("i32") || type_str.contains("i64") {
                "42".to_string()
            } else if type_str.contains("f32") || type_str.contains("f64") {
                "3.14".to_string()
            } else if type_str.contains("bool") {
                "true".to_string()
            } else {
                "\"value\"".to_string()
            }
        }
    }
}