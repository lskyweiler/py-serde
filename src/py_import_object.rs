use crate::{config::*, utils};
use pyo3::{
    exceptions::{PyKeyError, PyValueError},
    prelude::*,
    types::*,
};
use serde::de::DeserializeSeed;

const UNPACK_SERIALIZE_DUNDER: &str = "__unpack_dump__";
const UNPACK_DESERIALIZE_DUNDER: &str = "__unpack_load__";

/// Module to handle serializing and deserializing PyImportObjects. This is in this file to avoid circular imports
mod py_obj_serde {
    use super::*;
    use crate::utils;

    /// Deserialize
    pub mod de {
        use super::*;
        use serde::de::*;

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
                            data = match utils::serde_value_to_py_dict(self.py, val) {
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

    /// Serialize
    pub mod ser {
        use super::*;

        pub struct PyObjectSerializer<'py> {
            obj: Bound<'py, PyDict>,
        }
        impl<'py> PyObjectSerializer<'py> {
            pub fn new(obj: Bound<'py, PyAny>, config: PyDumpConfig) -> PyResult<Self> {
                let json_able_obj_dump: Bound<'py, PyDict> =
                    recursive_serialize_py_object(obj, &config)?.extract()?;
                Ok(Self {
                    obj: json_able_obj_dump,
                })
            }

            /// Convert the python object to a serde_json value
            pub fn to_serde_value(&self) -> PyResult<serde_json::Value> {
                utils::py_dict_to_serde_value(&self.obj)
            }
        }
    }
}

// re-export
pub use py_obj_serde::{de::PyObjectDeserializer, ser::PyObjectSerializer};

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

    /// Tries to construct this object recursively, importing all objects down the tree
    pub fn try_construct_object(&self) -> PyResult<Bound<'py, PyAny>> {
        let class_type = self.get_object_type()?;

