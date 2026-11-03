// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import "@testing-library/jest-dom/vitest";
import { PackageUploadDialog } from "./upload-dialog";

const { mutate } = vi.hoisted(() => ({ mutate: vi.fn() }));
vi.mock("@refinedev/core", () => ({ useCustomMutation: () => ({ mutateAsync: mutate }) }));
const t = (key: string) => key;
afterEach(cleanup);
beforeEach(() => {
  mutate.mockReset();
});
const setup = () => {
  const onUploaded = vi.fn(async () => {});
  render(<PackageUploadDialog repoId="42" canUpload t={t} onUploaded={onUploaded} />);
  return { user: userEvent.setup(), onUploaded };
};

describe("package upload interaction", () => {
  it("opens a modal, restores trigger focus on cancellation and clears the cancelled draft", async () => {
    const { user } = setup();
    const trigger = screen.getByRole("button", { name: "Upload package file" });
    expect(screen.queryByLabelText("Package name")).not.toBeInTheDocument();
    await user.click(trigger);
    const dialog = screen.getByRole("dialog", { name: "Upload package file" });
    await user.type(within(dialog).getByLabelText("Package name"), "cancelled");
    await user.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    await waitFor(() => expect(trigger).toHaveFocus());
    await user.click(trigger);
    expect(screen.getByLabelText("Package name")).toHaveValue("");
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(mutate).not.toHaveBeenCalled();
  });

  it("keeps an upload error and entered data inside the dialog, then retries successfully", async () => {
    mutate.mockRejectedValueOnce(new Error("Upload rejected")).mockResolvedValueOnce({ data: {} });
    const { user, onUploaded } = setup();
    await user.click(screen.getByRole("button", { name: "Upload package file" }));
    await user.type(screen.getByLabelText("Package name"), "release-notes");
    await user.upload(
      screen.getByLabelText("Package file"),
      new File(["发布说明"], "release-notes-2.3.4.txt", { type: "text/plain" }),
    );
    await waitFor(() => expect(screen.getByLabelText("Version")).toHaveValue("2.3.4"));
    await user.click(screen.getByRole("button", { name: "Upload file" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Upload rejected");
    expect(screen.getByLabelText("Package name")).toHaveValue("release-notes");
    expect(screen.getByText("release-notes-2.3.4.txt")).toBeInTheDocument();
    expect(onUploaded).not.toHaveBeenCalled();
    await user.click(screen.getByRole("button", { name: "Upload file" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
    expect(onUploaded).toHaveBeenCalledOnce();
    expect(mutate.mock.lastCall?.[0].values.name).toBe("release-notes");
  });

  it("reads a real file, prevents duplicate submit and dismissal while uploading, and refreshes after success", async () => {
    let finish: (value: unknown) => void = () => {};
    mutate.mockImplementation(
      () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
    );
    const { user, onUploaded } = setup();
    await user.click(screen.getByRole("button", { name: "Upload package file" }));
    await user.type(screen.getByLabelText("Package name"), "binary-artifact");
    await user.upload(
      screen.getByLabelText("Package file"),
      new File([new Uint8Array([0, 1, 254, 255])], "app.bin", { type: "application/octet-stream" }),
    );
    await waitFor(() => expect(screen.getByText("app.bin")).toBeInTheDocument());
    expect(screen.queryByLabelText("Text file content")).not.toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Upload file" }));
    expect(screen.getByRole("button", { name: "Uploading package file..." })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
    await user.keyboard("{Escape}");
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(mutate).toHaveBeenCalledOnce();
    expect(mutate.mock.calls[0][0].values).toMatchObject({
      file_name: "app.bin",
      content_base64: "AAH+/w==",
      content_type: "application/octet-stream",
    });
    finish({ data: {} });
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
    expect(onUploaded).toHaveBeenCalledOnce();
  });

  it("shows field errors and focuses missing package identity without sending a request", async () => {
    const { user } = setup();
    await user.click(screen.getByRole("button", { name: "Upload package file" }));
    await user.upload(screen.getByLabelText("Package file"), new File(["release"], "release-1.2.3.txt", { type: "text/plain" }));
    await waitFor(() => expect(screen.getByLabelText("Package name")).toHaveValue("release"));
    await user.clear(screen.getByLabelText("Package name"));
    await user.clear(screen.getByLabelText("Version"));
    await user.click(screen.getByRole("button", { name: "Upload file" }));
    expect(await screen.findByText("Package name is required")).toBeInTheDocument();
    expect(screen.getByText("Version is required")).toBeInTheDocument();
    await waitFor(() => expect(screen.getByLabelText("Package name")).toHaveFocus());
    expect(mutate).not.toHaveBeenCalled();
  });

  it("does not expose an upload action to a read-only member", () => {
    render(<PackageUploadDialog repoId="42" canUpload={false} t={t} onUploaded={vi.fn()} />);
    expect(screen.getByRole("button", { name: "Upload package file" })).toBeDisabled();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
});
