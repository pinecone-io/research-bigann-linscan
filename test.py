# a basic python file to test the python api

# the files for this test can be downloaded from the following gcs location:
# https://storage.googleapis.com/ann-challenge-sparse-vectors/csr/base_small.csr.gz
# https://storage.googleapis.com/ann-challenge-sparse-vectors/csr/queries.dev.csr.gz
# (use, e.g. wget and gunzip)

from tools.io import read_sparse_matrix
from pylinscan import LinscanIndex
from tqdm import tqdm
import time

filename = 'data/base_small.csr'
d = read_sparse_matrix(filename)

index = LinscanIndex()
print('Inserting vectors into index:')
for i in tqdm(range(d.shape[0])):
    d1 = d.getrow(i)
    index.insert(dict(zip(d1.indices, d1.data)))
print(index)

print("reading queries file..")
queries_file = 'data/queries.dev.csr'
queries = read_sparse_matrix(queries_file)
print(queries.shape)
nq = queries.shape[0]
print("running search queries:")

q_vec = []
for i in range(nq):
    qc = queries.getrow(i)
    q = dict(zip(qc.indices, qc.data))
    q_vec.append(q)

start = time.time()

res_vec = index.retrieve_parallel(q_vec, 10, 10)

end = time.time()
elapsed = end - start

print(f'Parallel queries issued. Elapsed time: {elapsed}; {round(nq/elapsed, 2)} QPS')

