FROM node:22-slim AS build

WORKDIR /app

# SDK must be available as a sibling directory at build time.
# See docker-compose.yml: context is the repo root.
COPY sdk/ /sdk/
COPY module-fpnotes/package.json module-fpnotes/vite.config.ts module-fpnotes/tsconfig.json ./
COPY module-fpnotes/src/ ./src/

RUN npm install && npm run build

FROM nginx:alpine
COPY --from=build /app/dist/ /usr/share/nginx/html/
COPY module-fpnotes/nginx.conf /etc/nginx/conf.d/default.conf
