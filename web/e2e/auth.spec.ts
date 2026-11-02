import { expect, test } from "@playwright/test";
import { uniqueKey } from "./support/api";

test("signs in through the browser, opens the dashboard, and signs out", async ({ page }) => {
  const username = uniqueKey("ui-login");

  await page.goto("/login");
  await page.getByLabel("Username / Email").fill(username);
  await page.getByLabel("Password").fill("password");
  await page.getByRole("button", { name: "Sign in" }).click();

  await expect(page).toHaveURL(/\/app\/dashboard$/);
  await expect
    .poll(() => page.evaluate(() => Boolean(localStorage.getItem("gity.access_token"))))
    .toBe(true);

  await page.goto("/app/dashboard");
  await expect(page.getByRole("link", { name: "Open Projects" })).toBeVisible();

  await page.getByRole("button", { name: new RegExp(username) }).click();
  await page.getByRole("menuitem", { name: "Logout" }).click();

  await expect(page).toHaveURL(/\/login$/);
  await expect
    .poll(() => page.evaluate(() => Boolean(localStorage.getItem("gity.access_token"))))
    .toBe(false);
});
