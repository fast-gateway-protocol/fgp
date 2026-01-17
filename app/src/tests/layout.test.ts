import { describe, expect, it } from "vitest";

import { ssr } from "../routes/+layout";

describe("layout config", () => {
  it("disables SSR for Tauri", () => {
    expect(ssr).toBe(false);
  });
});
