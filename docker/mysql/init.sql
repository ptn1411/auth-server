-- MySQL initialization script
-- This runs when the container is first created

-- Set character set
SET NAMES utf8mb4;
SET CHARACTER SET utf8mb4;

-- Grant permissions (if using non-root user)
-- The database and user are created by Docker environment variables
-- This script can be used for additional setup

-- Example: Create additional databases for testing
-- CREATE DATABASE IF NOT EXISTS auth_server_test;
-- GRANT ALL PRIVILEGES ON auth_server_test.* TO 'authserver'@'%';

-- Flush privileges
FLUSH PRIVILEGES;
