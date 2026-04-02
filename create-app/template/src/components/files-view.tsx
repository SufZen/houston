import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { FilesBrowser } from "@deck-ui/workspace";
import type { FileEntry } from "@deck-ui/workspace";
import { useAgentStore } from "../stores/agents";
import { tauriFiles } from "../lib/tauri";

export function FilesView() {
  const currentAgent = useAgentStore((s) => s.currentAgent);
  const [files, setFiles] = useState<FileEntry[]>([]);
  const [loading, setLoading] = useState(true);

  const load = useCallback(() => {
    if (!currentAgent) return;
    setLoading(true);
    tauriFiles
      .list(currentAgent.id)
      .then(setFiles)
      .catch(() => setFiles([]))
      .finally(() => setLoading(false));
  }, [currentAgent]);

  useEffect(load, [load]);

  // Tauri native drag-drop: catches files dropped from Finder
  useEffect(() => {
    const pid = currentAgent?.id;
    if (!pid) return;

    const unlisten = listen<{ paths: string[] }>(
      "tauri://drag-drop",
      (event) => {
        const paths = event.payload.paths;
        if (paths && paths.length > 0) {
          tauriFiles.import(pid, paths).then(load);
        }
      },
    );

    return () => { unlisten.then((fn) => fn()); };
  }, [currentAgent, load]);

  const pid = currentAgent?.id;
  if (!pid) return null;

  return (
    <FilesBrowser
      files={files}
      loading={loading}
      onOpen={(file) => tauriFiles.open(pid, file.path)}
      onReveal={(file) => tauriFiles.reveal(pid, file.path)}
      onDelete={(file) => {
        tauriFiles.delete(pid, file.path).then(load);
      }}
      emptyTitle="Your files show up here"
      emptyDescription="Drop files into this workspace or let your agent create them."
    />
  );
}
