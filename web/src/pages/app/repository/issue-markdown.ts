import DOMPurify from "dompurify";
import { marked } from "marked";

export const buildIssueDetailPath = (
  organizationId: string,
  repoId: string,
  issueNumber: number | string,
): string => {
  return `/app/projects/${organizationId}/${repoId}/issues/${issueNumber}`;
};

export const linkifyLocalIssueReferences = (
  markdown: string,
  organizationId: string,
  repoId: string,
): string => {
  if (!markdown.trim()) {
    return markdown;
  }

  return markdown.replace(/(^|[\s(])#(\d+)\b/g, (_match, prefix: string, rawNumber: string) => {
    const number = Number.parseInt(rawNumber, 10);
    if (!Number.isFinite(number) || number <= 0) {
      return `${prefix}#${rawNumber}`;
    }
    const path = buildIssueDetailPath(organizationId, repoId, number);
    return `${prefix}[#${number}](${path})`;
  });
};

export const renderIssueMarkdown = async (
  markdown: string,
  organizationId: string,
  repoId: string,
): Promise<string> => {
  const linked = linkifyLocalIssueReferences(markdown, organizationId, repoId);
  const html = await marked.parse(linked);
  return DOMPurify.sanitize(html);
};
