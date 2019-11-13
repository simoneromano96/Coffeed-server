FROM rust:slim

WORKDIR /upload-service

RUN cargo install cargo-watch

COPY . .

RUN cargo install --path .

EXPOSE 80

CMD ["cargo-watch", "-x 'run'"]
