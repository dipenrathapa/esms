# ESMS Backend - Structured Logging Documentation

## Overview

The ESMS backend uses **structured logging** with the `tracing` crate to provide rich, queryable logs in JSON format. Each log entry includes contextual fields that make debugging and monitoring in production environments much easier.

## Log Format

All logs are output in **JSON format** with the following structure:

```json
{
  "timestamp": "2026-01-26T10:30:45.123456Z",
  "level": "INFO",
  "operation": "mysql_insert",
  "timestamp": "2026-01-26T10:30:45Z",
  "stress_level": "Moderate",
  "stress_index": "0.52",
  "message": "Successfully stored sensor data in MySQL"
}
```

## Log Levels

- **INFO**: Normal operations, successful actions
- **WARN**: Non-critical issues, fallbacks triggered
- **ERROR**: Errors that need attention but don't crash the app

## Operations and Their Log Fields

### Application Lifecycle

#### **application_startup**
```json
{
  "level": "INFO",
  "operation": "application_startup",
  "use_serial": "true",
  "bind_addr": "0.0.0.0:8080",
  "serial_tcp_host": "host.docker.internal",
  "serial_tcp_port": "5555",
  "message": "Starting ESMS backend"
}
```

#### **redis_initialized**
```json
{
  "level": "INFO",
  "operation": "redis_initialized",
  "message": "Redis client initialized successfully"
}
```

#### **mysql_initialized**
```json
{
  "level": "INFO",
  "operation": "mysql_initialized",
  "message": "MySQL connection pool initialized successfully"
}
```

#### **http_server_started**
```json
{
  "level": "INFO",
  "operation": "http_server_started",
  "bind_addr": "0.0.0.0:8080",
  "message": "HTTP server is running"
}
```

---

### Configuration Validation

#### **config_validation_start**
```json
{
  "level": "INFO",
  "operation": "config_validation_start",
  "message": "Starting configuration validation"
}
```

#### **config_validation** (Per Component)
```json
{
  "level": "INFO",
  "operation": "config_validation",
  "component": "redis_url",
  "message": "Redis URL validated successfully"
}
```

```json
{
  "level": "INFO",
  "operation": "config_validation",
  "component": "bind_addr",
  "bind_addr": "0.0.0.0:8080",
  "message": "Bind address validated successfully"
}
```

#### **config_validation_complete**
```json
{
  "level": "INFO",
  "operation": "config_validation_complete",
  "message": "Configuration validation successful"
}
```

---

### Background Task Lifecycle

#### **sensor_task_start**
```json
{
  "level": "INFO",
  "operation": "sensor_task_start",
  "use_serial": "true",
  "serial_host": "host.docker.internal",
  "serial_port": "5555",
  "message": "Sensor background task started"
}
```


#### **sensor_task_process** (Error)
```json
{
  "level": "ERROR",
  "operation": "sensor_task_process",
  "error": "Validation(\"temperature out of range\")",
  "message": "Error processing sensor data in background task"
}
```

#### **sensor_task_shutdown**
```json
{
  "level": "INFO",
  "operation": "sensor_task_shutdown",
  "message": "Sensor task received shutdown signal, cleaning up..."
}
```

#### **sensor_task_stopped**
```json
{
  "level": "INFO",
  "operation": "sensor_task_stopped",
  "message": "Sensor task stopped gracefully"
}
```

---

### TCP Sensor Operations

#### **tcp_connect** (Success)
```json
{
  "level": "INFO",
  "operation": "tcp_connect",
  "host": "host.docker.internal",
  "port": "5555",
  "message": "Successfully connected to TCP sensor stream"
}
```

#### **tcp_connect** (Error)
```json
{
  "level": "ERROR",
  "operation": "tcp_connect",
  "host": "host.docker.internal",
  "port": "5555",
  "error": "Connection refused (os error 61)",
  "message": "Failed to connect to TCP sensor stream"
}
```

#### **tcp_read** (Success)
```json
{
  "level": "INFO",
  "operation": "tcp_read",
  "host": "host.docker.internal",
  "port": "5555",
  "bytes_read": "142",
  "temperature": "25.3",
  "heart_rate": "72.5",
  "message": "Successfully parsed sensor data from TCP"
}
```

