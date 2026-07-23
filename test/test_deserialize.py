import py_serde
import pydantic
import dataclasses


@dataclasses.dataclass
class TestFoo:
    a: int = 100
    b: list[float] = dataclasses.field(default_factory=lambda: [1.0, 2.0, 3.0])


class TestFooPydantic(pydantic.BaseModel):
    x: float = 100.0


class TestDeserialization:
    def test_dict_simple_dataclass(self):
        obj = {
            "import": "test_deserialization.TestFoo",
            "data": {"a": 500, "b": [5.0, 6.0]},
        }
        constructed = py_serde.construct_object(obj)
        assert isinstance(constructed, TestFoo)
        assert constructed.a == 100
        assert constructed.b == [5.0, 6.0]
