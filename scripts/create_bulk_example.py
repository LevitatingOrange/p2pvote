#!/usr/bin/python3

import random
import csv
import subprocess
import time
import scipy.stats as ss
import numpy as np
import matplotlib.pyplot as plt

x = np.arange(-2, 3)
xU, xL = x + 0.5, x - 0.5 
prob = ss.norm.cdf(xU, scale = 1) - ss.norm.cdf(xL, scale = 1)
prob = prob / prob.sum() #normalize the probabilities so their sum is 1
nums = np.random.choice(x, size = 10000, p = prob)
plt.hist(nums, bins = len(x))
plt.savefig("vote_dist.png")

pool_path = "./example_data/id_pool.csv"
vote_mapping_path = "./example_data/vote_mappings.csv"
out_path = "./example_data/bulk.csv"
number_of_votes = 1000

ids = []
with open(pool_path) as csvfile:
    reader = csv.reader(csvfile)
    for row in reader:
        ids.append(int(row[0]))

votelen = 0
with open(vote_mapping_path) as csvfile:
    votelen = len(list(csv.reader(csvfile)))

with open(out_path, "w") as csvfile:
    writer = csv.writer(csvfile, delimiter=",")
    writer.writerow(["key", "vote"])
    for _ in range(number_of_votes):
        writer.writerow([random.choice(ids), random.choice(nums) + 2])
