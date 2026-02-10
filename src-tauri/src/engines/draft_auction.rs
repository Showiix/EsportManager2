//! 选秀权拍卖引擎
//!
//! 实现选秀权拍卖系统，包括：
//! - AI 卖签决策
//! - AI 竞拍决策
//! - 多轮竞拍流程
//! - 拍卖事件生成

use crate::models::{
    AuctionEventType, AuctionStatus, BidStatus, DraftOrder, DraftPickAuction,
    DraftPickAuctionEvent, DraftPickBid, DraftPickListing, DraftListingStatus, EventImportance,
    FinancialStatus, Team,
    calculate_commission, calculate_seller_revenue, get_position_name, get_price_for_position,
    COMMISSION_RATE,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 选秀权拍卖引擎
pub struct DraftAuctionEngine {
    /// 当前拍卖会
    pub auction: DraftPickAuction,
    /// 所有挂牌
    pub listings: Vec<DraftPickListing>,
    /// 所有出价
    pub bids: Vec<DraftPickBid>,
    /// 生成的拍卖事件
    pub events: Vec<DraftPickAuctionEvent>,
    /// 球队信息
    pub teams: HashMap<u64, TeamAuctionInfo>,
    /// 选秀顺位（team_id -> draft_position）
    pub draft_orders: HashMap<u64, u32>,
    /// 配置
    pub config: DraftAuctionConfig,
}

/// 球队拍卖相关信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAuctionInfo {
    pub team_id: u64,
    pub team_name: String,
    pub balance: i64,
    pub financial_status: FinancialStatus,
    pub roster_count: u32,
    pub position_needs: HashMap<String, u8>, // position -> need_level (0-100)
    pub avg_ability: f64,
}

/// 拍卖引擎配置
#[derive(Debug, Clone)]
pub struct DraftAuctionConfig {
    /// 最大竞拍轮数
    pub max_rounds: u32,
    /// 联盟佣金比例
    pub commission_rate: f64,
}

impl Default for DraftAuctionConfig {
    fn default() -> Self {
        Self {
            max_rounds: 3,
            commission_rate: COMMISSION_RATE,
        }
    }
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
            events: Vec::new(),
            teams: HashMap::new(),
            draft_orders: HashMap::new(),
            config: DraftAuctionConfig::default(),
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

    /// 初始化拍卖引擎
    pub fn initialize(
        &mut self,
        teams: &[Team],
        draft_orders: &[DraftOrder],
        roster_counts: &HashMap<u64, u32>,
        position_needs: &HashMap<u64, HashMap<String, u8>>,
    ) {
        // 构建球队信息
        for team in teams {
            let roster_count = roster_counts.get(&team.id).copied().unwrap_or(0);
            let needs = position_needs.get(&team.id).cloned().unwrap_or_default();

            self.teams.insert(team.id, TeamAuctionInfo {
                team_id: team.id,
                team_name: team.name.clone(),
                balance: team.balance,
                financial_status: FinancialStatus::from_balance(team.balance),
                roster_count,
                position_needs: needs,
                avg_ability: team.power_rating,
            });
        }

        // 构建选秀顺位映射
        for order in draft_orders {
            self.draft_orders.insert(order.team_id, order.draft_position);
        }
    }

