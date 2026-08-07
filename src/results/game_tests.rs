use super::*;

#[test]
fn checked_points_reject_negative_and_overflowing_input() {
    assert_eq!(GamePoints::try_from(-1_i64), Err(GamePointsError::Negative));
    assert_eq!(
        GamePoints::try_from(i64::from(u16::MAX) + 1),
        Err(GamePointsError::ExceedsStorageLimit)
    );
    assert_eq!(
        GamePoints::try_from(i64::MAX),
        Err(GamePointsError::ExceedsStorageLimit)
    );
}

#[test]
fn checked_game_number_rejects_zero() {
    assert_eq!(GameScore::new(0, 11, 5), Err(GameNumberError));
}
