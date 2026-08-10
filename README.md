# Table-tennis tournament manager

The application consists of a Yew/WASM browser interface and one Axum API on
Vercel's official Rust runtime. The API is authoritative: browsers never
connect to PostgreSQL and never receive Google or database credentials.

## Architecture

```text
Yew/WASM -> /api/* -> Axum/Vercel Function -> Neon PostgreSQL
                         |
                         +-> Google OpenID Connect
```

The existing domain and pairing modules remain shared by the GUI, simulator and
server. `BlossomV2` is the active policy. `BlossomV1` remains available for
regression comparisons. SQL row types and Google-specific types stay within the
backend boundary under `server/`.

## Required environment variables

Copy `.env.example` to a local untracked environment file or configure the same
names in Vercel:

```text
DATABASE_URL=
GOOGLE_CLIENT_ID=
GOOGLE_CLIENT_SECRET=
APP_BASE_URL=http://localhost:3000
DATABASE_MAX_CONNECTIONS=4
```

`DATABASE_URL` must be the Neon pooled PostgreSQL URL. All values are server
secrets except `APP_BASE_URL`, but none are compiled into the WASM bundle.
`DATABASE_MAX_CONNECTIONS` is optional and defaults to `4` per warm function
instance.

Never commit `.env` files or credentials. If a credential has appeared in a
terminal transcript, issue, chat or commit, rotate it before use.

## Database migrations

Install the SQLx CLI version used by the application:

```bash
cargo install sqlx-cli --version 0.8.6 --locked \
  --no-default-features --features rustls,postgres
```

Apply committed migrations explicitly:

```bash
DATABASE_URL='postgresql://…' sqlx migrate run --source server/migrations
```

Normal request handlers never create or migrate tables. Run migrations before
deploying code that requires them.

## Google OAuth configuration

Create a Google OAuth client of type **Web application**. Configure these exact
authorized redirect URIs:

```text
http://localhost:3000/api/auth/google/callback
https://ttt-manager.vercel.app/api/auth/google/callback
```

Set `APP_BASE_URL` to the matching origin, without an `/api` suffix:

```text
Local:      http://localhost:3000
Production: https://ttt-manager.vercel.app
```

Google requires an exact redirect URI match. Dynamic Vercel Preview domains do
not support wildcard redirect URIs. A preview can use Google login only when
that exact preview callback URL is registered and `APP_BASE_URL` matches it.
This project intentionally has no cross-domain preview authentication relay.

Only `openid`, `email` and `profile` are requested. Provider access and refresh
tokens are not stored.

## Local development

On NixOS, enter the committed development shell before starting Vercel. The
shell exposes Nix-compatible Rust tools, including the `rustup` command checked
by Vercel's local Rust builder:

```bash
nix-shell
npx vercel dev --listen 3000
```

After changing `shell.nix`, leave and re-enter the shell so its updated `PATH`
is active.

Install Trunk, the WASM target and Vercel CLI, then link the project if needed:

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk --version 0.21.14
pnpm install --global vercel
vercel link
```

Provide the environment variables and run the production-like router from the
repository root:

```bash
vercel dev --listen 3000
```

Open `http://localhost:3000/` for normal operation or
`http://localhost:3000/dev` for simulation tooling. The Google localhost
redirect URI above must be registered before local sign-in works.

For frontend-only visual work, `cd gui && trunk serve` still runs the static
interface, but authenticated API actions require `vercel dev`.

## Vercel deployment

Configure `DATABASE_URL`, `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET` and
`APP_BASE_URL` in the Vercel Production environment. Apply migrations, then
deploy through the connected Git repository or:

```bash
vercel deploy --prod
```

`vercel.json` sends `/api/*` to the single Rust/Axum function, keeps the SPA and
`/dev` fallback routes, and places the function in Frankfurt (`fra1`) beside the
EU-central Neon database. No source changes are needed between environments.

## Authentication and authorization

Google login uses Authorization Code flow with PKCE S256, OAuth state and an
OIDC nonce. The ID token is verified by the `openid` OIDC client. Application
sessions are opaque 256-bit random values in an `HttpOnly`, `SameSite=Lax`,
host-only cookie. PostgreSQL stores only a SHA-256 token hash. Sessions expire
after 14 days and logout deletes the server-side session.

Tournament authorization is membership-based and enforced by the application
service rather than the browser or SQL repositories:

```text
owner  -> view, edit, share, change member roles, revoke access, delete
editor -> view and edit, including entering match results
viewer -> view only
```

The creator receives the permanent owner membership. Owner access cannot be
downgraded or removed. Sharing uses the recipient's verified Google email and
always creates a pending invitation. The recipient must explicitly accept or
decline it at the top of the dashboard. Invitations also appear on the first
login of an account that did not exist when it was invited. The application
does not send invitation email in this version.

Deleting a tournament is owner-only, revision-checked and permanent. One
database transaction removes result revisions, game scores, matches, rounds,
entrants, memberships, invitations and the tournament in dependency order.

Tournament mutations retain aggregate optimistic concurrency. Match-result
entry additionally uses each match's revision. Concurrent results for different
matches are retried against fresh authoritative state, while concurrent edits
to the same match return `result_revision_conflict` instead of overwriting data.

Saved results can be corrected from the active round's result-entry card. A
correction resubmits every individual game, requires the current match revision,
and is revalidated by the results domain. PostgreSQL appends the
replacement as a new `match_result_revisions` row with its own revision-keyed
`game_scores`; previous scores are never overwritten or deleted. Winner changes
recalculate standings and invalidate unpublished pairing previews, but never
rewrite already-published pairings. This uses the existing schema and requires
no additional database migration.

The dashboard API routes are:

```text
GET    /api/tournaments
POST   /api/tournaments
GET    /api/tournament-invitations
POST   /api/tournament-invitations/{invitation_id}/accept
POST   /api/tournament-invitations/{invitation_id}/decline
GET    /api/tournaments/{id}
DELETE /api/tournaments/{id}
GET    /api/tournaments/{id}/sharing
POST   /api/tournaments/{id}/sharing
PUT    /api/tournaments/{id}/members/{user_id}
DELETE /api/tournaments/{id}/members/{user_id}
DELETE /api/tournaments/{id}/invitations/{invitation_id}
```

## Verification

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p tabletennis_tournament_gui \
  --target wasm32-unknown-unknown -- -D warnings
(cd gui && trunk build --release)
cargo build --release --bin vercel-api
cargo audit
```
