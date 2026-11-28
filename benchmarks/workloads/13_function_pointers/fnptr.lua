local function fn1(x) return x * 2 + 1 end
local function fn2(x) return x * 3 - 2 end
local function fn3(x) return x * 5 + 4 end

local total = 0
for i = 0, 1999999 do
    local idx = i % 3
    local f = (idx == 0) and fn1 or ((idx == 1) and fn2 or fn3)
    total = (total + f(i % 50)) % 2147483648
end

if total == 165333298 then
    print("RESULT: 165333298")
else
    print("FAIL")
end
