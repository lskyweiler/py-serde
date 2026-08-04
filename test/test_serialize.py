import dataclasses
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


class TestSerialize:
    def test_simple(self):
        actual = unpack.dump_object(Foo(a=100, b=[1.0, 2.0, 3.0]))
        assert "object_import" in actual
        assert "data" in actual

    def test_serialize_deserialize(self):
        dump = unpack.dump_object(Foo(a=100, b=[1.0, 2.0, 3.0]))
        loaded = unpack.construct_object(dump)

        assert isinstance(loaded, Foo)
        assert loaded.a == 100
        assert loaded.b == [1.0, 2.0, 3.0]
