use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;

/// 生成的新秀信息
#[derive(Debug, Clone)]
pub struct GeneratedRookie {
    pub game_id: String,
    pub real_name: String,
    pub position: String,
    pub ability: u8,
    pub potential: u8,
    pub stability: u8,
    pub age: u8,
    pub tag: String,
    pub nationality: String,
}

/// 新秀生成器 —— 按赛区生成拟真 Game ID 和姓名的新秀
pub struct RookieGenerator {
    rng: StdRng,
}

impl RookieGenerator {
    /// 使用指定种子创建
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// 使用系统熵创建
    pub fn from_entropy() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }

    /// 为指定赛区生成 count 个新秀，保证 game_id 不与 existing_ids 重复
    pub fn generate_rookies(
        &mut self,
        region_id: u64,
        count: usize,
        existing_ids: &HashSet<String>,
    ) -> Vec<GeneratedRookie> {
        let mut results = Vec::with_capacity(count);
        let mut used_ids: HashSet<String> = existing_ids.clone();

        let game_id_pool = get_game_id_pool(region_id);
        let positions = ["Top", "Jungle", "Mid", "Bot", "Support"];

        for i in 0..count {
            // 选取 game_id（唯一性保证）
            let game_id = self.pick_unique_game_id(&game_id_pool, &mut used_ids);

            // 生成真实姓名
            let real_name = self.generate_real_name(region_id);

            // 属性生成
            let tag = self.roll_tag();
            let age = self.roll_age();
            let ability = self.roll_ability(&tag);
            let potential = self.roll_potential(ability, &tag);
            let stability = self.roll_stability(age);

            // 位置均匀分配
            let position = positions[i % positions.len()].to_string();

            let nationality = get_nationality(region_id).to_string();

            results.push(GeneratedRookie {
                game_id,
                real_name,
                position,
                ability,
                potential,
                stability,
                age,
                tag,
                nationality,
            });
        }

        results
    }

    // ==================== 内部方法 ====================

    /// 从池中挑选唯一 game_id，若全部占用则追加数字后缀
    fn pick_unique_game_id(
        &mut self,
        pool: &[&str],
        used: &mut HashSet<String>,
    ) -> String {
        // 先收集未使用的 ID
        let available: Vec<&&str> = pool.iter().filter(|id| !used.contains(**id)).collect();

        if !available.is_empty() {
            let idx = self.rng.gen_range(0..available.len());
            let id = available[idx].to_string();
            used.insert(id.clone());
            return id;
        }

        // 所有池内 ID 都被占用，追加数字后缀
        loop {
            let base = pool[self.rng.gen_range(0..pool.len())];
            let suffix = self.rng.gen_range(2..100);
            let candidate = format!("{}{}", base, suffix);
            if !used.contains(&candidate) {
                used.insert(candidate.clone());
                return candidate;
            }
        }
    }

    /// 按赛区生成拟真真实姓名
    fn generate_real_name(&mut self, region_id: u64) -> String {
        match region_id {
            2 => {
                // LCK: 韩式 "{姓}{名}"
                let (surnames, given_names) = get_lck_name_pool();
                let s = surnames[self.rng.gen_range(0..surnames.len())];
                let g = given_names[self.rng.gen_range(0..given_names.len())];
                format!("{}{}", s, g)
            }
            3 => {
                // LEC: 欧式 "{名}·{姓}"
                let (first_names, last_names) = get_lec_name_pool();
                let f = first_names[self.rng.gen_range(0..first_names.len())];
                let l = last_names[self.rng.gen_range(0..last_names.len())];
                format!("{}·{}", f, l)
            }
            4 => {
                // LCS: 美式 "{名}·{姓}"
                let (first_names, last_names) = get_lcs_name_pool();
                let f = first_names[self.rng.gen_range(0..first_names.len())];
                let l = last_names[self.rng.gen_range(0..last_names.len())];
                format!("{}·{}", f, l)
            }
            _ => {
                // LPL (1) 及其他: 中式 "{名}·{姓}"
                let (first_names, last_names) = get_lpl_name_pool();
                let f = first_names[self.rng.gen_range(0..first_names.len())];
                let l = last_names[self.rng.gen_range(0..last_names.len())];
                format!("{}·{}", f, l)
            }
        }
    }

    /// Tag 分布: Genius 20%, Normal 60%, Ordinary 20%
    fn roll_tag(&mut self) -> String {
        let roll: f64 = self.rng.gen();
        if roll < 0.20 {
            "Genius".to_string()
        } else if roll < 0.80 {
            "Normal".to_string()
        } else {
            "Ordinary".to_string()
        }
    }

    /// Age 分布: 16(10%), 17(22%), 18(30%), 19(25%), 20(13%)
    fn roll_age(&mut self) -> u8 {
        let roll: f64 = self.rng.gen();
        if roll < 0.10 {
            16
        } else if roll < 0.32 {
            17
        } else if roll < 0.62 {
            18
        } else if roll < 0.87 {
            19
        } else {
            20
        }
    }

    /// Ability: Genius 64-67, Normal 61-64, Ordinary 59-61
    fn roll_ability(&mut self, tag: &str) -> u8 {
        match tag {
            "Genius" => self.rng.gen_range(64..=67),
            "Normal" => self.rng.gen_range(61..=64),
            _ => self.rng.gen_range(59..=61), // Ordinary
        }
    }

    /// Potential: ability + (Genius 3~5, Normal 2~4, Ordinary 2~3)
    fn roll_potential(&mut self, ability: u8, tag: &str) -> u8 {
        let bonus = match tag {
            "Genius" => self.rng.gen_range(3..=5),
            "Normal" => self.rng.gen_range(2..=4),
            _ => self.rng.gen_range(2..=3), // Ordinary
        };
        ability + bonus
    }

    /// Stability: 16→49-50, 17→50-51, 18→51-53, 19→53-54, 20→54-56
    fn roll_stability(&mut self, age: u8) -> u8 {
        match age {
            16 => self.rng.gen_range(49..=50),
            17 => self.rng.gen_range(50..=51),
            18 => self.rng.gen_range(51..=53),
            19 => self.rng.gen_range(53..=54),
            20 => self.rng.gen_range(54..=56),
            _ => self.rng.gen_range(49..=56),
        }
    }
}

