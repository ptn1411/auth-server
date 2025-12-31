-- Migration: Add owner_id to OAuth Clients
-- Fix security issue: OAuth clients should have an owner who can manage them

-- Add owner_id column to oauth_clients
ALTER TABLE oauth_clients 
ADD COLUMN owner_id CHAR(36) NULL AFTER name;

-- Add foreign key constraint
ALTER TABLE oauth_clients
ADD CONSTRAINT fk_oauth_clients_owner 
FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE SET NULL;

-- Add index for performance
CREATE INDEX idx_oauth_clients_owner_id ON oauth_clients(owner_id);

-- Note: Existing clients will have NULL owner_id
-- They can be claimed by system admin or left as system-owned clients
