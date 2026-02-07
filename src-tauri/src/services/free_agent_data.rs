/// 自由选手初始数据（共110人）

pub struct FreeAgentConfig {
    pub game_id: &'static str,
    pub real_name: &'static str,
    pub position: &'static str, // "Top"/"Jug"/"Mid"/"Adc"/"Sup"
    pub ability: u8,
    pub potential: u8,
    pub age: u8,
}

/// 获取指定赛区的自由选手数据
pub fn get_free_agents(region_id: u64) -> Vec<FreeAgentConfig> {
    match region_id {
        1 => get_lpl_free_agents(),
        2 => get_lck_free_agents(),
        3 => get_lec_free_agents(),
        4 => get_lcs_free_agents(),
        _ => get_lpl_free_agents(),
    }
}

/// LPL 赛区自由选手（28人）
fn get_lpl_free_agents() -> Vec<FreeAgentConfig> {
    vec![
        FreeAgentConfig { game_id: "Longge", real_name: "龙戈", position: "Top", ability: 55, potential: 60, age: 24 },
        FreeAgentConfig { game_id: "Fenrir", real_name: "冯瑞", position: "Top", ability: 44, potential: 62, age: 17 },
        FreeAgentConfig { game_id: "Hammer", real_name: "韩默", position: "Top", ability: 53, potential: 58, age: 22 },
        FreeAgentConfig { game_id: "Vast", real_name: "王浩远", position: "Top", ability: 58, potential: 61, age: 21 },
        FreeAgentConfig { game_id: "Peak", real_name: "裴科", position: "Top", ability: 48, potential: 58, age: 20 },
        FreeAgentConfig { game_id: "Ember", real_name: "陈焱", position: "Top", ability: 39, potential: 60, age: 17 },
        FreeAgentConfig { game_id: "Lynx", real_name: "李云轩", position: "Jug", ability: 56, potential: 60, age: 22 },
        FreeAgentConfig { game_id: "Torque", real_name: "陶科", position: "Jug", ability: 47, potential: 61, age: 19 },
        FreeAgentConfig { game_id: "Prowl", real_name: "潘泽", position: "Jug", ability: 60, potential: 62, age: 23 },
        FreeAgentConfig { game_id: "Cinder", real_name: "辛德", position: "Jug", ability: 41, potential: 59, age: 18 },
        FreeAgentConfig { game_id: "Creek", real_name: "曲科", position: "Jug", ability: 51, potential: 58, age: 23 },
        FreeAgentConfig { game_id: "Sage", real_name: "赛谷", position: "Mid", ability: 61, potential: 63, age: 22 },
        FreeAgentConfig { game_id: "Cipher", real_name: "赵奇峰", position: "Mid", ability: 54, potential: 62, age: 20 },
        FreeAgentConfig { game_id: "Myth", real_name: "穆思", position: "Mid", ability: 46, potential: 60, age: 19 },
        FreeAgentConfig { game_id: "Comet", real_name: "柯铭", position: "Mid", ability: 50, potential: 59, age: 21 },
        FreeAgentConfig { game_id: "Drift", real_name: "杜立夫", position: "Mid", ability: 58, potential: 61, age: 23 },
        FreeAgentConfig { game_id: "Nebula", real_name: "聂步", position: "Mid", ability: 36, potential: 64, age: 17 },
        FreeAgentConfig { game_id: "Spark", real_name: "孙鹏", position: "Adc", ability: 59, potential: 62, age: 22 },
        FreeAgentConfig { game_id: "Bolt", real_name: "柏腾", position: "Adc", ability: 49, potential: 60, age: 20 },
        FreeAgentConfig { game_id: "Cannon", real_name: "曹楠", position: "Adc", ability: 44, potential: 58, age: 19 },
        FreeAgentConfig { game_id: "Flame", real_name: "方亮", position: "Adc", ability: 62, potential: 64, age: 24 },
        FreeAgentConfig { game_id: "Lotus", real_name: "罗特", position: "Adc", ability: 34, potential: 61, age: 17 },
        FreeAgentConfig { game_id: "Guard", real_name: "顾安", position: "Sup", ability: 58, potential: 60, age: 23 },
        FreeAgentConfig { game_id: "Haven", real_name: "韩文", position: "Sup", ability: 48, potential: 59, age: 21 },
        FreeAgentConfig { game_id: "Pillar", real_name: "蒲磊", position: "Sup", ability: 55, potential: 60, age: 22 },
        FreeAgentConfig { game_id: "Quill", real_name: "曲霖", position: "Sup", ability: 43, potential: 58, age: 19 },
        FreeAgentConfig { game_id: "Dew", real_name: "杜威", position: "Sup", ability: 53, potential: 58, age: 25 },
        FreeAgentConfig { game_id: "Dawn", real_name: "段恩", position: "Sup", ability: 36, potential: 62, age: 17 },
    ]
}

