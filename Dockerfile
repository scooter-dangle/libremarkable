FROM ubuntu:17.10

RUN apt-get update && apt-get -y upgrade

RUN apt-get install -y \
     curl build-essential gcc-arm-linux-gnueabihf

ENV WORKDIR=/src
RUN mkdir $WORKDIR
WORKDIR $WORKDIR

RUN set -eux; \
    curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/root/.cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    PATH=/root/.cargo/bin:$PATH

RUN rustup default nightly

RUN set -eux; \
    rustup target add --toolchain nightly armv7-unknown-linux-gnueabihf

RUN mkdir --parents ~/.cargo && \
      echo '[target.armv7-unknown-linux-gnueabihf]' >> ~/.cargo/config && \
      echo 'linker = "arm-linux-gnueabihf-gcc"' >> ~/.cargo/config

ADD Cargo.toml Cargo.lock ./

RUN cargo fetch
ENV USER=root
