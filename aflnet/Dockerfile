FROM ubuntu:18.04

RUN apt-get -y update && \
    apt-get -y install \
        apt-utils \
        build-essential \
        openssl \
        clang \
        graphviz-dev \
        libcap-dev \
        git \
        libpcre2-dev \
        libdb5.3 \
        libdb5.3-dev

# Download and compile AFLNet
ENV LLVM_CONFIG="llvm-config-6.0"

RUN git clone --depth=1 https://github.com/aflnet/aflnet && \
    cd /aflnet && \
    make clean all && \
    cd llvm_mode && \
    make

# Set up environment variables for AFLNet
ENV AFLNET="/aflnet"
ENV PATH="${PATH}:${AFLNET}"
ENV AFL_PATH="${AFLNET}"
ENV AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1
ENV AFL_SKIP_CPUFREQ=1

RUN useradd exim && \
    mkdir /result && \
    git clone --depth=1 https://github.com/pd-fkie/exim-fuzzer && \
    cd exim-fuzzer && \
    sed -r 's/pd/exim/' exim.conf > /result/exim.conf && \
    git submodule update --depth=1 --init ./Exim && \
    cd ./Exim && \
    git apply ../patches/base.diff && \
    git apply ../aflnet/patch.diff && \
    mkdir src/Local && \
    cp ../aflnet/LocalMakefile src/Local/Makefile && \
    export EXIM_RELEASE_VERSION="aflnet-fuzz" && \
    make -C src

RUN mkdir corpus && \
    /bin/echo -en 'EHLO fuzz\r\nAUTH PLAIN\r\nAHRlc3QAdGVzdA==\r\nMAIL FROM:<as@df>\r\nRCPT TO:<exim@localhost>\r\nDATA\r\n...email...\r\n.\r\nQUIT\r\n' > corpus/msg

COPY fuzz.sh /
RUN chmod +x /fuzz.sh

