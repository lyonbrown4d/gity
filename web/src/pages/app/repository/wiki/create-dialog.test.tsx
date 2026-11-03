// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import "@testing-library/jest-dom/vitest";
import { WikiCreateDialog } from "./create-dialog";

const t = (key: string) => key;
afterEach(cleanup);

describe("wiki page creation dialog", () => {
  it("keeps failed submission details in the modal and retries with an optional slug", async () => {
    const onCreate = vi.fn<(title: string, slug: string, content: string) => Promise<string>>();
    let attempts = 0;
    onCreate.mockImplementation(async () => {
      attempts += 1;
      if (attempts === 1) throw new Error("Creation failed");
      return "guide";
    });
    render(<WikiCreateDialog canWrite busy={false} t={t} onCreate={onCreate} />);
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "New wiki page" }));
    await user.type(screen.getByLabelText("Wiki page title"), "Guide");
    await user.type(screen.getByLabelText("Content"), "# Welcome");
    await user.click(screen.getByRole("button", { name: "Create wiki page" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Creation failed");
    expect(screen.getByLabelText("Wiki page title")).toHaveValue("Guide");
    expect(screen.getByLabelText("Content")).toHaveValue("# Welcome");
    expect(onCreate).toHaveBeenNthCalledWith(1, "Guide", "", "# Welcome");
    await user.click(screen.getByRole("button", { name: "Create wiki page" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
    expect(onCreate).toHaveBeenCalledTimes(2);
  });

  it("keeps the custom slug behind advanced options and disables duplicate submits", async () => {
    let finish: (slug: string) => void = () => {};
    const onCreate = vi.fn(
      () =>
        new Promise<string>((resolve) => {
          finish = resolve;
        }),
    );
    render(<WikiCreateDialog canWrite busy={false} t={t} onCreate={onCreate} />);
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "New wiki page" }));
    expect(screen.getByLabelText("Custom URL slug")).not.toBeVisible();
    await user.click(screen.getByText("Advanced options"));
    expect(screen.getByLabelText("Custom URL slug")).toBeInTheDocument();
    await user.type(screen.getByLabelText("Wiki page title"), "Guide");
    await user.type(screen.getByLabelText("Custom URL slug"), "getting-started");
    await user.click(screen.getByRole("button", { name: "Create wiki page" }));
    expect(screen.getByRole("button", { name: "Creating wiki page..." })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
    expect(screen.getByLabelText("Wiki page title")).toBeDisabled();
    await user.keyboard("{Escape}");
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(onCreate).toHaveBeenCalledOnce();
    expect(onCreate).toHaveBeenCalledWith("Guide", "getting-started", "");
    finish("getting-started");
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  });

  it("preserves the draft when cancelled and reopened", async () => {
    const onCreate = vi.fn(async () => "guide");
    render(<WikiCreateDialog canWrite busy={false} t={t} onCreate={onCreate} />);
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "New wiki page" }));
    await user.type(screen.getByLabelText("Wiki page title"), "Guide");
    await user.type(screen.getByLabelText("Content"), "Draft contents");
    await user.click(screen.getByRole("button", { name: "Cancel" }));
    await user.click(screen.getByRole("button", { name: "New wiki page" }));
    expect(screen.getByLabelText("Wiki page title")).toHaveValue("Guide");
    expect(screen.getByLabelText("Content")).toHaveValue("Draft contents");
    expect(onCreate).not.toHaveBeenCalled();
  });
});
