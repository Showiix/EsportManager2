/**
 * 真实选手数据配置
 * 每支队伍 5 名选手：TOP, JUG, MID, ADC, SUP
 */

import type { PlayerPosition, PlayerTalent } from '@/types/player'

export interface PlayerConfig {
  gameId: string      // 游戏ID
  name: string        // 真实姓名
  position: PlayerPosition
  age: number
  ability: number     // 能力值 (0-100)
  potential: number   // 潜力值
  tag: PlayerTalent   // 天赋标签
}

export interface TeamPlayersConfig {
  teamId: string
  teamName: string
  shortName: string
  regionId: string
  players: PlayerConfig[]
}

// LPL 战队选手数据
const LPL_PLAYERS: TeamPlayersConfig[] = [
  {
    teamId: '1', teamName: 'JD Gaming', shortName: 'JDG', regionId: 'LPL',
    players: [
      { gameId: '369', name: '白家浩', position: 'TOP', age: 23, ability: 88, potential: 90, tag: 'GENIUS' },
      { gameId: 'Kanavi', name: '徐进赫', position: 'JUG', age: 24, ability: 90, potential: 88, tag: 'GENIUS' },
      { gameId: 'Yagao', name: '曾奇', position: 'MID', age: 24, ability: 82, potential: 80, tag: 'NORMAL' },
      { gameId: 'Ruler', name: '朴载赫', position: 'ADC', age: 26, ability: 91, potential: 85, tag: 'GENIUS' },
      { gameId: 'Missing', name: '娄卓', position: 'SUP', age: 23, ability: 84, potential: 82, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '2', teamName: 'Bilibili Gaming', shortName: 'BLG', regionId: 'LPL',
    players: [
      { gameId: 'Bin', name: '陈泽彬', position: 'TOP', age: 22, ability: 89, potential: 92, tag: 'GENIUS' },
      { gameId: 'Xun', name: '彭立勋', position: 'JUG', age: 21, ability: 86, potential: 90, tag: 'GENIUS' },
      { gameId: 'knight', name: '卓定', position: 'MID', age: 23, ability: 92, potential: 90, tag: 'GENIUS' },
      { gameId: 'Elk', name: '马哲宇', position: 'ADC', age: 21, ability: 87, potential: 90, tag: 'GENIUS' },
      { gameId: 'ON', name: '蔡傑明', position: 'SUP', age: 22, ability: 85, potential: 86, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '3', teamName: 'Top Esports', shortName: 'TES', regionId: 'LPL',
    players: [
      { gameId: 'Wayward', name: '江炀', position: 'TOP', age: 21, ability: 84, potential: 88, tag: 'NORMAL' },
      { gameId: 'Tian', name: '高天亮', position: 'JUG', age: 24, ability: 85, potential: 82, tag: 'NORMAL' },
      { gameId: 'Creme', name: '周炎', position: 'MID', age: 20, ability: 83, potential: 90, tag: 'GENIUS' },
      { gameId: 'JackeyLove', name: '喻文波', position: 'ADC', age: 24, ability: 88, potential: 85, tag: 'GENIUS' },
      { gameId: 'Meiko', name: '田野', position: 'SUP', age: 26, ability: 87, potential: 80, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '4', teamName: 'Weibo Gaming', shortName: 'WBG', regionId: 'LPL',
    players: [
      { gameId: 'TheShy', name: '姜承録', position: 'TOP', age: 25, ability: 86, potential: 78, tag: 'GENIUS' },
      { gameId: 'Tarzan', name: '李承龙', position: 'JUG', age: 25, ability: 84, potential: 80, tag: 'NORMAL' },
      { gameId: 'Xiaohu', name: '李元浩', position: 'MID', age: 27, ability: 85, potential: 75, tag: 'NORMAL' },
      { gameId: 'Light', name: '王光宇', position: 'ADC', age: 22, ability: 84, potential: 88, tag: 'NORMAL' },
      { gameId: 'Crisp', name: '刘青松', position: 'SUP', age: 26, ability: 83, potential: 78, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '5', teamName: 'LNG Esports', shortName: 'LNG', regionId: 'LPL',
    players: [
      { gameId: 'Zika', name: '张浚豪', position: 'TOP', age: 21, ability: 83, potential: 88, tag: 'NORMAL' },
      { gameId: 'Weiwei', name: '周威伟', position: 'JUG', age: 23, ability: 82, potential: 84, tag: 'NORMAL' },
      { gameId: 'Scout', name: '李汭燦', position: 'MID', age: 26, ability: 87, potential: 80, tag: 'GENIUS' },
      { gameId: 'GALA', name: '陈唯', position: 'ADC', age: 24, ability: 86, potential: 84, tag: 'NORMAL' },
      { gameId: 'Hang', name: '吴明宸', position: 'SUP', age: 22, ability: 80, potential: 84, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '6', teamName: 'EDward Gaming', shortName: 'EDG', regionId: 'LPL',
    players: [
      { gameId: 'Ale', name: '谢熙勋', position: 'TOP', age: 22, ability: 82, potential: 86, tag: 'NORMAL' },
      { gameId: 'Jiejie', name: '赵礼杰', position: 'JUG', age: 23, ability: 84, potential: 85, tag: 'NORMAL' },
      { gameId: 'Fofo', name: '黄以軒', position: 'MID', age: 25, ability: 81, potential: 78, tag: 'NORMAL' },
      { gameId: 'Leave', name: '孙伟平', position: 'ADC', age: 21, ability: 83, potential: 88, tag: 'NORMAL' },
      { gameId: 'Meiko2', name: '郑宇承', position: 'SUP', age: 22, ability: 80, potential: 84, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '7', teamName: 'FunPlus Phoenix', shortName: 'FPX', regionId: 'LPL',
    players: [
      { gameId: 'Xiaolaohu', name: '张钊', position: 'TOP', age: 22, ability: 80, potential: 84, tag: 'NORMAL' },
      { gameId: 'Beichuan', name: '北川', position: 'JUG', age: 21, ability: 78, potential: 85, tag: 'NORMAL' },
      { gameId: 'Care', name: '郭建德', position: 'MID', age: 22, ability: 79, potential: 83, tag: 'NORMAL' },
      { gameId: 'Lwx', name: '林炜翔', position: 'ADC', age: 25, ability: 82, potential: 78, tag: 'NORMAL' },
      { gameId: 'Lele', name: '李乐', position: 'SUP', age: 22, ability: 78, potential: 82, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '8', teamName: 'Royal Never Give Up', shortName: 'RNG', regionId: 'LPL',
    players: [
      { gameId: 'Breathe', name: '陈念', position: 'TOP', age: 23, ability: 81, potential: 83, tag: 'NORMAL' },
      { gameId: 'Wei', name: '魏博文', position: 'JUG', age: 23, ability: 82, potential: 84, tag: 'NORMAL' },
      { gameId: 'Tangyuan', name: '胡佳明', position: 'MID', age: 20, ability: 78, potential: 86, tag: 'NORMAL' },
      { gameId: 'Betty', name: '王浩', position: 'ADC', age: 26, ability: 79, potential: 75, tag: 'NORMAL' },
      { gameId: 'LvMao', name: '李鑫', position: 'SUP', age: 24, ability: 78, potential: 78, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '9', teamName: 'Invictus Gaming', shortName: 'IG', regionId: 'LPL',
    players: [
      { gameId: 'YSKM', name: '姚文锦', position: 'TOP', age: 20, ability: 78, potential: 86, tag: 'NORMAL' },
      { gameId: 'Tianzhen', name: '天真', position: 'JUG', age: 21, ability: 76, potential: 84, tag: 'NORMAL' },
      { gameId: 'Dove', name: '邓宇杰', position: 'MID', age: 24, ability: 77, potential: 78, tag: 'NORMAL' },
      { gameId: 'Ahn', name: '安志明', position: 'ADC', age: 22, ability: 78, potential: 82, tag: 'NORMAL' },
      { gameId: 'Wink', name: '刘源', position: 'SUP', age: 23, ability: 76, potential: 80, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '10', teamName: 'Oh My God', shortName: 'OMG', regionId: 'LPL',
    players: [
      { gameId: 'shanji', name: '陈浩杰', position: 'TOP', age: 21, ability: 77, potential: 84, tag: 'NORMAL' },
      { gameId: 'Aki', name: '秋威', position: 'JUG', age: 22, ability: 76, potential: 82, tag: 'NORMAL' },
      { gameId: 'Creme2', name: '周尧', position: 'MID', age: 21, ability: 78, potential: 84, tag: 'NORMAL' },
      { gameId: 'Able', name: '文能', position: 'ADC', age: 23, ability: 77, potential: 80, tag: 'NORMAL' },
      { gameId: 'ppgod', name: '彭俊智', position: 'SUP', age: 24, ability: 78, potential: 78, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '11', teamName: 'Team WE', shortName: 'WE', regionId: 'LPL',
    players: [
      { gameId: 'Cube', name: '严浩', position: 'TOP', age: 21, ability: 75, potential: 82, tag: 'NORMAL' },
      { gameId: 'Heng', name: '吴恒', position: 'JUG', age: 20, ability: 74, potential: 84, tag: 'NORMAL' },
      { gameId: 'Shanks', name: '江星宇', position: 'MID', age: 23, ability: 76, potential: 80, tag: 'NORMAL' },
      { gameId: 'Hope', name: '王杰', position: 'ADC', age: 24, ability: 77, potential: 78, tag: 'NORMAL' },
      { gameId: 'Iwandy', name: '梁益旺', position: 'SUP', age: 23, ability: 75, potential: 78, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '12', teamName: 'Rare Atom', shortName: 'RA', regionId: 'LPL',
    players: [
      { gameId: 'Xiaoxu', name: '李晓旭', position: 'TOP', age: 22, ability: 74, potential: 80, tag: 'NORMAL' },
      { gameId: 'Leyan', name: '赵乐言', position: 'JUG', age: 22, ability: 75, potential: 80, tag: 'NORMAL' },
      { gameId: 'Strive', name: '张奋', position: 'MID', age: 21, ability: 74, potential: 82, tag: 'NORMAL' },
      { gameId: 'iBoy', name: '胡显昭', position: 'ADC', age: 24, ability: 76, potential: 76, tag: 'NORMAL' },
      { gameId: 'Yaoyao', name: '姚耀', position: 'SUP', age: 22, ability: 73, potential: 78, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '13', teamName: 'Anyone Legend', shortName: 'AL', regionId: 'LPL',
    players: [
      { gameId: 'Zdz', name: '张德昭', position: 'TOP', age: 20, ability: 72, potential: 82, tag: 'NORMAL' },
      { gameId: 'Xiaohao', name: '小号', position: 'JUG', age: 21, ability: 73, potential: 80, tag: 'NORMAL' },
      { gameId: 'Forge', name: '李昊', position: 'MID', age: 22, ability: 74, potential: 80, tag: 'NORMAL' },
      { gameId: 'Asura', name: '阿修罗', position: 'ADC', age: 21, ability: 73, potential: 82, tag: 'NORMAL' },
      { gameId: 'QiuQiu', name: '球球', position: 'SUP', age: 22, ability: 72, potential: 78, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '14', teamName: 'ThunderTalk Gaming', shortName: 'TT', regionId: 'LPL',
    players: [
      { gameId: 'Hoya', name: '何跃', position: 'TOP', age: 23, ability: 71, potential: 78, tag: 'ORDINARY' },
      { gameId: 'Beishang', name: '谭赞', position: 'JUG', age: 24, ability: 72, potential: 76, tag: 'ORDINARY' },
      { gameId: 'Ucal', name: '孙宇灿', position: 'MID', age: 24, ability: 73, potential: 76, tag: 'ORDINARY' },
      { gameId: 'Puff', name: '樸載永', position: 'ADC', age: 25, ability: 72, potential: 74, tag: 'ORDINARY' },
      { gameId: 'Southwind', name: '南风', position: 'SUP', age: 23, ability: 71, potential: 76, tag: 'ORDINARY' },
    ]
  },
]

// LCK 战队选手数据
const LCK_PLAYERS: TeamPlayersConfig[] = [
  {
    teamId: '101', teamName: 'T1', shortName: 'T1', regionId: 'LCK',
    players: [
      { gameId: 'Zeus', name: '崔宇帝', position: 'TOP', age: 20, ability: 90, potential: 95, tag: 'GENIUS' },
      { gameId: 'Oner', name: '文铉俊', position: 'JUG', age: 21, ability: 88, potential: 92, tag: 'GENIUS' },
      { gameId: 'Faker', name: '李相赫', position: 'MID', age: 28, ability: 93, potential: 80, tag: 'GENIUS' },
      { gameId: 'Gumayusi', name: '李敏亨', position: 'ADC', age: 22, ability: 89, potential: 90, tag: 'GENIUS' },
      { gameId: 'Keria', name: '柳旼锡', position: 'SUP', age: 21, ability: 91, potential: 93, tag: 'GENIUS' },
    ]
  },
  {
    teamId: '102', teamName: 'Gen.G', shortName: 'GEN', regionId: 'LCK',
    players: [
      { gameId: 'Kiin', name: '金基仁', position: 'TOP', age: 25, ability: 88, potential: 85, tag: 'GENIUS' },
      { gameId: 'Canyon', name: '金建步', position: 'JUG', age: 23, ability: 91, potential: 90, tag: 'GENIUS' },
      { gameId: 'Chovy', name: '郑志勋', position: 'MID', age: 23, ability: 92, potential: 92, tag: 'GENIUS' },
      { gameId: 'Peyz', name: '金成珉', position: 'ADC', age: 19, ability: 86, potential: 94, tag: 'GENIUS' },
      { gameId: 'Lehends', name: '孙诗佑', position: 'SUP', age: 26, ability: 85, potential: 80, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '103', teamName: 'Dplus KIA', shortName: 'DK', regionId: 'LCK',
    players: [
      { gameId: 'Canna', name: '金昌东', position: 'TOP', age: 23, ability: 83, potential: 85, tag: 'NORMAL' },
      { gameId: 'Lucid', name: '全镇赫', position: 'JUG', age: 20, ability: 84, potential: 90, tag: 'GENIUS' },
      { gameId: 'ShowMaker', name: '许誠焄', position: 'MID', age: 23, ability: 90, potential: 88, tag: 'GENIUS' },
      { gameId: 'Aiming', name: '金河睿', position: 'ADC', age: 24, ability: 86, potential: 84, tag: 'NORMAL' },
      { gameId: 'Kellin', name: '金亨圭', position: 'SUP', age: 24, ability: 82, potential: 80, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '104', teamName: 'Hanwha Life', shortName: 'HLE', regionId: 'LCK',
    players: [
      { gameId: 'Doran', name: '崔镇赫', position: 'TOP', age: 23, ability: 84, potential: 85, tag: 'NORMAL' },
      { gameId: 'Peanut', name: '韩旺昊', position: 'JUG', age: 26, ability: 86, potential: 80, tag: 'NORMAL' },
      { gameId: 'Zeka', name: '金建佑', position: 'MID', age: 21, ability: 87, potential: 90, tag: 'GENIUS' },
      { gameId: 'Viper', name: '朴道贤', position: 'ADC', age: 24, ability: 89, potential: 86, tag: 'GENIUS' },
      { gameId: 'Delight', name: '金荣俊', position: 'SUP', age: 23, ability: 83, potential: 84, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '105', teamName: 'KT Rolster', shortName: 'KT', regionId: 'LCK',
    players: [
      { gameId: 'Kingen', name: '金敬镐', position: 'TOP', age: 24, ability: 83, potential: 84, tag: 'NORMAL' },
      { gameId: 'Cuzz', name: '文宇灿', position: 'JUG', age: 25, ability: 82, potential: 80, tag: 'NORMAL' },
      { gameId: 'Bdd', name: '郭炳書', position: 'MID', age: 25, ability: 84, potential: 82, tag: 'NORMAL' },
      { gameId: 'Deft', name: '金赫奎', position: 'ADC', age: 27, ability: 87, potential: 78, tag: 'GENIUS' },
      { gameId: 'BeryL', name: '曹世赫', position: 'SUP', age: 26, ability: 84, potential: 80, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '106', teamName: 'DRX', shortName: 'DRX', regionId: 'LCK',
    players: [
      { gameId: 'Rascal', name: '金光喜', position: 'TOP', age: 25, ability: 81, potential: 80, tag: 'NORMAL' },
      { gameId: 'Sponge', name: '林亨俊', position: 'JUG', age: 22, ability: 80, potential: 84, tag: 'NORMAL' },
      { gameId: 'Fate', name: '柳秀赫', position: 'MID', age: 24, ability: 82, potential: 82, tag: 'NORMAL' },
      { gameId: 'Teddy', name: '朴镇成', position: 'ADC', age: 25, ability: 83, potential: 80, tag: 'NORMAL' },
      { gameId: 'Pleata', name: '黄镇浩', position: 'SUP', age: 22, ability: 79, potential: 82, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '107', teamName: 'Kwangdong Freecs', shortName: 'KDF', regionId: 'LCK',
    players: [
      { gameId: 'DuDu', name: '崔镇赫', position: 'TOP', age: 22, ability: 79, potential: 84, tag: 'NORMAL' },
      { gameId: 'YoungJae', name: '金英在', position: 'JUG', age: 21, ability: 78, potential: 84, tag: 'NORMAL' },
      { gameId: 'BuLLDoG', name: '金城浩', position: 'MID', age: 20, ability: 80, potential: 86, tag: 'NORMAL' },
      { gameId: 'Taeyoon', name: '金泰润', position: 'ADC', age: 21, ability: 79, potential: 84, tag: 'NORMAL' },
      { gameId: 'Jun', name: '李俊镐', position: 'SUP', age: 22, ability: 78, potential: 82, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '108', teamName: 'Nongshim RedForce', shortName: 'NS', regionId: 'LCK',
    players: [
      { gameId: 'DnDn', name: '崔正焕', position: 'TOP', age: 21, ability: 78, potential: 84, tag: 'NORMAL' },
      { gameId: 'Sylvie', name: '金旼玄', position: 'JUG', age: 20, ability: 77, potential: 86, tag: 'NORMAL' },
      { gameId: 'Fisher', name: '金在勋', position: 'MID', age: 21, ability: 79, potential: 84, tag: 'NORMAL' },
      { gameId: 'Vital', name: '朴志赫', position: 'ADC', age: 20, ability: 78, potential: 86, tag: 'NORMAL' },
      { gameId: 'Peter', name: '李承俊', position: 'SUP', age: 22, ability: 77, potential: 82, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '109', teamName: 'Liiv SANDBOX', shortName: 'LSB', regionId: 'LCK',
    players: [
      { gameId: 'Burdol', name: '朴晙书', position: 'TOP', age: 20, ability: 77, potential: 86, tag: 'NORMAL' },
      { gameId: 'Willer', name: '文浩成', position: 'JUG', age: 23, ability: 76, potential: 80, tag: 'NORMAL' },
      { gameId: 'Clozer', name: '李周城', position: 'MID', age: 21, ability: 78, potential: 84, tag: 'NORMAL' },
      { gameId: 'Envyy', name: '金佑赫', position: 'ADC', age: 20, ability: 77, potential: 86, tag: 'NORMAL' },
      { gameId: 'Execute', name: '崔胜桓', position: 'SUP', age: 21, ability: 76, potential: 82, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '110', teamName: 'BNK FearX', shortName: 'FOX', regionId: 'LCK',
    players: [
      { gameId: 'Clear', name: '金贤宇', position: 'TOP', age: 21, ability: 76, potential: 84, tag: 'NORMAL' },
      { gameId: 'Raptor', name: '金成镐', position: 'JUG', age: 22, ability: 75, potential: 82, tag: 'NORMAL' },
      { gameId: 'Karis', name: '朴书俊', position: 'MID', age: 20, ability: 77, potential: 86, tag: 'NORMAL' },
      { gameId: 'Hena', name: '金道赫', position: 'ADC', age: 21, ability: 76, potential: 84, tag: 'NORMAL' },
      { gameId: 'Andil', name: '崔镇赫', position: 'SUP', age: 22, ability: 75, potential: 80, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '111', teamName: 'OKSavingsBank BRION', shortName: 'BRO', regionId: 'LCK',
    players: [
      { gameId: 'Morgan', name: '金廷星', position: 'TOP', age: 24, ability: 75, potential: 78, tag: 'ORDINARY' },
      { gameId: 'UmTi', name: '崔宇城', position: 'JUG', age: 24, ability: 74, potential: 78, tag: 'ORDINARY' },
      { gameId: 'Kuro', name: '李书行', position: 'MID', age: 27, ability: 76, potential: 74, tag: 'ORDINARY' },
      { gameId: 'Hybrid', name: '申振成', position: 'ADC', age: 22, ability: 75, potential: 80, tag: 'NORMAL' },
      { gameId: 'Effort', name: '李相昊', position: 'SUP', age: 23, ability: 74, potential: 78, tag: 'ORDINARY' },
    ]
  },
  {
    teamId: '112', teamName: 'Fredit BRION', shortName: 'FB', regionId: 'LCK',
    players: [
      { gameId: 'Soboro', name: '崔镇赫', position: 'TOP', age: 22, ability: 73, potential: 80, tag: 'ORDINARY' },
      { gameId: 'Plex', name: '金镇赫', position: 'JUG', age: 21, ability: 72, potential: 80, tag: 'ORDINARY' },
      { gameId: 'Mireu', name: '金明在', position: 'MID', age: 20, ability: 74, potential: 82, tag: 'NORMAL' },
      { gameId: 'Paduck', name: '金成镐', position: 'ADC', age: 21, ability: 73, potential: 80, tag: 'ORDINARY' },
      { gameId: 'Soul', name: '金柱永', position: 'SUP', age: 22, ability: 72, potential: 78, tag: 'ORDINARY' },
    ]
  },
  {
    teamId: '113', teamName: 'DN Freecs', shortName: 'DNF', regionId: 'LCK',
    players: [
      { gameId: 'Rich', name: '李在元', position: 'TOP', age: 24, ability: 72, potential: 76, tag: 'ORDINARY' },
      { gameId: 'Gideon', name: '朴镇浩', position: 'JUG', age: 21, ability: 71, potential: 80, tag: 'ORDINARY' },
      { gameId: 'Mask', name: '金成镐', position: 'MID', age: 20, ability: 73, potential: 82, tag: 'NORMAL' },
      { gameId: 'Route', name: '金成镐', position: 'ADC', age: 23, ability: 72, potential: 78, tag: 'ORDINARY' },
      { gameId: 'GuGer', name: '金成镐', position: 'SUP', age: 22, ability: 71, potential: 78, tag: 'ORDINARY' },
    ]
  },
  {
    teamId: '114', teamName: 'Daejon Stars', shortName: 'DJS', regionId: 'LCK',
    players: [
      { gameId: 'Sword', name: '崔成元', position: 'TOP', age: 24, ability: 70, potential: 74, tag: 'ORDINARY' },
      { gameId: 'Malrang', name: '金京锡', position: 'JUG', age: 25, ability: 72, potential: 75, tag: 'ORDINARY' },
      { gameId: 'Gori', name: '金泰旿', position: 'MID', age: 23, ability: 71, potential: 76, tag: 'ORDINARY' },
      { gameId: 'Prince', name: '李载夏', position: 'ADC', age: 24, ability: 71, potential: 76, tag: 'ORDINARY' },
      { gameId: 'Life', name: '金正敏', position: 'SUP', age: 25, ability: 70, potential: 74, tag: 'ORDINARY' },
    ]
  },
]

// LEC 战队选手数据
const LEC_PLAYERS: TeamPlayersConfig[] = [
  {
    teamId: '201', teamName: 'G2 Esports', shortName: 'G2', regionId: 'LEC',
    players: [
      { gameId: 'BrokenBlade', name: 'Sergen Çelik', position: 'TOP', age: 24, ability: 86, potential: 86, tag: 'GENIUS' },
      { gameId: 'Yike', name: 'Martin Sundelin', position: 'JUG', age: 22, ability: 84, potential: 88, tag: 'NORMAL' },
      { gameId: 'Caps', name: 'Rasmus Winther', position: 'MID', age: 24, ability: 89, potential: 88, tag: 'GENIUS' },
      { gameId: 'Hans Sama', name: 'Steven Liv', position: 'ADC', age: 24, ability: 85, potential: 84, tag: 'NORMAL' },
      { gameId: 'Mikyx', name: 'Mihael Mehle', position: 'SUP', age: 25, ability: 84, potential: 82, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '202', teamName: 'Fnatic', shortName: 'FNC', regionId: 'LEC',
    players: [
      { gameId: 'Oscarinin', name: 'Oscar Muñoz', position: 'TOP', age: 21, ability: 83, potential: 88, tag: 'NORMAL' },
      { gameId: 'Razork', name: 'Iván Díaz', position: 'JUG', age: 23, ability: 82, potential: 84, tag: 'NORMAL' },
      { gameId: 'Humanoid', name: 'Marek Brázda', position: 'MID', age: 24, ability: 85, potential: 84, tag: 'NORMAL' },
      { gameId: 'Noah', name: 'Oh Hyeon-taek', position: 'ADC', age: 21, ability: 84, potential: 88, tag: 'NORMAL' },
      { gameId: 'Jun', name: 'Yoon Se-jun', position: 'SUP', age: 22, ability: 82, potential: 86, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '203', teamName: 'MAD Lions', shortName: 'MAD', regionId: 'LEC',
    players: [
      { gameId: 'Myrwn', name: 'Myrwn Tomiello', position: 'TOP', age: 20, ability: 80, potential: 86, tag: 'NORMAL' },
      { gameId: 'Elyoya', name: 'Javier Batalla', position: 'JUG', age: 23, ability: 84, potential: 86, tag: 'NORMAL' },
      { gameId: 'Fresskowy', name: 'Bartosz Przybylski', position: 'MID', age: 21, ability: 81, potential: 86, tag: 'NORMAL' },
      { gameId: 'Supa', name: 'Paweł Cańdo', position: 'ADC', age: 20, ability: 82, potential: 88, tag: 'NORMAL' },
      { gameId: 'Alvaro', name: 'Álvaro González', position: 'SUP', age: 22, ability: 80, potential: 84, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '204', teamName: 'Team Vitality', shortName: 'VIT', regionId: 'LEC',
    players: [
      { gameId: 'Photon', name: 'Kim Jun-hyeong', position: 'TOP', age: 20, ability: 82, potential: 88, tag: 'NORMAL' },
      { gameId: 'Daglas', name: 'Daglas Daglas', position: 'JUG', age: 21, ability: 80, potential: 86, tag: 'NORMAL' },
      { gameId: 'Perkz', name: 'Luka Perković', position: 'MID', age: 25, ability: 84, potential: 80, tag: 'NORMAL' },
      { gameId: 'Carzzy', name: 'Matyáš Orság', position: 'ADC', age: 22, ability: 83, potential: 84, tag: 'NORMAL' },
      { gameId: 'Kaiser', name: 'Norman Kaiser', position: 'SUP', age: 24, ability: 82, potential: 82, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '205', teamName: 'Rogue', shortName: 'RGE', regionId: 'LEC',
    players: [
      { gameId: 'Szygenda', name: 'Emil Szygenda', position: 'TOP', age: 22, ability: 80, potential: 84, tag: 'NORMAL' },
      { gameId: 'Markoon', name: 'Mark van Woensel', position: 'JUG', age: 22, ability: 79, potential: 84, tag: 'NORMAL' },
      { gameId: 'Larssen', name: 'Emil Larsson', position: 'MID', age: 24, ability: 82, potential: 82, tag: 'NORMAL' },
      { gameId: 'Comp', name: 'Markos Stamkopoulos', position: 'ADC', age: 22, ability: 81, potential: 84, tag: 'NORMAL' },
      { gameId: 'Zoelys', name: 'Benoît Fabre', position: 'SUP', age: 21, ability: 79, potential: 84, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '206', teamName: 'Excel Esports', shortName: 'XL', regionId: 'LEC',
    players: [
      { gameId: 'Odoamne', name: 'Andrei Pascu', position: 'TOP', age: 28, ability: 80, potential: 75, tag: 'NORMAL' },
      { gameId: 'Xerxe', name: 'Andrei Dragomir', position: 'JUG', age: 25, ability: 79, potential: 78, tag: 'NORMAL' },
      { gameId: 'Abbedagge', name: 'Felix Braun', position: 'MID', age: 24, ability: 81, potential: 80, tag: 'NORMAL' },
      { gameId: 'Patrik', name: 'Patrik Jírů', position: 'ADC', age: 25, ability: 80, potential: 78, tag: 'NORMAL' },
      { gameId: 'Targamas', name: 'Raphaël Crabbé', position: 'SUP', age: 24, ability: 78, potential: 78, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '207', teamName: 'Team BDS', shortName: 'BDS', regionId: 'LEC',
    players: [
      { gameId: 'Adam', name: 'Adam Maanane', position: 'TOP', age: 21, ability: 79, potential: 86, tag: 'NORMAL' },
      { gameId: 'Sheo', name: 'Hugo Seonnet', position: 'JUG', age: 22, ability: 78, potential: 84, tag: 'NORMAL' },
      { gameId: 'nuc', name: 'Ilias Bizriken', position: 'MID', age: 20, ability: 80, potential: 86, tag: 'NORMAL' },
      { gameId: 'Ice', name: 'Ilias Bizriken', position: 'ADC', age: 21, ability: 79, potential: 84, tag: 'NORMAL' },
      { gameId: 'Labrov', name: 'Labros Papoutsakis', position: 'SUP', age: 23, ability: 78, potential: 82, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '208', teamName: 'SK Gaming', shortName: 'SK', regionId: 'LEC',
    players: [
      { gameId: 'Irrelevant', name: 'Irrelevant Player', position: 'TOP', age: 21, ability: 77, potential: 84, tag: 'NORMAL' },
      { gameId: 'Isma', name: 'Isma Player', position: 'JUG', age: 22, ability: 76, potential: 82, tag: 'NORMAL' },
      { gameId: 'Sertuss', name: 'Sertuss Player', position: 'MID', age: 23, ability: 78, potential: 82, tag: 'NORMAL' },
      { gameId: 'Exakick', name: 'Exakick Player', position: 'ADC', age: 21, ability: 77, potential: 84, tag: 'NORMAL' },
      { gameId: 'Doss', name: 'Doss Player', position: 'SUP', age: 22, ability: 76, potential: 80, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '209', teamName: 'Astralis', shortName: 'AST', regionId: 'LEC',
    players: [
      { gameId: 'Finn', name: 'Finn Wiestål', position: 'TOP', age: 25, ability: 76, potential: 78, tag: 'NORMAL' },
      { gameId: 'Zanzarah', name: 'Zanzarah Player', position: 'JUG', age: 24, ability: 75, potential: 78, tag: 'NORMAL' },
      { gameId: '113', name: '113 Player', position: 'MID', age: 22, ability: 77, potential: 82, tag: 'NORMAL' },
      { gameId: 'Kobbe', name: 'Kasper Kobberup', position: 'ADC', age: 27, ability: 78, potential: 75, tag: 'NORMAL' },
      { gameId: 'JeongHoon', name: 'JeongHoon Player', position: 'SUP', age: 23, ability: 76, potential: 80, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '210', teamName: 'Team Heretics', shortName: 'TH', regionId: 'LEC',
    players: [
      { gameId: 'Wunder', name: 'Martin Hansen', position: 'TOP', age: 25, ability: 80, potential: 78, tag: 'NORMAL' },
      { gameId: 'Jankos', name: 'Marcin Jankowski', position: 'JUG', age: 28, ability: 82, potential: 75, tag: 'NORMAL' },
      { gameId: 'Vetheo', name: 'Vincent Berrié', position: 'MID', age: 22, ability: 81, potential: 84, tag: 'NORMAL' },
      { gameId: 'Flakked', name: 'Víctor Lirola', position: 'ADC', age: 22, ability: 79, potential: 82, tag: 'NORMAL' },
      { gameId: 'Trymbi', name: 'Adrian Trybus', position: 'SUP', age: 24, ability: 78, potential: 80, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '211', teamName: 'Giants Gaming', shortName: 'GIA', regionId: 'LEC',
    players: [
      { gameId: 'Th3Antonio', name: 'Antonio Espinosa', position: 'TOP', age: 22, ability: 74, potential: 80, tag: 'ORDINARY' },
      { gameId: 'Maxlore', name: 'Nubar Sarafian', position: 'JUG', age: 26, ability: 75, potential: 76, tag: 'ORDINARY' },
      { gameId: 'Czekolansen', name: 'Czekolansen Player', position: 'MID', age: 21, ability: 75, potential: 82, tag: 'NORMAL' },
      { gameId: 'Attila', name: 'Amadeu Carvalho', position: 'ADC', age: 26, ability: 74, potential: 75, tag: 'ORDINARY' },
      { gameId: 'Steeelback', name: 'Pierre Medjaldi', position: 'SUP', age: 27, ability: 73, potential: 74, tag: 'ORDINARY' },
    ]
  },
  {
    teamId: '212', teamName: 'Karmine Corp', shortName: 'KC', regionId: 'LEC',
    players: [
      { gameId: 'Cabochard', name: 'Lucas Simon', position: 'TOP', age: 27, ability: 78, potential: 76, tag: 'NORMAL' },
      { gameId: 'Skeanz', name: 'Skeanz Player', position: 'JUG', age: 23, ability: 76, potential: 80, tag: 'NORMAL' },
      { gameId: 'SAKEN', name: 'SAKEN Player', position: 'MID', age: 24, ability: 77, potential: 80, tag: 'NORMAL' },
      { gameId: 'Rekkles', name: 'Martin Larsson', position: 'ADC', age: 28, ability: 82, potential: 75, tag: 'NORMAL' },
      { gameId: 'Hantera', name: 'Hantera Player', position: 'SUP', age: 22, ability: 75, potential: 80, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '213', teamName: 'GIANTX', shortName: 'GX', regionId: 'LEC',
    players: [
      { gameId: 'Evi', name: 'Shunsuke Murase', position: 'TOP', age: 28, ability: 75, potential: 74, tag: 'ORDINARY' },
      { gameId: 'Peach', name: 'Peach Player', position: 'JUG', age: 22, ability: 73, potential: 80, tag: 'ORDINARY' },
      { gameId: 'Ruby', name: 'Ruby Player', position: 'MID', age: 21, ability: 74, potential: 82, tag: 'NORMAL' },
      { gameId: 'Jackspektra', name: 'Jakob Spekstra', position: 'ADC', age: 21, ability: 74, potential: 82, tag: 'NORMAL' },
      { gameId: 'Mersa', name: 'Mersa Player', position: 'SUP', age: 22, ability: 73, potential: 80, tag: 'ORDINARY' },
    ]
  },
  {
    teamId: '214', teamName: 'Movistar Riders', shortName: 'MRS', regionId: 'LEC',
    players: [
      { gameId: 'Tolkin', name: 'Tolkin Player', position: 'TOP', age: 21, ability: 72, potential: 80, tag: 'ORDINARY' },
      { gameId: 'Tsjilla', name: 'Tsjilla Player', position: 'JUG', age: 22, ability: 71, potential: 78, tag: 'ORDINARY' },
      { gameId: 'Mav', name: 'Mav Player', position: 'MID', age: 20, ability: 73, potential: 82, tag: 'NORMAL' },
      { gameId: 'Rafitta', name: 'Rafitta Player', position: 'ADC', age: 21, ability: 72, potential: 80, tag: 'ORDINARY' },
      { gameId: 'Marky', name: 'Marky Player', position: 'SUP', age: 22, ability: 71, potential: 78, tag: 'ORDINARY' },
    ]
  },
]

// LCS 战队选手数据
const LCS_PLAYERS: TeamPlayersConfig[] = [
  {
    teamId: '301', teamName: 'Cloud9', shortName: 'C9', regionId: 'LCS',
    players: [
      { gameId: 'Fudge', name: 'Ibrahim Allami', position: 'TOP', age: 22, ability: 82, potential: 86, tag: 'NORMAL' },
      { gameId: 'Blaber', name: 'Robert Huang', position: 'JUG', age: 24, ability: 84, potential: 84, tag: 'NORMAL' },
      { gameId: 'Jojo', name: 'Joseph Pyun', position: 'MID', age: 19, ability: 83, potential: 90, tag: 'GENIUS' },
      { gameId: 'Berserker', name: 'Kim Min-cheol', position: 'ADC', age: 20, ability: 86, potential: 90, tag: 'GENIUS' },
      { gameId: 'Vulcan', name: 'Philippe Laflamme', position: 'SUP', age: 25, ability: 82, potential: 80, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '302', teamName: 'Team Liquid', shortName: 'TL', regionId: 'LCS',
    players: [
      { gameId: 'Summit', name: 'Park Woo-tae', position: 'TOP', age: 23, ability: 83, potential: 84, tag: 'NORMAL' },
      { gameId: 'UmTi', name: 'Um Seong-hyeon', position: 'JUG', age: 23, ability: 80, potential: 82, tag: 'NORMAL' },
      { gameId: 'APA', name: 'Eain Stearns', position: 'MID', age: 21, ability: 82, potential: 88, tag: 'NORMAL' },
      { gameId: 'Yeon', name: 'Sean Sung', position: 'ADC', age: 21, ability: 81, potential: 86, tag: 'NORMAL' },
      { gameId: 'CoreJJ', name: 'Jo Yong-in', position: 'SUP', age: 29, ability: 84, potential: 78, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '303', teamName: '100 Thieves', shortName: '100T', regionId: 'LCS',
    players: [
      { gameId: 'Sniper', name: 'Sniper Player', position: 'TOP', age: 22, ability: 80, potential: 84, tag: 'NORMAL' },
      { gameId: 'River', name: 'Kim Dong-woo', position: 'JUG', age: 24, ability: 81, potential: 82, tag: 'NORMAL' },
      { gameId: 'Quid', name: 'Quid Player', position: 'MID', age: 21, ability: 79, potential: 86, tag: 'NORMAL' },
      { gameId: 'Doublelift', name: 'Yiliang Peng', position: 'ADC', age: 31, ability: 82, potential: 75, tag: 'NORMAL' },
      { gameId: 'Busio', name: 'Busio Player', position: 'SUP', age: 21, ability: 78, potential: 84, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '304', teamName: 'FlyQuest', shortName: 'FLY', regionId: 'LCS',
    players: [
      { gameId: 'Bwipo', name: 'Gabriël Rau', position: 'TOP', age: 25, ability: 82, potential: 80, tag: 'NORMAL' },
      { gameId: 'Inspired', name: 'Kacper Słoma', position: 'JUG', age: 23, ability: 84, potential: 84, tag: 'NORMAL' },
      { gameId: 'Quad', name: 'Quad Player', position: 'MID', age: 21, ability: 80, potential: 86, tag: 'NORMAL' },
      { gameId: 'Massu', name: 'Massu Player', position: 'ADC', age: 22, ability: 79, potential: 84, tag: 'NORMAL' },
      { gameId: 'Ignar', name: 'Lee Dong-geun', position: 'SUP', age: 27, ability: 80, potential: 78, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '305', teamName: 'NRG', shortName: 'NRG', regionId: 'LCS',
    players: [
      { gameId: 'Dhokla', name: 'Niship Doshi', position: 'TOP', age: 25, ability: 78, potential: 80, tag: 'NORMAL' },
      { gameId: 'Contractz', name: 'Juan Garcia', position: 'JUG', age: 25, ability: 79, potential: 78, tag: 'NORMAL' },
      { gameId: 'Palafox', name: 'Cristian Palafox', position: 'MID', age: 23, ability: 78, potential: 82, tag: 'NORMAL' },
      { gameId: 'FBI', name: 'Victor Huang', position: 'ADC', age: 24, ability: 80, potential: 80, tag: 'NORMAL' },
      { gameId: 'Huhi', name: 'Choi Jae-hyun', position: 'SUP', age: 28, ability: 79, potential: 76, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '306', teamName: 'Evil Geniuses', shortName: 'EG', regionId: 'LCS',
    players: [
      { gameId: 'Impact', name: 'Jeong Eon-young', position: 'TOP', age: 29, ability: 81, potential: 76, tag: 'NORMAL' },
      { gameId: 'Sven', name: 'Dennis Johnsen', position: 'JUG', age: 28, ability: 79, potential: 76, tag: 'NORMAL' },
      { gameId: 'Haeri', name: 'Haeri Player', position: 'MID', age: 21, ability: 78, potential: 86, tag: 'NORMAL' },
      { gameId: 'Danny', name: 'Kyle Sakamaki', position: 'ADC', age: 20, ability: 80, potential: 88, tag: 'NORMAL' },
      { gameId: 'Ignar2', name: 'Lee Dong-geun', position: 'SUP', age: 24, ability: 78, potential: 80, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '307', teamName: 'Dignitas', shortName: 'DIG', regionId: 'LCS',
    players: [
      { gameId: 'Licorice', name: 'Eric Ritchie', position: 'TOP', age: 27, ability: 78, potential: 76, tag: 'NORMAL' },
      { gameId: 'Santorin', name: 'Lucas Larsen', position: 'JUG', age: 26, ability: 79, potential: 78, tag: 'NORMAL' },
      { gameId: 'Blue', name: 'Blue Player', position: 'MID', age: 22, ability: 77, potential: 84, tag: 'NORMAL' },
      { gameId: 'Spawn', name: 'Spawn Player', position: 'ADC', age: 21, ability: 76, potential: 84, tag: 'NORMAL' },
      { gameId: 'Biofrost', name: 'Vincent Wang', position: 'SUP', age: 27, ability: 77, potential: 76, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '308', teamName: 'Immortals', shortName: 'IMT', regionId: 'LCS',
    players: [
      { gameId: 'Castle', name: 'Castle Player', position: 'TOP', age: 22, ability: 76, potential: 82, tag: 'NORMAL' },
      { gameId: 'Kenvi', name: 'Kenvi Player', position: 'JUG', age: 21, ability: 77, potential: 86, tag: 'NORMAL' },
      { gameId: 'Mask2', name: 'Mask Player', position: 'MID', age: 22, ability: 75, potential: 82, tag: 'NORMAL' },
      { gameId: 'Arrow', name: 'No Dong-hyeon', position: 'ADC', age: 28, ability: 76, potential: 74, tag: 'ORDINARY' },
      { gameId: 'Olleh', name: 'Kim Joo-sung', position: 'SUP', age: 29, ability: 75, potential: 74, tag: 'ORDINARY' },
    ]
  },
  {
    teamId: '309', teamName: 'TSM', shortName: 'TSM', regionId: 'LCS',
    players: [
      { gameId: 'Solo', name: 'Colin Earnest', position: 'TOP', age: 28, ability: 76, potential: 74, tag: 'ORDINARY' },
      { gameId: 'Bugi', name: 'Bugi Player', position: 'JUG', age: 22, ability: 75, potential: 82, tag: 'NORMAL' },
      { gameId: 'Maple', name: 'Huang Yi-tang', position: 'MID', age: 27, ability: 78, potential: 76, tag: 'NORMAL' },
      { gameId: 'Neo', name: 'Neo Player', position: 'ADC', age: 21, ability: 76, potential: 84, tag: 'NORMAL' },
      { gameId: 'Chime', name: 'Chime Player', position: 'SUP', age: 23, ability: 75, potential: 80, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '310', teamName: 'Golden Guardians', shortName: 'GG', regionId: 'LCS',
    players: [
      { gameId: 'Tony Top', name: 'Tony Player', position: 'TOP', age: 22, ability: 74, potential: 82, tag: 'NORMAL' },
      { gameId: 'Pridestalkr', name: 'Pridestalkr Player', position: 'JUG', age: 23, ability: 75, potential: 80, tag: 'NORMAL' },
      { gameId: 'Gori2', name: 'Gori Player', position: 'MID', age: 24, ability: 76, potential: 78, tag: 'NORMAL' },
      { gameId: 'Stixxay', name: 'Trevor Hayes', position: 'ADC', age: 27, ability: 75, potential: 74, tag: 'ORDINARY' },
      { gameId: 'Poome', name: 'Poome Player', position: 'SUP', age: 23, ability: 74, potential: 78, tag: 'NORMAL' },
    ]
  },
  {
    teamId: '311', teamName: 'Counter Logic Gaming', shortName: 'CLG', regionId: 'LCS',
    players: [
      { gameId: 'Dhokla2', name: 'Dhokla Player', position: 'TOP', age: 24, ability: 73, potential: 78, tag: 'ORDINARY' },
      { gameId: 'Keenan', name: 'Keenan Player', position: 'JUG', age: 22, ability: 74, potential: 82, tag: 'NORMAL' },
      { gameId: 'Triple', name: 'Triple Player', position: 'MID', age: 21, ability: 75, potential: 84, tag: 'NORMAL' },
      { gameId: 'Luger', name: 'Luger Player', position: 'ADC', age: 23, ability: 74, potential: 80, tag: 'NORMAL' },
      { gameId: 'Isles', name: 'Isles Player', position: 'SUP', age: 22, ability: 73, potential: 80, tag: 'ORDINARY' },
    ]
  },
  {
    teamId: '312', teamName: 'Shopify Rebellion', shortName: 'SR', regionId: 'LCS',
    players: [
      { gameId: 'Revenge', name: 'Mohamed Kaddoura', position: 'TOP', age: 23, ability: 74, potential: 80, tag: 'NORMAL' },
      { gameId: 'Closer', name: 'Can Çelik', position: 'JUG', age: 24, ability: 76, potential: 80, tag: 'NORMAL' },
      { gameId: 'Jensen', name: 'Nicolaj Jensen', position: 'MID', age: 29, ability: 80, potential: 76, tag: 'NORMAL' },
      { gameId: 'Bvoy', name: 'Bvoy Player', position: 'ADC', age: 22, ability: 73, potential: 82, tag: 'NORMAL' },
      { gameId: 'Dreams', name: 'Dreams Player', position: 'SUP', age: 24, ability: 74, potential: 78, tag: 'ORDINARY' },
    ]
  },
  {
    teamId: '313', teamName: 'Disguised', shortName: 'DSG', regionId: 'LCS',
    players: [
      { gameId: 'Armao', name: 'Armao Player', position: 'TOP', age: 24, ability: 72, potential: 78, tag: 'ORDINARY' },
      { gameId: 'Tarzaned', name: 'Tarzaned Player', position: 'JUG', age: 26, ability: 73, potential: 76, tag: 'ORDINARY' },
      { gameId: 'Disguised1', name: 'Player One', position: 'MID', age: 21, ability: 74, potential: 82, tag: 'NORMAL' },
      { gameId: 'K1ng', name: 'K1ng Player', position: 'ADC', age: 23, ability: 73, potential: 80, tag: 'ORDINARY' },
      { gameId: 'JayJ', name: 'JayJ Player', position: 'SUP', age: 25, ability: 72, potential: 76, tag: 'ORDINARY' },
    ]
  },
  {
    teamId: '314', teamName: 'Lyon Gaming', shortName: 'LYN', regionId: 'LCS',
    players: [
      { gameId: 'Josedeodo', name: 'Brandon Villegas', position: 'TOP', age: 23, ability: 71, potential: 78, tag: 'ORDINARY' },
      { gameId: 'Oddie', name: 'Oddie Player', position: 'JUG', age: 24, ability: 72, potential: 78, tag: 'ORDINARY' },
      { gameId: 'Seiya', name: 'Seiya Player', position: 'MID', age: 27, ability: 73, potential: 74, tag: 'ORDINARY' },
      { gameId: 'Whitelotus', name: 'Whitelotus Player', position: 'ADC', age: 26, ability: 71, potential: 74, tag: 'ORDINARY' },
      { gameId: 'Grell', name: 'Grell Player', position: 'SUP', age: 24, ability: 70, potential: 76, tag: 'ORDINARY' },
    ]
  },
]

// 导出所有选手数据
export const ALL_TEAM_PLAYERS: TeamPlayersConfig[] = [
  ...LPL_PLAYERS,
  ...LCK_PLAYERS,
  ...LEC_PLAYERS,
  ...LCS_PLAYERS,
]

// 根据战队ID获取选手配置
export const getTeamPlayersConfig = (teamId: string): TeamPlayersConfig | undefined => {
  return ALL_TEAM_PLAYERS.find(t => t.teamId === teamId)
}

// 根据赛区ID获取所有选手配置
export const getRegionPlayersConfig = (regionId: string): TeamPlayersConfig[] => {
  return ALL_TEAM_PLAYERS.filter(t => t.regionId === regionId)
}
