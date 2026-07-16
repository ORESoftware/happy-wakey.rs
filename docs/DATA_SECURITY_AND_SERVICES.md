# Data, Security, and Services

## Server Philosophy

The app is designed to avoid a custom always-on application server.

Direct desktop-to-provider calls handle calendar, weather, market, and news data. Supabase is the optional managed backend for OAuth brokering and user-scoped synchronization. There is no custom business-logic server in the current architecture.

This is viable because the app does not need to share a central operational database. It stores user preferences as JSON and keeps provider data transient in memory.

## Local Configuration

The local JSON schema includes:

- user ID and Supabase session;
- calendar provider metadata;
- up to five weather locations;
- up to twenty stock symbols;
- up to twenty news keywords;
- up to fifty browser bookmarks;
- Git repository/path setting;
- Supabase sync toggle;
- onboarding state.

Configuration is sanitized before save. The file is written to a temporary sibling, flushed, synchronized, and renamed into place. On Unix, mode `0600` restricts access to the current user.

### Important Secret Limitation

The local JSON currently contains Supabase and provider session tokens. Restrictive file permissions are useful but are not the final design. Production builds should store secrets in:

- macOS Keychain;
- Windows Credential Manager;
- Linux Secret Service/libsecret.

The JSON file should retain only non-secret provider/account identifiers and keychain lookup references.

## Remote Configuration

Before config is exposed to QML or sent to Supabase, `sync_safe_config` removes the Supabase session and clears calendar-provider access/refresh tokens.

Supabase has two tables:

### `public.user_config`

- `user_id uuid primary key`;
- `config jsonb`;
- `updated_at timestamptz`.

### `public.user_onboarding_state`

- `user_id uuid primary key`;
- `completed boolean`;
- constrained `current_step`;
- constrained `step_index` from 0 to 4;
- `updated_at timestamptz`.

The schema is declarative and idempotent. It does not use a migration-history table. Running `supabase_setup.sql` repeatedly converges the known tables, columns, constraints, policies, grants, and triggers toward the desired state.

## Row-Level Security

Both tables:

- enable RLS;
- force RLS;
- revoke anonymous access;
- grant only select/insert/update to authenticated users;
- compare `auth.uid()` to the row's `user_id` for read and write policies.

The desktop obtains the authenticated user ID from `/auth/v1/user` and includes that ID in REST upserts. RLS is still the security boundary; the client-provided ID is not trusted by itself.

The live project schema was not re-verified during the latest pass because a Supabase anon key/database connector was unavailable. The checked-in SQL and REST behavior were inspected and compiled.

## Current Sync Semantics

- Local JSON is always available, including before login.
- On startup after login, onboarding state is read from Supabase and merged with local state.
- Completion wins over incomplete state.
- Otherwise the newer valid timestamp wins.
- Config saves push a redacted config snapshot to Supabase.
- Full remote config hydration is implemented as a service function but is not connected to startup yet.
- The Git destination is stored, but no automatic Git operation is implemented.

Before enabling two-way config hydration, define per-field merge behavior. A whole-document last-write-wins policy could silently remove a bookmark or watchlist update made on another machine.

## OAuth

The login flow uses Authorization Code with PKCE:

1. Generate a random verifier and SHA-256 challenge.
2. Generate a random state nonce.
3. Bind a loopback listener on port 47217 by default.
4. Open Supabase `/auth/v1/authorize` in the system browser.
5. Validate the loopback callback path and state.
6. Exchange the authorization code with the verifier.
7. Persist the Supabase session and provider token.

Provider aliases are normalized. `microsoft` maps to Supabase's `azure` provider.

Calendar APIs use the provider's token, not the Supabase JWT.

## External Service Matrix

