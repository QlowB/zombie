#!/usr/bin/python3

import os
from timeit import default_timer as timer
import numpy as np
from subprocess import check_output


nTries = 10
tries = []

for i in range(1, nTries):
    start = timer()
    os.system("./target/release/zombie examples/mandel.bf > /dev/null")
    end = timer()
    tries.append((end - start) * 1000)


print("benchmark for commit: " + str(check_output(["git", "rev-parse", "HEAD"])))
print(str(nTries) + " tries were run")
print("")
print("average time [ms]: " + str(np.mean(tries)))
print("")
print("max time [ms]:     " + str(np.max(tries)))
print("min time [ms]:     " + str(np.min(tries)))
print("variance:          " + str(np.var(tries)))

