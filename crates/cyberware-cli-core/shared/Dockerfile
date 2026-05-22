ARG BUILDER_IMG_HOST="registry-1.docker.io/library/rust:1-bookworm"
ARG RUNNER_IMG_HOST="registry-1.docker.io/library/debian:bookworm-slim"

FROM ${BUILDER_IMG_HOST} AS builder

ARG BUILD_MODE="release"
ARG BUILDER_MANIFEST
ARG BUILDER_FLAGS=""
ARG ARTIFACT_NAME

RUN apt update && \
    apt install -y --no-install-recommends protobuf-compiler libprotobuf-dev && \
    rm -rf /var/lib/apt/lists/*

ENV PROTOC_INCLUDE=/usr/include

WORKDIR /app

COPY . .
RUN set -eu; \
    case $BUILD_MODE in \
      debug) BUILD_FLAG="" ;; \
      release) BUILD_FLAG="--release" ;; \
      *) echo "Invalid BUILD_MODE: $BUILD_MODE. Expected: debug or release" >&2; exit 1 ;; \
    esac; \
    cargo build --manifest-path=$BUILDER_MANIFEST --target-dir /app/target $BUILD_FLAG $BUILDER_FLAGS && \
    mv /app/target/$BUILD_MODE/$ARTIFACT_NAME /app/cyberfabric


FROM ${RUNNER_IMG_HOST} AS runner

ARG CONFIG_EXT="yml"
ARG LOCAL_CONFIG_PATH

WORKDIR /app

COPY --from=builder /app/cyberfabric /app/cyberfabric
COPY $LOCAL_CONFIG_PATH /app/config/config.$CONFIG_EXT

RUN apt update && \
    apt install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    useradd --system --uid 10001 --shell /usr/sbin/nologin -d /app app && \
    chown -R app:app /app

ENV RUST_LOG=info
ENV CF_CLI_CONFIG="/app/config/config.$CONFIG_EXT"

USER app

ENTRYPOINT ["/app/cyberfabric"]
