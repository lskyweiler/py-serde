use pyo3::{exceptions::PyNotImplementedError, prelude::*, types::{PyDict, PyFloat, PyType}};

pub const DEFAULT_ENTRY_POINT_GROUP: &str = "";

pub struct PyImportObject<'py> {
    import_path: Option<String>,
    entry_point: Option<String>,

    entry_point_group: Option<String>,

    data: Bound<'py, PyDict>,
}
impl<'py> PyImportObject<'py> {
    pub fn new(import_path: Option<String>, entry_point: Option<String>, data: Bound<'py, PyDict>) -> Self {
        Self {
            import_path,
            entry_point,
            data,
            entry_point_group: Some(DEFAULT_ENTRY_POINT_GROUP.to_string())
        }
    }
    pub fn from_dict(object_dict: Bound<'py, PyDict>) -> PyResult<Self> {
        Err(PyNotImplementedError::new_err("bad"))
    }
    /// Sets the entry point group used when lookup up entry points
    pub fn with_entry_point_group(&mut self, entry_point_group: String) -> &mut Self {
        self.entry_point_group = Some(entry_point_group);
        self
    }

    fn get_object_type(&self) -> Bound<'_, PyType>{
        let tmp = PyFloat::new(self.data.py(), 100.);
        tmp.get_type()
    }

    // !This shouldn't return a PyResult, but a rust result
    pub fn try_construct_object_bound(&self) -> PyResult<Bound<'_, PyAny>> {
        Err(PyNotImplementedError::new_err("bad"))
    }
    pub fn try_construct_object(&self) -> PyResult<Py<PyAny>> {
        Err(PyNotImplementedError::new_err("bad"))

    }
}
