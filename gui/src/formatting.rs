use std::time::Duration;

use tabletennis_tournament::pairing::algorithms::blossom_v1::RelaxationTier;
use tabletennis_tournament::results::MatchFormat;

pub(crate) fn compact_u64(value: u64) -> String {
    match value {
        1_000_000_000.. => format_scaled(value, 1_000_000_000, "B"),
        1_000_000.. => format_scaled(value, 1_000_000, "M"),
        1_000.. => format_scaled(value, 1_000, "K"),
        _ => value.to_string(),
    }
}

pub(crate) fn grouped_u64(value: u64) -> String {
    let digits = value.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, character) in digits.chars().enumerate() {
        if index != 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(character);
    }
    grouped
}

pub(crate) fn duration(duration: Duration) -> String {
    if duration.as_micros() < 1_000 {
        format!("{} µs", duration.as_micros())
    } else if duration.as_millis() < 1_000 {
        format!("{:.1} ms", duration.as_secs_f64() * 1_000.0)
    } else {
        format!("{:.2} s", duration.as_secs_f64())
    }
}

pub(crate) const fn match_format(format: MatchFormat) -> &'static str {
    match format {
        MatchFormat::BestOfThree => "Best of three",
        MatchFormat::BestOfFive => "Best of five",
    }
}

pub(crate) const fn relaxation_tier(tier: RelaxationTier) -> &'static str {
    match tier {
        RelaxationTier::Strict => "Strict",
        RelaxationTier::SameClubAllowed => "Same-club relaxation",
        RelaxationTier::RematchesAllowed => "Rematch relaxation",
    }
}

fn format_scaled(value: u64, divisor: u64, suffix: &str) -> String {
    let scaled = value as f64 / divisor as f64;
    if scaled >= 100.0 {
        format!("{scaled:.0}{suffix}")
    } else if scaled >= 10.0 {
        format!("{scaled:.1}{suffix}")
    } else {
        format!("{scaled:.2}{suffix}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_costs_are_compact_but_have_a_grouped_exact_form() {
        assert_eq!(compact_u64(1_479_408_456), "1.48B");
        assert_eq!(grouped_u64(1_479_408_456), "1,479,408,456");
    }
}
