#!/usr/bin/python3

import random
import csv


n = 999
baseport = 9000
path = "./example_data/network.csv"
ip = "127.0.0.1:"

randomIds = set()
while len(randomIds) < n:
    randomIds.add(random.randint(0, 4294967296))

randomIds = list(randomIds)

peers = [(ip+"9000", str(0), ip+"9033", str(randomIds[0]))]

for i, peer_id in enumerate(randomIds):
    other_peer = random.choice(peers)
    peers.append((ip + str(baseport + 33*(i+1)), str(peer_id), other_peer[0] , other_peer[1]))

# with open(path, 'w') as csvfile:
#     writer = csv.writer(csvfile, delimiter=',')
#     for peer in peers:
#         writer.writerow(peer)
with open(path, "w") as csvfile:
    writer = csv.writer(csvfile, delimiter=",")
    for peer in peers:
        writer.writerow(peer)