/// LCK 赛区自由选手（28人）
fn get_lck_free_agents() -> Vec<FreeAgentConfig> {
    vec![
        FreeAgentConfig { game_id: "Mist", real_name: "金宇成", position: "Top", ability: 55, potential: 59, age: 23 },
        FreeAgentConfig { game_id: "Blade", real_name: "朴敏赫", position: "Top", ability: 45, potential: 62, age: 18 },
        FreeAgentConfig { game_id: "Crimson", real_name: "崔赤焕", position: "Top", ability: 57, potential: 60, age: 22 },
        FreeAgentConfig { game_id: "Tempest", real_name: "金泰铉", position: "Top", ability: 61, potential: 63, age: 22 },
        FreeAgentConfig { game_id: "Iron", real_name: "李哲旭", position: "Top", ability: 47, potential: 58, age: 20 },
        FreeAgentConfig { game_id: "Grit", real_name: "郑志坚", position: "Top", ability: 40, potential: 60, age: 17 },
        FreeAgentConfig { game_id: "Dusk", real_name: "朴黄焕", position: "Jug", ability: 54, potential: 60, age: 22 },
        FreeAgentConfig { game_id: "Ridge", real_name: "李栋焕", position: "Jug", ability: 58, potential: 61, age: 23 },
        FreeAgentConfig { game_id: "Fang", real_name: "方成宇", position: "Jug", ability: 48, potential: 60, age: 19 },
        FreeAgentConfig { game_id: "Wisp", real_name: "吴智焕", position: "Jug", ability: 51, potential: 59, age: 24 },
        FreeAgentConfig { game_id: "Tremor", real_name: "金道贤", position: "Jug", ability: 42, potential: 62, age: 18 },
        FreeAgentConfig { game_id: "Apex", real_name: "安平信", position: "Mid", ability: 60, potential: 62, age: 22 },
        FreeAgentConfig { game_id: "Glint", real_name: "金光宇", position: "Mid", ability: 46, potential: 61, age: 19 },
        FreeAgentConfig { game_id: "Prong", real_name: "朴宗赫", position: "Mid", ability: 55, potential: 60, age: 21 },
        FreeAgentConfig { game_id: "Azure", real_name: "安志然", position: "Mid", ability: 62, potential: 64, age: 23 },
        FreeAgentConfig { game_id: "Flux", real_name: "丁民洙", position: "Mid", ability: 49, potential: 59, age: 22 },
        FreeAgentConfig { game_id: "Surge", real_name: "徐载赫", position: "Mid", ability: 37, potential: 64, age: 17 },
        FreeAgentConfig { game_id: "Hawk", real_name: "韩旭京", position: "Adc", ability: 58, potential: 61, age: 22 },
        FreeAgentConfig { game_id: "Dart", real_name: "都贤宇", position: "Adc", ability: 53, potential: 60, age: 21 },
        FreeAgentConfig { game_id: "Edge", real_name: "李承贤", position: "Adc", ability: 44, potential: 59, age: 19 },
        FreeAgentConfig { game_id: "Flint", real_name: "金贤石", position: "Adc", ability: 39, potential: 62, age: 18 },
        FreeAgentConfig { game_id: "Rift", real_name: "李敏宇", position: "Adc", ability: 56, potential: 60, age: 24 },
        FreeAgentConfig { game_id: "Halo", real_name: "韩石周", position: "Sup", ability: 57, potential: 60, age: 23 },
        FreeAgentConfig { game_id: "Anchor", real_name: "安进宇", position: "Sup", ability: 48, potential: 59, age: 21 },
        FreeAgentConfig { game_id: "Cache", real_name: "车英浩", position: "Sup", ability: 52, potential: 60, age: 22 },
        FreeAgentConfig { game_id: "Pact", real_name: "朴民石", position: "Sup", ability: 61, potential: 62, age: 24 },
        FreeAgentConfig { game_id: "Vow", real_name: "禹成旻", position: "Sup", ability: 44, potential: 58, age: 20 },
        FreeAgentConfig { game_id: "Aeon", real_name: "李俊瑞", position: "Sup", ability: 35, potential: 63, age: 17 },
    ]
}

