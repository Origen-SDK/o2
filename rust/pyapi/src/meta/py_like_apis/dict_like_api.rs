//#![feature(concat_idents)]

use indexmap::map::IndexMap;
use origen::DUT;
use pyo3::prelude::*;
// dut: &std::sync::MutexGuard<origen::core::dut::Dut>

// extern crate proc_macro;
// extern crate syn;
// // #[macro_use]
// extern crate quote;

// use proc_macro::TokenStream;

// #[proc_macro_derive(DictLikeAPI, attributes(lookup_code))]
// pub fn derive_dict_like_api(input: TokenStream) -> TokenStream {
//     let n = input.to_string();
//     let ast = syn::parse_derive_input(&n).unwrap();
//     let gen;
//     quote! {
//         impl DictLikeAPI for #name {
//             fn lookup_code() {

//             }
//         }
//     }
//     gen.parse().unwrap()
// }

// struct DictLikeParams {
//     lookup_code: syn::expr;
// }

// impl Parse for DictLikeParams {
//     fn parse(input: ParseStream) -> Result<Self> {
//         let lookup_code = input.parse()?;
//         Ok(DictLikeParams{lookup_code: lookup_code})
//     }
// }

pub trait DictLikeAPI {
    fn model_id(&self) -> usize;
    #[allow(dead_code)]
    fn lookup_key(&self) -> &str;
    fn lookup_table(
        &self,
        dut: &std::sync::MutexGuard<origen::core::dut::Dut>,
    ) -> IndexMap<String, usize>;
    fn new_pyitem(&self, py: Python, name: &str, model_id: usize) -> PyResult<PyObject>;

    //   #[proc_macro]
    //   fn lookup_code(input: TokenStream) -> TokenStream {

    //   };

    fn keys(&self) -> PyResult<Vec<String>> {
        let dut = DUT.lock().unwrap();
        let names = self.lookup_table(&dut); //model.lookup(self.lookup_key())?;
        Ok(names.iter().map(|(k, _)| k.clone()).collect())
    }

    fn values(&self) -> PyResult<Vec<PyObject>> {
        let items;
        {
            let dut = DUT.lock().unwrap();
            items = self.lookup_table(&dut);
        }

        Python::with_gil(|py| {
            let mut v: Vec<PyObject> = Vec::new();
            for (n, _item) in items {
                v.push(self.new_pyitem(py, &n, self.model_id())?);
            }
            Ok(v)
        })
    }

    fn items(&self) -> PyResult<Vec<(String, PyObject)>> {
        let items;
        {
            let dut = DUT.lock().unwrap();
            items = self.lookup_table(&dut);
        }

        Python::with_gil(|py| {
            let mut _items: Vec<(String, PyObject)> = Vec::new();
            for (n, _item) in items.iter() {
                _items.push((n.clone(), self.new_pyitem(py, &n, self.model_id())?));
            }
            Ok(_items)
        })
    }

    fn get(&self, name: &str) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            {
                let dut = DUT.lock().unwrap();
                let items = self.lookup_table(&dut);
                if items.get(name).is_none() {
                    return Ok(py.None());
                }
            }
            Ok(self.new_pyitem(py, name, self.model_id())?)
        })
    }

    // Functions for PyMappingProtocol
    fn __getitem__(&self, name: &str) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            {
                let dut = DUT.lock().unwrap();
                let items = self.lookup_table(&dut);
                if items.get(name).is_none() {
                    return Err(pyo3::exceptions::PyKeyError::new_err(format!(
                        "No item found for {}",
                        name
                    )));
                }
            }
            Ok(self.new_pyitem(py, name, self.model_id())?)
        })
    }

    fn __len__(&self) -> PyResult<usize> {
        let dut = DUT.lock().unwrap();
        let items = self.lookup_table(&dut);
        Ok(items.len())
    }

    fn __iter__(&self) -> PyResult<DictLikeIter> {
        let items;
        {
            let dut = DUT.lock().unwrap();
            items = self.lookup_table(&dut);
        }
        Ok(DictLikeIter {
            keys: items.iter().map(|(s, _)| s.clone()).collect(),
            i: 0,
        })
    }
}

#[pyclass]
pub struct DictLikeIter {
    keys: Vec<String>,
    i: usize,
}

#[pymethods]
impl DictLikeIter {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<Self>> {
        Ok(slf.into())
    }

    /// The Iterator will be created with an index starting at 0 and the pin names at the time of its creation.
    /// For each call to 'next', we'll create a pin object with the next value in the list, or None, if no more keys are available.
    /// Note: this means that the iterator can become stale if the PinContainer is changed. This can happen if the iterator is stored from Python code
    ///  directly. E.g.: i = dut.pins.__iter__() => iterator with the pin names at the time of creation,
    /// Todo: Fix the above using iterators. My Rust skills aren't there yet though... - Coreyeng
    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<String>> {
        if slf.i >= slf.keys.len() {
            return Ok(None);
        }
        let name = slf.keys[slf.i].clone();
        slf.i += 1;
        Ok(Some(name))
    }
}
