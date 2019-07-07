#!/usr/bin/python3

import random
import csv
import subprocess
import time

pool_path = "./example_data/id_pool.csv"
vote_mapping_path = "./example_data/vote_mappings.csv"
out_path = "./example_data/bulk.csv"
number_of_votes = 2000

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
        writer.writerow([random.choice(ids), random.randint(0, votelen-1)])
