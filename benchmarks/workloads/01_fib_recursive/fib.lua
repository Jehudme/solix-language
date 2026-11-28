local function fib(n)
    if n <= 1 then
        return n
    end
    return fib(n - 1) + fib(n - 2)
end

local result = fib(32)
if result == 2178309 then
    print("RESULT: 2178309")
else
    print("FAIL")
end
