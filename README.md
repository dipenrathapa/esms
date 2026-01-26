# 🌡️ Environmental Stress Monitoring System (ESMS)

A cloud-deployable, real-time sensor analytics system demonstrating **Innovation and Complexity Management (INCO)** principles through automated deployment, live data visualization, and end-to-end system integration.

---


## 🎯 Project Overview

ESMS is a full-stack IoT application that:
- **Ingests** real-time environmental sensor data from Arduino Uno
- **Processes** data to calculate stress indices using weighted algorithms
- **Stores** data in Redis (real-time) and MySQL (historical)
- **Visualizes** live sensor changes with immediate frontend reactions
- **Deploys** automatically in GitHub Codespaces with zero configuration

---

## 🏗️ System Architecture

```
┌─────────────┐      ┌──────────────┐      ┌─────────────┐
│  Arduino    │─────▶│  Rust        │─────▶│  Redis      │
│  Sensors    │ USB  │  Backend     │      │  (Real-time)│
└─────────────┘      │              │      └─────────────┘
                     │  - Serial I/O│
                     │  - JSON Parse│      ┌─────────────┐
                     │  - Stress Calc│─────▶│  MySQL      │
                     │  - REST API  │      │  (History)  │
                     └──────────────┘      └─────────────┘
                            │
                            │ HTTP/JSON
                            ▼
                     ┌──────────────┐
                     │  D3.js       │
                     │  Frontend    │
                     │              │
                     │  - Live Charts│
                     │  - Stress UI │
                     │  - Correlation│
                     └──────────────┘
```

---

## 📊 Stress Index Calculation

The system calculates environmental stress using a weighted formula:

```
Stress Index = (normalized_heart_rate × 0.5)
             + (temperature / 50 × 0.2)
             + (humidity / 100 × 0.2)
             + (noise / 100 × 0.1)
```

**Stress Levels:**
- **Low** (< 0.3): Green indicator
- **Moderate** (0.3 - 0.6): Yellow indicator
- **High** (> 0.6): Red indicator

The frontend **immediately reacts** to stress changes through:
- ✅ Color transitions on stress panel
- ✅ Live graph updates (1-second polling)
- ✅ Motion-based shading on time series
- ✅ Highlighted high-stress points in correlation plots

---

## 🚀 Quick Start

### **Option 1: Local Deployment (with Arduino)**

#### Prerequisites
- Docker & Docker Compose
- Arduino Uno with sensors connected to `/dev/cu.usbmodem113401`
- Rust 1.75+ (optional, for development)



#### Arduino Data Format
Your Arduino must send JSON over serial at 9600 baud:
```json
{
  "temperature": 30.5,
  "humidity": 65,
  "noise": 70,
  "heart_rate": 85,
  "motion": true,
  "timestamp": "2026-01-20T10:00:00Z"
}
```

#### Run the System
```bash
# Clone repository
git clone <your-repo-url>
cd esms

# Start all services
docker-compose up --build

# Access the dashboard
open http://localhost:3000
```
## Configuration
Copy `.env.example` to `.env` and provide your own credentials for local development.

**Services:**
- Frontend: `http://localhost:3000`
- Backend API: `http://localhost:8080`
- Redis: `localhost:6379`
- MySQL: `localhost:3306`

---

### **Option 2: Cloud Deployment (GitHub Codespaces)**

#### Why Codespaces?
✅ No Arduino required - uses **simulated sensor data**  
✅ Zero configuration - works out of the box  
✅ Same codebase for local and cloud  

#### Steps
1. **Open in Codespaces**
   - Click "Code" → "Create codespace on main"
   
2. **Start Services**
   ```bash
   docker-compose up --build
   ```

3. **Access Dashboard**
   - Click "Ports" tab
   - Open port 3000 (Frontend)
   - Backend runs on port 8080

