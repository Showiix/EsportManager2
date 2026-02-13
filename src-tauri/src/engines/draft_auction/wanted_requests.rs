//! 求购请求生成与处理逻辑

use super::{DraftAuctionEngine, TeamAuctionInfo};
use crate::models::{
    calculate_commission, calculate_seller_revenue, get_position_name, get_price_for_position,
    AuctionEventType, DraftPickAuctionEvent, DraftPickWanted, EventImportance, FinancialStatus,
    WantedStatus,
};
use rand::Rng;

impl DraftAuctionEngine {
    pub(super) fn generate_wanted_requests(&mut self) -> Vec<DraftPickAuctionEvent> {
        let mut rng = rand::thread_rng();
        let mut new_events = Vec::new();

        let listed_positions: Vec<u32> = self.listings.iter().map(|l| l.draft_position).collect();

        let unlisted_picks: Vec<(u64, u32)> = self
            .draft_orders
            .iter()
            .filter(|(_, &pos)| !listed_positions.contains(&pos))
            .map(|(&tid, &pos)| (tid, pos))
            .collect();

        if unlisted_picks.is_empty() {
            return new_events;
        }

        let already_wanted: std::collections::HashSet<(u64, u64, u32)> = self
            .wanted_requests
            .iter()
            .filter(|w| w.status != WantedStatus::Expired)
            .map(|w| (w.buyer_team_id, w.holder_team_id, w.target_position))
            .collect();

        let team_infos: Vec<(u64, TeamAuctionInfo)> = self
            .teams
            .iter()
            .map(|(&id, info)| (id, info.clone()))
            .collect();

        for (buyer_id, buyer_info) in &team_infos {
            if buyer_info.financial_status == FinancialStatus::Bankrupt {
                continue;
            }

            if buyer_info.roster_count >= 8 {
                continue;
            }

            let own_pick = self.draft_orders.get(buyer_id).copied();
            let buy_desire = self.calculate_buy_desire(buyer_info, own_pick);
            if rng.gen::<f64>() >= buy_desire {
                continue;
            }

            let max_requests = if buyer_info.roster_count < 5 {
                3
            } else if buyer_info.avg_ability < 55.0 {
                2
            } else {
                1
            };

            let mut targets: Vec<(u64, u32, f64)> = Vec::new();
            for &(holder_id, position) in &unlisted_picks {
                if holder_id == *buyer_id {
                    continue;
                }
                if let Some(own_pos) = own_pick {
                    if position >= own_pos {
                        continue;
                    }
                }
                if already_wanted.contains(&(*buyer_id, holder_id, position)) {
                    continue;
                }
                let match_score = self.calculate_rookie_bid_factor(buyer_info, position);
                let pick_value: f64 = match position {
                    1 => 100.0,
                    2 => 92.0,
                    3 => 85.0,
                    4 => 78.0,
                    5 => 72.0,
                    6 => 65.0,
                    7 => 58.0,
                    8 => 52.0,
                    9 => 45.0,
                    10 => 40.0,
                    _ => 30.0,
                };
                targets.push((holder_id, position, match_score * pick_value));
            }

            targets.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
            let take_count = max_requests.min(targets.len());

            for &(holder_id, position, _) in targets.iter().take(take_count) {
                if let Some(pricing) = get_price_for_position(position) {
                    let budget_ratio = match buyer_info.financial_status {
                        FinancialStatus::Wealthy => 0.40,
                        FinancialStatus::Healthy => 0.30,
                        FinancialStatus::Tight => 0.15,
                        FinancialStatus::Deficit => 0.08,
                        FinancialStatus::Bankrupt => 0.0,
                    };
                    let max_budget = (buyer_info.balance as f64 * budget_ratio) as i64;
                    let premium = 1.0 + rng.gen_range(0.0..0.3);
                    let offer = ((pricing.starting_price as f64 * premium) as i64).min(max_budget);

                    if offer < pricing.starting_price / 2 {
                        continue;
                    }

                    let holder_name = self
                        .teams
                        .get(&holder_id)
                        .map(|t| t.team_name.clone())
                        .unwrap_or_default();
                    let position_name = get_position_name(position);

                    let reason =
                        self.generate_wanted_reason(buyer_info, own_pick, position, &mut rng);

                    let wanted = DraftPickWanted {
                        id: 0,
                        save_id: self.auction.save_id.clone(),
                        season_id: self.auction.season_id,
                        region_id: self.auction.region_id,
                        auction_id: self.auction.id,
                        buyer_team_id: *buyer_id,
                        buyer_team_name: buyer_info.team_name.clone(),
                        target_position: position,
                        offer_price: offer,
                        reason: reason.clone(),
                        status: WantedStatus::Active,
                        holder_team_id: holder_id,
                        holder_team_name: holder_name.clone(),
                        response_reason: None,
                        final_price: None,
                        created_at: chrono::Utc::now().to_rfc3339(),
                        resolved_at: None,
                    };

                    self.wanted_requests.push(wanted);

                    new_events.push(self.create_event(
                        AuctionEventType::WantedCreated,
                        None,
                        Some(*buyer_id),
                        Some(position),
                        Some(offer),
                        format!(
                            "{}向{}求购{}！",
                            buyer_info.team_name, holder_name, position_name
                        ),
                        format!(
                            "{}主动出价{}万向{}求购{}。理由：{}",
                            buyer_info.team_name,
                            offer / 10000,
                            holder_name,
                            position_name,
                            reason,
                        ),
                        if position <= 3 {
                            EventImportance::Major
                        } else {
                            EventImportance::Normal
                        },
                        0,
                    ));
                }
            }
        }

        new_events
    }

