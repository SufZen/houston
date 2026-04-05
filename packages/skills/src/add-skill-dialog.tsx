/**
 * AddSkillDialog — Modal for installing skills from skills.sh.
 * Contains the community browser (search + install) and a repo address input.
 */
import { useCallback, useState } from "react"
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  Input,
  Spinner,
} from "@deck-ui/core"
import { AlertCircle } from "lucide-react"
import type { CommunitySkill } from "./types"
import { CommunitySkillsSection } from "./community-skills-browser"

export interface AddSkillDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** Search skills.sh. */
  onSearch: (query: string) => Promise<CommunitySkill[]>
  /** Install a single community skill. Returns the installed skill name. */
  onInstallCommunity: (skill: CommunitySkill) => Promise<string>
  /** Install all skills from a repo address. Returns installed skill names. */
  onInstallFromRepo?: (source: string) => Promise<string[]>
}

export function AddSkillDialog({
  open,
  onOpenChange,
  onSearch,
  onInstallCommunity,
  onInstallFromRepo,
}: AddSkillDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg max-h-[80vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>Add skills</DialogTitle>
          <DialogDescription>
            Search and install skills from{" "}
            <span className="font-medium text-foreground">skills.sh</span>
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto -mx-6 px-6 space-y-6">
          <CommunitySkillsSection
            onSearch={onSearch}
            onInstall={onInstallCommunity}
          />

          {onInstallFromRepo && (
            <RepoInstall onInstall={onInstallFromRepo} />
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}

// ── Repo address install (secondary option) ───────────────────────

function RepoInstall({
  onInstall,
}: {
  onInstall: (source: string) => Promise<string[]>
}) {
  const [source, setSource] = useState("")
  const [installing, setInstalling] = useState(false)
  const [error, setError] = useState("")

  const handleInstall = useCallback(async () => {
    const trimmed = source.trim()
    if (!trimmed) return

    setInstalling(true)
    setError("")
    try {
      await onInstall(trimmed)
      setSource("")
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setInstalling(false)
    }
  }, [source, onInstall])

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" && !installing && source.trim()) {
        handleInstall()
      }
    },
    [handleInstall, installing, source],
  )

  return (
    <section className="space-y-2">
      <div>
        <h2 className="text-sm font-medium text-foreground">
          Install from repo
        </h2>
        <p className="text-xs text-muted-foreground mt-0.5">
          Enter a GitHub repo address to install all its skills
        </p>
      </div>

      <div className="flex gap-2">
        <Input
          value={source}
          onChange={(e) => setSource(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="owner/repo"
          disabled={installing}
          className="flex-1"
        />
        <Button
          onClick={handleInstall}
          disabled={!source.trim() || installing}
          className="rounded-full shrink-0"
        >
          {installing ? <Spinner className="size-4" /> : "Install"}
        </Button>
      </div>

      {error && (
        <p className="flex items-center gap-1.5 text-xs text-destructive">
          <AlertCircle className="size-3.5 shrink-0" />
          {error}
        </p>
      )}
    </section>
  )
}
