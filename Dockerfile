ARG TARGET=aarch64-unknown-linux-musl

FROM messense/rust-musl-cross:aarch64-musl AS builder
ARG TARGET
WORKDIR /usr/src
RUN rustup target add $TARGET

RUN USER=root cargo new vult-server
WORKDIR /usr/src/vult-server
COPY Cargo.toml Cargo.lock ./
RUN cargo build --target $TARGET --release
RUN sleep 1s

COPY src ./src
RUN echo " " >> ./src/main.rs
RUN cargo build --target $TARGET --release
# RUN cargo install --target $TARGET --root ./install --path .

FROM alpine:3.15.1
ARG TARGET
COPY --from=builder /usr/src/vult-server/target/$TARGET/release/vult-server ./
# COPY --from=builder /usr/src/vult-server/install/vult-server ./
USER 1000

EXPOSE 8000
ENTRYPOINT [ "./vult-server" ]
CMD [ "run" ]