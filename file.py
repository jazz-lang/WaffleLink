import time
start = time.time()
i = 0
while i < 100000:
    i = i + 1
print("Result is: ",i)
end = time.time()
print("run in: ",(end - start)*1000,"ms")