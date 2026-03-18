import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import { Streamdown } from "streamdown"
import { code } from "@streamdown/code"
import "streamdown/styles.css"
import { cn } from "@/lib/utils"
import { Avatar, AvatarImage, AvatarFallback } from "@/components/ui/avatar"
import { Button } from "@/components/ui/button"
import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from "@/components/ui/resizable"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"
import { ArrowDownIcon, LoaderIcon, Trash2Icon } from "lucide-react"
import { useMessageStore } from "@/stores/message-store"
import type { AgentEntry, ChatMessage } from "@/types/chat"

const OPENCLAW_AVATAR = "/avatars/openclaw.png"

function ContactItem({
  agent,
  isSelected,
  lastMessage,
  onSelect,
}: {
  agent: AgentEntry
  isSelected: boolean
  lastMessage?: string
  onSelect: (id: string) => void
}) {
  const handleClick = useCallback(() => {
    onSelect(agent.id)
  }, [onSelect, agent.id])

  return (
    <button
      onClick={handleClick}
      className={cn(
        "flex w-full items-center gap-3 rounded-lg px-3 py-3 text-left transition-colors",
        isSelected
          ? "bg-secondary text-foreground"
          : "text-muted-foreground hover:bg-muted hover:text-foreground",
      )}
    >
      <Avatar size="lg" className="overflow-hidden">
        <AvatarImage
          src={OPENCLAW_AVATAR}
          alt={agent.name}
          className="scale-150 object-top"
        />
        <AvatarFallback>{agent.emoji || agent.name[0]}</AvatarFallback>
      </Avatar>
      <div className="min-w-0 flex-1">
        <div className="flex items-center justify-between">
          <span className="text-base font-medium text-foreground">
            {agent.name}
          </span>
        </div>
        <p className="truncate text-base text-muted-foreground">
          {lastMessage || agent.title}
        </p>
      </div>
    </button>
  )
}

function ChatBubble({
  message,
  isStreaming = false,
}: {
  message: ChatMessage
  isStreaming?: boolean
}) {
  const isUser = message.from === "user"

  return (
    <div
      className={cn("flex gap-3", isUser ? "flex-row-reverse" : "flex-row")}
    >
      {!isUser && (
        <Avatar className="mt-1 shrink-0 overflow-hidden">
          <AvatarImage
            src={OPENCLAW_AVATAR}
            className="scale-150 object-top"
          />
          <AvatarFallback>O</AvatarFallback>
        </Avatar>
      )}
      <div
        className={cn(
          "max-w-[75%] rounded-2xl px-4 py-2.5 text-base leading-relaxed",
          isUser
            ? "bg-primary text-primary-foreground"
            : "bg-secondary text-secondary-foreground",
        )}
      >
        {isUser ? (
          <p className="whitespace-pre-wrap break-words">{message.content}</p>
        ) : (
          <Streamdown plugins={{ code }} animated={isStreaming}>
            {message.content}
          </Streamdown>
        )}
      </div>
    </div>
  )
}

function TypingIndicator() {
  return (
    <div className="flex gap-3">
      <Avatar className="mt-1 shrink-0 overflow-hidden">
        <AvatarImage src={OPENCLAW_AVATAR} className="scale-150 object-top" />
        <AvatarFallback>O</AvatarFallback>
      </Avatar>
      <div className="flex items-center gap-1 rounded-2xl bg-secondary px-4 py-3">
        <span className="size-2 animate-bounce rounded-full bg-muted-foreground/60 [animation-delay:0ms]" />
        <span className="size-2 animate-bounce rounded-full bg-muted-foreground/60 [animation-delay:150ms]" />
        <span className="size-2 animate-bounce rounded-full bg-muted-foreground/60 [animation-delay:300ms]" />
      </div>
    </div>
  )
}

