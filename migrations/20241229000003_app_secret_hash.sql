-- Add secret_hash column to apps table for App Secret Authentication
-- This allows apps to authenticate using App ID + Secret instead of user JWT tokens
-- Requirements: 1.3 - Store only the hashed value using bcrypt

ALTER TABLE apps ADD COLUMN secret_hash VARCHAR(255);