#### Simulation Mode
When serial port is unavailable, the backend automatically generates realistic sensor data every second:
- Temperature: 20-35°C
- Humidity: 40-80%
- Noise: 50-90 dB
- Heart Rate: 60-100 bpm
- Motion: Random (30% probability)

---

## 🔌 API Endpoints

### **GET /health**
Health check endpoint
```json
{
  "status": "healthy",
  "timestamp": "2026-01-24T10:00:00Z"
}
```
## Local Setup
1. Copy `.env.example` to `.env`
2. Add your credentials in `.env`
3. Run `docker compose up --build`

### **GET /api/realtime**
Returns last 60 seconds of data from Redis
```json
[
  {
    "data": {
      "temperature": 28.5,
      "humidity": 62,
      "noise": 65,
      "heart_rate": 78,
      "motion": false,
      "timestamp": "2026-01-24T10:00:00Z"
    },
    "stress_index": 0.42,
    "stress_level": "Moderate"
  }
]
```

### **GET /api/history?start=&end=**
Returns historical data from MySQL
```bash
curl "http://localhost:8080/api/history?start=2026-01-24T09:00:00Z&end=2026-01-24T10:00:00Z"
```

### **GET /api/fhir/observation**
Returns latest data in FHIR-compatible format
```json
{
  "resourceType": "Observation",
  "status": "final",
  "code": {
    "coding": [{
      "system": "http://loinc.org",
      "code": "85354-9",
      "display": "Stress Index"
    }]
  },
  "valueQuantity": {
    "value": 0.42,
    "unit": "index"
  },
  "component": [...]
}
```

---

## 📈 Frontend Dashboard Components

### 1. **Stress Index Monitor**
- Real-time stress value with color coding
- Live statistics for all sensor readings
- Smooth transitions on value changes

### 2. **Environmental Time Series**
- Multi-line chart (Temperature, Humidity, Noise)
- Motion periods shown as orange shaded regions
- Time filters: 1 min, 5 min, 15 min
- Interactive tooltips with exact values

### 3. **Correlation Analysis**
- Scatter plot: Heart Rate vs Environmental Factors
- Color-coded by sensor type
- Highlights high-stress periods (motion = false)
- Larger dots for stress > 0.6

### 4. **Interactivity**
- Dynamic axis scaling
- Hover tooltips
- One-second update rate
- Responsive design

---

## 🔄 CI/CD Pipeline

The GitHub Actions workflow (`.github/workflows/ci-cd.yml`) ensures:

### **1. Code Quality**
- ✅ Rust: cargo check, clippy, fmt
- ✅ Frontend: HTML validation
- ✅ Docker build verification

### **2. Integration Testing**
- ✅ Backend health endpoint
- ✅ Real-time API returns valid JSON
- ✅ FHIR observation structure
- ✅ Frontend accessibility

### **3. Cloud Compatibility**
- ✅ Codespaces devcontainer validation
- ✅ Simulated sensor mode verification
- ✅ docker-compose config check

### **4. Security**
- ✅ Trivy vulnerability scanning
- ✅ SARIF upload to GitHub Security

**Triggers:**
- Push to `main` or `develop`
- Pull requests to `main`

---

## 🗄️ Data Storage

### **Redis (Real-time)**
- Stores last **10 minutes** (600 data points)
- In-memory for fast access
- Used by `/api/realtime` endpoint
- Thread-safe with Tokio Mutex

### **MySQL (Historical)**
- Stores **all historical data**
- Schema:
  ```sql
  sensor_data (
    id, temperature, humidity, noise,
    heart_rate, motion, stress_index,
    stress_level, timestamp, created_at
  )
  ```
- Indexed on `timestamp` and `stress_level`
- Used by `/api/history` endpoint

---

## 🛠️ Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Backend** | Rust + Actix-web | High-performance async I/O |
| **Frontend** | HTML + D3.js | Data visualization |
| **Real-time DB** | Redis | Last 10 minutes cache |
| **Historical DB** | MySQL 8.0 | Persistent storage |
| **Containerization** | Docker + Compose | Deployment automation |
| **CI/CD** | GitHub Actions | Build, test, deploy |
| **Cloud** | GitHub Codespaces | Zero-config environment |

