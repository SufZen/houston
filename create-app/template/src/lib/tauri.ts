import { invoke } from "@tauri-apps/api/core";
import type { Project, Issue } from "./types";
import type { Memory } from "@deck-ui/memory";

/** Type-safe wrappers around Tauri invoke() calls. */

export const tauriProjects = {
  list: () => invoke<Project[]>("list_projects"),
  create: (name: string, folderPath: string) =>
    invoke<Project>("create_project", { name, folderPath }),
  update: (id: string, name: string, folderPath: string) =>
    invoke<boolean>("update_project", { id, name, folderPath }),
  delete: (id: string) =>
    invoke<boolean>("delete_project", { id }),
};

export const tauriIssues = {
  list: (projectId: string) =>
    invoke<Issue[]>("list_issues", { projectId }),
  create: (projectId: string, title: string, description: string) =>
    invoke<Issue>("create_issue", { projectId, title, description }),
};

export const tauriSessions = {
  ensureWorkspace: (projectId: string) =>
    invoke<void>("ensure_workspace", { projectId }),
  start: (projectId: string, prompt: string) =>
    invoke<string>("start_session", { projectId, prompt }),
  loadFeed: (projectId: string) =>
    invoke<Array<{ feed_type: string; data: unknown }>>("load_chat_feed", { projectId }),
};

export const tauriMemory = {
  list: (projectId: string) =>
    invoke<Memory[]>("list_memories", { projectId }),
  create: (projectId: string, content: string, category: string, tags: string[]) =>
    invoke<Memory>("create_memory", { projectId, content, category, tags }),
  delete: (memoryId: string) =>
    invoke<void>("delete_memory", { memoryId }),
  search: (projectId: string, query: string) =>
    invoke<Memory[]>("search_memories", { projectId, query }),
};

export const tauriEvents = {
  list: (projectId: string) =>
    invoke<unknown[]>("list_events", { projectId }),
};

export const tauriScheduler = {
  addHeartbeat: (config: Record<string, unknown>) =>
    invoke<string>("add_heartbeat", { config }),
  removeHeartbeat: (id: string) =>
    invoke<void>("remove_heartbeat", { id }),
  addCron: (config: Record<string, unknown>) =>
    invoke<string>("add_cron", { config }),
  removeCron: (id: string) =>
    invoke<void>("remove_cron", { id }),
};

export interface WorkspaceFileInfo {
  name: string;
  description: string;
  exists: boolean;
}

export const tauriWorkspace = {
  listFiles: (projectId: string) =>
    invoke<WorkspaceFileInfo[]>("list_workspace_files", { projectId }),
  readFile: (projectId: string, fileName: string) =>
    invoke<string>("read_workspace_file", { projectId, fileName }),
};

export interface ProjectFile {
  path: string;
  name: string;
  extension: string;
  size: number;
}

export const tauriFiles = {
  list: (projectId: string) =>
    invoke<ProjectFile[]>("list_project_files", { projectId }),
  open: (projectId: string, relativePath: string) =>
    invoke<void>("open_file", { projectId, relativePath }),
  reveal: (projectId: string, relativePath: string) =>
    invoke<void>("reveal_file", { projectId, relativePath }),
  delete: (projectId: string, relativePath: string) =>
    invoke<void>("delete_file", { projectId, relativePath }),
  import: (projectId: string, filePaths: string[], targetFolder?: string) =>
    invoke<ProjectFile[]>("import_files", { projectId, filePaths, targetFolder }),
  createFolder: (projectId: string, folderName: string) =>
    invoke<string>("create_workspace_folder", { projectId, folderName }),
  revealWorkspace: (projectId: string) =>
    invoke<void>("reveal_workspace", { projectId }),
};

export const tauriChannels = {
  list: () => invoke<unknown[]>("list_channels"),
  connect: (channelId: string) =>
    invoke<void>("connect_channel", { channelId }),
  disconnect: (channelId: string) =>
    invoke<void>("disconnect_channel", { channelId }),
  addChannel: (channelType: string, name: string, config: Record<string, string>) =>
    invoke<unknown>("add_channel", { channelType, name, config }),
  removeChannel: (channelId: string) =>
    invoke<void>("remove_channel", { channelId }),
};
