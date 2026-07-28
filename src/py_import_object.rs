use crate::{py_import_config::PyImportConfig, utils};
use pyo3::{exceptions::PyKeyError, prelude::*, types::*};
use serde::de::DeserializeSeed;

/// Module to handle serializing and deserializing PyImportObjects. This is in this file to avoid circular imports
mod py_obj_serde {
    use super::*;
    use crate::utils;
    use serde::de::{Deserializer, Error, Visitor};

    /// Serialize a single PyImportDict into a Py<PyAny> object
    pub struct PyObjectDeserializer<'py> {
        py: Python<'py>,
        py_import_cfg: PyImportConfig,
    }
    impl<'py> PyObjectDeserializer<'py> {
        pub fn new(py: Python<'py>, py_import_cfg: PyImportConfig) -> Self {
            Self { py, py_import_cfg }
        }
    }
    impl<'py, 'de> DeserializeSeed<'de> for PyObjectDeserializer<'py> {
        type Value = PyImportObject<'py>;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(PyObjectVisitor {
                py: self.py,
                py_import_cfg: self.py_import_cfg,
            })
        }
    }
    struct PyObjectVisitor<'py> {
        py: Python<'py>,
        py_import_cfg: PyImportConfig,
    }
    impl<'py, 'de> Visitor<'de> for PyObjectVisitor<'py> {
        type Value = PyImportObject<'py>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("PyImportDict")
        }
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut import: Option<String> = None;
            let mut data: Option<Bound<'py, PyDict>> = None;
            let mut entry_point: Option<EntryPoint> = None;
            while let Some(key) = map.next_key::<String>()? {
                match key {
                    val if val == self.py_import_cfg.object_import_key => {
                        import = Some(map.next_value::<String>()?)
                    }
                    val if val == self.py_import_cfg.entry_point_group => {
                        entry_point = Some(map.next_value::<EntryPoint>()?);
                    }
                    val if val == self.py_import_cfg.data_key => {
                        let val = map.next_value::<serde_json::Value>()?;
                        data = match utils::json_value_to_py_dict(self.py, val) {
                            Ok(data) => Some(data),
                            Err(what) => return Err(A::Error::custom(format!("{:?}", what))),
                        };
                    }
                    val => {
                        return Err(A::Error::custom(format!(
                            "Got unknown field in PyImport dict {}",
                            val
                        )));
                    }
                }
            }

            if data.is_none() {
                return Err(A::Error::custom(
                    "Data must be defined in the python import dict",
                ));
            }

            match PyImportType::from_either(import, entry_point) {
                Some(py_imp) => Ok(PyImportObject::new(
                    py_imp,
                    self.py_import_cfg,
                    data.unwrap(),
                )),
                None => {
                    return Err(A::Error::custom(
                        "An object import or object entry_point must be dfined. But not both",
                    ));
                }
            }
        }
    }
}

// re-export
pub use py_obj_serde::PyObjectDeserializer;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct EntryPoint {
    pub name: String,
    pub group: Option<String>,
}

pub enum PyImportType {
    ImportPath(String),
    EntryPoint(EntryPoint),
}
impl PyImportType {
    pub fn from_either(import: Option<String>, entry_point: Option<EntryPoint>) -> Option<Self> {
        if !(import.is_none() ^ entry_point.is_none()) {
            return None;
        } else if import.is_some() {
            return Some(Self::ImportPath(import.unwrap()));
        } else {
            return Some(Self::EntryPoint(entry_point.unwrap()));
        }
    }
}

pub struct PyImportObject<'py> {
    import_type: PyImportType,
    config: PyImportConfig,
    data: Bound<'py, PyDict>,
}
impl<'py> PyImportObject<'py> {
    pub fn new(import_type: PyImportType, cfg: PyImportConfig, data: Bound<'py, PyDict>) -> Self {
        Self {
            import_type,
            data,
            config: cfg,
        }
    }
    pub fn from_dict(object_dict: Bound<'py, PyDict>, cfg: PyImportConfig) -> PyResult<Self> {
        let maybe_import: Option<String> = object_dict
            .get_item(&cfg.object_import_key)?
            .and_then(|val| val.extract().ok()?);
        let maybe_entry_point = match object_dict.get_item(&cfg.object_entry_point_key)? {
            Some(d) => {
                let ep_dict = d.cast::<PyDict>()?;
                let name = match ep_dict.get_item("name")? {
                    Some(n) => n.to_string(),
                    None => return Err(PyKeyError::new_err("An entry point requires a name")),
                };
                let group = match ep_dict.get_item("group")? {
                    Some(g) => Some(g.to_string()),
                    None => None,
                };

                Some(EntryPoint { name, group })
            }
            None => None,
        };

        let data = match object_dict.get_item(&cfg.data_key)? {
            Some(d) => d.extract()?,
            None => {
                return Err(PyKeyError::new_err(
                    "Data must exist in an python import dict",
                ));
            }
        };
        match PyImportType::from_either(maybe_import, maybe_entry_point) {
            Some(py_imp) => Ok(Self::new(py_imp, cfg.clone(), data)),
            None => {
                return Err(PyKeyError::new_err(
                    "Data must exist in an python import dict",
                ));
            }
        }
    }

