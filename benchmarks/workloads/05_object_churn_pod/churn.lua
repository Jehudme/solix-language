local total = 0
for i = 0, 299999 do
    local p = { x = i, y = i * 2 }
    total = (total + p.x + p.y) % 2147483648
end

if total == 913288672 then
    print("RESULT: 913288672")
else
    print("FAIL")
end
