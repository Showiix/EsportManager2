 Ban/Pick 英雄选择系统

 背景

 当前比赛模拟只看"战力值"，缺少
 BP（Ban/Pick）维度。电竞的核心观赏性之一是英雄选择——选手英雄池深浅、针对性 Ban
 人、阵容搭配都会影响比赛结果。

 目标：为比赛模拟增加具体英雄制的 BP 系统，让 BP 结果产生 ±5-10% 的胜率影响。

 改动范围

 新增文件
 ┌────────────────────────────────────┬─────────────────────────────────────┐
 │                文件                │                说明                 │
 ├────────────────────────────────────┼─────────────────────────────────────┤
 │ src-tauri/src/engines/champion.rs  │ 英雄数据定义（50 个英雄，静态常量） │
 ├────────────────────────────────────┼─────────────────────────────────────┤
 │ src-tauri/src/engines/bp_engine.rs │ BP 模拟引擎（选禁算法、AI 决策）    │
 └────────────────────────────────────┴─────────────────────────────────────┘
 修改文件
 ┌───────────────────────────────────────────┬─────────────────────────────────────┐
 │                   文件                    │                改动                 │
 ├───────────────────────────────────────────┼─────────────────────────────────────┤
 │ src-tauri/src/engines/mod.rs              │ 注册新模块                          │
 ├───────────────────────────────────────────┼─────────────────────────────────────┤
 │ src-tauri/src/engines/meta_engine.rs      │ MetaType 增加 favored_archetypes()  │
 ├───────────────────────────────────────────┼─────────────────────────────────────┤
 │ src-tauri/src/engines/match_simulation.rs │ 新增 simulate_match_with_draft 方法 │
 ├───────────────────────────────────────────┼─────────────────────────────────────┤
 │ src-tauri/src/services/game_flow.rs       │ 快速模拟路径集成 BP                 │
 ├───────────────────────────────────────────┼─────────────────────────────────────┤
 │ src-tauri/src/commands/match_commands.rs  │ 详细模拟路径集成 BP                 │
 ├───────────────────────────────────────────┼─────────────────────────────────────┤
 │ src-tauri/src/services/init_service.rs    │ 新存档初始化英雄熟练度              │
 ├───────────────────────────────────────────┼─────────────────────────────────────┤
 │ src-tauri/src/db/connection.rs            │ 新增迁移（2 张表 + 2 列）           │
 └───────────────────────────────────────────┴─────────────────────────────────────┘
 ---
 第一部分：英雄数据层 (champion.rs)

 1.1 英雄设计

 50 个虚构英雄，每个位置 10 个，纯静态数据（不存数据库）。

 #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
 pub struct ChampionId(pub u8); // 1-50

 #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
 pub enum ChampionArchetype {
     Aggressive,   // 激进型：适合 EarlyGame 类 Meta
     Scaling,      // 后期型：适合 LateGame 类 Meta
     Utility,      // 功能型：适合 VisionControl 类 Meta
     Splitpush,    // 分推型：适合 SplitPush 类 Meta
     Teamfight,    // 团战型：适合 Teamfight 类 Meta
 }

 pub struct Champion {
     pub id: ChampionId,
     pub name: &'static str,       // 英文名
     pub cn_name: &'static str,    // 中文名
     pub position: &'static str,   // "TOP"/"JUG"/"MID"/"ADC"/"SUP"
     pub archetype: ChampionArchetype,
     pub base_power: f64,          // 基础强度 0.95-1.05（1.0 为中性）
     pub counter_ids: &'static [u8], // 克制的同位置英雄 ID
 }

 1.2 英雄分布示例
 ┌──────┬─────────┬─────────────────────────────────────────────────────┐
 │ 位置 │ ID 范围 │                      示例名称                       │
 ├──────┼─────────┼─────────────────────────────────────────────────────┤
 │ TOP  │ 1-10    │ 铁树 Ironbark、石壁 Stonewall、寒卫 Frostguard...   │
 ├──────┼─────────┼─────────────────────────────────────────────────────┤
 │ JUG  │ 11-20   │ 影牙 Shadowfang、刺蔓 Thornmaw、风爪 Stormclaw...   │
 ├──────┼─────────┼─────────────────────────────────────────────────────┤
 │ MID  │ 21-30   │ 奥术师 Arcanis、炎灵 Pyralis、织冰者 Frostweaver... │
 ├──────┼─────────┼─────────────────────────────────────────────────────┤
 │ ADC  │ 31-40   │ 迅矢 Swiftarrow、铁弹 Ironshot、寒箭 Frostbolt...   │
 ├──────┼─────────┼─────────────────────────────────────────────────────┤
 │ SUP  │ 41-50   │ 盾卫 Aegisward、潮守 Tidekeeper、月盾 Moonshield... │
 └──────┴─────────┴─────────────────────────────────────────────────────┘
 1.3 克制关系

 每个英雄克制同位置的 1-3 个英雄。被克制方的对位选手受到 -2 ability 惩罚（等效于克制方
 +2）。

 1.4 Meta 联动

 在 meta_engine.rs 的 MetaType 上增加方法：

 impl MetaType {
     pub fn favored_archetypes(&self) -> &'static [ChampionArchetype] {
         // 每种 MetaType 偏好 1-2 种英雄风格
         // 匹配当前 Meta 的英雄额外 +1 ability
     }
 }

 ---
 第二部分：选手英雄熟练度

 2.1 熟练度分级
 ┌───────────┬──────────┬────────────────────────┐
 │   等级    │ 能力修正 │          含义          │
 ├───────────┼──────────┼────────────────────────┤
 │ S（招牌） │ +2       │ 标志性英雄，选手代名词 │
 ├───────────┼──────────┼────────────────────────┤
 │ A（精通） │ +0       │ 舒适区，正常发挥       │
 ├───────────┼──────────┼────────────────────────┤
 │ B（能用） │ -3       │ 能打但不理想           │
 ├───────────┼──────────┼────────────────────────┤
 │ C（生疏） │ -6       │ 显著惩罚               │
 └───────────┴──────────┴────────────────────────┘
 2.2 英雄池大小（按选手能力）
 ┌────────────────┬──────┬──────┬──────┬──────────────┐
 │    选手能力    │ S 级 │ A 级 │ B 级 │ C 级（剩余） │
 ├────────────────┼──────┼──────┼──────┼──────────────┤
 │ 75-100（明星） │ 2-3  │ 3-4  │ 2-3  │ 剩余         │
 ├────────────────┼──────┼──────┼──────┼──────────────┤
 │ 60-74（主力）  │ 1-2  │ 2-3  │ 3-4  │ 剩余         │
 ├────────────────┼──────┼──────┼──────┼──────────────┤
 │ 45-59（平均）  │ 1    │ 2-3  │ 2-3  │ 剩余         │
 ├────────────────┼──────┼──────┼──────┼──────────────┤
 │ <45（板凳）    │ 1    │ 1-2  │ 2-3  │ 剩余         │
 └────────────────┴──────┴──────┴──────┴──────────────┘
 每个选手对自己位置的 10 个英雄都有熟练度等级。

 2.3 数据库表

 CREATE TABLE IF NOT EXISTS player_champion_mastery (
     id INTEGER PRIMARY KEY AUTOINCREMENT,
     save_id TEXT NOT NULL,
     player_id INTEGER NOT NULL,
     champion_id INTEGER NOT NULL,
     mastery_tier TEXT NOT NULL DEFAULT 'C',
     FOREIGN KEY (save_id) REFERENCES saves(id) ON DELETE CASCADE,
     UNIQUE(save_id, player_id, champion_id)
 );

 2.4 初始数据生成

 在 init_service.rs 新存档创建时，为 280 名选手生成熟练度：
 - 按能力等级决定 S/A/B 数量
 - 随机分配到位置内的 10 个英雄
 - 仅存 S/A/B 级记录（C 级是默认值，不存库，节省空间）
 - 约 1,680 行初始数据

 ---
 第三部分：BP 模拟引擎 (bp_engine.rs)

 3.1 选禁流程（标准 LOL 格式）

 每局比赛（BO3/BO5 中的每一局）独立进行 BP：

 Ban 阶段 1：蓝方 Ban → 红方 Ban → 蓝方 Ban → 红方 Ban → 蓝方 Ban → 红方 Ban
 Pick 阶段 1：蓝方 Pick 1 → 红方 Pick 2 → 蓝方 Pick 2 → 红方 Pick 1
 Ban 阶段 2：红方 Ban → 蓝方 Ban → 红方 Ban → 蓝方 Ban
 Pick 阶段 2：红方 Pick 1 → 蓝方 Pick 2 → 红方 Pick 1

 每局 10 Ban + 10 Pick，所有英雄不重复。蓝红方每局交替（Game 1 主场蓝方，Game 2
 客场蓝方…）。

 3.2 数据结构

 pub struct DraftResult {
     pub blue_team_id: u64,
     pub red_team_id: u64,
     pub bans: Vec<DraftAction>,           // 10 个 Ban
     pub picks: Vec<DraftAction>,          // 10 个 Pick
     pub blue_picks: [(ChampionId, &str); 5], // (英雄ID, 位置) x5
     pub red_picks: [(ChampionId, &str); 5],
 }

 pub struct DraftAction {
     pub team_id: u64,
     pub champion_id: ChampionId,
     pub action_type: DraftActionType, // Ban / Pick
     pub position: Option<String>,     // 仅 Pick 有
 }

 3.3 AI Ban 决策

 对每个可 Ban 英雄计算 ban_score，Ban 得分最高的：

 ban_score = 对手威胁分 + Meta 分 + 针对分

 对手威胁分：
   - 对手明星选手的 S 级英雄（ability >= 75）：+10
   - 对手普通选手的 S 级英雄：+6
   - 对手明星选手的 A 级英雄：+3

 Meta 分：
   - 英雄风格匹配当前 Meta：+3

 针对分：
   - 对手某选手 S 级英雄 <= 2 个，Ban 掉一个可逼其用 B/C 级：+5

 自然产生"针对 Ban"行为：英雄池浅的明星选手会被重点针对。

 3.4 AI Pick 决策

 对每个可选英雄计算 pick_score：

 pick_score = 熟练度分 + Meta 分 + 克制分 + 阵容分

 熟练度分：S=10, A=6, B=3, C=0
 Meta 分：英雄风格匹配当前 Meta → +3
 克制分：克制对手已选同位置英雄 → +4
 阵容分：队伍已有 2+ 同风格英雄 → -1（鼓励多样性）；缺少功能型 → 功能型英雄 +2

 3.5 性能保障

 - 英雄数据是静态常量，零 DB 查询
 - 熟练度数据在模拟开始前一次性加载到 HashMap<u64, HashMap<ChampionId, MasteryTier>>
 - 每次选禁约 1,000 次比较，一个赛季 500 场比赛 = ~50 万次比较，总耗时远低于 100ms
 - 快速模拟路径不存储 BP 详情（节省 DB），仅详细模拟路径存储

 ---
 第四部分：比赛模拟集成

 4.1 能力修正应用点

 BP 的结果在构建 MatchPlayerInfo 之后、进入战力计算之前，修正 ability：

 effective_ability = base_ability + mastery_modifier + counter_modifier + meta_modifier

 mastery_modifier: S=+2, A=0, B=-3, C=-6
 counter_modifier: 被克制方 -2（如果对手同位置英雄克制自己的英雄）
 meta_modifier: 英雄风格匹配当前 Meta → +1

 修正后的 effective_ability 替换
 MatchPlayerInfo.ability，后续流程（特性、稳定性噪声、状态、Meta
 位置权重、Carry/Drag）不变。

 4.2 影响程度验证
 ┌────────────────────────────┬────────────┬──────────────┬──────────┐
 │            场景            │ 单选手修正 │ 团队战力影响 │ 胜率变化 │
 ├────────────────────────────┼────────────┼──────────────┼──────────┤
 │ S 级 + 克制 + Meta vs A 级 │ +5 vs 0    │ +1.0         │ +4%      │
 ├────────────────────────────┼────────────┼──────────────┼──────────┤
 │ 3 人 S 级 vs 3 人 B 级     │ +15 vs -9  │ +3.0         │ +10%     │
 ├────────────────────────────┼────────────┼──────────────┼──────────┤
 │ 明星被 Ban 穿用 C 级       │ -6         │ -1.2         │ -5%      │
 ├────────────────────────────┼────────────┼──────────────┼──────────┤
 │ 典型 BP 优势               │ 综合       │ +1.5~3.0     │ +5~10%   │
 └────────────────────────────┴────────────┴──────────────┴──────────┘
 符合"中度影响"目标。

 4.3 快速模拟路径集成 (game_flow.rs)

 simulate_all_phase_matches 修改：

 1. 在加载球队选手数据时，同时加载所有选手的英雄熟练度到内存
 2. 在 simulate_match_with_traits 的逐局循环中，每局先运行 BPEngine::simulate_draft()
 3. 根据 BP 结果修正每个选手的 ability
 4. 传入修正后的选手数据进行比赛模拟

 实现方式：新增 simulate_match_with_draft 方法，封装逐局 BP + 模拟逻辑。

 4.4 详细模拟路径集成 (match_commands.rs)

 simulate_match_detailed 修改：

 1. 在每局模拟前运行 BP
 2. 将 BP 结果存入 game_draft_results 表
 3. 在 game_player_performances 表中记录每个选手使用的英雄

 新增存储表：

 CREATE TABLE IF NOT EXISTS game_draft_results (
     id INTEGER PRIMARY KEY AUTOINCREMENT,
     save_id TEXT NOT NULL,
     match_id INTEGER NOT NULL,
     game_number INTEGER NOT NULL,
     blue_team_id INTEGER NOT NULL,
     red_team_id INTEGER NOT NULL,
     bans_json TEXT NOT NULL,
     blue_picks_json TEXT NOT NULL,
     red_picks_json TEXT NOT NULL,
     UNIQUE(save_id, match_id, game_number)
 );

 game_player_performances 表增加：
 ALTER TABLE game_player_performances ADD COLUMN champion_id INTEGER;
 ALTER TABLE game_player_performances ADD COLUMN mastery_tier TEXT;

 ---
 第五部分：赛季演变（后续增强，MVP 不包含）

 - 熟练度成长：每赛季，B 级有 10% 概率升 A 级，A 级有 5% 概率升 S 级
 - 老将衰退：30 岁以上选手每赛季 10% 概率 S 级降 A 级
 - 新英雄加入：每 2 个赛季新增 1-2 个英雄（上限 70 个），所有选手从 C 级开始

 ---
 实施顺序

 MVP 阶段（8 步）
 ┌──────┬───────────────┬───────────────────────────────────────┐
 │ 步骤 │     内容      │                 文件                  │
 ├──────┼───────────────┼───────────────────────────────────────┤
 │ 1    │ 英雄数据定义  │ 新增 engines/champion.rs              │
 ├──────┼───────────────┼───────────────────────────────────────┤
 │ 2    │ BP 引擎       │ 新增 engines/bp_engine.rs             │
 ├──────┼───────────────┼───────────────────────────────────────┤
 │ 3    │ 注册模块      │ 修改 engines/mod.rs                   │
 ├──────┼───────────────┼───────────────────────────────────────┤
 │ 4    │ 数据库迁移    │ 修改 db/connection.rs                 │
 ├──────┼───────────────┼───────────────────────────────────────┤
 │ 5    │ MetaType 联动 │ 修改 engines/meta_engine.rs           │
 ├──────┼───────────────┼───────────────────────────────────────┤
 │ 6    │ 初始数据生成  │ 修改 services/init_service.rs         │
 ├──────┼───────────────┼───────────────────────────────────────┤
 │ 7    │ 比赛模拟集成  │ 修改 engines/match_simulation.rs      │
 ├──────┼───────────────┼───────────────────────────────────────┤
 │ 8    │ 模拟路径更新  │ 修改 game_flow.rs + match_commands.rs │
 └──────┴───────────────┴───────────────────────────────────────┘
 后续阶段

 - 前端：BP 结果展示、英雄池查看、选禁回放
 - 赛季演变：熟练度成长/衰退、新英雄加入
 - AI 性格联动：激进型球队偏好 Aggressive 英雄
 - 用户操控：玩家队手动 Ban/Pick

 验证

 cargo check --manifest-path src-tauri/Cargo.toml

 编译通过后，可通过以下方式验证：
 1. 创建新存档，检查 player_champion_mastery 表有 ~1,680 行数据
 2. 运行快速模拟一个赛季，确认比赛结果正常
 3. 运行详细模拟单场比赛，检查 game_draft_results 和 game_player_performances 有 BP 数据