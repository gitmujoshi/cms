FROM amazonlinux:2

# Install dependencies
RUN yum update -y && \
    yum install -y \
    gcc \
    gcc-c++ \
    make \
    cmake \
    git \
    wget \
    unzip \
    python3 \
    python3-pip \
    && yum clean all

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install libtorch
RUN wget https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-1.13.1%2Bcpu.zip && \
    unzip libtorch-cxx11-abi-shared-with-deps-1.13.1+cpu.zip && \
    mv libtorch /usr/local/ && \
    rm libtorch-cxx11-abi-shared-with-deps-1.13.1+cpu.zip

# Set environment variables
ENV LD_LIBRARY_PATH=/usr/local/libtorch/lib:$LD_LIBRARY_PATH
ENV LIBRARY_PATH=/usr/local/libtorch/lib:$LIBRARY_PATH
ENV CPATH=/usr/local/libtorch/include:/usr/local/libtorch/include/torch/csrc/api/include:$CPATH

# Create app directory
WORKDIR /app

# Copy source code
COPY . .

# Build the application
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN cargo build --release

# Run the application
CMD ["./target/release/enclave-runtime"] 