# AGENTS.md amendment: match and individual-game results

## Result terminology

Use these terms consistently throughout the application:

- `Point`: an individual rally score within a game.
- `Game`: a game played to at least 11 points with a two-point winning margin.
- `Match`: the complete contest between two entrants, consisting of multiple games.
- `MatchResult`: the final match win or loss derived from the recorded games.

Do not use `set` and `game` interchangeably in domain code.

The user interface may include explanatory labels where tournament operators are accustomed to different terminology, but the Rust domain model must consistently use `Game`.

---

# Match format

Each tournament must configure one of the following match formats:

```rust
pub enum MatchFormat {
    BestOfThree,
    BestOfFive,
}
```

The configured format applies to every normal match in the tournament.

Rules:

```text
BestOfThree:
- Maximum of 3 games.
- First contestant to win 2 games wins the match.

BestOfFive:
- Maximum of 5 games.
- First contestant to win 3 games wins the match.
```

The match format must be selected when creating the tournament.

It may be changed while the tournament is in draft state.

It must be frozen when the tournament starts.

Do not initially support changing the match format for individual matches within the same tournament.

The domain design may permit additional formats later, but do not introduce unnecessary generic format abstractions in the MVP.

---

# Individual game scores

Every played game must be registered with the points scored by both contestants.

Use a domain type equivalent to:

```rust
pub struct GameScore {
    pub game_number: GameNumber,
    pub home_points: GamePoints,
    pub away_points: GamePoints,
}
```

A complete match result must contain the ordered game scores:

```rust
pub struct MatchResult {
    pub match_id: MatchId,
    pub games: Vec<GameScore>,
    pub home_games_won: GamesWon,
    pub away_games_won: GamesWon,
    pub winner_id: EntrantId,
    pub entered_at: DateTime<Utc>,
    pub corrected_at: Option<DateTime<Utc>>,
    pub revision: MatchResultRevision,
}
```

The following values must be derived from `games` and must not be independently accepted from the HTTP client:

- Home games won.
- Away games won.
- Match winner.
- Match loser.
- Match completion state.

Do not trust a submitted winner identifier without checking it against the individual game scores.

---

# Game validation

A normal game is valid when:

- One contestant has at least 11 points.
- The winner has at least two points more than the opponent.
- The losing contestant has a non-negative point score.
- Exactly one contestant satisfies the winning condition.

Examples of valid game scores:

```text
11–0
11–8
11–9
12–10
15–13
23–21
```

Examples of invalid game scores:

```text
10–8
11–10
12–11
9–7
11–11
```

Do not impose an arbitrary maximum game score. Games can continue beyond 11–11 until one contestant leads by two points.

Use checked integer types and a sensible storage limit to prevent malformed or abusive input from causing overflow.

---

# Match validation

A match result is valid only when:

- Every recorded game is valid.
- Games are stored in sequential order starting at game one.
- Exactly one contestant has reached the required number of game wins.
- No additional games are recorded after the match winner has already been determined.
- The number of games does not exceed the configured match-format maximum.
- Both entrants belong to the scheduled match.
- The result belongs to a published match.
- The result belongs to the currently active round, unless an explicit correction workflow is used.

Examples:

```text
Best of three:
2–0: valid
2–1: valid
1–0: incomplete
2–2: impossible
2–0 followed by another game: invalid

Best of five:
3–0: valid
3–1: valid
3–2: valid
2–2: incomplete
3–1 followed by another game: invalid
```

An incomplete result may be represented temporarily in the Yew result-entry form, but it must not be persisted as a completed `MatchResult`.

---

# Tournament algorithm input

The tournament scoring and pairing algorithm uses the final match outcome.

For the initial performance-score policy:

```text
win  = actual result 1.0
loss = actual result 0.0
```

The following must not affect the initial performance-score calculation:

- Number of games won.
- Number of games lost.
- Game difference.
- Points scored.
- Points conceded.
- Margin of victory.
- Whether the match ended 2–0, 2–1, 3–0, 3–1 or 3–2.

For example, these results count as the same tournament-algorithm outcome:

```text
11–0, 11–0
```

and:

```text
12–14, 15–13, 18–16
```

Both are one match win for the winning contestant and one match loss for the losing contestant.

Individual game and point results are still mandatory because they are required for complete tournament records and future official NTTB result registration.

A future policy may use game or point differences, but this must require:

- A new performance-score-policy version.
- Explicit documentation.
- New simulation comparisons.
- Updated golden tests.
- No modification of completed historical tournaments.

---

# Results module responsibilities

The results module owns:

- Game-score validation.
- Match-format validation.
- Match-completion detection.
- Match-winner derivation.
- Match-result entry.
- Match-result correction.
- Result revision history.
- Calculation of game totals.
- Calculation of point totals.
- Result-related domain events.

