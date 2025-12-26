#!/bin/bash

# Start PostgreSQL database for development
echo "🚀 Starting PostgreSQL database..."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Start PostgreSQL container
docker run -d \
    --name flashbill-db \
    -e POSTGRES_DB=flashbill \
    -e POSTGRES_USER=postgres \
    -e POSTGRES_PASSWORD=postgres \
    -p 5432:5432 \
    -v flashbill_pgdata:/var/lib/postgresql/data \
    postgres:15-alpine

echo "⏳ Waiting for PostgreSQL to be ready..."
sleep 5

# Check if database is ready
if docker exec flashbill-db pg_isready -U postgres > /dev/null 2>&1; then
    echo "✅ PostgreSQL is ready!"
    echo ""
    echo "Connection string:"
    echo "postgres://postgres:postgres@localhost:5432/flashbill"
    echo ""
    echo "To run migrations:"
    echo "cd backend && DATABASE_URL=\"postgres://postgres:postgres@localhost:5432/flashbill\" sqlx migrate run"
else
    echo "❌ PostgreSQL failed to start"
    exit 1
fi