    /// Gets a class object for this object using either the import path or entry point name
    fn get_object_type(&self) -> PyResult<Bound<'py, PyType>> {
        match &self.import_type {
            PyImportType::ImportPath(imp) => utils::import_obj_from_qual_path(self.data.py(), &imp),
            PyImportType::EntryPoint(ep) => utils::import_obj_from_entry_point(
                self.data.py(),
                &ep.name,
                // An object's entry point group overrides the global entry point config group
                match &ep.group {
                    Some(specific_group) => &specific_group,
                    None => &self.config.entry_point_group,
                },
            ),
        }
    }

    pub fn try_construct_object(&self) -> PyResult<Bound<'py, PyAny>> {
        let class_type = self.get_object_type()?;

        if utils::is_pydantic_baseclass(&class_type)? {
            class_type.call_method1("model_validate", (self.data.clone(),))
        } else {
            // Class(**kwargs)
            let de_data =
                recursive_deserialize_import_dict(self.data.as_any().clone(), &self.config)?;
            class_type.call((), Some(&de_data.cast_into::<PyDict>()?))
        }
    }

    pub fn from_serde_json_str(
        py: Python<'py>,
        data: &str,
        cfg: PyImportConfig,
    ) -> Result<Self, serde_json::Error> {
        let mut de = serde_json::de::Deserializer::from_str(data);
        let py_obj_de = py_obj_serde::PyObjectDeserializer::new(py, cfg);
        py_obj_de.deserialize(&mut de)
    }
}

/// Recursively import a serialized import dictionary
fn recursive_deserialize_import_dict<'py>(
    value: Bound<'py, PyAny>,
    config: &PyImportConfig,
) -> PyResult<Bound<'py, PyAny>> {
    if value.is_instance_of::<PyList>()
        || value.is_instance_of::<PyTuple>()
        || value.is_instance_of::<PySet>()
    {
        let r_list = PyList::empty(value.py());
        for item in value.try_iter()? {
            r_list.append(recursive_deserialize_import_dict(item.unwrap(), &config)?)?;
        }
        return Ok(r_list.into_any());
    } else if !value.is_instance_of::<PyDict>() {
        return Ok(value);
    }

    let new_dict = PyDict::new(value.py());
    for (key, value) in value.cast_into::<PyDict>()?.iter() {
        let new_key = recursive_deserialize_import_dict(key, &config)?;
        let new_val = recursive_deserialize_import_dict(value, &config)?;
        new_dict.set_item(new_key, new_val)?;
    }

    match PyImportObject::from_dict(new_dict.clone(), config.clone()) {
        Ok(val) => {
            return val.try_construct_object();
        }
        Err(_) => {
            // not an import dict, just pass along
            return Ok(new_dict.into_any());
        }
    }
}

#[cfg(test)]
mod test_py_object_import {
    use super::*;
    use crate::py_import_config;

    #[test]
    fn test_simple_deserde() {
        let json_str = r#"{
            "object_import": "test",
            "data": {
                "a": 100.0,
                "b": "hello"
            }
        }"#;
        Python::attach(|py| {
            let py_obj = PyImportObject::from_serde_json_str(
                py,
                json_str,
                py_import_config::PyImportConfig::default(),
            )
            .unwrap();

            match py_obj.import_type {
                PyImportType::EntryPoint(_) => panic!("Should be import"),
                PyImportType::ImportPath(imp) => assert_eq!(imp, "test"),
            };

            assert!(py_obj.data.contains("a").unwrap());
            assert!(py_obj.data.contains("b").unwrap());
        });
    }
    #[test]
    fn test_construct_object() {
        let json_str = r#"{
            "object_import": "ipaddress.IPv4Address",
            "data": {
                "address": "127.0.0.1"
            }
        }"#;
        Python::attach(|py| {
            let py_obj = PyImportObject::from_serde_json_str(
                py,
                json_str,
                py_import_config::PyImportConfig::default(),
            )
            .unwrap();

            let actual = py_obj.try_construct_object().unwrap();
            let actual_type = actual.get_type();
            assert_eq!(actual_type.name().unwrap(), "IPv4Address");
        });
    }

    #[test]
    fn test_from_dict() {
        let json_str = r#"{
            "object_import": "test",
            "data": {
                "a": 100.0,
                "b": "hello"
            }
        }"#;
        Python::attach(|py| {
            let json_mod = py.import("json").unwrap();
            let object_any = json_mod.call_method1("loads", (json_str,)).unwrap();
            let object_dict: Bound<'_, PyDict> = object_any.extract().unwrap();
            let py_obj = PyImportObject::from_dict(object_dict, PyImportConfig::default()).unwrap();

            match py_obj.import_type {
                PyImportType::EntryPoint(_) => panic!("Should be import"),
                PyImportType::ImportPath(imp) => assert_eq!(imp, "test"),
            };

            assert!(py_obj.data.contains("a").unwrap());
            assert!(py_obj.data.contains("b").unwrap());
        });
    }

    #[test]
    fn test_deserde_dataclass() {
        let py_mod = r#"
import dataclasses

@dataclasses.dataclass
class Foo:
    a: int = 100
    b: list[float] = dataclasses.field(default_factory=lambda: [1.0, 2.0, 3.0])      
        "#;

        Python::attach(|py| {
            utils::add_python_module_from_code(py, py_mod, "rstest").unwrap();

            let json_str = r#"{
                "object_import": "rstest.Foo",
                "data": {"a": 500, "b": [5.0, 6.0]}
            }"#;
            let py_obj =
                PyImportObject::from_serde_json_str(py, json_str, PyImportConfig::default())
                    .expect("Failed to deserialize");

            let actual = py_obj
                .try_construct_object()
                .expect("Unable to recursively construct object");
            let actual_type = actual.get_type();
            assert_eq!(actual_type.name().unwrap(), "Foo");
        });
    }
}
