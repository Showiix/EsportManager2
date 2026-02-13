use super::helpers::*;
use crate::models::{SeasonPhase, TournamentType};

#[test]
fn test_get_region_name() {
    assert_eq!(get_region_name(1), "LPL");
    assert_eq!(get_region_name(2), "LCK");
    assert_eq!(get_region_name(3), "LEC");
    assert_eq!(get_region_name(4), "LCS");
    assert_eq!(get_region_name(99), "Unknown");
}

#[test]
fn test_phase_to_tournament_type() {
    assert_eq!(
        SeasonPhase::SpringRegular.to_tournament_type(),
        Some(TournamentType::SpringRegular)
    );
    assert_eq!(
        SeasonPhase::Msi.to_tournament_type(),
        Some(TournamentType::Msi)
    );
    assert_eq!(SeasonPhase::TransferWindow.to_tournament_type(), None);
    assert_eq!(SeasonPhase::Draft.to_tournament_type(), None);
}

#[test]
fn test_get_phase_display_name() {
    assert_eq!(SeasonPhase::SpringRegular.display_name(), "春季常规赛");
    assert_eq!(SeasonPhase::Msi.display_name(), "MSI季中赛");
    assert_eq!(SeasonPhase::WorldChampionship.display_name(), "世界赛");
}
