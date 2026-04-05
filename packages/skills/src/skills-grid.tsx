/**
 * SkillsGrid — Full skills page with installed skills + community store.
 * Inspired by ChatGPT's Apps view. Props-driven.
 */
import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import { cn, Spinner } from "@deck-ui/core"
import { ChevronRight, Plus, Search, Loader2, Check } from "lucide-react"
import type { Skill, CommunitySkill } from "./types"
import { SkillRow } from "./skill-row"

export interface SkillsGridProps {
  skills: Skill[]
  loading: boolean
  onSkillClick: (skill: Skill) => void
  /** Search skills.sh. Enables the discover section. */
  onSearch?: (query: string) => Promise<CommunitySkill[]>
  /** Install a single community skill. Returns installed skill name. */
  onInstallCommunity?: (skill: CommunitySkill) => Promise<string>
  /** Install all skills from a repo address. Returns installed names. */
  onInstallFromRepo?: (source: string) => Promise<string[]>
}

export function SkillsGrid({
  skills,
  loading,
  onSkillClick,
  onSearch,
  onInstallCommunity,
  onInstallFromRepo,
}: SkillsGridProps) {
  const sorted = useMemo(() => {
    return [...skills].sort((a, b) => a.name.localeCompare(b.name))
  }, [skills])

  const hasStore = !!onSearch && !!onInstallCommunity

  if (loading && skills.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <Spinner className="size-5 text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="max-w-3xl mx-auto px-6 py-6 space-y-8">
        {/* Installed section */}
        {sorted.length > 0 && (
          <section className="space-y-3">
            <h2 className="text-sm font-medium text-foreground">Installed</h2>
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-2">
              {sorted.map((skill) => (
                <SkillRow
                  key={skill.id}
                  skill={skill}
                  onClick={() => onSkillClick(skill)}
                />
              ))}
            </div>
          </section>
        )}

        {/* Community store — always visible when callbacks provided */}
        {hasStore && (
          <SkillStore
            onSearch={onSearch}
            onInstall={onInstallCommunity}
            onInstallFromRepo={onInstallFromRepo}
            hasInstalledSkills={sorted.length > 0}
          />
        )}
      </div>
    </div>
  )
}

// ── Skill Store (inline community browser) ────────────────────────

function SkillStore({
  onSearch,
  onInstall,
  onInstallFromRepo,
  hasInstalledSkills,
}: {
  onSearch: (query: string) => Promise<CommunitySkill[]>
  onInstall: (skill: CommunitySkill) => Promise<string>
  onInstallFromRepo?: (source: string) => Promise<string[]>
  hasInstalledSkills: boolean
}) {
  const [query, setQuery] = useState("")
  const [results, setResults] = useState<CommunitySkill[]>([])
  const [loading, setLoading] = useState(true)
  const [installingIds, setInstallingIds] = useState<Set<string>>(new Set())
  const [installedIds, setInstalledIds] = useState<Set<string>>(new Set())
  const mountedRef = useRef(true)

  // Load featured on mount
  useEffect(() => {
    mountedRef.current = true
    doSearch("")
    return () => { mountedRef.current = false }
  }, [])

  // Debounced search on query change
  useEffect(() => {
    const timer = setTimeout(() => doSearch(query.trim()), 350)
    return () => clearTimeout(timer)
  }, [query])

  const doSearch = async (q: string) => {
    setLoading(true)
    try {
      const skills = await onSearch(q)
      if (mountedRef.current) setResults(skills)
    } catch {
      if (mountedRef.current) setResults([])
    } finally {
      if (mountedRef.current) setLoading(false)
    }
  }

  const handleInstall = useCallback(
    async (skill: CommunitySkill) => {
      setInstallingIds((prev) => new Set(prev).add(skill.id))
      try {
        await onInstall(skill)
        setInstalledIds((prev) => new Set(prev).add(skill.id))
      } finally {
        setInstallingIds((prev) => {
          const next = new Set(prev)
          next.delete(skill.id)
          return next
        })
      }
    },
    [onInstall],
  )

  return (
    <section className="space-y-5">
      {/* Header */}
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 className={cn(
            "font-semibold text-foreground",
            hasInstalledSkills ? "text-sm" : "text-xl",
          )}>
            {hasInstalledSkills ? "Discover" : "Skills"}
          </h2>
          <p className="text-sm text-muted-foreground mt-0.5">
            {hasInstalledSkills
              ? "Browse community skills on skills.sh"
              : "Teach your agent new abilities from the community"}
          </p>
        </div>
        <div className="relative shrink-0 w-56">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search skills"
            className="w-full h-9 pl-9 pr-3 rounded-full border border-border bg-background text-sm
                       placeholder:text-muted-foreground/60 focus:outline-none focus:ring-1 focus:ring-ring transition-colors"
          />
        </div>
      </div>

      {/* Results */}
      {loading && results.length === 0 && (
        <div className="flex justify-center py-8">
          <Spinner className="size-5 text-muted-foreground" />
        </div>
      )}

      {!loading && results.length === 0 && query.trim() && (
        <p className="text-sm text-muted-foreground py-4">
          No skills found for &ldquo;{query.trim()}&rdquo;
        </p>
      )}

      {results.length > 0 && (
        <div className="divide-y divide-border rounded-xl border border-border overflow-hidden">
          {results.map((skill) => (
            <StoreRow
              key={skill.id}
              skill={skill}
              installing={installingIds.has(skill.id)}
              installed={installedIds.has(skill.id)}
              onInstall={() => handleInstall(skill)}
            />
          ))}
        </div>
      )}

      {/* Repo install */}
      {onInstallFromRepo && (
        <RepoInstall onInstall={onInstallFromRepo} />
      )}
    </section>
  )
}

