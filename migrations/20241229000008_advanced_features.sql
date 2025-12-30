-- Advanced Features Migration
-- 1. Webhook Events
-- 2. Rate Limiting
-- 3. API Keys
-- 4. Refresh Token Rotation
-- 5. IP Whitelist/Blacklist
-- 6. WebAuthn/Passkeys

-- ============ Webhook Events ============

CREATE TABLE IF NOT EXISTS webhooks (
    id CHAR(36) PRIMARY KEY,
    app_id CHAR(36) NOT NULL,
    url VARCHAR(2048) NOT NULL,
    secret VARCHAR(255) NOT NULL,
    events JSON NOT NULL, -- ["user.login", "user.register", ...]
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (app_id) REFERENCES apps(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS webhook_deliveries (
    id CHAR(36) PRIMARY KEY,
    webhook_id CHAR(36) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    payload JSON NOT NULL,
    response_status INT,
    response_body TEXT,
    attempts INT NOT NULL DEFAULT 0,
    next_retry_at TIMESTAMP NULL,
    delivered_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (webhook_id) REFERENCES webhooks(id) ON DELETE CASCADE
);

CREATE INDEX idx_webhook_deliveries_pending ON webhook_deliveries(next_retry_at) WHERE delivered_at IS NULL;

-- ============ Rate Limiting ============

CREATE TABLE IF NOT EXISTS rate_limits (
    id CHAR(36) PRIMARY KEY,
    identifier VARCHAR(255) NOT NULL, -- IP address or user_id
    endpoint VARCHAR(255) NOT NULL,
    request_count INT NOT NULL DEFAULT 1,
    window_start TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY unique_rate_limit (identifier, endpoint, window_start)
);

CREATE INDEX idx_rate_limits_lookup ON rate_limits(identifier, endpoint, window_start);

-- ============ API Keys ============

CREATE TABLE IF NOT EXISTS api_keys (
    id CHAR(36) PRIMARY KEY,
    app_id CHAR(36) NOT NULL,
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL,
    key_prefix VARCHAR(10) NOT NULL, -- First 8 chars for identification
    scopes JSON NOT NULL, -- ["read:users", "write:users", ...]
    expires_at TIMESTAMP NULL,
    last_used_at TIMESTAMP NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (app_id) REFERENCES apps(id) ON DELETE CASCADE
);

CREATE INDEX idx_api_keys_prefix ON api_keys(key_prefix);
CREATE INDEX idx_api_keys_app ON api_keys(app_id);

-- ============ Refresh Token Rotation ============

ALTER TABLE refresh_tokens 
ADD COLUMN IF NOT EXISTS family_id CHAR(36) NULL,
ADD COLUMN IF NOT EXISTS rotated_at TIMESTAMP NULL,
ADD COLUMN IF NOT EXISTS replaced_by CHAR(36) NULL;

CREATE INDEX idx_refresh_tokens_family ON refresh_tokens(family_id);

-- ============ IP Whitelist/Blacklist ============

CREATE TABLE IF NOT EXISTS ip_rules (
    id CHAR(36) PRIMARY KEY,
    app_id CHAR(36) NULL, -- NULL = global rule
    ip_address VARCHAR(45) NOT NULL, -- Supports IPv6
    ip_range VARCHAR(50) NULL, -- CIDR notation e.g., "192.168.1.0/24"
    rule_type ENUM('whitelist', 'blacklist') NOT NULL,
    reason VARCHAR(500) NULL,
    expires_at TIMESTAMP NULL,
    created_by CHAR(36) NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (app_id) REFERENCES apps(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_ip_rules_lookup ON ip_rules(ip_address, rule_type);
CREATE INDEX idx_ip_rules_app ON ip_rules(app_id);

-- ============ WebAuthn/Passkeys ============

CREATE TABLE IF NOT EXISTS webauthn_credentials (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    credential_id VARBINARY(1024) NOT NULL,
    public_key VARBINARY(2048) NOT NULL,
    counter INT UNSIGNED NOT NULL DEFAULT 0,
    aaguid VARBINARY(16) NULL,
    device_name VARCHAR(255) NULL,
    transports JSON NULL, -- ["usb", "nfc", "ble", "internal"]
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_used_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE KEY unique_credential (credential_id(255))
);

CREATE INDEX idx_webauthn_user ON webauthn_credentials(user_id);

CREATE TABLE IF NOT EXISTS webauthn_challenges (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) NULL, -- NULL for registration
    challenge VARBINARY(64) NOT NULL,
    challenge_type ENUM('registration', 'authentication') NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_webauthn_challenges_expires ON webauthn_challenges(expires_at);