// ==================== 赛区国籍 ====================

fn get_nationality(region_id: u64) -> &'static str {
    match region_id {
        1 => "中国",
        2 => "韩国",
        3 => "欧洲",
        4 => "北美",
        _ => "中国",
    }
}

// ==================== Game ID 池 (每赛区 100 个) ====================

fn get_game_id_pool(region_id: u64) -> Vec<&'static str> {
    match region_id {
        2 => get_lck_game_ids(),
        3 => get_lec_game_ids(),
        4 => get_lcs_game_ids(),
        _ => get_lpl_game_ids(), // 1 = LPL
    }
}

/// LCK — 韩式极简 3-5 字母
fn get_lck_game_ids() -> Vec<&'static str> {
    vec![
        "Kaze", "Onyx", "Dusk", "Void", "Apex", "Nox", "Zen", "Ark", "Ryx", "Sol",
        "Hex", "Ion", "Aero", "Crux", "Zeta", "Cyan", "Aeon", "Oryx", "Flux", "Byte",
        "Nero", "Jax", "Vyn", "Kayn", "Yoru", "Dex", "Elm", "Grim", "Koda", "Mars",
        "Navi", "Odin", "Prax", "Riku", "Shin", "Tobi", "Wren", "Xero", "Yuki", "Zael",
        "Aven", "Bane", "Ciel", "Dion", "Ezra", "Fael", "Gale", "Hawk", "Izon", "Jinx",
        "Kael", "Lynx", "Miru", "Nova", "Omen", "Pike", "Quil", "Rift", "Sage", "Tyr",
        "Ulti", "Vex", "Wynn", "Xion", "Yato", "Zion", "Arik", "Blaz", "Cruz", "Dyre",
        "Echo", "Finn", "Ghor", "Haze", "Igni", "Juze", "Kira", "Lux", "Myst", "Nyx",
        "Obex", "Pyre", "Raze", "Sora", "Tael", "Uran", "Veil", "Wrex", "Xael", "Ymir",
        "Zeno", "Axle", "Bolt", "Coda", "Dray", "Enix", "Foss", "Gyro", "Hexa", "Izar",
    ]
}

