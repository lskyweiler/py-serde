import dataclasses
import enum
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


class MyEnum(enum.Enum):
    a = 1
    b = "here"


@dataclasses.dataclass
class FooEnum:
    e: MyEnum


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
