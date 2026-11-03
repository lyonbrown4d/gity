import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { OrganizationMemberUserPicker } from "./organization-member-user-picker";
import type { UserView } from "@/pages/types";

class ResizeObserverMock {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}
globalThis.ResizeObserver = ResizeObserverMock as unknown as typeof ResizeObserver;
Element.prototype.scrollIntoView = vi.fn();

const users: UserView[] = [
  {
    id: "9223372036854775806",
    username: "alice",
    display_name: "Alice Example",
    email: "alice@example.test",
    status: "active",
    is_super_admin: false,
  },
  { id: "12", username: "bob", email: "bob@example.test", status: "active", is_super_admin: false },
];

describe("organization member user picker", () => {
  it("searches human labels and returns the selected string ID", async () => {
    const onChange = vi.fn();
    render(
      <OrganizationMemberUserPicker users={users} value="" loading={false} t={(key) => key} onChange={onChange} />,
    );
    fireEvent.click(screen.getByRole("combobox", { name: "User" }));
    fireEvent.change(screen.getByPlaceholderText("Search members"), { target: { value: "alice@example" } });
    fireEvent.click(await screen.findByText("Alice Example"));
    await waitFor(() => expect(onChange).toHaveBeenCalledWith("9223372036854775806"));
  });
});
