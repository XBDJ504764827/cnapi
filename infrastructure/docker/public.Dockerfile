# syntax=docker/dockerfile:1

# ---------------------------------------------------------------------------
# Build stage
# ---------------------------------------------------------------------------
FROM node:24-alpine AS builder

ENV COREPACK_ENABLE_DOWNLOAD_PROMPT=0

RUN corepack enable

WORKDIR /src

# Used for canonical URLs, Open Graph tags and sitemap generation.
ARG VITE_SITE_URL=http://localhost:5173
ENV VITE_SITE_URL=$VITE_SITE_URL

# Workspace manifests first so dependency installation can be cached.
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY apps/web/package.json ./apps/web/package.json
COPY apps/public/package.json ./apps/public/package.json
COPY packages/api-client/package.json ./packages/api-client/package.json
COPY packages/shared-types/package.json ./packages/shared-types/package.json
COPY packages/ui/package.json ./packages/ui/package.json

RUN pnpm install --frozen-lockfile

COPY apps/public ./apps/public
COPY packages ./packages

# The API is reached through the static server proxy (same origin), so no build
# time API host is required.
RUN pnpm --filter @cngokz/public build

# ---------------------------------------------------------------------------
# Runtime stage
# ---------------------------------------------------------------------------
FROM node:24-alpine AS runtime

ENV NODE_ENV=production \
    PUBLIC_PORT=5173 \
    API_UPSTREAM=http://api:8080 \
    ADMIN_UPSTREAM=http://web:5173

WORKDIR /app

COPY --from=builder /src/apps/public/dist ./dist
COPY apps/public/server/static-server.mjs ./server/static-server.mjs

USER node
EXPOSE 5173

CMD ["node", "server/static-server.mjs"]
