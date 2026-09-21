import { describe, expect, it } from "vitest";
import { hasTrackingBranch } from "./branch-tracking";
import type { SkillMetadata } from "../types/skillsync";

const skill = (overrides: Partial<SkillMetadata>): SkillMetadata => ({
  id: "skill-fixture",
  name: "fixture",
  description: "Fixture",
  currentVersion: "unknown",
  latestVersion: null,
  author: "Fixture",
  path: "/tmp/fixture",
  isGitRepo: true,
  remoteUrl: "https://github.com/example/fixture",
  branchOrTag: null,
  agentScope: "global",
  status: "up_to_date",
  updateAvailable: false,
  changelog: null,
  dependencies: [],
  permissions: [],
  lastChecked: new Date().toISOString(),
  ...overrides,
});

describe("hasTrackingBranch", () => {
  it("does not mistake a detached HEAD SHA for a branch", () => {
    expect(hasTrackingBranch(skill({ branchOrTag: "c0ffee1" }))).toBe(false);
  });

  it("accepts a saved branch override immediately", () => {
    expect(hasTrackingBranch(skill({ branchOverride: "main" }))).toBe(true);
  });

  it("accepts a branch found in the local Git worktree", () => {
    expect(hasTrackingBranch(skill({ detectedBranch: "develop" }))).toBe(true);
  });
});
