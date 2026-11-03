// @vitest-environment jsdom
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, within, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import "@testing-library/jest-dom/vitest";
import { MergeConfirmation } from "./merge-confirmation";

afterEach(cleanup);
it("shows the actual branches, restores focus on cancellation and merges only on confirmation", async () => {
  const onMerge = vi.fn();
  const user = userEvent.setup();
  render(
    <MergeConfirmation
      source="feature/access"
      target="main"
      disabled={false}
      busy={false}
      reason={null}
      onMerge={onMerge}
      t={(key) => key}
    />,
  );
  const trigger = screen.getByRole("button", { name: "Merge" });
  await user.click(trigger);
  const dialog = screen.getByRole("dialog", { name: "Merge request" });
  expect(within(dialog).getByText("feature/access")).toBeInTheDocument();
  expect(within(dialog).getByText("main")).toBeInTheDocument();
  expect(onMerge).not.toHaveBeenCalled();
  await user.click(within(dialog).getByRole("button", { name: "Cancel" }));
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  await waitFor(() => expect(trigger).toHaveFocus());
  await user.click(trigger);
  await user.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Merge" }));
  expect(onMerge).toHaveBeenCalledOnce();
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
});
