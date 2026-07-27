use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

pub const DEFAULT_ENTRY_POINT_GROUP: &str = "";
pub const DEFAULT_OBJ_IMPORT_KEY: &str = "object_import";
pub const DEFAULT_OBJ_ENTRY_POINT_KEY: &str = "object_entry_point";
pub const DEFAULT_DATA_POINT_KEY: &str = "data";

#[derive(Clone)]
#[pyclass]
#[gen_stub_pyclass]
pub struct PyImportConfig {
    #[pyo3(get, set)]
    pub entry_point_group: String,

    #[pyo3(get, set)]
    pub object_import_key: String,
    #[pyo3(get, set)]
    pub object_entry_point_key: String,
    #[pyo3(get, set)]
    pub data_key: String,
}
impl Default for PyImportConfig {
    fn default() -> Self {
        Self {
            entry_point_group: DEFAULT_ENTRY_POINT_GROUP.to_string(),
            object_import_key: DEFAULT_OBJ_IMPORT_KEY.to_string(),
            object_entry_point_key: DEFAULT_OBJ_ENTRY_POINT_KEY.to_string(),
            data_key: DEFAULT_DATA_POINT_KEY.to_string(),
        }
    }
}
#[pymethods]
impl PyImportConfig {
    #[new]
    #[pyo3(
        signature = (
            entry_point_group=DEFAULT_ENTRY_POINT_GROUP.to_string(), 
            object_import_key=DEFAULT_OBJ_IMPORT_KEY.to_string(), 
            object_entry_point_key=DEFAULT_OBJ_ENTRY_POINT_KEY.to_string(), 
            data_key=DEFAULT_DATA_POINT_KEY.to_string()
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
