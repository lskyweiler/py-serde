from __future__ import annotations

import dataclasses
import ipaddress

import pydantic

import unpack


@dataclasses.dataclass
class Foo:
    a: int = 100
    b: list[float] = dataclasses.field(default_factory=lambda: [1.0, 2.0, 3.0])


class FooPydantic(pydantic.BaseModel):
    foo: Foo


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
