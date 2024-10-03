FROM rust:1 AS builder

ARG PROFILE="release"

WORKDIR /build

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --profile ${PROFILE}

RUN --mount=type=cache,target=/build/target \
    case ${PROFILE} in \
    dev) PROFILE_PATH="debug" ;; \
    release) PROFILE_PATH="release" ;; \
    esac \
    && mkdir -p /build/bin \
    && cp /build/target/${PROFILE_PATH}/downloader /build/bin/downloader

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /build/bin/downloader /downloader

ENTRYPOINT [ "/downloader" ]
CMD [ "--config-path", "/etc/downloader/config.toml" ]
