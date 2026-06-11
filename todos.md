# Supabase OAuth Provider Setup

In the Supabase Dashboard (Authentication → Providers), enable each provider and fill in the Client ID and Client Secret obtained from their developer consoles:

## Google
1. Go to [Google Cloud Console](https://console.cloud.google.com), create/select a project
2. APIs & Services → Credentials → Create Credentials → OAuth client ID (Web application)
3. Add `https://vgzyyfhnendriyrhakkp.supabase.co/auth/v1/callback` as an Authorized Redirect URI
4. Copy Client ID + Secret → paste into Supabase provider settings

## Apple
1. Go to [Apple Developer](https://developer.apple.com/account/resources/services) → Certificates, Identifiers & Profiles → Identifier → Register a Service ID
2. Check "Sign in with Apple", configure the Return URL: `https://vgzyyfhnendriyrhakkp.supabase.co/auth/v1/callback`
3. Create a key for that Service ID (Keys → Register) to get the private key
4. Fill in Supabase: Team ID (from your membership page), Key ID (from the key), Service ID, and the private key

## Microsoft (Azure AD)
1. Go to [Azure Portal](https://portal.azure.com) → App registrations → New registration
2. Set redirect URI to: `https://vgzyyfhnendriyrhakkp.supabase.co/auth/v1/callback`
3. Note the Application (client) ID and Directory (tenant) ID
4. Create a client secret (Certificates & secrets → New client secret)
5. Fill in Supabase: Client ID, Client Secret, Tenant ID

After saving each provider in Supabase, you can call `backend.login("google")` / `"apple"` / `"microsoft"` from the QML UI and the PKCE flow will work.

## Loopback redirect (required for desktop sign-in)

The app completes the PKCE flow on a **fixed** local port (default `47217`, overridable
with the `HAPPY_WAKEY_OAUTH_PORT` env var). In the Supabase Dashboard
(Authentication → URL Configuration → Redirect URLs) add:

```
http://127.0.0.1:47217/callback
```

A fixed port is required because a random/ephemeral port can never be allow-listed.

## Calendar access (provider scopes)

Calendar reads use the **provider's** OAuth token (`provider_token`), not the Supabase
JWT, so the provider must grant calendar scopes at sign-in. The app already requests:

- **Google:** `https://www.googleapis.com/auth/calendar.readonly` — enable the Google
  Calendar API for the project and add this scope on the OAuth consent screen.
- **Microsoft/Azure:** `Calendars.Read` (+ `offline_access`) — add these delegated
  permissions to the Azure app registration.
- **Apple:** no calendar API; calendar refresh is disabled for Apple sign-in.

If calendar refresh reports that access wasn't granted, sign out and back in so the
provider re-issues a token that includes these scopes.
