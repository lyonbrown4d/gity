import { useEffect, useMemo, useState } from "react";
import { useCustom, useDataProvider } from "@refinedev/core";
import type { RepositoryPackageFileView } from "@/pages/types";
import { extractErrorMessage } from "../issues-utils";
import type { RawRecord as RawPackage } from "../repository-normalizers";
import { resolvePackageList, normalizePackage, normalizePackageDetail, normalizePackageFileContent } from "./model";
import { saveBase64File } from "./file-transfer";
import { buildPackageProtocolGuides } from "./protocol-guides";

export const usePackages = (repoId: string, t: (key: string) => string, onError: (message: string | null) => void) => {
  const dataProvider = useDataProvider();
  const packagesQuery = useCustom<RawPackage[]>({
    url: `/projects/${repoId}/packages`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId),
      refetchOnWindowFocus: false,
    },
  });
  const [selectedPackageId, setSelectedPackageId] = useState<string | null>(null);
  const packageDetailQuery = useCustom<RawPackage>({
    url: selectedPackageId ? `/projects/${repoId}/packages/${selectedPackageId}` : `/projects/${repoId}/packages/0`,
    method: "get",
    queryOptions: {
      enabled: Boolean(repoId && selectedPackageId),
      refetchOnWindowFocus: false,
    },
  });
  const [isDownloading, setDownloading] = useState(false);
  const [copiedCommand, setCopiedCommand] = useState<string | null>(null);

  const packages = useMemo(
    () => resolvePackageList(packagesQuery.result.data).map(normalizePackage),
    [packagesQuery.result.data],
  );
  const selectedPackage = useMemo(
    () => packages.find((item) => item.id === selectedPackageId) ?? packages[0] ?? null,
    [packages, selectedPackageId],
  );
  const packageDetail = useMemo(
    () => normalizePackageDetail(packageDetailQuery.result.data),
    [packageDetailQuery.result.data],
  );
  const totalFiles = packageDetail?.versions.reduce((total, item) => total + item.files.length, 0) ?? 0;
  const selectedVersion = packageDetail?.versions[0] ?? null;
  const selectedFile = selectedVersion?.files[0] ?? null;
  const protocolGuides = useMemo(
    () => buildPackageProtocolGuides(repoId, selectedPackage, selectedVersion, selectedFile),
    [repoId, selectedPackage, selectedVersion, selectedFile],
  );
  const isLoadingPackages = packagesQuery.query.isFetching && !packagesQuery.query.data;
  const isLoadingDetail = packageDetailQuery.query.isFetching && !packageDetailQuery.query.data;

  const loadPackages = async () => {
    const result = await packagesQuery.query.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
    return resolvePackageList(result.data?.data).map(normalizePackage);
  };

  const loadSelectedPackage = async () => {
    const result = await packageDetailQuery.query.refetch();
    if (result.error) {
      onError(extractErrorMessage(result.error));
      return;
    }
    onError(null);
  };

  const uploaded = async (identity: { name: string; type: string }) => {
    const refreshed = await loadPackages();
    const created = refreshed?.find((item) => item.name === identity.name && item.type === identity.type);
    if (created) {
      setSelectedPackageId(created.id);
      if (created.id === selectedPackageId) await loadSelectedPackage();
    }
  };

  const downloadFile = async (file: RepositoryPackageFileView) => {
    const custom = dataProvider().custom;
    if (!custom) {
      onError(t("Package file download is not available."));
      return;
    }
    onError(null);
    setDownloading(true);
    try {
      const response = await custom<RawPackage>({
        url: `/projects/${repoId}/packages/files/${file.id}`,
        method: "get",
      });
      const payload = normalizePackageFileContent(response.data);
      saveBase64File(
        payload.content_base64,
        payload.file.file_name || file.file_name,
        payload.file.content_type ?? file.content_type ?? "application/octet-stream",
      );
    } catch (error) {
      onError(extractErrorMessage(error));
    } finally {
      setDownloading(false);
    }
  };

  const copyCommand = async (command: string) => {
    try {
      await navigator.clipboard.writeText(command);
      setCopiedCommand(command);
      window.setTimeout(() => setCopiedCommand((current) => (current === command ? null : current)), 1400);
      onError(null);
    } catch {
      onError(t("Failed to copy command."));
    }
  };

  useEffect(() => {
    if (!repoId) {
      return;
    }
    onError(null);
  }, [repoId, onError]);

  useEffect(() => {
    if (!packagesQuery.query.error) {
      return;
    }
    onError(extractErrorMessage(packagesQuery.query.error));
  }, [packagesQuery.query.error, onError]);

  useEffect(() => {
    if (!packageDetailQuery.query.error) {
      return;
    }
    onError(extractErrorMessage(packageDetailQuery.query.error));
  }, [packageDetailQuery.query.error, onError]);

  useEffect(() => {
    if (packages.length === 0) {
      setSelectedPackageId(null);
      return;
    }
    if (!selectedPackageId || !packages.some((item) => item.id === selectedPackageId)) {
      setSelectedPackageId(packages[0].id);
    }
  }, [packages, selectedPackageId]);

  return {
    packages,
    selectedPackage,
    packageDetail,
    totalFiles,
    protocolGuides,
    isLoadingPackages,
    isLoadingDetail,
    isDownloading,
    copiedCommand,
    setSelectedPackageId,
    loadPackages,
    loadSelectedPackage,
    downloadFile,
    copyCommand,
    uploaded,
    listError: packagesQuery.query.error,
    detailError: packageDetailQuery.query.error,
  };
};
