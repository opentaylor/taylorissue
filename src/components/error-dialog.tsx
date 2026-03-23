import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { XCircleIcon } from "lucide-react"

interface ErrorDialogProps {
  open: boolean
  title: string
  message: string
  stepLabel?: string
  closeLabel?: string
  onClose: () => void
}

export function ErrorDialog({
  open,
  title,
  message,
  stepLabel,
  closeLabel = "Close",
  onClose,
}: ErrorDialogProps) {
  return (
    <Dialog open={open} onOpenChange={() => onClose()}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <XCircleIcon className="size-5 text-destructive" />
            {title}
          </DialogTitle>
        </DialogHeader>

        <div className="flex flex-col gap-2">
          {stepLabel && (
            <p className="text-base font-medium">{stepLabel}</p>
          )}
          <div className="max-h-60 overflow-y-auto rounded-md bg-muted p-3">
            <p className="font-mono text-sm leading-relaxed">{message}</p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" size="lg" onClick={onClose}>
            {closeLabel}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
