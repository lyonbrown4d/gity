import { z } from "zod";

export const packageUploadSchema = (t: (key: string) => string) =>
  z.object({
    type: z.enum(["generic", "npm", "maven", "pypi", "nuget"]),
    name: z.string().trim().min(1, t("Package name is required")),
    version: z.string().trim().min(1, t("Version is required")),
    file_name: z.string(),
    file_path: z.string(),
    content_type: z.string(),
  });
export type PackageUploadFields = z.infer<ReturnType<typeof packageUploadSchema>>;
export const packageUploadDefaults: PackageUploadFields = {
  type: "generic",
  name: "",
  version: "0.1.0",
  file_name: "",
  file_path: "",
  content_type: "application/octet-stream",
};
