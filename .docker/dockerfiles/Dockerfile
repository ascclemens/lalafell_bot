FROM debian:stable

RUN apt-get update
RUN apt-get install --assume-yes --no-install-recommends \
  build-essential \
  git \
  cmake \
  pkg-config \
  automake autoconf libtool \
  libopus-dev libssl-dev libpq-dev libz-dev \
  curl ca-certificates

RUN git clone -b stable git://github.com/jedisct1/libsodium.git /root/libsodium_repo

WORKDIR /root/libsodium_repo

RUN ./autogen.sh

RUN ./configure --prefix="/root/libsodium"

RUN make install

RUN echo "export PKG_CONFIG_PATH=\"/root/libsodium/lib/pkgconfig\"" >> /root/.profile

WORKDIR /source

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly

RUN echo "source /root/.cargo/env" >> /root/.profile

CMD /bin/bash -l
