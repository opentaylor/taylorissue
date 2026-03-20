import React from "react"
import ReactDOM from "react-dom/client"
import { ThemeProvider } from "next-themes"
import App from "./App"
import { TooltipProvider } from "@/components/ui/tooltip"
import { Toaster } from "@/components/ui/sonner"
import "@/i18n"
import "./index.css"

// React DevTools warning suppression in dev.
// React logs a console message when the global hook isn't present.
if (
  typeof window !== "undefined" &&
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (window as any).__REACT_DEVTOOLS_GLOBAL_HOOK__ === undefined
) {
  // Minimal stub to keep React quiet; real DevTools will still inject later.
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ;(window as any).__REACT_DEVTOOLS_GLOBAL_HOOK__ = { inject: () => {}, onCommitFiberRoot: () => {} }
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
      <TooltipProvider>
        <App />
        <Toaster />
      </TooltipProvider>
    </ThemeProvider>
  </React.StrictMode>,
)
