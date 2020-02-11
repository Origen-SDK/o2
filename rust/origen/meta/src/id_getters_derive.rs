use crate::proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn;

#[derive(Default)]
struct Config {
    pub getter_type: String,
    pub field: String,
    pub field_container_name: String,
    pub parent_field: String,
    pub return_type: String,
}

pub fn impl_id_getters(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
  let name = &ast.ident;
  let mut getter_functions = quote! {};
  for (_i, attr) in ast.attrs.iter().enumerate() {
      // Go through the given attributes. If something doesn't parse, just ignore it. The Rust language will eithe catch for something
      // grossly malformed, or it may be a (very) customized attribute.
      // Likewise, ignore any atttributes that aren't ours.
      match attr.parse_meta()? {
        syn::Meta::List(syn::MetaList {ref path, ref nested, ..}) => {
            if path.is_ident("id_getters_by_index") || path.is_ident("id_getters_by_mapping") {
                let mut config = Config::default();
                if path.is_ident("id_getters_by_index") {
                    config.getter_type = "by_index".to_string();
                } else {
                    config.getter_type = "by_mapping".to_string();
                }
                for (_i, _attr) in nested.iter().enumerate() {
                    match _attr {
                        syn::NestedMeta::Meta(syn::Meta::NameValue(ref name_value_pair)) => {
                            // Match the name/value pair to the appropriate field in the config.
                            if name_value_pair.path.is_ident("field") {
                                match &name_value_pair.lit {
                                    syn::Lit::Str(l) => config.field = l.value(),
                                    _ => return Err(syn::Error::new_spanned(name_value_pair.lit.clone(), format!("Could not process name-value pair's value as a String")))
                                }
                            } else if name_value_pair.path.is_ident("parent_field") {
                                match &name_value_pair.lit {
                                    syn::Lit::Str(l) => config.parent_field = l.value(),
                                    _ => return Err(syn::Error::new_spanned(name_value_pair.lit.clone(), format!("Could not process name-value pair's value as a String")))
                                }
                            } else if name_value_pair.path.is_ident("field_container_name") {
                                match &name_value_pair.lit {
                                    syn::Lit::Str(l) => config.field_container_name = l.value(),
                                    _ => return Err(syn::Error::new_spanned(name_value_pair.lit.clone(), format!("Could not process name-value pair's value as a String")))
                                }
                            } else if name_value_pair.path.is_ident("return_type") {
                                match &name_value_pair.lit {
                                    syn::Lit::Str(l) => config.return_type = l.value(),
                                    _ => return Err(syn::Error::new_spanned(name_value_pair.lit.clone(), format!("Could not process name-value pair's value as a String")))
                                }
                            } else {
                                return Err(syn::Error::new_spanned(name_value_pair.path.clone(), format!("Unexpected name-value pair '{}'", name_value_pair.path.get_ident().unwrap())))
                            }
                        },
                        _ => return Err(syn::Error::new_spanned(_attr, format!("Could not process as a name-value pair!")))
                    }
                }

                let func_name = format_ident!("get_{}", config.field);
                let mut_func_name = format_ident!("get_mut_{}", config.field);
                let _func_name = format_ident!("_{}", func_name);
                let _mut_func_name = format_ident!("_{}", mut_func_name);
                let clone_func_name = format_ident!("_get_cloned_{}", config.field);
                
                let get_id_func = format_ident!("get_{}_id", config.field);
                let retn = format_ident!("{}", config.return_type);
                let field_container_name = format_ident!("{}", config.field_container_name);
                let parent_field = format_ident!("{}", config.parent_field);
        
                let (lookup_type, error_message, err_str);
                if config.getter_type == "by_index" {
                    lookup_type = quote! { usize };
                    err_str = format!("\"Could not find {} at index {{}}!\"", config.field);
                } else {
                    lookup_type = quote!{ &str };
                    err_str = format!("\"Could not find {} named {{}}!\"", config.field);
                }
                error_message = quote! { &format!(#err_str, identifier) };

                getter_functions.extend(quote! {
                    impl #name {
                        pub fn #func_name (&self, parent_field_id: usize, identifier: #lookup_type) -> Option<& #retn > {
                            if let Some(i) = self.#parent_field[parent_field_id].#get_id_func(identifier) {
                                Some(&self.#field_container_name[i])
                            } else {
                                Option::None
                            }
                        }
        
                        pub fn #mut_func_name(&mut self, parent_field_id: usize, identifier: #lookup_type) -> Option<&mut #retn> {
                            if let Some(i) = self.#parent_field[parent_field_id].#get_id_func(identifier) {
                                Some(&mut self.#field_container_name[i])
                            } else {
                                Option::None
                            }
                        }
        
                        pub fn #_func_name (&self, parent_field_id: usize, identifier: #lookup_type) -> core::result::Result<& #retn, crate::error::Error> {
                            if let Some(i) = self.#parent_field[parent_field_id].#get_id_func(identifier) {
                                Ok(&self.#field_container_name[i])
                            } else {
                                Err(crate::error::Error::new(#error_message))
                            }
                        }
        
                        pub fn #_mut_func_name(&mut self, parent_field_id: usize, identifier: #lookup_type) -> core::result::Result<&mut #retn, crate::error::Error> {
                            if let Some(i) = self.#parent_field[parent_field_id].#get_id_func(identifier) {
                                Ok(&mut self.#field_container_name[i])
                            } else {
                                Err(crate::error::Error::new(#error_message))
                            }
                        }
        
                        pub fn #clone_func_name(&self, parent_field_id: usize, identifier: #lookup_type) -> core::result::Result<#retn, crate::error::Error> {
                            if let Some(i) = self.#parent_field[parent_field_id].#get_id_func(identifier) {
                                Ok(self.#field_container_name[i].clone())
                            } else {
                                Err(crate::error::Error::new(#error_message))
                            }
                        }
                    }
                });
            }
        },
        _ => {},
      }
  }
  Ok(getter_functions.into())
}
