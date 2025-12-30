const { api, createTestUser } = require('./helpers');

describe('WebAuthn/Passkey API', () => {
  let userToken;

  beforeAll(async () => {
    const user = await createTestUser();
    userToken = user.token;
  });

  describe('POST /auth/webauthn/register/start', () => {
    it('should start passkey registration', async () => {
      const res = await api()
        .post('/auth/webauthn/register/start')
        .set('Authorization', `Bearer ${userToken}`)
        .send({ device_name: 'Test Device' });

      expect(res.status).toBe(200);
      expect(res.body).toHaveProperty('challenge');
      expect(res.body).toHaveProperty('rp');
      expect(res.body.rp).toHaveProperty('id');
      expect(res.body.rp).toHaveProperty('name');
      expect(res.body).toHaveProperty('user');
      expect(res.body.user).toHaveProperty('id');
      expect(res.body.user).toHaveProperty('name');
      expect(res.body).toHaveProperty('pub_key_cred_params');
      expect(res.body).toHaveProperty('timeout');
      expect(res.body).toHaveProperty('authenticator_selection');
    });

    it('should require authentication', async () => {
      const res = await api()
        .post('/auth/webauthn/register/start')
        .send({});

      expect(res.status).toBe(401);
    });
  });

  describe('POST /auth/webauthn/authenticate/start', () => {
    it('should start passkey authentication (public)', async () => {
      const res = await api()
        .post('/auth/webauthn/authenticate/start')
        .send({});

      expect(res.status).toBe(200);
      expect(res.body).toHaveProperty('challenge');
      expect(res.body).toHaveProperty('timeout');
      expect(res.body).toHaveProperty('rp_id');
      expect(res.body).toHaveProperty('allow_credentials');
      expect(res.body).toHaveProperty('user_verification');
    });

    it('should accept email hint', async () => {
      const res = await api()
        .post('/auth/webauthn/authenticate/start')
        .send({ email: 'test@example.com' });

      // Should work even if user doesn't exist (returns empty credentials)
      expect(res.status).toBe(200);
    });
  });

  describe('GET /auth/webauthn/credentials', () => {
    it('should list user passkeys (empty initially)', async () => {
      const res = await api()
        .get('/auth/webauthn/credentials')
        .set('Authorization', `Bearer ${userToken}`);

      expect(res.status).toBe(200);
      expect(Array.isArray(res.body)).toBe(true);
      expect(res.body.length).toBe(0);
    });

    it('should require authentication', async () => {
      const res = await api()
        .get('/auth/webauthn/credentials');

      expect(res.status).toBe(401);
    });
  });

  // Note: Full registration/authentication flow requires browser WebAuthn API
  // These tests verify the server endpoints work correctly
  describe('POST /auth/webauthn/register/finish', () => {
    it('should reject invalid registration response', async () => {
      const res = await api()
        .post('/auth/webauthn/register/finish')
        .set('Authorization', `Bearer ${userToken}`)
        .send({
          id: 'invalid',
          raw_id: 'invalid',
          response: {
            client_data_json: 'invalid',
            attestation_object: 'invalid'
          },
          type: 'public-key'
        });

      expect(res.status).toBe(400);
    });
  });

  describe('POST /auth/webauthn/authenticate/finish', () => {
    it('should reject invalid authentication response', async () => {
      const res = await api()
        .post('/auth/webauthn/authenticate/finish')
        .send({
          id: 'invalid',
          raw_id: 'invalid',
          response: {
            client_data_json: 'invalid',
            authenticator_data: 'invalid',
            signature: 'invalid'
          },
          type: 'public-key'
        });

      expect(res.status).toBe(400);
    });
  });
});