#### **tcp_parse** (Warning)
```json
{
  "level": "WARN",
  "operation": "tcp_parse",
  "host": "host.docker.internal",
  "port": "5555",
  "raw_data": "{invalid json}",
  "message": "Failed to parse JSON from TCP stream"
}
```

---

### Sensor Data Processing

#### **sensor_data_source**
```json
{
  "level": "INFO",
  "operation": "sensor_data_source",
  "source": "tcp",
  "message": "Using real sensor data from TCP stream"
}
```

```json
{
  "level": "WARN",
  "operation": "sensor_data_source",
  "source": "simulation_fallback",
  "message": "TCP read failed, falling back to simulated data"
}
```

```json
{
  "level": "INFO",
  "operation": "sensor_data_source",
  "source": "simulation",
  "message": "Using simulated sensor data"
}
```

#### **sensor_validation** (Warning)
```json
{
  "level": "WARN",
  "operation": "sensor_validation",
  "error": "temperature: must be between 0 and 60",
  "temperature": "65.2",
  "humidity": "45.0",
  "heart_rate": "75.0",
  "message": "Sensor data validation failed"
}
```

#### **memory_store**
```json
{
  "level": "INFO",
  "operation": "memory_store",
  "buffer_size": "123",
  "timestamp": "2026-01-26T10:30:45Z",
  "stress_level": "Moderate",
  "message": "Stored sensor data in memory buffer"
}
```

---

### Redis Operations

#### **redis_connection** (Error)
```json
{
  "level": "ERROR",
  "operation": "redis_connection",
  "timestamp": "2026-01-26T10:30:45Z",
  "error": "Connection refused",
  "message": "Failed to establish Redis connection"
}
```

#### **redis_connection** (Retry Success)
```json
{
  "level": "INFO",
  "operation": "redis_connection",
  "attempts": "3",
  "message": "Operation succeeded after retry"
}
```

#### **redis_serialization** (Error)
```json
{
  "level": "ERROR",
  "operation": "redis_serialization",
  "timestamp": "2026-01-26T10:30:45Z",
  "key": "sensor:2026-01-26T10:30:45Z",
  "error": "Serialization error",
  "message": "Failed to serialize sensor data for Redis"
}
```

#### **redis_set** (Error)
```json
{
  "level": "ERROR",
  "operation": "redis_set",
  "timestamp": "2026-01-26T10:30:45Z",
  "key": "sensor:2026-01-26T10:30:45Z",
  "ttl": "600",
  "error": "Redis error: Connection lost",
  "message": "Failed to set value in Redis"
}
```

#### **redis_set** (Retry Warning)
```json
{
  "level": "WARN",
  "operation": "redis_set",
  "attempt": "2",
  "max_attempts": "5",
  "delay_ms": "200",
  "error": "Timeout",
  "message": "Operation failed, retrying..."
}
```

#### **redis_set** (Success)
```json
{
  "level": "INFO",
  "operation": "redis_set",
  "timestamp": "2026-01-26T10:30:45Z",
  "key": "sensor:2026-01-26T10:30:45Z",
  "stress_level": "Moderate",
  "message": "Successfully stored sensor data in Redis"
}
```

#### **background_redis_store** (Final Failure)
```json
{
  "level": "WARN",
  "operation": "background_redis_store",
  "error": "Redis(\"Connection failed: ...\")",
  "message": "Redis background task failed"
}
```

---

### MySQL Operations

#### **mysql_connection** (Error)
```json
{
  "level": "ERROR",
  "operation": "mysql_connection",
  "timestamp": "2026-01-26T10:30:45Z",
  "error": "Too many connections",
  "message": "Failed to get MySQL connection from pool"
}
```

#### **mysql_insert** (Error)
```json
{
  "level": "ERROR",
  "operation": "mysql_insert",
  "timestamp": "2026-01-26T10:30:45Z",
  "temperature": "25.3",
  "humidity": "55.2",
  "heart_rate": "72.5",
  "stress_level": "Moderate",
  "error": "Duplicate entry",
  "message": "Failed to insert sensor data into MySQL"
}
```

