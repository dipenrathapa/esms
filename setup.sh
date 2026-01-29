#!/usr/bin/env bash

set -e

echo "Starting ESMS setup..."


# 0. Create .env if missing


ENV_FILE="./backend/.env"

if [ ! -f "$ENV_FILE" ]; then
    echo "📄 Creating backend .env file..."
    cat <<EOL > "$ENV_FILE"
REDIS_URL=redis://redis:6379
MYSQL_DATABASE_URL=mysql://esms_user:esms_pass@mysql:3306/esms_db
SERIAL_PORT=/dev/cu.usbmodem13401
USE_SERIAL=true
SERIAL_TCP_HOST=host.docker.internal
SERIAL_TCP_PORT=5555
RUST_LOG=info
BIND_ADDR=0.0.0.0:8080
EOL
    echo ".env file created at $ENV_FILE"
else
    echo ".env file already exists"
fi


# 1. Check Rust

if ! command -v rustc &> /dev/null
then
    echo "⚡ Rust not found. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
else
    echo "✅ Rust is installed: $(rustc --version)"
fi


# 2. Check Docker

if ! command -v docker &> /dev/null
then
    echo "Docker not found. Please install Docker first: https://docs.docker.com/get-docker/"
    exit 1
else
    echo "Docker is installed: $(docker --version)"
fi

if ! command -v docker-compose &> /dev/null
then
    echo "❌ docker-compose not found. Installing..."
    sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
    sudo chmod +x /usr/local/bin/docker-compose
    echo "docker-compose installed"
else
    echo "docker-compose is installed: $(docker-compose --version)"
fi


# 3. Build Backend Docker Image

echo "📦 Building backend Docker image..."
docker build --target builder -t esms-backend-builder ./backend
docker build -t esms-backend ./backend


# 4. Build Frontend Docker Image

echo "📦 Building frontend Docker image..."
docker build -t esms-frontend ./frontend


# 5. Start Docker Compose Stack

echo "Starting docker-compose stack..."
docker-compose up -d --build

echo "ESMS stack is up and running!"
echo "Frontend: http://localhost:3000"
echo "Backend API: http://localhost:8080"
echo "MySQL: localhost:3306, user: esms_user, password: esms_pass"
echo "Redis: localhost:6379"

echo "Setup complete!"
