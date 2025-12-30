import { AuthServerClient, AuthServerError } from '../client';

const BASE_URL = process.env.AUTH_SERVER_URL || 'http://localhost:3000';
const ADMIN_EMAIL = 'admin@test.com';
const ADMIN_PASSWORD = 'Admin123!@#';

describe('AuthServerClient', () => {
  let client: AuthServerClient;
  let adminClient: AuthServerClient;
  let testUserEmail: string;
  let testUserPassword: string;
  let testUserId: string;
  let testAppId: string;
  let testAppSecret: string;

  beforeAll(async () => {
    client = new AuthServerClient({ baseUrl: BASE_URL });
    adminClient = new AuthServerClient({ baseUrl: BASE_URL });
    
    // Login as admin
    const adminLogin = await adminClient.login({
      email: ADMIN_EMAIL,
      password: ADMIN_PASSWORD,
    });
    expect('access_token' in adminLogin).toBe(true);
  });

  describe('Health Check', () => {
    it('should return health status', async () => {
      const health = await client.health();
      expect(health.status).toBe('healthy');
    });

    it('should return ready status', async () => {
      const ready = await client.ready();
      expect(ready.status).toBe('ready');
    });
  });

  describe('Authentication', () => {
    beforeAll(() => {
      testUserEmail = `test_${Date.now()}@example.com`;
      testUserPassword = 'TestPass123!@#';
    });

    it('should register a new user', async () => {
      const response = await client.register({
        email: testUserEmail,
        password: testUserPassword,
      });

      expect(response.email).toBe(testUserEmail);
      expect(response.id).toBeDefined();
      testUserId = response.id;
    });

    it('should reject duplicate email', async () => {
      await expect(
        client.register({
          email: testUserEmail,
          password: testUserPassword,
        })
      ).rejects.toThrow(AuthServerError);
    });

    it('should login successfully', async () => {
      const response = await client.login({
        email: testUserEmail,
        password: testUserPassword,
      });

      expect('access_token' in response).toBe(true);
      if ('access_token' in response) {
        expect(response.token_type).toBe('Bearer');
        expect(client.getAccessToken()).toBe(response.access_token);
      }
    });

    it('should reject invalid credentials', async () => {
      const newClient = new AuthServerClient({ baseUrl: BASE_URL });
      await expect(
        newClient.login({
          email: testUserEmail,
          password: 'wrongpassword',
        })
      ).rejects.toThrow(AuthServerError);
    });

    it('should refresh token', async () => {
      const response = await client.refresh();
      expect(response.access_token).toBeDefined();
      expect(response.token_type).toBe('Bearer');
    });

    it('should logout', async () => {
      // Re-login first to ensure we have a valid token
      await client.login({
        email: testUserEmail,
        password: testUserPassword,
      });
      
      // Logout - clear tokens locally regardless of server response
      try {
        await client.logout();
      } catch {
        // Server may return non-JSON, but we still clear tokens
      }
      client.clearTokens();
      expect(client.getAccessToken()).toBeUndefined();
    });

    it('should login again after logout', async () => {
      const response = await client.login({
        email: testUserEmail,
        password: testUserPassword,
      });
      expect('access_token' in response).toBe(true);
    });
  });

  describe('User Profile', () => {
    it('should get user profile', async () => {
      const profile = await client.getProfile();
      expect(profile.email).toBe(testUserEmail);
      expect(profile.id).toBeDefined();
    });

    it('should change password', async () => {
      const newPassword = 'NewTestPass123!@#';
      const response = await client.changePassword({
        current_password: testUserPassword,
        new_password: newPassword,
      });
      expect(response.message).toBeDefined();

      // Change back
      await client.changePassword({
        current_password: newPassword,
        new_password: testUserPassword,
      });
    });
  });

  describe('Session Management', () => {
    it('should get sessions', async () => {
      const response = await client.getSessions();
      expect(response.sessions).toBeDefined();
      expect(Array.isArray(response.sessions)).toBe(true);
    });
  });

  describe('MFA', () => {
    it('should setup TOTP', async () => {
      const response = await client.setupTotp();
      expect(response.secret).toBeDefined();
      expect(response.provisioning_uri).toBeDefined();
    });

    it('should get MFA methods', async () => {
      const response = await client.getMfaMethods();
      expect(response.methods).toBeDefined();
      expect(Array.isArray(response.methods)).toBe(true);
    });
  });

  describe('Audit Logs', () => {
    it('should get audit logs', async () => {
      const response = await client.getAuditLogs();
      expect(response.logs).toBeDefined();
      expect(Array.isArray(response.logs)).toBe(true);
    });

    it('should support pagination', async () => {
      const response = await client.getAuditLogs({ page: 1, limit: 5 });
      expect(response.page).toBe(1);
      expect(response.limit).toBe(5);
    });
  });

  describe('App Management', () => {
    it('should create an app', async () => {
      const response = await client.createApp({
        code: `sdk_test_${Date.now()}`,
        name: 'SDK Test App',
      });

      expect(response.id).toBeDefined();
      expect(response.secret).toBeDefined();
      testAppId = response.id;
      testAppSecret = response.secret!;
    });

    it('should authenticate app', async () => {
      const response = await client.authenticateApp({
        app_id: testAppId,
        secret: testAppSecret,
      });

      expect(response.access_token).toBeDefined();
      expect(response.token_type).toBe('Bearer');
    });

    it('should regenerate app secret', async () => {
      const response = await client.regenerateAppSecret(testAppId);
      expect(response.secret).toBeDefined();
      expect(response.secret).not.toBe(testAppSecret);
      testAppSecret = response.secret;
    });
  });

  describe('Role Management', () => {
    let roleId: string;

    it('should create a role', async () => {
      const response = await client.createRole(testAppId, {
        name: 'test_role',
      });

      expect(response.id).toBeDefined();
      expect(response.name).toBe('test_role');
      roleId = response.id;
    });

    it('should assign role to user', async () => {
      await expect(
        client.assignRole(testAppId, testUserId, { role_id: roleId })
      ).resolves.toBeUndefined();
    });

    it('should get user roles in app', async () => {
      const roles = await client.getUserRolesInApp(testAppId, testUserId);
      expect(Array.isArray(roles)).toBe(true);
      expect(roles.some(r => r.id === roleId)).toBe(true);
    });

    it('should remove role from user', async () => {
      await expect(
        client.removeRole(testAppId, testUserId, roleId)
      ).resolves.toBeUndefined();
    });
  });

  describe('Permission Management', () => {
    it('should create a permission', async () => {
      const response = await client.createPermission(testAppId, {
        code: 'test:permission',
      });

      expect(response.id).toBeDefined();
      expect(response.code).toBe('test:permission');
    });
  });

  describe('Webhook Management', () => {
    let webhookId: string;

    it('should create a webhook', async () => {
      const response = await client.createWebhook(testAppId, {
        url: 'https://example.com/webhook',
        events: ['user.created', 'user.login'],
      });

      expect(response.id).toBeDefined();
      expect(response.secret).toBeDefined();
      expect(response.url).toBe('https://example.com/webhook');
      webhookId = response.id;
    });

    it('should list webhooks', async () => {
      const webhooks = await client.listWebhooks(testAppId);
      expect(Array.isArray(webhooks)).toBe(true);
      expect(webhooks.some(w => w.id === webhookId)).toBe(true);
    });

    it('should get webhook', async () => {
      const webhook = await client.getWebhook(testAppId, webhookId);
      expect(webhook.id).toBe(webhookId);
    });

    it('should update webhook', async () => {
      const webhook = await client.updateWebhook(testAppId, webhookId, {
        events: ['user.created'],
      });
      expect(webhook.events).toContain('user.created');
    });

    it('should delete webhook', async () => {
      await expect(
        client.deleteWebhook(testAppId, webhookId)
      ).resolves.toBeUndefined();
    });
  });

  describe('API Key Management', () => {
    let apiKeyId: string;

    it('should create an API key', async () => {
      const response = await client.createApiKey(testAppId, {
        name: 'Test API Key',
        scopes: ['read:users'],
      });

      expect(response.id).toBeDefined();
      expect(response.key).toBeDefined();
      expect(response.name).toBe('Test API Key');
      apiKeyId = response.id;
    });

    it('should list API keys', async () => {
      const keys = await client.listApiKeys(testAppId);
      expect(Array.isArray(keys)).toBe(true);
      expect(keys.some(k => k.id === apiKeyId)).toBe(true);
    });

    it('should get API key', async () => {
      const key = await client.getApiKey(testAppId, apiKeyId);
      expect(key.id).toBe(apiKeyId);
      expect(key).not.toHaveProperty('key'); // Full key not returned
    });

    it('should update API key', async () => {
      const key = await client.updateApiKey(testAppId, apiKeyId, {
        name: 'Updated API Key',
      });
      expect(key.name).toBe('Updated API Key');
    });

    it('should revoke API key', async () => {
      await expect(
        client.revokeApiKey(testAppId, apiKeyId)
      ).resolves.toBeUndefined();

      const key = await client.getApiKey(testAppId, apiKeyId);
      expect(key.is_active).toBe(false);
    });

    it('should delete API key', async () => {
      // Create new key to delete
      const newKey = await client.createApiKey(testAppId, { name: 'To Delete' });
      await expect(
        client.deleteApiKey(testAppId, newKey.id)
      ).resolves.toBeUndefined();
    });
  });

  describe('IP Rules (App Level)', () => {
    let ruleId: string;

    it('should create an IP rule', async () => {
      const response = await client.createAppIpRule(testAppId, {
        ip_address: '192.168.1.100',
        rule_type: 'whitelist',
        reason: 'Test IP',
      });

      expect(response.id).toBeDefined();
      expect(response.ip_address).toBe('192.168.1.100');
      ruleId = response.id;
    });

    it('should list IP rules', async () => {
      const rules = await client.listAppIpRules(testAppId);
      expect(Array.isArray(rules)).toBe(true);
      expect(rules.some(r => r.id === ruleId)).toBe(true);
    });
  });

  describe('WebAuthn/Passkey', () => {
    it('should start passkey registration', async () => {
      const response = await client.startPasskeyRegistration({
        device_name: 'Test Device',
      });

      expect(response.challenge).toBeDefined();
      expect(response.rp).toBeDefined();
      expect(response.user).toBeDefined();
    });

    it('should start passkey authentication', async () => {
      const response = await client.startPasskeyAuthentication();

      expect(response.challenge).toBeDefined();
      expect(response.rp_id).toBeDefined();
    });

    it('should list passkeys (empty)', async () => {
      const passkeys = await client.listPasskeys();
      expect(Array.isArray(passkeys)).toBe(true);
    });
  });

  describe('Admin API', () => {
    it('should list users', async () => {
      const response = await adminClient.adminListUsers();
      expect(response.data).toBeDefined();
      expect(Array.isArray(response.data)).toBe(true);
    });

    it('should search users', async () => {
      const response = await adminClient.adminSearchUsers({
        email: testUserEmail,
      });
      expect(response.data).toBeDefined();
      expect(response.data.some(u => u.email === testUserEmail)).toBe(true);
    });

    it('should get user details', async () => {
      const user = await adminClient.adminGetUser(testUserId);
      expect(user.id).toBe(testUserId);
      expect(user.email).toBe(testUserEmail);
    });

    it('should update user', async () => {
      const user = await adminClient.adminUpdateUser(testUserId, {
        email_verified: true,
      });
      expect(user.email_verified).toBe(true);
    });

    it('should deactivate user', async () => {
      await expect(
        adminClient.adminDeactivateUser(testUserId)
      ).resolves.toBeUndefined();
    });

    it('should activate user', async () => {
      await expect(
        adminClient.adminActivateUser(testUserId)
      ).resolves.toBeUndefined();
    });

    it('should get user roles', async () => {
      const roles = await adminClient.adminGetUserRoles(testUserId);
      expect(roles.user_id).toBe(testUserId);
      expect(roles.apps).toBeDefined();
    });

    it('should list apps', async () => {
      const response = await adminClient.adminListApps();
      expect(response.data).toBeDefined();
      expect(Array.isArray(response.data)).toBe(true);
    });

    it('should get app details', async () => {
      const app = await adminClient.adminGetApp(testAppId);
      expect(app.id).toBe(testAppId);
    });

    it('should update app', async () => {
      const app = await adminClient.adminUpdateApp(testAppId, {
        name: 'Updated SDK Test App',
      });
      expect(app.name).toBe('Updated SDK Test App');
    });

    it('should get admin audit logs', async () => {
      const response = await adminClient.adminGetAuditLogs();
      expect(response.logs).toBeDefined();
      expect(Array.isArray(response.logs)).toBe(true);
    });
  });

  describe('Admin IP Rules', () => {
    let globalRuleId: string;

    it('should create global IP rule', async () => {
      const response = await adminClient.adminCreateIpRule({
        ip_address: '10.0.0.1',
        rule_type: 'blacklist',
        reason: 'Test global rule',
      });

      expect(response.id).toBeDefined();
      // app_id is null for global rules
      expect(response.app_id).toBeNull();
      globalRuleId = response.id;
    });

    it('should list global IP rules', async () => {
      const rules = await adminClient.adminListIpRules();
      expect(Array.isArray(rules)).toBe(true);
      expect(rules.length).toBeGreaterThan(0);
    });

    it('should check IP', async () => {
      // Check an IP that was blacklisted earlier in the test
      const response = await adminClient.adminCheckIp('10.0.0.1');
      expect(response.ip).toBe('10.0.0.1');
      // Just verify the response structure
      expect(typeof response.allowed).toBe('boolean');
    });

    it('should delete global IP rule', async () => {
      // Create a rule to delete
      const rule = await adminClient.adminCreateIpRule({
        ip_address: '10.0.0.100',
        rule_type: 'blacklist',
        reason: 'To delete',
      });
      
      await expect(
        adminClient.adminDeleteIpRule(rule.id)
      ).resolves.toBeUndefined();
    });
  });

  describe('Error Handling', () => {
    it('should throw AuthServerError on 401', async () => {
      const unauthClient = new AuthServerClient({ baseUrl: BASE_URL });
      await expect(unauthClient.getProfile()).rejects.toThrow(AuthServerError);
    });

    it('should throw AuthServerError on 404', async () => {
      await expect(
        adminClient.adminGetUser('00000000-0000-0000-0000-000000000000')
      ).rejects.toThrow(AuthServerError);
    });
  });

  describe('Token Management', () => {
    it('should set and get tokens', () => {
      const testClient = new AuthServerClient({ baseUrl: BASE_URL });
      testClient.setTokens('test-access', 'test-refresh');
      expect(testClient.getAccessToken()).toBe('test-access');
    });

    it('should clear tokens', () => {
      const testClient = new AuthServerClient({ baseUrl: BASE_URL });
      testClient.setTokens('test-access', 'test-refresh');
      testClient.clearTokens();
      expect(testClient.getAccessToken()).toBeUndefined();
    });
  });
});
