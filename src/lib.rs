mod deserialize;
mod py_import_object;

use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
mod py_serde {
    use pyo3::prelude::*;

    /// Formats the sum of two numbers as string.
    #[pyfunction]
    fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
        Ok((a + b).to_string())
    }
}


pub mod prelude {
    use super::*;

    pub use deserialize::PyObjectDeserializer;
}