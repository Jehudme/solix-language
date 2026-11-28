local function compute(x, y)
    return x * 3 + y * 7
end

local total = 0
for i = 0, 2999999 do
    total = (total + compute(i % 100, i % 50)) % 2147483648
end

if total == 960000000 then
    print("RESULT: 960000000")
else
    print("FAIL")
end
