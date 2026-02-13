import { invokeCommand } from './client'

// ========================================
// Draft Pick Auction System (选秀权拍卖)
// ========================================

// 拍卖挂牌信息
export interface AuctionListing {
  id: number
  seller_team_id: number
  seller_team_name: string
  draft_position: number
  position_name: string
  starting_price: number
  current_price: number
  min_increment: number
  status: string  // PENDING/ACTIVE/SOLD/WITHDRAWN/EXPIRED
  buyer_team_id: number | null
  buyer_team_name: string | null
  final_price: number | null
  current_bid_round: number
}

// 拍卖状态信息
export interface AuctionStatus {
  id: number
  status: string  // PREPARING/IN_PROGRESS/COMPLETED
  current_round: number
  total_rounds: number
  total_auctions: number
  successful_auctions: number
  total_revenue: number
  total_commission: number
  listings: AuctionListing[]
}

// 拍卖事件
export interface AuctionEvent {
  id: number
  event_type: string  // AUCTION_START/LISTING_CREATED/BID_PLACED/BID_RAISED/SOLD/WITHDRAWN/EXPIRED/AUCTION_END
  team_id: number | null
  team_name: string | null
  draft_position: number | null
  position_name: string | null
  amount: number | null
  headline: string
  description: string
  importance: string  // BREAKING/MAJOR/NORMAL/MINOR
  round: number
  created_at: string
}

// 签位价格配置
export interface DraftPickPrice {
  position: number
  name: string
  starting_price: number
  min_increment: number
}

export const draftAuctionApi = {
  // 获取签位价格配置
  getDraftPickPrices: () =>
    invokeCommand<DraftPickPrice[]>('get_draft_pick_prices'),

  // 开始拍卖
  startAuction: (regionId: number) =>
    invokeCommand<AuctionStatus>('start_draft_auction', { regionId }),

  // 执行一轮竞拍
  executeRound: (regionId: number) =>
    invokeCommand<AuctionStatus>('execute_auction_round', { regionId }),

  // 快进完成所有轮次
  fastForward: (regionId: number) =>
    invokeCommand<AuctionStatus>('fast_forward_auction', { regionId }),

  // 获取拍卖状态
  getStatus: (regionId: number) =>
    invokeCommand<AuctionStatus | null>('get_auction_status', { regionId }),

  // 获取拍卖事件
  getEvents: (regionId: number) =>
    invokeCommand<AuctionEvent[]>('get_auction_events', { regionId }),

  // 完成拍卖并更新选秀顺位
  finalizeAuction: (regionId: number) =>
    invokeCommand<boolean>('finalize_auction', { regionId }),

  getWantedRequests: (regionId: number, seasonId?: number) =>
    invokeCommand<WantedRequestInfo[]>('get_auction_wanted_requests', { regionId, seasonId }),
}

export interface WantedRequestInfo {
  id: number
  buyer_team_id: number
  buyer_team_name: string
  target_position: number
  offer_price: number
  reason: string
  status: string
  holder_team_id: number
  holder_team_name: string
  response_reason: string | null
  final_price: number | null
}
