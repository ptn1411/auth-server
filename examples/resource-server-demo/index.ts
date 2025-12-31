import { AuthServerClient } from "auth-server-sdk";
import dotenv from "dotenv";
import readline from "readline";

dotenv.config();

// Define the RoleResponse interface
interface RoleResponse {
  id: string;
  name: string;
  // Add other properties as needed based on your API response
}

interface UserResponse {
  id: string;
  email: string;
  // Add other properties as needed
}

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

const question = (query: string): Promise<string> => {
  return new Promise((resolve) => {
    rl.question(query, resolve);
  });
};

async function main() {
  console.log("--- Auth Server Resource Server Demo ---");

  const baseUrl = process.env.AUTH_SERVER_URL || "http://localhost:3000";
  const authClient = new AuthServerClient({ baseUrl });

  try {
    // 1. Authenticate
    console.log(
      "\nPlease enter your App Credentials (create one at http://localhost:5173/apps):"
    );
    const appId = process.env.APP_ID || (await question("App ID (UUID): "));
    const appSecret =
      process.env.APP_SECRET || (await question("App Secret: "));

    console.log(`\nLogging in as App ${appId}...`);
    const tokenResponse = await authClient.authenticateApp({
      app_id: appId,
      secret: appSecret,
    });

    console.log("✅ Login successful!");
    console.log(
      "Access Token:",
      tokenResponse.access_token.substring(0, 20) + "..."
    );

    // 2. Create a Role
    console.log("\n--- Creating a Role ---");
    const roleName = await question(
      'Enter a name for a new role (e.g., "editor"): '
    );

    try {
      const role = await authClient.createRole(appId, {
        name: roleName,
      });
      console.log("✅ Role created:", role);
    } catch (e: any) {
      console.error("❌ Failed to create role (maybe it exists?):", e.message);
    }

    // 3. Create a Permission
    console.log("\n--- Creating a Permission ---");
    const permCode = await question(
      'Enter a permission code (e.g., "posts:read"): '
    );

    try {
      const perm = await authClient.createPermission(appId, {
        code: permCode,
      });
      console.log("✅ Permission created:", perm);
    } catch (e: any) {
      console.error("❌ Failed to create permission:", e.message);
    }

    // 4. List App Users
    console.log("\n--- Listing App Users ---");
    try {
      const users = await authClient.getAppUsers(appId);
      console.log(`Found ${users.total} users.`);
      if (users.total > 0) {
        console.log("First user:", users.users[0]);

        // 5. Assign Role to User
        const confirm = await question(
          `Assign role "${roleName}" to user ${users.users[0].email}? (y/n): `
        );
        if (confirm.toLowerCase() === "y") {
          // Fetch roles with explicit typing
          const roles: RoleResponse[] = await authClient.listRoles(appId);
          const targetRole = roles.find(
            (r: RoleResponse) => r.name === roleName
          );

          if (targetRole) {
            await authClient.assignRole(appId, users.users[0].id, {
              role_id: targetRole.id,
            });
            console.log("✅ Role assigned!");
          } else {
            console.log("Could not find the role object to assign.");
          }
        }
      }
    } catch (e: any) {
      console.error("❌ Failed to list users:", e.message);
    }
  } catch (error: any) {
    console.error("\n❌ Error:", error.message || error);
    if (error.response?.data) {
      console.error("Response data:", error.response.data);
    }
  } finally {
    rl.close();
  }
}

main();
