-- Migration: App User Management
-- Requirements: 9.1, 9.2, 9.3, 9.4

-- Add owner_id column to apps table (FK to users, nullable for existing apps)
-- Requirement 9.1
ALTER TABLE apps ADD COLUMN owner_id CHAR(36) NULL;
ALTER TABLE apps ADD CONSTRAINT fk_apps_owner FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE SET NULL;

-- Add is_system_admin column to users table (boolean, default false)
-- Requirement 9.3
ALTER TABLE users ADD COLUMN is_system_admin BOOLEAN NOT NULL DEFAULT FALSE;

-- Create user_apps table
-- Requirement 9.2, 9.4
CREATE TABLE user_apps (
    user_id CHAR(36) NOT NULL,
    app_id CHAR(36) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    banned_at TIMESTAMP NULL,
    banned_reason TEXT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, app_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (app_id) REFERENCES apps(id) ON DELETE CASCADE,
    CONSTRAINT chk_user_apps_status CHECK (status IN ('active', 'banned'))
);

-- Add indexes for user_apps table
CREATE INDEX idx_user_apps_app_id ON user_apps(app_id);
CREATE INDEX idx_user_apps_status ON user_apps(status);
CREATE INDEX idx_user_apps_user_id ON user_apps(user_id);

-- Add index for apps owner_id
CREATE INDEX idx_apps_owner_id ON apps(owner_id);