// ── Store row (ChatGPT-inspired) ──────────────────────────────────

function kebabToTitle(s: string): string {
  return s
    .split("-")
    .filter(Boolean)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ")
}

function formatInstalls(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(0)}K`
  return String(n)
}

function StoreRow({
  skill,
  installing,
  installed,
  onInstall,
}: {
  skill: CommunitySkill
  installing: boolean
  installed: boolean
  onInstall: () => void
}) {
  return (
    <div className="flex items-center gap-3 px-4 py-3 bg-background hover:bg-accent/50 transition-colors">
      {/* Icon placeholder */}
      <div className="shrink-0 size-10 rounded-xl bg-secondary flex items-center justify-center text-muted-foreground text-sm font-semibold">
        {kebabToTitle(skill.name).charAt(0)}
      </div>

      {/* Info */}
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium text-foreground truncate">
          {kebabToTitle(skill.name)}
        </p>
        <p className="text-xs text-muted-foreground truncate">
          {skill.source}
          {skill.installs > 0 && ` · ${formatInstalls(skill.installs)} installs`}
        </p>
      </div>

      {/* Install button */}
      <button
        onClick={onInstall}
        disabled={installing || installed}
        className={cn(
          "shrink-0 size-8 flex items-center justify-center rounded-full transition-colors",
          installed
            ? "text-muted-foreground cursor-default"
            : "text-muted-foreground hover:bg-secondary hover:text-foreground",
          installing && "opacity-50 cursor-wait",
        )}
      >
        {installing ? (
          <Loader2 className="size-4 animate-spin" />
        ) : installed ? (
          <Check className="size-4" />
        ) : (
          <Plus className="size-4" />
        )}
      </button>

      {/* Chevron */}
      <ChevronRight className="shrink-0 size-4 text-muted-foreground/50" />
    </div>
  )
}

// ── Repo address install ──────────────────────────────────────────

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

  return (
    <div className="pt-2">
      <p className="text-xs text-muted-foreground mb-2">
        Or install from a GitHub repo
      </p>
      <div className="flex gap-2">
        <input
          value={source}
          onChange={(e) => setSource(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && source.trim() && !installing) handleInstall()
          }}
          placeholder="owner/repo"
          disabled={installing}
          className="flex-1 h-9 px-3 rounded-full border border-border bg-background text-sm
                     placeholder:text-muted-foreground/60 focus:outline-none focus:ring-1 focus:ring-ring transition-colors"
        />
        <button
          onClick={handleInstall}
          disabled={!source.trim() || installing}
          className="h-9 px-4 rounded-full text-sm font-medium bg-primary text-primary-foreground
                     hover:bg-primary/90 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
        >
          {installing ? <Spinner className="size-4" /> : "Install"}
        </button>
      </div>
      {error && (
        <p className="text-xs text-destructive mt-1.5">{error}</p>
      )}
    </div>
  )
}
