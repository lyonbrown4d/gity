import { useCustom } from "@refinedev/core";
import type { GitignoreTemplate } from "@/lib/generated/model/gitignoreTemplate";
import type { LicenseTemplate } from "@/lib/generated/model/licenseTemplate";

export const useLicenseTemplates = () => {
  const query = useCustom<LicenseTemplate[]>({
    url: "/repository-templates/licenses",
    method: "get",
    meta: { auth: false },
    queryOptions: {
      queryKey: ["repository-templates", "licenses"],
      refetchOnWindowFocus: false,
      retry: false,
    },
  });

  return {
    templates: Array.isArray(query.result.data) ? query.result.data : [],
    query: query.query,
  };
};

export const useGitignoreTemplates = () => {
  const query = useCustom<GitignoreTemplate[]>({
    url: "/repository-templates/gitignores",
    method: "get",
    meta: { auth: false },
    queryOptions: {
      queryKey: ["repository-templates", "gitignores"],
      refetchOnWindowFocus: false,
      retry: false,
    },
  });

  return {
    templates: Array.isArray(query.result.data) ? query.result.data : [],
    query: query.query,
  };
};
