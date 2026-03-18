import { LoaderIcon } from "lucide-react"

export function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  })
}

export function PageLoading({ text }: { text: string }) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-16">
      <LoaderIcon className="size-5 animate-spin text-muted-foreground" />
      <p className="text-base text-muted-foreground">{text}</p>
    </div>
  )
}

export function EmptyState({
  icon,
  title,
  description,
}: {
  icon: React.ReactNode
  title: string
  description: string
}) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-16 text-center">
      <div className="text-muted-foreground">{icon}</div>
      <div className="flex flex-col gap-1">
        <p className="text-base font-medium">{title}</p>
        <p className="text-base text-muted-foreground">{description}</p>
      </div>
    </div>
  )
}
