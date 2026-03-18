import { useCallback, useEffect, useState } from "react"
import { useLexicalComposerContext } from "@lexical/react/LexicalComposerContext"
import {
  $getSelection,
  $isRangeSelection,
  FORMAT_TEXT_COMMAND,
  UNDO_COMMAND,
  REDO_COMMAND,
  COMMAND_PRIORITY_CRITICAL,
  SELECTION_CHANGE_COMMAND,
  $createParagraphNode,
  TextFormatType,
} from "lexical"
import { $setBlocksType } from "@lexical/selection"
import {
  $createHeadingNode,
  $createQuoteNode,
  HeadingTagType,
} from "@lexical/rich-text"
import {
  INSERT_ORDERED_LIST_COMMAND,
  INSERT_UNORDERED_LIST_COMMAND,
} from "@lexical/list"
import { $createCodeNode } from "@lexical/code"
import { INSERT_HORIZONTAL_RULE_COMMAND } from "@lexical/react/LexicalHorizontalRuleNode"
import {
  BoldIcon,
  ItalicIcon,
  UnderlineIcon,
  StrikethroughIcon,
  Heading1Icon,
  Heading2Icon,
  Heading3Icon,
  ListIcon,
  ListOrderedIcon,
  QuoteIcon,
  CodeIcon,
  CodeXmlIcon,
  MinusIcon,
  UndoIcon,
  RedoIcon,
  PilcrowIcon,
} from "lucide-react"
import { cn } from "@/lib/utils"

export function Toolbar() {
  const [editor] = useLexicalComposerContext()
  const [isBold, setIsBold] = useState(false)
  const [isItalic, setIsItalic] = useState(false)
  const [isUnderline, setIsUnderline] = useState(false)
  const [isStrikethrough, setIsStrikethrough] = useState(false)
  const [isCode, setIsCode] = useState(false)

  const updateToolbar = useCallback(() => {
    const selection = $getSelection()
    if ($isRangeSelection(selection)) {
      setIsBold(selection.hasFormat("bold"))
      setIsItalic(selection.hasFormat("italic"))
      setIsUnderline(selection.hasFormat("underline"))
      setIsStrikethrough(selection.hasFormat("strikethrough"))
      setIsCode(selection.hasFormat("code"))
    }
  }, [])

  useEffect(() => {
    return editor.registerCommand(
      SELECTION_CHANGE_COMMAND,
      () => {
        updateToolbar()
        return false
      },
      COMMAND_PRIORITY_CRITICAL
    )
  }, [editor, updateToolbar])

  useEffect(() => {
    return editor.registerUpdateListener(({ editorState }) => {
      editorState.read(() => {
        updateToolbar()
      })
    })
  }, [editor, updateToolbar])

  const formatText = (format: TextFormatType) => {
    editor.dispatchCommand(FORMAT_TEXT_COMMAND, format)
  }

  const formatHeading = (tag: HeadingTagType) => {
    editor.update(() => {
      const selection = $getSelection()
      if ($isRangeSelection(selection)) {
        $setBlocksType(selection, () => $createHeadingNode(tag))
      }
    })
  }

  const formatParagraph = () => {
    editor.update(() => {
      const selection = $getSelection()
      if ($isRangeSelection(selection)) {
        $setBlocksType(selection, () => $createParagraphNode())
      }
    })
  }

  const formatQuote = () => {
    editor.update(() => {
      const selection = $getSelection()
      if ($isRangeSelection(selection)) {
        $setBlocksType(selection, () => $createQuoteNode())
      }
    })
  }

  const formatCodeBlock = () => {
    editor.update(() => {
      const selection = $getSelection()
      if ($isRangeSelection(selection)) {
        $setBlocksType(selection, () => $createCodeNode())
      }
    })
  }

  return (
    <div className="flex flex-wrap items-center gap-0.5 border-b px-2 py-1">
      <ToolbarButton
        title="Undo"
        onClick={() => editor.dispatchCommand(UNDO_COMMAND, undefined)}
      >
        <UndoIcon />
      </ToolbarButton>
      <ToolbarButton
        title="Redo"
        onClick={() => editor.dispatchCommand(REDO_COMMAND, undefined)}
      >
        <RedoIcon />
      </ToolbarButton>

      <ToolbarSeparator />

      <ToolbarButton title="Paragraph" onClick={formatParagraph}>
        <PilcrowIcon />
      </ToolbarButton>
      <ToolbarButton title="Heading 1" onClick={() => formatHeading("h1")}>
        <Heading1Icon />
      </ToolbarButton>
      <ToolbarButton title="Heading 2" onClick={() => formatHeading("h2")}>
        <Heading2Icon />
      </ToolbarButton>
      <ToolbarButton title="Heading 3" onClick={() => formatHeading("h3")}>
        <Heading3Icon />
      </ToolbarButton>

      <ToolbarSeparator />

      <ToolbarButton
        title="Bold"
        active={isBold}
        onClick={() => formatText("bold")}
      >
        <BoldIcon />
      </ToolbarButton>
      <ToolbarButton
        title="Italic"
        active={isItalic}
        onClick={() => formatText("italic")}
      >
        <ItalicIcon />
      </ToolbarButton>
      <ToolbarButton
        title="Underline"
        active={isUnderline}
        onClick={() => formatText("underline")}
      >
        <UnderlineIcon />
      </ToolbarButton>
      <ToolbarButton
        title="Strikethrough"
        active={isStrikethrough}
        onClick={() => formatText("strikethrough")}
      >
        <StrikethroughIcon />
      </ToolbarButton>
      <ToolbarButton
        title="Inline Code"
        active={isCode}
        onClick={() => formatText("code")}
      >
        <CodeIcon />
      </ToolbarButton>

      <ToolbarSeparator />

      <ToolbarButton
        title="Bullet List"
        onClick={() =>
          editor.dispatchCommand(INSERT_UNORDERED_LIST_COMMAND, undefined)
        }
      >
        <ListIcon />
      </ToolbarButton>
      <ToolbarButton
        title="Numbered List"
        onClick={() =>
          editor.dispatchCommand(INSERT_ORDERED_LIST_COMMAND, undefined)
        }
      >
        <ListOrderedIcon />
      </ToolbarButton>

      <ToolbarSeparator />

      <ToolbarButton title="Quote" onClick={formatQuote}>
        <QuoteIcon />
      </ToolbarButton>
      <ToolbarButton title="Code Block" onClick={formatCodeBlock}>
        <CodeXmlIcon />
      </ToolbarButton>
      <ToolbarButton
        title="Horizontal Rule"
        onClick={() =>
          editor.dispatchCommand(INSERT_HORIZONTAL_RULE_COMMAND, undefined)
        }
      >
        <MinusIcon />
      </ToolbarButton>
    </div>
  )
}

function ToolbarButton({
  title,
  active,
  onClick,
  children,
}: {
  title: string
  active?: boolean
  onClick: () => void
  children: React.ReactNode
}) {
  return (
    <button
      type="button"
      title={title}
      onClick={onClick}
      className={cn(
        "inline-flex size-8 items-center justify-center rounded-md text-base",
        "transition-colors duration-150 ease-in-out hover:bg-accent hover:text-accent-foreground",
        "cursor-pointer [&_svg]:size-4",
        active && "bg-accent text-accent-foreground"
      )}
    >
      {children}
    </button>
  )
}

function ToolbarSeparator() {
  return <div className="mx-1 h-6 w-px shrink-0 bg-border" />
}
