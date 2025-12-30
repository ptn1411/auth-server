/**
 * Setup script to create admin user before running tests
 * Run: node setup-admin.js
 */
require('dotenv').config();
const request = require('supertest');

const API_URL = process.env.API_URL || 'http://localhost:3000';
const ADMIN_EMAIL = process.env.TEST_ADMIN_EMAIL || 'admin@test.com';
const ADMIN_PASSWORD = process.env.TEST_ADMIN_PASSWORD || 'Admin123!@#';

async function setupAdmin() {
  console.log('Setting up admin user...');
  console.log('API URL:', API_URL);
  
  // Check if server is running
  try {
    const healthRes = await request(API_URL).get('/health');
    if (healthRes.status !== 200) {
      console.error('Server is not healthy. Please start the server first.');
      process.exit(1);
    }
    console.log('Server is healthy');
  } catch (error) {
    console.error('Cannot connect to server. Please start the server first.');
    console.error('Run: cargo run --release');
    process.exit(1);
  }

  // Try to register admin user
  console.log(`Registering admin user: ${ADMIN_EMAIL}`);
  const registerRes = await request(API_URL)
    .post('/auth/register')
    .send({ email: ADMIN_EMAIL, password: ADMIN_PASSWORD });

  if (registerRes.status === 201) {
    console.log('Admin user registered successfully');
    console.log('User ID:', registerRes.body.id);
  } else if (registerRes.status === 409) {
    console.log('Admin user already exists');
  } else {
    console.error('Failed to register admin:', registerRes.body);
  }

  // Try to login
  const loginRes = await request(API_URL)
    .post('/auth/login')
    .send({ email: ADMIN_EMAIL, password: ADMIN_PASSWORD });

  if (loginRes.status === 200) {
    console.log('Admin login successful');
    console.log('\n⚠️  IMPORTANT: You need to manually set is_system_admin = TRUE');
    console.log('Run this command:');
    console.log('  cargo run --example set_admin');
    console.log('\nOr execute SQL:');
    console.log(`  UPDATE users SET is_system_admin = TRUE WHERE email = '${ADMIN_EMAIL}';`);
  } else {
    console.error('Admin login failed:', loginRes.body);
  }

  console.log('\nSetup complete!');
}

setupAdmin().catch(console.error);
