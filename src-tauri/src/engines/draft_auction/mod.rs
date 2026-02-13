//! 选秀权拍卖引擎
//!
//! 实现选秀权拍卖系统，包括：
//! - AI 卖签决策
//! - AI 竞拍决策
//! - 多轮竞拍流程
//! - 拍卖事件生成

mod ai_bidding;
mod auction_logic;
mod sell_decision;
mod wanted_requests;
mod types;
pub mod tests;

pub use types::{DraftAuctionConfig, DraftRookieInfo, TeamAuctionInfo};

use crate::models::{
    AuctionEventType, AuctionStatus, DraftPickAuction, DraftPickAuctionEvent, DraftPickWanted,
    EventImportance,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct DraftAuctionEngine {
    pub auction: DraftPickAuction,
    pub listings: Vec<crate::models::DraftPickListing>,
    pub bids: Vec<crate::models::DraftPickBid>,
    pub wanted_requests: Vec<DraftPickWanted>,
    pub events: Vec<DraftPickAuctionEvent>,
    pub teams: HashMap<u64, TeamAuctionInfo>,
    pub draft_orders: HashMap<u64, u32>,
    pub config: DraftAuctionConfig,
    pub draft_rookies: Vec<DraftRookieInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionRoundResult {
    pub round: u32,
    pub bids: Vec<crate::models::DraftPickBid>,
    pub sales: Vec<(u32, u64)>,
    pub expirations: Vec<u32>,
    pub wanted_results: Vec<DraftPickWanted>,
    pub events: Vec<DraftPickAuctionEvent>,
    pub is_auction_complete: bool,
}

impl Default for DraftAuctionEngine {
    fn default() -> Self {
        Self {
            auction: DraftPickAuction {
                id: 0,
                save_id: String::new(),
                season_id: 0,
                region_id: 0,
                status: AuctionStatus::Preparing,
                current_round: 0,
                total_rounds: 3,
                total_auctions: 0,
                successful_auctions: 0,
                total_revenue: 0,
                total_commission: 0,
                started_at: None,
                completed_at: None,
                created_at: String::new(),
            },
            listings: Vec::new(),
            bids: Vec::new(),
            wanted_requests: Vec::new(),
            events: Vec::new(),
            teams: HashMap::new(),
            draft_orders: HashMap::new(),
            config: DraftAuctionConfig::default(),
            draft_rookies: Vec::new(),
        }
    }
}

impl DraftAuctionEngine {
    pub fn new(save_id: String, season_id: u64, region_id: u64) -> Self {
        let mut engine = Self::default();
        engine.auction.save_id = save_id;
        engine.auction.season_id = season_id;
        engine.auction.region_id = region_id;
        engine.auction.total_rounds = engine.config.max_rounds;
        engine
    }

    pub fn initialize(
        &mut self,
        teams: &[crate::models::Team],
        draft_orders: &[crate::models::DraftOrder],
        roster_counts: &HashMap<u64, u32>,
        position_needs: &HashMap<u64, HashMap<String, u8>>,
    ) {
        for team in teams {
            let roster_count = roster_counts.get(&team.id).copied().unwrap_or(0);
            let needs = position_needs.get(&team.id).cloned().unwrap_or_default();

            self.teams.insert(
                team.id,
                TeamAuctionInfo {
                    team_id: team.id,
                    team_name: team.name.clone(),
                    balance: team.balance,
                    financial_status: crate::models::FinancialStatus::from_balance(team.balance),
                    roster_count,
                    position_needs: needs,
                    avg_ability: team.power_rating,
                },
            );
        }

        for order in draft_orders {
            self.draft_orders
                .insert(order.team_id, order.draft_position);
        }
    }

    fn create_event(
        &self,
        event_type: AuctionEventType,
        listing_id: Option<u64>,
        team_id: Option<u64>,
        draft_position: Option<u32>,
        amount: Option<i64>,
        headline: String,
        description: String,
        importance: EventImportance,
        round: u32,
    ) -> DraftPickAuctionEvent {
        DraftPickAuctionEvent {
            id: 0,
            save_id: self.auction.save_id.clone(),
            auction_id: self.auction.id,
            listing_id,
            event_type,
            team_id,
            team_name: team_id.and_then(|id| self.teams.get(&id).map(|t| t.team_name.clone())),
            draft_position,
            amount,
            headline,
            description,
            importance,
            round,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn get_status(&self) -> &DraftPickAuction {
        &self.auction
    }

    pub fn get_listings(&self) -> &[crate::models::DraftPickListing] {
        &self.listings
    }

    pub fn get_events(&self) -> &[DraftPickAuctionEvent] {
        &self.events
    }

    pub fn get_updated_draft_orders(&self) -> HashMap<u64, u32> {
        self.draft_orders.clone()
    }
}
