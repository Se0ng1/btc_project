FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt update && apt install -y \
    libevent-dev \
    libsqlite3-dev \
    autoconf \
    automake \
    libtool \
    pkg-config \
    build-essential \
    bison \
    curl \
    wget \
    unzip \
    git \
    ca-certificates

# boost 설치
WORKDIR /root
ENV BOOST_VER=1_83_0
ENV BOOST_DIR=boost_${BOOST_VER}

RUN wget -c https://sourceforge.net/projects/boost/files/boost/1.83.0/boost_1_83_0.tar.bz2 \
    && tar -xvf boost_1_83_0.tar.bz2 \
    && cd ${BOOST_DIR} \
    && ./bootstrap.sh \
    && ./b2 install \
    && cd .. \
    && rm -rf ${BOOST_DIR} boost_1_83_0.tar.bz2

# bitcoin-core-cat 빌드
RUN git clone --depth 1 --branch dont-success-cat https://github.com/rot13maxi/bitcoin.git bitcoin-core-cat \
    && cd bitcoin-core-cat \
    && ./autogen.sh \
    && ./configure --enable-wallet --with-sqlite --with-incompatible-bdb --without-gui --without-tests --disable-bench \
    && make -j"$(nproc)"

COPY build.sh /root/build.sh
RUN chmod +x /root/build.sh

CMD ["/root/build.sh"]
