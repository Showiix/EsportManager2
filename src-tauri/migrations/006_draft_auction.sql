-- 选秀权拍卖系统迁移
-- 版本: 006
-- 日期: 2025-01

-- ============================================
-- 选秀权拍卖主表
-- ============================================
CREATE TABLE IF NOT EXISTS draft_pick_auctions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    region_id INTEGER NOT NULL,

    -- 拍卖状态: PREPARING/IN_PROGRESS/COMPLETED
    status TEXT NOT NULL DEFAULT 'PREPARING',
    current_round INTEGER NOT NULL DEFAULT 0,
    total_rounds INTEGER NOT NULL DEFAULT 3,

    -- 统计数据
    total_auctions INTEGER NOT NULL DEFAULT 0,
    successful_auctions INTEGER NOT NULL DEFAULT 0,
    total_revenue INTEGER NOT NULL DEFAULT 0,
    total_commission INTEGER NOT NULL DEFAULT 0,

    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    UNIQUE(save_id, season_id, region_id)
);

-- ============================================
-- 选秀权挂牌表
-- ============================================
CREATE TABLE IF NOT EXISTS draft_pick_listings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    season_id INTEGER NOT NULL,
    region_id INTEGER NOT NULL,
    auction_id INTEGER NOT NULL,

    -- 卖家信息
    seller_team_id INTEGER NOT NULL,
    seller_team_name TEXT NOT NULL,
    draft_position INTEGER NOT NULL,

    -- 价格信息
    starting_price INTEGER NOT NULL,
    current_price INTEGER NOT NULL,
    min_increment INTEGER NOT NULL,

    -- 状态: PENDING/ACTIVE/SOLD/WITHDRAWN/EXPIRED
    status TEXT NOT NULL DEFAULT 'PENDING',

    -- 成交信息
    buyer_team_id INTEGER,
    buyer_team_name TEXT,
    final_price INTEGER,
    commission_fee INTEGER,
    seller_revenue INTEGER,

    current_bid_round INTEGER NOT NULL DEFAULT 0,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    sold_at TEXT,

    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (auction_id) REFERENCES draft_pick_auctions(id) ON DELETE CASCADE,
    FOREIGN KEY (seller_team_id) REFERENCES teams(id) ON DELETE CASCADE,
    FOREIGN KEY (buyer_team_id) REFERENCES teams(id) ON DELETE SET NULL
);

-- ============================================
-- 竞拍出价记录表
-- ============================================
CREATE TABLE IF NOT EXISTS draft_pick_bids (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    listing_id INTEGER NOT NULL,

    -- 出价信息
    bidder_team_id INTEGER NOT NULL,
    bidder_team_name TEXT NOT NULL,
    bid_amount INTEGER NOT NULL,
    bid_round INTEGER NOT NULL,

    -- 状态: ACTIVE/OUTBID/WON/WITHDRAWN
    status TEXT NOT NULL DEFAULT 'ACTIVE',

    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (listing_id) REFERENCES draft_pick_listings(id) ON DELETE CASCADE,
    FOREIGN KEY (bidder_team_id) REFERENCES teams(id) ON DELETE CASCADE
);

-- ============================================
-- 拍卖事件表（用于新闻播报）
-- ============================================
CREATE TABLE IF NOT EXISTS draft_pick_auction_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id TEXT NOT NULL,
    auction_id INTEGER NOT NULL,
    listing_id INTEGER,

    -- 事件类型: AUCTION_START/LISTING_CREATED/BID_PLACED/BID_RAISED/SOLD/WITHDRAWN/EXPIRED/AUCTION_END
    event_type TEXT NOT NULL,

    -- 事件详情
    team_id INTEGER,
    team_name TEXT,
    draft_position INTEGER,
    amount INTEGER,

    -- 新闻信息
    headline TEXT NOT NULL,
    description TEXT NOT NULL,
    -- 重要程度: BREAKING/MAJOR/NORMAL/MINOR
    importance TEXT NOT NULL DEFAULT 'NORMAL',

    round INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
    FOREIGN KEY (auction_id) REFERENCES draft_pick_auctions(id) ON DELETE CASCADE
);

-- ============================================
-- 索引
-- ============================================
CREATE INDEX IF NOT EXISTS idx_draft_pick_auctions_save_season_region
    ON draft_pick_auctions(save_id, season_id, region_id);

CREATE INDEX IF NOT EXISTS idx_draft_pick_listings_auction
    ON draft_pick_listings(auction_id);

CREATE INDEX IF NOT EXISTS idx_draft_pick_listings_status
    ON draft_pick_listings(status);

CREATE INDEX IF NOT EXISTS idx_draft_pick_bids_listing
    ON draft_pick_bids(listing_id);

CREATE INDEX IF NOT EXISTS idx_draft_pick_auction_events_auction
    ON draft_pick_auction_events(auction_id);
