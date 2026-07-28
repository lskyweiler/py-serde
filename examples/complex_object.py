from __future__ import annotations

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
