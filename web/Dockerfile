# ---------- deps + build ----------
FROM node:22-slim AS builder

WORKDIR /app

COPY package.json package-lock.json ./
# npm ci refuses this lockfile over an optional, platform-specific entry
# (an unmet @img/sharp-wasm32 optional dep from a Windows-generated lock) —
# npm install reconciles it instead of hard-failing.
RUN npm install

COPY . .

# Baked in at build time — this is a static site value, not a secret, and
# Next.js inlines NEXT_PUBLIC_* vars into the client bundle at build time.
ARG NEXT_PUBLIC_API_URL=http://localhost:8080
ENV NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL}

RUN npm run build

# ---------- runtime ----------
FROM node:22-slim AS runtime

WORKDIR /app
ENV NODE_ENV=production

RUN useradd --system --create-home --shell /usr/sbin/nologin fplx

COPY --from=builder /app/public ./public
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static

USER fplx
EXPOSE 3000

CMD ["node", "server.js"]
