local WorkerA = {
    work = function(self, x) return x + 5 end
}
local WorkerB = {
    work = function(self, x) return x * 3 end
}

local wa = WorkerA
local wb = WorkerB
local total = 0
for i = 0, 399999 do
    local w = (i % 2 == 0) and wa or wb
    total = (total + w:work(i % 10)) % 2147483648
end

if total == 4800000 then
    print("RESULT: 4800000")
else
    print("FAIL")
end
