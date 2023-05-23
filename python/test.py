# a basic python file to test the python api

from tools.io import read_sparse_matrix
from linscan import LinscanIndex
from tqdm import tqdm
import time

filename = '../data/base_small.csr'
d = read_sparse_matrix(filename)

index = LinscanIndex(True)
print('Inserting vectors into index:')
for i in tqdm(range(d.shape[0])):
    d1 = d.getrow(i)
    index.insert(dict(zip(d1.indices, d1.data)))
print(index)

index.finalize()

print("reading queries file..")
queries_file = '../data/queries.dev.csr'
queries = read_sparse_matrix(queries_file)
print(queries.shape)
nq = queries.shape[0]
print("running search queries:")

start = time.time()
for i in tqdm(range(nq)):
    qc = queries.getrow(i)
    q = dict(zip(qc.indices, qc.data))
    res = index.retrieve(q, 10)

end = time.time()
elapsed = end - start

print(f'Single thread: Elapsed time: {elapsed}; {round(nq/elapsed, 2)} QPS')
