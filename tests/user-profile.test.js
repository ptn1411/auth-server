const { api, createTestUser, getToken, generateEmail, generatePassword, registerUser } = require('./helpers');

describe('User Profile API', () => {
  let token;
  let userEmail;

  beforeAll(async () => {
    const user = await createTestUser();
    token = user.token;
    userEmail = user.email;
  });

  describe('GET /users/me', () => {
    it('should get current user profile', async () => {
      const res = await api()
        .get('/users/me')
        .set('Authorization', `Bearer ${token}`);

      expect(res.status).toBe(200);
      expect(res.body).toHaveProperty('id');
      expect(res.body.email).toBe(userEmail);
      expect(res.body).toHaveProperty('is_active');
      expect(res.body).toHaveProperty('created_at');
    });

    it('should reject without token', async () => {
      const res = await api().get('/users/me');

      expect(res.status).toBe(401);
    });

    it('should reject invalid token', async () => {
      const res = await api()
        .get('/users/me')
        .set('Authorization', 'Bearer invalid-token');

      expect(res.status).toBe(401);
    });
  });

  describe('PUT /users/me', () => {
    it('should update user profile', async () => {
      const res = await api()
        .put('/users/me')
        .set('Authorization', `Bearer ${token}`)
        .send({
          name: 'Test User',
          phone: '+1234567890',
        });

      expect(res.status).toBe(200);
      expect(res.body.name).toBe('Test User');
      expect(res.body.phone).toBe('+1234567890');
    });
  });

  describe('POST /users/me/change-password', () => {
    let changePasswordToken;
    let currentPassword;

    beforeAll(async () => {
      const email = generateEmail();
      currentPassword = generatePassword();
      await registerUser(email, currentPassword);
      changePasswordToken = await getToken(email, currentPassword);
    });

    it('should change password successfully', async () => {
      const newPassword = generatePassword();

      const res = await api()
        .post('/users/me/change-password')
        .set('Authorization', `Bearer ${changePasswordToken}`)
        .send({
          current_password: currentPassword,
          new_password: newPassword,
        });

      expect(res.status).toBe(200);
      expect(res.body).toHaveProperty('message');
    });

    it('should reject wrong current password', async () => {
      const res = await api()
        .post('/users/me/change-password')
        .set('Authorization', `Bearer ${changePasswordToken}`)
        .send({
          current_password: 'wrongpassword',
          new_password: generatePassword(),
        });

      expect(res.status).toBe(401);
    });
  });
});
