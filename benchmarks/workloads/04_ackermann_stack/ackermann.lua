local function ack(m, n)
    if m == 0 then
        return n + 1
    end
    if n == 0 then
        return ack(m - 1, 1)
    end
    return ack(m - 1, ack(m, n - 1))
end

local result = ack(3, 6)
if result == 509 then
    print("RESULT: 509")
else
    print("FAIL")
end
