# Python sample for tree-sitter parsing tests
import json
from dataclasses import dataclass
from typing import Optional


@dataclass
class Item:
    id: int
    name: str
    price: float


class ShoppingCart:
    def __init__(self):
        self.items: list[Item] = []

    def add_item(self, item: Item) -> None:
        self.items.append(item)

    def total(self) -> float:
        return sum(item.price for item in self.items)

    def to_json(self) -> str:
        return json.dumps(
            [{"id": i.id, "name": i.name, "price": i.price} for i in self.items]
        )
