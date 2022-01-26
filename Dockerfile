FROM rust as build

RUN USER=root cargo new --bin app
WORKDIR /app

COPY Cargo.lock Cargo.toml ./

RUN cargo build --release
RUN rm src/*.rs

COPY . .

RUN cargo build --release

FROM ubuntu

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates
WORKDIR /app

COPY --from=build /app/target/release/react-bot .
RUN chmod +x ./react-bot

CMD ./react-bot