/// LEC — 欧洲名/神话风 4-6 字母
fn get_lec_game_ids() -> Vec<&'static str> {
    vec![
        "Dante", "Elio", "Floki", "Gaius", "Hector", "Igor", "Jasper", "Karel", "Leif", "Marco",
        "Nolan", "Olaf", "Pelle", "Rafael", "Stefan", "Tomas", "Adrian", "Boris", "Cyril", "Dorian",
        "Edgar", "Fabian", "Gustav", "Henrik", "Johan", "Klaus", "Lorenz", "Milan", "Niklas", "Oskar",
        "Pascal", "Romain", "Soren", "Trond", "Urban", "Viktor", "Walden", "Xander", "Yngvar", "Zoran",
        "Albin", "Bjorn", "Casper", "Dieter", "Edvin", "Frode", "Gunnar", "Halvar", "Ivar", "Joris",
        "Konrad", "Ludwig", "Magnus", "Norbert", "Orsino", "Pierre", "Rainer", "Sander", "Theron", "Ulric",
        "Varek", "Werner", "Xenos", "Yannick", "Zorion", "Aldric", "Baldur", "Cedric", "Darius", "Emeric",
        "Fenris", "Geralt", "Hadric", "Ingmar", "Jareth", "Kellan", "Lander", "Morris", "Nestor", "Obelix",
        "Patrik", "Quincy", "Rupert", "Sigurd", "Tancred", "Ulrich", "Volker", "Wendel", "Xerxes", "Yorick",
        "Zephyr", "Anselm", "Benoit", "Clarus", "Dagmar", "Erland", "Floris", "Gideon", "Helios", "Isidor",
    ]
}

/// LCS — 美式潮流 4-5 字母
fn get_lcs_game_ids() -> Vec<&'static str> {
    vec![
        "Brix", "Colt", "Duke", "Edge", "Fury", "Grif", "Holt", "Indy", "Jace", "Kade",
        "Lark", "Mace", "Nile", "Oaks", "Pace", "Raid", "Slade", "Tank", "Vick", "Wynn",
        "York", "Zeke", "Asher", "Blake", "Cliff", "Drew", "Ford", "Grant", "Hayes", "Jude",
        "Knox", "Lane", "Miles", "Nash", "Orion", "Pike", "Quinn", "Reed", "Stone", "Trace",
        "Wade", "Axel", "Brock", "Chase", "Dane", "Flint", "Grit", "Hawk", "Ivan", "Jet",
        "Kane", "Lobo", "Mavic", "Noel", "Owen", "Puma", "Rock", "Scout", "Thor", "Uno",
        "Vance", "Wolf", "Xray", "Yogi", "Zeus", "Arrow", "Blaze", "Crew", "Drift", "Ember",
        "Frost", "Gale", "Haze", "Iron", "Jade", "Karma", "Lynx", "Mako", "Nova", "Onyx",
        "Pulse", "Rogue", "Surge", "Titan", "Ultra", "Volt", "Warp", "Xeno", "Yolo", "Zinc",
        "Ace", "Bolt", "Code", "Dusk", "Enzo", "Flex", "Gust", "Hex", "Ibex", "Jolt",
    ]
}

/// LPL — 短英文混搭风
fn get_lpl_game_ids() -> Vec<&'static str> {
    vec![
        "Arch", "Davi", "Eddy", "Fang", "Glen", "Ives", "Jago", "Kino", "Loki", "Mako",
        "Noel", "Puma", "Qing", "Rex", "Sven", "Tao", "Vega", "Xing", "Yang", "Alto",
        "Cozy", "Dawn", "Grit", "Halo", "Iris", "Jade", "Keen", "Muse", "Nord", "Onyx",
        "Peak", "Rein", "Sage", "Tide", "Umber", "Vale", "Wing", "Yato", "Zinc", "Bliss",
        "Crow", "Daze", "Echo", "Flux", "Glow", "Hive", "Ivy", "Jazz", "Kite", "Loom",
        "Mesa", "Nite", "Opal", "Plum", "Rune", "Silk", "Tusk", "Ursa", "Vine", "Welt",
        "Xeno", "Yew", "Zeal", "Aqua", "Brim", "Clay", "Dune", "Etch", "Flam", "Gaze",
        "Helm", "Icon", "Jolt", "Knot", "Lynx", "Morn", "Nest", "Oath", "Pond", "Rave",
        "Soot", "Trek", "Unit", "Vow", "Wasp", "Yoke", "Zone", "Aura", "Bolt", "Core",
        "Dash", "Elms", "Fern", "Gulf", "Hymn", "Ikon", "Jinn", "Kelp", "Lace", "Myth",
    ]
}

