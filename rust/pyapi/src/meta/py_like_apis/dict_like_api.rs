//#![feature(concat_idents)]

use indexmap::map::IndexMap;
use origen::error::Error;
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
    fn lookup_key(&self) -> &str;
    fn lookup_table(
        &self,
        dut: &std::sync::MutexGuard<origen::core::dut::Dut>,
    ) -> IndexMap<String, usize>;
    fn new_pyitem(&self, py: Python, name: &str, model_id: usize) -> Result<PyObject, Error>;

    //   #[proc_macro]
    //   fn lookup_code(input: TokenStream) -> TokenStream {

    //   };

    fn keys(&self) -> PyResult<Vec<String>> {
        let dut = DUT.lock().unwrap();
        let names = self.lookup_table(&dut); //model.lookup(self.lookup_key())?;
        Ok(names.iter().map(|(k, _)| k.clone()).collect())
    }

    fn values(&self) -> PyResult<Vec<PyObject>> {
        let dut = DUT.lock().unwrap();
        let items = self.lookup_table(&dut);

        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut v: Vec<PyObject> = Vec::new();
        for (n, _item) in items {
            v.push(self.new_pyitem(py, &n, self.model_id())?);
        }
        Ok(v)
    }

    fn items(&self) -> PyResult<Vec<(String, PyObject)>> {
        let dut = DUT.lock().unwrap();
        let items = self.lookup_table(&dut);

        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut _items: Vec<(String, PyObject)> = Vec::new();
        for (n, _item) in items.iter() {
            _items.push((n.clone(), self.new_pyitem(py, &n, self.model_id())?));
        }
        Ok(_items)
    }

    fn get(&self, name: &str) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let items = self.lookup_table(&dut);
        let item = items.get(name);

        let gil = Python::acquire_gil();
        let py = gil.python();
        match item {
            Some(_item) => Ok(self.new_pyitem(py, name, self.model_id())?),
            None => Ok(py.None()),
        }
    }

    // Functions for PyMappingProtocol
    fn __getitem__(&self, name: &str) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let items = self.lookup_table(&dut);
        let item = items.get(name);

        let gil = Python::acquire_gil();
        let py = gil.python();
        match item {
            Some(_item) => Ok(self.new_pyitem(py, name, self.model_id())?),
            None => Err(pyo3::exceptions::KeyError::py_err(format!(
                "No pin or pin alias found for {}",
                name
            ))),
        }
    }

    fn __len__(&self) -> PyResult<usize> {
        let dut = DUT.lock().unwrap();
        let items = self.lookup_table(&dut);
        Ok(items.len())
    }

    fn __iter__(&self) -> PyResult<DictLikeIter> {
        let dut = DUT.lock().unwrap();
        let items = self.lookup_table(&dut);
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

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for DictLikeIter {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(slf.to_object(py))
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