export default function MessagePage() {
  const { t } = useTranslation()
  const [inputText, setInputText] = useState("")
  const [showScrollButton, setShowScrollButton] = useState(false)

  const messagesEndRef = useRef<HTMLDivElement>(null)
  const messagesContainerRef = useRef<HTMLDivElement>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)
  const {
    agents,
    agentsLoading,
    selectedAgentId,
    conversations,
    isTyping,
    initialize,
    selectAgent,
    ensureConversation,
    sendMessage,
    clearAgentConversation,
  } = useMessageStore()

  const selectedAgent = useMemo(
    () => agents.find((a) => a.id === selectedAgentId) ?? null,
    [agents, selectedAgentId],
  )

  const currentMessages = useMemo(
    () => (selectedAgentId ? conversations[selectedAgentId] || [] : []),
    [conversations, selectedAgentId],
  )

  useEffect(() => {
    void initialize()
  }, [initialize])

  useEffect(() => {
    if (!selectedAgentId) return
    void ensureConversation(
      selectedAgentId,
      (agent) => t("page.message.greeting", { name: agent.name }),
    )
  }, [selectedAgentId, ensureConversation, t])

  const scrollToBottom = useCallback((behavior: ScrollBehavior = "smooth") => {
    messagesEndRef.current?.scrollIntoView({ behavior })
  }, [])

  const handleScroll = useCallback(() => {
    const container = messagesContainerRef.current
    if (!container) return
    const { scrollTop, scrollHeight, clientHeight } = container
    setShowScrollButton(scrollHeight - scrollTop - clientHeight > 100)
  }, [])

  useEffect(() => {
    scrollToBottom("instant")
  }, [selectedAgentId, scrollToBottom])

  useEffect(() => {
    if (currentMessages.length > 0) {
      scrollToBottom()
    }
  }, [currentMessages.length, scrollToBottom])

  const handleSend = useCallback(async () => {
    const text = inputText.trim()
    if (!text || isTyping || !selectedAgentId) return

    const agentId = selectedAgentId
    setInputText("")
    if (textareaRef.current) {
      textareaRef.current.value = ""
      textareaRef.current.style.height = "auto"
    }
    void sendMessage(agentId, text)
  }, [inputText, isTyping, selectedAgentId, sendMessage])

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) {
        e.preventDefault()
        handleSend()
      }
    },
    [handleSend],
  )

  const handleTextareaChange = useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      setInputText(e.target.value)
      const textarea = e.target
      textarea.style.height = "auto"
      textarea.style.height = `${Math.min(textarea.scrollHeight, 160)}px`
    },
    [],
  )

  const handleSelectAgent = useCallback((id: string) => {
    selectAgent(id)
  }, [selectAgent])

  const handleClearChat = useCallback(() => {
    if (!selectedAgentId || !selectedAgent) return
    void clearAgentConversation(
      selectedAgentId,
      (agent) => t("page.message.greeting", { name: agent.name }),
    )
  }, [selectedAgentId, selectedAgent, clearAgentConversation, t])

  const handleScrollToBottom = useCallback(() => {
    scrollToBottom()
  }, [scrollToBottom])

  return (
    <div className="flex h-[calc(100vh-var(--header-height))] overflow-hidden">
      <ResizablePanelGroup orientation="horizontal" className="flex-1">
        <ResizablePanel defaultSize="256px" minSize="160px" maxSize="400px">
          <div className="flex h-full flex-col">
            <div className="border-b px-4 py-3">
              <h2 className="text-base font-semibold">
                {t("page.message.title")}
              </h2>
              <p className="text-base text-muted-foreground">
                {t("page.message.description")}
              </p>
            </div>
            <div className="flex-1 overflow-y-auto p-2">
              {agentsLoading ? (
                <div className="flex items-center justify-center py-8">
                  <LoaderIcon className="size-5 animate-spin text-muted-foreground" />
                </div>
              ) : agents.length === 0 ? (
                <p className="px-3 py-4 text-base text-muted-foreground">
                  {t("page.message.noAgents")}
                </p>
              ) : (
                <div className="flex flex-col gap-0.5">
                  {agents.map((agent) => {
                    const msgs = conversations[agent.id]
                    const lastMsg = msgs?.[msgs.length - 1]
                    const preview = lastMsg
                      ? lastMsg.content.slice(0, 30) +
                        (lastMsg.content.length > 30 ? "..." : "")
                      : undefined
                    return (
                      <ContactItem
                        key={agent.id}
                        agent={agent}
                        isSelected={selectedAgentId === agent.id}
                        lastMessage={preview}
                        onSelect={handleSelectAgent}
                      />
                    )
                  })}
                </div>
              )}
            </div>
          </div>
        </ResizablePanel>
        <ResizableHandle />
        <ResizablePanel>
          {selectedAgent ? (
            <div className="flex h-full flex-col">
              <div className="flex items-center gap-3 border-b px-4 py-3">
                <Avatar className="overflow-hidden">
                  <AvatarImage
                    src={OPENCLAW_AVATAR}
                    alt={selectedAgent.name}
                    className="scale-150 object-top"
                  />
                  <AvatarFallback>
                    {selectedAgent.emoji || selectedAgent.name[0]}
                  </AvatarFallback>
                </Avatar>
                <div className="min-w-0 flex-1">
                  <h3 className="text-base font-semibold">
                    {selectedAgent.name}
                  </h3>
                  <p className="text-base text-muted-foreground">
                    {selectedAgent.title}
                  </p>
                </div>
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger
                      render={
                        <Button
                          variant="ghost"
                          size="icon"
                          className="shrink-0"
                          onClick={handleClearChat}
                        />
                      }
                    >
                      <Trash2Icon className="size-4" />
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>{t("page.message.clearChat")}</p>
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              </div>

              <div
                ref={messagesContainerRef}
                onScroll={handleScroll}
                className="relative flex-1 overflow-y-auto"
              >
                <div className="flex flex-col gap-4 p-4">
                  {currentMessages.map((message, index) => {
                    if (message.from === "assistant" && message.content === "") {
                      return null
                    }
                    return (
                      <ChatBubble
                        key={message.id}
                        message={message}
                        isStreaming={
                          isTyping &&
                          index === currentMessages.length - 1 &&
                          message.from === "assistant"
                        }
                      />
                    )
                  })}
                  {isTyping &&
                    currentMessages[currentMessages.length - 1]?.from ===
                      "assistant" &&
                    currentMessages[currentMessages.length - 1]?.content ===
                      "" && <TypingIndicator />}
                  <div ref={messagesEndRef} />
                </div>
                {showScrollButton && (
                  <Button
                    variant="outline"
                    size="icon"
                    className="absolute bottom-2 left-1/2 z-10 -translate-x-1/2 rounded-full shadow-md"
                    onClick={handleScrollToBottom}
                  >
                    <ArrowDownIcon className="size-4" />
                  </Button>
                )}
              </div>

              <div className="border-t p-4">
                <div className="flex items-center gap-2">
                  <textarea
                    ref={textareaRef}
                    value={inputText}
                    onChange={handleTextareaChange}
                    onKeyDown={handleKeyDown}
                    placeholder={t("page.message.inputPlaceholder", {
                      name: selectedAgent.name,
                    })}
                    rows={1}
                    className={cn(
                      "min-h-9 w-full flex-1 content-center resize-none rounded-lg border bg-background px-3 py-2 text-base",
                      "placeholder:text-muted-foreground",
                      "focus:border-ring focus:outline-none focus:ring-2 focus:ring-ring/50",
                      "disabled:cursor-not-allowed disabled:opacity-50",
                    )}
                    disabled={isTyping}
                  />
                  <Button
                    size="lg"
                    className="shrink-0"
                    onClick={handleSend}
                    disabled={!inputText.trim() || isTyping}
                  >
                    {t("page.message.send")}
                  </Button>
                </div>
              </div>
            </div>
          ) : (
            <div className="flex h-full items-center justify-center">
              <p className="text-base text-muted-foreground">
                {t("page.message.selectAgent")}
              </p>
            </div>
          )}
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  )
}
