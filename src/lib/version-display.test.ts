import { describe, expect, it } from "vitest";
import { formatSkillVersion, isKnownSkillVersion } from "./version-display";

describe("skill version display", () => {
  it("does not present an absent local version as v1.0.0", () => {
    expect(formatSkillVersion("unknown", "pl")).toBe("Nieznana wersja");
    expect(formatSkillVersion(undefined, "en")).toBe("Unknown version");
  });

  it("keeps explicitly declared SemVer values visible", () => {
    expect(isKnownSkillVersion("1.9.6")).toBe(true);
    expect(formatSkillVersion("1.9.6-codex.5", "en")).toBe("v1.9.6-codex.5");
  });

  it("does not turn a Git branch into a fake semantic version", () => {
    expect(formatSkillVersion("main", "en")).toBe("main");
    expect(formatSkillVersion("release/next", "pl")).toBe("release/next");
  });
});
