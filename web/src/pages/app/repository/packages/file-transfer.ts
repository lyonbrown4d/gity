import { normalizeString } from "../repository-normalizers";

export const readFileAsBase64 = (file: File): Promise<string> =>
  new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.addEventListener("load", () => {
      const result = normalizeString(reader.result);
      const [, content = ""] = result.split(",", 2);
      resolve(content);
    });
    reader.addEventListener("error", () => reject(reader.error ?? new Error("failed to read file")));
    reader.readAsDataURL(file);
  });

export const saveBase64File = (contentBase64: string, fileName: string, contentType: string) => {
  const binary = window.atob(contentBase64);
  const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0));
  const blob = new Blob([bytes], { type: contentType });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = fileName;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
};

export const formatBytes = (value: number): string => {
  if (value < 1024) {
    return `${value} B`;
  }
  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KiB`;
  }
  return `${(value / 1024 / 1024).toFixed(1)} MiB`;
};
