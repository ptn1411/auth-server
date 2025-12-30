const { api, createTestUser } = require('./helpers');

describe('API Keys', () => {
  let userToken;
  let appId;
  let apiKeyId;
  let apiKeySecret;

  beforeAll(async () => {
    const user = await createTestUser();
    userToken = user.token;

    // Create an app
    const appRes = await api()
      .post('/apps')
      .set('Authorization', `Bearer ${userToken}`)
      .send({ code: `apikey_test_${Date.now()}`, name: 'API Key Test App' });
    
    appId = appRes.body.id;
  });

  describe('POST /apps/:app_id/api-keys', () => {
    it('should create an API key', async () => {
      const res = await api()
        .post(`/apps/${appId}/api-keys`)
        .set('Authorization', `Bearer ${userToken}`)
        .send({
          name: 'Test API Key',
          scopes: ['read:users', 'write:users']
        });

      expect(res.status).toBe(201);
      expect(res.body).toHaveProperty('id');
      expect(res.body).toHaveProperty('key');
      expect(res.body).toHaveProperty('key_prefix');
      expect(res.body.name).toBe('Test API Key');
      expect(res.body.scopes).toContain('read:users');
      expect(res.body.is_active).toBe(true);
      
      apiKeyId = res.body.id;
      apiKeySecret = res.body.key;
    });

    it('should create an API key with expiration', async () => {
      const expiresAt = new Date(Date.now() + 86400000).toISOString(); // 1 day
      
      const res = await api()
        .post(`/apps/${appId}/api-keys`)
        .set('Authorization', `Bearer ${userToken}`)
        .send({
          name: 'Expiring Key',
          scopes: ['read:users'],
          expires_at: expiresAt
        });

      expect(res.status).toBe(201);
      expect(res.body.expires_at).toBeDefined();
    });
  });

  describe('GET /apps/:app_id/api-keys', () => {
    it('should list API keys', async () => {
      const res = await api()
        .get(`/apps/${appId}/api-keys`)
        .set('Authorization', `Bearer ${userToken}`);

      expect(res.status).toBe(200);
      expect(Array.isArray(res.body)).toBe(true);
      expect(res.body.length).toBeGreaterThan(0);
      
      // Should not expose full key
      expect(res.body[0]).not.toHaveProperty('key');
      expect(res.body[0]).toHaveProperty('key_prefix');
    });
  });

  describe('GET /apps/:app_id/api-keys/:key_id', () => {
    it('should get a specific API key', async () => {
      const res = await api()
        .get(`/apps/${appId}/api-keys/${apiKeyId}`)
        .set('Authorization', `Bearer ${userToken}`);

      expect(res.status).toBe(200);
      expect(res.body.id).toBe(apiKeyId);
      expect(res.body).not.toHaveProperty('key');
    });
  });

  describe('PUT /apps/:app_id/api-keys/:key_id', () => {
    it('should update an API key', async () => {
      const res = await api()
        .put(`/apps/${appId}/api-keys/${apiKeyId}`)
        .set('Authorization', `Bearer ${userToken}`)
        .send({
          name: 'Updated API Key',
          scopes: ['read:users', 'read:roles']
        });

      expect(res.status).toBe(200);
      expect(res.body.name).toBe('Updated API Key');
      expect(res.body.scopes).toContain('read:roles');
    });
  });

  describe('POST /apps/:app_id/api-keys/:key_id/revoke', () => {
    it('should revoke an API key', async () => {
      const res = await api()
        .post(`/apps/${appId}/api-keys/${apiKeyId}/revoke`)
        .set('Authorization', `Bearer ${userToken}`);

      expect(res.status).toBe(204);

      // Verify revocation
      const getRes = await api()
        .get(`/apps/${appId}/api-keys/${apiKeyId}`)
        .set('Authorization', `Bearer ${userToken}`);
      
      expect(getRes.body.is_active).toBe(false);
    });
  });

  describe('DELETE /apps/:app_id/api-keys/:key_id', () => {
    it('should delete an API key', async () => {
      // Create a new key to delete
      const createRes = await api()
        .post(`/apps/${appId}/api-keys`)
        .set('Authorization', `Bearer ${userToken}`)
        .send({ name: 'To Delete' });

      const keyId = createRes.body.id;

      const res = await api()
        .delete(`/apps/${appId}/api-keys/${keyId}`)
        .set('Authorization', `Bearer ${userToken}`);

      expect(res.status).toBe(204);
    });
  });
});
