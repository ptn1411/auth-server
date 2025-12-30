const { api, createTestUser, getAdminToken, generateEmail, generatePassword, registerUser } = require('./helpers');

describe('Roles API', () => {
  let ownerToken;
  let appId;
  let roleId;
  let testUserId;

  beforeAll(async () => {
    // Create app owner
    const owner = await createTestUser();
    ownerToken = owner.token;

    // Create an app
    const appCode = `role-test-app-${Date.now()}`;
    const appRes = await api()
      .post('/apps')
      .set('Authorization', `Bearer ${ownerToken}`)
      .send({ code: appCode, name: 'Role Test App' });
    appId = appRes.body.id;

    // Create a role
    const roleRes = await api()
      .post(`/apps/${appId}/roles`)
      .set('Authorization', `Bearer ${ownerToken}`)
      .send({ name: 'member' });
    roleId = roleRes.body.id;

    // Create a test user
    const email = generateEmail();
    const password = generatePassword();
    const userRes = await registerUser(email, password);
    testUserId = userRes.body.id;
  });

  describe('POST /apps/:app_id/users/:user_id/roles', () => {
    it('should assign role to user', async () => {
      const res = await api()
        .post(`/apps/${appId}/users/${testUserId}/roles`)
        .set('Authorization', `Bearer ${ownerToken}`)
        .send({ role_id: roleId });

      expect(res.status).toBe(204);
    });

    it('should reject assigning non-existent role', async () => {
      const res = await api()
        .post(`/apps/${appId}/users/${testUserId}/roles`)
        .set('Authorization', `Bearer ${ownerToken}`)
        .send({ role_id: '00000000-0000-0000-0000-000000000000' });

      expect(res.status).toBe(404);
    });
  });

  describe('GET /apps/:app_id/users/:user_id/roles', () => {
    it('should get user roles in app', async () => {
      const res = await api()
        .get(`/apps/${appId}/users/${testUserId}/roles`)
        .set('Authorization', `Bearer ${ownerToken}`);

      expect(res.status).toBe(200);
      expect(Array.isArray(res.body)).toBe(true);
      expect(res.body.some(r => r.id === roleId)).toBe(true);
    });
  });

  describe('DELETE /apps/:app_id/users/:user_id/roles/:role_id', () => {
    let removeRoleId;
    let removeUserId;

    beforeAll(async () => {
      // Create another role
      const roleRes = await api()
        .post(`/apps/${appId}/roles`)
        .set('Authorization', `Bearer ${ownerToken}`)
        .send({ name: `remove-role-${Date.now()}` });
      removeRoleId = roleRes.body.id;

      // Create another user
      const email = generateEmail();
      const password = generatePassword();
      const userRes = await registerUser(email, password);
      removeUserId = userRes.body.id;

      // Assign role
      await api()
        .post(`/apps/${appId}/users/${removeUserId}/roles`)
        .set('Authorization', `Bearer ${ownerToken}`)
        .send({ role_id: removeRoleId });
    });

    it('should remove role from user', async () => {
      const res = await api()
        .delete(`/apps/${appId}/users/${removeUserId}/roles/${removeRoleId}`)
        .set('Authorization', `Bearer ${ownerToken}`);

      expect(res.status).toBe(204);

      // Verify role is removed
      const rolesRes = await api()
        .get(`/apps/${appId}/users/${removeUserId}/roles`)
        .set('Authorization', `Bearer ${ownerToken}`);

      expect(rolesRes.body.some(r => r.id === removeRoleId)).toBe(false);
    });
  });
});
