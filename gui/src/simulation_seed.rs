use std::cell::RefCell;

thread_local! {
    static SEED_SEQUENCE: RefCell<Option<SeedSequence>> = const { RefCell::new(None) };
}

struct SeedSequence {
    base: u64,
    next: u64,
}

pub(crate) fn fresh_simulation_seed() -> u64 {
    SEED_SEQUENCE.with(|sequence| {
        let mut sequence = sequence.borrow_mut();
        let sequence = sequence.get_or_insert_with(|| SeedSequence {
            base: entropy(),
            next: 0,
        });
        let value = mix(sequence.base.wrapping_add(sequence.next));
        sequence.next = sequence.next.wrapping_add(1);
        value
    })
}

pub(crate) fn match_simulation_seed(run_seed: u64, match_id: &str) -> u64 {
    mix(run_seed) ^ stable_text_hash(match_id)
}

fn stable_text_hash(value: &str) -> u64 {
    value
        .bytes()
        .fold(14_695_981_039_346_656_037, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(1_099_511_628_211)
        })
}

fn mix(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(target_arch = "wasm32")]
fn entropy() -> u64 {
    js_sys::Date::now().to_bits().rotate_left(17) ^ js_sys::Math::random().to_bits()
}

#[cfg(not(target_arch = "wasm32"))]
fn entropy() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos() as u64)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{fresh_simulation_seed, match_simulation_seed};

    #[test]
    fn fresh_run_seeds_do_not_repeat_within_a_session() {
        let seeds = (0..256)
            .map(|_| fresh_simulation_seed())
            .collect::<Vec<_>>();
        assert_eq!(seeds.iter().copied().collect::<HashSet<_>>().len(), 256);
    }

    #[test]
    fn match_seed_is_reproducible_but_changes_between_runs_and_matches() {
        let first = match_simulation_seed(41, "round-1-match-1");

        assert_eq!(first, match_simulation_seed(41, "round-1-match-1"));
        assert_ne!(first, match_simulation_seed(42, "round-1-match-1"));
        assert_ne!(first, match_simulation_seed(41, "round-1-match-2"));
    }

    #[test]
    fn different_run_seeds_change_generated_game_results() {
        use tabletennis_tournament::pairing::EloRating;
        use tabletennis_tournament::results::MatchFormat;
        use tabletennis_tournament::simulation::simulate_match_games;

        let first = simulate_match_games(
            MatchFormat::BestOfFive,
            EloRating::new(1_300),
            EloRating::new(1_300),
            match_simulation_seed(41, "round-1-match-1"),
        )
        .unwrap();
        let second = simulate_match_games(
            MatchFormat::BestOfFive,
            EloRating::new(1_300),
            EloRating::new(1_300),
            match_simulation_seed(42, "round-1-match-1"),
        )
        .unwrap();

        assert_ne!(first, second);
    }
}
