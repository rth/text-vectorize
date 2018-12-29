#![feature(specialization)]

#[macro_use]
extern crate ndarray;
extern crate numpy;
#[macro_use]
extern crate pyo3;
extern crate text_vectorize;

use ndarray::arr1;
use numpy::{IntoPyArray, PyArray1};
use sprs::CsMat;

use pyo3::prelude::*;
use pyo3::prelude::{pymodinit, ObjectProtocol, Py, PyModule, PyObject, PyResult, Python};
use pyo3::types::PyIterator;

use text_vectorize::HashingVectorizer;

type PyCsrArray = (Py<PyArray1<i32>>, Py<PyArray1<i32>>, Py<PyArray1<i32>>);

fn iterable_to_collection(text: PyIterator) -> PyResult<Vec<String>> {
    // This should not be necessary, ideally PyIterator should be converted
    // to a Rust iterator

    let mut collection: Vec<String> = Vec::new();

    for document in text {
        let document = document?;
        let document = ObjectProtocol::extract::<String>(document)?;
        collection.push(document);
    }
    Ok(collection)
}

fn result_to_csr(py: Python, x: CsMat<i32>) -> PyResult<PyCsrArray> {
    // TODO: 1. use slices directly instead of creating new arrays
    //       2. Possibly avoid casing
    //          https://github.com/rust-ndarray/ndarray/issues/493#issuecomment-424043912
    let indices = arr1(x.indices()).mapv(|elem| elem as i32);
    let indptr = arr1(x.indptr()).mapv(|elem| elem as i32);
    let data = arr1(x.data());

    Ok((
        indices.into_pyarray(py).to_owned(),
        indptr.into_pyarray(py).to_owned(),
        data.into_pyarray(py).to_owned(),
    ))
}

#[pyclass]
pub struct _HashingVectorizerWrapper {
    inner: HashingVectorizer,
}

#[pymethods]
impl _HashingVectorizerWrapper {
    #[new]
    fn __new__(obj: &PyRawObject) -> PyResult<()> {
        let estimator = HashingVectorizer::new();
        obj.init(|_token| _HashingVectorizerWrapper { inner: estimator })
    }

    fn transform(&mut self, py: Python, x: PyObject) -> PyResult<PyCsrArray> {
        let text = PyIterator::from_object(py, &x)?;

        let collection = iterable_to_collection(text)?;

        let x = self.inner.fit_transform(&collection);

        result_to_csr(py, x)
    }
}

#[pymodinit]
fn _lib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<_HashingVectorizerWrapper>()?;

    Ok(())
}