    /// 开始拍卖阶段 - 评估卖签意愿，创建挂牌
    pub fn start_auction(&mut self) -> Vec<DraftPickAuctionEvent> {
        let mut rng = rand::thread_rng();
        let mut new_events = Vec::new();

        self.auction.status = AuctionStatus::InProgress;
        self.auction.started_at = Some(chrono::Utc::now().to_rfc3339());
        self.auction.current_round = 0;

        // 生成拍卖开始事件
        new_events.push(self.create_event(
            AuctionEventType::AuctionStart,
            None,
            None,
            None,
            None,
            "选秀权拍卖大会正式开始！".to_string(),
            "各支球队将根据自身情况决定是否出售手中的选秀权。本次拍卖将进行最多3轮竞价。".to_string(),
            EventImportance::Breaking,
            0,
        ));

        // 遍历所有选秀顺位，评估是否卖签
        let draft_orders: Vec<(u64, u32)> = self.draft_orders.iter()
            .map(|(&team_id, &position)| (team_id, position))
            .collect();

        for (team_id, position) in draft_orders {
            if let Some(team_info) = self.teams.get(&team_id) {
                let should_sell = self.evaluate_sell_decision(team_info, position, &mut rng);

                if should_sell {
                    if let Some(pricing) = get_price_for_position(position) {
                        let listing = DraftPickListing {
                            id: 0,
                            save_id: self.auction.save_id.clone(),
                            season_id: self.auction.season_id,
                            region_id: self.auction.region_id,
                            auction_id: self.auction.id,
                            seller_team_id: team_id,
                            seller_team_name: team_info.team_name.clone(),
                            draft_position: position,
                            starting_price: pricing.starting_price,
                            current_price: pricing.starting_price,
                            min_increment: pricing.min_increment,
                            status: DraftListingStatus::Active,
                            buyer_team_id: None,
                            buyer_team_name: None,
                            final_price: None,
                            commission_fee: None,
                            seller_revenue: None,
                            current_bid_round: 0,
                            created_at: chrono::Utc::now().to_rfc3339(),
                            sold_at: None,
                        };

                        self.listings.push(listing);
                        self.auction.total_auctions += 1;

                        // 生成挂牌事件
                        let position_name = get_position_name(position);
                        new_events.push(self.create_event(
                            AuctionEventType::ListingCreated,
                            Some(self.listings.len() as u64 - 1),
                            Some(team_id),
                            Some(position),
                            Some(pricing.starting_price),
                            format!("{}将{}挂牌出售！", team_info.team_name, position_name),
                            format!(
                                "{}决定出售手中的{}，起拍价{}万。",
                                team_info.team_name,
                                position_name,
                                pricing.starting_price / 10000
                            ),
                            if position <= 3 { EventImportance::Major } else { EventImportance::Normal },
                            0,
                        ));
                    }
                }
            }
        }

        // 如果没有任何挂牌，直接结束拍卖
        if self.listings.is_empty() {
            self.auction.status = AuctionStatus::Completed;
            self.auction.completed_at = Some(chrono::Utc::now().to_rfc3339());

            new_events.push(self.create_event(
                AuctionEventType::AuctionEnd,
                None,
                None,
                None,
                None,
                "本次选秀权拍卖无人挂牌".to_string(),
                "所有球队都选择保留自己的选秀权，本次拍卖会提前结束。".to_string(),
                EventImportance::Normal,
                0,
            ));
        }

        self.events.extend(new_events.clone());
        new_events
    }

    /// AI 卖签决策（4因素相乘模型）
    fn evaluate_sell_decision(&self, team_info: &TeamAuctionInfo, position: u32, rng: &mut impl Rng) -> bool {
        // 1. 财务动机（越穷越想卖）
        let financial_motivation: f64 = match team_info.financial_status {
            FinancialStatus::Bankrupt => 0.70,
            FinancialStatus::Deficit  => 0.50,
            FinancialStatus::Tight    => 0.25,
            FinancialStatus::Healthy  => 0.10,
            FinancialStatus::Wealthy  => 0.05,
        };

        // 2. 签位留存系数（高签越不想卖）
        let pick_retention = match position {
            1      => 0.10,
            2      => 0.15,
            3      => 0.20,
            4..=5  => 0.40,
            6..=8  => 0.60,
            9..=10 => 0.80,
            11..=12 => 1.00,
            _      => 1.20,
        };

        // 3. 阵容系数（缺人保签，满员甩签）
        let roster_factor = if team_info.roster_count < 5 {
            0.10
        } else if team_info.roster_count < 7 {
            0.50
        } else if team_info.roster_count >= 9 {
            1.50
        } else {
            1.00
        };

        // 4. 球队实力（强队不需要高签，弱队保留高签重建）
        let strength_factor = if team_info.avg_ability > 65.0 && position <= 5 {
            1.30
        } else if team_info.avg_ability < 55.0 && position <= 5 {
            0.50
        } else {
            1.00
        };

        let sell_prob: f64 = (financial_motivation * pick_retention * roster_factor * strength_factor)
            .clamp(0.0, 0.90);
        rng.gen::<f64>() < sell_prob
    }

