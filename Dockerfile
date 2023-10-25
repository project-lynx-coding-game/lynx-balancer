# Inspired by https://kerkour.com/rust-small-docker-image#/from-scratch
####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-gnu
RUN apt-get update -y && apt-get install -y pkg-config make g++ libssl-dev
RUN update-ca-certificates

WORKDIR /lynx-balancer

COPY ./Cargo.toml .
COPY ./src/ ./src/
RUN ls -la ./*

RUN cargo build --target x86_64-unknown-linux-gnu --release

# TODO: add pulling scene host image, unpacking to directory and copying to final image
# also install in final image lynx common through pip, along with uvicorn

####################################################################################################
## Final image
####################################################################################################
FROM debian:bookworm-slim

RUN apt-get update -y && apt-get install -y libssl-dev

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

# Copy our build
COPY --from=builder /lynx-balancer/target/x86_64-unknown-linux-gnu/release/lynx-balancer ./

# Use an unprivileged user.
USER lynx-balancer:lynx-balancer

ENTRYPOINT ["/lynx-balancer/lynx-balancer"]
