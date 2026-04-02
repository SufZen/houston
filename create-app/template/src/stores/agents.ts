import { create } from "zustand";
import { tauriProjects } from "../lib/tauri";
import type { Project } from "../lib/types";

const DEFAULT_AGENT_NAME = "My Agent";
const DOCUMENTS_BASE = "~/Documents/{{APP_NAME_TITLE}}";

interface AgentState {
  agents: Project[];
  currentAgent: Project | null;
  ready: boolean;
  init: () => Promise<void>;
  createAgent: (name: string) => Promise<Project>;
  deleteAgent: (id: string) => Promise<void>;
  setCurrentAgent: (id: string) => void;
}

export const useAgentStore = create<AgentState>((set, get) => ({
  agents: [],
  currentAgent: null,
  ready: false,

  init: async () => {
    const projects = await tauriProjects.list();

    if (projects.length === 0) {
      // First launch: create default agent (triggers BOOTSTRAP.md).
      const agent = await tauriProjects.create(
        DEFAULT_AGENT_NAME,
        `${DOCUMENTS_BASE}/${DEFAULT_AGENT_NAME}/`,
      );
      set({ agents: [agent], currentAgent: agent, ready: true });
      return;
    }

    // Use first project as current if none selected.
    set({ agents: projects, currentAgent: projects[0], ready: true });
  },

  createAgent: async (name: string) => {
    const folderPath = `${DOCUMENTS_BASE}/${name}/`;
    const agent = await tauriProjects.create(name, folderPath);
    set((s) => ({
      agents: [...s.agents, agent],
      currentAgent: agent,
    }));
    return agent;
  },

  deleteAgent: async (id: string) => {
    const { agents, currentAgent } = get();
    // Prevent deleting the last agent.
    if (agents.length <= 1) return;

    await tauriProjects.delete(id);
    const remaining = agents.filter((a) => a.id !== id);
    set({
      agents: remaining,
      currentAgent:
        currentAgent?.id === id ? remaining[0] ?? null : currentAgent,
    });
  },

  setCurrentAgent: (id: string) => {
    const agent = get().agents.find((a) => a.id === id) ?? null;
    set({ currentAgent: agent });
  },
}));
