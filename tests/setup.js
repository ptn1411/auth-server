require('dotenv').config();

// Global test setup
beforeAll(() => {
  console.log('Starting API tests against:', process.env.API_URL);
});

afterAll(() => {
  console.log('API tests completed');
});
