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
    /// 财务困难时卖签概率
    pub bankrupt_sell_prob: f64,
    /// 阵容充足时卖签概率
    pub roster_full_sell_prob: f64,
    /// 低顺位卖签概率
    pub low_pick_sell_prob: f64,
    /// 中顺位财务好时卖签概率
    pub mid_pick_wealthy_sell_prob: f64,
}

impl Default for DraftAuctionConfig {
    fn default() -> Self {
        Self {
            max_rounds: 3,
            commission_rate: COMMISSION_RATE,
            bankrupt_sell_prob: 0.80,
            roster_full_sell_prob: 0.60,
            low_pick_sell_prob: 0.30,
            mid_pick_wealthy_sell_prob: 0.15,
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

    /// AI 卖签决策（简化版本）
    fn evaluate_sell_decision(&self, team_info: &TeamAuctionInfo, position: u32, rng: &mut impl Rng) -> bool {
        let mut base_prob = 0.2; // 基础卖签概率

        // 1. 财务困难大幅提高
        if matches!(team_info.financial_status, FinancialStatus::Bankrupt | FinancialStatus::Deficit) {
            base_prob += 0.50;
        }

        // 2. 签位价值（高顺位签更难卖）
        base_prob *= if position <= 3 { 0.3 } else if position >= 10 { 1.5 } else { 1.0 };

        // 3. 阵容需求（缺人保留签）
        if team_info.roster_count < 6 {
            base_prob *= 0.4;
        }

        rng.gen::<f64>() < base_prob.clamp(0.0, 0.95)
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
    fn evaluate_bid_decision(
        &self,
        team_info: &TeamAuctionInfo,
        listing: &DraftPickListing,
        current_round: u32,
        rng: &mut impl Rng,
    ) -> Option<i64> {
        // 计算可用预算
        let budget_ratio = match team_info.financial_status {
            FinancialStatus::Wealthy => 0.40,
            FinancialStatus::Healthy => 0.25,
            FinancialStatus::Tight => 0.10,
            FinancialStatus::Deficit => 0.05,
            FinancialStatus::Bankrupt => 0.0,
        };

        let available_budget = (team_info.balance as f64 * budget_ratio) as i64;
        let min_bid = listing.current_price + listing.min_increment;

        // 预算不足
        if available_budget < min_bid {
            return None;
        }

        // 阵容已满(>=10人)不竞拍
        if team_info.roster_count >= 10 {
            return None;
        }

        // 计算竞拍意愿
        let mut bid_probability = 0.0;

        // 高顺位签(1-5)更有吸引力
        if listing.draft_position <= 5 {
            bid_probability += 0.40;
        } else if listing.draft_position <= 10 {
            bid_probability += 0.20;
        } else {
            bid_probability += 0.10;
        }

        // 财务状况影响
        bid_probability *= match team_info.financial_status {
            FinancialStatus::Wealthy => 1.5,
            FinancialStatus::Healthy => 1.0,
            FinancialStatus::Tight => 0.5,
            _ => 0.0,
        };

        // 阵容需求影响 - 缺人更愿意买签
        if team_info.roster_count < 6 {
            bid_probability += 0.30;
        } else if team_info.roster_count < 8 {
            bid_probability += 0.15;
        }

        // 如果价格已经很高，降低竞拍意愿
        if let Some(pricing) = get_price_for_position(listing.draft_position) {
            let price_ratio = listing.current_price as f64 / pricing.starting_price as f64;
            if price_ratio > 1.5 {
                bid_probability *= 0.5;
            } else if price_ratio > 2.0 {
                bid_probability *= 0.2;
            }
        }

        // 后续轮次竞争更激烈
        if current_round > 1 && listing.buyer_team_id.is_some() {
            // 已经有人出价，需要更高意愿才会跟进
            bid_probability *= 0.7;
        }

        let roll: f64 = rng.gen();
        if roll >= bid_probability {
            return None;
        }

        // 决定出价金额
        let max_bid = (available_budget * 8 / 10).min(min_bid * 3 / 2); // 最多出可用预算的80%
        if max_bid < min_bid {
            return None;
        }

        // 出价策略：在最低加价和最高可接受价格之间随机
        let bid_range = max_bid - min_bid;
        let random_addition = if bid_range > 0 {
            rng.gen_range(0..=bid_range / 2)
        } else {
            0
        };

        Some(min_bid + random_addition)
    }

    /// AI 竞拍决策（简化版本）
    fn evaluate_bid_for_listing(
        &self,
        team_info: &TeamAuctionInfo,
        draft_position: u32,
        current_price: i64,
        min_increment: i64,
        current_bid_round: u32,
        buyer_team_id: Option<u64>,
        current_round: u32,
        rng: &mut impl Rng,
    ) -> Option<i64> {
        // 1. 可用预算
        let budget_ratio = 0.6; // 默认 60%
        let available_budget = (team_info.balance as f64 * budget_ratio) as i64;
        let min_bid = current_price + min_increment;

        if available_budget < min_bid || team_info.roster_count >= 10 {
            return None;
        }

        // 2. 签位价值评分
        let pick_value = Self::calculate_pick_value(draft_position);

        // 3. 竞拍意愿概率
        let mut bid_prob = (pick_value as f64 / 100.0) * 0.6;

        // 阵容需求加成
        if team_info.roster_count < 6 {
            bid_prob += 0.30;
        }

        // 价格敏感度
        if let Some(pricing) = get_price_for_position(draft_position) {
            let price_ratio = current_price as f64 / pricing.starting_price as f64;
            if price_ratio > 1.3 {
                bid_prob *= (0.5_f64).powf((price_ratio - 1.0));
            }
        }

        // 后续轮次抑制
        if current_round > 1 && buyer_team_id.is_some() {
            bid_prob *= 0.7;
        }

        if rng.gen::<f64>() >= bid_prob {
            return None;
        }

        // 4. 决定出价金额（温和策略）
        let max_bid = (min_bid as f64 * 1.3).min(available_budget as f64) as i64;
        if max_bid <= min_bid {
            return None;
        }

        Some(rng.gen_range(min_bid..=max_bid))
    }

    /// 计算签位价值评分（0-100）- 简化版本
    fn calculate_pick_value(position: u32) -> u8 {
        match position {
            1..=3 => 95,
            4..=5 => 80,
            6..=8 => 65,
            9..=10 => 50,
            _ => 35,
        }
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

    /// 完成一笔交易 (旧版，保留兼容)
    fn finalize_sale(&mut self, listing: &mut DraftPickListing, events: &mut Vec<DraftPickAuctionEvent>, round: u32) {
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
            for bid in &mut self.bids {
                if bid.listing_id == listing.id && bid.bidder_team_id == buyer_id && bid.status == BidStatus::Active {
                    bid.status = BidStatus::Won;
                }
            }

            // 更新选秀顺位映射
            self.draft_orders.remove(&listing.seller_team_id);
            self.draft_orders.insert(buyer_id, listing.draft_position);

            let position_name = get_position_name(listing.draft_position);
            events.push(self.create_event(
                AuctionEventType::Sold,
                Some(listing.id),
                Some(buyer_id),
                Some(listing.draft_position),
                Some(listing.current_price),
                format!("{}成功拍得{}！", listing.buyer_team_name.as_deref().unwrap_or("买家"), position_name),
                format!(
                    "{}以{}万从{}手中购得{}。扣除{}%佣金后，{}将获得{}万收入。",
                    listing.buyer_team_name.as_deref().unwrap_or("买家"),
                    listing.current_price / 10000,
                    listing.seller_team_name,
                    position_name,
                    (COMMISSION_RATE * 100.0) as u32,
                    listing.seller_team_name,
                    listing.seller_revenue.unwrap_or(0) / 10000
                ),
                if listing.draft_position <= 3 { EventImportance::Breaking } else { EventImportance::Major },
                round,
            ));
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

/// 出价策略
#[derive(Debug, Clone, Copy)]
enum BidStrategy {
    Minimal,      // 最低加价
    Conservative, // 保守试探
    Moderate,     // 中等出价
    Aggressive,   // 激进竞价
}

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
        assert_eq!(FinancialStatus::from_balance(10000_0000), FinancialStatus::Wealthy);
        assert_eq!(FinancialStatus::from_balance(3000_0000), FinancialStatus::Healthy);
        assert_eq!(FinancialStatus::from_balance(1000_0000), FinancialStatus::Tight);
        assert_eq!(FinancialStatus::from_balance(100_0000), FinancialStatus::Deficit);
        assert_eq!(FinancialStatus::from_balance(-100_0000), FinancialStatus::Bankrupt);
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
