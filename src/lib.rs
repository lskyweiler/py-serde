mod config;
mod py_import_object;
pub mod utils;

#[cfg(feature = "build-py-lib")]
use pyo3::prelude::pymodule;
use pyo3_stub_gen::define_stub_info_gatherer;

/// A Python module implemented in Rust.
#[cfg(feature = "build-py-lib")]
#[pymodule(name = "unpack")]
mod py_unpack {
    use super::*;
    use pyo3::{PyAny, exceptions::PyValueError, prelude::*, types::PyDict};
    use pyo3_stub_gen::derive::*;

    #[pymodule_export]
    use super::config::PyImportConfig;

    /// Constructs a single python object from a serialized PyImport json string
    ///
    /// Equivalent to ```unpack.construct_object(json.loads(json_str))```
    ///
    /// See [`unpack.construct_object`] for more details
    ///
    #[gen_stub_pyfunction]
    #[pyfunction]
    #[pyo3(signature = (object_json_str, config = None))]
    fn construct_object_json<'py>(
        py: Python<'py>,
        object_json_str: String,
        config: Option<config::PyImportConfig>,
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
    /// # Examples
    /// ```
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
    ///
    /// You can customize the deserialization behavior
    /// ```python
    /// obj = {
    ///     "my_import": "foo.Foo",
    ///     "my_data": {"foo": {"a": 500, "b": [7.0, 8.0]}},
    /// }
    /// constructed = unpack.construct_object(
    ///     obj,
    ///     unpack.PyImportConfig(object_import_key="my_import", data_key="my_data"),
    /// )
    /// ```
    #[gen_stub_pyfunction]
    #[pyfunction]
    #[pyo3(signature = (object_dict, config = None))]
    fn construct_object<'py>(
        object_dict: Bound<'py, PyDict>,
        config: Option<config::PyImportConfig>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let import_obj =
            py_import_object::PyImportObject::from_dict(object_dict, config.unwrap_or_default())?;
        import_obj.try_construct_object()
    }

    /// Recursively dumps all public (non leading _) members to a json-able dict
    #[gen_stub_pyfunction]
    #[pyfunction]
    #[pyo3(signature = (object, config = None))]
    fn dump_object<'py>(
        object: Bound<'py, PyAny>,
        config: Option<config::PyDumpConfig>,
    ) -> PyResult<Bound<'py, PyAny>> {
        py_import_object::recursive_serialize_py_object(object, &config.unwrap_or_default())
    }
    /// Dump an object to a PythonImportObject json string
    /// Equivalent to
    /// `json.dumps(dump_object(obj))`
    #[gen_stub_pyfunction]
    #[pyfunction]
    #[pyo3(signature = (object, config = None, pretty=false))]
    fn dump_object_json<'py>(
        object: Bound<'py, PyAny>,
        config: Option<config::PyDumpConfig>,
        pretty: bool,
    ) -> PyResult<String> {
        let obj =
            py_import_object::recursive_serialize_py_object(object, &config.unwrap_or_default())?;
        let obj_dict = obj.cast_into::<PyDict>()?;
        let dumped = utils::py_dict_to_serde_value(&obj_dict)?;
        let out = match pretty {
            true => serde_json::to_string_pretty(&dumped),
            false => serde_json::to_string(&dumped),
        };
        out.map_err(|e| PyValueError::new_err(format!("{:?}", e)))
    }
}
define_stub_info_gatherer!(stub_info);

pub mod prelude {
    use super::*;
    pub use config::{PyDumpConfig, PyImportConfig};
    pub use py_import_object::{
        EntryPoint, PyImportObject, PyImportType, PyObjectDeserializer, PyObjectSerializer,
    };
}
