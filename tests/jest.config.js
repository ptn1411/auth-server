module.exports = {
  testEnvironment: 'node',
  testTimeout: 30000,
  globalSetup: './globalSetup.js',
  setupFilesAfterEnv: ['./setup.js'],
  testMatch: ['**/*.test.js'],
  verbose: true,
};
