import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useOrganizationAdmin } from "./use-organization-admin";

const { useList, mutate } = vi.hoisted(() => ({ useList: vi.fn(), mutate: vi.fn() }));
vi.mock("@refinedev/core", () => ({
  useCreate: () => ({ mutate, mutation: { isPending: false } }),
  useDelete: () => ({ mutate: vi.fn(), mutation: { isPending: false } }),
  useList,
  useUpdate: () => ({ mutate: vi.fn(), mutation: { isPending: false } }),
}));
vi.mock("@/lib/i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

describe("organization admin member controls", () => {
  beforeEach(() => {
    mutate.mockReset();
    useList.mockImplementation(({ resource }: { resource: string }) => ({
      result: { data: resource === "organizations" ? [{ id: "org-1", key: "acme", name: "Acme" }] : [] },
      query: { error: null, isLoading: false, isFetching: false, refetch: vi.fn() },
    }));
  });

  it("loads all user pages only while the member dialog is open", async () => {
    const { result } = renderHook(() => useOrganizationAdmin());
    const userQuery = useList.mock.calls.find(([args]) => args.resource === "users")?.[0];
    expect(userQuery.pagination).toEqual({ mode: "off" });
    expect(userQuery.queryOptions.enabled).toBe(false);
    act(() => result.current.setMemberForm({ open: true, userId: "", role: "member" }));
    await waitFor(() => expect(useList.mock.calls[useList.mock.calls.length - 1]?.[0].queryOptions.enabled).toBe(true));
  });

  it("submits at most once when the add-member form fires twice before rerender", async () => {
    const { result } = renderHook(() => useOrganizationAdmin());
    await waitFor(() => expect(result.current.selectedOrg).toBe("org-1"));
    act(() => result.current.setMemberForm({ open: true, userId: "9223372036854775806", role: "owner" }));
    const event = { preventDefault: vi.fn() } as unknown as React.FormEvent<HTMLFormElement>;
    act(() => {
      result.current.submitMember(event);
      result.current.submitMember(event);
    });
    expect(mutate).toHaveBeenCalledTimes(1);
    expect(mutate.mock.calls[0][0].values).toEqual({
      organization_id: "org-1",
      user_id: "9223372036854775806",
      role: "owner",
    });
    await act(async () => {
      await mutate.mock.calls[0][1].onSuccess();
    });
    act(() => mutate.mock.calls[0][1].onSettled());
    expect(result.current.memberForm).toEqual({ open: false, userId: "", role: "member" });
  });

  it("validates a selected user before creating the membership", async () => {
    const { result } = renderHook(() => useOrganizationAdmin());
    await waitFor(() => expect(result.current.selectedOrg).toBe("org-1"));
    act(() => result.current.setMemberForm({ open: true, userId: "", role: "member" }));
    act(() => result.current.submitMember({ preventDefault: vi.fn() } as unknown as React.FormEvent<HTMLFormElement>));
    expect(mutate).not.toHaveBeenCalled();
    expect(result.current.memberError).toBe("Select user");
  });
});
