local caught_count = 0
local normal_sum = 0

for i = 0, 1999999 do
    local ok = pcall(function()
        normal_sum = (normal_sum + (i % 10)) % 2147483648
    end)
    if not ok then
        caught_count = caught_count + 1
    end
end

for i = 0, 19999 do
    local ok, err = pcall(function()
        error(i, 0)
    end)
    if not ok then
        caught_count = caught_count + tonumber(err)
    end
end

local checksum = (normal_sum + caught_count) % 2147483648
if checksum == 208990000 then
    print("RESULT: 208990000")
else
    print("FAIL")
end
