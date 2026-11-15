class Node:
    __slots__ = ("item", "left", "right")

    def __init__(self, item: int):
        self.item = item
        self.left = None
        self.right = None

def build_tree(depth: int, item: int) -> Node:
    node = Node(item)
    if depth > 0:
        node.left = build_tree(depth - 1, item * 2)
        node.right = build_tree(depth - 1, item * 2 + 1)
    return node

def check_tree(node) -> int:
    if node is None:
        return 0
    return (node.item + check_tree(node.left) + check_tree(node.right)) & 0x7FFFFFFF

if __name__ == "__main__":
    total = 0
    for i in range(10):
        root = build_tree(10, i + 1)
        total = (total + check_tree(root)) & 0x7FFFFFFF

    if total == 83875825:
        print("RESULT: 83875825")
    else:
        print("FAIL")