    pub(super) fn calculate_buy_desire(
        &self,
        team_info: &TeamAuctionInfo,
        own_pick: Option<u32>,
    ) -> f64 {
        let roster_need = if team_info.roster_count < 5 {
            0.90
        } else if team_info.roster_count < 7 {
            0.70
        } else if team_info.roster_count < 9 {
            0.50
        } else {
            0.30
        };

        let strength_need = if team_info.avg_ability < 55.0 {
            1.30
        } else if team_info.avg_ability < 60.0 {
            1.00
        } else {
            0.60
        };

        let financial_confidence = match team_info.financial_status {
            FinancialStatus::Wealthy => 1.20,
            FinancialStatus::Healthy => 1.00,
            FinancialStatus::Tight => 0.50,
            FinancialStatus::Deficit => 0.15,
            FinancialStatus::Bankrupt => 0.00,
        };

        let upgrade_factor = match own_pick {
            Some(pos) if pos >= 5 => 0.80,
            Some(_) => 0.30,
            None => 1.00,
        };

        let desire: f64 = roster_need * strength_need * financial_confidence * upgrade_factor;
        desire.clamp(0.0, 0.85)
    }

    pub(super) fn generate_wanted_reason(
        &self,
        buyer_info: &TeamAuctionInfo,
        own_pick: Option<u32>,
        target_position: u32,
        rng: &mut impl Rng,
    ) -> String {
        let target_rookie = self
            .draft_rookies
            .iter()
            .filter(|r| r.draft_rank <= target_position + 1)
            .min_by_key(|r| (r.draft_rank as i32 - target_position as i32).unsigned_abs());

        let pos_name = target_rookie
            .map(|r| {
                match r.position.as_str() {
                    "TOP" => "上单",
                    "JUG" => "打野",
                    "MID" => "中单",
                    "ADC" => "ADC",
                    "SUP" => "辅助",
                    _ => "",
                }
                .to_string()
            })
            .unwrap_or_default();

        let rookie_label = target_rookie
            .map(|r| {
                format!(
                    "{}位新秀",
                    match r.position.as_str() {
                        "TOP" => "上单",
                        "JUG" => "打野",
                        "MID" => "中单",
                        "ADC" => "ADC",
                        "SUP" => "辅助",
                        _ => "",
                    }
                )
            })
            .unwrap_or_default();

        let mut reasons: Vec<String> = Vec::new();

        if buyer_info.roster_count < 5 {
            reasons.push("阵容严重不足，急需补充新血液".to_string());
        }
        if buyer_info.avg_ability < 55.0 {
            reasons.push("球队实力偏弱，寄望高潜新秀带来突破".to_string());
        }
        if let Some(own_pos) = own_pick {
            if own_pos >= 8 {
                reasons.push(format!("手握第{}签位置靠后，想争取更前的选秀权", own_pos));
            } else {
                reasons.push(format!("已有第{}签，但希望获得更优质的选秀位", own_pos));
            }
        } else {
            reasons.push("目前没有选秀权，主动出击争取签位".to_string());
        }
        if !pos_name.is_empty() {
            let pos_key = target_rookie
                .map(|r| r.position.clone())
                .unwrap_or_default();
            let need = buyer_info
                .position_needs
                .get(&pos_key)
                .copied()
                .unwrap_or(50);
            if need >= 70 {
                reasons.push(format!(
                    "{}位置急需补强，看好该签位{}的潜力",
                    pos_name, rookie_label
                ));
            } else if !rookie_label.is_empty() {
                reasons.push(format!("目标锁定该签位{}，愿意溢价求购", rookie_label));
            }
        }
        if buyer_info.financial_status == FinancialStatus::Wealthy {
            reasons.push("财力雄厚，不惜代价争夺优质签位".to_string());
        }

        if reasons.is_empty() {
            return "看好本届新秀，主动求购签位".to_string();
        }

        let idx = rng.gen_range(0..reasons.len());
        reasons.swap_remove(idx)
    }

