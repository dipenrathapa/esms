// #include <DHT.h>
// #include <WiFi.h>
// #include <time.h>

// // ---- Pins ----
// #define DHTPIN     2
// #define DHTTYPE    DHT11
// #define PIR_PIN    3
// #define SOUND_PIN  A0
// #define PULSE_PIN  A1   // pulse sensor signal

// DHT dht(DHTPIN, DHTTYPE);

// // ---- Pulse settings ----
// const unsigned long SAMPLE_INTERVAL_MS = 2;      // ~500 Hz sampling
// const unsigned long REPORT_INTERVAL_MS = 1000;   // 1 sec like code1

// int baseline = 512;
// int thresholdOffset = 40;
// bool wasAbove = false;

// unsigned long lastSampleMs = 0;
// unsigned long lastBeatMs   = 0;
// unsigned long lastReportMs = 0;

// int bpm = 0;

// // For timestamp simulation
// int hour = 10;

// // ---- Noise conversion: 0–1023 -> 0–120 (integer) ----
// int mapNoiseToBackend(int raw) {
//   float mapped = (raw / 1023.0) * 120.0;
//   if (mapped < 0) mapped = 0;
//   if (mapped > 120) mapped = 120;
//   return round(mapped); // always int like code1
// }

// void setup() {
//   Serial.begin(9600);
//   dht.begin();
//   pinMode(PIR_PIN, INPUT);
//   randomSeed(analogRead(0)); // for motion and minute/second randomness
// }

// void updatePulse() {
//   unsigned long now = millis();
//   if (now - lastSampleMs < SAMPLE_INTERVAL_MS) return;
//   lastSampleMs = now;

//   int raw = analogRead(PULSE_PIN);
//   baseline = (baseline * 15 + raw) / 16;
//   int threshold = baseline + thresholdOffset;

//   bool isAbove = raw > threshold;

//   if (isAbove && !wasAbove && (now - lastBeatMs) > 250) {
//     unsigned long ibi = now - lastBeatMs;
//     lastBeatMs = now;

//     if (ibi > 0) {
//       bpm = 60000UL / ibi;
//     }
//   }

//   wasAbove = isAbove;
// }

// void loop() {
//   // ---- Update pulse ----
//   updatePulse();

//   unsigned long now = millis();
//   if (now - lastReportMs < REPORT_INTERVAL_MS) return;
//   lastReportMs = now;

//   // ---- Read sensors ----
//   float temperature = dht.readTemperature();
//   float humidity    = dht.readHumidity();
//   bool motion       = digitalRead(PIR_PIN) == HIGH;

//   int rawNoise = analogRead(SOUND_PIN);
//   int noise  = mapNoiseToBackend(rawNoise);

//   // ---- Fallbacks to keep frontend safe ----
//   if (isnan(temperature)) temperature = 10.0;
//   if (temperature < 0 || temperature > 60) temperature = 10.0;

//   if (isnan(humidity)) humidity = 33;
//   if (humidity < 0 || humidity > 100) humidity = 33;

//   int heartRateOut = (bpm >= 30 && bpm <= 200) ? bpm : 66;

//   // ---- Build JSON ----
//   int minute = random(0, 60);
//   int second = random(0, 60);

//   String json = "{";
//   json += "\"temperature1\":0,"; // fake field for tcp cutoff to resolve
//   json += "\"temperature\":" + String(temperature, 2) + ",";
//   json += "\"humidity\":" + String(humidity) + ",";
//   json += "\"noise\":" + String(noise) + ",";
//   json += "\"heart_rate\":" + String(heartRateOut) + ",";
//   json += "\"motion\":" + String(motion ? "true" : "false") + ",";
//   json += "\"timestamp\":\"2026-01-24 ";

//   if (hour < 10) json += "0";
//   json += hour;
//   json += ":";

//   if (minute < 10) json += "0";
//   json += minute;
//   json += ":";

//   if (second < 10) json += "0";
//   json += second;
//   json += "\"}";

//   // ---- Output ----
//   Serial.println(json);
// }


#include <DHT.h>
#include <time.h>

// ---- Pins ----
#define DHTPIN     2
#define DHTTYPE    DHT11
#define PIR_PIN    3
#define SOUND_PIN  A0
#define PULSE_PIN  A1

DHT dht(DHTPIN, DHTTYPE);

// ---- Pulse settings ----
const unsigned long SAMPLE_INTERVAL_MS = 2;
const unsigned long REPORT_INTERVAL_MS = 1000;

int baseline = 512;
int thresholdOffset = 40;
bool wasAbove = false;

unsigned long lastSampleMs = 0;
unsigned long lastBeatMs = 0;
unsigned long lastReportMs = 0;
unsigned long startTime = 0;  // NEW: for fake time

int bpm = 0;

// ---- Noise conversion ----
int mapNoiseToBackend(int raw) {
  float mapped = (raw / 1023.0) * 120.0;
  if (mapped < 0) mapped = 0;
  if (mapped > 120) mapped = 120;
  return round(mapped);
}

void setup() {
  Serial.begin(9600);
  dht.begin();
  pinMode(PIR_PIN, INPUT);
  startTime = millis();  // NEW: start fake clock
  randomSeed(analogRead(0));
}

void updatePulse() {
  unsigned long now = millis();
  if (now - lastSampleMs < SAMPLE_INTERVAL_MS) return;
  lastSampleMs = now;

  int raw = analogRead(PULSE_PIN);
  baseline = (baseline * 15 + raw) / 16;
  int threshold = baseline + thresholdOffset;

  bool isAbove = raw > threshold;

  if (isAbove && !wasAbove && (now - lastBeatMs) > 250) {
    unsigned long ibi = now - lastBeatMs;
    lastBeatMs = now;
    if (ibi > 0) {
      bpm = 60000UL / ibi;
    }
  }
  wasAbove = isAbove;
}

void loop() {
  updatePulse();

  unsigned long now = millis();
  if (now - lastReportMs < REPORT_INTERVAL_MS) return;
  lastReportMs = now;

  float temperature = dht.readTemperature();
  float humidity = dht.readHumidity();
  bool motion = digitalRead(PIR_PIN) == HIGH;

  int rawNoise = analogRead(SOUND_PIN);
  int noise = mapNoiseToBackend(rawNoise);

  // Fallbacks
  if (isnan(temperature)) temperature = 10.0;
  if (temperature < 0 || temperature > 60) temperature = 10.0;
  if (isnan(humidity)) humidity = 33;
  if (humidity < 0 || humidity > 100) humidity = 33;
  int heartRateOut = (bpm >= 30 && bpm <= 200) ? bpm : 66;

  // ---- DYNAMIC TIME FROM STARTUP (no WiFi) ----
  unsigned long elapsed = (millis() - startTime) / 1000;
  int hours = (elapsed / 3600) % 24;
  int minutes = (elapsed / 60) % 60;
  int seconds = elapsed % 60;

  String json = "{";
  json += "\"temperature1\":0,";
  json += "\"temperature\":" + String(temperature, 2) + ",";
  json += "\"humidity\":" + String(humidity) + ",";
  json += "\"noise\":" + String(noise) + ",";
  json += "\"heart_rate\":" + String(heartRateOut) + ",";
  json += "\"motion\":" + String(motion ? "true" : "false") + ",";
  json += "\"timestamp\":\"2026-01-28 ";  // Date stays static
  
  if (hours < 10) json += "0";
  json += hours;
  json += ":";
  if (minutes < 10) json += "0";
  json += minutes;
  json += ":";
  if (seconds < 10) json += "0";
  json += seconds;
  json += "\"}";

  Serial.println(json);
}


