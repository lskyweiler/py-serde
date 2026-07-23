use pyo3::{prelude::*, types::PyFloat};
use serde::{
    de::{DeserializeSeed, Deserializer, Visitor},
    Deserialize,
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
pub struct PyObjectDeserializer<'py>(pub Python<'py>);
impl<'py, 'de> DeserializeSeed<'de> for PyObjectDeserializer<'py> {
    type Value = Py<PyAny>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(PyObjectVisitor(self.0))
    }
}
struct PyObjectVisitor<'py>(Python<'py>);
impl<'py, 'de> Visitor<'de> for PyObjectVisitor<'py> {
    type Value = Py<PyAny>;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("PyImportDict")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut import: Option<String> = None;
        let mut kwargs: Option<Py<PyAny>> = None;
        let mut entry_point: Option<String> = None;
        while let Some(key) = map.next_key()? {
            match key {
                PyImportDictFields::ObjectImport => import = Some(map.next_value::<String>()?),
                PyImportDictFields::ObjectEntryPoint => entry_point = Some(map.next_value::<String>()?),
                PyImportDictFields::Data => {
                    let _ = map.next_value::<String>()?;
                    kwargs = Some(
                        PyFloat::new(self.0, 100.)
                            .unbind()
                            .clone_ref(self.0)
                            .into_any(),
                    )
                }
            }
        }

        Ok(kwargs.unwrap())
    }
}
