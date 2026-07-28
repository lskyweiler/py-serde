from __future__ import annotations

import dataclasses
import ipaddress

import pydantic

import unpack


@dataclasses.dataclass
class Foo:
    a: int = 100
    b: list[float] = dataclasses.field(default_factory=list)


class FooPydantic(pydantic.BaseModel):
    foo: Foo


class MyObject:
    def __init__(self, foo_pydantic: dict[str, FooPydantic], foos: list[Foo | float]):
        self.foo_pydantic = foo_pydantic
        self.foos = foos


class TestDeserialization:
    def test_builtin(self):
        obj = {
            "object_import": "ipaddress.IPv4Address",
            "data": {"address": "127.0.0.1"},
        }
        constructed = unpack.construct_object(obj)
        assert isinstance(constructed, ipaddress.IPv4Address)
        assert str(constructed) == "127.0.0.1"

    def test_dict_simple_dataclass(self):
        obj = {
            "object_import": "test_deserialize.Foo",
            "data": {"a": 500, "b": [5.0, 6.0]},
        }
        constructed = unpack.construct_object(obj)
        assert isinstance(constructed, Foo)
        assert constructed.a == 500
        assert constructed.b == [5.0, 6.0]

    def test_pydantic_obj(self):
        obj = {
            "object_import": "test_deserialize.FooPydantic",
            "data": {"foo": {"a": 500, "b": [7.0, 8.0]}},
        }
        constructed = unpack.construct_object(obj)
        assert isinstance(constructed, FooPydantic)
        assert isinstance(constructed.foo, Foo)

        assert constructed.foo.b == [7.0, 8.0]

    def test_custom_config(self):
        obj = {
            "my_import": "test_deserialize.FooPydantic",
            "my_data": {"foo": {"a": 500, "b": [7.0, 8.0]}},
        }
        constructed = unpack.construct_object(
            obj,
            unpack.PyImportConfig(object_import_key="my_import", data_key="my_data"),
        )
        assert isinstance(constructed, FooPydantic)
        assert isinstance(constructed.foo, Foo)

        assert constructed.foo.b == [7.0, 8.0]

    def test_complex_recursive(self):
        obj = {
            "object_import": "test_deserialize.MyObject",
            "data": {
                "foo_pydantic": {
                    "a": {
                        "object_import": "test_deserialize.FooPydantic",
                        "data": {"foo": {"a": 500, "b": [7.0, 8.0]}},
                    },
                    "b": {
                        "object_import": "test_deserialize.FooPydantic",
                        "data": {"foo": {"a": -100, "b": [100]}},
                    },
                },
                "foos": [
                    -10.0,
                    {
                        "object_import": "test_deserialize.Foo",
                        "data": {"a": 500, "b": [5.0, 6.0]},
                    },
                    -150.0,
                    {
                        "object_import": "test_deserialize.Foo",
                        "data": {"a": 700.0, "b": [100000.0]},
                    },
                ],
            },
        }
        constructed = unpack.construct_object(obj)
        assert isinstance(constructed, MyObject)
        assert len(constructed.foo_pydantic) == 2

        assert isinstance(constructed.foo_pydantic["a"], FooPydantic)
        assert constructed.foo_pydantic["a"].foo.a == 500
        assert constructed.foo_pydantic["a"].foo.b == [7.0, 8.0]
        assert isinstance(constructed.foo_pydantic["b"], FooPydantic)
        assert constructed.foo_pydantic["b"].foo.a == -100
        assert constructed.foo_pydantic["b"].foo.b == [100]

        assert len(constructed.foos) == 4

        assert constructed.foos[0] == -10.0
        assert isinstance(constructed.foos[1], Foo)
        assert constructed.foos[1].a == 500
        assert constructed.foos[1].b == [5.0, 6.0]
        assert constructed.foos[2] == -150.0
        assert isinstance(constructed.foos[3], Foo)
        assert constructed.foos[3].a == 700.0
        assert constructed.foos[3].b == [100000.0]