    pub(super) fn process_wanted_requests(
        &mut self,
        current_round: u32,
        events: &mut Vec<DraftPickAuctionEvent>,
    ) -> Vec<DraftPickWanted> {
        let mut rng = rand::thread_rng();
        let mut results = Vec::new();

        let mut groups: std::collections::HashMap<(u64, u32), Vec<usize>> =
            std::collections::HashMap::new();

        for (i, w) in self.wanted_requests.iter().enumerate() {
            if w.status == WantedStatus::Active {
                groups
                    .entry((w.holder_team_id, w.target_position))
                    .or_default()
                    .push(i);
            }
        }

        for ((holder_id, position), indices) in &groups {
            if !self.draft_orders.contains_key(holder_id)
                || self.draft_orders.get(holder_id).copied() != Some(*position)
            {
                for &idx in indices {
                    self.wanted_requests[idx].status = WantedStatus::Expired;
                    self.wanted_requests[idx].resolved_at = Some(chrono::Utc::now().to_rfc3339());
                    self.wanted_requests[idx].response_reason = Some("签位已转手".to_string());
                    results.push(self.wanted_requests[idx].clone());
                }
                continue;
            }

            let holder_info = match self.teams.get(holder_id) {
                Some(info) => info.clone(),
                None => continue,
            };

            if indices.len() >= 2 {
                let bid_premium = (1.0 + (indices.len() as f64 - 1.0) * 0.08).min(1.30);
                for &idx in indices {
                    self.wanted_requests[idx].offer_price =
                        (self.wanted_requests[idx].offer_price as f64 * bid_premium) as i64;
                }
            }

            let mut sorted_indices: Vec<usize> = indices.clone();
            sorted_indices.sort_by(|&a, &b| {
                self.wanted_requests[b]
                    .offer_price
                    .cmp(&self.wanted_requests[a].offer_price)
            });

            let mut winner_idx: Option<usize> = None;
            for &idx in &sorted_indices {
                let offer = self.wanted_requests[idx].offer_price;
                let accept_prob =
                    self.evaluate_wanted_accept(&holder_info, *position, offer, &mut rng);
                if rng.gen::<f64>() < accept_prob {
                    winner_idx = Some(idx);
                    break;
                }
            }

            let position_name = get_position_name(*position);

            if let Some(widx) = winner_idx {
                let offer = self.wanted_requests[widx].offer_price;
                let counter_ratio = 1.0 + rng.gen_range(0.0..0.10);
                let final_price = ((offer as f64 * counter_ratio) as i64).max(offer);
                let commission = calculate_commission(final_price);
                let seller_revenue = calculate_seller_revenue(final_price);

                let buyer_name = self.wanted_requests[widx].buyer_team_name.clone();
                let holder_name = self.wanted_requests[widx].holder_team_name.clone();

                self.wanted_requests[widx].status = WantedStatus::Fulfilled;
                self.wanted_requests[widx].final_price = Some(final_price);
                self.wanted_requests[widx].response_reason = Some("双方达成协议".to_string());
                self.wanted_requests[widx].resolved_at = Some(chrono::Utc::now().to_rfc3339());

                let buyer_id = self.wanted_requests[widx].buyer_team_id;
                self.draft_orders.remove(holder_id);
                self.draft_orders.insert(buyer_id, *position);

                self.auction.successful_auctions += 1;
                self.auction.total_revenue += final_price;
                self.auction.total_commission += commission;

                let compete_note = if indices.len() >= 2 {
                    format!("（击败{}支球队的竞争报价）", indices.len() - 1)
                } else {
                    String::new()
                };

                events.push(DraftPickAuctionEvent {
                    id: 0,
                    save_id: self.auction.save_id.clone(),
                    auction_id: self.auction.id,
                    listing_id: None,
                    event_type: AuctionEventType::WantedAccepted,
                    team_id: Some(buyer_id),
                    team_name: Some(buyer_name.clone()),
                    draft_position: Some(*position),
                    amount: Some(final_price),
                    headline: format!("求购成交！{}获得{}！", buyer_name, position_name),
                    description: format!(
                        "{}同意了{}的求购请求，以{}万出售{}。扣除佣金后获得{}万。{}",
                        holder_name,
                        buyer_name,
                        final_price / 10000,
                        position_name,
                        seller_revenue / 10000,
                        compete_note,
                    ),
                    importance: if *position <= 3 {
                        EventImportance::Breaking
                    } else {
                        EventImportance::Major
                    },
                    round: current_round,
                    created_at: chrono::Utc::now().to_rfc3339(),
                });

                results.push(self.wanted_requests[widx].clone());

                for &idx in &sorted_indices {
                    if idx == widx {
                        continue;
                    }
                    self.wanted_requests[idx].status = WantedStatus::Rejected;
                    self.wanted_requests[idx].response_reason = Some("被更高报价击败".to_string());
                    self.wanted_requests[idx].resolved_at = Some(chrono::Utc::now().to_rfc3339());

                    let loser_name = self.wanted_requests[idx].buyer_team_name.clone();
                    events.push(DraftPickAuctionEvent {
                        id: 0,
                        save_id: self.auction.save_id.clone(),
                        auction_id: self.auction.id,
                        listing_id: None,
                        event_type: AuctionEventType::WantedRejected,
                        team_id: Some(self.wanted_requests[idx].buyer_team_id),
                        team_name: Some(loser_name.clone()),
                        draft_position: Some(*position),
                        amount: Some(self.wanted_requests[idx].offer_price),
                        headline: format!("{}竞争{}失败", loser_name, position_name),
                        description: format!(
                            "{}对{}的求购被更高报价击败，签位被{}获得。",
                            loser_name, position_name, buyer_name,
                        ),
                        importance: EventImportance::Normal,
                        round: current_round,
                        created_at: chrono::Utc::now().to_rfc3339(),
                    });

                    results.push(self.wanted_requests[idx].clone());
                }
            } else {
                for &idx in &sorted_indices {
                    let offer = self.wanted_requests[idx].offer_price;
                    let reject_reason = if offer
                        < get_price_for_position(*position)
                            .map(|p| p.starting_price)
                            .unwrap_or(0)
                    {
                        "报价太低".to_string()
                    } else if holder_info.roster_count < 6 {
                        "球队需要新秀补充阵容".to_string()
                    } else {
                        "不愿出售该签位".to_string()
                    };

                    self.wanted_requests[idx].status = WantedStatus::Rejected;
                    self.wanted_requests[idx].response_reason = Some(reject_reason.clone());
                    self.wanted_requests[idx].resolved_at = Some(chrono::Utc::now().to_rfc3339());

                    let buyer_name = self.wanted_requests[idx].buyer_team_name.clone();
                    let holder_name = self.wanted_requests[idx].holder_team_name.clone();
                    events.push(DraftPickAuctionEvent {
                        id: 0,
                        save_id: self.auction.save_id.clone(),
                        auction_id: self.auction.id,
                        listing_id: None,
                        event_type: AuctionEventType::WantedRejected,
                        team_id: Some(*holder_id),
                        team_name: Some(holder_name.clone()),
                        draft_position: Some(*position),
                        amount: Some(offer),
                        headline: format!("{}拒绝了{}的求购", holder_name, buyer_name),
                        description: format!(
                            "{}拒绝了{}以{}万求购{}的请求。原因：{}",
                            holder_name,
                            buyer_name,
                            offer / 10000,
                            position_name,
                            reject_reason,
                        ),
                        importance: EventImportance::Normal,
                        round: current_round,
                        created_at: chrono::Utc::now().to_rfc3339(),
                    });

                    results.push(self.wanted_requests[idx].clone());
                }
            }
        }

        results
    }