    /// 执行一轮竞拍
    pub fn execute_round(&mut self) -> AuctionRoundResult {
        let mut rng = rand::thread_rng();
        let mut new_events = Vec::new();
        let mut round_bids = Vec::new();
        let mut round_sales = Vec::new();
        let mut round_expirations = Vec::new();

        self.auction.current_round += 1;
        let current_round = self.auction.current_round;
        let max_rounds = self.config.max_rounds;
        let save_id = self.auction.save_id.clone();

        // 收集所有活跃挂牌的索引
        let active_indices: Vec<usize> = self.listings
            .iter()
            .enumerate()
            .filter(|(_, l)| l.status == DraftListingStatus::Active)
            .map(|(i, _)| i)
            .collect();

        // 处理每个活跃挂牌
        for idx in active_indices {
            // 克隆必要数据以避免借用冲突
            let listing_data = {
                let listing = &self.listings[idx];
                (
                    listing.id,
                    listing.seller_team_id,
                    listing.seller_team_name.clone(),
                    listing.draft_position,
                    listing.current_price,
                    listing.min_increment,
                    listing.current_bid_round,
                    listing.buyer_team_id,
                )
            };

            let (listing_id, seller_team_id, seller_team_name, draft_position,
                 current_price, min_increment, current_bid_round, buyer_team_id) = listing_data;

            // 收集本轮出价
            let mut round_bidders: Vec<(u64, i64, String)> = Vec::new();

            // 让每个非卖家球队决定是否竞拍
            for (team_id, team_info) in &self.teams {
                if *team_id == seller_team_id {
                    continue;
                }

                // 内联竞拍决策逻辑以避免借用冲突
                let bid_amount = self.evaluate_bid_for_listing(
                    team_info,
                    draft_position,
                    current_price,
                    min_increment,
                    current_bid_round,
                    buyer_team_id,
                    current_round,
                    &mut rng
                );

                if let Some(amount) = bid_amount {
                    round_bidders.push((*team_id, amount, team_info.team_name.clone()));
                }
            }

            // 处理本轮出价
            if round_bidders.is_empty() {
                // 无人出价
                if current_round >= max_rounds || current_bid_round == 0 {
                    // 已达最大轮数或从未有人出价，流拍
                    self.listings[idx].status = DraftListingStatus::Expired;
                    round_expirations.push(draft_position);

                    let position_name = get_position_name(draft_position);
                    new_events.push(DraftPickAuctionEvent {
                        id: 0,
                        save_id: save_id.clone(),
                        auction_id: self.auction.id,
                        listing_id: Some(listing_id),
                        event_type: AuctionEventType::Expired,
                        team_id: Some(seller_team_id),
                        team_name: Some(seller_team_name.clone()),
                        draft_position: Some(draft_position),
                        amount: None,
                        headline: format!("{}流拍！", position_name),
                        description: format!(
                            "无人竞拍{}，{}将保留该选秀权。",
                            position_name,
                            seller_team_name
                        ),
                        importance: EventImportance::Normal,
                        round: current_round,
                        created_at: chrono::Utc::now().to_rfc3339(),
                    });
                } else if current_round >= max_rounds {
                    // 有人出过价但本轮无新出价，成交
                    if let Some(buyer_id) = buyer_team_id {
                        self.finalize_listing_sale(idx, current_round, &mut new_events);
                        round_sales.push((draft_position, buyer_id));
                    }
                }
            } else {
                // 有出价，取最高出价
                round_bidders.sort_by(|a, b| b.1.cmp(&a.1));
                let (winner_id, winning_bid, winner_team_name) = round_bidders[0].clone();

                // 标记之前的出价为被超
                for bid in &mut self.bids {
                    if bid.listing_id == listing_id && bid.status == BidStatus::Active {
                        bid.status = BidStatus::Outbid;
                    }
                }

                // 创建新出价
                let new_bid = DraftPickBid {
                    id: 0,
                    save_id: save_id.clone(),
                    listing_id,
                    bidder_team_id: winner_id,
                    bidder_team_name: winner_team_name.clone(),
                    bid_amount: winning_bid,
                    bid_round: current_round,
                    status: BidStatus::Active,
                    created_at: chrono::Utc::now().to_rfc3339(),
                };

                // 更新挂牌
                self.listings[idx].current_price = winning_bid;
                self.listings[idx].current_bid_round = current_round;
                self.listings[idx].buyer_team_id = Some(winner_id);
                self.listings[idx].buyer_team_name = Some(winner_team_name.clone());

                round_bids.push(new_bid.clone());
                self.bids.push(new_bid);

                // 生成出价事件
                let position_name = get_position_name(draft_position);
                let event_type = if current_bid_round == 0 {
                    AuctionEventType::BidPlaced
                } else {
                    AuctionEventType::BidRaised
                };

                new_events.push(DraftPickAuctionEvent {
                    id: 0,
                    save_id: save_id.clone(),
                    auction_id: self.auction.id,
                    listing_id: Some(listing_id),
                    event_type,
                    team_id: Some(winner_id),
                    team_name: Some(winner_team_name.clone()),
                    draft_position: Some(draft_position),
                    amount: Some(winning_bid),
                    headline: format!("{}出价竞拍{}！", winner_team_name, position_name),
                    description: format!(
                        "{}以{}万竞拍{}的{}。",
                        winner_team_name,
                        winning_bid / 10000,
                        seller_team_name,
                        position_name
                    ),
                    importance: if draft_position <= 3 { EventImportance::Major } else { EventImportance::Normal },
                    round: current_round,
                    created_at: chrono::Utc::now().to_rfc3339(),
                });

                // 检查是否已达最大轮数
                if current_round >= max_rounds {
                    self.finalize_listing_sale(idx, current_round, &mut new_events);
                    round_sales.push((draft_position, winner_id));
                }
            }
        }

        // 检查是否所有挂牌都已处理完毕
        let active_listings = self.listings.iter().filter(|l| l.status == DraftListingStatus::Active).count();
        if active_listings == 0 || current_round >= max_rounds {
            self.complete_auction(&mut new_events);
        }

        self.events.extend(new_events.clone());

        AuctionRoundResult {
            round: current_round,
            bids: round_bids,
            sales: round_sales,
            expirations: round_expirations,
            events: new_events,
            is_auction_complete: self.auction.status == AuctionStatus::Completed,
        }
    }

