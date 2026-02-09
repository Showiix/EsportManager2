export interface StandingItem {
  rank: 1 | 2 | 3 | 4
  label: string
  name: string
  regionName?: string
  regionFlag?: string
  points: string
  pointsDetail?: string[]
}
