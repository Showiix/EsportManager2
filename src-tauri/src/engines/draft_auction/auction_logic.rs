//! 核心拍卖逻辑与流程控制

use super::{AuctionRoundResult, DraftAuctionEngine};
use crate::models::{
    calculate_commission, calculate_seller_revenue, get_position_name, get_price_for_position,
    AuctionEventType, AuctionStatus, BidStatus, DraftListingStatus, DraftPickAuctionEvent,
    DraftPickBid, DraftPickListing, EventImportance, WantedStatus, COMMISSION_RATE,
};

impl DraftAuctionEngine {
    pub fn start_auction(&mut self) -> Vec<DraftPickAuctionEvent> {
        let mut rng = rand::thread_rng();
        let mut new_events = Vec::new();

        self.auction.status = AuctionStatus::InProgress;
        self.auction.started_at = Some(chrono::Utc::now().to_rfc3339());
        self.auction.current_round = 0;

        new_events.push(
            self.create_event(
                AuctionEventType::AuctionStart,
                None,
                None,
                None,
                None,
                "选秀权拍卖大会正式开始！".to_string(),
                "各支球队将根据自身情况决定是否出售手中的选秀权。本次拍卖将进行最多3轮竞价。"
                    .to_string(),
                EventImportance::Breaking,
                0,
            ),
        );

        let draft_orders: Vec<(u64, u32)> = self
            .draft_orders
            .iter()
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
        }

        let wanted_events = self.generate_wanted_requests();
        new_events.extend(wanted_events);

        if self.listings.is_empty() && self.wanted_requests.is_empty() {
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

        let active_indices: Vec<usize> = self
            .listings
            .iter()
            .enumerate()
            .filter(|(_, l)| l.status == DraftListingStatus::Active)
            .map(|(i, _)| i)
            .collect();

        for idx in active_indices {
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

            let (
                listing_id,
                seller_team_id,
                seller_team_name,
                draft_position,
                current_price,
                min_increment,
                current_bid_round,
                buyer_team_id,
            ) = listing_data;

            let mut round_bidders: Vec<(u64, i64, String)> = Vec::new();

            for (team_id, team_info) in &self.teams {
                if *team_id == seller_team_id {
                    continue;
                }

                let bid_amount = self.evaluate_bid_for_listing(
                    team_info,
                    draft_position,
                    current_price,
                    min_increment,
                    current_bid_round,
                    buyer_team_id,
                    current_round,
                    &mut rng,
                );

                if let Some(amount) = bid_amount {
                    round_bidders.push((*team_id, amount, team_info.team_name.clone()));
                }
            }

            if round_bidders.is_empty() {
                if current_bid_round == 0 && current_round >= max_rounds {
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
                            position_name, seller_team_name
                        ),
                        importance: EventImportance::Normal,
                        round: current_round,
                        created_at: chrono::Utc::now().to_rfc3339(),
                    });
                } else if current_bid_round > 0 && current_round >= max_rounds {
                    if let Some(buyer_id) = buyer_team_id {
                        self.finalize_listing_sale(idx, current_round, &mut new_events);
                        round_sales.push((draft_position, buyer_id));
                    }
                }
            } else {
                round_bidders.sort_by(|a, b| b.1.cmp(&a.1));
                let (winner_id, winning_bid, winner_team_name) = round_bidders[0].clone();

                for bid in &mut self.bids {
                    if bid.listing_id == listing_id && bid.status == BidStatus::Active {
                        bid.status = BidStatus::Outbid;
                    }
                }

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

                self.listings[idx].current_price = winning_bid;
                self.listings[idx].current_bid_round = current_round;
                self.listings[idx].buyer_team_id = Some(winner_id);
                self.listings[idx].buyer_team_name = Some(winner_team_name.clone());

                round_bids.push(new_bid.clone());
                self.bids.push(new_bid);

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
                    importance: if draft_position <= 3 {
                        EventImportance::Major
                    } else {
                        EventImportance::Normal
                    },
                    round: current_round,
                    created_at: chrono::Utc::now().to_rfc3339(),
                });

                if current_round >= max_rounds {
                    self.finalize_listing_sale(idx, current_round, &mut new_events);
                    round_sales.push((draft_position, winner_id));
                }
            }
        }

        let new_wanted_events = self.generate_wanted_requests();
        new_events.extend(new_wanted_events);

        let wanted_results = self.process_wanted_requests(current_round, &mut new_events);

        let active_listings = self
            .listings
            .iter()
            .filter(|l| l.status == DraftListingStatus::Active)
            .count();
        let had_activity_this_round =
            !round_bids.is_empty() || !round_sales.is_empty() || !wanted_results.is_empty();
        let should_complete =
            current_round >= max_rounds || (active_listings == 0 && !had_activity_this_round);
        if should_complete {
            for w in &mut self.wanted_requests {
                if w.status == WantedStatus::Active {
                    w.status = WantedStatus::Expired;
                }
            }
            self.complete_auction(&mut new_events);
        }

        self.events.extend(new_events.clone());

        AuctionRoundResult {
            round: current_round,
            bids: round_bids,
            sales: round_sales,
            expirations: round_expirations,
            wanted_results,
            events: new_events,
            is_auction_complete: self.auction.status == AuctionStatus::Completed,
        }
    }

    pub(super) fn finalize_listing_sale(
        &mut self,
        idx: usize,
        round: u32,
        events: &mut Vec<DraftPickAuctionEvent>,
    ) {
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

            let listing_id = listing.id;
            for bid in &mut self.bids {
                if bid.listing_id == listing_id
                    && bid.bidder_team_id == buyer_id
                    && bid.status == BidStatus::Active
                {
                    bid.status = BidStatus::Won;
                }
            }

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
                importance: if draft_position <= 3 {
                    EventImportance::Breaking
                } else {
                    EventImportance::Major
                },
                round,
                created_at: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    pub fn fast_forward(&mut self) -> Vec<AuctionRoundResult> {
        let mut results = Vec::new();

        while self.auction.status == AuctionStatus::InProgress {
            let result = self.execute_round();
            results.push(result);
        }

        results
    }

    pub(super) fn complete_auction(&mut self, events: &mut Vec<DraftPickAuctionEvent>) {
        self.auction.status = AuctionStatus::Completed;
        self.auction.completed_at = Some(chrono::Utc::now().to_rfc3339());

        let sold_count = self
            .listings
            .iter()
            .filter(|l| l.status == DraftListingStatus::Sold)
            .count();
        let expired_count = self
            .listings
            .iter()
            .filter(|l| l.status == DraftListingStatus::Expired)
            .count();

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
}
