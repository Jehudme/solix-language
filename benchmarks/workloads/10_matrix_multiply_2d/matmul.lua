local n = 60
local a = {}
local b = {}

for i = 0, n - 1 do
    for j = 0, n - 1 do
        a[i * n + j] = (i + j) % 100
        b[i * n + j] = (i * j) % 100
    end
end

local trace = 0
for i = 0, n - 1 do
    local c_ii = 0
    for k = 0, n - 1 do
        c_ii = (c_ii + a[i * n + k] * b[k * n + i]) % 2147483648
    end
    trace = (trace + c_ii) % 2147483648
end

if trace == 9118300 then
    print("RESULT: 9118300")
else
    print("FAIL")
end