/// LEC 赛区自由选手（27人）
fn get_lec_free_agents() -> Vec<FreeAgentConfig> {
    vec![
        FreeAgentConfig { game_id: "Titan", real_name: "提坦·舒尔茨", position: "Top", ability: 56, potential: 60, age: 22 },
        FreeAgentConfig { game_id: "Slate", real_name: "斯莱特·维尔纳", position: "Top", ability: 45, potential: 61, age: 18 },
        FreeAgentConfig { game_id: "Cobalt", real_name: "科巴特·约翰森", position: "Top", ability: 58, potential: 62, age: 23 },
        FreeAgentConfig { game_id: "Rubble", real_name: "鲁布尔·马丁", position: "Top", ability: 50, potential: 58, age: 24 },
        FreeAgentConfig { game_id: "Mason", real_name: "梅森·佩蒂", position: "Top", ability: 40, potential: 60, age: 18 },
        FreeAgentConfig { game_id: "Raven", real_name: "雷文·科勒", position: "Jug", ability: 55, potential: 60, age: 22 },
        FreeAgentConfig { game_id: "Briar", real_name: "布莱尔·杜邦", position: "Jug", ability: 61, potential: 63, age: 23 },
        FreeAgentConfig { game_id: "Scorch", real_name: "斯科奇·汉森", position: "Jug", ability: 48, potential: 60, age: 20 },
        FreeAgentConfig { game_id: "Gale", real_name: "盖尔·奥尔森", position: "Jug", ability: 53, potential: 61, age: 24 },
        FreeAgentConfig { game_id: "Pike", real_name: "派克·穆勒", position: "Jug", ability: 44, potential: 59, age: 19 },
        FreeAgentConfig { game_id: "Nexus", real_name: "内克斯·里维拉", position: "Jug", ability: 38, potential: 62, age: 17 },
        FreeAgentConfig { game_id: "Scepter", real_name: "塞普特·安德森", position: "Mid", ability: 59, potential: 62, age: 22 },
        FreeAgentConfig { game_id: "Herald", real_name: "赫拉德·皮尔斯", position: "Mid", ability: 50, potential: 60, age: 21 },
        FreeAgentConfig { game_id: "Locus", real_name: "洛库斯·布朗", position: "Mid", ability: 62, potential: 64, age: 23 },
        FreeAgentConfig { game_id: "Prion", real_name: "普里昂·韦伯", position: "Mid", ability: 46, potential: 61, age: 19 },
        FreeAgentConfig { game_id: "Crest", real_name: "克雷斯特·罗斯", position: "Mid", ability: 54, potential: 59, age: 25 },
        FreeAgentConfig { game_id: "Lance", real_name: "兰斯·施密特", position: "Adc", ability: 57, potential: 60, age: 22 },
        FreeAgentConfig { game_id: "Quasar", real_name: "夸萨·费舍尔", position: "Adc", ability: 61, potential: 62, age: 23 },
        FreeAgentConfig { game_id: "Trace", real_name: "特雷斯·沃尔夫", position: "Adc", ability: 47, potential: 60, age: 20 },
        FreeAgentConfig { game_id: "Orbit", real_name: "奥比特·哈里斯", position: "Adc", ability: 52, potential: 59, age: 24 },
        FreeAgentConfig { game_id: "Pixel", real_name: "皮克塞·迪亚兹", position: "Adc", ability: 42, potential: 62, age: 18 },
        FreeAgentConfig { game_id: "Styx", real_name: "斯蒂克斯·瑞恩", position: "Adc", ability: 36, potential: 58, age: 17 },
        FreeAgentConfig { game_id: "Bastion", real_name: "巴斯蒂安·霍夫曼", position: "Sup", ability: 56, potential: 60, age: 22 },
        FreeAgentConfig { game_id: "Ward", real_name: "沃德·佩特森", position: "Sup", ability: 48, potential: 59, age: 21 },
        FreeAgentConfig { game_id: "Aegis", real_name: "艾吉斯·拉尔森", position: "Sup", ability: 58, potential: 61, age: 23 },
        FreeAgentConfig { game_id: "Latch", real_name: "拉奇·布伦纳", position: "Sup", ability: 44, potential: 58, age: 20 },
        FreeAgentConfig { game_id: "Nimbus", real_name: "宁布斯·马丁内斯", position: "Sup", ability: 39, potential: 62, age: 17 },
    ]
}

