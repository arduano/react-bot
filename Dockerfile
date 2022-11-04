FROM rust as build

RUN USER=root cargo new --bin app
WORKDIR /app

COPY Cargo.lock Cargo.toml ./

RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm src/*.rs

COPY . .


RUN rm ./target/release/deps/react_bot*
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates
WORKDIR /app

COPY --from=build /app/target/release/react-bot .
RUN chmod +x ./react-bot

CMD ./react-bot
