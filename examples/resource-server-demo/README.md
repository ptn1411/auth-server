# Resource Server Demo

This example demonstrates how to use the Auth Server SDK to act as a Resource Server (an App).
It authenticates using `App ID` and `App Secret` and then manages resources (Roles, Permissions).

## Prerequisites

1.  **Auth Server** running at `http://localhost:3000`.
2.  **Frontend** running at `http://localhost:5173`.
3.  An **App** created in the Frontend ("My Apps").

## Setup

1.  Install dependencies:

    ```bash
    npm install
    ```

2.  Build the SDK (if not done):
    ```bash
    cd ../../sdk
    npm install
    npm run build
    cd ../examples/resource-server-demo
    ```

## Running the Demo

1.  Start the script:

    ```bash
    npm start
    ```

2.  Follow the prompts:
    - Enter the **App ID** (UUID) from the App Details page.
    - Enter the **App Secret** (copied from the Frontend creation dialog).
    - Follow the interactive steps to create roles and permissions.

## Environment Variables

You can skip prompts by setting `.env` file:

```env
AUTH_SERVER_URL=http://localhost:3000
APP_CODE=22ssssss_1767146578771
APP_SECRET="s1z&O-Gf+Ojt#+ssaZE9t*VRQDkp#UX0J71E539SNyEYVZu&"
APP_ID=74f56fd5-b74d-4434-b47f-b17a077f575c
```
