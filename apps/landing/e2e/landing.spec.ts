import { test, expect } from "@playwright/test";
import { LandingPage } from "./pages/LandingPage";

let landing: LandingPage;

test.beforeEach(async ({ page }) => {
  landing = new LandingPage(page);
  await landing.goto();
});

test.describe("Page basics", () => {
  test("has correct title", async ({ page }) => {
    await expect(page).toHaveTitle(/OGBank/);
  });

  test("has meta description", async ({ page }) => {
    const meta = page.locator('meta[name="description"]');
    await expect(meta).toHaveAttribute("content", /OGBank/);
  });
});

test.describe("Header", () => {
  test("renders logo and navigation", async () => {
    await expect(landing.header).toBeVisible();
    await expect(landing.header.getByText("OGBank")).toBeVisible();
  });

  test("has navigation links on desktop", async () => {
    const links = landing.navLinks;
    await expect(links).toHaveCount(6); // 5 nav + 1 CTA
  });

  test("mobile menu toggles", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 812 });
    await expect(landing.mobileMenuButton).toBeVisible();
    await landing.mobileMenuButton.click();
    await expect(
      page.getByRole("link", { name: "Problem", exact: true }).first(),
    ).toBeVisible();
  });
});

test.describe("Hero section", () => {
  test("shows headline", async () => {
    await expect(landing.heroHeadline).toContainText("Access DeFi. Keep your ZEC.");
  });

  test("shows subheadline", async ({ page }) => {
    await expect(
      page.getByText("Lock your ZCash. Get liquidity on Avalanche."),
    ).toBeVisible();
  });

  test("has CTA buttons", async () => {
    await expect(landing.heroCta).toBeVisible();
    await expect(
      landing.page.getByRole("link", { name: "Read the Docs" }).first(),
    ).toBeVisible();
  });
});

test.describe("Problem section", () => {
  test("renders lending position scene", async ({ page }) => {
    await expect(
      landing.sectionHeading("How DeFi Lending Works Today"),
    ).toBeVisible();
    await expect(
      page.getByText("Every position is fully transparent"),
    ).toBeVisible();
  });

  test("renders idle capital scene", async () => {
    await expect(
      landing.sectionHeading("5.1M ZEC. Zero DeFi Access."),
    ).toBeVisible();
  });
});

test.describe("Solution section", () => {
  test("shows 3 steps", async ({ page }) => {
    await expect(page.getByText("01_")).toBeVisible();
    await expect(page.getByText("02_")).toBeVisible();
    await expect(page.getByText("03_")).toBeVisible();
  });

  test("shows step titles", async () => {
    await expect(landing.sectionHeading("Lock")).toBeVisible();
    await expect(landing.sectionHeading("Unlock Liquidity")).toBeVisible();
    await expect(landing.sectionHeading("Return")).toBeVisible();
  });
});

test.describe("Market data section", () => {
  test("shows key stats", async ({ page }) => {
    const market = landing.section("market");
    await expect(market.getByText("5.1M")).toBeVisible();
    await expect(market.getByText("$0")).toBeVisible();
    await expect(market.getByText("100%")).toBeVisible();
    await expect(market.getByText("10-12K")).toBeVisible();
  });
});

test.describe("Team section", () => {
  test("shows both team members", async () => {
    await expect(landing.sectionHeading("Franco")).toBeVisible();
    await expect(landing.sectionHeading("Seba")).toBeVisible();
  });

  test("shows roles", async ({ page }) => {
    const cofounderTexts = page.getByText("Co-founder");
    await expect(cofounderTexts).toHaveCount(2);
  });
});

test.describe("Footer", () => {
  test("renders with links", async () => {
    await expect(landing.footer).toBeVisible();
    await expect(
      landing.footer.getByText("OGBank", { exact: true }),
    ).toBeVisible();
    await expect(
      landing.footer.getByRole("link", { name: "Documentation" }),
    ).toBeVisible();
  });
});

test.describe("Responsive", () => {
  test("renders properly on mobile", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 812 });
    await expect(landing.heroHeadline).toBeVisible();
    await expect(landing.footer).toBeVisible();
  });

  test("renders properly on tablet", async ({ page }) => {
    await page.setViewportSize({ width: 768, height: 1024 });
    await expect(landing.heroHeadline).toBeVisible();
  });
});