#### **mysql_insert** (Retry)
```json
{
  "level": "WARN",
  "operation": "mysql_insert",
  "attempt": "1",
  "max_attempts": "5",
  "delay_ms": "100",
  "error": "Connection reset",
  "message": "Operation failed, retrying..."
}
```

#### **mysql_insert** (Success)
```json
{
  "level": "INFO",
  "operation": "mysql_insert",
  "timestamp": "2026-01-26T10:30:45Z",
  "stress_level": "Moderate",
  "stress_index": "0.52",
  "message": "Successfully stored sensor data in MySQL"
}
```

#### **background_mysql_store** (Final Failure)
```json
{
  "level": "WARN",
  "operation": "background_mysql_store",
  "error": "Database(\"Connection failed: ...\")",
  "message": "MySQL background task failed"
}
```

---

### Graceful Shutdown

#### **shutdown_signal_received**
```json
{
  "level": "INFO",
  "operation": "shutdown_signal_received",
  "message": "Shutdown signal received, initiating graceful shutdown..."
}
```

#### **http_server_stopped**
```json
{
  "level": "INFO",
  "operation": "http_server_stopped",
  "message": "HTTP server stopped"
}
```

#### **shutdown_signal_handled**
```json
{
  "level": "INFO",
  "operation": "shutdown_signal_handled",
  "message": "Shutdown signal handled"
}
```

#### **background_task_stopped**
```json
{
  "level": "INFO",
  "operation": "background_task_stopped",
  "message": "Background task stopped successfully"
}
```

#### **background_task_error**
```json
{
  "level": "ERROR",
  "operation": "background_task_error",
  "error": "JoinError { ... }",
  "message": "Background task encountered an error during shutdown"
}
```

#### **background_task_timeout**
```json
{
  "level": "ERROR",
  "operation": "background_task_timeout",
  "timeout_seconds": "10",
  "message": "Background task did not stop within timeout"
}
```

#### **application_shutdown_complete**
```json
{
  "level": "INFO",
  "operation": "application_shutdown_complete",
  "message": "Application shutdown complete"
}
```

---

## Querying Logs

### Using jq (command-line JSON processor)

**Find all Redis errors:**
```bash
cat logs.json | jq 'select(.operation | contains("redis")) | select(.level == "ERROR")'
```

**Find all operations with a specific timestamp:**
```bash
cat logs.json | jq 'select(.timestamp == "2026-01-26T10:30:45Z")'
```

**Count errors by operation:**
```bash
cat logs.json | jq -s 'group_by(.operation) | map({operation: .[0].operation, errors: map(select(.level == "ERROR")) | length})'
```

**Find high stress events:**
```bash
cat logs.json | jq 'select(.stress_level == "High")'
```

**Find all retry attempts:**
```bash
cat logs.json | jq 'select(.attempts != null)'
```

**Monitor failed operations:**
```bash
cat logs.json | jq 'select(.message | contains("failed"))'
```

---

### Using Log Aggregation Tools

#### Elasticsearch/Kibana Query
```
operation:"mysql_insert" AND level:"ERROR"
```

```
operation:"redis_set" AND attempts:>1
```

#### Grafana Loki Query
```
{job="esms-backend"} |= "ERROR" | json | operation="redis_set"
```

```
{job="esms-backend"} | json | stress_level="High"
```

#### CloudWatch Insights Query
```sql
fields @timestamp, operation, error, message
| filter level = "ERROR"
| stats count() by operation
```

```sql
fields @timestamp, operation, attempts
| filter attempts > 1
| sort @timestamp desc
```

---

## Environment Variables for Logging

Set the `RUST_LOG` environment variable to control log verbosity:

```bash
# Show all logs (including debug)
RUST_LOG=debug

# Show only warnings and errors
RUST_LOG=warn

# Show info and above for your app, debug for specific modules
RUST_LOG=info,esms_backend::sensor=debug

# Production setting (info level, JSON output)
RUST_LOG=info
```

