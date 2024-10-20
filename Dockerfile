# This dockerfile is the embed version of the downloader
# To host the frontend sparately, use another docker image

FROM node:20 AS node_builder

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable

WORKDIR /build

COPY ./frontend /build

RUN --mount=type=cache,target=/pnpm/store pnpm \
    pnpm install --frozen-lockfile
RUN pnpm build

FROM rust:1 AS builder

ARG PROFILE="release"

WORKDIR /build

COPY . .

COPY --from=node_builder /build/build /build/frontend/build

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --profile ${PROFILE} --features embed

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
