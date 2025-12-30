const { api, getAdminToken, createTestUser, generateEmail, generatePassword, registerUser } = require('./helpers');

describe('Admin API', () => {
  let adminToken;
  let testUserId;
  let testAppId;

  beforeAll(async () => {
    adminToken = await getAdminToken();
    
    // Create a test user for admin operations
    const email = generateEmail();
    const password = generatePassword();
    const regRes = await registerUser(email, password);
    testUserId = regRes.body.id;
  });

  describe('GET /admin/users', () => {
    it('should list all users (admin only)', async () => {
      const res = await api()
        .get('/admin/users')
        .set('Authorization', `Bearer ${adminToken}`);

      expect(res.status).toBe(200);
      expect(res.body).toHaveProperty('data');
      expect(res.body).toHaveProperty('page');
      expect(res.body).toHaveProperty('limit');
      expect(res.body).toHaveProperty('total');
      expect(Array.isArray(res.body.data)).toBe(true);
    });

    it('should support pagination', async () => {
      const res = await api()
        .get('/admin/users?page=1&limit=5')
        .set('Authorization', `Bearer ${adminToken}`);

      expect(res.status).toBe(200);
      expect(res.body.page).toBe(1);
      expect(res.body.limit).toBe(5);
    });

    it('should reject non-admin users', async () => {
      const user = await createTestUser();
      
      const res = await api()
        .get('/admin/users')
        .set('Authorization', `Bearer ${user.token}`);

      expect(res.status).toBe(403);
      expect(res.body.error).toBe('not_system_admin');
    });
  });

  describe('GET /admin/users/:user_id', () => {
    it('should get user details', async () => {
      const res = await api()
        .get(`/admin/users/${testUserId}`)
        .set('Authorization', `Bearer ${adminToken}`);

      expect(res.status).toBe(200);
      expect(res.body).toHaveProperty('id');
      expect(res.body).toHaveProperty('email');
      expect(res.body).toHaveProperty('is_active');
      expect(res.body).toHaveProperty('is_system_admin');
      expect(res.body).toHaveProperty('mfa_enabled');
    });

    it('should return 404 for non-existent user', async () => {
      const res = await api()
        .get('/admin/users/00000000-0000-0000-0000-000000000000')
        .set('Authorization', `Bearer ${adminToken}`);

      expect(res.status).toBe(404);
    });
  });

  describe('PUT /admin/users/:user_id', () => {
    it('should update user', async () => {
      const res = await api()
        .put(`/admin/users/${testUserId}`)
        .set('Authorization', `Bearer ${adminToken}`)
        .send({ email_verified: true });

      expect(res.status).toBe(200);
      expect(res.body.email_verified).toBe(true);
    });

    it('should update multiple fields', async () => {
      const res = await api()
        .put(`/admin/users/${testUserId}`)
        .set('Authorization', `Bearer ${adminToken}`)
        .send({
          is_active: true,
          email_verified: true,
        });

      expect(res.status).toBe(200);
      expect(res.body.is_active).toBe(true);
      expect(res.body.email_verified).toBe(true);
    });
  });

  describe('POST /admin/users/:user_id/deactivate', () => {
    let deactivateUserId;

    beforeAll(async () => {
      const email = generateEmail();
      const password = generatePassword();
      const res = await registerUser(email, password);
      deactivateUserId = res.body.id;
    });

    it('should deactivate user', async () => {
      const res = await api()
        .post(`/admin/users/${deactivateUserId}/deactivate`)
        .set('Authorization', `Bearer ${adminToken}`);

      expect(res.status).toBe(204);

      // Verify user is deactivated
      const userRes = await api()
        .get(`/admin/users/${deactivateUserId}`)
        .set('Authorization', `Bearer ${adminToken}`);

      expect(userRes.body.is_active).toBe(false);
    });
  });

  describe('POST /admin/users/:user_id/activate', () => {
    let activateUserId;

    beforeAll(async () => {
      const email = generateEmail();
      const password = generatePassword();
      const res = await registerUser(email, password);
      activateUserId = res.body.id;

      // Deactivate first
      await api()
        .post(`/admin/users/${activateUserId}/deactivate`)
        .set('Authorization', `Bearer ${adminToken}`);
    });

    it('should activate user', async () => {
      const res = await api()
        .post(`/admin/users/${activateUserId}/activate`)
        .set('Authorization', `Bearer ${adminToken}`);

      expect(res.status).toBe(204);

      // Verify user is activated
      const userRes = await api()
        .get(`/admin/users/${activateUserId}`)
        .set('Authorization', `Bearer ${adminToken}`);

      expect(userRes.body.is_active).toBe(true);
    });
  });

  describe('GET /admin/apps', () => {
    it('should list all apps', async () => {
      const res = await api()
        .get('/admin/apps')
        .set('Authorization', `Bearer ${adminToken}`);

      expect(res.status).toBe(200);
      expect(res.body).toHaveProperty('data');
      expect(res.body).toHaveProperty('total');
      expect(Array.isArray(res.body.data)).toBe(true);

      if (res.body.data.length > 0) {
        testAppId = res.body.data[0].id;
      }
    });
  });

  describe('GET /admin/apps/:app_id', () => {
    it('should get app details', async () => {
      if (!testAppId) {
        console.log('Skipping - no apps available');
        return;
      }

      const res = await api()
        .get(`/admin/apps/${testAppId}`)
        .set('Authorization', `Bearer ${adminToken}`);

      expect(res.status).toBe(200);
      expect(res.body).toHaveProperty('id');
      expect(res.body).toHaveProperty('code');
      expect(res.body).toHaveProperty('name');
      expect(res.body).toHaveProperty('has_secret');
    });
  });

  describe('PUT /admin/apps/:app_id', () => {
    it('should update app', async () => {
      if (!testAppId) {
        console.log('Skipping - no apps available');
        return;
      }

      const newName = `Updated App ${Date.now()}`;
      const res = await api()
        .put(`/admin/apps/${testAppId}`)
        .set('Authorization', `Bearer ${adminToken}`)
        .send({ name: newName });

      expect(res.status).toBe(200);
      expect(res.body.name).toBe(newName);
    });
  });

  describe('GET /admin/users/:user_id/roles', () => {
    it('should get user roles across all apps', async () => {
      const res = await api()
        .get(`/admin/users/${testUserId}/roles`)
        .set('Authorization', `Bearer ${adminToken}`);

      expect(res.status).toBe(200);
      expect(res.body).toHaveProperty('user_id');
      expect(res.body).toHaveProperty('apps');
      expect(Array.isArray(res.body.apps)).toBe(true);
    });
  });

  describe('GET /admin/users/search', () => {
    it('should search users by email', async () => {
      const res = await api()
        .get('/admin/users/search?email=test')
        .set('Authorization', `Bearer ${adminToken}`);

      expect(res.status).toBe(200);
      expect(res.body).toHaveProperty('data');
    });

    it('should filter by is_active', async () => {
      const res = await api()
        .get('/admin/users/search?is_active=true')
        .set('Authorization', `Bearer ${adminToken}`);

      expect(res.status).toBe(200);
      res.body.data.forEach(user => {
        expect(user.is_active).toBe(true);
      });
    });
  });

  describe('DELETE /admin/users/:user_id', () => {
    let deleteUserId;

    beforeAll(async () => {
      const email = generateEmail();
      const password = generatePassword();
      const res = await registerUser(email, password);
      deleteUserId = res.body.id;
    });

    it('should delete user permanently', async () => {
      const res = await api()
        .delete(`/admin/users/${deleteUserId}`)
        .set('Authorization', `Bearer ${adminToken}`);

      expect(res.status).toBe(204);

      // Verify user is deleted
      const userRes = await api()
        .get(`/admin/users/${deleteUserId}`)
        .set('Authorization', `Bearer ${adminToken}`);

      expect(userRes.status).toBe(404);
    });
  });
});
