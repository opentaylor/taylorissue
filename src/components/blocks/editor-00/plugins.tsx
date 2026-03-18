import { LexicalErrorBoundary } from "@lexical/react/LexicalErrorBoundary"
import { RichTextPlugin } from "@lexical/react/LexicalRichTextPlugin"
import { ListPlugin } from "@lexical/react/LexicalListPlugin"
import { HistoryPlugin } from "@lexical/react/LexicalHistoryPlugin"
import { MarkdownShortcutPlugin } from "@lexical/react/LexicalMarkdownShortcutPlugin"
import { HorizontalRulePlugin } from "@lexical/react/LexicalHorizontalRulePlugin"
import { TRANSFORMERS } from "@lexical/markdown"

import { ContentEditable } from "@/components/editor/editor-ui/content-editable"
import { Toolbar } from "@/components/editor/editor-ui/toolbar"

export function Plugins() {
  return (
    <div className="flex h-full flex-col">
      <Toolbar />
      <div className="relative min-h-0 flex-1 overflow-y-auto">
        <RichTextPlugin
          contentEditable={
            <div>
              <div>
                <ContentEditable placeholder={"Start typing ..."} />
              </div>
            </div>
          }
          ErrorBoundary={LexicalErrorBoundary}
        />
        <ListPlugin />
        <HistoryPlugin />
        <HorizontalRulePlugin />
        <MarkdownShortcutPlugin transformers={TRANSFORMERS} />
      </div>
    </div>
  )
}
