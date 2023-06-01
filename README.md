# research-bigann-linscan

Build instructions using docker:

- Build the docker image: `docker build -t linscan .`
- Run the docker:  `docker run -it linscan`
- Within the docker container: 
  - download some data: `source get_data.sh`
  - run a test script to run the sparse index: `python3 test.py`.

Expected result:
```
root@e2ee469e416a:/research-bigann-linscan# source get_data.sh 
--2023-06-01 11:45:44--  https://storage.googleapis.com/ann-challenge-sparse-vectors/csr/base_small.csr.gz
Resolving storage.googleapis.com (storage.googleapis.com)... 142.251.37.80, 142.251.142.208, 172.217.22.16, ...
Connecting to storage.googleapis.com (storage.googleapis.com)|142.251.37.80|:443... connected.
HTTP request sent, awaiting response... 200 OK
Length: 67426902 (64M) [application/x-gzip]
Saving to: 'base_small.csr.gz'

base_small.csr.gz                                                    100%[=====================================================================================================================================================================>]  64.30M  20.0MB/s    in 3.2s    

2023-06-01 11:45:47 (20.0 MB/s) - 'base_small.csr.gz' saved [67426902/67426902]

--2023-06-01 11:45:47--  https://storage.googleapis.com/ann-challenge-sparse-vectors/csr/queries.dev.csr.gz
Resolving storage.googleapis.com (storage.googleapis.com)... 142.251.37.80, 142.251.142.208, 172.217.22.16, ...
Connecting to storage.googleapis.com (storage.googleapis.com)|142.251.37.80|:443... connected.
HTTP request sent, awaiting response... 200 OK
Length: 1849192 (1.8M) [application/x-gzip]
Saving to: 'queries.dev.csr.gz'

queries.dev.csr.gz                                                   100%[=====================================================================================================================================================================>]   1.76M  --.-KB/s    in 0.1s    

2023-06-01 11:45:48 (14.2 MB/s) - 'queries.dev.csr.gz' saved [1849192/1849192]

root@e2ee469e416a:/research-bigann-linscan# python3 test.py 
Initializing a new LinscanIndex.
Inserting vectors into index:
100%|███████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████| 100000/100000 [00:05<00:00, 17104.90it/s]
Linscan Index [100000 documents, 27197 unique tokens, avg. nnz: 127.29954]
reading queries file..
(6980, 30109)
running search queries:
Parallel queries issued. Elapsed time: 0.4776911735534668; 14611.95 QPS
```
