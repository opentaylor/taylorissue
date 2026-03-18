export interface MissingRequirements {
  bins: string[]
  env: string[]
  config: string[]
  os: string[]
}

export interface InstallInstruction {
  id: string
  kind: string
  label: string
  bins: string[]
}

export interface Skill {
  name: string
  description: string
  emoji: string | null
  eligible: boolean
  disabled: boolean
  source: string
  bundled: boolean
  homepage: string | null
  missing: MissingRequirements
  install: InstallInstruction[]
}

export interface ClawHubSkill {
  slug: string
  name: string
  summary: string
  version: string | null
  updated_at: number | null
}

export interface ClawHubSkillDetail {
  slug: string
  name: string
  summary: string
  version: string | null
  license: string | null
  changelog: string | null
  downloads: number
  installs: number
  stars: number
  owner_handle: string | null
  owner_avatar: string | null
  created_at: number | null
  updated_at: number | null
}
