mod deserialize;
mod py_import_object;
use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};

/// A Python module implemented in Rust.
#[pymodule]
mod py_serde {
    use super::*;
    use pyo3::{exceptions::PyValueError, types::PyDict};
    use serde::de::DeserializeSeed;

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
    #[pyo3(signature = (object_json_str, entry_point_group = ""))]
    fn construct_object_json<'py>(
        py: Python<'py>,
        object_json_str: String,
        entry_point_group: &str,
    ) -> PyResult<Py<PyAny>> {
        let mut json_de: serde_json::Deserializer<serde_json::de::StrRead<'_>> =
            serde_json::de::Deserializer::from_str(&object_json_str);
        let py_import_obj_de = deserialize::PyObjectDeserializer::new(py);
        
        match py_import_obj_de.deserialize(&mut json_de) {
            Ok(mut py_import_obj) => py_import_obj
                .with_entry_point_group(entry_point_group.to_string())
                .try_construct_object(),
            Err(what) => Err(PyValueError::new_err(format!("{:?}", what))),
        }
    }
    /// Constructs a single python object from a PyImport dictionary
    ///
    /// Equivalent to ```py_serde.construct_object_json(json.loads(json_str))```
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
    #[pyo3(signature = (object_dict, entry_point_group = py_import_object::DEFAULT_ENTRY_POINT_GROUP))]
    fn construct_object<'py>(
        object_dict: Bound<'py, PyDict>,
        entry_point_group: &str,
    ) -> PyResult<Py<PyAny>> {
        let mut import_obj = py_import_object::PyImportObject::from_dict(object_dict)?;
        import_obj.with_entry_point_group(entry_point_group.to_string());

        import_obj.try_construct_object()
    }
}
define_stub_info_gatherer!(stub_info);

pub mod prelude {
    use super::*;

    pub use deserialize::PyObjectDeserializer;
    pub use py_import_object::PyImportObject;
}
