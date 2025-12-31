-- Seed default OAuth scopes
-- These are standard OpenID Connect scopes

INSERT INTO oauth_scopes (id, code, description, is_active) VALUES
(UUID(), 'openid', 'Verify your identity (required for OpenID Connect)', true),
(UUID(), 'profile', 'Access your basic profile information (name, picture)', true),
(UUID(), 'email', 'Access your email address', true),
(UUID(), 'profile.read', 'Read your profile information', true),
(UUID(), 'email.read', 'Read your email address', true),
(UUID(), 'offline_access', 'Access your data when you are not present (refresh tokens)', true)
ON DUPLICATE KEY UPDATE description = VALUES(description);
