local total = 0
local has_bit, bit = pcall(require, "bit")

if has_bit then
    local band = bit.band
    local bxor = bit.bxor
    for i = 0, 7999999 do
        total = band(total + bxor(i, i % 7), 0x7FFFFFFF)
    end
else
    local load_fn = load([[
        local t = 0
        for i = 0, 7999999 do
            t = (t + (i ~ (i % 7))) & 0x7FFFFFFF
        end
        return t
    ]])
    total = load_fn()
end

if total == 342161131 then
    print("RESULT: 342161131")
else
    print("FAIL")
end
