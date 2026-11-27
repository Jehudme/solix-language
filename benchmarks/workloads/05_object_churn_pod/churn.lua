local total = 0
for i = 0, 299999 do
    local p = { x = i, y = i * 2 }
    total = (total + p.x + p.y) % 2147483648
end

if total == 1855563824 then
    print("RESULT: 1855563824")
else
    print("FAIL")
end
