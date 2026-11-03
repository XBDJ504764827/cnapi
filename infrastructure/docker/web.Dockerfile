# syntax=docker/dockerfile:1

# ---------------------------------------------------------------------------
# Build stage
# ---------------------------------------------------------------------------
FROM node:24-alpine AS builder

ENV COREPACK_ENABLE_DOWNLOAD_PROMPT=0

RUN corepack enable

WORKDIR /src

# Install workspace dependencies with a frozen lockfile.
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY apps/web/package.json ./apps/web/package.json
COPY packages/api-client/package.json ./packages/api-client/package.json
COPY packages/shared-types/package.json ./packages/shared-types/package.json

RUN pnpm install --frozen-lockfile

COPY apps/web ./apps/web
COPY packages ./packages

# The API is reached through the static server proxy (same origin), so no build
# time API host is required.
RUN pnpm --filter @cngokz/web build

# ---------------------------------------------------------------------------
# Runtime stage
# ---------------------------------------------------------------------------
FROM node:24-alpine AS runtime

ENV NODE_ENV=production \
    WEB_PORT=5173 \
    API_UPSTREAM=http://api:8080

WORKDIR /app

COPY --from=builder /src/apps/web/dist ./dist
COPY apps/web/server/static-server.mjs ./server/static-server.mjs

USER node
EXPOSE 5173

CMD ["node", "server/static-server.mjs"]
