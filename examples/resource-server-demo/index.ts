import { AuthServerClient } from "auth-server-sdk";
import dotenv from "dotenv";
import readline from "readline";

dotenv.config();

// Define response interfaces
interface RoleResponse {
  id: string;
  name: string;
  description?: string;
}

interface PermissionResponse {
  id: string;
  code: string;
  description?: string;
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
  console.log("╔════════════════════════════════════════════════════════════╗");
  console.log("║       Auth Server - App Self-Management Demo               ║");
  console.log("╚════════════════════════════════════════════════════════════╝");
  console.log("\nThis demo shows the App API endpoints available after");
  console.log("authenticating with /apps/auth (prefix: /app-api/apps):\n");
  console.log("┌──────────────────────────────────────────────────────────┐");
  console.log("│ Endpoint                              │ Method │ Action │");
  console.log("├──────────────────────────────────────────────────────────┤");
  console.log("│ /app-api/apps/:id/roles               │ POST   │ Create │");
  console.log("│ /app-api/apps/:id/roles               │ GET    │ List   │");
  console.log("│ /app-api/apps/:id/permissions         │ POST   │ Create │");
  console.log("│ /app-api/apps/:id/permissions         │ GET    │ List   │");
  console.log("│ /app-api/apps/:id/roles/:role_id/perm │ POST   │ Assign │");
  console.log("└──────────────────────────────────────────────────────────┘");

  const baseUrl = process.env.AUTH_SERVER_URL || "http://localhost:3000";
  const authClient = new AuthServerClient({ baseUrl });

