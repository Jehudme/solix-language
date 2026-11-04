local base = 7
local adder = function(n)
    return n + base
end

local total = 0
for i = 0, 49999 do
    total = (total + adder(i % 100)) % 2147483648
end

if total == 2825000 then
    print("RESULT: 2825000")
else
    print("FAIL")
end
