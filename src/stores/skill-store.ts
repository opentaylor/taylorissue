import { create } from "zustand"
import type { Skill, ClawHubSkill } from "@/types/skills"
import * as api from "@/services/api/skill"
interface SkillState {
  skills: Skill[]
  hubResults: ClawHubSkill[]
  skillsLoading: boolean
  hubLoading: boolean
  error: string | null
  installingSkills: Set<string>
  uninstallingSkills: Set<string>
  installingHubSkills: Set<string>

  fetchSkills: () => Promise<void>
  searchHub: (query: string) => Promise<void>
  installSkill: (name: string) => Promise<api.InstallResponse | null>
  uninstallSkill: (name: string) => Promise<void>
  installHubSkill: (slug: string) => Promise<api.InstallResponse | null>
}

export const useSkillStore = create<SkillState>((set) => ({
  skills: [],
  hubResults: [],
  skillsLoading: true,
  hubLoading: false,
  error: null,
  installingSkills: new Set<string>(),
  uninstallingSkills: new Set<string>(),
  installingHubSkills: new Set<string>(),

  fetchSkills: async () => {
    set({ skillsLoading: true, error: null })
    try {
      const skills = await api.fetchSkills()
      set({ skills, skillsLoading: false })
    } catch {
      set({ skillsLoading: false, error: "Failed to load skills" })
    }
  },

  searchHub: async (query) => {
    set({ hubLoading: true })
    try {
      const hubResults = await api.searchClawHub(query)
      set({ hubResults, hubLoading: false })
    } catch {
      set({ hubResults: [], hubLoading: false })
    }
  },

  installSkill: async (name) => {
    set((s) => ({ installingSkills: new Set(s.installingSkills).add(name) }))
    try {
      const result = await api.installSkill(name)
      const skills = await api.fetchSkills()
      set((s) => {
        const next = new Set(s.installingSkills)
        next.delete(name)
        return { skills, installingSkills: next }
      })
      return result
    } catch {
      set((s) => {
        const next = new Set(s.installingSkills)
        next.delete(name)
        return { installingSkills: next }
      })
      return null
    }
  },

  uninstallSkill: async (name) => {
    set((s) => ({ uninstallingSkills: new Set(s.uninstallingSkills).add(name) }))
    try {
      await api.uninstallSkill(name)
      set((s) => {
        const next = new Set(s.uninstallingSkills)
        next.delete(name)
        return {
          skills: s.skills.filter((sk) => sk.name !== name),
          uninstallingSkills: next,
        }
      })
    } catch {
      set((s) => {
        const next = new Set(s.uninstallingSkills)
        next.delete(name)
        return { uninstallingSkills: next }
      })
    }
  },

  installHubSkill: async (slug) => {
    set((s) => ({
      installingHubSkills: new Set(s.installingHubSkills).add(slug),
    }))
    try {
      const result = await api.installClawHubSkill(slug)
      const skills = await api.fetchSkills()
      set((s) => {
        const next = new Set(s.installingHubSkills)
        next.delete(slug)
        return { skills, installingHubSkills: next }
      })
      return result
    } catch {
      set((s) => {
        const next = new Set(s.installingHubSkills)
        next.delete(slug)
        return { installingHubSkills: next }
      })
      return null
    }
  },
}))