// ==================== 真实姓名池 ====================

/// LCK: 姓(15) × 名(30) = 450 组合
fn get_lck_name_pool() -> (&'static [&'static str], &'static [&'static str]) {
    static SURNAMES: &[&str] = &[
        "金", "李", "朴", "崔", "郑", "安", "姜", "申", "林", "徐",
        "黄", "闵", "赵", "权", "宋",
    ];
    static GIVEN_NAMES: &[&str] = &[
        "宇烈", "承焕", "敏锡", "志勋", "建佑", "宰贤", "道贤", "仁浩", "珉宇", "贤宇",
        "成浩", "成民", "晙赫", "泰勋", "晙奭", "正宇", "俊熙", "在宇", "承民", "尚赫",
        "民奎", "泰贤", "秀焕", "东辉", "炫宇", "圣勋", "瑞民", "柱焕", "相赫", "星旿",
    ];
    (SURNAMES, GIVEN_NAMES)
}

/// LEC: 名(25) × 姓(20) = 500 组合
fn get_lec_name_pool() -> (&'static [&'static str], &'static [&'static str]) {
    static FIRST_NAMES: &[&str] = &[
        "阿克塞尔", "列奥", "米洛", "西奥", "埃里克", "卢卡", "诺亚", "雨果", "菲利克斯",
        "伊莱亚斯", "马克斯", "奥斯卡", "利亚姆", "雷米", "朱尔斯", "尼尔斯", "安东",
        "布鲁诺", "迭戈", "恩佐", "马泰奥", "维克托", "拉斯", "于戈", "扬",
    ];
    static LAST_NAMES: &[&str] = &[
        "杜邦", "费尔南德斯", "施密特", "约翰森", "罗西", "诺瓦克", "莫雷蒂", "佩特罗夫",
        "拉丰", "贝克", "伊万诺夫", "霍尔特", "勒克莱尔", "桑切斯", "穆勒", "杜兰",
        "马丁", "贝特朗", "科瓦奇", "拉森",
    ];
    (FIRST_NAMES, LAST_NAMES)
}

/// LCS: 名(20) × 姓(15) = 300 组合
fn get_lcs_name_pool() -> (&'static [&'static str], &'static [&'static str]) {
    static FIRST_NAMES: &[&str] = &[
        "艾斯", "布雷兹", "切斯", "德雷克", "伊莱", "芬恩", "盖奇", "汉克", "伊恩", "杰",
        "凯", "利奥", "马克斯", "诺亚", "欧文", "帕克斯", "塞斯", "特洛伊", "万斯", "韦德",
    ];
    static LAST_NAMES: &[&str] = &[
        "金", "马丁", "威廉姆斯", "李", "罗德里格斯", "帕克", "史密斯", "约翰逊", "陈",
        "怀特", "马丁内斯", "刘", "王", "戴维斯", "布朗",
    ];
    (FIRST_NAMES, LAST_NAMES)
}

/// LPL: 名(20) × 姓(15) = 300 组合
fn get_lpl_name_pool() -> (&'static [&'static str], &'static [&'static str]) {
    static FIRST_NAMES: &[&str] = &[
        "安森", "迪恩", "伊凡", "杰克", "利奥", "保罗", "威尔", "彦", "本", "亨利",
        "尼克", "加文", "山姆", "汤姆", "雷", "凯", "维克", "赛恩", "波", "蒂姆",
    ];
    static LAST_NAMES: &[&str] = &[
        "林", "陈", "王", "李", "张", "黄", "周", "吴", "郑", "徐",
        "杨", "郭", "何", "谢", "马",
    ];
    (FIRST_NAMES, LAST_NAMES)
}
