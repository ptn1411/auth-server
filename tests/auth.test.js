const { api, generateEmail, generatePassword, registerUser, login, getToken } = require('./helpers');

describe('Authentication API', () => {
  describe('POST /auth/register', () => {
    it('should register a new user successfully', async () => {
      const email = generateEmail();
      const password = generatePassword();

      const res = await api()
        .post('/auth/register')
        .send({ email, password });

      expect(res.status).toBe(201);
      expect(res.body).toHaveProperty('id');
      expect(res.body.email).toBe(email);
    });

    it('should reject duplicate email', async () => {
      const email = generateEmail();
      const password = generatePassword();

      // First registration
      await registerUser(email, password);

      // Second registration with same email
      const res = await api()
        .post('/auth/register')
        .send({ email, password });

      expect(res.status).toBe(409);
      expect(res.body.error).toBe('email_exists');
    });

    it('should reject invalid email format', async () => {
      const res = await api()
        .post('/auth/register')
        .send({ email: 'invalid-email', password: generatePassword() });

      expect(res.status).toBe(400);
      expect(res.body.error).toBe('invalid_email');
    });

    it('should reject weak password', async () => {
      const res = await api()
        .post('/auth/register')
        .send({ email: generateEmail(), password: '123' });

      expect(res.status).toBe(400);
      expect(res.body.error).toBe('weak_password');
    });
  });

  describe('POST /auth/login', () => {
    let testEmail, testPassword;

    beforeAll(async () => {
      testEmail = generateEmail();
      testPassword = generatePassword();
      await registerUser(testEmail, testPassword);
    });

    it('should login successfully with valid credentials', async () => {
      const res = await login(testEmail, testPassword);

      expect(res.status).toBe(200);
      expect(res.body).toHaveProperty('access_token');
      expect(res.body).toHaveProperty('refresh_token');
      expect(res.body.token_type).toBe('Bearer');
      expect(res.body.expires_in).toBe(900);
    });

    it('should reject invalid password', async () => {
      const res = await login(testEmail, 'wrongpassword');

      expect(res.status).toBe(401);
      expect(res.body.error).toBe('invalid_credentials');
    });

    it('should reject non-existent user', async () => {
      const res = await login('nonexistent@example.com', 'password');

      expect(res.status).toBe(401);
      expect(res.body.error).toBe('invalid_credentials');
    });
  });

  describe('POST /auth/refresh', () => {
    let refreshToken;

    beforeAll(async () => {
      const email = generateEmail();
      const password = generatePassword();
      await registerUser(email, password);
      const loginRes = await login(email, password);
      refreshToken = loginRes.body.refresh_token;
    });

    it('should refresh token successfully', async () => {
      const res = await api()
        .post('/auth/refresh')
        .send({ refresh_token: refreshToken });

      expect(res.status).toBe(200);
      expect(res.body).toHaveProperty('access_token');
      expect(res.body).toHaveProperty('refresh_token');
    });

    it('should reject invalid refresh token', async () => {
      const res = await api()
        .post('/auth/refresh')
        .send({ refresh_token: 'invalid-token' });

      expect(res.status).toBe(401);
    });
  });

  describe('POST /auth/forgot-password', () => {
    it('should always return success (security)', async () => {
      const res = await api()
        .post('/auth/forgot-password')
        .send({ email: 'any@example.com' });

      expect(res.status).toBe(200);
      expect(res.body).toHaveProperty('message');
    });
  });
});
