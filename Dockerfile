FROM ubuntu:20.04

SHELL ["/bin/bash", "-c"]

# Get basic dependencies for rust + rsp2
#   DEBIAN_FRONTEND: https://serverfault.com/a/992421
#   pkg-config: for finding libffi later on
#   wget: for installing rustup
RUN apt-get update \
    && DEBIAN_FRONTEND="noninteractive" \
        apt-get install --yes --no-install-recommends \
            clang \
            cmake \
            g++ \
            gfortran \
            libffi-dev \
            libopenblas-dev \
            make \
            pkg-config \
            python3-venv \
            python3.8-dev \
            wget \
    && rm -rf /var/lib/apt/lists/*

# Rust install, taken from https://github.com/rust-lang/docker-rust/blob/master/1.48.0/buster/Dockerfile
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.48.0

RUN set -eu; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='49c96f3f74be82f4752b8bffcf81961dea5e6e94ce1ccba94435f12e871c3bdb' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='5a2be2919319e8778698fa9998002d1ec720efe7cb4f6ee4affb006b5e73f1be' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='d93ef6f91dab8299f46eef26a56c2d97c66271cea60bf004f2f088a86a697078' ;; \
        i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='e3d0ae3cfce5c6941f74fed61ca83e53d4cd2deb431b906cbd0687f246efede4' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.22.1/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME;

WORKDIR /app

# Setup python virtual environment
#   Note that we install numpy first because phonopy gets angry otherwise
COPY requirements.txt .
RUN python3 -m venv .venv \
    && source .venv/bin/activate \
    && pip install --upgrade pip wheel \
    && pip install -r <(grep numpy requirements.txt) \
    && pip install -r requirements.txt

# Set python environment variables so we don't need to activate the venv
ENV VIRTUAL_ENV=/app/.venv \
    PATH=/app/.venv/bin:$PATH

# Install rsp2. The cp command is needed because cargo doesn't install liblammps.so
# and there doesn't seem to be an easy way to deal with this besides using cargo run.
COPY rsp2 rsp2
RUN cd rsp2 \
    && cargo install --no-default-features --path . \
    && cp target/release/build/lammps-sys-*/out/lib/liblammps.so.0 /usr/local/lib \
    && cargo clean

# We don't support lammps potentials, but we can't disable it in rsp2 so just
# set this to a dummy value to appease rsp2. We also need to set LD_LIBRARY_PATH
# to pick up the lammps library.
ENV LAMMPS_POTENTIALS=/dev/null \
    LD_LIBRARY_PATH=/usr/local/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}

# Install generation code
COPY generation generation
RUN cd generation \
    && cargo install --path . \
    && cargo clean

ENTRYPOINT ["/bin/sh", "-c"]
CMD ["bash"]
