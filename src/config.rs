use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

pub const DEFAULT_ENTRY_POINT_GROUP: &str = "";
pub const DEFAULT_OBJ_IMPORT_KEY: &str = "object_import";
pub const DEFAULT_OBJ_ENTRY_POINT_KEY: &str = "object_entry_point";
pub const DEFAULT_DATA_KEY: &str = "data";

/// Configures the behavior when loading a python object
#[derive(Clone)]
#[gen_stub_pyclass]
#[pyclass]
pub struct PyImportConfig {
    /// Apply this entry point to all objects that do not define their own
    #[pyo3(get, set)]
    pub entry_point_group: String,

    /// Use this key to pull out the object's optional import path
    #[pyo3(get, set)]
    pub object_import_key: String,
    /// Use this key to pull out the object's optional entry point
    #[pyo3(get, set)]
    pub object_entry_point_key: String,

    /// Key to use to grab data from. Used as Object(**kwargs)
    #[pyo3(get, set)]
    pub data_key: String,
}
impl Default for PyImportConfig {
    fn default() -> Self {
        Self {
            entry_point_group: DEFAULT_ENTRY_POINT_GROUP.to_string(),
            object_import_key: DEFAULT_OBJ_IMPORT_KEY.to_string(),
            object_entry_point_key: DEFAULT_OBJ_ENTRY_POINT_KEY.to_string(),
            data_key: DEFAULT_DATA_KEY.to_string(),
        }
    }
}
#[gen_stub_pymethods]
#[pymethods]
impl PyImportConfig {
    #[new]
    #[pyo3(
        signature = (
            entry_point_group=DEFAULT_ENTRY_POINT_GROUP.to_string(), 
            object_import_key=DEFAULT_OBJ_IMPORT_KEY.to_string(), 
            object_entry_point_key=DEFAULT_OBJ_ENTRY_POINT_KEY.to_string(), 
            data_key=DEFAULT_DATA_KEY.to_string()
        )
    )]
    fn py_new(
        entry_point_group: String,
        object_import_key: String,
        object_entry_point_key: String,
        data_key: String,
    ) -> Self {
        Self {
            entry_point_group,
            object_import_key,
            object_entry_point_key,
            data_key
        }
    }
}

/// Configures the behavior when serializing a python object
#[derive(Clone)]
#[gen_stub_pyclass]
#[pyclass]
pub struct PyDumpConfig {
    /// Use this key to pull out the object's optional import path
    #[pyo3(get, set)]
    pub object_import_key: String,
    /// Key to use to grab data from. Used as Object(**kwargs)
    #[pyo3(get, set)]
    pub data_key: String,

    /// Allow dumping of private members (leading _)
    #[pyo3(get, set)]
    pub dump_privates: bool,

    /// Never fail serializing an object. Will always be able to cast object to a string
    #[pyo3(get, set)]
    pub never_fail: bool,

    /// Dump enum.name instead of enum.value
    #[pyo3(get, set)]
    pub use_enum_name: bool,
}
impl Default for PyDumpConfig {
    fn default() -> Self {
        Self {
            object_import_key: DEFAULT_OBJ_IMPORT_KEY.to_string(),
            data_key: DEFAULT_DATA_KEY.to_string(),
            never_fail: false,
            dump_privates: false,
            use_enum_name: true
        }
    }
}
#[gen_stub_pymethods]
#[pymethods]
impl PyDumpConfig {
    #[new]
    #[pyo3(
        signature = (
            object_import_key=DEFAULT_OBJ_IMPORT_KEY.to_string(), 
            data_key=DEFAULT_DATA_KEY.to_string(),
            never_fail=false,
            dump_privates=false,
            use_enum_name=true
        )
    )]
    fn py_new(
        object_import_key: String,
        data_key: String,
        never_fail: bool,
        dump_privates: bool,
        use_enum_name: bool
    ) -> Self {
        Self {
            object_import_key,
            data_key,
            never_fail,
            dump_privates,
            use_enum_name
        }
    }
}