  try {
    // =====================================================================
    // Step 1: App Authentication
    // =====================================================================
    console.log("\n" + "=".repeat(60));
    console.log("STEP 1: App Authentication (/apps/auth)");
    console.log("=".repeat(60));
    console.log(
      "\nPlease enter your App Credentials (create one at http://localhost:5173/apps):"
    );
    const appId = process.env.APP_ID || (await question("App ID (UUID): "));
    const appSecret =
      process.env.APP_SECRET || (await question("App Secret: "));

    console.log(`\n🔐 Authenticating as App ${appId}...`);
    const tokenResponse = await authClient.authenticateApp({
      app_id: appId,
      secret: appSecret,
    });

    console.log("✅ Authentication successful!");
    console.log(
      "   Access Token:",
      tokenResponse.access_token.substring(0, 30) + "..."
    );
    console.log("   Token Type:", tokenResponse.token_type);

    // =====================================================================
    // Step 2: Create a Role
    // =====================================================================
    console.log("\n" + "=".repeat(60));
    console.log("STEP 2: Create Role (POST /app-api/apps/:id/roles)");
    console.log("=".repeat(60));

    const roleName = await question(
      '\nEnter a name for a new role (e.g., "admin", "editor"): '
    );

    let createdRole: RoleResponse | null = null;
    try {
      createdRole = await authClient.appCreateRole(appId, {
        name: roleName,
      });
      console.log("✅ Role created successfully:");
      console.log("   ID:", createdRole.id);
      console.log("   Name:", createdRole.name);
    } catch (e: any) {
      console.error("❌ Failed to create role:", e.message);
      console.log("   (Role might already exist)");
    }

    // =====================================================================
    // Step 3: List All Roles
    // =====================================================================
    console.log("\n" + "=".repeat(60));
    console.log("STEP 3: List Roles (GET /app-api/apps/:id/roles)");
    console.log("=".repeat(60));

    try {
      const roles: RoleResponse[] = await authClient.appListRoles(appId);
      console.log(`\n✅ Found ${roles.length} role(s):`);
      roles.forEach((role, index) => {
        console.log(`   ${index + 1}. ${role.name} (ID: ${role.id})`);
      });

      // Find the created role for later steps
      if (!createdRole && roles.length > 0) {
        createdRole = roles.find((r) => r.name === roleName) || roles[0];
      }
    } catch (e: any) {
      console.error("❌ Failed to list roles:", e.message);
    }

    // =====================================================================
    // Step 4: Create a Permission
    // =====================================================================
    console.log("\n" + "=".repeat(60));
    console.log(
      "STEP 4: Create Permission (POST /app-api/apps/:id/permissions)"
    );
    console.log("=".repeat(60));

    const permCode = await question(
      '\nEnter a permission code (e.g., "posts:read", "users:write"): '
    );

    let createdPermission: PermissionResponse | null = null;
    try {
      createdPermission = await authClient.appCreatePermission(appId, {
        code: permCode,
      });
      console.log("✅ Permission created successfully:");
      console.log("   ID:", createdPermission.id);
      console.log("   Code:", createdPermission.code);
    } catch (e: any) {
      console.error("❌ Failed to create permission:", e.message);
      console.log("   (Permission might already exist)");
    }

    // =====================================================================
    // Step 5: List All Permissions
    // =====================================================================
    console.log("\n" + "=".repeat(60));
    console.log("STEP 5: List Permissions (GET /app-api/apps/:id/permissions)");
    console.log("=".repeat(60));

    try {
      const permissions: PermissionResponse[] =
        await authClient.appListPermissions(appId);
      console.log(`\n✅ Found ${permissions.length} permission(s):`);
      permissions.forEach((perm, index) => {
        console.log(`   ${index + 1}. ${perm.code} (ID: ${perm.id})`);
      });

      // Find the created permission for later steps
      if (!createdPermission && permissions.length > 0) {
        createdPermission =
          permissions.find((p) => p.code === permCode) || permissions[0];
      }
    } catch (e: any) {
      console.error("❌ Failed to list permissions:", e.message);
    }

    // =====================================================================
    // Step 6: Assign Permission to Role
    // =====================================================================
    console.log("\n" + "=".repeat(60));
    console.log(
      "STEP 6: Assign Permission to Role (POST /app-api/apps/:id/roles/:role_id/permissions)"
    );
    console.log("=".repeat(60));

    if (createdRole && createdPermission) {
      const confirm = await question(
        `\nAssign permission "${createdPermission.code}" to role "${createdRole.name}"? (y/n): `
      );

      if (confirm.toLowerCase() === "y") {
        try {
          await authClient.appAssignPermissionToRole(appId, createdRole.id, {
            permission_id: createdPermission.id,
          });
          console.log("✅ Permission assigned to role successfully!");
          console.log(`   Role: ${createdRole.name}`);
          console.log(`   Permission: ${createdPermission.code}`);
        } catch (e: any) {
          console.error("❌ Failed to assign permission to role:", e.message);
        }
      } else {
        console.log("   Skipped permission assignment.");
      }
    } else {
      console.log(
        "\n⚠️ Cannot assign permission - missing role or permission."
      );
      if (!createdRole) console.log("   No role available.");
      if (!createdPermission) console.log("   No permission available.");
    }

    // =====================================================================
    // Summary
    // =====================================================================
    console.log("\n" + "=".repeat(60));
    console.log("DEMO COMPLETE");
    console.log("=".repeat(60));
    console.log("\nSummary of App API endpoints demonstrated:");
    console.log("  ✓ POST /apps/auth           - App authentication");
    console.log("  ✓ POST /app-api/apps/:id/roles       - Create role");
    console.log("  ✓ GET  /app-api/apps/:id/roles       - List roles");
    console.log("  ✓ POST /app-api/apps/:id/permissions - Create permission");
    console.log("  ✓ GET  /app-api/apps/:id/permissions - List permissions");
    console.log(
      "  ✓ POST /app-api/apps/:id/roles/:role_id/permissions - Assign permission"
    );
  } catch (error: any) {
    console.error("\n❌ Error:", error.message || error);
    if (error.response?.data) {
      console.error("Response data:", error.response.data);
    }
  } finally {
    rl.close();
    console.log("\n👋 Goodbye!\n");
  }
}

main();
