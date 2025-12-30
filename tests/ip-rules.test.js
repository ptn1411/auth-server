const { api, createTestUser, getAdminToken } = require('./helpers');

describe('IP Rules API', () => {
  let adminToken;
  let userToken;
  let appId;
  let globalRuleId;
  let appRuleId;

  beforeAll(async () => {
    adminToken = await getAdminToken();
    
    const user = await createTestUser();
    userToken = user.token;

    // Create an app
    const appRes = await api()
      .post('/apps')
      .set('Authorization', `Bearer ${userToken}`)
      .send({ code: `iprule_test_${Date.now()}`, name: 'IP Rule Test App' });
    
    appId = appRes.body.id;
  });

  describe('Admin Global IP Rules', () => {
    describe('POST /admin/ip-rules', () => {
      it('should create a global blacklist rule', async () => {
        const res = await api()
          .post('/admin/ip-rules')
          .set('Authorization', `Bearer ${adminToken}`)
          .send({
            ip_address: '192.168.1.100',
            rule_type: 'blacklist',
            reason: 'Suspicious activity'
          });

        expect(res.status).toBe(201);
        expect(res.body).toHaveProperty('id');
        expect(res.body.ip_address).toBe('192.168.1.100');
        expect(res.body.rule_type).toBe('blacklist');
        expect(res.body.app_id).toBeNull();
        
        globalRuleId = res.body.id;
      });

      it('should create a global whitelist rule', async () => {
        const res = await api()
          .post('/admin/ip-rules')
          .set('Authorization', `Bearer ${adminToken}`)
          .send({
            ip_address: '10.0.0.0',
            ip_range: '10.0.0.0/8',
            rule_type: 'whitelist',
            reason: 'Internal network'
          });

        expect(res.status).toBe(201);
        expect(res.body.rule_type).toBe('whitelist');
        expect(res.body.ip_range).toBe('10.0.0.0/8');
      });

      it('should reject non-admin users', async () => {
        const res = await api()
          .post('/admin/ip-rules')
          .set('Authorization', `Bearer ${userToken}`)
          .send({
            ip_address: '1.2.3.4',
            rule_type: 'blacklist'
          });

        expect(res.status).toBe(403);
      });
    });

    describe('GET /admin/ip-rules', () => {
      it('should list global IP rules', async () => {
        const res = await api()
          .get('/admin/ip-rules')
          .set('Authorization', `Bearer ${adminToken}`);

        expect(res.status).toBe(200);
        expect(Array.isArray(res.body)).toBe(true);
        expect(res.body.length).toBeGreaterThan(0);
      });
    });

    describe('GET /admin/ip-rules/check', () => {
      it('should check if IP is blocked', async () => {
        const res = await api()
          .get('/admin/ip-rules/check')
          .query({ ip: '192.168.1.100' })
          .set('Authorization', `Bearer ${adminToken}`);

        expect(res.status).toBe(200);
        expect(res.body.ip).toBe('192.168.1.100');
        expect(res.body.allowed).toBe(false);
        expect(res.body.rule_type).toBe('blacklist');
      });

      it('should check if IP is allowed', async () => {
        const res = await api()
          .get('/admin/ip-rules/check')
          .query({ ip: '8.8.8.8' })
          .set('Authorization', `Bearer ${adminToken}`);

        expect(res.status).toBe(200);
        expect(res.body.allowed).toBe(true);
      });
    });

    describe('DELETE /admin/ip-rules/:rule_id', () => {
      it('should delete a global IP rule', async () => {
        const res = await api()
          .delete(`/admin/ip-rules/${globalRuleId}`)
          .set('Authorization', `Bearer ${adminToken}`);

        expect(res.status).toBe(204);
      });
    });
  });

  describe('App-level IP Rules', () => {
    describe('POST /apps/:app_id/ip-rules', () => {
      it('should create an app-level IP rule', async () => {
        const res = await api()
          .post(`/apps/${appId}/ip-rules`)
          .set('Authorization', `Bearer ${userToken}`)
          .send({
            ip_address: '172.16.0.1',
            rule_type: 'whitelist',
            reason: 'Office IP'
          });

        expect(res.status).toBe(201);
        expect(res.body.app_id).toBe(appId);
        expect(res.body.ip_address).toBe('172.16.0.1');
        
        appRuleId = res.body.id;
      });
    });

    describe('GET /apps/:app_id/ip-rules', () => {
      it('should list app IP rules', async () => {
        const res = await api()
          .get(`/apps/${appId}/ip-rules`)
          .set('Authorization', `Bearer ${userToken}`);

        expect(res.status).toBe(200);
        expect(Array.isArray(res.body)).toBe(true);
        expect(res.body.some(r => r.id === appRuleId)).toBe(true);
      });
    });
  });
});
