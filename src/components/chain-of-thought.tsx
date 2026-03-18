import { cn } from "@/lib/utils"
import { Badge } from "@/components/ui/badge"
import {
  Collapsible,
  CollapsibleTrigger,
  CollapsibleContent,
} from "@/components/ui/collapsible"
import type { LucideIcon } from "lucide-react"
import { BrainIcon, ChevronDownIcon, DotIcon } from "lucide-react"
import type { ComponentProps, ReactNode } from "react"
import { createContext, memo, useCallback, useContext, useState } from "react"

interface ChainOfThoughtContextValue {
  isOpen: boolean
  setIsOpen: (open: boolean) => void
}

const ChainOfThoughtContext = createContext<ChainOfThoughtContextValue | null>(
  null
)

const useChainOfThought = () => {
  const context = useContext(ChainOfThoughtContext)
  if (!context) {
    throw new Error(
      "ChainOfThought components must be used within ChainOfThought"
    )
  }
  return context
}

export type ChainOfThoughtProps = ComponentProps<"div"> & {
  open?: boolean
  defaultOpen?: boolean
  onOpenChange?: (open: boolean) => void
}

export const ChainOfThought = memo(
  ({
    className,
    open: controlledOpen,
    defaultOpen = false,
    onOpenChange,
    children,
    ...props
  }: ChainOfThoughtProps) => {
    const [internalOpen, setInternalOpen] = useState(defaultOpen)

    const isOpen = controlledOpen !== undefined ? controlledOpen : internalOpen
    const setIsOpen = useCallback(
      (val: boolean) => {
        if (controlledOpen === undefined) setInternalOpen(val)
        onOpenChange?.(val)
      },
      [controlledOpen, onOpenChange]
    )

    return (
      <ChainOfThoughtContext.Provider value={{ isOpen, setIsOpen }}>
        <Collapsible open={isOpen} onOpenChange={setIsOpen}>
          <div
            className={cn("not-prose w-full flex flex-col gap-4", className)}
            {...props}
          >
            {children}
          </div>
        </Collapsible>
      </ChainOfThoughtContext.Provider>
    )
  }
)

export type ChainOfThoughtHeaderProps = ComponentProps<"button"> & {
  label?: string
}

export const ChainOfThoughtHeader = memo(
  ({ className, children, label, ...props }: ChainOfThoughtHeaderProps) => {
    const { isOpen } = useChainOfThought()

    return (
      <CollapsibleTrigger
        className={cn(
          "flex w-full items-center gap-2 text-muted-foreground text-base transition-colors hover:text-foreground",
          className
        )}
        {...props}
      >
        <BrainIcon className="size-4" />
        <span className="flex-1 text-left">
          {children ?? label ?? "Chain of Thought"}
        </span>
        <ChevronDownIcon
          className={cn(
            "size-4 transition-transform",
            isOpen ? "rotate-180" : "rotate-0"
          )}
        />
      </CollapsibleTrigger>
    )
  }
)

export type ChainOfThoughtStepProps = ComponentProps<"div"> & {
  icon?: LucideIcon
  label: ReactNode
  description?: ReactNode
  status?: "complete" | "active" | "pending"
}

const stepStatusStyles = {
  active: "text-foreground",
  complete: "text-muted-foreground",
  pending: "text-muted-foreground/50",
}

export const ChainOfThoughtStep = memo(
  ({
    className,
    icon: Icon = DotIcon,
    label,
    description,
    status = "complete",
    children,
    ...props
  }: ChainOfThoughtStepProps) => (
    <div
      className={cn(
        "flex gap-2 text-base",
        stepStatusStyles[status],
        status !== "pending" &&
          "fade-in-0 slide-in-from-top-2 animate-in fill-mode-both",
        className
      )}
      {...props}
    >
      <div className="relative mt-0.5">
        <Icon className="size-4" />
        <div className="absolute top-7 bottom-0 left-1/2 -mx-px w-px bg-border" />
      </div>
      <div className="flex-1 flex flex-col gap-2 overflow-hidden">
        <div>{label}</div>
        {description && (
          <div className="text-muted-foreground text-base">{description}</div>
        )}
        {children}
      </div>
    </div>
  )
)

export type ChainOfThoughtSearchResultsProps = ComponentProps<"div">

export const ChainOfThoughtSearchResults = memo(
  ({ className, ...props }: ChainOfThoughtSearchResultsProps) => (
    <div
      className={cn("flex flex-wrap items-center gap-2", className)}
      {...props}
    />
  )
)

export type ChainOfThoughtSearchResultProps = ComponentProps<typeof Badge>

export const ChainOfThoughtSearchResult = memo(
  ({ className, children, ...props }: ChainOfThoughtSearchResultProps) => (
    <Badge
      className={cn(
        "h-9 max-w-xs gap-1 rounded-lg px-2.5 font-normal text-base",
        className,
      )}
      variant="secondary"
      title={typeof children === "string" ? children : undefined}
      {...props}
    >
      <span className="truncate">{children}</span>
    </Badge>
  )
)

export type ChainOfThoughtContentProps = ComponentProps<"div">

export const ChainOfThoughtContent = memo(
  ({ className, children, ...props }: ChainOfThoughtContentProps) => (
    <CollapsibleContent>
      <div className={cn("flex flex-col gap-3", className)} {...props}>
        {children}
      </div>
    </CollapsibleContent>
  )
)

ChainOfThought.displayName = "ChainOfThought"
ChainOfThoughtHeader.displayName = "ChainOfThoughtHeader"
ChainOfThoughtStep.displayName = "ChainOfThoughtStep"
ChainOfThoughtSearchResults.displayName = "ChainOfThoughtSearchResults"
ChainOfThoughtSearchResult.displayName = "ChainOfThoughtSearchResult"
ChainOfThoughtContent.displayName = "ChainOfThoughtContent"
