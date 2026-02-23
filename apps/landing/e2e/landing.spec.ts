import { test, expect } from "@playwright/test";
import { LandingPage } from "./pages/LandingPage";

let landing: LandingPage;

test.beforeEach(async ({ page }) => {
  landing = new LandingPage(page);
});

test.describe("Page basics", () => {
  test("has correct title", async ({ page }) => {
    await landing.goto();
    await expect(page).toHaveTitle(/OGBank/);
  });

  test("has meta description", async ({ page }) => {
    await landing.goto();
    const meta = page.locator('meta[name="description"]');
    await expect(meta).toHaveAttribute("content", /OGBank/);
  });
});

test.describe("Header", () => {
  test.beforeEach(async () => {
    await landing.goto();
  });

  test("renders logo and navigation", async () => {
    await expect(landing.header).toBeVisible();
    await expect(landing.header.getByText("OGBank")).toBeVisible();
  });

  test("has navigation links on desktop", async () => {
    const links = landing.navLinks;
    await expect(links).toHaveCount(4); // 3 nav + 1 CTA
  });

  test("mobile menu toggles", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 812 });
    await expect(landing.mobileMenuButton).toBeVisible();
    await landing.mobileMenuButton.click();
    await expect(
      page.getByRole("link", { name: "Home", exact: true }).first(),
    ).toBeVisible();
  });
});

test.describe("Home page (/)", () => {
  test.beforeEach(async () => {
    await landing.goto();
  });

  test("shows hero headline", async () => {
    await expect(landing.heroHeadline).toContainText("Access DeFi. Keep your ZEC.");
  });

  test("shows hero subheadline", async ({ page }) => {
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

  test("shows problem section", async () => {
    await expect(
      landing.sectionHeading("What Your ZEC Gives You"),
    ).toBeVisible();
    await expect(
      landing.sectionHeading("Lock ZEC. Get Stables. Access Everything."),
    ).toBeVisible();
  });

  test("shows solution steps", async () => {
    const solution = landing.section("solution");
    await expect(solution.getByText("01_").first()).toBeVisible();
    await expect(solution.getByText("02_").first()).toBeVisible();
    await expect(solution.getByText("03_").first()).toBeVisible();
  });

  test("shows solution step titles", async () => {
    const solution = landing.section("solution");
    await expect(solution.getByRole("heading", { name: "Lock", exact: true })).toBeVisible();
    await expect(solution.getByRole("heading", { name: "Unlock Liquidity" })).toBeVisible();
    await expect(solution.getByRole("heading", { name: "Return" })).toBeVisible();
  });

  test("ends with CTA", async () => {
    await expect(landing.section("cta")).toBeVisible();
  });
});

test.describe("Technology page (/technology)", () => {
  test.beforeEach(async () => {
    await landing.gotoTechnology();
  });

  test("shows how-it-works section", async () => {
    await expect(
      landing.sectionHeading("Zero-Knowledge. Full Access."),
    ).toBeVisible();
  });

  test("shows features section", async () => {
    await expect(
      landing.sectionHeading("Built on Proven Primitives"),
    ).toBeVisible();
  });

  test("ends with CTA", async () => {
    await expect(landing.section("cta")).toBeVisible();
  });
});

test.describe("Market page (/market)", () => {
  test.beforeEach(async () => {
    await landing.gotoMarket();
  });

  test("shows key stats", async () => {
    const market = landing.section("market");
    await market.scrollIntoViewIfNeeded();
    await expect(market.getByText("5.1M")).toBeVisible();
    await expect(market.getByText("$0")).toBeVisible();
    await expect(market.getByText("100%")).toBeVisible();
    await expect(market.getByText("10-12K")).toBeVisible();
  });

  test("shows trust section", async () => {
    await expect(
      landing.sectionHeading("Transparent About Trade-Offs"),
    ).toBeVisible();
  });

  test("ends with CTA", async () => {
    await expect(landing.section("cta")).toBeVisible();
  });
});

test.describe("Navigation", () => {
  test("navigates from home to technology", async ({ page }) => {
    await landing.goto();
    await page.getByRole("link", { name: "Technology" }).first().click();
    await expect(page).toHaveURL(/\/technology/);
    await expect(
      landing.sectionHeading("Zero-Knowledge. Full Access."),
    ).toBeVisible();
  });

  test("navigates from home to market", async ({ page }) => {
    await landing.goto();
    await page.getByRole("link", { name: "Market" }).first().click();
    await expect(page).toHaveURL(/\/market/);
  });
});

test.describe("404 page", () => {
  test("shows not found for unknown routes", async ({ page }) => {
    await landing.goto("/nonexistent");
    await expect(page.getByText("404")).toBeVisible();
    await expect(page.getByRole("link", { name: "Back to Home" })).toBeVisible();
  });
});

test.describe("Footer", () => {
  test("renders with links", async () => {
    await landing.goto();
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
    await landing.goto();
    await page.setViewportSize({ width: 375, height: 812 });
    await expect(landing.heroHeadline).toBeVisible();
    await expect(landing.footer).toBeVisible();
  });

  test("renders properly on tablet", async ({ page }) => {
    await landing.goto();
    await page.setViewportSize({ width: 768, height: 1024 });
    await expect(landing.heroHeadline).toBeVisible();
  });
});
