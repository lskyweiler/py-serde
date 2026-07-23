use pyo3::{exceptions::PyNotImplementedError, prelude::*, types::{PyDict, PyFloat, PyType}};

pub struct PyImportObject<'py> {
    import_path: Option<String>,
    entry_point: Option<String>,

    data: Bound<'py, PyDict>,
}
impl<'py> PyImportObject<'py> {
    pub fn new(import_path: Option<String>, entry_point: Option<String>, data: Bound<'py, PyDict>) -> Self {
        Self {
            import_path,
            entry_point,
            data,
        }
    }
    pub fn from_dict(object_dict: Bound<'py, PyDict>) -> PyResult<Self> {
        Err(PyNotImplementedError::new_err("bad"))
        
    }

    fn get_object(&self) -> Bound<'_, PyType>{
        let tmp = PyFloat::new(self.data.py(), 100.);
        tmp.get_type()
    }

    pub fn construct_object_bound(&self) -> PyResult<Bound<'_, PyAny>> {
        Err(PyNotImplementedError::new_err("bad"))
    }
    pub fn construct_object(&self) -> PyResult<Py<PyAny>> {
        Err(PyNotImplementedError::new_err("bad"))

    }
}
