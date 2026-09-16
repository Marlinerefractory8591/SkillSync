import { beforeEach, describe, expect, it, vi } from "vitest";
import type { SkillMetadata } from "../types/skillsync";

vi.mock("../lib/ipc", () => ({
  api: {
    updateSingleSkill: vi.fn(),
  },
}));

import { api } from "../lib/ipc";
import { useSkillStore } from "./useSkillStore";

const updateableSkill: SkillMetadata = {
  id: "skill-ui-ux-pro-max",
  name: "ui-ux-pro-max",
  description: "Test fixture",
  currentVersion: "2.11.0",
  latestVersion: "2.12.0",
  author: "NextLevelBuilder",
  path: "~/.agents/skills/ui-ux-pro-max-skill",
  isGitRepo: true,
  remoteUrl: "https://github.com/nextlevelbuilder/ui-ux-pro-max-skill",
  branchOrTag: "main",
  agentScope: "global",
  status: "update_available",
  updateAvailable: true,
  changelog: null,
  dependencies: [],
  permissions: [],
  lastChecked: new Date().toISOString(),
};

describe("batchUpdateAll", () => {
  beforeEach(() => {
    vi.mocked(api.updateSingleSkill).mockReset();
    useSkillStore.setState({
      skills: [{ ...updateableSkill }],
      selectedSkill: { ...updateableSkill },
      error: null,
      batchUpdating: false,
      batchProgress: { total: 0, completed: 0 },
    });
  });

  it("clears the updating state and exposes a failed batch update", async () => {
    vi.mocked(api.updateSingleSkill).mockRejectedValueOnce(
      new Error(
        "Katalog roboczy zawiera niezacommitowane zmiany w śledzonych plikach",
      ),
    );

    await useSkillStore.getState().batchUpdateAll();

    const state = useSkillStore.getState();
    expect(state.batchUpdating).toBe(false);
    expect(state.skills[0].status).toBe("error");
    expect(state.skills[0].updateAvailable).toBe(true);
    expect(state.selectedSkill?.status).toBe("error");
    expect(state.error).toContain("niezacommitowane zmiany");
    expect(state.pendingDirtyUpdate?.skillId).toBe("skill-ui-ux-pro-max");
  });
});
