import { tauriInvoke, getAppConfig } from "@/services/api/core/context"
import type { Skill, ClawHubSkill } from "@/types/skills"

export interface InstallResponse {
  ok: boolean
  outputs: string[]
}

export function fetchSkills() {
  return tauriInvoke<Skill[]>("list_skills", { config: getAppConfig() })
}

export function installSkill(name: string) {
  return tauriInvoke<InstallResponse>("install_skill", {
    name,
    config: getAppConfig(),
  })
}

export function uninstallSkill(name: string) {
  return tauriInvoke<{ ok: boolean }>("uninstall_skill", {
    name,
    config: getAppConfig(),
  })
}

export function searchClawHub(query: string) {
  return tauriInvoke<ClawHubSkill[]>("search_clawhub", { query })
}

export function installClawHubSkill(slug: string) {
  return tauriInvoke<InstallResponse>("install_clawhub_skill", {
    slug,
    config: getAppConfig(),
  })
}
