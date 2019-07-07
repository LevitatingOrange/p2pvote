#!/usr/bin/python3

import random
import csv


n = 1000
path = "./example_data/id_pool.csv"

randomIds = set()
while len(randomIds) < n:
    randomIds.add(random.randint(0, 4294967296))

with open(path, "w") as csvfile:
    writer = csv.writer(csvfile, delimiter=",")
    for key in randomIds:
        writer.writerow([key])

