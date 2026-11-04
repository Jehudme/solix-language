local Box = {
    new = function(self, val)
        return { value = val, get_val = function(s) return s.value end }
    end
}

local total = 0
for i = 0, 49999 do
    local b1 = Box:new(i)
    local b2 = Box:new(i * 4 + 7)
    total = (total + b1:get_val() + b2:get_val()) % 2147483648
end

if total == 1955257704 then
    print("RESULT: 1955257704")
else
    print("FAIL")
end
