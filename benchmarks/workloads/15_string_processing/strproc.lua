local function process(s, factor)
    if s == "alpha" then return factor * 2 end
    if s == "beta" then return factor * 3 end
    if s == "gamma" then return factor * 5 end
    return factor
end

local words = { "alpha", "beta", "gamma" }
local total = 0
for i = 0, 1499999 do
    local w = words[(i % 3) + 1]
    total = (total + process(w, i % 20)) % 2147483648
end

if total == 47500000 then
    print("RESULT: 47500000")
else
    print("FAIL")
end
