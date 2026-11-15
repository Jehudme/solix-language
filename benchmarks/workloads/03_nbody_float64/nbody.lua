local x = 0.5
local y = 1.2
local vx = 0.01
local vy = 0.02

for i = 1, 50000 do
    local dist_sq = x * x + y * y + 0.1
    local force = 1.0 / dist_sq
    vx = vx + force * 0.001
    vy = vy + force * 0.001
    x = x + vx
    y = y + vy
end

local energy = math.floor(x * 100.0) + math.floor(y * 100.0)
if energy == 358036 then
    print("RESULT: 358036")
else
    print("FAIL")
end