The results module must expose an operation equivalent to:

```rust
pub fn validate_and_complete_match(
    scheduled_match: &ScheduledMatch,
    match_format: MatchFormat,
    submitted_games: Vec<GameScore>,
) -> Result<MatchResult, MatchResultError>;
```

HTTP handlers and Yew components must not:

- Determine the winner.
- Count game wins.
- Decide whether a match is complete.
- Validate deuce scores.
- Decide whether too many games were entered.

Those rules belong exclusively to the results domain.

---

# Result correction

A corrected result replaces the active result but must not delete its history.

Store every accepted revision.

```rust
pub struct MatchResultRevisionRecord {
    pub match_id: MatchId,
    pub revision: MatchResultRevision,
    pub previous_games: Vec<GameScore>,
    pub replacement_games: Vec<GameScore>,
    pub corrected_at: DateTime<Utc>,
    pub correction_reason: String,
}
```

A correction reason is mandatory.

Correcting the individual game scores may or may not change the match winner.

If the match winner changes, recalculate:

- The match outcome.
- Both contestants’ performance-score contributions.
- Both contestants’ aggregate performance scores.
- Standings.
- Opponent-strength tie-break values.
- Any unpublished pairing preview calculated from the old result.

If only the individual point scores change and the match winner remains the same:

- Preserve the tournament performance scores.
- Recalculate game and point statistics.
- Invalidate any unpublished view or export based on the previous detailed result.
- Record the correction revision.

Never silently recalculate published historical pairings.

---

# Standings data

Even though the initial ranking algorithm primarily uses match outcomes and performance score, standings must retain detailed result totals.

Store or derive:

```rust
pub struct ContestantStanding {
    pub entrant_id: EntrantId,
    pub performance_score: PerformanceScore,
    pub matches_played: u32,
    pub matches_won: u32,
    pub matches_lost: u32,
    pub games_won: u32,
    pub games_lost: u32,
    pub game_difference: i32,
    pub points_won: u32,
    pub points_lost: u32,
    pub point_difference: i32,
    pub opponent_score_sum: PerformanceScore,
    pub bye_count: u32,
}
```

Game and point statistics must be available for:

- Tournament administration.
- Result verification.
- Tie-break policies.
- Future exports.
- Future NTTB NAS integration.
- Auditing corrected results.

Do not discard detailed scores merely because the active pairing policy does not consume them.

---

# Persistence requirements

Persist individual games separately from the match summary.

Conceptual table ownership:

```text
results.matches
results.match_results
results.game_scores
results.match_result_revisions
results.game_score_revisions
```

A possible relational representation is:

```text
matches
- match_id
- round_id
- home_entrant_id
- away_entrant_id
- match_format
- status

match_results
- match_id
- revision
- winner_entrant_id
- home_games_won
- away_games_won
- entered_at
- corrected_at

game_scores
- match_id
- result_revision
- game_number
- home_points
- away_points
```

The exact schema may differ, but it must preserve:

- Game order.
- Every point score.
- Match format.
- Active result revision.
- Previous result revisions.
- Derived winner.
- Audit timestamps.

Publishing a result and its individual game scores must be one database transaction.

It must never be possible to persist a completed match summary without its corresponding game scores.

---

# API contract

Submit match results using individual games.

Example request:

```json
{
  "expected_revision": 0,
  "games": [
    {
      "game_number": 1,
      "home_points": 11,
      "away_points": 7
    },
    {
      "game_number": 2,
      "home_points": 8,
      "away_points": 11
    },
    {
      "game_number": 3,
      "home_points": 13,
      "away_points": 11
    }
  ]
}
```

The API response must return the derived result:

```json
{
  "match_id": "match-id",
  "format": "best_of_three",
  "home_games_won": 2,
  "away_games_won": 1,
  "winner_entrant_id": "home-entrant-id",
  "games": [
    {
      "game_number": 1,
      "home_points": 11,
      "away_points": 7
    },
    {
      "game_number": 2,
      "home_points": 8,
      "away_points": 11
    },
    {
      "game_number": 3,
      "home_points": 13,
      "away_points": 11
    }
  ],
  "revision": 1
}
```

Return specific validation error codes such as:

```text
invalid_game_score
game_numbers_not_sequential
match_not_complete
too_many_games
games_recorded_after_match_completion
result_revision_conflict
entrant_not_part_of_match
match_not_published
```

---

# Yew result-entry interface

The result-entry screen must allow fast entry of individual game scores.

For each scheduled match, show:

