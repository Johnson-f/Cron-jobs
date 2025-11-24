# Server Environment Variables

This document lists all the environment variables required for the server to run properly.

## Required Environment Variables

Create a `.env` file in the project root directory with the following variables:

### Turso Database Configuration

```bash
# Registry database URL - The main database that stores user database registry
REGISTRY_DB_URL=libsql://your-registry-db.turso.io

# Registry database authentication token
REGISTRY_DB_TOKEN=your-registry-db-token-here

# Turso API token for creating/managing databases
TURSO_API_TOKEN=your-turso-api-token-here

# Turso organization name
TURSO_ORG=your-turso-org-name
```

### Supabase Configuration

```bash
# Your Supabase project URL (e.g., https://xxxxx.supabase.co)
VITE_SUPABASE_URL=https://your-project.supabase.co

# Supabase anonymous/public key (safe to expose to client)
VITE_SUPABASE_ANON_KEY=your-supabase-anon-key-here

# Supabase service role key (KEEP SECRET - server-side only)
# This key has admin privileges and should never be exposed to the client
SUPABASE_SERVICE_ROLE_KEY=your-supabase-service-role-key-here
```

## How to Get These Values

### Turso Values (Using Turso CLI)

First, make sure you're logged in:
```bash
turso auth login
```

#### 1. Get TURSO_ORG (Organization Name)

```bash
# List your organizations
turso org list

# The organization marked with "(current)" is your active organization
# Use the SLUG column value for TURSO_ORG
```

Use the **SLUG** column value (not the NAME) for `TURSO_ORG`. The organization marked with "(current)" is your active organization.

#### 2. Create Registry Database and Get REGISTRY_DB_URL & REGISTRY_DB_TOKEN

Create a database for the registry (this stores user database mappings):

```bash
# Create the registry database (replace 'registry' with your preferred name)
turso db create registry

# Get the database URL
turso db show registry --url

# Get the authentication token for the database
turso db tokens create registry
```

**Note**: The `--url` flag gives you the full connection string. For `REGISTRY_DB_URL`, use the format: `libsql://registry-{org}.turso.io`

Alternatively, you can get both at once:
```bash
# Get connection string (includes URL and token)
turso db shell registry --url
```

#### 3. Get TURSO_API_TOKEN

```bash
# Create a new API token
turso auth api-tokens mint <token-name>
```

**Example**:
```bash
turso auth api-tokens mint cron-job
```

The token created will have the necessary permissions to create and manage databases. Copy this token for `TURSO_API_TOKEN`.

**Note**: The token name is just a label for your reference; it doesn't affect functionality.

#### Quick Setup Script

You can also run these commands to quickly get all values:

```bash
# Get organization
echo "TURSO_ORG=$(turso org current)"

# Create registry database if it doesn't exist (will error if exists, that's ok)
turso db create registry 2>/dev/null || true

# Get registry database URL
echo "REGISTRY_DB_URL=$(turso db show registry --url)"

# Create and get registry database token
echo "REGISTRY_DB_TOKEN=$(turso db tokens create registry --expiration 1y)"

# Create API token
echo "TURSO_API_TOKEN=$(turso auth api-tokens mint cron-job)"
```

**Note**: The registry database token creation command will output just the token. You may need to parse the output or use `turso db tokens create registry --expiration 1y | tail -n 1` to get just the token value.

### Supabase Values

1. **VITE_SUPABASE_URL**:
   - Found in Supabase dashboard → Settings → API → Project URL

2. **VITE_SUPABASE_ANON_KEY**:
   - Found in Supabase dashboard → Settings → API → Project API keys → `anon` `public` key

3. **SUPABASE_SERVICE_ROLE_KEY**:
   - Found in Supabase dashboard → Settings → API → Project API keys → `service_role` `secret` key
   - ⚠️ **WARNING**: This key has admin privileges. Never expose it to the client!

## Environment Loading

The server automatically loads environment variables from a `.env` file in the project root using `dotenv::dotenv().ok()` in `main.rs`. Make sure your `.env` file is in the root directory of the project.

## Security Notes

- Never commit your `.env` file to version control
- Add `.env` to your `.gitignore` file
- The `SUPABASE_SERVICE_ROLE_KEY` should only be used server-side
- The `VITE_SUPABASE_URL` and `VITE_SUPABASE_ANON_KEY` are safe to expose to the client (they're prefixed with `VITE_`)

