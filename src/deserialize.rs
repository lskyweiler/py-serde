use crate::py_import_object::PyImportObject;
use pyo3::{prelude::*, types::PyDict};
use serde::{
    Deserialize,
    de::{DeserializeSeed, Deserializer, Error, Visitor},
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
}
impl<'py> PyObjectDeserializer<'py> {
    pub fn new(py: Python<'py>) -> Self {
        Self { py }
    }
}
impl<'py, 'de> DeserializeSeed<'de> for PyObjectDeserializer<'py> {
    type Value = PyImportObject<'py>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(PyObjectVisitor { py: self.py })
    }
}
struct PyObjectVisitor<'py> {
    py: Python<'py>,
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
        while let Some(key) = map.next_key()? {
            match key {
                PyImportDictFields::ObjectImport => import = Some(map.next_value::<String>()?),
                PyImportDictFields::ObjectEntryPoint => {
                    entry_point = Some(map.next_value::<String>()?)
                }
                PyImportDictFields::Data => {
                    let _ = map.next_value::<String>()?;
                    data = Some(PyDict::new(self.py))
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

        Ok(PyImportObject::new(import, entry_point, data.unwrap()))
    }
}
