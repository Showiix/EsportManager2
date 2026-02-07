//! 选秀选手池初始数据
//! 包含四大赛区各50名新秀选手，共200人
//! 数据来源：初始化四个赛区选秀选手池.md

/// 选秀选手配置
pub struct DraftPlayerConfig {
    pub game_id: &'static str,
    pub real_name: &'static str,
    pub position: &'static str, // "Top" | "Jungle" | "Mid" | "Bot" | "Support"
    pub ability: u8,
    pub potential: u8,
    pub stability: u8,
    pub age: u8,
    pub tag: &'static str, // "Genius" | "Normal" | "Ordinary"
}

/// 获取赛区选秀选手池
/// region_id: 1=LPL, 2=LCK, 3=LEC, 4=LCS
pub fn get_draft_pool(region_id: u64) -> Vec<DraftPlayerConfig> {
    match region_id {
        // LCK (韩国赛区)
        2 => get_lck_draft_pool(),
        // LEC (欧洲赛区)
        3 => get_lec_draft_pool(),
        // LCS (北美赛区)
        4 => get_lcs_draft_pool(),
        // LPL (中国赛区) - 使用 PCS 数据作为 LPL 选秀池
        1 | _ => get_lpl_draft_pool(),
    }
}

/// LCK 新人池（50人）- 韩式极简·冷峻科技感
/// 上单10 / 打野10 / 中单12 / ADC9 / 辅助9
fn get_lck_draft_pool() -> Vec<DraftPlayerConfig> {
    vec![
        DraftPlayerConfig { game_id: "Auro", real_name: "金宇烈", position: "Top", ability: 64, potential: 67, stability: 53, age: 18, tag: "Genius" },
        DraftPlayerConfig { game_id: "Vex", real_name: "李承焕", position: "Jungle", ability: 63, potential: 66, stability: 53, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Rye", real_name: "崔敏锡", position: "Mid", ability: 65, potential: 68, stability: 54, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Kev", real_name: "朴志勋", position: "Jungle", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Zyen", real_name: "郑建佑", position: "Mid", ability: 64, potential: 67, stability: 54, age: 17, tag: "Genius" },
        DraftPlayerConfig { game_id: "Doph", real_name: "金宰贤", position: "Mid", ability: 62, potential: 65, stability: 51, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "Snow", real_name: "安道贤", position: "Top", ability: 61, potential: 64, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Flay", real_name: "姜仁浩", position: "Top", ability: 60, potential: 63, stability: 50, age: 17, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Echo", real_name: "徐珉宇", position: "Mid", ability: 66, potential: 69, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Nyon", real_name: "林贤宇", position: "Support", ability: 63, potential: 65, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Cloz", real_name: "金建辉", position: "Jungle", ability: 62, potential: 67, stability: 52, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Mira", real_name: "闵艺琳", position: "Bot", ability: 64, potential: 66, stability: 53, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "Zeru", real_name: "赵成民", position: "Top", ability: 60, potential: 62, stability: 49, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Haze", real_name: "黄晙赫", position: "Mid", ability: 64, potential: 67, stability: 54, age: 18, tag: "Genius" },
        DraftPlayerConfig { game_id: "Luxe", real_name: "申艺灿", position: "Mid", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Bolt", real_name: "金泰勋", position: "Jungle", ability: 61, potential: 63, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Nova", real_name: "李晙奭", position: "Mid", ability: 65, potential: 68, stability: 54, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Rain", real_name: "朴雨范", position: "Support", ability: 60, potential: 62, stability: 50, age: 17, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Frost", real_name: "崔寒星", position: "Top", ability: 63, potential: 65, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Acez", real_name: "金胜浩", position: "Bot", ability: 64, potential: 67, stability: 53, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Zenk", real_name: "朴正宇", position: "Top", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Skyu", real_name: "李天空", position: "Bot", ability: 59, potential: 61, stability: 49, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Blad", real_name: "金刃宇", position: "Top", ability: 64, potential: 66, stability: 54, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "Puls", real_name: "安泰勋", position: "Jungle", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Ghst", real_name: "申幽然", position: "Support", ability: 61, potential: 62, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Rune", real_name: "朴卢恩", position: "Mid", ability: 65, potential: 67, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Myth", real_name: "金神话", position: "Mid", ability: 63, potential: 65, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Edgx", real_name: "李锋宇", position: "Top", ability: 60, potential: 63, stability: 50, age: 17, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Core", real_name: "崔核心", position: "Support", ability: 64, potential: 67, stability: 53, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Vale", real_name: "金谷贤", position: "Jungle", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Drft", real_name: "朴漂宇", position: "Jungle", ability: 60, potential: 62, stability: 49, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Quil", real_name: "申羽笔", position: "Bot", ability: 64, potential: 68, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Tide", real_name: "李潮宇", position: "Jungle", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Shde", real_name: "金影锡", position: "Support", ability: 61, potential: 63, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Orbt", real_name: "崔轨道", position: "Mid", ability: 66, potential: 69, stability: 54, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Lynx", real_name: "朴林克", position: "Jungle", ability: 63, potential: 65, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Mist", real_name: "安雾贤", position: "Support", ability: 60, potential: 62, stability: 50, age: 17, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Peak", real_name: "金峰宇", position: "Top", ability: 64, potential: 67, stability: 53, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Flow", real_name: "李流锡", position: "Jungle", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Dust", real_name: "朴尘宇", position: "Bot", ability: 59, potential: 61, stability: 49, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Spar", real_name: "申火花", position: "Bot", ability: 65, potential: 67, stability: 54, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Gale", real_name: "金风宇", position: "Jungle", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Embr", real_name: "崔余烬", position: "Support", ability: 61, potential: 63, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Prsm", real_name: "朴棱镜", position: "Bot", ability: 64, potential: 69, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Veil", real_name: "李面纱", position: "Support", ability: 63, potential: 65, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Thorn", real_name: "金荆宇", position: "Top", ability: 60, potential: 62, stability: 50, age: 17, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Rift", real_name: "安裂宇", position: "Top", ability: 64, potential: 67, stability: 53, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Wisp", real_name: "朴微光", position: "Support", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Ecko", real_name: "申回声", position: "Bot", ability: 60, potential: 62, stability: 49, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Kyro", real_name: "金凯罗", position: "Mid", ability: 64, potential: 67, stability: 54, age: 18, tag: "Genius" },
    ]
}

/// LEC 新人池（50人）- 欧洲多元·神话/自然风
/// 上单10 / 打野10 / 中单10 / ADC10 / 辅助10
fn get_lec_draft_pool() -> Vec<DraftPlayerConfig> {
    vec![
        DraftPlayerConfig { game_id: "Axel", real_name: "阿克塞尔·杜邦", position: "Top", ability: 64, potential: 67, stability: 54, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Fenr", real_name: "列奥·费尔南德斯", position: "Jungle", ability: 63, potential: 65, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Milo", real_name: "米洛·施密特", position: "Support", ability: 62, potential: 64, stability: 52, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Kairo", real_name: "凯·约翰森", position: "Mid", ability: 65, potential: 68, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Theo", real_name: "西奥·罗西", position: "Bot", ability: 62, potential: 64, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Erik", real_name: "埃里克·诺瓦克", position: "Jungle", ability: 61, potential: 63, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Luca", real_name: "卢卡·莫雷蒂", position: "Mid", ability: 66, potential: 69, stability: 54, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Nyx", real_name: "诺亚·佩特罗夫", position: "Support", ability: 64, potential: 66, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Hugo", real_name: "雨果·拉丰", position: "Top", ability: 60, potential: 62, stability: 50, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Felix", real_name: "菲利克斯·贝克", position: "Bot", ability: 64, potential: 67, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Arlo", real_name: "阿洛·伊万诺夫", position: "Jungle", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Elias", real_name: "伊莱亚斯·霍尔特", position: "Mid", ability: 63, potential: 65, stability: 53, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "Maxx", real_name: "马克斯·勒克莱尔", position: "Top", ability: 65, potential: 68, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Oscar", real_name: "奥斯卡·桑切斯", position: "Bot", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Liam", real_name: "利亚姆·穆勒", position: "Support", ability: 61, potential: 63, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Remy", real_name: "雷米·杜兰", position: "Mid", ability: 67, potential: 69, stability: 55, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Jules", real_name: "朱尔斯·马丁", position: "Jungle", ability: 64, potential: 66, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Finn", real_name: "芬恩·奥康奈尔", position: "Top", ability: 60, potential: 62, stability: 50, age: 17, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Theon", real_name: "西昂·贝特朗", position: "Mid", ability: 64, potential: 67, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Nils", real_name: "尼尔斯·科瓦奇", position: "Jungle", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Emil", real_name: "埃米尔·拉森", position: "Support", ability: 63, potential: 65, stability: 53, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "Viktor", real_name: "维克托·索伦森", position: "Bot", ability: 66, potential: 69, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Anton", real_name: "安东·雷耶斯", position: "Top", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Bruno", real_name: "布鲁诺·加西亚", position: "Jungle", ability: 61, potential: 63, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Casp", real_name: "卡斯佩尔·诺瓦克", position: "Support", ability: 65, potential: 67, stability: 54, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Diego", real_name: "迭戈·罗西", position: "Bot", ability: 64, potential: 66, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Enzo", real_name: "恩佐·杜邦", position: "Top", ability: 60, potential: 62, stability: 50, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Gabe", real_name: "加布·彼得森", position: "Mid", ability: 64, potential: 67, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Huxley", real_name: "雨果·施密特", position: "Jungle", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Ivo", real_name: "伊沃·科瓦奇", position: "Support", ability: 63, potential: 65, stability: 53, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "Jayce", real_name: "杰·威廉姆斯", position: "Bot", ability: 67, potential: 69, stability: 55, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Kian", real_name: "基安·穆罕默德", position: "Top", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Lars", real_name: "拉斯·霍尔特", position: "Jungle", ability: 61, potential: 63, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Matteo", real_name: "马泰奥·罗西", position: "Mid", ability: 66, potential: 68, stability: 54, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Nico", real_name: "尼科·贝克", position: "Support", ability: 64, potential: 66, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Otto", real_name: "奥托·施密特", position: "Top", ability: 60, potential: 62, stability: 50, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Pablo", real_name: "巴勃罗·费尔南德斯", position: "Bot", ability: 65, potential: 67, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Quinn", real_name: "昆·李", position: "Jungle", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Rico", real_name: "里科·加西亚", position: "Support", ability: 63, potential: 65, stability: 53, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "Sami", real_name: "萨米·伊万诺夫", position: "Mid", ability: 64, potential: 67, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Tomm", real_name: "汤姆·约翰森", position: "Top", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Ugo", real_name: "于戈·杜兰", position: "Jungle", ability: 61, potential: 63, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Valen", real_name: "瓦伦·莫雷蒂", position: "Bot", ability: 67, potential: 69, stability: 56, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Wesk", real_name: "韦斯·彼得森", position: "Support", ability: 64, potential: 66, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Xavi", real_name: "哈维·罗西", position: "Top", ability: 60, potential: 62, stability: 50, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Yann", real_name: "扬·勒布朗", position: "Mid", ability: 65, potential: 68, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Zane", real_name: "泽恩·科瓦奇", position: "Jungle", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Alaric", real_name: "阿拉里克·穆勒", position: "Support", ability: 63, potential: 65, stability: 53, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "Bjorn", real_name: "比约恩·霍尔特", position: "Bot", ability: 64, potential: 67, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Cael", real_name: "凯厄斯·贝克", position: "Top", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
    ]
}

/// LCS 新人池（50人）- 美式潮流·街头/游戏风
/// 上单10 / 打野10 / 中单10 / ADC10 / 辅助10
fn get_lcs_draft_pool() -> Vec<DraftPlayerConfig> {
    vec![
        DraftPlayerConfig { game_id: "Acey", real_name: "艾斯·金", position: "Bot", ability: 64, potential: 67, stability: 54, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Blaz", real_name: "布雷兹·马丁", position: "Top", ability: 63, potential: 65, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Chas", real_name: "切斯·威廉姆斯", position: "Jungle", ability: 62, potential: 64, stability: 52, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Drak", real_name: "德雷克·李", position: "Mid", ability: 65, potential: 68, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "EliX", real_name: "伊莱·罗德里格斯", position: "Support", ability: 62, potential: 64, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Fynn", real_name: "芬恩·帕克", position: "Top", ability: 61, potential: 63, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Gage", real_name: "盖奇·史密斯", position: "Jungle", ability: 66, potential: 69, stability: 54, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Hank", real_name: "汉克·约翰逊", position: "Support", ability: 64, potential: 66, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "IanZ", real_name: "伊恩·陈", position: "Bot", ability: 60, potential: 62, stability: 50, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Jayx", real_name: "杰·金", position: "Mid", ability: 64, potential: 67, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "KaiR", real_name: "凯·怀特", position: "Jungle", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Leov", real_name: "利奥·马丁内斯", position: "Top", ability: 63, potential: 65, stability: 53, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "Maxo", real_name: "马克斯·刘", position: "Bot", ability: 67, potential: 69, stability: 55, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Noak", real_name: "诺亚·王", position: "Support", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Owen", real_name: "欧文·戴维斯", position: "Mid", ability: 61, potential: 63, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Paxx", real_name: "帕克斯·罗德里格斯", position: "Jungle", ability: 65, potential: 67, stability: 54, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "QuinNA", real_name: "昆·李", position: "Support", ability: 64, potential: 66, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Rexx", real_name: "雷克斯·金", position: "Top", ability: 60, potential: 62, stability: 50, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Seth", real_name: "塞斯·帕克", position: "Mid", ability: 64, potential: 68, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Troy", real_name: "特洛伊·约翰逊", position: "Bot", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Uriel", real_name: "尤里·陈", position: "Support", ability: 63, potential: 65, stability: 53, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "Vanc", real_name: "万斯·史密斯", position: "Jungle", ability: 66, potential: 69, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Wade", real_name: "韦德·马丁", position: "Top", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Xand", real_name: "桑德·李", position: "Bot", ability: 61, potential: 63, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Yale", real_name: "耶尔·金", position: "Mid", ability: 67, potential: 69, stability: 56, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Zyan", real_name: "泽恩·威廉姆斯", position: "Support", ability: 64, potential: 66, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "AcezNA", real_name: "艾斯·罗德里格斯", position: "Bot", ability: 60, potential: 62, stability: 50, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "BoltNA", real_name: "博尔特·戴维斯", position: "Jungle", ability: 65, potential: 67, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Cruz", real_name: "克鲁兹·马丁内斯", position: "Top", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Dexx", real_name: "德克斯·帕克", position: "Mid", ability: 63, potential: 65, stability: 53, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "EchoNA", real_name: "埃科·陈", position: "Support", ability: 64, potential: 67, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Flint", real_name: "弗林特·史密斯", position: "Jungle", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Gray", real_name: "格雷·约翰逊", position: "Bot", ability: 61, potential: 63, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Hawk", real_name: "霍克·金", position: "Top", ability: 67, potential: 69, stability: 55, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Ikez", real_name: "艾克·李", position: "Support", ability: 64, potential: 66, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Jett", real_name: "杰特·威廉姆斯", position: "Bot", ability: 60, potential: 62, stability: 50, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Knox", real_name: "诺克斯·马丁", position: "Jungle", ability: 66, potential: 68, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Lane", real_name: "莱恩·罗德里格斯", position: "Mid", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Mile", real_name: "迈尔斯·帕克", position: "Support", ability: 63, potential: 65, stability: 53, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "Nash", real_name: "纳什·陈", position: "Bot", ability: 65, potential: 67, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Ori", real_name: "奥赖恩·史密斯", position: "Top", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Pike", real_name: "派克·约翰逊", position: "Jungle", ability: 61, potential: 63, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "QuilNA", real_name: "奎尔·金", position: "Mid", ability: 64, potential: 67, stability: 54, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Rhys", real_name: "莱斯·李", position: "Support", ability: 64, potential: 66, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Sage", real_name: "塞奇·威廉姆斯", position: "Bot", ability: 60, potential: 62, stability: 50, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Tate", real_name: "泰特·马丁内斯", position: "Top", ability: 67, potential: 69, stability: 55, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Ulyx", real_name: "尤利·罗德里格斯", position: "Jungle", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Veyne", real_name: "万斯·帕克", position: "Mid", ability: 63, potential: 65, stability: 53, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "Wolf", real_name: "沃尔夫·陈", position: "Support", ability: 66, potential: 69, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Kove", real_name: "科夫·李", position: "Mid", ability: 67, potential: 69, stability: 56, age: 19, tag: "Genius" },
    ]
}

/// LPL/PCS 新人池（50人）- 东亚/东南亚·拼音创意+自然词
/// 上单10 / 打野10 / 中单10 / ADC10 / 辅助10
fn get_lpl_draft_pool() -> Vec<DraftPlayerConfig> {
    vec![
        DraftPlayerConfig { game_id: "Anzo", real_name: "安森·林", position: "Top", ability: 64, potential: 67, stability: 53, age: 18, tag: "Genius" },
        DraftPlayerConfig { game_id: "Benx", real_name: "本·陈", position: "Jungle", ability: 63, potential: 65, stability: 53, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Carl", real_name: "卡尔·王", position: "Support", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Dean", real_name: "迪恩·李", position: "Mid", ability: 65, potential: 68, stability: 54, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Evan", real_name: "伊凡·张", position: "Bot", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "FelixCN", real_name: "菲力·黄", position: "Top", ability: 61, potential: 63, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Gavn", real_name: "加文·周", position: "Jungle", ability: 66, potential: 69, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Henr", real_name: "亨利·吴", position: "Support", ability: 64, potential: 66, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Ivan", real_name: "伊万·郑", position: "Bot", ability: 60, potential: 62, stability: 50, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Jack", real_name: "杰克·徐", position: "Mid", ability: 64, potential: 67, stability: 54, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Kenz", real_name: "肯·杨", position: "Jungle", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Leon", real_name: "利奥·郭", position: "Top", ability: 63, potential: 65, stability: 53, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "MarkCN", real_name: "马克·何", position: "Bot", ability: 67, potential: 69, stability: 55, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Nick", real_name: "尼克·谢", position: "Support", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "OwenCN", real_name: "欧文·林", position: "Mid", ability: 61, potential: 63, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Paul", real_name: "保罗·陈", position: "Jungle", ability: 65, potential: 67, stability: 54, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "QuinCN", real_name: "昆·王", position: "Support", ability: 64, potential: 66, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Rayn", real_name: "雷·李", position: "Bot", ability: 60, potential: 62, stability: 50, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Samx", real_name: "山姆·张", position: "Mid", ability: 64, potential: 68, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Tomz", real_name: "汤姆·黄", position: "Top", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Urix", real_name: "尤里·周", position: "Jungle", ability: 63, potential: 65, stability: 53, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "Vicx", real_name: "维克·吴", position: "Support", ability: 66, potential: 69, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Will", real_name: "威尔·郑", position: "Bot", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Xylo", real_name: "赛恩·徐", position: "Mid", ability: 61, potential: 63, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Yanz", real_name: "彦·杨", position: "Top", ability: 67, potential: 69, stability: 56, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Zack", real_name: "扎克·林", position: "Jungle", ability: 64, potential: 66, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Acex", real_name: "艾斯·陈", position: "Bot", ability: 60, potential: 62, stability: 50, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Boyz", real_name: "波·王", position: "Support", ability: 65, potential: 67, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Cyra", real_name: "赛·李", position: "Mid", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Dexy", real_name: "德克斯·张", position: "Top", ability: 63, potential: 65, stability: 53, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "Eliu", real_name: "伊莱·黄", position: "Jungle", ability: 64, potential: 67, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Finx", real_name: "芬恩·周", position: "Support", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Gusx", real_name: "格斯·吴", position: "Bot", ability: 61, potential: 63, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Hawx", real_name: "霍克·郑", position: "Mid", ability: 67, potential: 69, stability: 55, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Ianz", real_name: "伊恩·徐", position: "Top", ability: 64, potential: 66, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "JayxCN", real_name: "杰·杨", position: "Jungle", ability: 60, potential: 62, stability: 50, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Kaiz", real_name: "凯·林", position: "Support", ability: 66, potential: 68, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Lenx", real_name: "伦·陈", position: "Bot", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Maxi", real_name: "麦克斯·王", position: "Mid", ability: 63, potential: 65, stability: 53, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "Neox", real_name: "尼奥·李", position: "Jungle", ability: 65, potential: 67, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Olix", real_name: "奥利·张", position: "Support", ability: 62, potential: 64, stability: 51, age: 17, tag: "Normal" },
        DraftPlayerConfig { game_id: "Petz", real_name: "皮特·黄", position: "Bot", ability: 61, potential: 63, stability: 51, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Quix", real_name: "昆·周", position: "Mid", ability: 64, potential: 67, stability: 54, age: 19, tag: "Genius" },
        DraftPlayerConfig { game_id: "Royx", real_name: "罗伊·吴", position: "Top", ability: 64, potential: 66, stability: 53, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Sidz", real_name: "西德·郑", position: "Jungle", ability: 60, potential: 62, stability: 50, age: 16, tag: "Ordinary" },
        DraftPlayerConfig { game_id: "Timx", real_name: "蒂姆·徐", position: "Support", ability: 67, potential: 69, stability: 55, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Uzix", real_name: "乌兹·杨", position: "Bot", ability: 62, potential: 64, stability: 52, age: 18, tag: "Normal" },
        DraftPlayerConfig { game_id: "Vanx", real_name: "范·林", position: "Mid", ability: 63, potential: 65, stability: 53, age: 19, tag: "Normal" },
        DraftPlayerConfig { game_id: "Wesx", real_name: "韦斯·陈", position: "Jungle", ability: 66, potential: 69, stability: 54, age: 20, tag: "Genius" },
        DraftPlayerConfig { game_id: "Tenx", real_name: "天·张", position: "Top", ability: 67, potential: 69, stability: 56, age: 19, tag: "Genius" },
    ]
}

/// 获取赛区对应的国籍
pub fn get_region_nationality(region_id: u64) -> &'static str {
    match region_id {
        1 => "CN", // LPL
        2 => "KR", // LCK
        3 => "EU", // LEC
        4 => "NA", // LCS
        _ => "CN",
    }
}
