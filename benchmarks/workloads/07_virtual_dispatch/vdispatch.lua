local Circle = {
    get_area = function(self, factor) return factor * 2 end
}
local Square = {
    get_area = function(self, factor) return factor * 3 end
}
local Triangle = {
    get_area = function(self, factor) return factor * 4 end
}

local shapes = { Circle, Square, Triangle }
local total = 0
for i = 0, 99999 do
    local s = shapes[(i % 3) + 1]
    total = (total + s:get_area(i % 10)) % 2147483648
end

if total == 1349997 then
    print("RESULT: 1349997")
else
    print("FAIL")
end
