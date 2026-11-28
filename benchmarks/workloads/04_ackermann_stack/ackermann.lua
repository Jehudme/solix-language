local function ack(m, n)
    if m == 0 then
        return n + 1
    end
    if n == 0 then
        return ack(m - 1, 1)
    end
    return ack(m - 1, ack(m, n - 1))
end

local total = 0
for rep = 1, 15 do
    total = (total + ack(3, 6)) % 2147483648
end
if total == 7635 then
    print("RESULT: 7635")
else
    print("FAIL")
end
