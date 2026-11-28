local base = 7
local adder = function(n)
    return n + base
end

local total = 0
for i = 0, 3999999 do
    total = (total + adder(i % 100)) % 2147483648
end

if total == 226000000 then
    print("RESULT: 226000000")
else
    print("FAIL")
end
