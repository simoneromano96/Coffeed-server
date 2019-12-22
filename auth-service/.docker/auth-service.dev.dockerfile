FROM rust:slim

WORKDIR /auth-service

RUN cargo install cargo-watch

# Dependency for argon
RUN apt-get update && apt-get install clang llvm-dev libclang-dev -y

# Dependency for MySQL TLS
RUN apt-get update && apt-get install pkg-config libssl-dev -y

# COPY . .
# RUN cargo install --path .

EXPOSE 80

RUN export PATH=/bin/cargo/bin/:$PATH

CMD ["cargo-watch", "-x 'run'"]
