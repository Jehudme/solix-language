local function fib(n)
    if n <= 1 then
        return n
    end
    return fib(n - 1) + fib(n - 2)
end

local result = fib(28)
if result == 317811 then
    print("RESULT: 317811")
else
    print("FAIL")
end
