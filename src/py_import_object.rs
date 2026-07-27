use crate::{py_import_config::PyImportConfig, utils};
use pyo3::{
    exceptions::{PyKeyError, PyNotImplementedError},
    prelude::*,
    types::{PyDict, PyType},
};
use serde::de::DeserializeSeed;

/// Module to handle serializing and deserializing PyImportObjects. This is in this file to avoid circular imports
mod py_obj_serde {
    use super::*;
    use crate::utils;
    use serde::{
        Deserialize,
        de::{Deserializer, Error, Visitor},
    };

    /// An object to allow deserialization of any python object
    ///
    /// Strategy for importing an object goes as follows:
    ///     - get class object from import path or entrypoint
    ///     - If object is a pydantic model: use model_validate(**kwargs)
    ///     - Else recursively construct objects from the kwargs, then send into the object's constructor as Object(**kwargs)
    ///
    /// Note: Either an import or an entry_point must be defined, but not both
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
    #[derive(Deserialize)]
    #[serde(field_identifier, rename_all = "snake_case")]
    enum PyImportDictFields {
        /// Full qualpath to python class
        ///
        /// * Example
        /// `foo.bar.baz.Foo`
        ObjectImport,
        /// Entrypoint name to lookup under the simplipy.components group
        ObjectEntryPoint,

        /// Data to instantiate the object as **kwargs
        /// May contain other PyImportDict objects to recursively instantiate
        Data,
    }

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
            let mut entry_point: Option<String> = None;
            // todo: support keys from PyImportConfig
            while let Some(key) = map.next_key()? {
                match key {
                    PyImportDictFields::ObjectImport => import = Some(map.next_value::<String>()?),
                    PyImportDictFields::ObjectEntryPoint => {
                        entry_point = Some(map.next_value::<String>()?)
                    }
                    PyImportDictFields::Data => {
                        let val = map.next_value::<serde_json::Value>()?;
                        data = match utils::json_value_to_py_dict(self.py, val) {
                            Ok(data) => Some(data),
                            Err(what) => return Err(A::Error::custom(format!("{:?}", what))),
                        };
                    }
                }
            }

            if data.is_none() {
                return Err(A::Error::custom(
                    "Data must be defined in the python import dict",
                ));
            }
            if !(import.is_none() ^ entry_point.is_none()) {
                return Err(A::Error::custom(
                    "An object import or object entry_point must be dfined. But not both",
                ));
            }

            Ok(PyImportObject::new(
                import,
                entry_point,
                self.py_import_cfg,
                data.unwrap(),
            ))
        }
    }
}

// re-export
pub use py_obj_serde::PyObjectDeserializer;

pub struct PyImportObject<'py> {
    import_path: Option<String>,
    entry_point: Option<String>,

    config: PyImportConfig,

    data: Bound<'py, PyDict>,
}
impl<'py> PyImportObject<'py> {
    pub fn new(
        import_path: Option<String>,
        entry_point: Option<String>,
        cfg: PyImportConfig,
        data: Bound<'py, PyDict>,
    ) -> Self {
        Self {
            import_path,
            entry_point,
            data,
            config: cfg,
        }
    }
    pub fn from_dict(object_dict: Bound<'py, PyDict>, cfg: PyImportConfig) -> PyResult<Self> {
        let maybe_import: Option<String> = object_dict
            .get_item(&cfg.object_import_key)?
            .and_then(|val| val.extract().ok()?);
        let maybe_entry_point: Option<String> = object_dict
            .get_item(&cfg.object_entry_point_key)?
            .and_then(|val| val.extract().ok()?);

        let data = match object_dict.get_item(&cfg.data_key)? {
            Some(d) => d.extract()?,
            None => {
                return Err(PyKeyError::new_err(
                    "Data must exist in an python import dict",
                ));
            }
        };

        Ok(Self {
            import_path: maybe_import,
            entry_point: maybe_entry_point,
            config: cfg.clone(),
            data: data,
        })
    }

    /// Gets a class object for this object using either the import path or entry point name
    fn get_object_type(&self) -> PyResult<Bound<'py, PyType>> {
        if self.import_path.is_some() {
            return utils::import_obj_from_qual_path(
                self.data.py(),
                self.import_path.as_ref().unwrap(),
            );
        } else if self.entry_point.is_some() {
            return utils::import_obj_from_entry_point(
                self.data.py(),
                self.entry_point.as_ref().unwrap(),
                &self.config.entry_point_group,
            );
        } else {
            Err(PyNotImplementedError::new_err(
                "You must define either an import path or entry point name",
            ))
        }
    }

    pub fn try_construct_object_bound(&self) -> PyResult<Bound<'py, PyAny>> {
        let class_type = self.get_object_type()?;
        class_type.call((), Some(&self.data))
    }
    pub fn try_construct_object(&self) -> PyResult<Py<PyAny>> {
        let bound = self.try_construct_object_bound()?;
        Ok(bound.unbind())
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

#[cfg(test)]
mod test_py_object_import {
    use crate::py_import_config;

    use super::*;
    use pyo3::types::{PyDictMethods, PyTypeMethods};
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

            assert!(py_obj.import_path.is_some());
            let imp = py_obj.import_path.unwrap();
            assert_eq!(imp, "test");

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
            let actual_type = actual.bind(py).get_type();
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

            assert!(py_obj.import_path.is_some());
            let imp = py_obj.import_path.unwrap();
            assert_eq!(imp, "test");

            assert!(py_obj.data.contains("a").unwrap());
            assert!(py_obj.data.contains("b").unwrap());
        });
    }
}
