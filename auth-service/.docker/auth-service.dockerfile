FROM rust:slim

WORKDIR /auth-service

# Dependency for argon
RUN apt-get update && apt-get install clang llvm-dev libclang-dev -y

COPY . .

RUN cargo install --path .

EXPOSE 80

CMD ["auth-service"]