    /// AI 竞拍决策
    fn evaluate_bid_for_listing(
        &self,
        team_info: &TeamAuctionInfo,
        draft_position: u32,
        current_price: i64,
        min_increment: i64,
        _current_bid_round: u32,
        buyer_team_id: Option<u64>,
        current_round: u32,
        rng: &mut impl Rng,
    ) -> Option<i64> {
        // 1. 财务差异化预算
        let budget_ratio = match team_info.financial_status {
            FinancialStatus::Wealthy  => 0.40,
            FinancialStatus::Healthy  => 0.30,
            FinancialStatus::Tight    => 0.15,
            FinancialStatus::Deficit  => 0.05,
            FinancialStatus::Bankrupt => return None,
        };
        let available_budget = (team_info.balance as f64 * budget_ratio) as i64;
        let min_bid = current_price + min_increment;

        if available_budget < min_bid || team_info.roster_count >= 15 {
            return None;
        }

        // 2. 签位价值（14级梯度）
        let pick_value: f64 = match draft_position {
            1  => 100.0,
            2  => 92.0,
            3  => 85.0,
            4  => 78.0,
            5  => 72.0,
            6  => 65.0,
            7  => 58.0,
            8  => 52.0,
            9  => 45.0,
            10 => 40.0,
            11 => 35.0,
            12 => 30.0,
            13 => 25.0,
            _  => 20.0,
        };

        // 3. 阵容需求（超过10人奢侈税起征线后大幅降低）
        let need_score = if team_info.roster_count < 5 {
            1.00
        } else if team_info.roster_count < 7 {
            0.60
        } else if team_info.roster_count < 9 {
            0.30
        } else if team_info.roster_count <= 10 {
            0.10
        } else {
            // 超过奢侈税起征线，急剧降低
            (0.10 - (team_info.roster_count - 10) as f64 * 0.02).max(0.01)
        };

        // 4. 实力因素（弱队更需要新秀补强）
        let strength_desire = if team_info.avg_ability < 55.0 {
            1.40
        } else if team_info.avg_ability < 60.0 {
            1.15
        } else if team_info.avg_ability > 65.0 {
            0.70
        } else {
            1.00
        };

        // 5. 基础竞拍概率
        let mut bid_prob = (pick_value / 100.0) * 0.50 * need_score * strength_desire;

        // 6. 价格敏感度（指数衰减）
        if let Some(pricing) = get_price_for_position(draft_position) {
            let price_ratio = current_price as f64 / pricing.starting_price as f64;
            if price_ratio > 1.0 {
                bid_prob *= (0.6_f64).powf(price_ratio - 1.0);
            }
        }

        // 7. 财务信心
        bid_prob *= match team_info.financial_status {
            FinancialStatus::Wealthy  => 1.30,
            FinancialStatus::Healthy  => 1.00,
            FinancialStatus::Tight    => 0.60,
            FinancialStatus::Deficit  => 0.25,
            FinancialStatus::Bankrupt => 0.00,
        };

        // 8. 轮次动态
        if current_round > 1 && buyer_team_id.is_some() {
            bid_prob *= 0.65;
        }

        if rng.gen::<f64>() >= bid_prob {
            return None;
        }

        // 9. 差异化出价上限
        let aggression = match team_info.financial_status {
            FinancialStatus::Wealthy  => 1.5,
            FinancialStatus::Healthy  => 1.3,
            FinancialStatus::Tight    => 1.15,
            _                         => 1.05,
        };
        let max_bid = (min_bid as f64 * aggression).min(available_budget as f64) as i64;
        if max_bid <= min_bid {
            return None;
        }

        Some(rng.gen_range(min_bid..=max_bid))
    }

