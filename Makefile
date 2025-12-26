.PHONY: help setup dev test build deploy clean

# Colors
GREEN = \033[0;32m
YELLOW = \033[1;33m
NC = \033[0m

help:
	@echo "$(GREEN)FlashBill Makefile$(NC)"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Development:"
	@echo "  setup          - Setup development environment"
	@echo "  dev            - Run development servers (backend + frontend)"
	@echo "  dev-backend    - Run backend only"
	@echo "  dev-frontend   - Run frontend only"
	@echo ""
	@echo "Testing:"
	@echo "  test           - Run all tests"
	@echo "  test-backend   - Run backend tests"
	@echo "  test-frontend  - Run frontend tests"
	@echo ""
	@echo "Building:"
	@echo "  build          - Build all"
	@echo "  build-backend  - Build backend"
	@echo "  build-frontend - Build frontend"
	@echo ""
	@echo "Docker:"
	@echo "  docker-up      - Start all services with Docker"
	@echo "  docker-down    - Stop all services"
	@echo "  docker-logs    - View logs"
	@echo ""
	@echo "Database:"
	@echo "  db-migrate     - Run database migrations"
	@echo "  db-reset       - Reset database"
	@echo ""
	@echo "Clean:"
	@echo "  clean          - Clean build artifacts"

# Setup
setup:
	@echo "$(YELLOW)Setting up development environment...$(NC)"
	@echo "Backend setup..."
	@cd backend && cargo build
	@echo "Frontend setup..."
	@cd frontend && flutter pub get
	@echo "$(GREEN)Setup complete!$(NC)"

# Development
dev: dev-backend dev-frontend

dev-backend:
	@echo "$(YELLOW)Starting backend...$(NC)"
	@cd backend && cargo run

dev-frontend:
	@echo "$(YELLOW)Starting frontend...$(NC)"
	@cd frontend && flutter run

# Testing
test: test-backend test-frontend

test-backend:
	@echo "$(YELLOW)Running backend tests...$(NC)"
	@cd backend && cargo test

test-frontend:
	@echo "$(YELLOW)Running frontend tests...$(NC)"
	@cd frontend && flutter test

# Building
build: build-backend build-frontend

build-backend:
	@echo "$(YELLOW)Building backend...$(NC)"
	@cd backend && cargo build --release

build-frontend:
	@echo "$(YELLOW)Building frontend...$(NC)"
	@cd frontend && flutter build apk --release

# Docker
docker-up:
	@echo "$(YELLOW)Starting Docker services...$(NC)"
	docker-compose up -d

docker-down:
	@echo "$(YELLOW)Stopping Docker services...$(NC)"
	docker-compose down

docker-logs:
	docker-compose logs -f

# Database
db-migrate:
	@echo "$(YELLOW)Running migrations...$(NC)"
	@cd backend && sqlx migrate run

db-reset:
	@echo "$(YELLOW)Resetting database...$(NC)"
	@cd backend && sqlx database drop && sqlx database create && sqlx migrate run

# Clean
clean:
	@echo "$(YELLOW)Cleaning...$(NC)"
	@cd backend && cargo clean
	@cd frontend && flutter clean
	@echo "$(GREEN)Clean complete!$(NC)"
