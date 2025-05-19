FROM rust:1.85.0-alpine3.21 as builder

RUN apk add musl-dev libressl-dev

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release
RUN cd jwt_auth ; cargo build --release

FROM alpine:3.21

COPY --from=builder /usr/src/app/target/release/public-transport-expense-tracker /usr/local/bin/public-transport-expense-tracker
COPY --from=builder /usr/src/app/jwt_auth/target/release/token /usr/local/bin/token
CMD ["public-transport-expense-tracker"]
