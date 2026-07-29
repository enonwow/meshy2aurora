import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: [
      "src/features/animation-studio/persistence.browser.test.ts",
      "src/features/project/persistence.browser.test.ts",
    ],
    fileParallelism: false,
    browser: {
      enabled: true,
      headless: true,
      provider: playwright({
        launchOptions: {
          channel: process.env.M2A_BROWSER_CHANNEL ?? "chrome",
          args: [
            "--disable-background-networking",
            "--disable-component-update",
            "--disable-default-apps",
            "--disable-sync",
          ],
        },
      }),
      instances: [{ browser: "chromium" }],
    },
  },
});
