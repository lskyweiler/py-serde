use pyo3::{
    exceptions::{PyKeyError, PyValueError},
    prelude::*,
    types::*,
};
use serde_json;
use std::ffi::CString;

/// Converts a serde_json::Value object into a PyDict
pub fn json_value_to_py_dict<'py>(
    py: Python<'py>,
    value: serde_json::Value,
) -> PyResult<Bound<'py, PyDict>> {
    let json_mod = py.import("json")?;
    let dumped_any = json_mod.call_method1("loads", (value.to_string(),))?;
    let dumped_dict: Bound<'py, PyDict> = dumped_any.extract()?;

    Ok(dumped_dict)
}

/// Import an object from a fully qualified path
///
/// # Examples
/// ```python
/// foo_type = import_obj_from_qual_path("foo_mod.foo.Foo")
/// print(foo_type.__name__)  #> "Foo"
/// ```
pub fn import_obj_from_qual_path<'py>(
    py: Python<'py>,
    obj_import: &str,
) -> PyResult<Bound<'py, PyType>> {
    let (path, class_name) = match obj_import.rsplit_once(".") {
        Some((path, class_name)) => (path, class_name),
        None => {
            return Err(PyValueError::new_err(
                "Could not split string into module path and class name. Use format mod.mod.Class",
            ));
        }
    };

    let importlib_mod = py.import("importlib")?;
    let imported = importlib_mod.call_method1("import_module", (path,))?;
    let class_type = imported.getattr(class_name)?;

    match class_type.extract::<Bound<'py, PyType>>() {
        Ok(res) => Ok(res),
        // Getting a weird conversion wehre extract cant bounce back a Box error
        Err(what) => Err(PyValueError::new_err(format!("{:?}", what))),
    }
}

/// Get the entry point group dictionary from importlib
fn get_entry_points<'py>(py: Python<'py>, group: &str) -> PyResult<Bound<'py, PyAny>> {
    let metadata_mo = py.import("importlib.metadata")?;
    let entry_points_fn = metadata_mo.getattr("entry_points")?;
    let kwargs = PyDict::new(py);
    kwargs.set_item("group", PyString::new(py, group.into()))?;

    let entry_point_group = entry_points_fn.call((), Some(&kwargs))?;
    Ok(entry_point_group) // > importlib.metadata.EntryPoints object
}

/// Import an object from the built-in python entry points using a name and group
pub fn import_obj_from_entry_point<'py>(
    py: Python<'py>,
    name: &str,
    group: &str,
) -> PyResult<Bound<'py, PyType>> {
    let all_points_dict = get_entry_points(py, group)?;
    match all_points_dict.call_method1("__getitem__", (name,)) {
        Ok(found) => {
            let loaded = found.call_method0("load")?;
            let extracted: Bound<'py, PyType> = loaded.extract()?;
            return Ok(extracted);
        }
        Err(_) => Err(PyKeyError::new_err(format!(
            "Could not find object {} in entry point group {}",
            name, group
        ))),
    }
}

/// Check if a py object is a sequence
pub fn is_iterable<'py>(py_obj: &Bound<'py, PyAny>) -> bool {
    py_obj.is_instance_of::<PyList>()
        || py_obj.is_instance_of::<PyTuple>()
        || py_obj.is_instance_of::<PySet>()
}

pub fn is_pydantic_baseclass<'py>(py_obj: &Bound<'py, PyAny>) -> PyResult<bool> {
    let py = py_obj.py();
    // If pydantic is not a module, it can never be a pydantic baseclass
    match py.import("pydantic") {
        Ok(pydantic_mod) => {
            let baseclass = pydantic_mod.getattr("BaseModel")?;
            py_obj.is_instance(&baseclass)
        }
        Err(_) => Ok(false),
    }
}

/// Loads python code as a module and adds it to the sys modules
pub fn add_python_module_from_code<'py>(
    py: Python<'py>,
    code: &str,
    mod_name: &str,
) -> PyResult<()> {
    let module = PyModule::from_code(
        py,
        CString::new(code).unwrap().as_c_str(),
        CString::new(format!("{}.py", mod_name)).unwrap().as_c_str(),
        CString::new(mod_name).unwrap().as_c_str(),
    )
    .unwrap();

    let sys = py.import("sys")?;
    let sys_modules: Bound<'_, PyDict> = sys.getattr("modules")?.extract()?;
    sys_modules.set_item(mod_name, &module)?;

    Ok(())
}

#[cfg(test)]
mod test_utils {
    use super::*;

    mod test_import_obj {
        use super::*;

        #[test]
        fn test_import_object() {
            Python::attach(|py| {
                let actual = import_obj_from_qual_path(py, "ipaddress.IPv4Address").unwrap();
                assert_eq!(actual.name().unwrap(), "IPv4Address");
            });
        }
        #[test]
        fn test_bad_import() {
            Python::attach(|py| {
                let actual = import_obj_from_qual_path(py, "ipaddress.Bad");
                assert!(actual.is_err());
            });
        }
        #[test]
        fn test_builtins() {
            Python::attach(|py| {
                let actual = import_obj_from_qual_path(py, "builtins.float").unwrap();
                assert_eq!(actual.name().unwrap(), "float");
            });
        }
    }

    // mod test_entry_point {
    //     use pyo3::types::PyFloat;

    //     use super::*;
    //     #[test]
    //     fn test_entry_point() {
    //         Python::attach(|py| {
    //             add_entry_point(py, "test", "test", PyFloat::new(py, 5.).into_any().unbind())
    //                 .unwrap();

    //             let actual = import_obj_from_entry_point(py, "test", "test").unwrap();
    //             assert_eq!(actual.name().unwrap(), "float");
    //         });
    //     }
    // }
}
