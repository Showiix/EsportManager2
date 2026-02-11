use serde::{Deserialize, Serialize};

/// 拍卖状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuctionStatus {
    Preparing,
    InProgress,
    Completed,
}

impl From<&str> for AuctionStatus {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "IN_PROGRESS" => AuctionStatus::InProgress,
            "COMPLETED" => AuctionStatus::Completed,
            _ => AuctionStatus::Preparing,
        }
    }
}

impl ToString for AuctionStatus {
    fn to_string(&self) -> String {
        match self {
            AuctionStatus::Preparing => "PREPARING".to_string(),
            AuctionStatus::InProgress => "IN_PROGRESS".to_string(),
            AuctionStatus::Completed => "COMPLETED".to_string(),
        }
    }
}

/// 挂牌状态 (选秀权拍卖专用)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DraftListingStatus {
    Pending,
    Active,
    Sold,
    Withdrawn,
    Expired,
}

impl From<&str> for DraftListingStatus {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "ACTIVE" => DraftListingStatus::Active,
            "SOLD" => DraftListingStatus::Sold,
            "WITHDRAWN" => DraftListingStatus::Withdrawn,
            "EXPIRED" => DraftListingStatus::Expired,
            _ => DraftListingStatus::Pending,
        }
    }
}

impl ToString for DraftListingStatus {
    fn to_string(&self) -> String {
        match self {
            DraftListingStatus::Pending => "PENDING".to_string(),
            DraftListingStatus::Active => "ACTIVE".to_string(),
            DraftListingStatus::Sold => "SOLD".to_string(),
            DraftListingStatus::Withdrawn => "WITHDRAWN".to_string(),
            DraftListingStatus::Expired => "EXPIRED".to_string(),
        }
    }
}

/// 竞拍状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BidStatus {
    Active,
    Outbid,
    Won,
    Withdrawn,
}

impl From<&str> for BidStatus {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "OUTBID" => BidStatus::Outbid,
            "WON" => BidStatus::Won,
            "WITHDRAWN" => BidStatus::Withdrawn,
            _ => BidStatus::Active,
        }
    }
}

impl ToString for BidStatus {
    fn to_string(&self) -> String {
        match self {
            BidStatus::Active => "ACTIVE".to_string(),
            BidStatus::Outbid => "OUTBID".to_string(),
            BidStatus::Won => "WON".to_string(),
            BidStatus::Withdrawn => "WITHDRAWN".to_string(),
        }
    }
}

/// 拍卖事件类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuctionEventType {
    AuctionStart,
    ListingCreated,
    BidPlaced,
    BidRaised,
    Sold,
    Withdrawn,
    Expired,
    AuctionEnd,
    WantedCreated,
    WantedAccepted,
    WantedRejected,
}

impl From<&str> for AuctionEventType {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "AUCTION_START" => AuctionEventType::AuctionStart,
            "LISTING_CREATED" => AuctionEventType::ListingCreated,
            "BID_PLACED" => AuctionEventType::BidPlaced,
            "BID_RAISED" => AuctionEventType::BidRaised,
            "SOLD" => AuctionEventType::Sold,
            "WITHDRAWN" => AuctionEventType::Withdrawn,
            "EXPIRED" => AuctionEventType::Expired,
            "AUCTION_END" => AuctionEventType::AuctionEnd,
            "WANTED_CREATED" => AuctionEventType::WantedCreated,
            "WANTED_ACCEPTED" => AuctionEventType::WantedAccepted,
            "WANTED_REJECTED" => AuctionEventType::WantedRejected,
            _ => AuctionEventType::AuctionStart,
        }
    }
}

