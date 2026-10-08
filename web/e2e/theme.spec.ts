import { expect, test } from "@playwright/test";

test("switches and persists the workspace theme", async ({ page }) => {
  await page.goto("/login");
  await page.evaluate(() => {
    localStorage.setItem("gity.theme", "light");
  });
  await page.reload();

  const root = page.locator("html");
  await expect(root).toHaveAttribute("data-theme", "light");
  await expect(root).not.toHaveClass(/dark/);

  await page.getByRole("button", { name: "Switch theme" }).click();
  await expect(root).toHaveAttribute("data-theme", "dark");
  await expect(root).toHaveClass(/dark/);
  await expect.poll(() => page.evaluate(() => localStorage.getItem("gity.theme"))).toBe("dark");

  await page.reload();
  await expect(root).toHaveAttribute("data-theme", "dark");
  await expect(root).toHaveClass(/dark/);
});
