import { useState, type FormEvent } from "react";
import { useAgentStore } from "../stores/agents";

interface CreateAgentDialogProps {
  onClose: () => void;
}

export function CreateAgentDialog({ onClose }: CreateAgentDialogProps) {
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const createAgent = useAgentStore((s) => s.createAgent);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) return;
    try {
      await createAgent(trimmed);
      onClose();
    } catch (err) {
      setError(String(err));
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <form
        onSubmit={handleSubmit}
        className="bg-background rounded-2xl p-6 w-[360px] shadow-lg space-y-4"
      >
        <h2 className="text-lg font-semibold text-foreground">
          Create agent
        </h2>
        <input
          autoFocus
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="Agent name"
          className="w-full text-sm rounded-lg border border-border px-3 py-2 outline-none focus:ring-1 focus:ring-ring bg-background text-foreground"
        />
        {error && <p className="text-xs text-red-500">{error}</p>}
        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="text-sm px-3 h-9 rounded-full border border-black/15 hover:bg-accent transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={!name.trim()}
            className="text-sm px-3 h-9 rounded-full bg-primary text-primary-foreground hover:opacity-90 transition-colors disabled:opacity-40"
          >
            Create
          </button>
        </div>
      </form>
    </div>
  );
}
