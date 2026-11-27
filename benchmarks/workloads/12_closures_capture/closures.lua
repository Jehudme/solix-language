local base = 7
local adder = function(n)
    return n + base
end

local total = 0
for i = 0, 399999 do
    total = (total + adder(i % 100)) % 2147483648
end

if total == 22600000 then
    print("RESULT: 22600000")
else
    print("FAIL")
end
