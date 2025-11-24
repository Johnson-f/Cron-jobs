# Turso Database Setup Guide

This guide walks you through setting up Turso databases for the Cron Jobs application.

## Prerequisites

1. **Install Turso CLI**: Follow the official installation guide at https://docs.turso.tech/cli/installation
2. **Verify installation**:
   ```bash
   turso --version
   ```

## Step 1: Authenticate with Turso

```bash
turso auth login
```

This will open your browser to authenticate with Turso. Once authenticated, you're ready to proceed.

## Step 2: Select Your Organization

List available organizations:
```bash
turso org list
```

The output shows:
- **NAME**: Display name
- **SLUG**: Organization identifier (use this for `TURSO_ORG`)

Switch to your desired organization:
```bash
turso org switch <slug>
```

**Example**: If you want to use the `cron-jobs` organization:
```bash
turso org switch cron-jobs
```

## Step 3: Create Registry Database

The registry database stores user database mappings and metadata.

```bash
# Create the registry database
turso db create registry

# Verify it was created
turso db list
```

## Step 4: Get Registry Database Credentials

### Get Database URL

```bash
turso db show registry --url
```

**Output example**: `libsql://registry-cron-jobs.aws-eu-west-1.turso.io`

This is your `REGISTRY_DB_URL`.

### Get Database Authentication Token

```bash
turso db tokens create registry
```

**Output**: A JWT token (long string starting with `eyJ...`)

This is your `REGISTRY_DB_TOKEN`.

## Step 5: Create API Token

The API token is used to create and manage databases via the Turso API.

```bash
turso auth api-tokens mint <token-name>
```

**Example**:
```bash
turso auth api-tokens mint cron-job
```

**Output**: A JWT token (long string starting with `eyJ...`)

This is your `TURSO_API_TOKEN`.

**Note**: The token name is just a label for your reference; it doesn't affect functionality.

## Step 6: Get Organization Name

You already know this from Step 2, but to confirm:

```bash
turso org list
```

Use the **SLUG** value from the organization you're using. This is your `TURSO_ORG`.

## Complete Environment Variables

After completing all steps, your `.env` file should contain:

```bash
# Turso Configuration
TURSO_ORG=cron-jobs
REGISTRY_DB_URL=libsql://registry-cron-jobs.aws-eu-west-1.turso.io
REGISTRY_DB_TOKEN=eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...
TURSO_API_TOKEN=eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...
```

## Quick Reference: All Commands

```bash
# 1. Authenticate
turso auth login

# 2. List and switch organizations
turso org list
turso org switch <slug>

# 3. Create registry database
turso db create registry

# 4. Get registry database URL
turso db show registry --url

# 5. Get registry database token
turso db tokens create registry

# 6. Create API token
turso auth api-tokens mint <token-name>
```

## What Each Variable Does

- **`TURSO_ORG`**: Your Turso organization identifier. Used in API calls to create databases.
- **`REGISTRY_DB_URL`**: Connection URL for the registry database (stores user database mappings).
- **`REGISTRY_DB_TOKEN`**: Authentication token for the registry database.
- **`TURSO_API_TOKEN`**: API token with permissions to create and manage databases in your organization.

## Troubleshooting

### "Database already exists" error
If you see an error when creating the registry database, it means it already exists. That's fine! Just proceed to get the URL and token.

### "Unknown command" errors
Make sure you have the latest Turso CLI version:
```bash
turso update
```

### Token expiration
Database tokens can be created with expiration:
```bash
turso db tokens create registry --expiration 1y
```

API tokens created via `turso auth api-tokens mint` don't expire by default.

## Next Steps

After setting up Turso environment variables, you'll also need to configure Supabase variables. See `src/server/ENV_SETUP.md` for complete environment setup.

## Architecture Overview

This application uses a **multi-tenant database architecture**:

1. **Registry Database**: A single database that stores metadata about all user databases
   - Tracks which user owns which database
   - Stores database URLs and tokens
   - Manages user database lifecycle

2. **User Databases**: Each user gets their own isolated Turso database
   - Created on-demand when users sign up
   - Contains user-specific cron job data
   - Isolated from other users' data

The `TURSO_API_TOKEN` is used to create new user databases via the Turso API when users register.

