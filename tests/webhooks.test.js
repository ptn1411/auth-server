const { api, createTestUser, getAdminToken } = require('./helpers');

describe('Webhooks API', () => {
  let userToken;
  let appId;
  let webhookId;

  beforeAll(async () => {
    const user = await createTestUser();
    userToken = user.token;

    // Create an app
    const appRes = await api()
      .post('/apps')
      .set('Authorization', `Bearer ${userToken}`)
      .send({ code: `webhook_test_${Date.now()}`, name: 'Webhook Test App' });
    
    appId = appRes.body.id;
  });

  describe('POST /apps/:app_id/webhooks', () => {
    it('should create a webhook', async () => {
      const res = await api()
        .post(`/apps/${appId}/webhooks`)
        .set('Authorization', `Bearer ${userToken}`)
        .send({
          url: 'http://localhost:8080/webhook',
          events: ['user.login', 'user.register']
        });

      expect(res.status).toBe(201);
      expect(res.body).toHaveProperty('id');
      expect(res.body).toHaveProperty('secret');
      expect(res.body.url).toBe('http://localhost:8080/webhook');
      expect(res.body.events).toContain('user.login');
      expect(res.body.is_active).toBe(true);
      
      webhookId = res.body.id;
    });

    it('should reject non-HTTPS URLs (except localhost)', async () => {
      const res = await api()
        .post(`/apps/${appId}/webhooks`)
        .set('Authorization', `Bearer ${userToken}`)
        .send({
          url: 'http://example.com/webhook',
          events: ['user.login']
        });

      expect(res.status).toBe(400);
    });
  });

  describe('GET /apps/:app_id/webhooks', () => {
    it('should list webhooks', async () => {
      const res = await api()
        .get(`/apps/${appId}/webhooks`)
        .set('Authorization', `Bearer ${userToken}`);

      expect(res.status).toBe(200);
      expect(Array.isArray(res.body)).toBe(true);
      expect(res.body.length).toBeGreaterThan(0);
    });
  });

  describe('GET /apps/:app_id/webhooks/:webhook_id', () => {
    it('should get a specific webhook', async () => {
      const res = await api()
        .get(`/apps/${appId}/webhooks/${webhookId}`)
        .set('Authorization', `Bearer ${userToken}`);

      expect(res.status).toBe(200);
      expect(res.body.id).toBe(webhookId);
    });
  });

  describe('PUT /apps/:app_id/webhooks/:webhook_id', () => {
    it('should update a webhook', async () => {
      const res = await api()
        .put(`/apps/${appId}/webhooks/${webhookId}`)
        .set('Authorization', `Bearer ${userToken}`)
        .send({
          events: ['user.login', 'user.logout'],
          is_active: false
        });

      expect(res.status).toBe(200);
      expect(res.body.events).toContain('user.logout');
      expect(res.body.is_active).toBe(false);
    });
  });

  describe('DELETE /apps/:app_id/webhooks/:webhook_id', () => {
    it('should delete a webhook', async () => {
      const res = await api()
        .delete(`/apps/${appId}/webhooks/${webhookId}`)
        .set('Authorization', `Bearer ${userToken}`);

      expect(res.status).toBe(204);

      // Verify deletion
      const getRes = await api()
        .get(`/apps/${appId}/webhooks/${webhookId}`)
        .set('Authorization', `Bearer ${userToken}`);
      
      expect(getRes.status).toBe(404);
    });
  });
});
