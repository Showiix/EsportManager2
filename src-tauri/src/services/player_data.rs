//! 真实选手数据
//! 包含四大赛区主要战队的真实选手信息
//! 数据来源：初始化数据库.md

use crate::models::Position;
use crate::engines::TraitType;

/// 选手配置信息
pub struct PlayerConfig {
    pub game_id: &'static str,
    pub real_name: Option<&'static str>,
    pub nationality: &'static str,
    pub position: Position,
    pub age: u8,
    pub ability: u8,
    pub potential: u8,
    pub is_starter: bool,
    pub traits: &'static [TraitType],  // 预定义特性列表
}

/// 获取战队选手数据
/// 返回 (team_short_name, players)
pub fn get_team_players(team_short_name: &str) -> Vec<PlayerConfig> {
    match team_short_name {
        // ==================== LPL ====================
        "TES" => vec![
            PlayerConfig { game_id: "Zeka", real_name: Some("林泽凯"), nationality: "CN", position: Position::Top, age: 24, ability: 67, potential: 68, is_starter: true, traits: &[TraitType::Clutch] },
            PlayerConfig { game_id: "Xun", real_name: Some("彭啸"), nationality: "CN", position: Position::Jug, age: 24, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Consistent] },
            PlayerConfig { game_id: "Foxy", real_name: Some("陈睿"), nationality: "CN", position: Position::Mid, age: 22, ability: 69, potential: 72, is_starter: true, traits: &[TraitType::Clutch, TraitType::FastStarter, TraitType::RisingStar] },
            PlayerConfig { game_id: "Light", real_name: Some("王光宇"), nationality: "CN", position: Position::Adc, age: 23, ability: 68, potential: 69, is_starter: true, traits: &[TraitType::Consistent] },
            PlayerConfig { game_id: "Mark", real_name: Some("凌旭"), nationality: "CN", position: Position::Sup, age: 26, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::TeamLeader, TraitType::Veteran] },
        ],
        "BLG" => vec![
            PlayerConfig { game_id: "Bin", real_name: Some("陈泽彬"), nationality: "CN", position: Position::Top, age: 23, ability: 69, potential: 70, is_starter: true, traits: &[TraitType::Explosive, TraitType::ComebackKing] },
            PlayerConfig { game_id: "Junjia", real_name: Some("黄俊杰"), nationality: "CN", position: Position::Jug, age: 22, ability: 64, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Yagao", real_name: Some("刘青松"), nationality: "CN", position: Position::Mid, age: 26, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Consistent, TraitType::Veteran] },
            PlayerConfig { game_id: "Elk", real_name: Some("赵嘉豪"), nationality: "CN", position: Position::Adc, age: 22, ability: 68, potential: 71, is_starter: true, traits: &[TraitType::Clutch, TraitType::RisingStar] },
            PlayerConfig { game_id: "ON", real_name: Some("骆文俊"), nationality: "CN", position: Position::Sup, age: 23, ability: 64, potential: 67, is_starter: true, traits: &[] },
        ],
        "JDG" => vec![
            PlayerConfig { game_id: "369", real_name: Some("白家浩"), nationality: "CN", position: Position::Top, age: 25, ability: 66, potential: 66, is_starter: true, traits: &[TraitType::SlowStarter, TraitType::Veteran] },
            PlayerConfig { game_id: "Kanavi", real_name: Some("徐进赫"), nationality: "KR", position: Position::Jug, age: 24, ability: 70, potential: 72, is_starter: true, traits: &[TraitType::Clutch, TraitType::MentalFortress] },
            PlayerConfig { game_id: "Knight", real_name: Some("卓定"), nationality: "CN", position: Position::Mid, age: 24, ability: 71, potential: 72, is_starter: true, traits: &[TraitType::Clutch, TraitType::TeamLeader] },
            PlayerConfig { game_id: "Ruler", real_name: Some("朴载赫"), nationality: "KR", position: Position::Adc, age: 27, ability: 69, potential: 69, is_starter: true, traits: &[TraitType::Consistent, TraitType::Veteran, TraitType::TeamLeader] },
            PlayerConfig { game_id: "Missing", real_name: Some("娄运峰"), nationality: "CN", position: Position::Sup, age: 24, ability: 65, potential: 67, is_starter: true, traits: &[] },
        ],
        "WBG" => vec![
            PlayerConfig { game_id: "TheShy", real_name: Some("姜承録"), nationality: "KR", position: Position::Top, age: 27, ability: 69, potential: 69, is_starter: true, traits: &[TraitType::Explosive, TraitType::Volatile, TraitType::Veteran] },
            PlayerConfig { game_id: "Weiwei", real_name: Some("王成勇"), nationality: "CN", position: Position::Jug, age: 24, ability: 64, potential: 66, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Xiaohu", real_name: Some("李虎"), nationality: "CN", position: Position::Mid, age: 28, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran, TraitType::Consistent, TraitType::TeamLeader] },
            PlayerConfig { game_id: "Aphelios", real_name: Some("张明轩"), nationality: "CN", position: Position::Adc, age: 22, ability: 66, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Crisp", real_name: Some("刘青松"), nationality: "CN", position: Position::Sup, age: 27, ability: 66, potential: 66, is_starter: true, traits: &[TraitType::TeamLeader, TraitType::Veteran] },
        ],
        "RNG" => vec![
            PlayerConfig { game_id: "Breathe", real_name: Some("陈晨"), nationality: "CN", position: Position::Top, age: 24, ability: 64, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Wei", real_name: Some("闫扬威"), nationality: "CN", position: Position::Jug, age: 24, ability: 67, potential: 69, is_starter: true, traits: &[TraitType::Clutch] },
            PlayerConfig { game_id: "Angel", real_name: Some("郑凌熙"), nationality: "CN", position: Position::Mid, age: 25, ability: 67, potential: 68, is_starter: true, traits: &[TraitType::FastStarter, TraitType::Veteran] },
            PlayerConfig { game_id: "GALA", real_name: Some("陈伟"), nationality: "CN", position: Position::Adc, age: 24, ability: 68, potential: 69, is_starter: true, traits: &[TraitType::Clutch, TraitType::Explosive] },
            PlayerConfig { game_id: "Ming", real_name: Some("史森明"), nationality: "CN", position: Position::Sup, age: 27, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::TeamLeader, TraitType::Veteran] },
        ],
        "FPX" => vec![
            PlayerConfig { game_id: "Xiaolaohu", real_name: Some("胡硕杰"), nationality: "CN", position: Position::Top, age: 22, ability: 63, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Tian", real_name: Some("高天亮"), nationality: "CN", position: Position::Jug, age: 25, ability: 69, potential: 69, is_starter: true, traits: &[TraitType::Clutch, TraitType::Fragile] },
            PlayerConfig { game_id: "Care", real_name: Some("曹睿"), nationality: "CN", position: Position::Mid, age: 21, ability: 64, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Lwx", real_name: Some("林炜翔"), nationality: "CN", position: Position::Adc, age: 26, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Consistent, TraitType::Veteran] },
            PlayerConfig { game_id: "Hang", real_name: Some("牛宝权"), nationality: "CN", position: Position::Sup, age: 22, ability: 62, potential: 67, is_starter: true, traits: &[] },
        ],
        "LNG" => vec![
            PlayerConfig { game_id: "Zika", real_name: Some("孙振楷"), nationality: "CN", position: Position::Top, age: 23, ability: 64, potential: 68, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Tarzan", real_name: Some("李承勇"), nationality: "KR", position: Position::Jug, age: 26, ability: 68, potential: 68, is_starter: true, traits: &[TraitType::Consistent, TraitType::Ironman, TraitType::Veteran] },
            PlayerConfig { game_id: "Scout", real_name: Some("李汭燦"), nationality: "KR", position: Position::Mid, age: 26, ability: 69, potential: 69, is_starter: true, traits: &[TraitType::Clutch, TraitType::MentalFortress, TraitType::Veteran] },
            PlayerConfig { game_id: "Huanfeng", real_name: Some("唐焕峰"), nationality: "CN", position: Position::Adc, age: 23, ability: 66, potential: 69, is_starter: true, traits: &[TraitType::Explosive] },
            PlayerConfig { game_id: "Lvmao", real_name: Some("左名豪"), nationality: "CN", position: Position::Sup, age: 25, ability: 64, potential: 64, is_starter: true, traits: &[] },
        ],
        "TT" => vec![
            PlayerConfig { game_id: "HOYA", real_name: Some("李浩渊"), nationality: "CN", position: Position::Top, age: 23, ability: 62, potential: 65, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Karsa", real_name: Some("高学成"), nationality: "TW", position: Position::Jug, age: 28, ability: 65, potential: 65, is_starter: true, traits: &[TraitType::Veteran, TraitType::TeamLeader] },
            PlayerConfig { game_id: "Ucal", real_name: Some("崔祐齐"), nationality: "KR", position: Position::Mid, age: 24, ability: 63, potential: 66, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Beichuan", real_name: Some("余峻嘉"), nationality: "CN", position: Position::Adc, age: 22, ability: 61, potential: 65, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "QiuQiu", real_name: Some("赵志铭"), nationality: "CN", position: Position::Sup, age: 23, ability: 60, potential: 64, is_starter: true, traits: &[] },
        ],
        "IG" => vec![
            PlayerConfig { game_id: "Ning", real_name: Some("高振宁"), nationality: "CN", position: Position::Top, age: 26, ability: 65, potential: 65, is_starter: true, traits: &[TraitType::Volatile, TraitType::Veteran] },
            PlayerConfig { game_id: "River", real_name: Some("金东宇"), nationality: "KR", position: Position::Jug, age: 24, ability: 66, potential: 68, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Rookie", real_name: Some("宋义进"), nationality: "KR", position: Position::Mid, age: 28, ability: 69, potential: 69, is_starter: true, traits: &[TraitType::Clutch, TraitType::Veteran, TraitType::TeamLeader] },
            PlayerConfig { game_id: "Jinjiao", real_name: Some("王杰"), nationality: "CN", position: Position::Adc, age: 25, ability: 64, potential: 64, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Baolan", real_name: Some("王柳羿"), nationality: "CN", position: Position::Sup, age: 26, ability: 63, potential: 63, is_starter: true, traits: &[TraitType::Veteran] },
        ],
        "UP" => vec![
            PlayerConfig { game_id: "Zoom", real_name: Some("张星冉"), nationality: "CN", position: Position::Top, age: 26, ability: 64, potential: 64, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Heng", real_name: Some("闵衡"), nationality: "CN", position: Position::Jug, age: 22, ability: 63, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Creme", real_name: Some("姚浩仁"), nationality: "CN", position: Position::Mid, age: 21, ability: 64, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Betty", real_name: Some("卢禹宏"), nationality: "TW", position: Position::Adc, age: 26, ability: 65, potential: 65, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "SwordArt", real_name: Some("史益豪"), nationality: "TW", position: Position::Sup, age: 28, ability: 62, potential: 62, is_starter: true, traits: &[TraitType::Veteran, TraitType::TeamLeader] },
        ],
        "AL" => vec![
            PlayerConfig { game_id: "Chenlun", real_name: Some("陈伦"), nationality: "CN", position: Position::Top, age: 24, ability: 62, potential: 64, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Xiang", real_name: Some("罗珦"), nationality: "CN", position: Position::Jug, age: 24, ability: 62, potential: 65, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "ZekaAL", real_name: Some("林泽凯"), nationality: "CN", position: Position::Mid, age: 24, ability: 67, potential: 68, is_starter: true, traits: &[TraitType::Clutch] },
            PlayerConfig { game_id: "Photic", real_name: Some("王涛"), nationality: "CN", position: Position::Adc, age: 24, ability: 64, potential: 66, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Iwandy", real_name: Some("刘万迪"), nationality: "CN", position: Position::Sup, age: 24, ability: 61, potential: 63, is_starter: true, traits: &[] },
        ],
        "NIP" => vec![
            PlayerConfig { game_id: "Flandre", real_name: Some("李炫君"), nationality: "CN", position: Position::Top, age: 26, ability: 66, potential: 66, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Swift", real_name: Some("崔润锡"), nationality: "KR", position: Position::Jug, age: 24, ability: 64, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Maple", real_name: Some("黄熠棠"), nationality: "TW", position: Position::Mid, age: 27, ability: 66, potential: 66, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Smlz", real_name: Some("韩金"), nationality: "CN", position: Position::Adc, age: 27, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Peter", real_name: Some("张博文"), nationality: "CN", position: Position::Sup, age: 23, ability: 62, potential: 65, is_starter: true, traits: &[] },
        ],
        "MR" => vec![
            PlayerConfig { game_id: "Adder", real_name: Some("周锐"), nationality: "CN", position: Position::Top, age: 24, ability: 62, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Shadow", real_name: Some("赵志强"), nationality: "CN", position: Position::Jug, age: 24, ability: 65, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "YunO", real_name: Some("吴云舟"), nationality: "CN", position: Position::Mid, age: 24, ability: 67, potential: 69, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Ghost", real_name: Some("杨洋"), nationality: "CN", position: Position::Adc, age: 24, ability: 64, potential: 66, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "KeriaMR", real_name: Some("柳岷析"), nationality: "KR", position: Position::Sup, age: 24, ability: 66, potential: 68, is_starter: true, traits: &[] },
        ],
        "EDG" => vec![
            PlayerConfig { game_id: "Ale", real_name: Some("胡嘉乐"), nationality: "CN", position: Position::Top, age: 23, ability: 67, potential: 69, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Jiejie", real_name: Some("赵礼杰"), nationality: "CN", position: Position::Jug, age: 24, ability: 67, potential: 69, is_starter: true, traits: &[TraitType::Clutch] },
            PlayerConfig { game_id: "FoFo", real_name: Some("余峻嘉"), nationality: "TW", position: Position::Mid, age: 26, ability: 68, potential: 68, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Leave", real_name: Some("黄馨弘"), nationality: "CN", position: Position::Adc, age: 21, ability: 66, potential: 70, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Meiko", real_name: Some("田野"), nationality: "CN", position: Position::Sup, age: 27, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran, TraitType::TeamLeader] },
        ],

        // ==================== LCK ====================
        "T1" => vec![
            PlayerConfig { game_id: "Zeus", real_name: Some("崔祐齐"), nationality: "KR", position: Position::Top, age: 21, ability: 68, potential: 72, is_starter: true, traits: &[TraitType::Clutch, TraitType::Explosive, TraitType::RisingStar] },
            PlayerConfig { game_id: "Oner", real_name: Some("文炫竣"), nationality: "KR", position: Position::Jug, age: 22, ability: 67, potential: 70, is_starter: true, traits: &[TraitType::Consistent, TraitType::RisingStar] },
            PlayerConfig { game_id: "Faker", real_name: Some("李相赫"), nationality: "KR", position: Position::Mid, age: 28, ability: 72, potential: 72, is_starter: true, traits: &[TraitType::Clutch, TraitType::MentalFortress, TraitType::TeamLeader, TraitType::Veteran] },
            PlayerConfig { game_id: "Gumayusi", real_name: Some("李玟炯"), nationality: "KR", position: Position::Adc, age: 22, ability: 67, potential: 70, is_starter: true, traits: &[TraitType::Explosive, TraitType::RisingStar] },
            PlayerConfig { game_id: "Keria", real_name: Some("柳岷析"), nationality: "KR", position: Position::Sup, age: 21, ability: 69, potential: 72, is_starter: true, traits: &[TraitType::Consistent, TraitType::FastStarter, TraitType::RisingStar] },
        ],
        "DRX" => vec![
            PlayerConfig { game_id: "Kingen", real_name: Some("黄星勋"), nationality: "KR", position: Position::Top, age: 25, ability: 66, potential: 67, is_starter: true, traits: &[TraitType::ComebackKing] },
            PlayerConfig { game_id: "Pyosik", real_name: Some("朴相赫"), nationality: "KR", position: Position::Jug, age: 24, ability: 65, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Quad", real_name: Some("金奎成"), nationality: "KR", position: Position::Mid, age: 21, ability: 64, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Deft", real_name: Some("金赫奎"), nationality: "KR", position: Position::Adc, age: 28, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran, TraitType::Consistent, TraitType::Clutch] },
            PlayerConfig { game_id: "BeryL", real_name: Some("赵容柱"), nationality: "KR", position: Position::Sup, age: 27, ability: 64, potential: 64, is_starter: true, traits: &[TraitType::Veteran] },
        ],
        "DK" => vec![
            PlayerConfig { game_id: "Canna", real_name: Some("金昌东"), nationality: "KR", position: Position::Top, age: 24, ability: 65, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Canyon", real_name: Some("金建敷"), nationality: "KR", position: Position::Jug, age: 23, ability: 69, potential: 72, is_starter: true, traits: &[TraitType::Clutch, TraitType::Consistent, TraitType::FastStarter] },
            PlayerConfig { game_id: "ShowMaker", real_name: Some("许秀"), nationality: "KR", position: Position::Mid, age: 24, ability: 69, potential: 71, is_starter: true, traits: &[TraitType::Clutch, TraitType::Explosive] },
            PlayerConfig { game_id: "Teddy", real_name: Some("朴志秀"), nationality: "KR", position: Position::Adc, age: 26, ability: 66, potential: 66, is_starter: true, traits: &[TraitType::Consistent, TraitType::Veteran] },
            PlayerConfig { game_id: "Kellin", real_name: Some("金焕熙"), nationality: "KR", position: Position::Sup, age: 25, ability: 64, potential: 65, is_starter: true, traits: &[] },
        ],
        "GEN" => vec![
            PlayerConfig { game_id: "Doran", real_name: Some("崔佑振"), nationality: "KR", position: Position::Top, age: 24, ability: 66, potential: 67, is_starter: true, traits: &[TraitType::Consistent] },
            PlayerConfig { game_id: "Peanut", real_name: Some("韩王浩"), nationality: "KR", position: Position::Jug, age: 27, ability: 68, potential: 68, is_starter: true, traits: &[TraitType::Veteran, TraitType::Clutch] },
            PlayerConfig { game_id: "Chovy", real_name: Some("郑志勋"), nationality: "KR", position: Position::Mid, age: 23, ability: 70, potential: 73, is_starter: true, traits: &[TraitType::Consistent, TraitType::Ironman] },
            PlayerConfig { game_id: "Peyz", real_name: Some("金修奂"), nationality: "KR", position: Position::Adc, age: 20, ability: 67, potential: 71, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Delight", real_name: Some("柳焕硕"), nationality: "KR", position: Position::Sup, age: 24, ability: 65, potential: 67, is_starter: true, traits: &[] },
        ],
        "KT" => vec![
            PlayerConfig { game_id: "Kiin", real_name: Some("金奇敏"), nationality: "KR", position: Position::Top, age: 26, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Consistent, TraitType::Veteran] },
            PlayerConfig { game_id: "Cuzz", real_name: Some("闵景焕"), nationality: "KR", position: Position::Jug, age: 26, ability: 65, potential: 65, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Bdd", real_name: Some("郭普成"), nationality: "KR", position: Position::Mid, age: 26, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran, TraitType::SlowStarter] },
            PlayerConfig { game_id: "Aiming", real_name: Some("金河润"), nationality: "KR", position: Position::Adc, age: 24, ability: 66, potential: 67, is_starter: true, traits: &[TraitType::Consistent] },
            PlayerConfig { game_id: "Lehends", real_name: Some("孙施尤"), nationality: "KR", position: Position::Sup, age: 27, ability: 64, potential: 64, is_starter: true, traits: &[TraitType::Veteran] },
        ],
        "KF" => vec![
            PlayerConfig { game_id: "DuDu", real_name: Some("朴晙锡"), nationality: "KR", position: Position::Top, age: 23, ability: 63, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Youngjin", real_name: Some("金永振"), nationality: "KR", position: Position::Jug, age: 22, ability: 64, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "BuLLDoG", real_name: Some("李宰焕"), nationality: "KR", position: Position::Mid, age: 23, ability: 62, potential: 66, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Viper", real_name: Some("朴到贤"), nationality: "KR", position: Position::Adc, age: 24, ability: 69, potential: 70, is_starter: true, traits: &[TraitType::Consistent, TraitType::Clutch] },
            PlayerConfig { game_id: "Hena", real_name: Some("金东宇"), nationality: "KR", position: Position::Sup, age: 24, ability: 63, potential: 64, is_starter: true, traits: &[] },
        ],
        "SB" => vec![
            PlayerConfig { game_id: "Rascal", real_name: Some("金光熙"), nationality: "KR", position: Position::Top, age: 27, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "OnFleek", real_name: Some("申东旭"), nationality: "KR", position: Position::Jug, age: 22, ability: 62, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "ZekaSB", real_name: Some("金建佑"), nationality: "KR", position: Position::Mid, age: 22, ability: 67, potential: 71, is_starter: true, traits: &[TraitType::Clutch, TraitType::RisingStar] },
            PlayerConfig { game_id: "Prince", real_name: Some("李载宪"), nationality: "KR", position: Position::Adc, age: 24, ability: 64, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Kael", real_name: Some("朴成勋"), nationality: "KR", position: Position::Sup, age: 23, ability: 62, potential: 65, is_starter: true, traits: &[] },
        ],
        "BRO" => vec![
            PlayerConfig { game_id: "Morgan", real_name: Some("李旻奭"), nationality: "KR", position: Position::Top, age: 24, ability: 64, potential: 66, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "UmTi", real_name: Some("严宰镒"), nationality: "KR", position: Position::Jug, age: 24, ability: 63, potential: 65, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Karis", real_name: Some("金泰珉"), nationality: "KR", position: Position::Mid, age: 22, ability: 64, potential: 68, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Envyy", real_name: Some("金俊熙"), nationality: "KR", position: Position::Adc, age: 21, ability: 65, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Effort", real_name: Some("金范洙"), nationality: "KR", position: Position::Sup, age: 25, ability: 64, potential: 64, is_starter: true, traits: &[TraitType::Veteran] },
        ],
        "NS" => vec![
            PlayerConfig { game_id: "Croc", real_name: Some("金건우"), nationality: "KR", position: Position::Top, age: 23, ability: 64, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Sylvie", real_name: Some("朴世河"), nationality: "KR", position: Position::Jug, age: 21, ability: 63, potential: 68, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Seo", real_name: Some("徐承淏"), nationality: "KR", position: Position::Mid, age: 21, ability: 64, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Luger", real_name: Some("金在焕"), nationality: "KR", position: Position::Adc, age: 22, ability: 66, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Irean", real_name: Some("金民석"), nationality: "KR", position: Position::Sup, age: 24, ability: 62, potential: 64, is_starter: true, traits: &[] },
        ],
        "BNK" => vec![
            PlayerConfig { game_id: "Aster", real_name: Some("朴星旿"), nationality: "KR", position: Position::Top, age: 23, ability: 64, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Bonnie", real_name: Some("李炳权"), nationality: "KR", position: Position::Jug, age: 22, ability: 62, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "FATE", real_name: Some("金泰旻"), nationality: "KR", position: Position::Mid, age: 24, ability: 64, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Taeyoon", real_name: Some("尹泰允"), nationality: "KR", position: Position::Adc, age: 23, ability: 63, potential: 66, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Jun", real_name: Some("金俊熙"), nationality: "KR", position: Position::Sup, age: 24, ability: 62, potential: 64, is_starter: true, traits: &[] },
        ],
        "FX" => vec![
            PlayerConfig { game_id: "DuSt", real_name: Some("金성훈"), nationality: "KR", position: Position::Top, age: 22, ability: 62, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Ruin", real_name: Some("崔재훈"), nationality: "KR", position: Position::Jug, age: 23, ability: 63, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Clozer", real_name: Some("金건휘"), nationality: "KR", position: Position::Mid, age: 21, ability: 65, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Iseul", real_name: Some("李슬기"), nationality: "KR", position: Position::Adc, age: 22, ability: 64, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Plead", real_name: Some("金민수"), nationality: "KR", position: Position::Sup, age: 24, ability: 61, potential: 63, is_starter: true, traits: &[] },
        ],
        "LZ" => vec![
            PlayerConfig { game_id: "Khan", real_name: Some("金东河"), nationality: "KR", position: Position::Top, age: 28, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran, TraitType::Explosive] },
            PlayerConfig { game_id: "Lyn", real_name: Some("朴정현"), nationality: "KR", position: Position::Jug, age: 24, ability: 64, potential: 66, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Gori", real_name: Some("权暋宇"), nationality: "KR", position: Position::Mid, age: 23, ability: 66, potential: 69, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "PraY", real_name: Some("金钟仁"), nationality: "KR", position: Position::Adc, age: 29, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran, TraitType::Consistent] },
            PlayerConfig { game_id: "Gorilla", real_name: Some("崔寅旭"), nationality: "KR", position: Position::Sup, age: 29, ability: 65, potential: 65, is_starter: true, traits: &[TraitType::Veteran, TraitType::TeamLeader] },
        ],
        "HLE" => vec![
            PlayerConfig { game_id: "Dubu", real_name: Some("金东贤"), nationality: "KR", position: Position::Top, age: 21, ability: 67, potential: 70, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Clid", real_name: Some("金泰珉"), nationality: "KR", position: Position::Jug, age: 26, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran, TraitType::Consistent] },
            PlayerConfig { game_id: "Mireu", real_name: Some("朴承焕"), nationality: "KR", position: Position::Mid, age: 20, ability: 69, potential: 73, is_starter: true, traits: &[TraitType::RisingStar, TraitType::Clutch] },
            PlayerConfig { game_id: "Route", real_name: Some("崔在宇"), nationality: "KR", position: Position::Adc, age: 21, ability: 68, potential: 72, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Moham", real_name: Some("李俊熙"), nationality: "KR", position: Position::Sup, age: 22, ability: 65, potential: 68, is_starter: true, traits: &[] },
        ],
        "AF" => vec![
            PlayerConfig { game_id: "Clear", real_name: Some("朴成浩"), nationality: "KR", position: Position::Top, age: 21, ability: 66, potential: 70, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Ellim", real_name: Some("崔允浩"), nationality: "KR", position: Position::Jug, age: 23, ability: 64, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Fly", real_name: Some("金相仁"), nationality: "KR", position: Position::Mid, age: 26, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Leo", real_name: Some("韩坚熙"), nationality: "KR", position: Position::Adc, age: 20, ability: 65, potential: 70, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Life", real_name: Some("金正民"), nationality: "KR", position: Position::Sup, age: 24, ability: 64, potential: 66, is_starter: true, traits: &[] },
        ],

        // ==================== LEC ====================
        "FNC" => vec![
            PlayerConfig { game_id: "Orome", real_name: Some("马库斯·林德"), nationality: "DE", position: Position::Top, age: 23, ability: 64, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Razork", real_name: Some("伊万·马丁内斯"), nationality: "ES", position: Position::Jug, age: 23, ability: 65, potential: 68, is_starter: true, traits: &[TraitType::Consistent] },
            PlayerConfig { game_id: "Humanoid", real_name: Some("丹尼尔·马利克"), nationality: "CZ", position: Position::Mid, age: 24, ability: 68, potential: 69, is_starter: true, traits: &[TraitType::Clutch] },
            PlayerConfig { game_id: "Noxi", real_name: Some("卢卡斯·佩特罗夫"), nationality: "DE", position: Position::Adc, age: 21, ability: 64, potential: 68, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "JunFNC", real_name: Some("尤恩·桑切斯"), nationality: "ES", position: Position::Sup, age: 23, ability: 63, potential: 66, is_starter: true, traits: &[] },
        ],
        "TH" => vec![
            PlayerConfig { game_id: "Alvaro", real_name: Some("阿尔瓦罗·费尔南德斯"), nationality: "ES", position: Position::Top, age: 22, ability: 63, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Jankos", real_name: Some("乔纳斯·温尼克"), nationality: "PL", position: Position::Jug, age: 29, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran, TraitType::TeamLeader, TraitType::Clutch] },
            PlayerConfig { game_id: "Perkz", real_name: Some("克里斯蒂安·普尔科维奇"), nationality: "HR", position: Position::Mid, age: 26, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Clutch, TraitType::Veteran, TraitType::TeamLeader] },
            PlayerConfig { game_id: "Neo", real_name: Some("马泰奥·罗西"), nationality: "IT", position: Position::Adc, age: 21, ability: 64, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Mersa", real_name: Some("塞巴斯蒂安·雷耶斯"), nationality: "ES", position: Position::Sup, age: 23, ability: 62, potential: 65, is_starter: true, traits: &[] },
        ],
        "MAD" => vec![
            PlayerConfig { game_id: "Armut", real_name: Some("伊赫桑·耶尔马兹"), nationality: "TR", position: Position::Top, age: 25, ability: 66, potential: 67, is_starter: true, traits: &[TraitType::ComebackKing] },
            PlayerConfig { game_id: "Elyoya", real_name: Some("约翰·梅嫩德斯"), nationality: "ES", position: Position::Jug, age: 23, ability: 67, potential: 70, is_starter: true, traits: &[TraitType::RisingStar, TraitType::Clutch] },
            PlayerConfig { game_id: "Nisqy", real_name: Some("尼古拉斯·迪亚曼蒂诺"), nationality: "BE", position: Position::Mid, age: 26, ability: 65, potential: 65, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Carzzy", real_name: Some("塞尔吉奥·冈萨雷斯"), nationality: "ES", position: Position::Adc, age: 22, ability: 67, potential: 69, is_starter: true, traits: &[TraitType::Explosive, TraitType::RisingStar] },
            PlayerConfig { game_id: "Hylissang", real_name: Some("海利·兰塔宁"), nationality: "FI", position: Position::Sup, age: 28, ability: 66, potential: 66, is_starter: true, traits: &[TraitType::Volatile, TraitType::Veteran] },
        ],
        "G2" => vec![
            PlayerConfig { game_id: "BrokenBlade", real_name: Some("马丁·穆勒"), nationality: "DE", position: Position::Top, age: 24, ability: 67, potential: 68, is_starter: true, traits: &[TraitType::Consistent] },
            PlayerConfig { game_id: "Yike", real_name: Some("保罗·杜邦"), nationality: "FR", position: Position::Jug, age: 21, ability: 66, potential: 70, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Caps", real_name: Some("鲁卡斯·哈斯廷斯"), nationality: "DK", position: Position::Mid, age: 25, ability: 69, potential: 71, is_starter: true, traits: &[TraitType::Clutch, TraitType::Explosive, TraitType::TeamLeader] },
            PlayerConfig { game_id: "HansSama", real_name: Some("让·勒克莱尔"), nationality: "FR", position: Position::Adc, age: 25, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Consistent, TraitType::Veteran] },
            PlayerConfig { game_id: "Mikyx", real_name: Some("米哈尔·比洛夫斯基"), nationality: "SI", position: Position::Sup, age: 26, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran] },
        ],
        "FAL" => vec![
            PlayerConfig { game_id: "Odoamne", real_name: Some("安德烈·帕潘德里欧"), nationality: "RO", position: Position::Top, age: 28, ability: 65, potential: 65, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Malrang", real_name: Some("金贤宇"), nationality: "KR", position: Position::Jug, age: 25, ability: 67, potential: 68, is_starter: true, traits: &[TraitType::Clutch] },
            PlayerConfig { game_id: "Larssen", real_name: Some("西蒙·拉森"), nationality: "SE", position: Position::Mid, age: 24, ability: 64, potential: 67, is_starter: true, traits: &[TraitType::Consistent] },
            PlayerConfig { game_id: "Comp", real_name: Some("托马斯·维尔德"), nationality: "NL", position: Position::Adc, age: 22, ability: 64, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Trymbi", real_name: Some("马可·特里姆比"), nationality: "PL", position: Position::Sup, age: 23, ability: 62, potential: 65, is_starter: true, traits: &[] },
        ],
        "TW" => vec![
            PlayerConfig { game_id: "Finn", real_name: Some("芬恩·奥康奈尔"), nationality: "IE", position: Position::Top, age: 24, ability: 63, potential: 66, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Xerxe", real_name: Some("阿德里安·托多罗维奇"), nationality: "RO", position: Position::Jug, age: 26, ability: 64, potential: 64, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Nisquick", real_name: Some("尼古拉·彼得罗夫"), nationality: "BG", position: Position::Mid, age: 21, ability: 62, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Patrik", real_name: Some("帕特里克·约根森"), nationality: "DK", position: Position::Adc, age: 25, ability: 64, potential: 64, is_starter: true, traits: &[TraitType::Consistent] },
            PlayerConfig { game_id: "Kaiser", real_name: Some("扬·科瓦奇"), nationality: "DE", position: Position::Sup, age: 24, ability: 62, potential: 64, is_starter: true, traits: &[] },
        ],
        "AMB" => vec![
            PlayerConfig { game_id: "Bwipo", real_name: Some("弗洛里安·里乌"), nationality: "BE", position: Position::Top, age: 26, ability: 66, potential: 66, is_starter: true, traits: &[TraitType::Volatile, TraitType::Veteran] },
            PlayerConfig { game_id: "Selfmade", real_name: Some("马尔钦·沃伊塔谢克"), nationality: "PL", position: Position::Jug, age: 25, ability: 65, potential: 66, is_starter: true, traits: &[TraitType::Explosive] },
            PlayerConfig { game_id: "Jensen", real_name: Some("尼克拉斯·延森"), nationality: "DK", position: Position::Mid, age: 28, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran, TraitType::Consistent] },
            PlayerConfig { game_id: "Upset", real_name: Some("埃利亚斯·拉斯特"), nationality: "DE", position: Position::Adc, age: 24, ability: 68, potential: 69, is_starter: true, traits: &[TraitType::Consistent, TraitType::Clutch] },
            PlayerConfig { game_id: "Targamas", real_name: Some("雷米·拉丰"), nationality: "FR", position: Position::Sup, age: 25, ability: 64, potential: 65, is_starter: true, traits: &[] },
        ],
        "MSF" => vec![
            PlayerConfig { game_id: "Szy", real_name: Some("西里尔·勒布朗"), nationality: "FR", position: Position::Top, age: 21, ability: 64, potential: 68, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Zanzarah", real_name: Some("扎恩·阿尔法罗"), nationality: "ES", position: Position::Jug, age: 22, ability: 63, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Vetheo", real_name: Some("雷米·贝特朗"), nationality: "FR", position: Position::Mid, age: 21, ability: 65, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Crownie", real_name: Some("克劳恩·埃里克森"), nationality: "DK", position: Position::Adc, age: 22, ability: 62, potential: 66, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "IgNar", real_name: Some("金景焕"), nationality: "KR", position: Position::Sup, age: 27, ability: 64, potential: 64, is_starter: true, traits: &[TraitType::Veteran] },
        ],
        "WLF" => vec![
            PlayerConfig { game_id: "Adam", real_name: Some("阿达姆·博纳"), nationality: "FR", position: Position::Top, age: 21, ability: 63, potential: 67, is_starter: true, traits: &[TraitType::Explosive, TraitType::RisingStar] },
            PlayerConfig { game_id: "Kacper", real_name: Some("卡斯佩尔·诺瓦克"), nationality: "PL", position: Position::Jug, age: 22, ability: 64, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Nukeduck", real_name: Some("马丁·霍尔特"), nationality: "NO", position: Position::Mid, age: 28, ability: 66, potential: 66, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Rekkles", real_name: Some("马丁·拉尔森"), nationality: "SE", position: Position::Adc, age: 28, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran, TraitType::Consistent] },
            PlayerConfig { game_id: "Sakasa", real_name: Some("萨沙·伊万诺夫"), nationality: "BG", position: Position::Sup, age: 23, ability: 62, potential: 65, is_starter: true, traits: &[] },
        ],
        "NKE" => vec![
            PlayerConfig { game_id: "Wunder", real_name: Some("马丁·汉森"), nationality: "DK", position: Position::Top, age: 26, ability: 66, potential: 66, is_starter: true, traits: &[TraitType::Veteran, TraitType::Clutch] },
            PlayerConfig { game_id: "Raz", real_name: Some("拉兹·科恩"), nationality: "IL", position: Position::Jug, age: 22, ability: 64, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Zen", real_name: Some("泽恩·莫雷蒂"), nationality: "IT", position: Position::Mid, age: 21, ability: 64, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Kobbe", real_name: Some("科布·克里斯滕森"), nationality: "DK", position: Position::Adc, age: 27, ability: 65, potential: 65, is_starter: true, traits: &[TraitType::Veteran, TraitType::Consistent] },
            PlayerConfig { game_id: "Vander", real_name: Some("维克多·埃尔南德斯"), nationality: "ES", position: Position::Sup, age: 26, ability: 63, potential: 63, is_starter: true, traits: &[TraitType::Veteran] },
        ],
        "AST" => vec![
            PlayerConfig { game_id: "Auro", real_name: Some("奥罗·贝克"), nationality: "DE", position: Position::Top, age: 22, ability: 62, potential: 66, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Blanc", real_name: Some("白·勒克莱尔"), nationality: "FR", position: Position::Jug, age: 22, ability: 63, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Magi", real_name: Some("马吉·索伦森"), nationality: "DK", position: Position::Mid, age: 21, ability: 64, potential: 68, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Galio", real_name: Some("加里奥·费尔南德斯"), nationality: "ES", position: Position::Adc, age: 24, ability: 64, potential: 64, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Destiny", real_name: Some("德斯蒂尼·约翰逊"), nationality: "UK", position: Position::Sup, age: 23, ability: 62, potential: 64, is_starter: true, traits: &[] },
        ],
        "VIT" => vec![
            PlayerConfig { game_id: "Photon", real_name: Some("菲利普·杜兰"), nationality: "FR", position: Position::Top, age: 21, ability: 64, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Bo", real_name: Some("博·马丁内斯"), nationality: "ES", position: Position::Jug, age: 22, ability: 65, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "NeoVIT", real_name: Some("尼奥·杜邦"), nationality: "FR", position: Position::Mid, age: 21, ability: 66, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Flakk", real_name: Some("弗拉克·米勒"), nationality: "DE", position: Position::Adc, age: 22, ability: 64, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Jactroll", real_name: Some("雅各布·佩德森"), nationality: "DK", position: Position::Sup, age: 24, ability: 62, potential: 64, is_starter: true, traits: &[] },
        ],
        "XL" => vec![
            PlayerConfig { game_id: "Kungs", real_name: Some("康斯坦丁·贝克"), nationality: "LV", position: Position::Top, age: 22, ability: 64, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Rho", real_name: Some("罗·范德梅尔"), nationality: "NL", position: Position::Jug, age: 21, ability: 64, potential: 68, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Diplex", real_name: Some("迪普莱克斯·莫兰"), nationality: "ES", position: Position::Mid, age: 21, ability: 65, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Vizicsacsi", real_name: Some("维齐·拉斯洛"), nationality: "HU", position: Position::Adc, age: 27, ability: 66, potential: 66, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Twerly", real_name: Some("特威利·约翰森"), nationality: "DK", position: Position::Sup, age: 23, ability: 62, potential: 64, is_starter: true, traits: &[] },
        ],
        "SK" => vec![
            PlayerConfig { game_id: "Chen", real_name: Some("陈·施密特"), nationality: "DE", position: Position::Top, age: 22, ability: 63, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "JankosJr", real_name: Some("雅尼克·温尼克"), nationality: "PL", position: Position::Jug, age: 21, ability: 62, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Kold", real_name: Some("科尔德·安德森"), nationality: "DK", position: Position::Mid, age: 23, ability: 64, potential: 66, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Zwyroo", real_name: Some("兹维罗·彼得森"), nationality: "SE", position: Position::Adc, age: 22, ability: 62, potential: 65, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Hades", real_name: Some("黑德斯·穆勒"), nationality: "DE", position: Position::Sup, age: 23, ability: 61, potential: 64, is_starter: true, traits: &[] },
        ],

        // ==================== LCS ====================
        "FQ" => vec![
            PlayerConfig { game_id: "Frost", real_name: Some("弗罗斯特·李"), nationality: "US", position: Position::Top, age: 24, ability: 64, potential: 66, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Quake", real_name: Some("奎克·马丁"), nationality: "US", position: Position::Jug, age: 24, ability: 64, potential: 65, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Nova", real_name: Some("诺瓦·陈"), nationality: "US", position: Position::Mid, age: 24, ability: 66, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Aurora", real_name: Some("奥罗拉·金"), nationality: "US", position: Position::Adc, age: 24, ability: 65, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Glace", real_name: Some("格雷斯·怀特"), nationality: "US", position: Position::Sup, age: 24, ability: 62, potential: 63, is_starter: true, traits: &[] },
        ],
        "100T" => vec![
            PlayerConfig { game_id: "Ssumday", real_name: Some("金成勋"), nationality: "KR", position: Position::Top, age: 29, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran, TraitType::Consistent] },
            PlayerConfig { game_id: "Closer", real_name: Some("李柱延"), nationality: "KR", position: Position::Jug, age: 24, ability: 66, potential: 68, is_starter: true, traits: &[TraitType::Explosive] },
            PlayerConfig { game_id: "Bjergsen", real_name: Some("比约格·斯特纳森"), nationality: "DK", position: Position::Mid, age: 28, ability: 68, potential: 68, is_starter: true, traits: &[TraitType::Veteran, TraitType::TeamLeader, TraitType::Consistent] },
            PlayerConfig { game_id: "Doublelift", real_name: Some("彼得·卢"), nationality: "US", position: Position::Adc, age: 31, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran, TraitType::Clutch, TraitType::TeamLeader] },
            PlayerConfig { game_id: "Busio", real_name: Some("布西奥·罗德里格斯"), nationality: "US", position: Position::Sup, age: 22, ability: 64, potential: 68, is_starter: true, traits: &[TraitType::RisingStar] },
        ],
        "C9" => vec![
            PlayerConfig { game_id: "Fudge", real_name: Some("费奇·菲利普斯"), nationality: "AU", position: Position::Top, age: 22, ability: 65, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Blaber", real_name: Some("丹尼斯·沃尔什"), nationality: "US", position: Position::Jug, age: 25, ability: 67, potential: 69, is_starter: true, traits: &[TraitType::Explosive, TraitType::Clutch] },
            PlayerConfig { game_id: "EMENES", real_name: Some("伊曼纽尔·洛佩兹"), nationality: "MX", position: Position::Mid, age: 21, ability: 67, potential: 70, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Berserker", real_name: Some("金敏宇"), nationality: "KR", position: Position::Adc, age: 20, ability: 68, potential: 72, is_starter: true, traits: &[TraitType::RisingStar, TraitType::Explosive] },
            PlayerConfig { game_id: "Zven", real_name: Some("杰斯珀·斯文森"), nationality: "DK", position: Position::Sup, age: 27, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran, TraitType::Consistent] },
        ],
        "TL" => vec![
            PlayerConfig { game_id: "Summit", real_name: Some("尹灿宇"), nationality: "KR", position: Position::Top, age: 24, ability: 66, potential: 68, is_starter: true, traits: &[TraitType::Explosive] },
            PlayerConfig { game_id: "Santorin", real_name: Some("卢卡斯·陶林"), nationality: "DK", position: Position::Jug, age: 28, ability: 65, potential: 65, is_starter: true, traits: &[TraitType::Veteran, TraitType::Consistent] },
            PlayerConfig { game_id: "Haeri", real_name: Some("海瑞·帕克"), nationality: "KR", position: Position::Mid, age: 21, ability: 64, potential: 69, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Yeon", real_name: Some("延尚赫"), nationality: "KR", position: Position::Adc, age: 20, ability: 67, potential: 71, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "CoreJJ", real_name: Some("乔伊·赵"), nationality: "KR", position: Position::Sup, age: 29, ability: 68, potential: 68, is_starter: true, traits: &[TraitType::Veteran, TraitType::TeamLeader, TraitType::Consistent] },
        ],
        "NRG" => vec![
            PlayerConfig { game_id: "Kimchee", real_name: Some("金池·朴"), nationality: "KR", position: Position::Top, age: 22, ability: 64, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Spica", real_name: Some("马修·霍普金斯"), nationality: "CN", position: Position::Jug, age: 24, ability: 66, potential: 68, is_starter: true, traits: &[TraitType::Clutch] },
            PlayerConfig { game_id: "Soligo", real_name: Some("索利戈·雷耶斯"), nationality: "US", position: Position::Mid, age: 24, ability: 64, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Tactical", real_name: Some("塔克·刘易斯"), nationality: "US", position: Position::Adc, age: 24, ability: 65, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Huhi", real_name: Some("胡希·李"), nationality: "KR", position: Position::Sup, age: 29, ability: 64, potential: 64, is_starter: true, traits: &[TraitType::Veteran] },
        ],
        "DIG" => vec![
            PlayerConfig { game_id: "NeoDIG", real_name: Some("尼奥·威廉姆斯"), nationality: "US", position: Position::Top, age: 21, ability: 63, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Kenvi", real_name: Some("肯·维加"), nationality: "US", position: Position::Jug, age: 20, ability: 64, potential: 68, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "FakeGod", real_name: Some("法克·戈德斯坦"), nationality: "US", position: Position::Mid, age: 23, ability: 62, potential: 64, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Stixxay", real_name: Some("特里斯坦·谢弗"), nationality: "US", position: Position::Adc, age: 28, ability: 64, potential: 64, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Aphromoo", real_name: Some("阿夫罗·穆罕默德"), nationality: "US", position: Position::Sup, age: 30, ability: 65, potential: 65, is_starter: true, traits: &[TraitType::Veteran, TraitType::TeamLeader] },
        ],
        "EG" => vec![
            PlayerConfig { game_id: "Jojopyun", real_name: Some("乔乔·朴"), nationality: "CA", position: Position::Top, age: 19, ability: 66, potential: 71, is_starter: true, traits: &[TraitType::RisingStar, TraitType::Clutch] },
            PlayerConfig { game_id: "Inspired", real_name: Some("安德烈亚斯·霍恩"), nationality: "PL", position: Position::Jug, age: 24, ability: 67, potential: 69, is_starter: true, traits: &[TraitType::Consistent, TraitType::Clutch] },
            PlayerConfig { game_id: "Danny", real_name: Some("丹尼·金"), nationality: "US", position: Position::Mid, age: 20, ability: 67, potential: 72, is_starter: true, traits: &[TraitType::RisingStar, TraitType::Explosive] },
            PlayerConfig { game_id: "Bwiper", real_name: Some("布赖恩·威尔逊"), nationality: "US", position: Position::Adc, age: 22, ability: 64, potential: 68, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Vulcan", real_name: Some("斯坦利·黄"), nationality: "CA", position: Position::Sup, age: 25, ability: 67, potential: 68, is_starter: true, traits: &[TraitType::Consistent] },
        ],
        "SR" => vec![
            PlayerConfig { game_id: "Lourlo", real_name: Some("劳尔·马丁内斯"), nationality: "US", position: Position::Top, age: 24, ability: 62, potential: 64, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Palafox", real_name: Some("帕拉福克斯·冈萨雷斯"), nationality: "US", position: Position::Jug, age: 24, ability: 63, potential: 64, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Josedeodo", real_name: Some("何塞·多德"), nationality: "AR", position: Position::Mid, age: 24, ability: 64, potential: 65, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Arrow", real_name: Some("阿罗·金"), nationality: "KR", position: Position::Adc, age: 24, ability: 66, potential: 67, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Poome", real_name: Some("普姆·苏克查"), nationality: "TH", position: Position::Sup, age: 24, ability: 62, potential: 62, is_starter: true, traits: &[] },
        ],
        "TSM" => vec![
            PlayerConfig { game_id: "Huni", real_name: Some("海尼·海沃德"), nationality: "KR", position: Position::Top, age: 27, ability: 65, potential: 65, is_starter: true, traits: &[TraitType::Volatile, TraitType::Veteran] },
            PlayerConfig { game_id: "Mithy", real_name: Some("米蒂·罗德里格斯"), nationality: "ES", position: Position::Jug, age: 29, ability: 64, potential: 64, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "JensenTSM", real_name: Some("尼克·詹森"), nationality: "DK", position: Position::Mid, age: 29, ability: 67, potential: 67, is_starter: true, traits: &[TraitType::Veteran, TraitType::Consistent] },
            PlayerConfig { game_id: "Biofrost", real_name: Some("比奥·弗罗斯特"), nationality: "CA", position: Position::Adc, age: 27, ability: 64, potential: 64, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Smoothie", real_name: Some("斯穆西·王"), nationality: "CA", position: Position::Sup, age: 28, ability: 66, potential: 66, is_starter: true, traits: &[TraitType::Veteran] },
        ],
        "EUB" => vec![
            PlayerConfig { game_id: "BearTop", real_name: Some("贝尔·托普"), nationality: "IS", position: Position::Top, age: 24, ability: 63, potential: 64, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "IceClaw", real_name: Some("艾斯·克拉夫"), nationality: "IS", position: Position::Jug, age: 24, ability: 62, potential: 65, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "FrostByte", real_name: Some("弗罗斯特拜特·李"), nationality: "IS", position: Position::Mid, age: 24, ability: 64, potential: 66, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "AuroraX", real_name: Some("奥罗拉X·金"), nationality: "IS", position: Position::Adc, age: 24, ability: 64, potential: 64, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Nord", real_name: Some("诺德·埃里克森"), nationality: "IS", position: Position::Sup, age: 24, ability: 62, potential: 62, is_starter: true, traits: &[] },
        ],
        "SASY" => vec![
            PlayerConfig { game_id: "Syke", real_name: Some("赛克·罗德里格斯"), nationality: "US", position: Position::Top, age: 24, ability: 62, potential: 64, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Riptide", real_name: Some("莱普泰德·琼斯"), nationality: "US", position: Position::Jug, age: 24, ability: 63, potential: 65, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Neon", real_name: Some("尼昂·陈"), nationality: "US", position: Position::Mid, age: 24, ability: 64, potential: 66, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Blaze", real_name: Some("布雷兹·马丁"), nationality: "US", position: Position::Adc, age: 24, ability: 62, potential: 63, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Echo", real_name: Some("埃科·怀特"), nationality: "US", position: Position::Sup, age: 24, ability: 61, potential: 62, is_starter: true, traits: &[] },
        ],
        "IMT" => vec![
            PlayerConfig { game_id: "Revenge", real_name: Some("里文奇·金"), nationality: "US", position: Position::Top, age: 22, ability: 63, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Xmithie", real_name: Some("金·史密斯"), nationality: "PH", position: Position::Jug, age: 31, ability: 64, potential: 64, is_starter: true, traits: &[TraitType::Veteran, TraitType::Consistent] },
            PlayerConfig { game_id: "PowerOfEvil", real_name: Some("鲍威尔·陈"), nationality: "DE", position: Position::Mid, age: 28, ability: 66, potential: 66, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Stix", real_name: Some("斯蒂克斯·李"), nationality: "US", position: Position::Adc, age: 21, ability: 64, potential: 67, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Hakuho", real_name: Some("白鹏·田中"), nationality: "JP", position: Position::Sup, age: 26, ability: 62, potential: 62, is_starter: true, traits: &[TraitType::Veteran] },
        ],
        "CLG" => vec![
            PlayerConfig { game_id: "Dame", real_name: Some("戴姆·约翰逊"), nationality: "US", position: Position::Top, age: 21, ability: 62, potential: 66, is_starter: true, traits: &[TraitType::RisingStar] },
            PlayerConfig { game_id: "Wiggily", real_name: Some("威格利·帕克"), nationality: "US", position: Position::Jug, age: 26, ability: 63, potential: 63, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Pobelter", real_name: Some("波贝尔特·金"), nationality: "US", position: Position::Mid, age: 28, ability: 65, potential: 65, is_starter: true, traits: &[TraitType::Veteran] },
            PlayerConfig { game_id: "Piglet", real_name: Some("皮杰特·朴"), nationality: "KR", position: Position::Adc, age: 30, ability: 64, potential: 64, is_starter: true, traits: &[TraitType::Veteran, TraitType::Clutch] },
            PlayerConfig { game_id: "ZionSpartan", real_name: Some("锡安·李"), nationality: "US", position: Position::Sup, age: 29, ability: 64, potential: 64, is_starter: true, traits: &[TraitType::Veteran] },
        ],
        "LG" => vec![
            PlayerConfig { game_id: "Zero", real_name: Some("零·张"), nationality: "US", position: Position::Top, age: 24, ability: 62, potential: 63, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "One", real_name: Some("一·王"), nationality: "US", position: Position::Jug, age: 24, ability: 62, potential: 64, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Two", real_name: Some("二·李"), nationality: "US", position: Position::Mid, age: 24, ability: 63, potential: 64, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Three", real_name: Some("三·赵"), nationality: "US", position: Position::Adc, age: 24, ability: 61, potential: 62, is_starter: true, traits: &[] },
            PlayerConfig { game_id: "Four", real_name: Some("四·周"), nationality: "US", position: Position::Sup, age: 24, ability: 60, potential: 62, is_starter: true, traits: &[] },
        ],

        // 默认情况 - 生成默认选手
        _ => generate_default_players(team_short_name, "CN"),
    }
}

/// 为没有详细数据的队伍生成默认选手
fn generate_default_players(team_short_name: &str, nationality: &'static str) -> Vec<PlayerConfig> {
    let positions = [Position::Top, Position::Jug, Position::Mid, Position::Adc, Position::Sup];
    let mut players = Vec::new();

    for (i, pos) in positions.iter().enumerate() {
        // 首发
        players.push(PlayerConfig {
            game_id: Box::leak(format!("{}_{}", team_short_name, i + 1).into_boxed_str()),
            real_name: None,
            nationality,
            position: *pos,
            age: 24,
            ability: 60,
            potential: 63,
            is_starter: true,
            traits: &[],
        });
    }

    players
}
