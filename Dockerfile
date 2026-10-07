# syntax=docker/dockerfile:1

# Cross-compile Dockerfile supporting both x86_64-unknown-linux-musl and
# aarch64-unknown-linux-musl targets using zig to link against musl libc. Note
# that this is to be used from an x86_64 host.
#
# The final images are based on scratch and contain the statically linked
# `weave` binary and nothing else. They carry no CA store, so the URL clipper
# needs the host's mounted, e.g. `-v /etc/ssl/certs:/etc/ssl/certs:ro`.

# --- build image

FROM rust:1.95 AS builder

# rustls uses the aws-lc-rs provider, whose C sources are built with CMake.
RUN apt-get update && \
    apt-get install --yes --no-install-recommends cmake && \
    rm -rf /var/lib/apt/lists/*

RUN rustup target add \
    aarch64-unknown-linux-musl \
    x86_64-unknown-linux-musl

ENV ZIGVERSION=0.15.2

RUN ARCH=$(uname -m) && \
    if [ "$ARCH" = "aarch64" ]; then ZIGARCH="aarch64"; else ZIGARCH="x86_64"; fi && \
    wget https://ziglang.org/download/$ZIGVERSION/zig-$ZIGARCH-linux-$ZIGVERSION.tar.xz && \
    tar -C /usr/local --strip-components=1 -xf zig-$ZIGARCH-linux-$ZIGVERSION.tar.xz && \
    mv /usr/local/zig /usr/local/bin && \
    rm zig-$ZIGARCH-linux-$ZIGVERSION.tar.xz

RUN cargo install --locked cargo-zigbuild

# The workspace profile only strips debug info and a scratch image brings no
# tooling to strip a binary afterwards, so drop all symbols while building.
ENV CARGO_PROFILE_RELEASE_STRIP=true

WORKDIR /app

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo zigbuild \
    --locked \
    --release \
    --target aarch64-unknown-linux-musl \
    --target x86_64-unknown-linux-musl \
    --bin weave

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "10001" \
    "app" && \
    mkdir /notebook && \
    chown 10001:10001 /notebook


# --- x86_64-unknown-linux-musl final image

FROM scratch AS amd64

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder --chown=10001:10001 /notebook /notebook

WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/weave ./
USER app:app
ENV ZK_NOTEBOOK_DIR=/notebook \
    WEAVE_HOST=0.0.0.0
EXPOSE 8000
CMD ["/app/weave"]

# --- aarch64-unknown-linux-musl final image

FROM scratch AS arm64

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder --chown=10001:10001 /notebook /notebook

WORKDIR /app
COPY --from=builder /app/target/aarch64-unknown-linux-musl/release/weave ./
USER app:app
ENV ZK_NOTEBOOK_DIR=/notebook \
    WEAVE_HOST=0.0.0.0
EXPOSE 8000
CMD ["/app/weave"]