- Table or match number.
- Both contestant names.
- Both clubs.
- Both ELO ratings.
- Configured match format.
- Input fields for each possible game.
- Current derived game score.
- Current derived winner.
- Save state.
- Correction state where applicable.

For best-of-three matches, show up to three game rows.

For best-of-five matches, show up to five game rows.

The form must dynamically determine when the match is complete.

Once one contestant has won the required number of games:

- Mark the winner visibly.
- Ignore or disable unnecessary later game rows.
- Reject previously entered extra games rather than silently discarding them.
- Enable result submission.

Keyboard workflow must support:

1. Enter home points.
2. Move to away points.
3. Move to the next game.
4. Submit when the match is complete.
5. Move to the next match.

Do not require mouse interaction for normal result entry.

Display the completed match result in a compact form such as:

```text
3–1
11–8, 9–11, 11–6, 11–7
```

---

# Future NTTB NAS export boundary

The future NAS export module must receive complete match details.

Extend the export model:

```rust
pub struct CompletedMatchExport {
    pub match_id: MatchId,
    pub round_number: RoundNumber,
    pub match_format: MatchFormat,
    pub home_entrant: EntrantExportIdentity,
    pub away_entrant: EntrantExportIdentity,
    pub games: Vec<GameScoreExport>,
    pub home_games_won: u32,
    pub away_games_won: u32,
    pub winner_entrant_id: EntrantId,
}
```

Entrant export identities may later include:

```rust
pub struct EntrantExportIdentity {
    pub entrant_id: EntrantId,
    pub name: String,
    pub club_name: String,
    pub starting_elo: EloRating,
    pub nttb_member_id: Option<NttbMemberId>,
    pub nttb_club_id: Option<NttbClubId>,
}
```

The future NAS adapter must serialize the official match details from the stored game scores.

It must not reconstruct game scores from:

- Final match score.
- Performance score.
- Game difference.
- Point difference.
- User-entered free text.

The tournament performance score is internal tournament-ranking information and must remain separate from official NTTB rating-result data.

The application does not calculate or claim the official post-tournament NTTB ELO changes. It records and exports the source match results required by the external NTTB process.

Do not implement NAS-specific field mappings until the official integration format is known.

---

# Pairing algorithm module boundary

Keep each pairing algorithm in its own module. `BlossomV1` belongs under a dedicated `pairing::algorithms::blossom_v1` module; future algorithms must be addable as sibling modules without copying match publication or table assignment.

The required flow is:

```text
Tournament/application layer
    -> builds an immutable pairing snapshot

BlossomV1 module
    -> validates the snapshot and decides who plays whom
    -> returns contestant pairings plus diagnostics

Match publication
    -> creates MatchIds

Table-assignment policy
    -> independently ranks published matches by average starting ELO
    -> assigns lower table numbers to higher-ranked matches
```

The Blossom module must not receive tournament aggregates, repositories, entrant names, club names, individual game scores, table counts, scheduled-match types or UI types. Use stable IDs and scalar snapshot values.

The `BlossomV1` request must contain:

- Round number.
- Active entrant IDs and club IDs.
- Starting ELO.
- Current performance score, using zero rather than `Option` in round one.
- Matches won.
- Opponent-score sum.
- Bye count.
- Previous contestant pairings with their round numbers.
- An explicit versioned `BlossomV1Policy`.

The first policy owns the same-club and rematch switches/window, component weights, bye/same-club/rematch penalties and maximum supported entrant count. Do not introduce a generic configuration language or scatter policy constants through edge-building code.

Graph node indexes, edge indexes and solver-library types are private implementation details. For an odd entrant count, use one private synthetic bye node; an edge to that node represents a bye.

Keep a small, exhaustive minimum-cost matching implementation as a test-only correctness oracle. A production in-house Blossom kernel must live behind the same private solver boundary and be differential-tested against the oracle on generated small graphs before it is trusted for publication. The stable-ID candidate graph and cost breakdown may be exposed for diagnostics and future visualization; private solver state may not.

Tournament-operator controls for policy weights are future UI polish. If added, the application must validate them and create an explicit versioned policy snapshot for the round; the UI must never mutate graph edges or solver state directly.

Relaxation is expected algorithm behavior rather than an immediate error. Attempt tiers in this order:

```text
Strict
SameClubAllowed
RematchesAllowed
```

The successful proposal must report its relaxation tier. Same-club matches are forbidden in `Strict`; rematches are forbidden until `RematchesAllowed`. Allowed exceptions remain strongly penalized and produce warnings.

Round-one competitive cost is primarily absolute ELO difference. Later rounds may also use absolute performance-score, match-win and opponent-strength differences. Use checked integer arithmetic for every weighted component and total. Overflow returns a typed `PairingCostOverflow`; never wrap or saturate.

