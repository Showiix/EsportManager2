use super::champion::{
    self, get_champions_by_position, Archetype, Champion, MasteryTier, VersionTier,
};
use super::meta_engine::MetaType;
use super::traits::TraitType;
use crate::models::player::Position;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

const POSITIONS: [Position; 5] = [
    Position::Top,
    Position::Jug,
    Position::Mid,
    Position::Adc,
    Position::Sup,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompType {
    Rush,
    PickOff,
    AllIn,
    MidJungle,
    TopJungle,
    Protect,
    Fortress,
    UtilityComp,
    Stall,
    BotLane,
    Teamfight,
    Dive,
    Skirmish,
    DualCarry,
    Flex,
    Splitpush,
    SideLane,
    Control,
    TripleThreat,
    LateGame,
}

impl CompType {
    pub fn all() -> &'static [CompType] {
        &[
            CompType::Rush,
            CompType::PickOff,
            CompType::AllIn,
            CompType::MidJungle,
            CompType::TopJungle,
            CompType::Protect,
            CompType::Fortress,
            CompType::UtilityComp,
            CompType::Stall,
            CompType::BotLane,
            CompType::Teamfight,
            CompType::Dive,
            CompType::Skirmish,
            CompType::DualCarry,
            CompType::Flex,
            CompType::Splitpush,
            CompType::SideLane,
            CompType::Control,
            CompType::TripleThreat,
            CompType::LateGame,
        ]
    }

    fn detection_priority() -> &'static [CompType] {
        &[
            CompType::Rush,
            CompType::Protect,
            CompType::Fortress,
            CompType::Stall,
            CompType::Flex,
            CompType::SideLane,
            CompType::TripleThreat,
            CompType::PickOff,
            CompType::MidJungle,
            CompType::TopJungle,
            CompType::BotLane,
            CompType::Dive,
            CompType::Skirmish,
            CompType::DualCarry,
            CompType::Splitpush,
            CompType::Control,
            CompType::AllIn,
            CompType::UtilityComp,
            CompType::Teamfight,
            CompType::LateGame,
        ]
    }

    pub fn difficulty_bonus(&self) -> f64 {
        match self {
            CompType::Rush
            | CompType::Protect
            | CompType::Fortress
            | CompType::Stall
            | CompType::Flex
            | CompType::SideLane
            | CompType::TripleThreat => 2.0,
            CompType::PickOff
            | CompType::MidJungle
            | CompType::TopJungle
            | CompType::BotLane
            | CompType::Dive
            | CompType::Skirmish
            | CompType::DualCarry
            | CompType::Splitpush
            | CompType::Control => 1.5,
            CompType::AllIn | CompType::UtilityComp | CompType::Teamfight | CompType::LateGame => {
                1.0
            }
        }
    }

    pub fn core_archetypes(&self) -> &'static [Archetype] {
        match self {
            CompType::Rush
            | CompType::PickOff
            | CompType::AllIn
            | CompType::MidJungle
            | CompType::TopJungle => &[Archetype::Aggressive],
            CompType::Protect | CompType::Stall | CompType::LateGame | CompType::DualCarry => {
                &[Archetype::Scaling]
            }
            CompType::Fortress | CompType::UtilityComp | CompType::Control => &[Archetype::Utility],
            CompType::Splitpush | CompType::SideLane | CompType::TripleThreat => {
                &[Archetype::Splitpush]
            }
            CompType::Teamfight | CompType::Dive | CompType::Skirmish => &[Archetype::Teamfight],
            CompType::BotLane => &[Archetype::Scaling, Archetype::Utility],
            CompType::Flex => Archetype::all(),
        }
    }

    pub fn is_meta_favored(&self, meta_type: MetaType) -> bool {
        let favored = meta_type.favored_archetypes();
        if matches!(self, CompType::Flex) {
            return !favored.is_empty();
        }
        self.core_archetypes()
            .iter()
            .any(|arch| favored.contains(arch))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeamSide {
    Home,
    Away,
}

const BAN_ORDER: [(TeamSide, u8); 10] = [
    (TeamSide::Home, 1),
    (TeamSide::Away, 1),
    (TeamSide::Home, 1),
    (TeamSide::Away, 1),
    (TeamSide::Home, 1),
    (TeamSide::Away, 1),
    (TeamSide::Home, 2),
    (TeamSide::Away, 2),
    (TeamSide::Home, 2),
    (TeamSide::Away, 2),
];

const PICK_ORDER: [TeamSide; 10] = [
    TeamSide::Home,
    TeamSide::Away,
    TeamSide::Away,
    TeamSide::Home,
    TeamSide::Home,
    TeamSide::Away,
    TeamSide::Away,
    TeamSide::Home,
    TeamSide::Home,
    TeamSide::Away,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanEntry {
    pub team_side: TeamSide,
    pub champion_id: u8,
    pub ban_phase: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickEntry {
    pub player_id: u64,
    pub champion_id: u8,
    pub position: Position,
    pub mastery_tier: MasteryTier,
    #[serde(skip)]
    pub traits: Vec<TraitType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftResult {
    pub bans: Vec<BanEntry>,
    pub home_picks: Vec<PickEntry>,
    pub away_picks: Vec<PickEntry>,
    pub home_comp: Option<CompType>,
    pub away_comp: Option<CompType>,
    pub home_bp_modifiers: HashMap<u64, f64>,
    pub away_bp_modifiers: HashMap<u64, f64>,
}

#[derive(Debug, Clone)]
pub struct PlayerChampionPool {
    pub player_id: u64,
    pub position: Position,
    pub ability: u8,
    pub masteries: HashMap<u8, MasteryTier>,
    pub traits: Vec<TraitType>,
}

pub struct BpEngine;

impl BpEngine {
    pub fn run_draft(
        home_players: &[PlayerChampionPool],
        away_players: &[PlayerChampionPool],
        version_tiers: &HashMap<u8, VersionTier>,
        meta_type: MetaType,
        rng: &mut StdRng,
    ) -> DraftResult {
        let mut bans = Vec::with_capacity(10);
        let mut banned_champions = HashSet::new();

        for (team_side, ban_phase) in BAN_ORDER {
            let target_players = match team_side {
                TeamSide::Home => away_players,
                TeamSide::Away => home_players,
            };
            if let Some(champion_id) =
                select_best_ban(target_players, version_tiers, &banned_champions, rng)
            {
                banned_champions.insert(champion_id);
                bans.push(BanEntry {
                    team_side,
                    champion_id,
                    ban_phase,
                });
            }
        }

        let mut picked_champions = HashSet::new();
        let mut home_picks = Vec::with_capacity(5);
        let mut away_picks = Vec::with_capacity(5);

        for team_side in PICK_ORDER {
            let next_pick = match team_side {
                TeamSide::Home => select_best_pick(
                    home_players,
                    &home_picks,
                    &banned_champions,
                    &picked_champions,
                    version_tiers,
                    rng,
                ),
                TeamSide::Away => select_best_pick(
                    away_players,
                    &away_picks,
                    &banned_champions,
                    &picked_champions,
                    version_tiers,
                    rng,
                ),
            };

            if let Some(pick) = next_pick {
                if picked_champions.insert(pick.champion_id) {
                    match team_side {
                        TeamSide::Home => home_picks.push(pick),
                        TeamSide::Away => away_picks.push(pick),
                    }
                }
            }
        }

        fill_missing_positions(
            home_players,
            &mut home_picks,
            &banned_champions,
            &mut picked_champions,
            rng,
        );
        fill_missing_positions(
            away_players,
            &mut away_picks,
            &banned_champions,
            &mut picked_champions,
            rng,
        );

        let home_comp = detect_comp(&picks_to_comp_view(&home_picks));
        let away_comp = detect_comp(&picks_to_comp_view(&away_picks));

        let home_bp_modifiers = calculate_team_bp_modifiers(
            &home_picks,
            version_tiers,
            home_comp,
            away_comp,
            meta_type,
        );
        let away_bp_modifiers = calculate_team_bp_modifiers(
            &away_picks,
            version_tiers,
            away_comp,
            home_comp,
            meta_type,
        );

        DraftResult {
            bans,
            home_picks,
            away_picks,
            home_comp,
            away_comp,
            home_bp_modifiers,
            away_bp_modifiers,
        }
    }
}

pub fn detect_comp(picks: &[(Position, Archetype)]) -> Option<CompType> {
    let snapshot = CompSnapshot::from_picks(picks);
    if snapshot.total != POSITIONS.len() {
        return None;
    }

    for comp in CompType::detection_priority() {
        if comp_matches_full(*comp, &snapshot) {
            return Some(*comp);
        }
    }
    None
}

pub fn hard_counter_pairs() -> &'static [(CompType, CompType)] {
    HARD_COUNTERS
}

pub fn soft_counter_pairs() -> &'static [(CompType, CompType)] {
    SOFT_COUNTERS
}

pub fn calculate_counter_modifier(my_comp: &CompType, enemy_comp: &CompType) -> f64 {
    if HARD_COUNTERS
        .iter()
        .any(|(attacker, victim)| attacker == my_comp && victim == enemy_comp)
    {
        return 1.5;
    }
    if HARD_COUNTERS
        .iter()
        .any(|(attacker, victim)| attacker == enemy_comp && victim == my_comp)
    {
        return -1.5;
    }

    if SOFT_COUNTERS
        .iter()
        .any(|(attacker, victim)| attacker == my_comp && victim == enemy_comp)
    {
        return 0.5;
    }
    if SOFT_COUNTERS
        .iter()
        .any(|(attacker, victim)| attacker == enemy_comp && victim == my_comp)
    {
        return -0.5;
    }

    0.0
}

const HARD_COUNTERS: &[(CompType, CompType)] = &[
    (CompType::Rush, CompType::Control),
    (CompType::PickOff, CompType::Splitpush),
    (CompType::AllIn, CompType::LateGame),
    (CompType::MidJungle, CompType::TripleThreat),
    (CompType::TopJungle, CompType::SideLane),
    (CompType::Protect, CompType::PickOff),
    (CompType::Fortress, CompType::AllIn),
    (CompType::UtilityComp, CompType::TopJungle),
    (CompType::Stall, CompType::Rush),
    (CompType::BotLane, CompType::MidJungle),
    (CompType::Teamfight, CompType::Skirmish),
    (CompType::Dive, CompType::DualCarry),
    (CompType::Skirmish, CompType::Stall),
    (CompType::DualCarry, CompType::Fortress),
    (CompType::Flex, CompType::Teamfight),
    (CompType::Splitpush, CompType::Protect),
    (CompType::SideLane, CompType::UtilityComp),
    (CompType::Control, CompType::BotLane),
    (CompType::TripleThreat, CompType::Dive),
    (CompType::LateGame, CompType::Flex),
];

const SOFT_COUNTERS: &[(CompType, CompType)] = &[
    (CompType::Rush, CompType::LateGame),
    (CompType::Rush, CompType::DualCarry),
    (CompType::PickOff, CompType::LateGame),
    (CompType::PickOff, CompType::Control),
    (CompType::AllIn, CompType::TripleThreat),
    (CompType::AllIn, CompType::Splitpush),
    (CompType::MidJungle, CompType::SideLane),
    (CompType::MidJungle, CompType::Control),
    (CompType::TopJungle, CompType::Splitpush),
    (CompType::TopJungle, CompType::TripleThreat),
    (CompType::Protect, CompType::Dive),
    (CompType::Protect, CompType::AllIn),
    (CompType::Fortress, CompType::PickOff),
    (CompType::Fortress, CompType::MidJungle),
    (CompType::UtilityComp, CompType::PickOff),
    (CompType::UtilityComp, CompType::Rush),
    (CompType::Stall, CompType::AllIn),
    (CompType::Stall, CompType::TopJungle),
    (CompType::BotLane, CompType::Skirmish),
    (CompType::BotLane, CompType::Dive),
    (CompType::Teamfight, CompType::Flex),
    (CompType::Teamfight, CompType::SideLane),
    (CompType::Dive, CompType::Stall),
    (CompType::Dive, CompType::Protect),
    (CompType::Skirmish, CompType::BotLane),
    (CompType::Skirmish, CompType::Fortress),
    (CompType::DualCarry, CompType::UtilityComp),
    (CompType::DualCarry, CompType::Flex),
    (CompType::Flex, CompType::Stall),
    (CompType::Flex, CompType::MidJungle),
    (CompType::Splitpush, CompType::BotLane),
    (CompType::Splitpush, CompType::UtilityComp),
    (CompType::SideLane, CompType::Fortress),
    (CompType::SideLane, CompType::Skirmish),
    (CompType::Control, CompType::Rush),
    (CompType::Control, CompType::DualCarry),
    (CompType::TripleThreat, CompType::Protect),
    (CompType::TripleThreat, CompType::Teamfight),
    (CompType::LateGame, CompType::TopJungle),
    (CompType::LateGame, CompType::Teamfight),
];

#[derive(Debug, Clone, Default)]
struct CompSnapshot {
    top: Option<Archetype>,
    jug: Option<Archetype>,
    mid: Option<Archetype>,
    adc: Option<Archetype>,
    sup: Option<Archetype>,
    agg: usize,
    scal: usize,
    util: usize,
    splitpush: usize,
    teamfight: usize,
    total: usize,
}

impl CompSnapshot {
    fn from_picks(picks: &[(Position, Archetype)]) -> Self {
        let mut snapshot = CompSnapshot::default();
        for (position, archetype) in picks {
            match position {
                Position::Top => snapshot.top = Some(*archetype),
                Position::Jug => snapshot.jug = Some(*archetype),
                Position::Mid => snapshot.mid = Some(*archetype),
                Position::Adc => snapshot.adc = Some(*archetype),
                Position::Sup => snapshot.sup = Some(*archetype),
            }
        }

        for archetype in [
            snapshot.top,
            snapshot.jug,
            snapshot.mid,
            snapshot.adc,
            snapshot.sup,
        ]
        .into_iter()
        .flatten()
        {
            snapshot.total += 1;
            match archetype {
                Archetype::Aggressive => snapshot.agg += 1,
                Archetype::Scaling => snapshot.scal += 1,
                Archetype::Utility => snapshot.util += 1,
                Archetype::Splitpush => snapshot.splitpush += 1,
                Archetype::Teamfight => snapshot.teamfight += 1,
            }
        }

        snapshot
    }

    fn remaining_slots(&self) -> usize {
        POSITIONS.len().saturating_sub(self.total)
    }

    fn archetype_count(&self, archetype: Archetype) -> usize {
        match archetype {
            Archetype::Aggressive => self.agg,
            Archetype::Scaling => self.scal,
            Archetype::Utility => self.util,
            Archetype::Splitpush => self.splitpush,
            Archetype::Teamfight => self.teamfight,
        }
    }
}

fn comp_matches_full(comp: CompType, snapshot: &CompSnapshot) -> bool {
    match comp {
        CompType::Rush => {
            snapshot.agg >= 3
                && snapshot.top == Some(Archetype::Aggressive)
                && snapshot.jug == Some(Archetype::Aggressive)
        }
        CompType::PickOff => snapshot.agg >= 3 && snapshot.jug == Some(Archetype::Aggressive),
        CompType::AllIn => snapshot.agg >= 4,
        CompType::MidJungle => {
            snapshot.mid == Some(Archetype::Aggressive)
                && snapshot.jug == Some(Archetype::Aggressive)
                && snapshot.util >= 1
        }
        CompType::TopJungle => {
            snapshot.top == Some(Archetype::Aggressive)
                && snapshot.jug == Some(Archetype::Aggressive)
                && snapshot.teamfight >= 1
        }
        CompType::Protect => {
            snapshot.adc == Some(Archetype::Scaling)
                && snapshot.sup == Some(Archetype::Utility)
                && snapshot.teamfight >= 1
        }
        CompType::Fortress => snapshot.teamfight >= 2 && snapshot.util >= 2 && snapshot.agg == 0,
        CompType::UtilityComp => snapshot.util >= 3,
        CompType::Stall => snapshot.scal >= 2 && snapshot.util >= 2 && snapshot.teamfight >= 1,
        CompType::BotLane => {
            snapshot.adc == Some(Archetype::Scaling)
                && matches!(
                    snapshot.sup,
                    Some(Archetype::Utility) | Some(Archetype::Teamfight)
                )
                && snapshot.agg >= 1
        }
        CompType::Teamfight => snapshot.teamfight >= 3,
        CompType::Dive => {
            snapshot.agg >= 2
                && snapshot.teamfight >= 2
                && matches!(
                    snapshot.jug,
                    Some(Archetype::Aggressive) | Some(Archetype::Teamfight)
                )
        }
        CompType::Skirmish => {
            snapshot.agg >= 2
                && snapshot.scal >= 1
                && (snapshot.jug == Some(Archetype::Aggressive)
                    || snapshot.mid == Some(Archetype::Aggressive))
        }
        CompType::DualCarry => {
            snapshot.mid == Some(Archetype::Scaling)
                && snapshot.adc == Some(Archetype::Scaling)
                && snapshot.util >= 1
        }
        CompType::Flex => [
            Archetype::Aggressive,
            Archetype::Scaling,
            Archetype::Utility,
            Archetype::Splitpush,
            Archetype::Teamfight,
        ]
        .iter()
        .all(|archetype| snapshot.archetype_count(*archetype) >= 1),
        CompType::Splitpush => {
            snapshot.splitpush >= 2
                && (snapshot.top == Some(Archetype::Splitpush)
                    || snapshot.mid == Some(Archetype::Splitpush))
        }
        CompType::SideLane => {
            snapshot.top == Some(Archetype::Splitpush)
                && snapshot.adc == Some(Archetype::Scaling)
                && snapshot.teamfight >= 2
        }
        CompType::Control => snapshot.util >= 2 && snapshot.scal >= 2,
        CompType::TripleThreat => {
            matches!(
                snapshot.top,
                Some(Archetype::Splitpush) | Some(Archetype::Aggressive)
            ) && matches!(
                snapshot.mid,
                Some(Archetype::Aggressive) | Some(Archetype::Splitpush)
            ) && snapshot.adc == Some(Archetype::Scaling)
        }
        CompType::LateGame => snapshot.scal >= 3,
    }
}

fn select_best_ban(
    opponent_players: &[PlayerChampionPool],
    version_tiers: &HashMap<u8, VersionTier>,
    banned_champions: &HashSet<u8>,
    rng: &mut StdRng,
) -> Option<u8> {
    let mut candidate_set = HashSet::new();
    for player in opponent_players {
        for champion in get_champions_by_position(player.position) {
            candidate_set.insert(champion.id);
        }
        for champion_id in player.masteries.keys() {
            candidate_set.insert(*champion_id);
        }
    }

    let mut candidates: Vec<u8> = candidate_set.into_iter().collect();
    candidates.sort_unstable();
    candidates.retain(|champion_id| !banned_champions.contains(champion_id));

    if candidates.is_empty() {
        candidates = champion::CHAMPIONS
            .iter()
            .map(|champion| champion.id)
            .filter(|champion_id| !banned_champions.contains(champion_id))
            .collect();
    }

    if candidates.is_empty() {
        return None;
    }

    let mut scored: Vec<(u8, i32)> = candidates
        .into_iter()
        .map(|champion_id| {
            (
                champion_id,
                calculate_ban_score(champion_id, opponent_players, version_tiers),
            )
        })
        .collect();

    scored.sort_by(|left, right| right.1.cmp(&left.1));
    let best_score = match scored.first() {
        Some((_, score)) => *score,
        None => return None,
    };

    let mut best_candidates: Vec<u8> = scored
        .into_iter()
        .filter(|(_, score)| *score == best_score)
        .map(|(champion_id, _)| champion_id)
        .collect();
    best_candidates.shuffle(rng);
    best_candidates.first().copied()
}

fn calculate_ban_score(
    champion_id: u8,
    opponent_players: &[PlayerChampionPool],
    version_tiers: &HashMap<u8, VersionTier>,
) -> i32 {
    let mut threat_score = 0i32;
    let mut has_ss_master = false;

    for player in opponent_players {
        let mastery = mastery_for_player(player, champion_id);
        threat_score = threat_score.max(i32::from(mastery.pick_score()));
        if mastery == MasteryTier::SS {
            has_ss_master = true;
        }
    }

    let version_score = i32::from(version_tier_for(champion_id, version_tiers).ban_score());
    let disruption_score = if has_ss_master { 5 } else { 0 };
    threat_score + version_score + disruption_score
}

fn select_best_pick(
    team_players: &[PlayerChampionPool],
    team_picks: &[PickEntry],
    banned_champions: &HashSet<u8>,
    picked_champions: &HashSet<u8>,
    version_tiers: &HashMap<u8, VersionTier>,
    rng: &mut StdRng,
) -> Option<PickEntry> {
    let current_comp_view = picks_to_comp_view(team_picks);
    let mut candidates: Vec<(PickEntry, i32)> = Vec::new();

    for player in team_players {
        if is_position_filled(team_picks, player.position) {
            continue;
        }

        for champion in available_champions_for_position(player.position) {
            if banned_champions.contains(&champion.id) || picked_champions.contains(&champion.id) {
                continue;
            }

            let mastery = mastery_for_player(player, champion.id);
            let version_tier = version_tier_for(champion.id, version_tiers);

            let mastery_score = i32::from(mastery.pick_score());
            let version_score = pick_version_score(mastery, version_tier);
            let comp_synergy_score = if pick_helps_form_comp(
                &current_comp_view,
                (player.position, champion.archetype),
            ) {
                3
            } else {
                0
            };

            let total_score = mastery_score + version_score + comp_synergy_score;
            candidates.push((
                PickEntry {
                    player_id: player.player_id,
                    champion_id: champion.id,
                    position: player.position,
                    mastery_tier: mastery,
                    traits: player.traits.clone(),
                },
                total_score,
            ));
        }
    }

    if candidates.is_empty() {
        return fallback_pick(
            team_players,
            team_picks,
            banned_champions,
            picked_champions,
            rng,
        );
    }

    candidates.sort_by(|left, right| right.1.cmp(&left.1));
    let best_score = match candidates.first() {
        Some((_, score)) => *score,
        None => return None,
    };
    let mut best_candidates: Vec<PickEntry> = candidates
        .into_iter()
        .filter(|(_, score)| *score == best_score)
        .map(|(pick_entry, _)| pick_entry)
        .collect();
    best_candidates.shuffle(rng);
    best_candidates.first().cloned()
}

fn fill_missing_positions(
    team_players: &[PlayerChampionPool],
    team_picks: &mut Vec<PickEntry>,
    banned_champions: &HashSet<u8>,
    picked_champions: &mut HashSet<u8>,
    rng: &mut StdRng,
) {
    while !all_team_positions_filled(team_players, team_picks) {
        let Some(next_pick) = fallback_pick(
            team_players,
            team_picks,
            banned_champions,
            picked_champions,
            rng,
        ) else {
            break;
        };

        if picked_champions.insert(next_pick.champion_id) {
            team_picks.push(next_pick);
        } else {
            break;
        }
    }
}

fn fallback_pick(
    team_players: &[PlayerChampionPool],
    team_picks: &[PickEntry],
    banned_champions: &HashSet<u8>,
    picked_champions: &HashSet<u8>,
    rng: &mut StdRng,
) -> Option<PickEntry> {
    let player = team_players
        .iter()
        .find(|candidate| !is_position_filled(team_picks, candidate.position))?;

    let mut position_champions = available_champions_for_position(player.position);
    position_champions.shuffle(rng);

    if let Some(champion) = position_champions
        .iter()
        .find(|candidate| {
            !banned_champions.contains(&candidate.id) && !picked_champions.contains(&candidate.id)
        })
        .copied()
    {
        return Some(PickEntry {
            player_id: player.player_id,
            champion_id: champion.id,
            position: player.position,
            mastery_tier: mastery_for_player(player, champion.id),
            traits: player.traits.clone(),
        });
    }

    let mut global_candidates: Vec<&Champion> = champion::CHAMPIONS
        .iter()
        .filter(|champion| {
            !banned_champions.contains(&champion.id) && !picked_champions.contains(&champion.id)
        })
        .collect();
    global_candidates.shuffle(rng);

    global_candidates.first().map(|champion| PickEntry {
        player_id: player.player_id,
        champion_id: champion.id,
        position: player.position,
        mastery_tier: mastery_for_player(player, champion.id),
        traits: player.traits.clone(),
    })
}

fn all_team_positions_filled(
    team_players: &[PlayerChampionPool],
    team_picks: &[PickEntry],
) -> bool {
    team_players
        .iter()
        .all(|player| is_position_filled(team_picks, player.position))
}

fn available_champions_for_position(position: Position) -> Vec<&'static Champion> {
    get_champions_by_position(position)
}

fn is_position_filled(team_picks: &[PickEntry], position: Position) -> bool {
    team_picks.iter().any(|pick| pick.position == position)
}

fn picks_to_comp_view(team_picks: &[PickEntry]) -> Vec<(Position, Archetype)> {
    team_picks
        .iter()
        .filter_map(|pick| {
            champion::get_champion(pick.champion_id)
                .map(|champion| (pick.position, champion.archetype))
        })
        .collect()
}

fn pick_helps_form_comp(
    current_picks: &[(Position, Archetype)],
    candidate_pick: (Position, Archetype),
) -> bool {
    let current_snapshot = CompSnapshot::from_picks(current_picks);

    let mut next_picks = current_picks.to_vec();
    next_picks.push(candidate_pick);
    let next_snapshot = CompSnapshot::from_picks(&next_picks);

    let mut current_best = -1;
    let mut next_best = -1;

    for comp in CompType::all() {
        current_best = current_best.max(comp_partial_score(*comp, &current_snapshot));
        next_best = next_best.max(comp_partial_score(*comp, &next_snapshot));
    }

    next_best >= 0 && next_best > current_best
}

fn comp_partial_score(comp: CompType, snapshot: &CompSnapshot) -> i32 {
    macro_rules! add_or_fail {
        ($score:expr, $maybe:expr) => {
            match $maybe {
                Some(value) => {
                    $score += value;
                }
                None => return -1,
            }
        };
    }

    let mut score = 0i32;
    match comp {
        CompType::Rush => {
            add_or_fail!(
                score,
                multi_count_progress(snapshot, &[(Archetype::Aggressive, 3)])
            );
            add_or_fail!(
                score,
                pos_exact_progress(snapshot.top, Archetype::Aggressive)
            );
            add_or_fail!(
                score,
                pos_exact_progress(snapshot.jug, Archetype::Aggressive)
            );
        }
        CompType::PickOff => {
            add_or_fail!(
                score,
                multi_count_progress(snapshot, &[(Archetype::Aggressive, 3)])
            );
            add_or_fail!(
                score,
                pos_exact_progress(snapshot.jug, Archetype::Aggressive)
            );
        }
        CompType::AllIn => {
            add_or_fail!(
                score,
                multi_count_progress(snapshot, &[(Archetype::Aggressive, 4)])
            );
        }
        CompType::MidJungle => {
            add_or_fail!(
                score,
                pos_exact_progress(snapshot.mid, Archetype::Aggressive)
            );
            add_or_fail!(
                score,
                pos_exact_progress(snapshot.jug, Archetype::Aggressive)
            );
            add_or_fail!(
                score,
                multi_count_progress(snapshot, &[(Archetype::Utility, 1)])
            );
        }
        CompType::TopJungle => {
            add_or_fail!(
                score,
                pos_exact_progress(snapshot.top, Archetype::Aggressive)
            );
            add_or_fail!(
                score,
                pos_exact_progress(snapshot.jug, Archetype::Aggressive)
            );
            add_or_fail!(
                score,
                multi_count_progress(snapshot, &[(Archetype::Teamfight, 1)])
            );
        }
        CompType::Protect => {
            add_or_fail!(score, pos_exact_progress(snapshot.adc, Archetype::Scaling));
            add_or_fail!(score, pos_exact_progress(snapshot.sup, Archetype::Utility));
            add_or_fail!(
                score,
                multi_count_progress(snapshot, &[(Archetype::Teamfight, 1)])
            );
        }
        CompType::Fortress => {
            add_or_fail!(
                score,
                multi_count_progress(
                    snapshot,
                    &[(Archetype::Teamfight, 2), (Archetype::Utility, 2)]
                )
            );
            add_or_fail!(score, no_archetype_progress(snapshot.agg));
        }
        CompType::UtilityComp => {
            add_or_fail!(
                score,
                multi_count_progress(snapshot, &[(Archetype::Utility, 3)])
            );
        }
        CompType::Stall => {
            add_or_fail!(
                score,
                multi_count_progress(
                    snapshot,
                    &[
                        (Archetype::Scaling, 2),
                        (Archetype::Utility, 2),
                        (Archetype::Teamfight, 1),
                    ]
                )
            );
        }
        CompType::BotLane => {
            add_or_fail!(score, pos_exact_progress(snapshot.adc, Archetype::Scaling));
            add_or_fail!(
                score,
                pos_one_of_progress(snapshot.sup, &[Archetype::Utility, Archetype::Teamfight])
            );
            add_or_fail!(
                score,
                multi_count_progress(snapshot, &[(Archetype::Aggressive, 1)])
            );
        }
        CompType::Teamfight => {
            add_or_fail!(
                score,
                multi_count_progress(snapshot, &[(Archetype::Teamfight, 3)])
            );
        }
        CompType::Dive => {
            add_or_fail!(
                score,
                multi_count_progress(
                    snapshot,
                    &[(Archetype::Aggressive, 2), (Archetype::Teamfight, 2)]
                )
            );
            add_or_fail!(
                score,
                pos_one_of_progress(snapshot.jug, &[Archetype::Aggressive, Archetype::Teamfight])
            );
        }
        CompType::Skirmish => {
            add_or_fail!(
                score,
                multi_count_progress(
                    snapshot,
                    &[(Archetype::Aggressive, 2), (Archetype::Scaling, 1)]
                )
            );
            add_or_fail!(
                score,
                either_pos_exact_progress(snapshot.jug, snapshot.mid, Archetype::Aggressive)
            );
        }
        CompType::DualCarry => {
            add_or_fail!(score, pos_exact_progress(snapshot.mid, Archetype::Scaling));
            add_or_fail!(score, pos_exact_progress(snapshot.adc, Archetype::Scaling));
            add_or_fail!(
                score,
                multi_count_progress(snapshot, &[(Archetype::Utility, 1)])
            );
        }
        CompType::Flex => {
            add_or_fail!(
                score,
                multi_count_progress(
                    snapshot,
                    &[
                        (Archetype::Aggressive, 1),
                        (Archetype::Scaling, 1),
                        (Archetype::Utility, 1),
                        (Archetype::Splitpush, 1),
                        (Archetype::Teamfight, 1),
                    ]
                )
            );
        }
        CompType::Splitpush => {
            add_or_fail!(
                score,
                multi_count_progress(snapshot, &[(Archetype::Splitpush, 2)])
            );
            add_or_fail!(
                score,
                either_pos_exact_progress(snapshot.top, snapshot.mid, Archetype::Splitpush)
            );
        }
        CompType::SideLane => {
            add_or_fail!(
                score,
                pos_exact_progress(snapshot.top, Archetype::Splitpush)
            );
            add_or_fail!(score, pos_exact_progress(snapshot.adc, Archetype::Scaling));
            add_or_fail!(
                score,
                multi_count_progress(snapshot, &[(Archetype::Teamfight, 2)])
            );
        }
        CompType::Control => {
            add_or_fail!(
                score,
                multi_count_progress(
                    snapshot,
                    &[(Archetype::Utility, 2), (Archetype::Scaling, 2)]
                )
            );
        }
        CompType::TripleThreat => {
            add_or_fail!(
                score,
                pos_one_of_progress(snapshot.top, &[Archetype::Splitpush, Archetype::Aggressive])
            );
            add_or_fail!(
                score,
                pos_one_of_progress(snapshot.mid, &[Archetype::Aggressive, Archetype::Splitpush])
            );
            add_or_fail!(score, pos_exact_progress(snapshot.adc, Archetype::Scaling));
        }
        CompType::LateGame => {
            add_or_fail!(
                score,
                multi_count_progress(snapshot, &[(Archetype::Scaling, 3)])
            );
        }
    }

    score
}

fn multi_count_progress(
    snapshot: &CompSnapshot,
    requirements: &[(Archetype, usize)],
) -> Option<i32> {
    let mut total_deficit = 0usize;
    let mut progress = 0i32;

    for (archetype, required_count) in requirements {
        let current = snapshot.archetype_count(*archetype);
        if current < *required_count {
            total_deficit += required_count - current;
        }
        progress += std::cmp::min(current, *required_count) as i32;
    }

    if total_deficit > snapshot.remaining_slots() {
        None
    } else {
        Some(progress)
    }
}

fn no_archetype_progress(count: usize) -> Option<i32> {
    if count == 0 {
        Some(1)
    } else {
        None
    }
}

fn pos_exact_progress(current: Option<Archetype>, required: Archetype) -> Option<i32> {
    match current {
        Some(found) if found == required => Some(1),
        Some(_) => None,
        None => Some(0),
    }
}

fn pos_one_of_progress(current: Option<Archetype>, allowed: &[Archetype]) -> Option<i32> {
    match current {
        Some(found) if allowed.contains(&found) => Some(1),
        Some(_) => None,
        None => Some(0),
    }
}

fn either_pos_exact_progress(
    left: Option<Archetype>,
    right: Option<Archetype>,
    required: Archetype,
) -> Option<i32> {
    if left == Some(required) || right == Some(required) {
        return Some(1);
    }
    if left.is_some() && right.is_some() {
        None
    } else {
        Some(0)
    }
}

fn version_tier_for(champion_id: u8, version_tiers: &HashMap<u8, VersionTier>) -> VersionTier {
    version_tiers
        .get(&champion_id)
        .copied()
        .unwrap_or(VersionTier::T2)
}

fn mastery_for_player(player: &PlayerChampionPool, champion_id: u8) -> MasteryTier {
    player
        .masteries
        .get(&champion_id)
        .copied()
        .unwrap_or(MasteryTier::B)
}

fn pick_version_score(mastery_tier: MasteryTier, version_tier: VersionTier) -> i32 {
    if mastery_tier == MasteryTier::SS && version_tier == VersionTier::T3 {
        0
    } else {
        i32::from(version_tier.modifier())
    }
}

fn mastery_modifier(mastery_tier: MasteryTier, traits: &[TraitType]) -> f64 {
    match mastery_tier {
        MasteryTier::SS => {
            if traits.contains(&TraitType::LoneWolf) {
                4.0
            } else {
                3.0
            }
        }
        MasteryTier::S => 1.5,
        MasteryTier::A => 0.0,
        MasteryTier::B => {
            if traits.contains(&TraitType::Adaptable) {
                -1.0
            } else {
                -3.0
            }
        }
    }
}

fn version_modifier_for_player(mastery_tier: MasteryTier, version_tier: VersionTier) -> f64 {
    if mastery_tier == MasteryTier::SS && version_tier == VersionTier::T3 {
        0.0
    } else {
        match version_tier {
            VersionTier::T1 => 2.0,
            VersionTier::T2 => 0.0,
            VersionTier::T3 => -2.0,
        }
    }
}

fn calculate_team_bp_modifiers(
    team_picks: &[PickEntry],
    version_tiers: &HashMap<u8, VersionTier>,
    team_comp: Option<CompType>,
    enemy_comp: Option<CompType>,
    meta_type: MetaType,
) -> HashMap<u64, f64> {
    let mut result = HashMap::with_capacity(team_picks.len());

    let comp_bonus = calculate_team_comp_bonus(team_comp, meta_type);
    let counter_mod = match (team_comp, enemy_comp) {
        (Some(my_comp), Some(enemy_comp_type)) => {
            calculate_counter_modifier(&my_comp, &enemy_comp_type)
        }
        _ => 0.0,
    };

    for pick in team_picks {
        let version_tier = version_tier_for(pick.champion_id, version_tiers);
        let personal_mastery_mod = mastery_modifier(pick.mastery_tier, &pick.traits);
        let personal_version_mod = version_modifier_for_player(pick.mastery_tier, version_tier);
        let total_modifier = personal_mastery_mod + personal_version_mod + comp_bonus + counter_mod;
        result.insert(pick.player_id, total_modifier);
    }

    result
}

fn calculate_team_comp_bonus(team_comp: Option<CompType>, meta_type: MetaType) -> f64 {
    match team_comp {
        Some(comp_type) => {
            let base_bonus = comp_type.difficulty_bonus();
            let meta_bonus = if comp_type.is_meta_favored(meta_type) {
                2.5
            } else {
                0.0
            };
            base_bonus + meta_bonus
        }
        None => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comp_detection_teamfight() {
        let picks = vec![
            (Position::Top, Archetype::Teamfight),
            (Position::Jug, Archetype::Teamfight),
            (Position::Mid, Archetype::Teamfight),
            (Position::Adc, Archetype::Aggressive),
            (Position::Sup, Archetype::Utility),
        ];

        assert_eq!(detect_comp(&picks), Some(CompType::Teamfight));
    }

    #[test]
    fn test_comp_detection_flex() {
        let picks = vec![
            (Position::Top, Archetype::Splitpush),
            (Position::Jug, Archetype::Aggressive),
            (Position::Mid, Archetype::Utility),
            (Position::Adc, Archetype::Scaling),
            (Position::Sup, Archetype::Teamfight),
        ];

        assert_eq!(detect_comp(&picks), Some(CompType::Flex));
    }

    #[test]
    fn test_comp_detection_none() {
        let picks = vec![
            (Position::Top, Archetype::Utility),
            (Position::Jug, Archetype::Splitpush),
            (Position::Mid, Archetype::Utility),
            (Position::Adc, Archetype::Aggressive),
            (Position::Sup, Archetype::Scaling),
        ];

        assert_eq!(detect_comp(&picks), None);
    }

    #[test]
    fn test_hard_counter_symmetry() {
        for comp in CompType::all() {
            let hard_outgoing = hard_counter_pairs()
                .iter()
                .filter(|(attacker, _)| attacker == comp)
                .count();
            let hard_incoming = hard_counter_pairs()
                .iter()
                .filter(|(_, victim)| victim == comp)
                .count();

            assert_eq!(
                hard_outgoing, 1,
                "{comp:?} should hard-counter exactly 1 comp"
            );
            assert_eq!(
                hard_incoming, 1,
                "{comp:?} should be hard-countered by exactly 1 comp"
            );
        }
    }

    #[test]
    fn test_soft_counter_symmetry() {
        for comp in CompType::all() {
            let soft_outgoing = soft_counter_pairs()
                .iter()
                .filter(|(attacker, _)| attacker == comp)
                .count();
            let soft_incoming = soft_counter_pairs()
                .iter()
                .filter(|(_, victim)| victim == comp)
                .count();

            assert_eq!(
                soft_outgoing, 2,
                "{comp:?} should soft-counter exactly 2 comps"
            );
            assert_eq!(
                soft_incoming, 2,
                "{comp:?} should be soft-countered by exactly 2 comps"
            );
        }
    }

    #[test]
    fn test_counter_modifier_hard() {
        let modifier = calculate_counter_modifier(&CompType::Rush, &CompType::Control);
        assert!((modifier - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_counter_modifier_none() {
        let modifier = calculate_counter_modifier(&CompType::Rush, &CompType::Teamfight);
        assert!((modifier - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_bp_modifier_ss_ignores_t3() {
        let version_mod = version_modifier_for_player(MasteryTier::SS, VersionTier::T3);
        assert!((version_mod - 0.0).abs() < f64::EPSILON);
    }
}