/// LCS 赛区自由选手（27人）
fn get_lcs_free_agents() -> Vec<FreeAgentConfig> {
    vec![
        FreeAgentConfig { game_id: "Granite", real_name: "格兰特·史密斯", position: "Top", ability: 57, potential: 60, age: 23 },
        FreeAgentConfig { game_id: "Tusk", real_name: "塔斯克·约翰逊", position: "Top", ability: 47, potential: 61, age: 19 },
        FreeAgentConfig { game_id: "Monolith", real_name: "莫诺利斯·陈", position: "Top", ability: 62, potential: 63, age: 24 },
        FreeAgentConfig { game_id: "Veil", real_name: "韦尔·帕克", position: "Top", ability: 44, potential: 60, age: 18 },
        FreeAgentConfig { game_id: "Rust", real_name: "拉斯特·戴维斯", position: "Top", ability: 52, potential: 58, age: 25 },
        FreeAgentConfig { game_id: "Vapor", real_name: "瓦波·金", position: "Jug", ability: 56, potential: 60, age: 22 },
        FreeAgentConfig { game_id: "Siege", real_name: "西格·李", position: "Jug", ability: 60, potential: 62, age: 23 },
        FreeAgentConfig { game_id: "Flare", real_name: "弗莱尔·布朗", position: "Jug", ability: 46, potential: 60, age: 19 },
        FreeAgentConfig { game_id: "Mortar", real_name: "莫塔·威尔逊", position: "Jug", ability: 50, potential: 59, age: 24 },
        FreeAgentConfig { game_id: "Rapid", real_name: "莱皮德·马丁", position: "Jug", ability: 41, potential: 61, age: 18 },
        FreeAgentConfig { game_id: "Kindle", real_name: "金德尔·安德森", position: "Jug", ability: 36, potential: 63, age: 17 },
        FreeAgentConfig { game_id: "Vector", real_name: "维克特·赵", position: "Mid", ability: 58, potential: 61, age: 22 },
        FreeAgentConfig { game_id: "Vertex", real_name: "弗泰克斯·刘", position: "Mid", ability: 62, potential: 64, age: 23 },
        FreeAgentConfig { game_id: "Fractal", real_name: "弗拉克塔·金", position: "Mid", ability: 48, potential: 60, age: 20 },
        FreeAgentConfig { game_id: "Signal", real_name: "西格纳·王", position: "Mid", ability: 53, potential: 60, age: 24 },
        FreeAgentConfig { game_id: "Relic", real_name: "雷利克·陈", position: "Mid", ability: 41, potential: 62, age: 18 },
        FreeAgentConfig { game_id: "Volley", real_name: "沃利·约翰逊", position: "Adc", ability: 58, potential: 61, age: 22 },
        FreeAgentConfig { game_id: "Strafe", real_name: "斯特雷夫·李", position: "Adc", ability: 49, potential: 60, age: 21 },
        FreeAgentConfig { game_id: "Caliber", real_name: "卡利伯·布朗", position: "Adc", ability: 62, potential: 64, age: 23 },
        FreeAgentConfig { game_id: "Ricochet", real_name: "里科·马丁", position: "Adc", ability: 45, potential: 59, age: 19 },
        FreeAgentConfig { game_id: "Brim", real_name: "布里姆·怀特", position: "Adc", ability: 54, potential: 60, age: 25 },
        FreeAgentConfig { game_id: "Bloom", real_name: "布卢姆·帕克", position: "Adc", ability: 39, potential: 61, age: 18 },
        FreeAgentConfig { game_id: "Vault", real_name: "沃特·金", position: "Sup", ability: 57, potential: 60, age: 23 },
        FreeAgentConfig { game_id: "Rally", real_name: "拉利·李", position: "Sup", ability: 48, potential: 59, age: 21 },
        FreeAgentConfig { game_id: "Merit", real_name: "梅利特·安德森", position: "Sup", ability: 62, potential: 62, age: 24 },
        FreeAgentConfig { game_id: "Pylon", real_name: "帕伦·戴维斯", position: "Sup", ability: 45, potential: 60, age: 19 },
        FreeAgentConfig { game_id: "Atlas", real_name: "阿特拉斯·黄", position: "Sup", ability: 36, potential: 61, age: 17 },
    ]
}
