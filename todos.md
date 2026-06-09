# Supabase OAuth Provider Setup

In the Supabase Dashboard (Authentication → Providers), enable each provider and fill in the Client ID and Client Secret obtained from their developer consoles:

## Google
1. Go to [Google Cloud Console](https://console.cloud.google.com), create/select a project
2. APIs & Services → Credentials → Create Credentials → OAuth client ID (Web application)
3. Add `https://gtbeuxcolbpuipvqiibn.supabase.co/auth/v1/callback` as an Authorized Redirect URI
4. Copy Client ID + Secret → paste into Supabase provider settings

## Apple
1. Go to [Apple Developer](https://developer.apple.com/account/resources/services) → Certificates, Identifiers & Profiles → Identifier → Register a Service ID
2. Check "Sign in with Apple", configure the Return URL: `https://gtbeuxcolbpuipvqiibn.supabase.co/auth/v1/callback`
3. Create a key for that Service ID (Keys → Register) to get the private key
4. Fill in Supabase: Team ID (from your membership page), Key ID (from the key), Service ID, and the private key

## Microsoft (Azure AD)
1. Go to [Azure Portal](https://portal.azure.com) → App registrations → New registration
2. Set redirect URI to: `https://gtbeuxcolbpuipvqiibn.supabase.co/auth/v1/callback`
3. Note the Application (client) ID and Directory (tenant) ID
4. Create a client secret (Certificates & secrets → New client secret)
5. Fill in Supabase: Client ID, Client Secret, Tenant ID

After saving each provider in Supabase, you can call `backend.login("google")` / `"apple"` / `"microsoft"` from the QML UI and the PKCE flow will work.
