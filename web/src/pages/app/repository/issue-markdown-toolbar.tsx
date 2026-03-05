import {
  Bold,
  Code2,
  Heading1,
  Heading2,
  ImagePlus,
  Italic,
  Link2,
  List,
  ListOrdered,
  MessageSquareQuote,
  Strikethrough,
} from "lucide-react";
import type { ComponentType } from "react";
import { Button } from "@/components/ui/button";

export type IssueEditorToolbarAction =
  | "bold"
  | "italic"
  | "strike"
  | "heading1"
  | "heading2"
  | "quote"
  | "bullet-list"
  | "ordered-list"
  | "inline-code"
  | "code-block"
  | "link"
  | "image"
  | "issue-ref";

interface IssueMarkdownToolbarProps {
  t: (text: string) => string;
  disabled?: boolean;
  onAction: (action: IssueEditorToolbarAction) => void;
}

interface ToolbarItem {
  action: IssueEditorToolbarAction;
  icon: ComponentType<{ className?: string }>;
  label: string;
}

const GROUPS: ToolbarItem[][] = [
  [
    { action: "bold", icon: Bold, label: "Bold" },
    { action: "italic", icon: Italic, label: "Italic" },
    { action: "strike", icon: Strikethrough, label: "Strikethrough" },
    { action: "inline-code", icon: Code2, label: "Inline code" },
  ],
  [
    { action: "heading1", icon: Heading1, label: "Heading 1" },
    { action: "heading2", icon: Heading2, label: "Heading 2" },
    { action: "quote", icon: MessageSquareQuote, label: "Quote" },
    { action: "code-block", icon: Code2, label: "Code block" },
  ],
  [
    { action: "bullet-list", icon: List, label: "Bullet list" },
    { action: "ordered-list", icon: ListOrdered, label: "Ordered list" },
    { action: "link", icon: Link2, label: "Link" },
    { action: "image", icon: ImagePlus, label: "Image" },
  ],
];

export function IssueMarkdownToolbar({
  t,
  disabled,
  onAction,
}: IssueMarkdownToolbarProps): JSX.Element {
  return (
    <div className="issue-editor-toolbar">
      {GROUPS.map((group, groupIndex) => (
        <div key={groupIndex} className="issue-editor-toolbar-group">
          {group.map((item) => (
            <Button
              key={item.action}
              type="button"
              size="icon"
              variant="ghost"
              className="h-8 w-8"
              title={t(item.label)}
              disabled={disabled}
              onMouseDown={(event) => event.preventDefault()}
              onClick={() => onAction(item.action)}
            >
              <item.icon className="size-4" />
              <span className="sr-only">{t(item.label)}</span>
            </Button>
          ))}
        </div>
      ))}
      <div className="issue-editor-toolbar-group">
        <Button
          type="button"
          size="sm"
          variant="ghost"
          className="h-8 px-2 text-xs"
          title={t("Insert issue reference")}
          disabled={disabled}
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => onAction("issue-ref")}
        >
          #123
        </Button>
      </div>
    </div>
  );
}
