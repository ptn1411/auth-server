const { api, generateEmail, generatePassword } = require('./helpers');

const TOTAL = 1_000_000;
const CONCURRENCY = 50; // số request song song (đừng để quá cao)

async function registerOne() {
  const email = generateEmail();
  const password = generatePassword();

  return api()
    .post('/auth/register')
    .send({ email, password })
    .then(() => true)
    .catch(err => {
      console.error(err.response?.status);
      return false;
    });
}

async function run() {
  let success = 0;

  for (let i = 0; i < TOTAL; i += CONCURRENCY) {
    const batch = [];

    for (let j = 0; j < CONCURRENCY && i + j < TOTAL; j++) {
      batch.push(registerOne());
    }

    const results = await Promise.all(batch);
    success += results.filter(Boolean).length;

    if (i % 1000 === 0) {
      console.log(`Created ${success}/${TOTAL}`);
    }
  }

  console.log('DONE:', success);
}

run();
