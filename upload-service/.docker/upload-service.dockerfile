FROM rust:slim

WORKDIR /upload-service

COPY . .

RUN cargo install --path .

EXPOSE 80

CMD ["upload-service"]