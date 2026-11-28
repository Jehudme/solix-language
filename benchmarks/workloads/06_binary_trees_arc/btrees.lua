local function build_tree(depth, item)
    local node = { item = item, left = nil, right = nil }
    if depth > 0 then
        node.left = build_tree(depth - 1, item * 2)
        node.right = build_tree(depth - 1, item * 2 + 1)
    end
    return node
end

local function check_tree(node)
    if not node then
        return 0
    end
    return (node.item + check_tree(node.left) + check_tree(node.right)) % 2147483648
end

local total = 0
for i = 0, 9 do
    local root = build_tree(10, i + 1)
    total = (total + check_tree(root)) % 2147483648
end

if total == 687761806 then
    print("RESULT: 687761806")
else
    print("FAIL")
end