impl ToString for AuctionEventType {
    fn to_string(&self) -> String {
        match self {
            AuctionEventType::AuctionStart => "AUCTION_START".to_string(),
            AuctionEventType::ListingCreated => "LISTING_CREATED".to_string(),
            AuctionEventType::BidPlaced => "BID_PLACED".to_string(),
            AuctionEventType::BidRaised => "BID_RAISED".to_string(),
            AuctionEventType::Sold => "SOLD".to_string(),
            AuctionEventType::Withdrawn => "WITHDRAWN".to_string(),
            AuctionEventType::Expired => "EXPIRED".to_string(),
            AuctionEventType::AuctionEnd => "AUCTION_END".to_string(),
            AuctionEventType::WantedCreated => "WANTED_CREATED".to_string(),
            AuctionEventType::WantedAccepted => "WANTED_ACCEPTED".to_string(),
            AuctionEventType::WantedRejected => "WANTED_REJECTED".to_string(),
        }
    }
}

/// 事件重要性
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventImportance {
    Breaking,
    Major,
    Normal,
    Minor,
}

impl From<&str> for EventImportance {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "BREAKING" => EventImportance::Breaking,
            "MAJOR" => EventImportance::Major,
            "MINOR" => EventImportance::Minor,
            _ => EventImportance::Normal,
        }
    }
}

impl ToString for EventImportance {
    fn to_string(&self) -> String {
        match self {
            EventImportance::Breaking => "BREAKING".to_string(),
            EventImportance::Major => "MAJOR".to_string(),
            EventImportance::Normal => "NORMAL".to_string(),
            EventImportance::Minor => "MINOR".to_string(),
        }
    }
}

/// 选秀权拍卖主记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPickAuction {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub region_id: u64,
    pub status: AuctionStatus,
    pub current_round: u32,
    pub total_rounds: u32,
    pub total_auctions: u32,
    pub successful_auctions: u32,
    pub total_revenue: i64,
    pub total_commission: i64,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

/// 选秀权挂牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPickListing {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub region_id: u64,
    pub auction_id: u64,
    pub seller_team_id: u64,
    pub seller_team_name: String,
    pub draft_position: u32,
    pub starting_price: i64,
    pub current_price: i64,
    pub min_increment: i64,
    pub status: DraftListingStatus,
    pub buyer_team_id: Option<u64>,
    pub buyer_team_name: Option<String>,
    pub final_price: Option<i64>,
    pub commission_fee: Option<i64>,
    pub seller_revenue: Option<i64>,
    pub current_bid_round: u32,
    pub created_at: String,
    pub sold_at: Option<String>,
}

/// 竞拍出价记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPickBid {
    pub id: u64,
    pub save_id: String,
    pub listing_id: u64,
    pub bidder_team_id: u64,
    pub bidder_team_name: String,
    pub bid_amount: i64,
    pub bid_round: u32,
    pub status: BidStatus,
    pub created_at: String,
}

/// 拍卖事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPickAuctionEvent {
    pub id: u64,
    pub save_id: String,
    pub auction_id: u64,
    pub listing_id: Option<u64>,
    pub event_type: AuctionEventType,
    pub team_id: Option<u64>,
    pub team_name: Option<String>,
    pub draft_position: Option<u32>,
    pub amount: Option<i64>,
    pub headline: String,
    pub description: String,
    pub importance: EventImportance,
    pub round: u32,
    pub created_at: String,
}

/// 签位价格配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPickPricing {
    pub position: u32,
    pub name: String,
    pub starting_price: i64,
    pub min_increment: i64,
}

