from linscan import LinscanIndex

ind = LinscanIndex()

v1 = {0: 0.4, 4: 0.6, 1234: -0.666}
v2 = {1: 0.4, 4: 10, 123456: -0.666}

ind.insert(v1)
ind.insert(v2)

print("Index built:", ind)

q = {2: 0.4, 4: 0.1, 5678: 1.33}

print("some search results:")

r = ind.retrieve(q, 1)

print(r)
assert (r == [1])
