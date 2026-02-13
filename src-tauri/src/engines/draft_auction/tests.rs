//! 选秀权拍卖引擎单元测试

#[cfg(test)]
mod tests {
    use crate::models::{AuctionStatus, FinancialStatus};
    use crate::engines::draft_auction::DraftAuctionEngine;

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
        assert_eq!(
            FinancialStatus::from_balance(20_000_000),
            FinancialStatus::Wealthy
        );
        assert_eq!(
            FinancialStatus::from_balance(8_000_000),
            FinancialStatus::Healthy
        );
        assert_eq!(
            FinancialStatus::from_balance(3_000_000),
            FinancialStatus::Tight
        );
        assert_eq!(
            FinancialStatus::from_balance(500_000),
            FinancialStatus::Deficit
        );
        assert_eq!(
            FinancialStatus::from_balance(-1_000_000),
            FinancialStatus::Bankrupt
        );
    }

    #[test]
    fn test_commission_calculation() {
        use crate::models::{calculate_commission, calculate_seller_revenue};
        let price = 1000_0000;
        let commission = calculate_commission(price);
        let revenue = calculate_seller_revenue(price);

        assert_eq!(commission, 50_0000);
        assert_eq!(revenue, 950_0000);
    }
}
