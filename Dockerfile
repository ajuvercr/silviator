FROM rust:1.65.0-slim

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --path .

CMD ["silviator"]
