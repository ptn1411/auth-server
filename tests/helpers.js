process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0'; // Allow self-signed certs for testing

require('dotenv').config();
const request = require('supertest');

const API_URL = process.env.API_URL || 'http://localhost:3000';
console.log(API_URL)
/**
 * Create a supertest request instance
 */
const api = () => request(API_URL);

/**
 * Generate a unique email for testing
 */
const generateEmail = () => `test_${Date.now()}_${Math.random().toString(36).substring(2, 11)}@example.com`;

/**
 * Generate a strong password
 */
const generatePassword = () => `Test${Date.now()}!@#`;

/**
 * Register a new user
 */
async function registerUser(email, password) {
  const res = await api()
    .post('/auth/register')
    .send({ email, password });
  return res;
}

/**
 * Login and get tokens
 */
async function login(email, password) {
  const res = await api()
    .post('/auth/login')
    .send({ email, password });
  return res;
}

/**
 * Get access token for a user
 */
async function getToken(email, password) {
  const res = await login(email, password);
  if (res.status === 200 && res.body.access_token) {
    return res.body.access_token;
  }
  throw new Error(`Failed to get token: ${JSON.stringify(res.body)}`);
}

/**
 * Create a test user and return their token
 */
async function createTestUser() {
  const email = generateEmail();
  const password = generatePassword();
  
  await registerUser(email, password);
  const token = await getToken(email, password);
  
  return { email, password, token };
}

/**
 * Get admin token (assumes admin user exists)
 */
async function getAdminToken() {
  const email = process.env.TEST_ADMIN_EMAIL || 'admin@test.com';
  const password = process.env.TEST_ADMIN_PASSWORD || 'Admin123!@#';
  return getToken(email, password);
}

module.exports = {
  api,
  generateEmail,
  generatePassword,
  registerUser,
  login,
  getToken,
  createTestUser,
  getAdminToken,
  API_URL,
};
