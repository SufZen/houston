import { useEffect, useCallback, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { AppSidebar, TabBar } from "@deck-ui/layout";
import { ChatPanel } from "@deck-ui/chat";
import type { FeedItem } from "@deck-ui/chat";
import {
  Empty,
  EmptyHeader,
  EmptyTitle,
  EmptyDescription,
} from "@deck-ui/core";
import { Bot } from "lucide-react";
import { useAgentStore } from "./stores/agents";
import { useFeedStore } from "./stores/feeds";
import { useSessionEvents } from "./hooks/use-session-events";
import { tauriSessions } from "./lib/tauri";
import { FilesView } from "./components/files-view";

type TabId = "chat" | "files";

const TABS: { id: TabId; label: string }[] = [
  { id: "chat", label: "Chat" },
  { id: "files", label: "Files" },
];

const MAIN_KEY = "main";

export function App() {
  const { agents, currentAgent, init, createAgent, deleteAgent, setCurrentAgent } =
    useAgentStore();
  const mainFeed = useFeedStore((s) => s.items[MAIN_KEY]);
  const pushFeedItem = useFeedStore((s) => s.pushFeedItem);
  const setFeed = useFeedStore((s) => s.setFeed);
  const [isLoading, setIsLoading] = useState(false);
  const [activeTab, setActiveTab] = useState<TabId>("chat");
  const [fileRefreshKey, setFileRefreshKey] = useState(0);
  const sendingRef = useRef(false);

  useSessionEvents();

  useEffect(() => {
    init();
  }, [init]);

  useEffect(() => {
    if (currentAgent) {
      tauriSessions.ensureWorkspace(currentAgent.id).catch(console.error);
      tauriSessions.loadFeed(currentAgent.id).then((rows) => {
        if (rows.length > 0) {
          setFeed(MAIN_KEY, rows as FeedItem[]);
        }
      });
    }
  }, [currentAgent, setFeed]);

  // Auto-refresh file list when a session completes
  useEffect(() => {
    const unlisten = listen<{ type: string; data: { status?: string } }>(
      "keel-event",
      (event) => {
        const payload = event.payload;
        if (payload.type === "SessionStatus" && payload.data.status === "completed") {
          setFileRefreshKey((k) => k + 1);
        }
      },
    );
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const handleSend = useCallback(
    async (text: string) => {
      if (!currentAgent || sendingRef.current) return;
      sendingRef.current = true;
      setIsLoading(true);

      pushFeedItem(MAIN_KEY, { feed_type: "user_message", data: text });

      try {
        await tauriSessions.start(currentAgent.id, text);
      } catch (err) {
        pushFeedItem(MAIN_KEY, {
          feed_type: "system_message",
          data: `Failed to start session: ${err}`,
        });
      } finally {
        setIsLoading(false);
        sendingRef.current = false;
      }
    },
    [currentAgent, pushFeedItem],
  );

  const handleAddAgent = useCallback(() => {
    const name = window.prompt("Agent name:");
    if (name?.trim()) {
      createAgent(name.trim());
    }
  }, [createAgent]);

  if (!currentAgent) {
    return (
      <div className="h-screen flex items-center justify-center bg-background text-foreground">
        <p className="text-muted-foreground text-sm">Starting...</p>
      </div>
    );
  }

  return (
    <div className="h-screen flex bg-background text-foreground">
      <AppSidebar
        logo={
          <div className="flex items-center gap-2">
            <div className="size-7 rounded-lg bg-primary flex items-center justify-center">
              <Bot className="size-4 text-primary-foreground" strokeWidth={2} />
            </div>
            <span className="text-sm font-semibold">{{APP_NAME_TITLE}}</span>
          </div>
        }
        items={agents.map((a) => ({ id: a.id, name: a.name }))}
        selectedId={currentAgent.id}
        onSelect={setCurrentAgent}
        onAdd={handleAddAgent}
        onDelete={deleteAgent}
        sectionLabel="Your agents"
      >
        <div className="flex-1 flex flex-col min-h-0">
          <TabBar
            title={currentAgent.name}
            tabs={TABS}
            activeTab={activeTab}
            onTabChange={(id) => setActiveTab(id as TabId)}
          />
          <div className="flex-1 min-h-0">
            {activeTab === "chat" && (
              <div className="h-full flex flex-col max-w-3xl mx-auto">
                <ChatPanel
                  sessionKey={MAIN_KEY}
                  feedItems={mainFeed ?? []}
                  isLoading={isLoading}
                  onSend={handleSend}
                  placeholder="Ask your agent anything..."
                  emptyState={
                    <Empty className="border-0">
                      <EmptyHeader>
                        <EmptyTitle>Start a conversation</EmptyTitle>
                        <EmptyDescription>
                          Type a message to talk to your agent.
                        </EmptyDescription>
                      </EmptyHeader>
                    </Empty>
                  }
                />
              </div>
            )}
            {activeTab === "files" && (
              <FilesView key={`files-${fileRefreshKey}`} />
            )}
          </div>
        </div>
      </AppSidebar>
    </div>
  );
}
