-- @up
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


-- @down
DROP TABLE IF EXISTS sensor_data;