    /// 完成指定索引的挂牌交易
    fn finalize_listing_sale(&mut self, idx: usize, round: u32, events: &mut Vec<DraftPickAuctionEvent>) {
        let listing = &mut self.listings[idx];
        if let Some(buyer_id) = listing.buyer_team_id {
            listing.status = DraftListingStatus::Sold;
            listing.final_price = Some(listing.current_price);
            listing.commission_fee = Some(calculate_commission(listing.current_price));
            listing.seller_revenue = Some(calculate_seller_revenue(listing.current_price));
            listing.sold_at = Some(chrono::Utc::now().to_rfc3339());

            self.auction.successful_auctions += 1;
            self.auction.total_revenue += listing.current_price;
            self.auction.total_commission += listing.commission_fee.unwrap_or(0);

            // 标记获胜出价
            let listing_id = listing.id;
            for bid in &mut self.bids {
                if bid.listing_id == listing_id && bid.bidder_team_id == buyer_id && bid.status == BidStatus::Active {
                    bid.status = BidStatus::Won;
                }
            }

            // 更新选秀顺位映射
            let seller_team_id = listing.seller_team_id;
            let draft_position = listing.draft_position;
            self.draft_orders.remove(&seller_team_id);
            self.draft_orders.insert(buyer_id, draft_position);

            let position_name = get_position_name(draft_position);
            let buyer_team_name = listing.buyer_team_name.clone().unwrap_or_default();
            let seller_team_name = listing.seller_team_name.clone();
            let current_price = listing.current_price;
            let seller_revenue = listing.seller_revenue.unwrap_or(0);

            events.push(DraftPickAuctionEvent {
                id: 0,
                save_id: self.auction.save_id.clone(),
                auction_id: self.auction.id,
                listing_id: Some(listing_id),
                event_type: AuctionEventType::Sold,
                team_id: Some(buyer_id),
                team_name: Some(buyer_team_name.clone()),
                draft_position: Some(draft_position),
                amount: Some(current_price),
                headline: format!("{}成功拍得{}！", buyer_team_name, position_name),
                description: format!(
                    "{}以{}万从{}手中购得{}。扣除{}%佣金后，{}将获得{}万收入。",
                    buyer_team_name,
                    current_price / 10000,
                    seller_team_name,
                    position_name,
                    (COMMISSION_RATE * 100.0) as u32,
                    seller_team_name,
                    seller_revenue / 10000
                ),
                importance: if draft_position <= 3 { EventImportance::Breaking } else { EventImportance::Major },
                round,
                created_at: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    /// 完成拍卖
    fn complete_auction(&mut self, events: &mut Vec<DraftPickAuctionEvent>) {
        self.auction.status = AuctionStatus::Completed;
        self.auction.completed_at = Some(chrono::Utc::now().to_rfc3339());

        let sold_count = self.listings.iter().filter(|l| l.status == DraftListingStatus::Sold).count();
        let expired_count = self.listings.iter().filter(|l| l.status == DraftListingStatus::Expired).count();

        events.push(self.create_event(
            AuctionEventType::AuctionEnd,
            None,
            None,
            None,
            Some(self.auction.total_revenue),
            "选秀权拍卖大会圆满结束！".to_string(),
            format!(
                "本次拍卖共有{}个选秀权挂牌，{}个成功售出，{}个流拍。总成交额{}万，联盟收取佣金{}万。",
                self.auction.total_auctions,
                sold_count,
                expired_count,
                self.auction.total_revenue / 10000,
                self.auction.total_commission / 10000
            ),
            EventImportance::Breaking,
            self.auction.current_round,
        ));
    }

    /// 快进完成所有轮次
    pub fn fast_forward(&mut self) -> Vec<AuctionRoundResult> {
        let mut results = Vec::new();

        while self.auction.status == AuctionStatus::InProgress {
            let result = self.execute_round();
            results.push(result);
        }

        results
    }

    /// 创建拍卖事件
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

    /// 获取当前拍卖状态
    pub fn get_status(&self) -> &DraftPickAuction {
        &self.auction
    }

    /// 获取所有挂牌
    pub fn get_listings(&self) -> &[DraftPickListing] {
        &self.listings
    }

    /// 获取所有事件
    pub fn get_events(&self) -> &[DraftPickAuctionEvent] {
        &self.events
    }

    /// 获取更新后的选秀顺位
    pub fn get_updated_draft_orders(&self) -> HashMap<u64, u32> {
        self.draft_orders.clone()
    }
}

/// 拍卖轮次结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionRoundResult {
    pub round: u32,
    pub bids: Vec<DraftPickBid>,
    pub sales: Vec<(u32, u64)>, // (draft_position, buyer_team_id)
    pub expirations: Vec<u32>,  // draft_position
    pub events: Vec<DraftPickAuctionEvent>,
    pub is_auction_complete: bool,
}

// FinancialStatus::from_balance 已在 team.rs 中定义

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auction_engine_creation() {
        let engine = DraftAuctionEngine::new("save1".to_string(), 1, 1);
        assert_eq!(engine.auction.save_id, "save1");
        assert_eq!(engine.auction.season_id, 1);
        assert_eq!(engine.auction.region_id, 1);
        assert_eq!(engine.auction.status, AuctionStatus::Preparing);
    }

    #[test]
    fn test_financial_status_from_balance() {
        // 阈值（元）：Wealthy > 10_000_000, Healthy > 5_000_000, Tight > 1_000_000, Deficit >= 0, Bankrupt < 0
        assert_eq!(FinancialStatus::from_balance(20_000_000), FinancialStatus::Wealthy);
        assert_eq!(FinancialStatus::from_balance(8_000_000), FinancialStatus::Healthy);
        assert_eq!(FinancialStatus::from_balance(3_000_000), FinancialStatus::Tight);
        assert_eq!(FinancialStatus::from_balance(500_000), FinancialStatus::Deficit);
        assert_eq!(FinancialStatus::from_balance(-1_000_000), FinancialStatus::Bankrupt);
    }

    #[test]
    fn test_commission_calculation() {
        let price = 1000_0000;
        let commission = calculate_commission(price);
        let revenue = calculate_seller_revenue(price);

        assert_eq!(commission, 50_0000); // 5% of 1000万
        assert_eq!(revenue, 950_0000);   // 1000万 - 50万
    }
}
