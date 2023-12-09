
FROM rust AS build
WORKDIR /src

RUN rustup override set nightly

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./framework ./framework
COPY ./service_a ./service_a
COPY ./service_b ./service_b

RUN cargo build --release

FROM rust:slim
ARG service
WORKDIR /src

COPY --from=build /src/target/release/${service} ./service

CMD ["./service"]
