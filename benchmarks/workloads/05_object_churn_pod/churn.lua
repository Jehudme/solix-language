local total = 0
for i = 0, 49999 do
    local p = { x = i, y = i * 2 }
    total = (total + p.x + p.y) % 2147483648
end

if total == 1602441352 then
    print("RESULT: 1602441352")
else
    print("FAIL")
end