---

## Best Practices

1. **Always include `operation` field** - Makes filtering and grouping easier
2. **Include timestamps for time-series data** - The `timestamp` field in the data payload
3. **Add error context** - Include relevant IDs, values that caused the error
4. **Use appropriate log levels**:
   - `ERROR` for failures that need attention
   - `WARN` for degraded functionality but still operational
   - `INFO` for important state changes and successful operations
5. **Keep messages human-readable** - The `message` field should explain what happened
6. **Include performance metrics** - Add fields like `buffer_size`, `bytes_read`, `attempts` when relevant

---

## Monitoring Alerts (Suggested)

### Critical Alerts
- **Redis connection failures** > 5 in 5 minutes
- **MySQL connection failures** > 5 in 5 minutes
- **Background task timeout** - Any occurrence
- **Configuration validation failure** - Any occurrence

### Warning Alerts
- **TCP connection failures** > 10 in 5 minutes
- **Validation failures** > 20 in 5 minutes
- **Memory buffer size** > 500 entries
- **Retry attempts** > 3 for any single operation

### Info Monitoring
- **Successful data storage rate** - Track INFO logs per minute
- **Stress level distribution** - Monitor High/Moderate/Low percentages
- **Fallback to simulation** - Track how often TCP fails
- **Average retry attempts** - Monitor system health

---

## Example Queries for Production Monitoring

### Find All Failed Operations in Last Hour
```bash
cat logs.json | jq 'select(.timestamp > "'$(date -u -d '1 hour ago' +%Y-%m-%dT%H:%M:%S)'Z") | select(.level == "ERROR")'
```

### Calculate Retry Success Rate
```bash
cat logs.json | jq -s '
  group_by(.operation) | 
  map({
    operation: .[0].operation,
    total_retries: map(select(.attempts != null)) | length,
    successful_retries: map(select(.attempts != null and .attempts > 1)) | length
  })
'
```

### Monitor Stress Level Distribution
```bash
cat logs.json | jq -s 'group_by(.stress_level) | map({level: .[0].stress_level, count: length})'
```

### Find Operations Taking Multiple Retries
```bash
cat logs.json | jq 'select(.attempts > 2)'
```

---

## Troubleshooting Common Issues

### Issue: No logs appearing
**Solution:** Check `RUST_LOG` environment variable is set
```bash
export RUST_LOG=info
```

### Issue: Logs not in JSON format
**Solution:** Ensure tracing_subscriber is configured for JSON output (already done in code)

### Issue: Too many logs
**Solution:** Increase log level to `warn` or `error`
```bash
export RUST_LOG=warn
```

### Issue: Can't find specific operation
**Solution:** Use grep with the operation name
```bash
docker compose logs backend | grep "redis_set"
```

---

## Viewing Logs in Docker

### View real-time logs
```bash
docker compose logs -f backend
```

### View logs with timestamps
```bash
docker compose logs -t backend
```

### View last 100 lines
```bash
docker compose logs --tail=100 backend
```

### Save logs to file
```bash
docker compose logs backend > backend-logs.json
```

---

## Integration with Monitoring Tools

### Prometheus/Grafana
You can export metrics from these structured logs using log-to-metrics tools like `mtail` or `promtail`.

### Example metric from logs:
- `esms_redis_errors_total` (counter) - from ERROR logs with operation="redis_*"
- `esms_retry_attempts` (histogram) - from logs with attempts field
- `esms_stress_level` (gauge) - from logs with stress_level field

### ELK Stack
Direct integration - ship JSON logs to Logstash/Elasticsearch and visualize in Kibana.

### Datadog
Use Datadog agent to ship logs with automatic parsing of JSON fields.

---

## Conclusion

This structured logging approach provides:
- **Easy debugging** - All context in one log entry
- **Production monitoring** - Query by operation, timestamp, error type
- **Performance tracking** - Monitor retry rates, success rates, buffer sizes
- **Alerting** - Set alerts on specific operations or error patterns
- **Compliance** - Audit trail for all operations with timestamps