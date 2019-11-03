FROM rust:slim

WORKDIR /api-gateway

COPY . .

RUN cargo install --path .

EXPOSE 80

CMD ["api-gateway"]