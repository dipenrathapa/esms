#!/bin/bash

# ESMS Setup Script
# Automatically creates project structure and files

set -e

echo "🚀 Setting up Environmental Stress Monitoring System (ESMS)..."

# Create directory structure
echo "📁 Creating directory structure..."
mkdir -p backend/src
mkdir -p frontend
mkdir -p .github/workflows
mkdir -p .devcontainer

# Create backend files
echo "🦀 Setting up Rust backend..."
cat > backend/src/main.rs << 'EOF'
// Paste the main.rs content here
EOF

cat > backend/Cargo.toml << 'EOF'
[package]
name = "esms-backend"
version = "1.0.0"
edition = "2021"

[dependencies]
actix-web = "4.4"
actix-cors = "0.7"
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
tokio-serial = "5.4"
rand = "0.8"
EOF

cat > backend/Dockerfile << 'EOF'
FROM rust:1.75 as builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/esms-backend /app/esms-backend
EXPOSE 8080
ENV RUST_LOG=info
ENV SERIAL_PORT=/dev/cu.usbmodem113401
CMD ["./esms-backend"]
EOF

# Create frontend files
echo "🎨 Setting up D3.js frontend..."
cat > frontend/nginx.conf << 'EOF'
server {
    listen 80;
    server_name localhost;
    root /usr/share/nginx/html;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location /api {
        proxy_pass http://backend:8080;
        proxy_set_header Host $host;
        add_header 'Access-Control-Allow-Origin' '*' always;
    }
}
EOF

cat > frontend/Dockerfile << 'EOF'
FROM nginx:alpine
COPY index.html /usr/share/nginx/html/
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
EOF

# Create docker-compose.yml
echo "🐳 Creating Docker Compose configuration..."
cat > docker-compose.yml << 'EOF'
version: '3.8'

services:
  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
    container_name: esms-backend
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - SERIAL_PORT=/dev/cu.usbmodem113401
    volumes:
      - /dev:/dev
    devices:
      - /dev/cu.usbmodem113401:/dev/cu.usbmodem113401
    privileged: true
    depends_on:
      - redis
      - mysql
    restart: unless-stopped
    networks:
      - esms-network

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    container_name: esms-frontend
    ports:
      - "3000:80"
    depends_on:
      - backend
    restart: unless-stopped
    networks:
      - esms-network

  redis:
    image: redis:7-alpine
    container_name: esms-redis
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    command: redis-server --appendonly yes
    restart: unless-stopped
    networks:
      - esms-network

  mysql:
    image: mysql:8.0
    container_name: esms-mysql
    ports:
      - "3306:3306"
    environment:
      - MYSQL_ROOT_PASSWORD=esms_password
      - MYSQL_DATABASE=esms_db
      - MYSQL_USER=esms_user
      - MYSQL_PASSWORD=esms_pass
    volumes:
      - mysql-data:/var/lib/mysql
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql
    restart: unless-stopped
    networks:
      - esms-network

volumes:
  redis-data:
  mysql-data:

networks:
  esms-network:
    driver: bridge
EOF

# Create init.sql
echo "💾 Creating MySQL initialization script..."
cat > init.sql << 'EOF'
CREATE DATABASE IF NOT EXISTS esms_db;
USE esms_db;

CREATE TABLE IF NOT EXISTS sensor_data (
    id INT AUTO_INCREMENT PRIMARY KEY,
    temperature DECIMAL(5,2),
    humidity DECIMAL(5,2),
    noise DECIMAL(5,2),
    heart_rate DECIMAL(5,2),
    motion BOOLEAN,
    stress_index DECIMAL(5,3),
    stress_level VARCHAR(20),
    timestamp DATETIME,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_timestamp (timestamp),
    INDEX idx_stress_level (stress_level)
);
EOF

# Create .gitignore
echo "📝 Creating .gitignore..."
cat > .gitignore << 'EOF'
# Rust
backend/target/
backend/Cargo.lock
**/*.rs.bk

# Docker
*.log

# Database
*.db
*.sqlite

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Environment
.env
*.env
EOF

# Create Makefile for convenience
echo "⚙️ Creating Makefile..."
cat > Makefile << 'EOF'
.PHONY: build run stop clean logs test

build:
	docker-compose build

run:
	docker-compose up

run-detached:
	docker-compose up -d

stop:
	docker-compose down

clean:
	docker-compose down -v
	rm -rf backend/target

logs:
	docker-compose logs -f

logs-backend:
	docker-compose logs -f backend

logs-frontend:
	docker-compose logs -f frontend

test:
	curl http://localhost:8080/health
	curl http://localhost:8080/api/realtime

restart:
	docker-compose restart

rebuild:
	docker-compose down
	docker-compose build --no-cache
	docker-compose up
EOF

echo ""
echo "✅ Project structure created successfully!"
echo ""
echo "📋 Next steps:"
echo "   1. Copy your backend/src/main.rs content"
echo "   2. Copy your frontend/index.html content"
echo "   3. Update SERIAL_PORT in docker-compose.yml if needed"
echo "   4. Run: docker-compose up --build"
echo ""
echo "🌐 Access points:"
echo "   - Frontend: http://localhost:3000"
echo "   - Backend:  http://localhost:8080"
echo ""
echo "🎓 For INCO evaluation, ensure:"
echo "   ✓ Arduino sends JSON to /dev/cu.usbmodem113401"
echo "   ✓ GitHub Actions CI/CD is configured"
echo "   ✓ README.md explains stress index calculation"
echo ""
