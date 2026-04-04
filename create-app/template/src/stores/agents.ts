import { create } from "zustand";
import { tauriAgents } from "../lib/tauri";
import type { Agent } from "../lib/types";

interface AgentState {
  agents: Agent[];
  current: Agent | null;
  ready: boolean;
  loadAgents: () => Promise<void>;
  createAgent: (name: string) => Promise<void>;
  setCurrentAgent: (agent: Agent) => void;
}

export const useAgentStore = create<AgentState>((set, get) => ({
  agents: [],
  current: null,
  ready: false,

  loadAgents: async () => {
    const agents = await tauriAgents.list();
    const current = get().current;
    const selected =
      agents.find((a) => a.path === current?.path) ?? agents[0] ?? null;
    set({ agents, current: selected, ready: true });
  },

  createAgent: async (name: string) => {
    const agent = await tauriAgents.create(name);
    set((s) => ({
      agents: [...s.agents, agent].sort((a, b) =>
        a.name.localeCompare(b.name),
      ),
      current: agent,
    }));
  },

  setCurrentAgent: (agent) => set({ current: agent }),
}));
