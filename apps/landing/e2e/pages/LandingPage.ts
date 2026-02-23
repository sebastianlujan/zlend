import { type Page, type Locator } from "@playwright/test";

export class LandingPage {
  readonly page: Page;
  readonly header: Locator;
  readonly heroHeadline: Locator;
  readonly heroCta: Locator;
  readonly navLinks: Locator;
  readonly mobileMenuButton: Locator;
  readonly footer: Locator;

  constructor(page: Page) {
    this.page = page;
    this.header = page.locator("header");
    this.heroHeadline = page.getByRole("heading", { level: 1 });
    this.heroCta = page.getByRole("link", { name: "Go to App" });
    this.navLinks = page.getByRole("navigation", { name: "Main" }).getByRole("link");
    this.mobileMenuButton = page.getByRole("button", { name: "Toggle menu" });
    this.footer = page.locator("footer");
  }

  async goto(path: string = "/") {
    await this.page.goto(path);
  }

  async gotoTechnology() {
    await this.page.goto("/technology");
  }

section(id: string) {
    return this.page.locator(`#${id}`);
  }

  sectionHeading(text: string) {
    return this.page.getByRole("heading", { name: text });
  }
}
