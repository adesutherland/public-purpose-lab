FROM node:24.19.0-alpine AS build

WORKDIR /workspace
RUN npm install --global pnpm@11.19.0
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml .npmrc ./
COPY frontend ./frontend
RUN pnpm install --frozen-lockfile \
    && pnpm build:web

FROM busybox:1.37.0-musl

COPY deploy/web/index.html /www/index.html
COPY --from=build /workspace/frontend/apps/workbench/dist /www/workbench
COPY --from=build /workspace/frontend/apps/director/dist /www/director
COPY --from=build /workspace/frontend/apps/presentation/dist /www/presentation
USER 65532:65532
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --retries=3 \
  CMD ["wget", "-q", "-O", "/dev/null", "http://127.0.0.1:8080/"]

ENTRYPOINT ["httpd", "-f", "-p", "8080", "-h", "/www"]
