import { useState, useEffect, useCallback } from "react";
import { Button } from "@deck-ui/core";
import { Trash2, Plus } from "lucide-react";
import { tauriLearnings } from "../lib/tauri";
import type { LearningsData } from "../lib/types";

interface LearningsTabProps {
  workspacePath: string;
}

export function LearningsTab({ workspacePath }: LearningsTabProps) {
  const [data, setData] = useState<LearningsData | null>(null);
  const [newText, setNewText] = useState("");

  const load = useCallback(async () => {
    try {
      const result = await tauriLearnings.load(workspacePath);
      setData(result);
    } catch (e) {
      console.error("[learnings] Failed to load:", e);
    }
  }, [workspacePath]);

  useEffect(() => {
    load();
  }, [load]);

  const handleAdd = async (text: string) => {
    if (!text.trim()) return;
    try {
      await tauriLearnings.add(workspacePath, text.trim());
      setNewText("");
      await load();
    } catch (e) {
      console.error("[learnings] Failed to add entry:", e);
    }
  };

  const handleRemove = async (index: number) => {
    try {
      await tauriLearnings.remove(workspacePath, index);
      await load();
    } catch (e) {
      console.error("[learnings] Failed to remove entry:", e);
    }
  };

  if (!data) {
    return (
      <div className="p-6 text-sm text-muted-foreground">Loading...</div>
    );
  }

  const percent = data.limit > 0 ? Math.round((data.chars / data.limit) * 100) : 0;

  return (
    <div className="max-w-3xl mx-auto p-6">
      <div className="flex items-baseline justify-between mb-1">
        <h3 className="text-sm font-medium">Learnings</h3>
        <span className="text-xs text-muted-foreground">
          {percent}% ({data.chars.toLocaleString()}/{data.limit.toLocaleString()} chars)
        </span>
      </div>
      <p className="text-xs text-muted-foreground mb-3">
        Everything the agent has learned: your preferences, environment facts, tool behaviors, conventions.
      </p>

      {data.entries.length === 0 ? (
        <p className="text-sm text-muted-foreground italic mb-3">
          No entries yet. The agent will save learnings here as it works.
        </p>
      ) : (
        <ul className="space-y-2 mb-3">
          {data.entries.map((entry) => (
            <li
              key={entry.index}
              className="flex items-start gap-2 text-sm"
            >
              <span className="flex-1 bg-secondary rounded-lg px-3 py-2">
                {entry.text}
              </span>
              <button
                onClick={() => handleRemove(entry.index)}
                className="shrink-0 mt-2 text-muted-foreground hover:text-foreground transition-colors"
                aria-label="Remove entry"
              >
                <Trash2 className="h-3.5 w-3.5" />
              </button>
            </li>
          ))}
        </ul>
      )}

      <div className="flex gap-2">
        <input
          type="text"
          value={newText}
          onChange={(e) => setNewText(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") handleAdd(newText);
          }}
          placeholder="Add a learning..."
          className="flex-1 rounded-full border border-border bg-background px-3 py-1.5 text-sm outline-none focus:ring-1 focus:ring-ring"
        />
        <Button
          size="sm"
          variant="outline"
          className="rounded-full"
          onClick={() => handleAdd(newText)}
          disabled={!newText.trim()}
        >
          <Plus className="h-4 w-4 mr-1" />
          Add
        </Button>
      </div>
    </div>
  );
}
