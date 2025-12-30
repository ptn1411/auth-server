-- Migration: Security Enhancements
-- Features: Rate Limiting, Account Lockout, Token Revocation, Audit Logging, 2FA/MFA, Session Management

-- 1. Account Lockout - Add fields to users table
ALTER TABLE users ADD COLUMN failed_login_attempts INT NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN locked_until TIMESTAMP NULL;
ALTER TABLE users ADD COLUMN last_failed_login TIMESTAMP NULL;

-- 2. Audit Logs table - Comprehensive logging for security events
CREATE TABLE audit_logs (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NULL,
    action VARCHAR(50) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id CHAR(36) NULL,
    ip_address VARCHAR(45) NULL,
    user_agent TEXT NULL,
    details JSON NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'success',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    INDEX idx_audit_logs_user_id (user_id),
    INDEX idx_audit_logs_action (action),
    INDEX idx_audit_logs_created_at (created_at),
    INDEX idx_audit_logs_resource (resource_type, resource_id)
);

-- 3. User Sessions table - Track active sessions
CREATE TABLE user_sessions (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL,
    refresh_token_hash VARCHAR(255) NOT NULL,
    device_name VARCHAR(255) NULL,
    device_type VARCHAR(50) NULL,
    ip_address VARCHAR(45) NULL,
    user_agent TEXT NULL,
    last_active_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,
    is_revoked BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_user_sessions_user_id (user_id),
    INDEX idx_user_sessions_token_hash (refresh_token_hash),
    INDEX idx_user_sessions_expires_at (expires_at)
);

-- 4. Revoked Tokens table - Token blacklist for logout
CREATE TABLE revoked_tokens (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    token_type VARCHAR(20) NOT NULL,
    user_id CHAR(36) NULL,
    expires_at TIMESTAMP NOT NULL,
    revoked_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    reason VARCHAR(100) NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    INDEX idx_revoked_tokens_hash (token_hash),
    INDEX idx_revoked_tokens_expires_at (expires_at)
);

-- 5. Rate Limiting table - Track request rates per IP/user
CREATE TABLE rate_limit_entries (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    identifier VARCHAR(255) NOT NULL,
    endpoint VARCHAR(100) NOT NULL,
    request_count INT NOT NULL DEFAULT 1,
    window_start TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY unique_rate_limit (identifier, endpoint),
    INDEX idx_rate_limit_window (window_start)
);

-- 6. MFA Methods table - Store user's MFA configurations
CREATE TABLE user_mfa_methods (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL,
    method_type VARCHAR(20) NOT NULL,
    secret_encrypted VARCHAR(500) NULL,
    phone_number VARCHAR(20) NULL,
    email VARCHAR(255) NULL,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    last_used_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_user_mfa_user_id (user_id),
    CONSTRAINT chk_mfa_method_type CHECK (method_type IN ('totp', 'sms', 'email'))
);

-- 7. MFA Backup Codes table
CREATE TABLE user_mfa_backup_codes (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL,
    code_hash VARCHAR(255) NOT NULL,
    is_used BOOLEAN NOT NULL DEFAULT FALSE,
    used_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_mfa_backup_user_id (user_id)
);

-- 8. Add MFA enabled flag to users
ALTER TABLE users ADD COLUMN mfa_enabled BOOLEAN NOT NULL DEFAULT FALSE;

-- 9. MFA Verification Attempts (for rate limiting MFA)
CREATE TABLE mfa_verification_attempts (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL,
    attempt_type VARCHAR(20) NOT NULL,
    is_successful BOOLEAN NOT NULL DEFAULT FALSE,
    ip_address VARCHAR(45) NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_mfa_attempts_user_id (user_id),
    INDEX idx_mfa_attempts_created_at (created_at)
);
