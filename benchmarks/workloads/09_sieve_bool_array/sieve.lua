local limit = 1000000
local is_prime = {}
for i = 2, limit do
    is_prime[i] = true
end

local p = 2
while p * p <= limit do
    if is_prime[p] then
        for m = p * p, limit, p do
            is_prime[m] = false
        end
    end
    p = p + 1
end

local count = 0
for i = 2, limit do
    if is_prime[i] then
        count = count + 1
    end
end

if count == 78498 then
    print("RESULT: 78498")
else
    print("FAIL")
end
