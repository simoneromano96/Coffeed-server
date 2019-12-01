FROM rust:slim

WORKDIR /auth-service

COPY . .

RUN cargo install --path .

EXPOSE 80

CMD ["auth-service"]