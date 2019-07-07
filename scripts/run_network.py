#!/usr/bin/python3

import subprocess
import time
import csv

delay = 0.05
path = "./example_data/network.csv"
exec_path = "./target/release/p2pvote"

with open(path) as csvfile:
    reader = csv.reader(csvfile, delimiter=",")
    for row in reader:
        subprocess.Popen([exec_path] + row + ["headless"])
        time.sleep(delay)

