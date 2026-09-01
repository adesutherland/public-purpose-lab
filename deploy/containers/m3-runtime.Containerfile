FROM node:24.11.1-bookworm-slim AS web-build

WORKDIR /workspace
RUN corepack enable
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY frontend ./frontend
COPY contracts ./contracts
COPY docs ./docs
COPY scenarios ./scenarios
COPY tools ./tools
RUN pnpm install --frozen-lockfile \
    && pnpm check:contracts \
    && pnpm build:web

FROM rust:1.98.0-bookworm AS rust-build

WORKDIR /workspace
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY backend ./backend
COPY contracts ./contracts
COPY scenarios ./scenarios
RUN cargo build --release --locked --bin ppl-m3-runtime --bin ppl-component-host

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 65532 ppl \
    && useradd --system --uid 65532 --gid 65532 --no-create-home ppl \
    && mkdir --parents /opt/public-purpose-lab/frontend /opt/public-purpose-lab/contracts \
       /opt/public-purpose-lab/scenarios /var/lib/public-purpose-lab \
    && chown 65532:65532 /var/lib/public-purpose-lab
COPY --from=rust-build /workspace/target/release/ppl-m3-runtime /usr/local/bin/ppl-m3-runtime
COPY --from=rust-build /workspace/target/release/ppl-component-host /usr/local/bin/ppl-component-host
COPY --from=rust-build /workspace/contracts /opt/public-purpose-lab/contracts
COPY --from=rust-build /workspace/scenarios /opt/public-purpose-lab/scenarios
COPY --from=web-build /workspace/frontend/apps/director/dist /opt/public-purpose-lab/frontend/director
COPY --from=web-build /workspace/frontend/apps/presentation/dist /opt/public-purpose-lab/frontend/presentation
COPY --from=web-build /workspace/frontend/apps/workbench/dist /opt/public-purpose-lab/frontend/workbench
COPY --from=web-build /workspace/frontend/apps/operations/dist /opt/public-purpose-lab/frontend/operations

ENV PPL_PACKAGE_DIRECTORY=/opt/public-purpose-lab/scenarios/presentation-control-assurance \
    PPL_SOURCE_REVISION=container-build \
    PPL_IMAGE_DIGEST=unresolved-build-digest
USER 65532:65532
ENTRYPOINT ["ppl-m3-runtime"]
