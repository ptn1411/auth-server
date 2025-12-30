const { execSync } = require('child_process');
const path = require('path');

module.exports = async () => {
  console.log('\n🔧 Setting up admin user before tests...');
  
  try {
    execSync('cargo run --example set_admin', { 
      cwd: path.join(__dirname, '..'),
      stdio: 'inherit'
    });
    console.log('✅ Admin user setup complete');
    
    // Wait a bit for database to sync
    await new Promise(resolve => setTimeout(resolve, 1000));
    console.log('');
  } catch (e) {
    console.log('⚠️ Could not run set_admin, admin may already be unlocked\n');
  }
};
