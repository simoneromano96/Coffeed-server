FROM rust:alpine

RUN mkdir /app

WORKDIR /app

COPY . .

RUN cargo build --release

CMD [ "/app/target/release/auth-service" ]