---

## 📂 Project Structure

```
esms/
├── backend/
│   ├── src/
│   │   └── main.rs           # Rust backend
│   ├── Cargo.toml            # Dependencies
│   └── Dockerfile            # Backend container
├── frontend/
│   ├── index.html            # D3.js dashboard
│   ├── nginx.conf            # Web server config
│   └── Dockerfile            # Frontend container
├── .github/
│   └── workflows/
│       └── ci-cd.yml         # CI/CD pipeline
├── .devcontainer/
│   └── devcontainer.json     # Codespaces config
├── docker-compose.yml        # Multi-container orchestration
├── init.sql                  # MySQL schema
└── README.md                 # This file
```

---

## 🎓 INCO Evaluation Criteria

### **1️⃣ Automatic Cloud Deployment**
✅ **One-command startup:** `docker-compose up`  
✅ **GitHub Codespaces ready:** Zero manual configuration  
✅ **CI/CD verification:** Automated on every commit  

### **2️⃣ Direct Frontend Effect from Sensor Data**
✅ **Visible real-time updates:** 1-second polling interval  
✅ **Color changes:** Stress indicator transitions (green/yellow/red)  
✅ **Live graphs:** D3.js redraws on every data point  
✅ **Motion shading:** Orange regions for motion periods  

### **3️⃣ End-to-End Complexity Management**
✅ **Hardware integration:** Arduino serial communication  
✅ **Data processing:** JSON parsing + stress calculation  
✅ **Storage layer:** Redis (cache) + MySQL (persistence)  
✅ **API layer:** RESTful endpoints with FHIR compatibility  
✅ **Visualization:** Multi-chart dashboard with correlation analysis  
✅ **Deployment:** Docker orchestration + GitHub Actions  

---

## 🔧 Development

### **Backend Development**
```bash
cd backend
cargo run
```

### **Frontend Development**
```bash
cd frontend
python3 -m http.server 8000
# Open http://localhost:8000
```

### **View Logs**
```bash
docker-compose logs -f backend
docker-compose logs -f frontend
```

### **Reset Databases**
```bash
docker-compose down -v
docker-compose up --build
```

---

## 🧪 Testing

### **Manual API Testing**
```bash
# Health check
curl http://localhost:8080/health

# Real-time data
curl http://localhost:8080/api/realtime | jq

# Historical data
curl "http://localhost:8080/api/history?start=2026-01-24T00:00:00Z&end=2026-01-24T23:59:59Z" | jq
```

### **Automated CI/CD**
Push to GitHub and check Actions tab for:
- Build status
- Test results
- Security scan reports

---

## 🐛 Troubleshooting

### **Serial Port Not Found**
```bash
# macOS - Find your Arduino port
ls /dev/cu.*

# Update docker-compose.yml with correct port
SERIAL_PORT=/dev/cu.usbmodem113401
```

### **Frontend Can't Connect to Backend**
- Check backend is running: `curl http://localhost:8080/health`
- Verify ports in `docker-compose.yml`
- Check browser console for CORS errors

### **Simulated Data in Local Mode**
If Arduino is connected but simulation runs:
- Verify serial port name
- Check Arduino is sending valid JSON
- View backend logs: `docker-compose logs backend`

---

## 📝 License

This project is created for educational purposes as part of the Innovation and Complexity Management (INCO) course.

---

## 👥 Contributors

- Your Name - Backend & Integration
- Your Name - Frontend & Visualization
- Your Name - DevOps & CI/CD

---

## 🙏 Acknowledgments

- **INCO Course Team** for project requirements
- **Anthropic Claude** for system architecture guidance
- **Rust Community** for async I/O libraries
- **D3.js** for powerful visualization primitives

---

**🚀 Ready to deploy? Run `docker-compose up` and access http://localhost:3000**