use super::champion::{
    self, get_champions_by_position, Archetype, Champion, MasteryTier, VersionTier,
};
use super::meta_engine::MetaType;
use super::traits::TraitType;
use crate::models::player::Position;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;
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

    pub fn from_id(id: &str) -> Option<CompType> {
        match id {
            "Rush" => Some(CompType::Rush),
            "PickOff" => Some(CompType::PickOff),
            "AllIn" => Some(CompType::AllIn),
            "MidJungle" => Some(CompType::MidJungle),
            "TopJungle" => Some(CompType::TopJungle),
            "Protect" => Some(CompType::Protect),
            "Fortress" => Some(CompType::Fortress),
            "UtilityComp" => Some(CompType::UtilityComp),
            "Stall" => Some(CompType::Stall),
            "BotLane" => Some(CompType::BotLane),
            "Teamfight" => Some(CompType::Teamfight),
            "Dive" => Some(CompType::Dive),
            "Skirmish" => Some(CompType::Skirmish),
            "DualCarry" => Some(CompType::DualCarry),
            "Flex" => Some(CompType::Flex),
            "Splitpush" => Some(CompType::Splitpush),
            "SideLane" => Some(CompType::SideLane),
            "Control" => Some(CompType::Control),
            "TripleThreat" => Some(CompType::TripleThreat),
            "LateGame" => Some(CompType::LateGame),
            _ => None,
        }
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

#[derive(Debug, Clone, Default)]
pub struct SeriesContext {
    pub prev_winner_picks: Vec<u8>,
    pub prev_loser_side: Option<TeamSide>,
    pub prev_home_comp: Option<CompType>,
    pub prev_away_comp: Option<CompType>,
    pub home_score: u8,
    pub away_score: u8,
    pub game_number: u8,
    pub wins_needed: u8,
}

#[derive(Debug, Clone, Copy)]
struct DraftPlan {
    primary_comp: CompType,
    backup_comp: CompType,
    switched_to_backup: bool,
}

impl DraftPlan {
    fn active_comp(&self) -> CompType {
        if self.switched_to_backup {
            self.backup_comp
        } else {
            self.primary_comp
        }
    }
}

#[derive(Debug)]
struct DraftState {
    banned: HashSet<u8>,
    picked: HashSet<u8>,
    home_picks: Vec<PickEntry>,
    away_picks: Vec<PickEntry>,
    phase: u8,
}

impl DraftState {
    fn new() -> Self {
        Self {
            banned: HashSet::new(),
            picked: HashSet::new(),
            home_picks: Vec::with_capacity(5),
            away_picks: Vec::with_capacity(5),
            phase: 1,
        }
    }
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
pub struct DraftNarrative {
    pub home_entries: Vec<NarrativeEntry>,
    pub away_entries: Vec<NarrativeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEntry {
    pub phase: String,
    pub message: String,
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
    pub narrative: Option<DraftNarrative>,
}

#[derive(Debug, Clone)]
pub struct PlayerChampionPool {
    pub player_id: u64,
    pub position: Position,
    pub ability: u8,
    pub masteries: HashMap<u8, MasteryTier>,
    pub games_played: HashMap<u8, u32>,
    pub games_won: HashMap<u8, u32>,
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
        home_team_comp_history: &[(CompType, u32)],
        away_team_comp_history: &[(CompType, u32)],
        series_ctx: Option<&SeriesContext>,
    ) -> DraftResult {
        let mut bans = Vec::with_capacity(BAN_ORDER.len());
        let mut draft_state = DraftState::new();
        let phase_one_pick_count = BAN_ORDER.iter().filter(|(_, phase)| *phase == 1).count();

        let mut home_plan = plan_comp(
            home_players,
            version_tiers,
            home_team_comp_history,
            meta_type,
            series_ctx,
            TeamSide::Home,
            rng,
        );
        let mut away_plan = plan_comp(
            away_players,
            version_tiers,
            away_team_comp_history,
            meta_type,
            series_ctx,
            TeamSide::Away,
            rng,
        );

        let mut narrative = DraftNarrative {
            home_entries: Vec::new(),
            away_entries: Vec::new(),
        };

        narrative.home_entries.push(NarrativeEntry {
            phase: "plan".to_string(),
            message: format!(
                "规划主体系: {}，备选: {}",
                comp_display_name(home_plan.primary_comp),
                comp_display_name(home_plan.backup_comp)
            ),
        });
        narrative.away_entries.push(NarrativeEntry {
            phase: "plan".to_string(),
            message: format!(
                "规划主体系: {}，备选: {}",
                comp_display_name(away_plan.primary_comp),
                comp_display_name(away_plan.backup_comp)
            ),
        });

        for (team_side, ban_phase) in BAN_ORDER {
            if ban_phase != 1 {
                continue;
            }

            let (target_players, target_comp_history, target_picks, my_plan, my_picks) =
                match team_side {
                    TeamSide::Home => (
                        away_players,
                        away_team_comp_history,
                        draft_state.away_picks.as_slice(),
                        &home_plan,
                        draft_state.home_picks.as_slice(),
                    ),
                    TeamSide::Away => (
                        home_players,
                        home_team_comp_history,
                        draft_state.home_picks.as_slice(),
                        &away_plan,
                        draft_state.away_picks.as_slice(),
                    ),
                };

            if let Some(champion_id) = select_best_ban(
                target_players,
                version_tiers,
                &draft_state.banned,
                &draft_state.picked,
                rng,
                target_comp_history,
                ban_phase,
                target_picks,
                team_side,
                series_ctx,
                Some(my_plan),
                my_picks,
            ) {
                draft_state.banned.insert(champion_id);
                bans.push(BanEntry {
                    team_side,
                    champion_id,
                    ban_phase,
                });

                let ban_msg = comp_history_ban_narrative(champion_id, target_comp_history)
                    .unwrap_or_else(|| {
                        generate_ban_narrative(
                            champion_id,
                            ban_phase,
                            target_players,
                            series_ctx,
                            team_side,
                            my_plan,
                            my_picks,
                            target_picks,
                        )
                    });
                match team_side {
                    TeamSide::Home => narrative.home_entries.push(NarrativeEntry {
                        phase: "ban_phase1".to_string(),
                        message: ban_msg,
                    }),
                    TeamSide::Away => narrative.away_entries.push(NarrativeEntry {
                        phase: "ban_phase1".to_string(),
                        message: ban_msg,
                    }),
                }
            }
        }

        let mut pick_index = 0usize;
        for team_side in PICK_ORDER.iter().copied().take(phase_one_pick_count) {
            let (plan, players, opponent_players, team_picks, opponent_picks) = match team_side {
                TeamSide::Home => (
                    &home_plan,
                    home_players,
                    away_players,
                    &draft_state.home_picks,
                    &draft_state.away_picks,
                ),
                TeamSide::Away => (
                    &away_plan,
                    away_players,
                    home_players,
                    &draft_state.away_picks,
                    &draft_state.home_picks,
                ),
            };
            let next_pick = select_best_pick(
                players,
                opponent_players,
                team_picks,
                opponent_picks,
                &draft_state.banned,
                &draft_state.picked,
                version_tiers,
                rng,
                plan,
                pick_index,
            );

            if let Some(pick) = next_pick {
                if draft_state.picked.insert(pick.champion_id) {
                    let pick_msg = generate_pick_narrative(
                        &pick,
                        plan,
                        opponent_picks,
                        opponent_players,
                        pick_index,
                        version_tiers,
                    );
                    match team_side {
                        TeamSide::Home => {
                            draft_state.home_picks.push(pick);
                            narrative.home_entries.push(NarrativeEntry {
                                phase: "pick".to_string(),
                                message: pick_msg,
                            });
                        }
                        TeamSide::Away => {
                            draft_state.away_picks.push(pick);
                            narrative.away_entries.push(NarrativeEntry {
                                phase: "pick".to_string(),
                                message: pick_msg,
                            });
                        }
                    }
                }
            }

            pick_index += 1;
        }

        if should_switch_to_backup(&home_plan, &draft_state.home_picks, &draft_state.away_picks) {
            home_plan.switched_to_backup = true;
        }
        if should_switch_to_backup(&away_plan, &draft_state.away_picks, &draft_state.home_picks) {
            away_plan.switched_to_backup = true;
        }

        if home_plan.switched_to_backup {
            if let Some(opponent_direction) =
                detect_partial_comp_direction(&picks_to_comp_view(&draft_state.away_picks))
            {
                narrative.home_entries.push(NarrativeEntry {
                    phase: "switch".to_string(),
                    message: format!(
                        "检测到对手可能打{}方向，切换到备选体系: {}",
                        comp_display_name(opponent_direction),
                        comp_display_name(home_plan.backup_comp)
                    ),
                });
            }
        }
        if away_plan.switched_to_backup {
            if let Some(opponent_direction) =
                detect_partial_comp_direction(&picks_to_comp_view(&draft_state.home_picks))
            {
                narrative.away_entries.push(NarrativeEntry {
                    phase: "switch".to_string(),
                    message: format!(
                        "检测到对手可能打{}方向，切换到备选体系: {}",
                        comp_display_name(opponent_direction),
                        comp_display_name(away_plan.backup_comp)
                    ),
                });
            }
        }

        draft_state.phase = 2;

        for (team_side, ban_phase) in BAN_ORDER {
            if ban_phase != draft_state.phase {
                continue;
            }

            let (target_players, target_comp_history, target_picks, my_plan, my_picks) =
                match team_side {
                    TeamSide::Home => (
                        away_players,
                        away_team_comp_history,
                        draft_state.away_picks.as_slice(),
                        &home_plan,
                        draft_state.home_picks.as_slice(),
                    ),
                    TeamSide::Away => (
                        home_players,
                        home_team_comp_history,
                        draft_state.home_picks.as_slice(),
                        &away_plan,
                        draft_state.away_picks.as_slice(),
                    ),
                };

            if let Some(champion_id) = select_best_ban(
                target_players,
                version_tiers,
                &draft_state.banned,
                &draft_state.picked,
                rng,
                target_comp_history,
                ban_phase,
                target_picks,
                team_side,
                series_ctx,
                Some(my_plan),
                my_picks,
            ) {
                draft_state.banned.insert(champion_id);
                bans.push(BanEntry {
                    team_side,
                    champion_id,
                    ban_phase,
                });

                let ban_msg = comp_history_ban_narrative(champion_id, target_comp_history)
                    .unwrap_or_else(|| {
                        generate_ban_narrative(
                            champion_id,
                            ban_phase,
                            target_players,
                            series_ctx,
                            team_side,
                            my_plan,
                            my_picks,
                            target_picks,
                        )
                    });
                match team_side {
                    TeamSide::Home => narrative.home_entries.push(NarrativeEntry {
                        phase: "ban_phase2".to_string(),
                        message: ban_msg,
                    }),
                    TeamSide::Away => narrative.away_entries.push(NarrativeEntry {
                        phase: "ban_phase2".to_string(),
                        message: ban_msg,
                    }),
                }
            }
        }

        for team_side in PICK_ORDER.iter().copied().skip(phase_one_pick_count) {
            let is_home = matches!(team_side, TeamSide::Home);
            if is_home {
                if !home_plan.switched_to_backup
                    && should_switch_to_backup(
                        &home_plan,
                        &draft_state.home_picks,
                        &draft_state.away_picks,
                    )
                {
                    home_plan.switched_to_backup = true;
                    if let Some(opp_dir) =
                        detect_partial_comp_direction(&picks_to_comp_view(&draft_state.away_picks))
                    {
                        narrative.home_entries.push(NarrativeEntry {
                            phase: "switch".to_string(),
                            message: format!(
                                "Phase 2检测到对手{}方向加强，紧急切换到备选体系: {}",
                                comp_display_name(opp_dir),
                                comp_display_name(home_plan.backup_comp)
                            ),
                        });
                    }
                }
            } else if !away_plan.switched_to_backup
                && should_switch_to_backup(
                    &away_plan,
                    &draft_state.away_picks,
                    &draft_state.home_picks,
                )
            {
                away_plan.switched_to_backup = true;
                if let Some(opp_dir) =
                    detect_partial_comp_direction(&picks_to_comp_view(&draft_state.home_picks))
                {
                    narrative.away_entries.push(NarrativeEntry {
                        phase: "switch".to_string(),
                        message: format!(
                            "Phase 2检测到对手{}方向加强，紧急切换到备选体系: {}",
                            comp_display_name(opp_dir),
                            comp_display_name(away_plan.backup_comp)
                        ),
                    });
                }
            }

            let (plan, players, opponent_players, team_picks, opponent_picks) = match team_side {
                TeamSide::Home => (
                    &home_plan,
                    home_players,
                    away_players,
                    &draft_state.home_picks,
                    &draft_state.away_picks,
                ),
                TeamSide::Away => (
                    &away_plan,
                    away_players,
                    home_players,
                    &draft_state.away_picks,
                    &draft_state.home_picks,
                ),
            };
            let next_pick = select_best_pick(
                players,
                opponent_players,
                team_picks,
                opponent_picks,
                &draft_state.banned,
                &draft_state.picked,
                version_tiers,
                rng,
                plan,
                pick_index,
            );

            if let Some(pick) = next_pick {
                if draft_state.picked.insert(pick.champion_id) {
                    let pick_msg = generate_pick_narrative(
                        &pick,
                        plan,
                        opponent_picks,
                        opponent_players,
                        pick_index,
                        version_tiers,
                    );
                    match team_side {
                        TeamSide::Home => {
                            draft_state.home_picks.push(pick);
                            narrative.home_entries.push(NarrativeEntry {
                                phase: "pick".to_string(),
                                message: pick_msg,
                            });
                        }
                        TeamSide::Away => {
                            draft_state.away_picks.push(pick);
                            narrative.away_entries.push(NarrativeEntry {
                                phase: "pick".to_string(),
                                message: pick_msg,
                            });
                        }
                    }
                }
            }

            pick_index += 1;
        }

        fill_missing_positions(
            home_players,
            &mut draft_state.home_picks,
            &draft_state.banned,
            &mut draft_state.picked,
            rng,
        );
        fill_missing_positions(
            away_players,
            &mut draft_state.away_picks,
            &draft_state.banned,
            &mut draft_state.picked,
            rng,
        );

        let home_comp =
            detect_comp_with_plan(&picks_to_comp_view(&draft_state.home_picks), &home_plan);
        let away_comp =
            detect_comp_with_plan(&picks_to_comp_view(&draft_state.away_picks), &away_plan);

        let home_bp_modifiers = calculate_team_bp_modifiers(
            &draft_state.home_picks,
            version_tiers,
            home_comp,
            away_comp,
            meta_type,
        );
        let away_bp_modifiers = calculate_team_bp_modifiers(
            &draft_state.away_picks,
            version_tiers,
            away_comp,
            home_comp,
            meta_type,
        );

        DraftResult {
            bans,
            home_picks: draft_state.home_picks,
            away_picks: draft_state.away_picks,
            home_comp,
            away_comp,
            home_bp_modifiers,
            away_bp_modifiers,
            narrative: Some(narrative),
        }
    }
}

fn comp_display_name(comp: CompType) -> &'static str {
    match comp {
        CompType::Rush => "速推",
        CompType::PickOff => "抓单",
        CompType::AllIn => "莽夫",
        CompType::MidJungle => "中野联动",
        CompType::TopJungle => "上野联动",
        CompType::Protect => "保C",
        CompType::Fortress => "铁桶阵",
        CompType::UtilityComp => "功能流",
        CompType::Stall => "龟缩",
        CompType::BotLane => "下路统治",
        CompType::Teamfight => "团战",
        CompType::Dive => "开团",
        CompType::Skirmish => "小规模团战",
        CompType::DualCarry => "双C",
        CompType::Flex => "全能",
        CompType::Splitpush => "分推",
        CompType::SideLane => "4-1分带",
        CompType::Control => "运营",
        CompType::TripleThreat => "三线施压",
        CompType::LateGame => "后期发育",
    }
}

fn comp_history_ban_narrative(
    champion_id: u8,
    opponent_comp_history: &[(CompType, u32)],
) -> Option<String> {
    let champion = champion::get_champion(champion_id)?;

    let mut sorted_history: Vec<(CompType, u32)> = opponent_comp_history.to_vec();
    sorted_history.sort_by(|a, b| b.1.cmp(&a.1));

    for (comp, usage_count) in sorted_history.into_iter().take(2) {
        if usage_count < 3 {
            continue;
        }
        if !comp_core_positions(comp).contains(&champion.position) {
            continue;
        }
        if !comp.core_archetypes().contains(&champion.archetype) {
            continue;
        }

        return Some(format!(
            "Ban {}，预判对手常用{}体系",
            champion.name_cn,
            comp_display_name(comp)
        ));
    }

    None
}

fn generate_ban_narrative(
    champion_id: u8,
    ban_phase: u8,
    opponent_players: &[PlayerChampionPool],
    series_ctx: Option<&SeriesContext>,
    banning_team_side: TeamSide,
    my_plan: &DraftPlan,
    my_picks: &[PickEntry],
    opponent_picks: &[PickEntry],
) -> String {
    let champion_name = champion::get_champion(champion_id)
        .map(|c| c.name_cn)
        .unwrap_or("未知");

    if ban_phase == 2 {
        let self_prot = self_protection_ban_bonus(champion_id, my_plan, my_picks, opponent_players);
        if self_prot >= 5 {
            return format!(
                "Ban {}，保护己方{}体系不被克制",
                champion_name,
                comp_display_name(my_plan.active_comp())
            );
        }
    }

    if series_revenge_bonus(champion_id, banning_team_side, series_ctx) > 0 {
        return format!("Ban {}，针对对手上局获胜阵容", champion_name);
    }

    if let Some(ctx) = series_ctx {
        if ctx.prev_winner_picks.contains(&champion_id) {
            return format!("Ban {}，针对对手上局获胜阵容", champion_name);
        }
    }

    for player in opponent_players {
        let mastery = mastery_for_player(player, champion_id);
        if mastery == MasteryTier::SS {
            return format!(
                "Ban {}，对手{:?}位有SS级熟练度",
                champion_name, player.position
            );
        }
    }

    if ban_phase == 2 {
        if let Some(opp_direction) =
            detect_partial_comp_direction(&picks_to_comp_view(opponent_picks))
        {
            let core_positions = comp_core_positions(opp_direction);
            if let Some(champ) = champion::get_champion(champion_id) {
                if core_positions.contains(&champ.position) {
                    return format!(
                        "Ban {}，破坏对手{}体系核心位",
                        champion_name,
                        comp_display_name(opp_direction)
                    );
                }
            }
        }
    }

    format!("Ban {}，削弱对手英雄池", champion_name)
}

fn generate_pick_narrative(
    pick: &PickEntry,
    plan: &DraftPlan,
    opponent_picks: &[PickEntry],
    opponent_players: &[PlayerChampionPool],
    pick_index: usize,
    version_tiers: &HashMap<u8, VersionTier>,
) -> String {
    let champion_name = champion::get_champion(pick.champion_id)
        .map(|c| c.name_cn)
        .unwrap_or("未知");
    let position_name = format!("{:?}", pick.position);

    let mut reasons: Vec<&str> = Vec::new();

    if let Some(champ) = champion::get_champion(pick.champion_id) {
        let target_comp = plan.active_comp();
        if target_comp.core_archetypes().contains(&champ.archetype)
            && comp_core_positions(target_comp).contains(&champ.position)
        {
            reasons.push("体系核心");
        }
    }

    let version_tier = version_tier_for(pick.champion_id, version_tiers);
    if matches!(version_tier, VersionTier::T1) {
        reasons.push("版本强势");
    }

    if pick.mastery_tier == MasteryTier::SS {
        reasons.push("签名英雄");
    } else if pick.mastery_tier == MasteryTier::S {
        reasons.push("高熟练度");
    }

    if opponent_picks
        .iter()
        .any(|entry| entry.position == pick.position)
    {
        if let Some(my_champ) = champion::get_champion(pick.champion_id) {
            let matchup = lane_matchup_bonus(my_champ.position, my_champ.archetype, opponent_picks);
            if matchup >= 2 {
                reasons.push(if pick_index >= 4 {
                    "后手对线克制"
                } else {
                    "对线克制"
                });
            }
        }
    }

    if pick_index < 4 {
        let contested = opponent_players.iter().any(|player| {
            player.position == pick.position
                && matches!(
                    mastery_for_player(player, pick.champion_id),
                    MasteryTier::SS | MasteryTier::S
                )
        });
        if contested && matches!(version_tier, VersionTier::T1 | VersionTier::T2) {
            reasons.push("抢选热门");
        }
    }

    let denial = denial_pick_bonus(
        pick.champion_id,
        pick.position,
        version_tier,
        opponent_players,
    );
    if denial >= 5 {
        reasons.push("拒绝对手");
    }

    let reason_str = if reasons.is_empty() {
        "综合评分最优".to_string()
    } else {
        reasons.join("、")
    };

    format!(
        "{}位选{}（{}），{}",
        position_name,
        champion_name,
        match pick.mastery_tier {
            MasteryTier::SS => "SS",
            MasteryTier::S => "S",
            MasteryTier::A => "A",
            MasteryTier::B => "B",
        },
        reason_str
    )
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

fn detect_comp_with_plan(picks: &[(Position, Archetype)], plan: &DraftPlan) -> Option<CompType> {
    let snapshot = CompSnapshot::from_picks(picks);
    if snapshot.total != POSITIONS.len() {
        return None;
    }

    let planned = plan.active_comp();
    if comp_matches_full(planned, &snapshot) {
        return Some(planned);
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
    picked_champions: &HashSet<u8>,
    rng: &mut StdRng,
    opponent_comp_history: &[(CompType, u32)],
    ban_phase: u8,
    opponent_picks: &[PickEntry],
    banning_team_side: TeamSide,
    series_ctx: Option<&SeriesContext>,
    my_plan: Option<&DraftPlan>,
    my_picks: &[PickEntry],
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
    candidates.retain(|champion_id| {
        !banned_champions.contains(champion_id) && !picked_champions.contains(champion_id)
    });

    if candidates.is_empty() {
        candidates = champion::CHAMPIONS
            .iter()
            .map(|champion| champion.id)
            .filter(|champion_id| {
                !banned_champions.contains(champion_id) && !picked_champions.contains(champion_id)
            })
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
                calculate_ban_score(
                    champion_id,
                    opponent_players,
                    version_tiers,
                    rng,
                    opponent_comp_history,
                    ban_phase,
                    opponent_picks,
                    banning_team_side,
                    series_ctx,
                    my_plan,
                    my_picks,
                ),
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
    rng: &mut StdRng,
    opponent_comp_history: &[(CompType, u32)],
    ban_phase: u8,
    opponent_picks: &[PickEntry],
    banning_team_side: TeamSide,
    series_ctx: Option<&SeriesContext>,
    my_plan: Option<&DraftPlan>,
    my_picks: &[PickEntry],
) -> i32 {
    let mut threat_score = 0i32;
    let mut has_ss_master = false;
    let mut usage_score = 0i32;
    let mut comp_target_score = 0i32;
    let core_positions = top_comp_core_positions(opponent_comp_history);

    for player in opponent_players {
        let mastery = mastery_for_player(player, champion_id);
        let weighted_threat =
            (f64::from(mastery.pick_score()) * ability_factor(player.ability)).round() as i32;
        threat_score = threat_score.max(weighted_threat);
        if mastery == MasteryTier::SS {
            has_ss_master = true;
        }

        let gp = player.games_played.get(&champion_id).copied().unwrap_or(0);
        let gw = player.games_won.get(&champion_id).copied().unwrap_or(0);
        let player_usage = games_played_ban_bonus(gp);
        let player_winrate = winrate_ban_bonus(gp, gw);
        usage_score = usage_score.max(player_usage + player_winrate);

        if core_positions.contains(&player.position) {
            comp_target_score = comp_target_score.max(core_position_mastery_bonus(mastery));
        }
    }

    let version_score = ban_version_score(version_tier_for(champion_id, version_tiers));
    let disruption_score = if has_ss_master { 5 } else { 0 };
    let phase2_targeting_score = phase2_targeting_bonus(champion_id, ban_phase, opponent_picks);
    let series_revenge_score = series_revenge_bonus(champion_id, banning_team_side, series_ctx);
    let self_protection_score = if ban_phase == 2 {
        my_plan
            .map(|plan| self_protection_ban_bonus(champion_id, plan, my_picks, opponent_players))
            .unwrap_or(0)
    } else {
        0
    };
    let comp_history_score = comp_history_ban_bonus(champion_id, opponent_comp_history);
    let random_noise = rng.gen_range(0..3) as i32;

    threat_score
        + version_score
        + disruption_score
        + usage_score
        + comp_target_score
        + phase2_targeting_score
        + series_revenge_score
        + self_protection_score
        + comp_history_score
        + random_noise
}

fn ability_factor(ability: u8) -> f64 {
    1.0 + f64::from(ability.saturating_sub(50)) / 100.0
}

fn ban_version_score(version_tier: VersionTier) -> i32 {
    match version_tier {
        VersionTier::T1 => 6,
        VersionTier::T2 => 3,
        VersionTier::T3 => 0,
        VersionTier::T4 => -3,
        VersionTier::T5 => -6,
    }
}

fn phase2_targeting_bonus(champion_id: u8, ban_phase: u8, opponent_picks: &[PickEntry]) -> i32 {
    if ban_phase != 2 || opponent_picks.is_empty() {
        return 0;
    }

    let Some(candidate) = champion::get_champion(champion_id) else {
        return 0;
    };

    if opponent_picks
        .iter()
        .any(|pick| pick.position == candidate.position)
    {
        return 0;
    }

    let opponent_view = picks_to_comp_view(opponent_picks);
    let Some(opponent_direction) = detect_partial_comp_direction(&opponent_view) else {
        return 0;
    };

    if pick_helps_specific_comp(
        &opponent_view,
        (candidate.position, candidate.archetype),
        opponent_direction,
    ) {
        8
    } else {
        0
    }
}

fn series_revenge_bonus(
    champion_id: u8,
    banning_team_side: TeamSide,
    series_ctx: Option<&SeriesContext>,
) -> i32 {
    match series_ctx {
        Some(ctx)
            if ctx.prev_loser_side == Some(banning_team_side)
                && ctx.prev_winner_picks.contains(&champion_id) =>
        {
            12
        }
        _ => 0,
    }
}

fn self_protection_ban_bonus(
    champion_id: u8,
    my_plan: &DraftPlan,
    my_picks: &[PickEntry],
    opponent_players: &[PlayerChampionPool],
) -> i32 {
    let my_comp = my_plan.active_comp();
    let my_comp_view = picks_to_comp_view(my_picks);
    let my_direction = detect_partial_comp_direction(&my_comp_view).unwrap_or(my_comp);

    let counter_comps: Vec<CompType> = HARD_COUNTERS
        .iter()
        .filter(|(_, victim)| *victim == my_direction)
        .map(|(attacker, _)| *attacker)
        .collect();

    let Some(champion) = champion::get_champion(champion_id) else {
        return 0;
    };

    let mut bonus = 0i32;
    for counter_comp in &counter_comps {
        let core_positions = comp_core_positions(*counter_comp);
        if !core_positions.contains(&champion.position) {
            continue;
        }
        if !counter_comp.core_archetypes().contains(&champion.archetype) {
            continue;
        }

        for opponent in opponent_players {
            if opponent.position != champion.position {
                continue;
            }

            let mastery = mastery_for_player(opponent, champion_id);
            if matches!(mastery, MasteryTier::SS | MasteryTier::S) {
                bonus = bonus.max(10);
            } else if mastery == MasteryTier::A {
                bonus = bonus.max(5);
            }
        }
    }

    bonus
}

fn comp_history_ban_bonus(champion_id: u8, opponent_comp_history: &[(CompType, u32)]) -> i32 {
    let Some(champion) = champion::get_champion(champion_id) else {
        return 0;
    };

    let mut sorted_history: Vec<(CompType, u32)> = opponent_comp_history.to_vec();
    sorted_history.sort_by(|a, b| b.1.cmp(&a.1));

    let mut bonus = 0i32;
    for (comp, usage_count) in sorted_history.iter().take(2) {
        let core_positions = comp_core_positions(*comp);
        let core_archetypes = comp.core_archetypes();

        if !core_positions.contains(&champion.position) {
            continue;
        }
        if !core_archetypes.contains(&champion.archetype) {
            continue;
        }

        let usage_bonus = match usage_count {
            10.. => 8,
            5..=9 => 5,
            3..=4 => 3,
            _ => 0,
        };
        bonus = bonus.max(usage_bonus);
    }

    bonus
}

fn games_played_ban_bonus(games_played: u32) -> i32 {
    match games_played {
        50.. => 12,
        30..=49 => 10,
        20..=29 => 8,
        10..=19 => 6,
        5..=9 => 4,
        3..=4 => 2,
        _ => 0,
    }
}

fn winrate_ban_bonus(games_played: u32, games_won: u32) -> i32 {
    if games_played < 5 {
        return 0;
    }
    let winrate = games_won as f64 / games_played as f64;
    match winrate {
        w if w >= 0.85 => 10,
        w if w >= 0.75 => 7,
        w if w >= 0.65 => 4,
        w if w >= 0.55 => 2,
        _ => 0,
    }
}

fn core_position_mastery_bonus(mastery: MasteryTier) -> i32 {
    match mastery {
        MasteryTier::SS => 5,
        MasteryTier::S => 4,
        MasteryTier::A => 3,
        MasteryTier::B => 0,
    }
}

fn top_comp_core_positions(opponent_comp_history: &[(CompType, u32)]) -> Vec<Position> {
    let mut sorted_history: Vec<(CompType, u32)> = opponent_comp_history
        .iter()
        .copied()
        .filter(|(_, count)| *count > 0)
        .collect();
    sorted_history.sort_by(|left, right| right.1.cmp(&left.1));

    let mut core_positions = Vec::new();
    for (comp, _) in sorted_history.into_iter().take(2) {
        for position in comp_core_positions(comp) {
            if !core_positions.contains(position) {
                core_positions.push(*position);
            }
        }
    }

    core_positions
}

fn comp_core_positions(comp: CompType) -> &'static [Position] {
    match comp {
        CompType::Rush => &[Position::Jug, Position::Mid],
        CompType::PickOff => &[Position::Jug, Position::Mid],
        CompType::AllIn => &[Position::Top, Position::Jug],
        CompType::MidJungle => &[Position::Mid, Position::Jug],
        CompType::TopJungle => &[Position::Top, Position::Jug],
        CompType::Protect => &[Position::Adc, Position::Sup],
        CompType::Fortress => &[Position::Sup, Position::Top],
        CompType::UtilityComp => &[Position::Sup, Position::Mid],
        CompType::Stall => &[Position::Adc, Position::Sup],
        CompType::BotLane => &[Position::Adc, Position::Sup],
        CompType::Teamfight => &[Position::Mid, Position::Adc],
        CompType::Dive => &[Position::Top, Position::Jug],
        CompType::Skirmish => &[Position::Jug, Position::Mid],
        CompType::DualCarry => &[Position::Mid, Position::Adc],
        CompType::Flex => &[Position::Mid, Position::Top],
        CompType::Splitpush => &[Position::Top, Position::Mid],
        CompType::SideLane => &[Position::Top, Position::Adc],
        CompType::Control => &[Position::Mid, Position::Sup],
        CompType::TripleThreat => &[Position::Top, Position::Mid, Position::Adc],
        CompType::LateGame => &[Position::Adc, Position::Mid],
    }
}

fn plan_comp(
    players: &[PlayerChampionPool],
    version_tiers: &HashMap<u8, VersionTier>,
    comp_history: &[(CompType, u32)],
    meta_type: MetaType,
    series_ctx: Option<&SeriesContext>,
    team_side: TeamSide,
    rng: &mut StdRng,
) -> DraftPlan {
    let mut scored: Vec<(CompType, i32)> = CompType::all()
        .iter()
        .copied()
        .map(|comp| {
            let player_pool_score = player_pool_fit_score(comp, players);
            let version_score = version_fit_score(comp, version_tiers);
            let meta_score = if comp.is_meta_favored(meta_type) {
                15
            } else {
                0
            };
            let history_score = history_proficiency_score(comp, comp_history);
            let rotation_score = tactical_rotation_score(comp, series_ctx, team_side);
            let random_noise = rng.gen_range(0..8) as i32;

            (
                comp,
                player_pool_score
                    + version_score
                    + meta_score
                    + history_score
                    + rotation_score
                    + random_noise,
            )
        })
        .collect();

    scored.sort_by(|left, right| right.1.cmp(&left.1));

    let fallback = CompType::all().first().copied().unwrap_or(CompType::Flex);
    let primary_comp = scored.first().map(|(comp, _)| *comp).unwrap_or(fallback);
    let backup_comp = scored.get(1).map(|(comp, _)| *comp).unwrap_or(primary_comp);

    DraftPlan {
        primary_comp,
        backup_comp,
        switched_to_backup: false,
    }
}

fn player_pool_fit_score(comp: CompType, players: &[PlayerChampionPool]) -> i32 {
    let core_archetypes = comp.core_archetypes();
    let mut raw_score = 0i32;
    let mut max_score = 0i32;

    for position in comp_core_positions(comp) {
        let Some(player) = players
            .iter()
            .find(|candidate| candidate.position == *position)
        else {
            continue;
        };

        let archetype_champions: Vec<&Champion> = get_champions_by_position(*position)
            .into_iter()
            .filter(|champion| core_archetypes.contains(&champion.archetype))
            .collect();

        if archetype_champions.is_empty() {
            continue;
        }

        max_score += (archetype_champions.len() as i32) * 4;
        for champion in archetype_champions {
            raw_score += mastery_planning_weight(mastery_for_player(player, champion.id));
        }
    }

    if max_score == 0 {
        0
    } else {
        (((raw_score as f64 / max_score as f64) * 40.0).round() as i32).clamp(0, 40)
    }
}

fn mastery_planning_weight(mastery: MasteryTier) -> i32 {
    match mastery {
        MasteryTier::SS => 4,
        MasteryTier::S => 2,
        MasteryTier::A => 1,
        MasteryTier::B => 0,
    }
}

fn version_fit_score(comp: CompType, version_tiers: &HashMap<u8, VersionTier>) -> i32 {
    let core_archetypes = comp.core_archetypes();
    let mut total = 0usize;
    let mut tier_one_or_two = 0usize;

    for champion in champion::CHAMPIONS
        .iter()
        .filter(|champion| core_archetypes.contains(&champion.archetype))
    {
        total += 1;
        if matches!(
            version_tier_for(champion.id, version_tiers),
            VersionTier::T1 | VersionTier::T2
        ) {
            tier_one_or_two += 1;
        }
    }

    if total == 0 {
        0
    } else {
        (((tier_one_or_two as f64 / total as f64) * 20.0).round() as i32).clamp(0, 20)
    }
}

fn history_proficiency_score(comp: CompType, comp_history: &[(CompType, u32)]) -> i32 {
    let usage = comp_history
        .iter()
        .find_map(|(history_comp, count)| {
            if *history_comp == comp {
                Some(*count)
            } else {
                None
            }
        })
        .unwrap_or(0);

    match usage {
        10.. => 15,
        5..=9 => 10,
        3..=4 => 7,
        1..=2 => 4,
        _ => 0,
    }
}

fn tactical_rotation_score(
    comp: CompType,
    series_ctx: Option<&SeriesContext>,
    team_side: TeamSide,
) -> i32 {
    let mut score = 5;

    let Some(ctx) = series_ctx else {
        return score;
    };

    let prev_winner_side = match ctx.prev_loser_side {
        Some(TeamSide::Home) => Some(TeamSide::Away),
        Some(TeamSide::Away) => Some(TeamSide::Home),
        None => None,
    };

    let my_prev_comp = match team_side {
        TeamSide::Home => ctx.prev_home_comp,
        TeamSide::Away => ctx.prev_away_comp,
    };

    let (my_score, opp_score) = match team_side {
        TeamSide::Home => (ctx.home_score, ctx.away_score),
        TeamSide::Away => (ctx.away_score, ctx.home_score),
    };

    let is_match_point = my_score + 1 >= ctx.wins_needed && ctx.wins_needed > 0;
    let is_desperate = opp_score + 1 >= ctx.wins_needed && ctx.wins_needed > 0;
    let _is_big_lead = my_score >= 2 && opp_score == 0;

    if my_prev_comp == Some(comp) {
        match prev_winner_side {
            Some(side) if side == team_side => {
                score -= 2;
            }
            Some(_) => {
                if is_desperate {
                    score -= 10;
                } else {
                    score -= 5;
                }
            }
            None => {}
        }
    }

    let opponent_side = match team_side {
        TeamSide::Home => TeamSide::Away,
        TeamSide::Away => TeamSide::Home,
    };
    let opponent_prev_comp = match team_side {
        TeamSide::Home => ctx.prev_away_comp,
        TeamSide::Away => ctx.prev_home_comp,
    };
    if prev_winner_side == Some(opponent_side) {
        if opponent_prev_comp.is_some_and(|opp_comp| {
            HARD_COUNTERS
                .iter()
                .any(|(attacker, victim)| *attacker == comp && *victim == opp_comp)
        }) {
            if is_desperate {
                score += 8;
            } else {
                score += 5;
            }
        }
    }

    if is_match_point {
        score += 3;
    }

    score.clamp(0, 15)
}

fn comp_target_synergy_score(
    target_comp: CompType,
    current_picks: &[(Position, Archetype)],
    candidate_pick: (Position, Archetype),
) -> i32 {
    let (position, archetype) = candidate_pick;
    let core_archetypes = target_comp.core_archetypes();
    let core_positions = comp_core_positions(target_comp);
    let mut score = 0;

    if core_archetypes.contains(&archetype) {
        // 体系核心 archetype 大幅加分
        score += 8;
        if core_positions.contains(&position) {
            // 核心位置 + 核心 archetype = 最高优先
            score += 10;
        }
    } else if core_positions.contains(&position) {
        // 核心位置但非核心 archetype — 惩罚，浪费了关键位
        score -= 6;
    } else {
        // 非核心位置、非核心 archetype — 轻微惩罚
        score -= 2;
    }

    if pick_conflicts_with_target_comp(target_comp, current_picks, candidate_pick) {
        score -= 5;
    }

    score
}

fn pick_conflicts_with_target_comp(
    target_comp: CompType,
    current_picks: &[(Position, Archetype)],
    candidate_pick: (Position, Archetype),
) -> bool {
    let current_snapshot = CompSnapshot::from_picks(current_picks);

    let mut next_picks = current_picks.to_vec();
    next_picks.push(candidate_pick);
    let next_snapshot = CompSnapshot::from_picks(&next_picks);

    let current_score = comp_partial_score(target_comp, &current_snapshot);
    let next_score = comp_partial_score(target_comp, &next_snapshot);

    current_score >= 0 && next_score < 0
}

fn priority_urgency_bonus(
    mastery: MasteryTier,
    version_tier: VersionTier,
    pick_index: usize,
    champion_id: u8,
    position: Position,
    opponent_players: &[PlayerChampionPool],
) -> i32 {
    if pick_index >= 4 {
        return 0;
    }

    let base = match (version_tier, mastery) {
        (VersionTier::T1, MasteryTier::SS) => 8,
        (VersionTier::T1, MasteryTier::S) => 4,
        (VersionTier::T2, MasteryTier::SS) => 3,
        _ => 0,
    };
    let decay = match pick_index {
        0..=1 => 1.0,
        2..=3 => 0.5,
        _ => 0.0,
    };
    let opponent_contest = if opponent_players.iter().any(|player| {
        player.position == position
            && matches!(
                mastery_for_player(player, champion_id),
                MasteryTier::SS | MasteryTier::S
            )
    }) {
        3
    } else {
        0
    };

    ((base as f64 * decay) as i32) + opponent_contest
}

fn archetype_preference_bonus(player: &PlayerChampionPool, champion_id: u8) -> i32 {
    let Some(champion) = champion::get_champion(champion_id) else {
        return 0;
    };

    let total_games: u32 = player.games_played.values().copied().sum();
    if total_games < 5 {
        return 0;
    }

    let archetype_games: u32 = player
        .games_played
        .iter()
        .filter_map(|(cid, count)| {
            champion::get_champion(*cid)
                .filter(|candidate| candidate.archetype == champion.archetype)
                .map(|_| *count)
        })
        .sum();
    let ratio = archetype_games as f64 / total_games as f64;

    if ratio >= 0.50 {
        3
    } else if ratio >= 0.30 {
        1
    } else {
        0
    }
}

fn signature_hero_bonus(
    player: &PlayerChampionPool,
    champion_id: u8,
    pick_index: usize,
    opponent_players: &[PlayerChampionPool],
) -> i32 {
    let mastery = mastery_for_player(player, champion_id);
    if mastery != MasteryTier::SS {
        return 0;
    }

    let gp = player.games_played.get(&champion_id).copied().unwrap_or(0);
    let gw = player.games_won.get(&champion_id).copied().unwrap_or(0);
    if gp < 15 {
        return 0;
    }
    let win_rate = if gp > 0 {
        (gw as f64 / gp as f64) * 100.0
    } else {
        0.0
    };
    if win_rate < 60.0 {
        return 0;
    }

    let base = match pick_index {
        0..=1 => 10,
        2..=3 => 5,
        _ => 2,
    };

    let contested = opponent_players.iter().any(|p| {
        p.position == player.position
            && matches!(
                mastery_for_player(p, champion_id),
                MasteryTier::SS | MasteryTier::S
            )
    });
    let contest_bonus = if contested { 4 } else { 0 };

    base + contest_bonus
}

fn pick_order_strategy_bonus(
    champion_id: u8,
    position: Position,
    pick_index: usize,
    plan: &DraftPlan,
    opponent_picks: &[PickEntry],
    current_comp_view: &[(Position, Archetype)],
    version_tier: VersionTier,
) -> i32 {
    let Some(champion) = champion::get_champion(champion_id) else {
        return 0;
    };

    let target_comp = plan.active_comp();
    let core_archetypes = target_comp.core_archetypes();
    let core_positions = comp_core_positions(target_comp);

    match pick_index {
        0..=1 => {
            let mut bonus = 0i32;
            if matches!(version_tier, VersionTier::T1) {
                bonus += 3;
            }
            if core_archetypes.contains(&champion.archetype) && !core_positions.contains(&position)
            {
                bonus += 3;
            }
            let backup_archetypes = plan.backup_comp.core_archetypes();
            if core_archetypes.contains(&champion.archetype)
                && backup_archetypes.contains(&champion.archetype)
            {
                bonus += 5;
            }
            bonus
        }
        4.. => {
            let mut bonus = 0i32;
            let matchup = lane_matchup_bonus(position, champion.archetype, opponent_picks);
            if matchup > 0 {
                bonus += matchup;
            }
            let opp_comp_view = picks_to_comp_view(opponent_picks);
            if let Some(opp_dir) = detect_partial_comp_direction(&opp_comp_view) {
                let mut projected = current_comp_view.to_vec();
                projected.push((position, champion.archetype));
                if let Some(my_dir) = detect_partial_comp_direction(&projected) {
                    if HARD_COUNTERS
                        .iter()
                        .any(|(a, v)| *a == my_dir && *v == opp_dir)
                    {
                        bonus += 6;
                    }
                }
            }
            let unfilled_core = core_positions
                .iter()
                .any(|pos| !current_comp_view.iter().any(|(p, _)| p == pos) && *pos == position);
            if unfilled_core && core_archetypes.contains(&champion.archetype) {
                bonus += 6;
            }
            bonus
        }
        _ => 0,
    }
}

fn lane_matchup_bonus(
    position: Position,
    champion_archetype: Archetype,
    opponent_picks: &[PickEntry],
) -> i32 {
    let opponent_pick = opponent_picks.iter().find(|pick| pick.position == position);
    let Some(opp_pick) = opponent_pick else {
        return 0;
    };

    let Some(opp_archetype) =
        champion::get_champion(opp_pick.champion_id).map(|champion| champion.archetype)
    else {
        return 0;
    };

    match (champion_archetype, opp_archetype) {
        (Archetype::Aggressive, Archetype::Scaling) => 3,
        (Archetype::Aggressive, Archetype::Utility) => 2,
        (Archetype::Scaling, Archetype::Aggressive) => -2,
        (Archetype::Scaling, Archetype::Splitpush) => -1,
        (Archetype::Teamfight, Archetype::Splitpush) => 2,
        (Archetype::Splitpush, Archetype::Utility) => 1,
        _ => 0,
    }
}

fn should_switch_to_backup(
    plan: &DraftPlan,
    my_picks: &[PickEntry],
    opponent_picks: &[PickEntry],
) -> bool {
    if my_picks.is_empty() {
        return false;
    }

    let opponent_direction = detect_partial_comp_direction(&picks_to_comp_view(opponent_picks));
    let Some(opponent_direction) = opponent_direction else {
        return false;
    };

    let is_hard_countered = HARD_COUNTERS
        .iter()
        .any(|(attacker, victim)| *attacker == opponent_direction && *victim == plan.primary_comp);
    if !is_hard_countered {
        return false;
    }

    let my_comp_view = picks_to_comp_view(my_picks);
    let backup_archetypes: HashSet<Archetype> =
        plan.backup_comp.core_archetypes().iter().copied().collect();
    let compatible_count = my_comp_view
        .iter()
        .filter(|(_, archetype)| backup_archetypes.contains(archetype))
        .count();

    compatible_count * 2 >= my_picks.len()
}

fn select_best_pick(
    team_players: &[PlayerChampionPool],
    opponent_players: &[PlayerChampionPool],
    team_picks: &[PickEntry],
    opponent_picks: &[PickEntry],
    banned_champions: &HashSet<u8>,
    picked_champions: &HashSet<u8>,
    version_tiers: &HashMap<u8, VersionTier>,
    rng: &mut StdRng,
    plan: &DraftPlan,
    pick_index: usize,
) -> Option<PickEntry> {
    let current_comp_view = picks_to_comp_view(team_picks);
    let opponent_comp_view = picks_to_comp_view(opponent_picks);
    let target_comp = plan.active_comp();
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
            let comp_synergy_score = comp_target_synergy_score(
                target_comp,
                &current_comp_view,
                (player.position, champion.archetype),
            );
            let priority_urgency_score = priority_urgency_bonus(
                mastery,
                version_tier,
                pick_index,
                champion.id,
                player.position,
                opponent_players,
            );
            let archetype_preference_score = archetype_preference_bonus(player, champion.id);
            let signature_hero_score =
                signature_hero_bonus(player, champion.id, pick_index, opponent_players);
            let pick_order_score = pick_order_strategy_bonus(
                champion.id,
                player.position,
                pick_index,
                plan,
                opponent_picks,
                &current_comp_view,
                version_tier,
            );
            let lane_matchup_score =
                lane_matchup_bonus(player.position, champion.archetype, opponent_picks);
            let usage_confidence_score = usage_confidence_pick_bonus(player, champion.id);
            let counter_pick_score = counter_pick_bonus(
                &current_comp_view,
                (player.position, champion.archetype),
                &opponent_comp_view,
            );
            let denial_pick_score =
                denial_pick_bonus(champion.id, player.position, version_tier, opponent_players);
            let random_noise = rng.gen_range(0..3) as i32;

            let total_score = mastery_score
                + version_score
                + comp_synergy_score
                + priority_urgency_score
                + archetype_preference_score
                + signature_hero_score
                + pick_order_score
                + lane_matchup_score
                + usage_confidence_score
                + counter_pick_score
                + denial_pick_score
                + random_noise;
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

fn detect_partial_comp_direction(picks: &[(Position, Archetype)]) -> Option<CompType> {
    if picks.is_empty() {
        return None;
    }

    let snapshot = CompSnapshot::from_picks(picks);
    let mut best_comp = None;
    let mut best_score = -1;

    for comp in CompType::detection_priority() {
        let score = comp_partial_score(*comp, &snapshot);
        if score > best_score {
            best_score = score;
            best_comp = Some(*comp);
        }
    }

    if best_score > 0 {
        best_comp
    } else {
        None
    }
}

fn pick_helps_specific_comp(
    current_picks: &[(Position, Archetype)],
    candidate_pick: (Position, Archetype),
    target_comp: CompType,
) -> bool {
    let current_snapshot = CompSnapshot::from_picks(current_picks);

    let mut next_picks = current_picks.to_vec();
    next_picks.push(candidate_pick);
    let next_snapshot = CompSnapshot::from_picks(&next_picks);

    let current_score = comp_partial_score(target_comp, &current_snapshot);
    let next_score = comp_partial_score(target_comp, &next_snapshot);

    next_score >= 0 && next_score > current_score
}

fn usage_confidence_pick_bonus(player: &PlayerChampionPool, champion_id: u8) -> i32 {
    let wins = player.games_won.get(&champion_id).copied().unwrap_or(0);
    (((wins as f64) * 0.5).floor() as i32).min(5)
}

fn counter_pick_bonus(
    current_team_comp_view: &[(Position, Archetype)],
    candidate_pick: (Position, Archetype),
    opponent_comp_view: &[(Position, Archetype)],
) -> i32 {
    let Some(opponent_direction) = detect_partial_comp_direction(opponent_comp_view) else {
        return 0;
    };

    let mut projected_team_view = current_team_comp_view.to_vec();
    projected_team_view.push(candidate_pick);

    let Some(my_direction) = detect_partial_comp_direction(&projected_team_view) else {
        return 0;
    };

    if HARD_COUNTERS
        .iter()
        .any(|(attacker, victim)| *attacker == my_direction && *victim == opponent_direction)
    {
        6
    } else if SOFT_COUNTERS
        .iter()
        .any(|(attacker, victim)| *attacker == my_direction && *victim == opponent_direction)
    {
        3
    } else {
        0
    }
}

fn denial_pick_bonus(
    champion_id: u8,
    position: Position,
    version_tier: VersionTier,
    opponent_players: &[PlayerChampionPool],
) -> i32 {
    if version_tier != VersionTier::T1 {
        return 0;
    }

    let mut score = 4;
    if opponent_players.iter().any(|player| {
        player.position == position && mastery_for_player(player, champion_id) == MasteryTier::SS
    }) {
        score += 7;
    }
    score
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
        .unwrap_or(VersionTier::T3)
}

fn mastery_for_player(player: &PlayerChampionPool, champion_id: u8) -> MasteryTier {
    player
        .masteries
        .get(&champion_id)
        .copied()
        .unwrap_or(MasteryTier::B)
}

fn pick_version_score(mastery_tier: MasteryTier, version_tier: VersionTier) -> i32 {
    if mastery_tier == MasteryTier::SS && matches!(version_tier, VersionTier::T4 | VersionTier::T5)
    {
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
    if mastery_tier == MasteryTier::SS && matches!(version_tier, VersionTier::T4 | VersionTier::T5)
    {
        0.0
    } else {
        match version_tier {
            VersionTier::T1 => 3.0,
            VersionTier::T2 => 1.0,
            VersionTier::T3 => 0.0,
            VersionTier::T4 => -1.5,
            VersionTier::T5 => -3.0,
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
