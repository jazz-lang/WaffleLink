local start = os.clock();
local i = 0
while i < 100000 do
    i = i + 1
end
print("Result is: "..i)
local end_ = os.clock();
print(string.format("Elapsed: %f\n",end_ - start))