# Build stage
FROM node:20-alpine AS builder

WORKDIR /app

# Copy manifest files
COPY pubspec.yaml ./

# Install Flutter (simplified - in production use pre-built image)
# This is a placeholder for Flutter web build

# Copy source code
COPY lib ./lib
COPY assets ./assets

# Build command (placeholder - Flutter web build)
# RUN flutter pub get
# RUN flutter build web

# Runtime stage (nginx for Flutter web)
FROM nginx:alpine

# Copy nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

# Copy built app (placeholder)
# COPY --from=builder /app/build/web /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
