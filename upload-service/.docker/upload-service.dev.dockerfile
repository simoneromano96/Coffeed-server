FROM rust:slim

WORKDIR /upload-service

RUN cargo install cargo-watch

# COPY . .
# RUN cargo install --path .

EXPOSE 80

RUN export PATH=/bin/cargo/bin/:$PATH

CMD ["cargo-watch", "-x 'run'"]