/// 获取签位价格配置
/// 价格单位: 万
pub fn get_draft_pick_pricing() -> Vec<DraftPickPricing> {
    vec![
        DraftPickPricing {
            position: 1,
            name: "状元签".to_string(),
            starting_price: 2000_0000,
            min_increment: 200_0000,
        },
        DraftPickPricing {
            position: 2,
            name: "榜眼签".to_string(),
            starting_price: 1500_0000,
            min_increment: 150_0000,
        },
        DraftPickPricing {
            position: 3,
            name: "探花签".to_string(),
            starting_price: 1000_0000,
            min_increment: 100_0000,
        },
        DraftPickPricing {
            position: 4,
            name: "第4签".to_string(),
            starting_price: 700_0000,
            min_increment: 80_0000,
        },
        DraftPickPricing {
            position: 5,
            name: "第5签".to_string(),
            starting_price: 500_0000,
            min_increment: 60_0000,
        },
        DraftPickPricing {
            position: 6,
            name: "第6签".to_string(),
            starting_price: 400_0000,
            min_increment: 50_0000,
        },
        DraftPickPricing {
            position: 7,
            name: "第7签".to_string(),
            starting_price: 300_0000,
            min_increment: 40_0000,
        },
        DraftPickPricing {
            position: 8,
            name: "第8签".to_string(),
            starting_price: 250_0000,
            min_increment: 30_0000,
        },
        DraftPickPricing {
            position: 9,
            name: "第9签".to_string(),
            starting_price: 200_0000,
            min_increment: 25_0000,
        },
        DraftPickPricing {
            position: 10,
            name: "第10签".to_string(),
            starting_price: 150_0000,
            min_increment: 20_0000,
        },
        DraftPickPricing {
            position: 11,
            name: "第11签".to_string(),
            starting_price: 120_0000,
            min_increment: 15_0000,
        },
        DraftPickPricing {
            position: 12,
            name: "第12签".to_string(),
            starting_price: 100_0000,
            min_increment: 15_0000,
        },
        DraftPickPricing {
            position: 13,
            name: "第13签".to_string(),
            starting_price: 80_0000,
            min_increment: 10_0000,
        },
        DraftPickPricing {
            position: 14,
            name: "第14签".to_string(),
            starting_price: 60_0000,
            min_increment: 10_0000,
        },
    ]
}

/// 获取指定签位的价格
pub fn get_price_for_position(position: u32) -> Option<DraftPickPricing> {
    get_draft_pick_pricing()
        .into_iter()
        .find(|p| p.position == position)
}

/// 联盟佣金比例 (5%)
pub const COMMISSION_RATE: f64 = 0.05;

/// 计算佣金
pub fn calculate_commission(price: i64) -> i64 {
    (price as f64 * COMMISSION_RATE) as i64
}

/// 计算卖家收益 (扣除佣金后)
pub fn calculate_seller_revenue(price: i64) -> i64 {
    price - calculate_commission(price)
}

/// 获取签位名称
pub fn get_position_name(position: u32) -> String {
    match position {
        1 => "状元签".to_string(),
        2 => "榜眼签".to_string(),
        3 => "探花签".to_string(),
        _ => format!("第{}签", position),
    }
}

/// 求购状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WantedStatus {
    Active,
    Fulfilled,
    Rejected,
    Expired,
}

impl From<&str> for WantedStatus {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "FULFILLED" => WantedStatus::Fulfilled,
            "REJECTED" => WantedStatus::Rejected,
            "EXPIRED" => WantedStatus::Expired,
            _ => WantedStatus::Active,
        }
    }
}

impl ToString for WantedStatus {
    fn to_string(&self) -> String {
        match self {
            WantedStatus::Active => "ACTIVE".to_string(),
            WantedStatus::Fulfilled => "FULFILLED".to_string(),
            WantedStatus::Rejected => "REJECTED".to_string(),
            WantedStatus::Expired => "EXPIRED".to_string(),
        }
    }
}

/// 选秀权求购请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPickWanted {
    pub id: u64,
    pub save_id: String,
    pub season_id: u64,
    pub region_id: u64,
    pub auction_id: u64,
    pub buyer_team_id: u64,
    pub buyer_team_name: String,
    pub target_position: u32,
    pub offer_price: i64,
    pub reason: String,
    pub status: WantedStatus,
    pub holder_team_id: u64,
    pub holder_team_name: String,
    pub response_reason: Option<String>,
    pub final_price: Option<i64>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}
