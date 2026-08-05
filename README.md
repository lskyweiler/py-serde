# unpack

Complete object deserialization, including reflected type information needed to fully reconstruct the object. Basically a `pickle` for json

This was originally developed as a way to embed serialized python objects inside a rust configuration system, but can be used from python

### Python Usage
```python
import dataclasses

import pydantic

import unpack


@dataclasses.dataclass
class Foo:
    a: int = 100
    b: list[float] = dataclasses.field(default_factory=list)


# Uses pydantic model_validate for any pydantic.BaseModels
class FooPydantic(pydantic.BaseModel):
    foo: Foo


# Works for any python object, will recursively unpack as Object(**kwargs)
class MyObject:
    def __init__(self, foo_pydantic: dict[str, FooPydantic], foos: list[Foo | float]):
        self.foo_pydantic = foo_pydantic
        self.foos = foos


obj = {
    "object_import": "__main__.MyObject",
    "data": {
        "foo_pydantic": {
            "a": {
                "object_import": "__main__.FooPydantic",
                "data": {"foo": {"a": 500, "b": [7.0, 8.0]}},
            },
            "b": {
                "object_import": "__main__.FooPydantic",
                "data": {"foo": {"a": -100, "b": [100]}},
            },
        },
        "foos": [
            -10.0,
            {
                "object_import": "__main__.Foo",
                "data": {"a": 500, "b": [5.0, 6.0]},
            },
            -150.0,
            {
                "object_import": "__main__.Foo",
                "data": {"a": 700.0, "b": [100000.0]},
            },
        ],
    },
}
constructed = unpack.construct_object(obj)

print(constructed.__class__.__name__)  # > MyObject

dumped = unpack.dump_object(constructed)
#> {
#>     "object_import": "__main__.MyObject",
#>     "data": {
#>         "foo_pydantic": {
#>             "a": {
#>                 "object_import": "__main__.FooPydantic",
#>                 "data": {"foo": {"a": 500, "b": [7.0, 8.0]}},
#>             },
#>             "b": {
#>                 "object_import": "__main__.FooPydantic",
#>                 "data": {"foo": {"a": -100, "b": [100]}},
#>             },
#>         },
#>         "foos": [
#>             -10.0,
#>             {
#>                 "object_import": "__main__.Foo",
#>                 "data": {"a": 500, "b": [5.0, 6.0]},
#>             },
#>             -150.0,
#>             {
#>                 "object_import": "__main__.Foo",
#>                 "data": {"a": 700.0, "b": [100000.0]},
#>             },
#>         ],
#>     },
#> }
```


### Rust Usage
```rust
use py_unpack::prelude::*;

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
```

## Who is this for?

- If you need a way to serialize complex, heterogenous python objects and recove their type information without any bespoke logic in your models
    - plugin systems
    - complex polymorphic data types that you'd need to know the type of to pydantically validate
- This is **not** a pydantic replacement. This uses pydantic under the hood for validating Pydantic objects

## Considerations

- This will serialize/deserialize simple python objects, but anything complex should be a `pydantic.Baseclass`. This is not intended to reimplement pydantic
- Normal python objects are not validated, they will recursively be constructed based on their import type, but will not be checked against type hints