| Feature | Provider | Authentication | Current request pattern | Notes |
| --- | --- | --- | --- | --- |
| Google calendar | Google Calendar API | Google provider OAuth token | Current week, primary calendar | Requests read-only scope |
| Microsoft calendar | Microsoft Graph | Microsoft provider OAuth token | Current week, calendar view | Requests `Calendars.Read` and UTC response timezone |
| Apple identity | Supabase Apple provider | Apple OAuth | Login only | Apple Sign-In does not expose calendar events |
| Weather | Open-Meteo | None for eligible free use; key for paid customer API | One request per location, up to five in parallel | Current conditions + five days |
| Weather fallback | OpenWeather | API key in query per provider contract | Used only after Open-Meteo failure | Current conditions only |
| Stocks | Finnhub | API token | One quote call per symbol, up to twenty | Profile call removed to halve request count |
| News | NewsAPI | `X-Api-Key` header | 25 candidates, five retained | Local keyword match and dedupe |
| Radar | Windy web map | None | External HTTPS URL | Opens interactive map centered on coordinates |
| Auth/config | Supabase | anon key + user access token | Auth and PostgREST | RLS protects per-user rows |
| Gmail invitations | Gmail API | User OAuth, minimum required Gmail scope | Incremental polling on installed clients | Optional enrichment for invites not yet in Calendar |
| Calendly | Calendly API v2 | Native OAuth 2.1 with PKCE | Incremental polling without server | Webhooks require public receiver and eligible plan |
| Apple local calendar | EventKit on macOS | OS calendar permission | Local event-store query | Apple Sign-In is unrelated to calendar access |

## Open-Meteo Licensing

Open-Meteo data requires attribution. The app includes visible attribution and a link. The public free endpoint has non-commercial usage conditions and rate limits. A commercial Happy Wakey release should use an appropriate Open-Meteo paid customer endpoint and API key, or select another provider with compatible commercial terms.

Environment settings:

- `OPEN_METEO_BASE_URL`;
- `OPEN_METEO_API_KEY`;
- `OPENWEATHER_API_KEY` for fallback.

## HTTP Security and Reliability

External GET calls share one bounded client. Important safeguards include:

- TLS through Rustls;
- explicit connect/request timeouts;
- response-size limits;
- limited redirects;
- transient-only retries;
- URL construction through the `url` crate;
- API key headers where supported;
- HTTP/HTTPS validation for user URLs and news results;
- bounded errors that avoid echoing request URLs containing secrets.

Remaining work:

- cache freshness and stale-while-revalidate behavior;
- provider-specific rate-limit budgets;
- circuit breaking after repeated outages;
- token refresh before calendar calls;
- certificate pinning decision (normally not recommended without a rotation plan);
- privacy review for transmitting favorite coordinates and keywords.

## Google Cloud Service Account Decision

A GCP service account is not the default credential for personal Google Calendar or Gmail accounts. Personal accounts should use the user's installed-app OAuth consent and token.

A service account is appropriate in two narrower cases:

1. A Google Workspace administrator explicitly grants domain-wide delegation so a managed enterprise deployment can impersonate users in that domain.
2. A small backend relay runs on Google Cloud and needs infrastructure access such as Pub/Sub. In that case, use the runtime's attached identity/workload credentials instead of downloading a long-lived JSON key.

Never embed a service-account private key in the desktop executable, `.env`, config JSON, or Git backup.

## Git Backup Design

The intended backup artifact is a redacted JSON document, not the local secret-bearing config file.

A safe implementation should:

1. Validate that the configured path is an allowed local repository or explicitly clone a user-provided private remote.
2. Write `happy-wakey.json` without tokens.
3. Acquire a repository lock.
4. Fetch and rebase/merge remote changes semantically.
5. Merge collections by stable IDs rather than replacing the whole document.
6. Commit only when content changed.
7. Push with clear authentication and conflict errors.
8. Never store Git credentials in the JSON file.
9. Keep a local recovery copy before conflict resolution.

Git hosting tokens or SSH keys should remain in the user's existing credential manager/SSH agent.
