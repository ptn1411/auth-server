const { api } = require('./helpers');

describe('Health Check API', () => {
  describe('GET /health', () => {
    it('should return healthy status', async () => {
      const res = await api().get('/health');

      expect(res.status).toBe(200);
      expect(res.body.status).toBe('healthy');
      expect(res.body).toHaveProperty('version');
    });
  });

  describe('GET /ready', () => {
    it('should return ready status when database is connected', async () => {
      const res = await api().get('/ready');

      expect(res.status).toBe(200);
      expect(res.body.status).toBe('ready');
      expect(res.body).toHaveProperty('version');
    });
  });
});
