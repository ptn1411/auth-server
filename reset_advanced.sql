DROP TABLE IF EXISTS webauthn_challenges;
DROP TABLE IF EXISTS webauthn_credentials;
DROP TABLE IF EXISTS ip_rules;
DROP TABLE IF EXISTS api_keys;
DROP TABLE IF EXISTS rate_limits;
DROP TABLE IF EXISTS webhook_deliveries;
DROP TABLE IF EXISTS webhooks;
DELETE FROM _sqlx_migrations WHERE version = 20241229000008;
