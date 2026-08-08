CREATE TABLE tournament_members (
    tournament_id UUID NOT NULL REFERENCES tournaments(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('owner', 'editor', 'viewer')),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (tournament_id, user_id)
);

CREATE UNIQUE INDEX tournament_members_single_owner_index
    ON tournament_members(tournament_id)
    WHERE role = 'owner';

CREATE INDEX tournament_members_user_id_index
    ON tournament_members(user_id, tournament_id);

INSERT INTO tournament_members (
    tournament_id, user_id, role, created_at, updated_at
)
SELECT id, created_by_user_id, 'owner', created_at, updated_at
FROM tournaments;

CREATE TABLE tournament_invitations (
    id UUID PRIMARY KEY,
    tournament_id UUID NOT NULL REFERENCES tournaments(id) ON DELETE CASCADE,
    invited_email TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('editor', 'viewer')),
    invited_by_user_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT tournament_invitations_email_normalized CHECK (
        invited_email = LOWER(invited_email)
    ),
    CONSTRAINT tournament_invitations_tournament_email_unique
        UNIQUE (tournament_id, invited_email)
);

CREATE INDEX tournament_invitations_email_index
    ON tournament_invitations(invited_email, tournament_id);

CREATE INDEX users_normalized_email_index ON users(LOWER(email));
