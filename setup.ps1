# Requires PowerShell 7+
Write-Host "Starting ESMS setup..." -ForegroundColor Cyan


# 0. Create .env if missing

$envFile = ".\backend\.env"

if (-Not (Test-Path $envFile)) {
    Write-Host "Creating backend .env file..."
@"
REDIS_URL=redis://redis:6379
MYSQL_DATABASE_URL=mysql://esms_user:esms_pass@mysql:3306/esms_db
SERIAL_PORT=/dev/cu.usbmodem13401
USE_SERIAL=true
SERIAL_TCP_HOST=host.docker.internal
SERIAL_TCP_PORT=5555
RUST_LOG=info
BIND_ADDR=0.0.0.0:8080
"@ | Out-File -Encoding UTF8 $envFile
    Write-Host ".env file created at $envFile"
} else {
    Write-Host ".env file already exists"
}


# 1. Check Rust

if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "⚡ Rust not found. Installing Rust..."
    Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://sh.rustup.rs')) -ArgumentList "-y"
} else {
    Write-Host "Rust is installed: $(rustc --version)"
}


# 2. Check Docker

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host "Docker not found. Please install Docker Desktop: https://docs.docker.com/get-docker/" -ForegroundColor Red
    exit 1
} else {
    Write-Host "Docker is installed: $(docker --version)"
}

if (-not (Get-Command docker-compose -ErrorAction SilentlyContinue)) {
    Write-Host "docker-compose not found. Installing..."
    Invoke-WebRequest "https://github.com/docker/compose/releases/latest/download/docker-compose-Windows-x86_64.exe" -OutFile "$env:ProgramFiles\docker-compose.exe"
    Write-Host "docker-compose installed"
} else {
    Write-Host "docker-compose is installed: $(docker-compose --version)"
}


# 3. Build backend

Write-Host "Building backend Docker image..."
docker build --target builder -t esms-backend-builder .\backend
docker build -t esms-backend .\backend


# 4. Build frontend

Write-Host "Building frontend Docker image..."
docker build -t esms-frontend .\frontend


# 5. Start docker-compose stack

Write-Host "Starting docker-compose stack..."
docker-compose up -d --build

Write-Host "ESMS stack is up and running!"
Write-Host "Frontend: http://localhost:3000"
Write-Host "Backend API: http://localhost:8080"
Write-Host "MySQL: localhost:3306, user: esms_user, password: esms_pass"
Write-Host "Redis: localhost:6379"
Write-Host "Setup complete!"
