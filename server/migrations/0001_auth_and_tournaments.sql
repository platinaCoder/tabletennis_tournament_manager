CREATE TABLE users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL,
    display_name TEXT,
    avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    last_login_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE auth_identities (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_subject TEXT NOT NULL,
    provider_email TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT auth_identities_provider_subject_unique
        UNIQUE (provider, provider_subject)
);

CREATE TABLE auth_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash BYTEA NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL,
    last_seen_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX auth_sessions_user_id_index ON auth_sessions(user_id);
CREATE INDEX auth_sessions_expires_at_index ON auth_sessions(expires_at);

CREATE TABLE oauth_login_attempts (
    id UUID PRIMARY KEY,
    state_hash BYTEA NOT NULL UNIQUE,
    pkce_verifier TEXT NOT NULL,
    oidc_nonce TEXT NOT NULL,
    return_to TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ
);

CREATE INDEX oauth_login_attempts_expires_at_index
    ON oauth_login_attempts(expires_at);

CREATE TABLE tournaments (
    id UUID PRIMARY KEY,
    created_by_user_id UUID NOT NULL REFERENCES users(id),
    domain_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('draft', 'started')),
    match_format TEXT NOT NULL CHECK (
        match_format IN ('best_of_three', 'best_of_five')
    ),
    table_count INTEGER NOT NULL CHECK (table_count > 0),
    maximum_round_count INTEGER NOT NULL CHECK (maximum_round_count > 0),
    active_pairing_policy_version TEXT NOT NULL,
    active_scoring_policy_version TEXT NOT NULL,
    revision BIGINT NOT NULL DEFAULT 0 CHECK (revision >= 0),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT tournaments_creator_domain_id_unique
        UNIQUE (created_by_user_id, domain_id)
);

CREATE INDEX tournaments_created_by_user_id_index
    ON tournaments(created_by_user_id);

CREATE TABLE entrants (
    tournament_id UUID NOT NULL REFERENCES tournaments(id) ON DELETE CASCADE,
    entrant_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    club_id TEXT NOT NULL,
    club_name TEXT NOT NULL,
    starting_elo INTEGER NOT NULL CHECK (starting_elo >= 0),
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (tournament_id, entrant_id)
);

CREATE TABLE rounds (
    id UUID PRIMARY KEY,
    tournament_id UUID NOT NULL REFERENCES tournaments(id) ON DELETE CASCADE,
    round_number INTEGER NOT NULL CHECK (round_number > 0),
    status TEXT NOT NULL CHECK (status IN ('preview', 'active', 'completed')),
    pairing_policy_version TEXT NOT NULL,
    relaxation_tier TEXT,
    pairing_snapshot JSONB NOT NULL,
    pairing_proposal JSONB NOT NULL,
    bye_entrant_id TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    UNIQUE (tournament_id, round_number),
    UNIQUE (id, tournament_id),
    FOREIGN KEY (tournament_id, bye_entrant_id)
        REFERENCES entrants(tournament_id, entrant_id)
);

CREATE TABLE matches (
    id UUID PRIMARY KEY,
    tournament_id UUID NOT NULL REFERENCES tournaments(id) ON DELETE CASCADE,
    round_id UUID NOT NULL,
    match_id TEXT NOT NULL,
    home_entrant_id TEXT NOT NULL,
    away_entrant_id TEXT NOT NULL,
    table_number INTEGER CHECK (table_number > 0),
    publication_status TEXT NOT NULL CHECK (
        publication_status IN ('draft', 'published')
    ),
    round_activity TEXT NOT NULL CHECK (
        round_activity IN ('active', 'inactive')
    ),
    winner_entrant_id TEXT,
    home_games_won INTEGER,
    away_games_won INTEGER,
    revision BIGINT NOT NULL DEFAULT 0 CHECK (revision >= 0),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    UNIQUE (tournament_id, match_id),
    UNIQUE (id, tournament_id),
    FOREIGN KEY (round_id, tournament_id)
        REFERENCES rounds(id, tournament_id) ON DELETE CASCADE,
    FOREIGN KEY (tournament_id, home_entrant_id)
        REFERENCES entrants(tournament_id, entrant_id),
    FOREIGN KEY (tournament_id, away_entrant_id)
        REFERENCES entrants(tournament_id, entrant_id),
    FOREIGN KEY (tournament_id, winner_entrant_id)
        REFERENCES entrants(tournament_id, entrant_id),
    CHECK (home_entrant_id <> away_entrant_id),
    CHECK (
        winner_entrant_id IS NULL
        OR winner_entrant_id = home_entrant_id
        OR winner_entrant_id = away_entrant_id
    )
);

CREATE INDEX matches_round_id_index ON matches(round_id);

CREATE TABLE match_result_revisions (
    match_id UUID NOT NULL,
    tournament_id UUID NOT NULL,
    revision BIGINT NOT NULL CHECK (revision > 0),
    winner_entrant_id TEXT NOT NULL,
    home_games_won INTEGER NOT NULL CHECK (home_games_won >= 0),
    away_games_won INTEGER NOT NULL CHECK (away_games_won >= 0),
    entered_at TIMESTAMPTZ NOT NULL,
    corrected_at TIMESTAMPTZ,
    correction_reason TEXT,
    PRIMARY KEY (match_id, revision),
    FOREIGN KEY (match_id, tournament_id)
        REFERENCES matches(id, tournament_id) ON DELETE CASCADE,
    FOREIGN KEY (tournament_id, winner_entrant_id)
        REFERENCES entrants(tournament_id, entrant_id)
);

CREATE TABLE game_scores (
    match_id UUID NOT NULL,
    result_revision BIGINT NOT NULL,
    game_number INTEGER NOT NULL CHECK (game_number > 0),
    home_points INTEGER NOT NULL CHECK (home_points >= 0),
    away_points INTEGER NOT NULL CHECK (away_points >= 0),
    PRIMARY KEY (match_id, result_revision, game_number),
    FOREIGN KEY (match_id, result_revision)
        REFERENCES match_result_revisions(match_id, revision) ON DELETE CASCADE
);