        if class_type.hasattr(UNPACK_DESERIALIZE_DUNDER)? {
            class_type.call_method1(UNPACK_DESERIALIZE_DUNDER, (self.data.clone(),))
        } else if utils::is_pydantic_baseclass(&class_type)? {
            class_type.call_method1("model_validate", (self.data.clone(),))
        } else if utils::is_enum(&class_type)? {
            // enums are special since they cant be instantiated with a normal obj()
            if let Some(name) = self.data.get_item("name")? {
                class_type.call_method1("__getitem__", (name,))
            } else if let Some(value) = self.data.get_item("value")? {
                class_type.call1((value,))
            } else {
                return Err(PyValueError::new_err(
                    "Enums require either a value or name to instantiate",
                ));
            }
        } else if utils::is_pathlib(&class_type)? {
            // pathlibs are special since they can either be a Posix or Windows path and they dont take any **kwargs
            if let Some(path) = self.data.get_item("path")? {
                class_type.call1((path.clone(),))
            } else {
                return Err(PyValueError::new_err(
                    "Pathlib paths require a path to instantiate",
                ));
            }
        } else if utils::is_datetime(&class_type)? {
            if let Some(dt) = self.data.get_item("datetime")? {
                class_type.call_method1("fromisoformat", (dt.clone(),))
            } else {
                return Err(PyValueError::new_err(
                    "Pathlib paths require a path to instantiate",
                ));
            }
        } else {
            // Class(**kwargs)
            let de_data =
                recursive_deserialize_import_dict(self.data.as_any().clone(), &self.config)?;
            class_type.call((), Some(&de_data.cast_into::<PyDict>()?))
        }
    }

    /// Instantiate this object from a json string without constructing the python objects recursively
    pub fn from_serde_json_str(
        py: Python<'py>,
        data: &str,
        cfg: PyImportConfig,
    ) -> Result<Self, serde_json::Error> {
        let mut de = serde_json::de::Deserializer::from_str(data);
        let py_obj_de = py_obj_serde::de::PyObjectDeserializer::new(py, cfg);
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

/// Recursively serialize any python object into a PyImportDict
/// Note that currently, this loses entry point information
pub fn recursive_serialize_py_object<'py>(
    value: Bound<'py, PyAny>,
    config: &PyDumpConfig,
) -> PyResult<Bound<'py, PyAny>> {
    // If the class has defined its own custom hook to handle serialization/deserialization
    /*
    class MyClass:
        def __unpack_dump__(self) -> dict:
            ...
        @staticmethod
        def __unpack_load__() -> MyClass:
            ...
    */
    if value.hasattr(UNPACK_SERIALIZE_DUNDER)? {
        let dumped = value.call_method0(UNPACK_SERIALIZE_DUNDER)?;
        let r_dict = PyDict::new(value.py());
        r_dict.set_item(&config.data_key, dumped)?;
        let import_path = utils::get_obj_import_path(&value.get_type())?;
        r_dict.set_item(&config.object_import_key, import_path)?;
        return Ok(r_dict.into_any());
    }

    if utils::is_pydantic_baseclass(&value.get_type())? {
        let kwargs = PyDict::new(value.py());
        kwargs.set_item("mode", "json")?;
        let data = value.call_method("model_dump", (), Some(&kwargs))?;

        let import_path = utils::get_obj_import_path(&value.get_type())?;
        let r_val = PyDict::new(value.py());
        r_val.set_item(&config.data_key, data)?;
        r_val.set_item(&config.object_import_key, import_path)?;
        return Ok(r_val.into_any());
    }
    if utils::is_enum(&value)? {
        let r_dict = PyDict::new(value.py());

        let data = PyDict::new(value.py());
        if config.use_enum_name {
            data.set_item("name", value.getattr("name")?)?;
        } else {
            data.set_item("value", value.getattr("value")?)?;
        };

        r_dict.set_item(&config.data_key, data)?;
        let import_path = utils::get_obj_import_path(&value.get_type())?;
        r_dict.set_item(&config.object_import_key, import_path)?;

        return Ok(r_dict.into_any());
    }
    if utils::is_pathlib(&value)? {
        let r_dict = PyDict::new(value.py());
        let path = value.to_string();
        // Pathlib is complicted where it creates Windows or Posix paths, and we want this to be portable between systems
        r_dict.set_item(&config.object_import_key, "pathlib.Path")?;
        let data = PyDict::new(value.py());
        data.set_item("path", path)?;
        r_dict.set_item(&config.data_key, data)?;

        return Ok(r_dict.into_any());
    }
    if utils::is_datetime(&value)? {
        let r_dict = PyDict::new(value.py());
        let path = value.to_string();
        let import_path = utils::get_obj_import_path(&value.get_type())?;
        r_dict.set_item(&config.object_import_key, import_path)?;

        let data = PyDict::new(value.py());
        data.set_item("datetime", path)?;
        r_dict.set_item(&config.data_key, data)?;

        return Ok(r_dict.into_any());
    }
    if value.is_instance_of::<PyList>()
        || value.is_instance_of::<PyTuple>()
        || value.is_instance_of::<PySet>()
    {
        let r_list = PyList::empty(value.py());
        for v in value.try_iter()? {
            r_list.append(recursive_serialize_py_object(v.unwrap(), config)?)?;
        }
        return Ok(r_list.into_any());
    }
    if value.is_instance_of::<PyDict>() {
        let r_dict = PyDict::new(value.py());
        for (k, v) in value.cast_into::<PyDict>()?.iter() {
            let s_k = recursive_serialize_py_object(k, config)?;
            let s_v = recursive_serialize_py_object(v, config)?;
            r_dict.set_item(s_k, s_v)?;
        }
        return Ok(r_dict.into_any());
    }
    if value.hasattr("__dict__")? {
        let members = value.getattr("__dict__")?.cast_into::<PyDict>()?;

        let data = PyDict::new(value.py());
        for (k, v) in members.iter() {
            let name: String = k.extract()?;
            if name.starts_with("_") && !config.dump_privates {
                continue;
            }
            let s_k = recursive_serialize_py_object(k, config)?;
            let s_v = recursive_serialize_py_object(v, config)?;
            data.set_item(s_k, s_v)?;
        }
        let r_dict = PyDict::new(value.py());
        r_dict.set_item(&config.data_key, data)?;
        let import_path = utils::get_obj_import_path(&value.get_type())?;
        r_dict.set_item(&config.object_import_key, import_path)?;

        return Ok(r_dict.into_any());
    }
    if value.is_instance_of::<PyFloat>()
        || value.is_instance_of::<PyInt>()
        || value.is_instance_of::<PyString>()
        || value.is_instance_of::<PyBool>()
        || value.is_none()
    {
        return Ok(value);
    }

    // If we get here, all other things have failed. Either just cast to a string or return error
    if config.never_fail {
        let cast_string = PyString::new(value.py(), &value.to_string());
        Ok(cast_string.into_any())
    } else {
        Err(PyValueError::new_err(format!(
            "Could not serialize object {} and never_fail was false",
            value.get_type().name()?
        )))
    }
}

#[cfg(test)]
mod test_py_object_import {
    use super::*;
    use crate::config;

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
                config::PyImportConfig::default(),
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
                config::PyImportConfig::default(),
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
