import { beforeEach, describe, expect, it, vi } from "vitest";
import type { SkillMetadata } from "../types/skillsync";

vi.mock("../lib/ipc", () => ({
  api: {
    updateSingleSkill: vi.fn(),
    checkAppUpdate: vi.fn(),
    installAppUpdate: vi.fn(),
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
    vi.mocked(api.checkAppUpdate).mockReset();
    vi.mocked(api.installAppUpdate).mockReset();
    useSkillStore.setState({
      skills: [{ ...updateableSkill }],
      selectedSkill: { ...updateableSkill },
      error: null,
      batchUpdating: false,
      batchProgress: { total: 0, completed: 0 },
    });
  });

  it("stores an available application update for the persistent footer", async () => {
    vi.mocked(api.checkAppUpdate).mockResolvedValueOnce({
      currentVersion: "1.1.0",
      latestVersion: "1.1.1",
      updateAvailable: true,
      releaseUrl:
        "https://github.com/tomaszboloz/SkillSync/releases/tag/v1.1.1",
    });

    await useSkillStore.getState().checkAppUpdate();

    const state = useSkillStore.getState();
    expect(state.isCheckingAppUpdate).toBe(false);
    expect(state.appUpdate?.updateAvailable).toBe(true);
    expect(state.appUpdate?.latestVersion).toBe("1.1.1");
  });

  it("reports signed update download progress before restart", async () => {
    useSkillStore.setState({
      appUpdate: {
        currentVersion: "1.1.0",
        latestVersion: "1.1.1",
        updateAvailable: true,
        releaseUrl: "https://github.com/tomaszboloz/SkillSync/releases",
      },
    });
    vi.mocked(api.installAppUpdate).mockImplementationOnce(
      async (onProgress) => {
        onProgress({
          phase: "downloading",
          downloadedBytes: 50,
          contentLength: 100,
        });
        onProgress({
          phase: "installing",
          downloadedBytes: 100,
          contentLength: null,
        });
        onProgress({
          phase: "restarting",
          downloadedBytes: 100,
          contentLength: null,
        });
      },
    );

    await useSkillStore.getState().installAppUpdate();

    const state = useSkillStore.getState();
    expect(state.appUpdateProgress).toEqual({
      phase: "restarting",
      downloadedBytes: 100,
      contentLength: 100,
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
