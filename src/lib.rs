mod py_import_config;
mod py_import_object;
mod utils;

use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};

/// A Python module implemented in Rust.
#[pymodule]
mod unpack {
    use super::*;
    use pyo3::{exceptions::PyValueError, types::PyDict};

    /// Constructs a single python object from a serialized PyImport json string
    ///
    /// # Examples
    /// ```json
    /// {
    ///     "object_import": "foo.bar.Baz",
    ///     "data": {
    ///         "x": 100.0,
    ///         "complex": {
    ///             "object_import": "foo.Foo",
    ///             "data": {"a": [100.0]}
    ///         },
    ///         "pydantic_obj": {
    ///             "a": 100.0,
    ///             "b": {
    ///                 "x": 100, "y": 100, "z": 100
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[pyfunction]
    #[gen_stub_pyfunction]
    #[pyo3(signature = (object_json_str, config = None))]
    fn construct_object_json<'py>(
        py: Python<'py>,
        object_json_str: String,
        config: Option<py_import_config::PyImportConfig>,
    ) -> PyResult<Bound<'py, PyAny>> {
        match py_import_object::PyImportObject::from_serde_json_str(
            py,
            &object_json_str,
            config.unwrap_or_default(),
        ) {
            Ok(py_import_obj) => py_import_obj.try_construct_object(),
            Err(what) => Err(PyValueError::new_err(format!("{:?}", what))),
        }
    }
    /// Constructs a single python object from a PyImport dictionary
    ///
    /// Equivalent to ```unpack.construct_object_json(json.loads(json_str))```
    ///
    /// # Examples
    /// ```json
    /// {
    ///     "object_import": "foo.bar.Baz",
    ///     "data": {
    ///         "x": 100.0,
    ///         "complex": {
    ///             "object_import": "foo.Foo",
    ///             "data": {"a": [100.0]}
    ///         },
    ///         "pydantic_obj": {
    ///             "a": 100.0,
    ///             "b": {
    ///                 "x": 100, "y": 100, "z": 100
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[pyfunction]
    #[gen_stub_pyfunction]
    #[pyo3(signature = (object_dict, config = None))]
    fn construct_object<'py>(
        object_dict: Bound<'py, PyDict>,
        config: Option<py_import_config::PyImportConfig>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let import_obj =
            py_import_object::PyImportObject::from_dict(object_dict, config.unwrap_or_default())?;
        import_obj.try_construct_object()
    }
}
define_stub_info_gatherer!(stub_info);

pub mod prelude {
    use super::*;
    pub use py_import_object::{PyImportObject, PyObjectDeserializer};
    pub use utils::*;
}
