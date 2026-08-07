use std::collections::HashSet;

use crate::identity::EntrantId;

use super::{BlossomPairingError, PairingRequest, PreviousMatch};

pub fn validate_request(request: &PairingRequest) -> Result<(), BlossomPairingError> {
    let entrant_count = request.entrants.len();
    if entrant_count < 2 {
        return Err(BlossomPairingError::NotEnoughEntrants { entrant_count });
    }

    if entrant_count > request.policy.maximum_entrant_count {
        return Err(BlossomPairingError::EntrantLimitExceeded {
            entrant_count,
            maximum: request.policy.maximum_entrant_count,
        });
    }

    let entrant_ids = validated_entrant_ids(request)?;
    for previous_match in &request.previous_matches {
        validate_history_entry(request, &entrant_ids, previous_match)?;
    }

    Ok(())
}

fn validated_entrant_ids(
    request: &PairingRequest,
) -> Result<HashSet<&EntrantId>, BlossomPairingError> {
    let mut entrant_ids = HashSet::with_capacity(request.entrants.len());
    for entrant in &request.entrants {
        if !entrant_ids.insert(&entrant.entrant_id) {
            return Err(BlossomPairingError::DuplicateEntrant {
                entrant_id: entrant.entrant_id.clone(),
            });
        }
    }
    Ok(entrant_ids)
}

fn validate_history_entry(
    request: &PairingRequest,
    entrant_ids: &HashSet<&EntrantId>,
    previous_match: &PreviousMatch,
) -> Result<(), BlossomPairingError> {
    for entrant_id in [
        &previous_match.first_entrant_id,
        &previous_match.second_entrant_id,
    ] {
        if !entrant_ids.contains(entrant_id) {
            return Err(BlossomPairingError::UnknownEntrantInHistory {
                unknown_entrant_id: entrant_id.clone(),
            });
        }
    }

    if previous_match.first_entrant_id == previous_match.second_entrant_id {
        return Err(BlossomPairingError::SelfMatchInHistory {
            entrant_id: previous_match.first_entrant_id.clone(),
        });
    }

    if previous_match.round_number > request.round_number {
        return Err(BlossomPairingError::InvalidHistoryRound {
            history_round: previous_match.round_number,
            requested_round: request.round_number,
        });
    }

    Ok(())
}