    pub(super) fn evaluate_wanted_accept(
        &self,
        holder_info: &TeamAuctionInfo,
        position: u32,
        offer: i64,
        rng: &mut impl Rng,
    ) -> f64 {
        let base_price = get_price_for_position(position)
            .map(|p| p.starting_price)
            .unwrap_or(100_0000);
        let price_ratio = offer as f64 / base_price as f64;

        let price_factor = if price_ratio >= 1.5 {
            0.85
        } else if price_ratio >= 1.2 {
            0.60
        } else if price_ratio >= 1.0 {
            0.40
        } else if price_ratio >= 0.8 {
            0.20
        } else {
            0.05
        };

        let financial_motivation = match holder_info.financial_status {
            FinancialStatus::Bankrupt => 0.90,
            FinancialStatus::Deficit => 0.70,
            FinancialStatus::Tight => 0.45,
            FinancialStatus::Healthy => 0.25,
            FinancialStatus::Wealthy => 0.10,
        };

        let retention = match position {
            1 => 0.15,
            2 => 0.25,
            3 => 0.35,
            4..=5 => 0.55,
            6..=8 => 0.75,
            _ => 0.90,
        };

        let roster_factor = if holder_info.roster_count >= 9 {
            1.40
        } else if holder_info.roster_count < 5 {
            0.30
        } else {
            1.00
        };

        let rookie_match = self.calculate_rookie_match_factor(holder_info, position);
        let _ = rng;

        (price_factor * financial_motivation / (1.0 - retention + 0.1)
            * roster_factor
            * rookie_match)
            .clamp(0.02, 0.85)
    }
}
