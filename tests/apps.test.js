const { api, createTestUser, getAdminToken } = require('./helpers');

describe('Apps API', () => {
  let token;
  let appId;
  let appCode;
  let appSecret;

  beforeAll(async () => {
    const user = await createTestUser();
    token = user.token;
    appCode = `app-${Date.now()}`;
    
    // Create app immediately in beforeAll
    const res = await api()
      .post('/apps')
      .set('Authorization', `Bearer ${token}`)
      .send({
        code: appCode,
        name: 'Test Application',
      });
    
    if (res.status === 201) {
      appId = res.body.id;
      appSecret = res.body.secret;
    }
  });

  describe('POST /apps', () => {
    it('should create a new app', async () => {
      const newAppCode = `new-app-${Date.now()}`;
      const res = await api()
        .post('/apps')
        .set('Authorization', `Bearer ${token}`)
        .send({
          code: newAppCode,
          name: 'New Test Application',
        });

      expect(res.status).toBe(201);
      expect(res.body).toHaveProperty('id');
      expect(res.body.code).toBe(newAppCode);
      expect(res.body.name).toBe('New Test Application');
      expect(res.body).toHaveProperty('secret');
    });

    it('should reject duplicate app code', async () => {
      const res = await api()
        .post('/apps')
        .set('Authorization', `Bearer ${token}`)
        .send({
          code: appCode,
          name: 'Another App',
        });

      expect(res.status).toBe(409);
      expect(res.body.error).toBe('app_code_exists');
    });

    it('should reject without authentication', async () => {
      const res = await api()
        .post('/apps')
        .send({
          code: 'new-app',
          name: 'New App',
        });

      expect(res.status).toBe(401);
    });
  });

  describe('POST /apps/:app_id/roles', () => {
    it('should create a role for the app', async () => {
      expect(appId).toBeDefined();
      
      const res = await api()
        .post(`/apps/${appId}/roles`)
        .set('Authorization', `Bearer ${token}`)
        .send({ name: `admin-${Date.now()}` });

      expect(res.status).toBe(201);
      expect(res.body).toHaveProperty('id');
      expect(res.body).toHaveProperty('name');
      expect(res.body.app_id).toBe(appId);
    });

    it('should reject duplicate role name in same app', async () => {
      const roleName = `dup-role-${Date.now()}`;
      
      // Create first
      await api()
        .post(`/apps/${appId}/roles`)
        .set('Authorization', `Bearer ${token}`)
        .send({ name: roleName });

      // Try duplicate
      const res = await api()
        .post(`/apps/${appId}/roles`)
        .set('Authorization', `Bearer ${token}`)
        .send({ name: roleName });

      expect(res.status).toBe(409);
    });
  });

  describe('POST /apps/:app_id/permissions', () => {
    it('should create a permission for the app', async () => {
      expect(appId).toBeDefined();
      
      const permCode = `perm-${Date.now()}`;
      const res = await api()
        .post(`/apps/${appId}/permissions`)
        .set('Authorization', `Bearer ${token}`)
        .send({ code: permCode });

      expect(res.status).toBe(201);
      expect(res.body).toHaveProperty('id');
      expect(res.body.code).toBe(permCode);
      expect(res.body.app_id).toBe(appId);
    });

    it('should reject duplicate permission code in same app', async () => {
      const permCode = `dup-perm-${Date.now()}`;
      
      // Create first
      await api()
        .post(`/apps/${appId}/permissions`)
        .set('Authorization', `Bearer ${token}`)
        .send({ code: permCode });

      // Try duplicate
      const res = await api()
        .post(`/apps/${appId}/permissions`)
        .set('Authorization', `Bearer ${token}`)
        .send({ code: permCode });

      expect(res.status).toBe(409);
    });
  });

  describe('POST /apps/auth', () => {
    let authAppId;
    let authAppSecret;

    beforeAll(async () => {
      // Create a new app to get fresh secret
      const newAppCode = `auth-app-${Date.now()}`;
      const res = await api()
        .post('/apps')
        .set('Authorization', `Bearer ${token}`)
        .send({
          code: newAppCode,
          name: 'Auth Test App',
        });
      authAppId = res.body.id;
      authAppSecret = res.body.secret;
    });

    it('should authenticate app with valid credentials', async () => {
      expect(authAppId).toBeDefined();
      expect(authAppSecret).toBeDefined();
      
      const res = await api()
        .post('/apps/auth')
        .send({
          app_id: authAppId,
          secret: authAppSecret,
        });

      expect(res.status).toBe(200);
      expect(res.body).toHaveProperty('access_token');
      expect(res.body.token_type).toBe('Bearer');
    });

    it('should reject invalid app secret', async () => {
      const res = await api()
        .post('/apps/auth')
        .send({
          app_id: authAppId,
          secret: 'wrong-secret',
        });

      expect(res.status).toBe(401);
    });
  });
});
