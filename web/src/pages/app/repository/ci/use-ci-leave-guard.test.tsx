// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { createMemoryRouter, RouterProvider, useNavigate } from "react-router";
import { useCILeaveGuard } from "./use-ci-leave-guard";
import { EditorLeaveDialog } from "./editor-leave-dialog";
afterEach(cleanup);
const Draft = ({ dirty, onDiscard }: { dirty: boolean; onDiscard: () => void }) => {
  const navigate = useNavigate();
  const guard = useCILeaveGuard(dirty, onDiscard);
  return (
    <>
      <button onClick={() => navigate("/away")}>Leave editor</button>
      <EditorLeaveDialog open={guard.open} onKeep={guard.keep} onDiscard={guard.discard} t={(key) => key} />
    </>
  );
};
const setup = (dirty: boolean, onDiscard = vi.fn()) => {
  const router = createMemoryRouter(
    [
      { path: "/editor", element: <Draft dirty={dirty} onDiscard={onDiscard} /> },
      { path: "/away", element: <p>Destination</p> },
    ],
    { initialEntries: ["/editor"] },
  );
  render(<RouterProvider router={router} />);
  return { router, onDiscard };
};
describe("CI editor SPA navigation guard", () => {
  it("retains the editor when keeping changes and navigates only after explicit discard", async () => {
    const { router, onDiscard } = setup(true);
    fireEvent.click(screen.getByText("Leave editor"));
    await screen.findByRole("dialog");
    expect(router.state.location.pathname).toBe("/editor");
    fireEvent.click(screen.getByRole("button", { name: "Keep editing" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(router.state.location.pathname).toBe("/editor");
    expect(onDiscard).not.toHaveBeenCalled();
    fireEvent.click(screen.getByText("Leave editor"));
    fireEvent.click(await screen.findByRole("button", { name: "Discard changes" }));
    await screen.findByText("Destination");
    expect(router.state.location.pathname).toBe("/away");
    expect(onDiscard).toHaveBeenCalledOnce();
  });
  it("navigates directly when the configuration has no unsaved changes", async () => {
    const { onDiscard } = setup(false);
    fireEvent.click(screen.getByText("Leave editor"));
    await screen.findByText("Destination");
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(onDiscard).not.toHaveBeenCalled();
  });
});
