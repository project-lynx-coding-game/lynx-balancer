# Inspired by https://kerkour.com/rust-small-docker-image#/from-scratch
####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-gnu
RUN apt-get update -y && apt-get install -y pkg-config make g++ libssl-dev
RUN update-ca-certificates

# Create appuser
ENV USER=lynx-balancer
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /lynx-balancer

COPY ./Cargo.toml .
COPY ./src/ ./src/
RUN ls -la ./*

RUN cargo build --target x86_64-unknown-linux-gnu --release

####################################################################################################
## Final image
####################################################################################################
FROM gcr.io/distroless/cc

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /lynx-balancer

# Copy our build
COPY --from=builder /lynx-balancer/target/x86_64-unknown-linux-gnu/release/lynx-balancer ./
COPY --from=builder /usr/lib/x86_64-linux-gnu/libssl.so* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libcrypto.so* /usr/lib/x86_64-linux-gnu/

# Use an unprivileged user.
USER lynx-balancer:lynx-balancer

ENTRYPOINT ["/lynx-balancer/lynx-balancer"]