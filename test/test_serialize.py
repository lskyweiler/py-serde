from __future__ import annotations

import dataclasses
import enum
import datetime
import pydantic
import pathlib
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


class MyEnum(enum.Enum):
    a = 1
    b = "here"


@dataclasses.dataclass
class FooEnum:
    e: MyEnum


@dataclasses.dataclass
class MyPathWrap:
    fp: pathlib.Path


@dataclasses.dataclass
class MyDatetimeWrap:
    dt: datetime.datetime


class Private:
    def __init__(self, a: float):
        self.a = a
        self._b = a


class CustomUnpack:
    def __init__(self) -> None:
        self._x, self._y, self._z = 1, 2, 3

    def __unpack_dump__(self) -> dict:
        return {"x": self._x, "y": self._y, "z": self._z}

    @staticmethod
    def __unpack_load__(value: dict) -> CustomUnpack:
        out = CustomUnpack()
        out._x = value["x"]
        out._y = value["y"]
        out._z = value["z"]
        return out


@dataclasses.dataclass
class Nested:
    c: CustomUnpack


class TestSerialize:
    def test_simple(self):
        actual = unpack.dump_object(Foo(a=100, b=[1.0, 2.0, 3.0]))
        assert "object_import" in actual
        assert "data" in actual

    def test_complex_obj(self):
        actual = unpack.dump_object(
            MyObject(
                foo_pydantic={
                    "a": FooPydantic(foo=Foo(a=2, b=[1.5])),
                    "b": FooPydantic(foo=Foo(a=1, b=[-1.5])),
                },
                foos=[100.0, Foo(a=-1, b=[10, 11])],
            )
        )
        assert "object_import" in actual
        assert "data" in actual

    def test_serialize_deserialize(self):
        dump = unpack.dump_object(Foo(a=100, b=[1.0, 2.0, 3.0]))
        loaded = unpack.construct_object(dump)

        assert isinstance(loaded, Foo)
        assert loaded.a == 100
        assert loaded.b == [1.0, 2.0, 3.0]

    def test_serialize_enum_name(self):
        actual = unpack.dump_object(
            FooEnum(e=MyEnum.a), unpack.PyDumpConfig(use_enum_name=True)
        )
        assert actual["data"]["e"]["data"]["name"] == "a"
        assert actual["data"]["e"]["object_import"] == "test_serialize.MyEnum"

        loaded = unpack.construct_object(actual)
        assert isinstance(loaded, FooEnum)
        assert loaded.e == MyEnum.a

    def test_serialize_enum_value(self):
        actual = unpack.dump_object(
            FooEnum(e=MyEnum.a), unpack.PyDumpConfig(use_enum_name=False)
        )
        assert actual["data"]["e"]["data"]["value"] == 1
        assert actual["data"]["e"]["object_import"] == "test_serialize.MyEnum"

        loaded = unpack.construct_object(actual)
        assert isinstance(loaded, FooEnum)
        assert loaded.e == MyEnum.a

    def test_serialize_base_value(self):
        actual = unpack.dump_object(100.0)
        assert actual == 100.0

    def test_no_privates(self):
        actual = unpack.dump_object(
            Private(a=100.0), unpack.PyDumpConfig(dump_privates=False)
        )
        assert actual["data"]["a"] == 100.0
        assert "_b" not in actual["data"]

    def test_privates(self):
        class Private:
            def __init__(self, a: float):
                self.a = a
                self._b = a

        actual = unpack.dump_object(
            Private(a=100.0), unpack.PyDumpConfig(dump_privates=True)
        )
        assert actual["data"]["a"] == 100.0
        assert actual["data"]["_b"] == 100.0

    def test_pathlib(self):
        actual = unpack.dump_object(MyPathWrap(pathlib.Path(__file__)))
        assert actual["data"]["fp"]["data"]["path"] == __file__
        assert actual["data"]["fp"]["object_import"] == "pathlib.Path"

        actual_constructed = unpack.construct_object(actual)
        assert actual_constructed.fp == pathlib.Path(__file__)

    def test_datetime(self):
        dt = datetime.datetime.now()
        actual = unpack.dump_object(MyDatetimeWrap(dt=dt))
        assert actual["data"]["dt"]["data"]
        assert actual["data"]["dt"]["object_import"] == "datetime.datetime"

        actual_constructed = unpack.construct_object(actual)
        assert actual_constructed.dt == dt

    def test_custom(self):
        actual = unpack.dump_object(CustomUnpack())
        assert actual["data"] == {"x": 1, "y": 2, "z": 3}

        loaded = unpack.construct_object(actual)
        assert loaded._x == 1
        assert loaded._y == 2
        assert loaded._z == 3

        actual = unpack.dump_object(Nested(c=CustomUnpack()))
        assert actual["data"]["c"]["data"] == {"x": 1, "y": 2, "z": 3}
