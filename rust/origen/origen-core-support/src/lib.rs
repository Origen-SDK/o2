extern crate proc_macro;
use crate::proc_macro::TokenStream;
use syn;

mod id_getters_derive;

#[proc_macro_derive(IdGetters, attributes(id_getters_by_index, id_getters_by_mapping))]
pub fn id_getters_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    match id_getters_derive::impl_id_getters(&ast) {
        Ok(obj) => obj,
        Err(message) => panic!("{}", message),
    }
}
