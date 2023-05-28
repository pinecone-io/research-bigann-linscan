
FROM ubuntu:18.04
RUN apt-get update
RUN apt-get install -y sudo build-essential git axel wget curl


# for python 3.10
RUN sudo apt install software-properties-common -y
RUN sudo add-apt-repository ppa:deadsnakes/ppa
RUN DEBIAN_FRONTEND=noninteractive TZ=Etc/UTC apt-get -y install tzdata
RUN sudo apt-get -y install python3.10
RUN apt-get -y install python3-numpy python3-scipy python3-pip

# Get Rust; NOTE: using sh for better compatibility with other base images
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Add .cargo/bin to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Check cargo is visible
# RUN cargo --help

RUN git clone https://github.com/pinecone-io/research-bigann-linscan
WORKDIR /research-bigann-linscan

# fix python3 link (required for pyo3)
RUN ln -fs /usr/bin/python3.10 /usr/bin/python3

 # fix pip3
RUN curl -sS https://bootstrap.pypa.io/get-pip.py | python3.10

RUN pip3 install maturin

# build a whl file
RUN maturin build -r

RUN pip3 install target/wheels/linscan-0.1.0-cp310-cp310-manylinux_2_27_x86_64.whl

RUN pip3 install -r requirements.txt

