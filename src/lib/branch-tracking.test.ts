import { describe, expect, it } from "vitest";
import { hasUpdateSource } from "./branch-tracking";
import type { SkillMetadata } from "../types/skillsync";

const skill = (overrides: Partial<SkillMetadata> = {}): SkillMetadata => ({
  id: "skill-test",
  name: "test",
  description: "fixture",
  currentVersion: "1.0.0",
  latestVersion: null,
  author: "test",
  path: "/tmp/test",
  isGitRepo: true,
  remoteUrl: null,
  branchOrTag: null,
  agentScope: "global",
  status: "up_to_date",
  updateAvailable: false,
  changelog: null,
  dependencies: [],
  permissions: [],
  lastChecked: new Date(0).toISOString(),
  ...overrides,
});

describe("hasUpdateSource", () => {
  it("treats a remote-backed detached HEAD as trackable", () => {
    expect(
      hasUpdateSource(
        skill({
          remoteUrl: "https://github.com/example/repo",
          branchOrTag: "abc1234",
        }),
      ),
    ).toBe(true);
  });

  it("treats a GitHub release source without a local Git checkout as trackable", () => {
    expect(
      hasUpdateSource(
        skill({
          isGitRepo: false,
          remoteUrl: "https://github.com/example/repo",
        }),
      ),
    ).toBe(true);
  });

  it("includes items with no remote in the missing-source filter", () => {
    expect(hasUpdateSource(skill({ remoteUrl: "  " }))).toBe(false);
    expect(hasUpdateSource(skill({ remoteUrl: null }))).toBe(false);
  });
});
