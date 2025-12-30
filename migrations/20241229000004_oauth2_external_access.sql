-- Migration: OAuth2 External Access
-- Requirements: 1.1, 2.1, 4.3, 5.6, 9.5, 10.6

-- OAuth Clients table
-- Requirement 1.1: Store client_id, client_secret, redirect_uris, and is_internal flag
CREATE TABLE oauth_clients (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    client_id VARCHAR(64) UNIQUE NOT NULL,
    client_secret_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    redirect_uris JSON NOT NULL,
    is_internal BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- OAuth Scopes table
-- Requirement 2.1: Support defining scopes with unique code and description
CREATE TABLE oauth_scopes (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    code VARCHAR(100) UNIQUE NOT NULL,
    description TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- User Consents table
-- Requirement 4.3: Store consent record with user_id, client_id, scopes, and timestamp
CREATE TABLE user_consents (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL,
    client_id CHAR(36) NOT NULL,
    scopes JSON NOT NULL,
    granted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY unique_user_client (user_id, client_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (client_id) REFERENCES oauth_clients(id) ON DELETE CASCADE
);

-- Authorization Codes table
-- Requirement 3.4: Store authorization code with PKCE support
CREATE TABLE oauth_authorization_codes (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    code_hash VARCHAR(255) NOT NULL,
    client_id CHAR(36) NOT NULL,
    user_id CHAR(36) NOT NULL,
    redirect_uri VARCHAR(2048) NOT NULL,
    scopes JSON NOT NULL,
    code_challenge VARCHAR(128) NOT NULL,
    code_challenge_method VARCHAR(10) NOT NULL DEFAULT 'S256',
    expires_at TIMESTAMP NOT NULL,
    used BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (client_id) REFERENCES oauth_clients(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- OAuth Tokens table
-- Requirement 5.1, 5.6: Store tokens with hashed values
CREATE TABLE oauth_tokens (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36),
    client_id CHAR(36) NOT NULL,
    access_token_hash VARCHAR(255) NOT NULL,
    refresh_token_hash VARCHAR(255),
    scopes JSON NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    revoked BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (client_id) REFERENCES oauth_clients(id) ON DELETE CASCADE
);

-- OAuth Audit Logs table
-- Requirement 9.5, 10.6: Log all authorization events for audit
CREATE TABLE oauth_audit_logs (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    event_type VARCHAR(50) NOT NULL,
    client_id CHAR(36),
    user_id CHAR(36),
    ip_address VARCHAR(45),
    details JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX idx_oauth_clients_client_id ON oauth_clients(client_id);
CREATE INDEX idx_oauth_clients_is_active ON oauth_clients(is_active);
CREATE INDEX idx_oauth_scopes_code ON oauth_scopes(code);
CREATE INDEX idx_oauth_scopes_is_active ON oauth_scopes(is_active);
CREATE INDEX idx_user_consents_user_id ON user_consents(user_id);
CREATE INDEX idx_user_consents_client_id ON user_consents(client_id);
CREATE INDEX idx_oauth_codes_code_hash ON oauth_authorization_codes(code_hash);
CREATE INDEX idx_oauth_codes_client_id ON oauth_authorization_codes(client_id);
CREATE INDEX idx_oauth_codes_user_id ON oauth_authorization_codes(user_id);
CREATE INDEX idx_oauth_codes_expires_at ON oauth_authorization_codes(expires_at);
CREATE INDEX idx_oauth_tokens_access_hash ON oauth_tokens(access_token_hash);
CREATE INDEX idx_oauth_tokens_refresh_hash ON oauth_tokens(refresh_token_hash);
CREATE INDEX idx_oauth_tokens_user_id ON oauth_tokens(user_id);
CREATE INDEX idx_oauth_tokens_client_id ON oauth_tokens(client_id);
CREATE INDEX idx_oauth_tokens_revoked ON oauth_tokens(revoked);
CREATE INDEX idx_oauth_audit_user_id ON oauth_audit_logs(user_id);
CREATE INDEX idx_oauth_audit_client_id ON oauth_audit_logs(client_id);
CREATE INDEX idx_oauth_audit_event_type ON oauth_audit_logs(event_type);
CREATE INDEX idx_oauth_audit_created_at ON oauth_audit_logs(created_at);
