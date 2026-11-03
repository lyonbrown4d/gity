import { useRef, useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useCustomMutation } from "@refinedev/core";
import { extractErrorMessage } from "../issues-utils";
import { suggestPackageMetadata } from "./upload-metadata";
import { readFileAsBase64 } from "./file-transfer";
import { packageUploadSchema, packageUploadDefaults, type PackageUploadFields } from "./upload-schema";

export const usePackageUpload = (
  repoId: string,
  t: (key: string) => string,
  onUploaded: (identity: { name: string; type: string }) => Promise<void>,
) => {
  const { mutateAsync } = useCustomMutation();
  const form = useForm<PackageUploadFields>({
    resolver: zodResolver(packageUploadSchema(t)),
    defaultValues: packageUploadDefaults,
  });
  const values = form.watch();
  const [open, setOpen] = useState(false);
  const [fileBase64, setFileBase64] = useState("");
  const [selectedFileName, setSelectedFileName] = useState("");
  const [fileSize, setFileSize] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [reading, setReading] = useState(false);
  const busy = useRef(false);
  const readGeneration = useRef(0);
  const close = () => {
    if (busy.current) return;
    readGeneration.current += 1;
    setOpen(false);
    form.reset(packageUploadDefaults);
    setFileBase64("");
    setSelectedFileName("");
    setFileSize(0);
    setError(null);
    setReading(false);
  };
  const selectFile = async (file: File | undefined) => {
    const generation = ++readGeneration.current;
    setFileBase64("");
    setSelectedFileName("");
    setReading(Boolean(file));
    setError(null);
    if (!file) return;
    try {
      const encoded = await readFileAsBase64(file);
      if (generation !== readGeneration.current) return;
      setFileBase64(encoded);
      setSelectedFileName(file.name);
      setFileSize(file.size);
      const suggested = suggestPackageMetadata(file.name);
      for (const key of ["name", "version", "type"] as const) {
        if (!form.getFieldState(key).isDirty)
          form.setValue(key, suggested[key] as PackageUploadFields[typeof key], {
            shouldValidate: form.formState.isSubmitted,
          });
      }
      form.setValue("file_name", file.name);
      form.setValue("content_type", file.type || "application/octet-stream");
    } catch (failure) {
      if (generation === readGeneration.current) setError(extractErrorMessage(failure));
    } finally {
      if (generation === readGeneration.current) setReading(false);
    }
  };
  const submit = form.handleSubmit(async (fields) => {
    if (busy.current || reading) return;
    if (!selectedFileName) {
      setError(t("Choose a file and confirm its package name and version."));
      return;
    }
    busy.current = true;
    setPending(true);
    setError(null);
    try {
      await mutateAsync({
        url: `/projects/${repoId}/packages/files`,
        method: "post",
        values: {
          type: fields.type,
          name: fields.name,
          version: fields.version,
          file_name: fields.file_name,
          file_path: fields.file_path.trim() || fields.file_name,
          content_type: fields.content_type,
          content_base64: fileBase64,
        },
      });
    } catch (failure) {
      setError(extractErrorMessage(failure));
      return;
    } finally {
      busy.current = false;
      setPending(false);
    }
    close();
    await onUploaded({ name: fields.name, type: fields.type });
  });
  return {
    open,
    setOpen,
    close,
    values,
    form,
    selectedFileName,
    fileSize,
    error,
    setError,
    pending,
    reading,
    selectFile,
    submit,
  };
};
