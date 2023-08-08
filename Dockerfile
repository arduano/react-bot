FROM rust as build

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update
RUN apt install musl-tools -y

RUN USER=root cargo new --bin app
WORKDIR /app

COPY Cargo.lock Cargo.toml ./

RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm src/*.rs

COPY . .

RUN rm ./target/x86_64-unknown-linux-musl/release/deps/react_bot*
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3.18.3

WORKDIR /app

COPY --from=build /app/target/x86_64-unknown-linux-musl/release/react-bot .
RUN chmod +x ./react-bot

CMD ./react-bot
