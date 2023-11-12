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



####################################################################################################
## Scene host
####################################################################################################
FROM ghcr.io/group-project-gut/lynx-scene-host-python:latest AS scenehost


####################################################################################################
## Final image
####################################################################################################
FROM python:3.10-slim-bookworm

RUN apt-get update -y && apt-get install -y libssl-dev

### scene host stuff
COPY --from=scenehost /scene-host /scene-host
RUN apt-get install git -y
RUN pip install -r /scene-host/requirements.txt
###

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

ENTRYPOINT ["/lynx-balancer/lynx-balancer", "--app-path", "/scene-host"]