Retain a public cost breakdown containing competitive gaps, policy penalties, deterministic tie-break and total. Successful proposals contain contestant pairings, an optional bye, total cost, policy version, relaxation tier, warnings and timing/candidate diagnostics. They never contain `MatchId`s.

Proposal order has no sporting meaning. Make it deterministic for auditing, but table assignment must never consume solver order. It independently calculates average starting ELO from published match participants and entrant snapshots, with deterministic stable-ID tie-breaking.

Distinguish:

- Invalid caller snapshots.
- Checked cost overflow.
- No complete matching after every relaxation tier.
- Solver-library failure, rejection, timeout or contained panic.
- Invalid solver output.

Ordinary failure to match at a stricter tier is not a solver failure.

Before returning a successful proposal, validate that:

- Every entrant appears exactly once as a contestant or bye recipient.
- No entrant plays itself.
- Every match corresponds to an eligible edge for the reported tier.
- Strict proposals contain no same-club match.
- Proposals before `RematchesAllowed` contain no rematch.
- Odd entrant counts have exactly one bye and even counts have none.
- An avoidable repeated bye was not assigned.
- Identical input produces identical output.

Invalid solver output must become a typed error and must never reach match publication.

---

# Table configuration and match assignment

Tournament creation must also accept the number of available tables.

Use a checked positive integer type equivalent to:

```rust
pub struct TableCount(NonZeroU16);
```

The table count:

- May be changed while the tournament is in draft state.
- Must be frozen when the tournament starts.
- Defines the only valid table numbers: `1..=table_count`.
- Must not be inferred from the number of entrants or generated matches.

Table assignment happens only after the matching algorithm has finalized the ordered matches for a round. Pairing and ranking logic must not depend on table numbers.

Assign available tables in ascending numeric order to matches in descending match-rank order:

```text
highest-ranked match -> table 1
next-highest-ranked match -> table 2
...
lowest-ranked assigned match -> highest table number in use
```

When a round contains more matches than available tables, assign the highest-ranked matches first. Remaining matches stay explicitly unassigned until a table becomes available; do not create table numbers above the configured table count.

Byes do not receive a table assignment.

Persist the assigned table number with the published scheduled match. Re-running an unpublished pairing preview may replace its table assignments, but never silently change table assignments on published historical rounds.

Test that:

- Zero and overflowing table counts are rejected.
- Draft tournaments may change their table count.
- Started tournaments may not change their table count.
- Higher-ranked matches receive lower table numbers.
- Excess matches remain unassigned when there are fewer tables than matches.
- Byes remain unassigned.
- No assigned table number exceeds the configured table count.

---

# Additional tests

## Game validation tests

Test:

- `11–0`.
- `11–9`.
- `12–10`.
- Long deuce game such as `24–22`.
- Invalid `11–10`.
- Invalid `12–11`.
- Invalid tied score.
- Negative or overflowing input.
- Sequential game numbering.
- Duplicate game numbering.

## Best-of-three tests

Test:

- Valid `2–0`.
- Valid `2–1`.
- Incomplete `1–0`.
- Incomplete `1–1`.
- Rejection of a fourth game.
- Rejection of a third game after a `2–0` completion.

## Best-of-five tests

Test:

- Valid `3–0`.
- Valid `3–1`.
- Valid `3–2`.
- Incomplete `2–0`.
- Incomplete `2–2`.
- Rejection of a sixth game.
- Rejection of additional games after a `3–0`, `3–1` or `3–2` completion.

## Algorithm-isolation tests

Verify:

- A `2–0` win and a `2–1` win produce the same performance-score input.
- A `3–0` win and a `3–2` win produce the same performance-score input.
- Point margins do not change `EloExpectationDeltaV1`.
- Correcting points without changing the winner does not change the performance score.
- Correcting a result so the winner changes reverses and recalculates the score contributions.
- Game and point totals remain available even though the algorithm ignores them.

## Persistence and export tests

Verify:

- A match result cannot be persisted without game scores.
- Game order survives a save and reload.
- High deuce scores survive a save and reload.
- Result revisions preserve old game scores.
- The export model contains every registered game.
- The export match winner agrees with the registered games.
- Best-of-three and best-of-five formats remain distinguishable in stored and exported records.

---

# Updated definition of done for result entry

Result functionality is complete only when:

- Both best-of-three and best-of-five are supported.
- Every individual game score is registered.
- Match winners are derived rather than trusted from client input.
- Deuce scores are validated correctly.
- The tournament algorithm receives only the final win or loss.
- Detailed results are retained for future NTTB NAS export.
- Corrections preserve an audit history.
- Keyboard-only result entry is practical during a tournament.
- All relevant invariants, persistence paths and export models are tested.
