-- Migration: MFA Pending Tokens
-- Table to store temporary MFA tokens during 2-step login

CREATE TABLE mfa_pending_tokens (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL,
    token_hash VARCHAR(255) NOT NULL,
    app_id CHAR(36) NULL,
    used BOOLEAN NOT NULL DEFAULT FALSE,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_mfa_pending_token_hash (token_hash),
    INDEX idx_mfa_pending_user_id (user_id),
    INDEX idx_mfa_pending_expires_at (expires_at)
);
