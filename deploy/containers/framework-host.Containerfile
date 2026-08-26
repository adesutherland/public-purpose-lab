FROM rust:1.98.0-bookworm AS build

WORKDIR /workspace
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY backend ./backend
RUN cargo build --release --locked --bin ppl-framework-host

FROM debian:bookworm-slim

RUN groupadd --system --gid 65532 ppl \
    && useradd --system --uid 65532 --gid 65532 --no-create-home ppl \
    && mkdir --parents /var/lib/public-purpose-lab \
    && chown 65532:65532 /var/lib/public-purpose-lab
COPY --from=build /workspace/target/release/ppl-framework-host /usr/local/bin/ppl-framework-host
ENV PPL_STATE_DIR=/var/lib/public-purpose-lab \
    PPL_INTERACTION_STATE_DIR=/var/lib/public-purpose-lab/int-01 \
    PPL_ENVIRONMENT_ID=env-container-assurance
USER 65532:65532

HEALTHCHECK --interval=30s --timeout=3s --retries=3 \
  CMD ["ppl-framework-host", "healthcheck"]

ENTRYPOINT ["ppl-framework-host"]
CMD ["serve"]
