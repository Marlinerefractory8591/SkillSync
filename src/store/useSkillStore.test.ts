import { beforeEach, describe, expect, it, vi } from "vitest";
import type { SkillMetadata } from "../types/skillsync";

vi.mock("../lib/ipc", () => ({
  api: {
    updateSingleSkill: vi.fn(),
    scanSkills: vi.fn(),
    checkGitHubUpdate: vi.fn(),
    setBranchOverride: vi.fn(),
    setRepositoryOverride: vi.fn(),
    removeSkill: vi.fn(),
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
    vi.mocked(api.scanSkills).mockReset();
    vi.mocked(api.checkGitHubUpdate).mockReset();
    vi.mocked(api.setBranchOverride).mockReset();
    vi.mocked(api.setRepositoryOverride).mockReset();
    vi.mocked(api.removeSkill).mockReset();
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

  it("refreshes the active skill and checks GitHub immediately after saving a repository", async () => {
    const tracked = {
      ...updateableSkill,
      remoteUrl: "https://github.com/PrefectHQ/fastmcp",
      status: "checking" as const,
    };
    const checked = {
      ...tracked,
      latestVersion: "2.0.0",
      status: "up_to_date" as const,
    };
    vi.mocked(api.scanSkills).mockResolvedValueOnce([tracked]);
    vi.mocked(api.checkGitHubUpdate).mockResolvedValueOnce(checked);

    await useSkillStore
      .getState()
      .setRepositoryOverride(updateableSkill.id, tracked.remoteUrl);

    expect(api.setRepositoryOverride).toHaveBeenCalledWith(
      updateableSkill.id,
      tracked.remoteUrl,
    );
    expect(api.checkGitHubUpdate).toHaveBeenCalledWith(updateableSkill.id);
    expect(useSkillStore.getState().selectedSkill).toEqual(checked);
  });

  it("updates the manual branch in state before the remote check completes", async () => {
    vi.mocked(api.checkGitHubUpdate).mockImplementation(
      () => new Promise(() => undefined),
    );

    void useSkillStore.getState().setBranchOverride(updateableSkill.id, "main");

    await vi.waitFor(() => {
      expect(api.setBranchOverride).toHaveBeenCalledWith(
        updateableSkill.id,
        "main",
      );
    });
    expect(useSkillStore.getState().skills[0].branchOverride).toBe("main");
  });

  it("removes only the chosen location and refreshes the discovered item", async () => {
    const multiLocationSkill = {
      ...updateableSkill,
      installedLocations: ["/tmp/claude/fixture", "/tmp/codex/fixture"],
    };
    useSkillStore.setState({
      skills: [multiLocationSkill],
      selectedSkill: multiLocationSkill,
    });
    vi.mocked(api.removeSkill).mockResolvedValueOnce(["/tmp/claude/fixture"]);
    vi.mocked(api.scanSkills).mockResolvedValueOnce([
      {
        ...multiLocationSkill,
        installedLocations: ["/tmp/codex/fixture"],
      },
    ]);

    await useSkillStore
      .getState()
      .removeSkill(multiLocationSkill.id, ["/tmp/claude/fixture"]);

    expect(api.removeSkill).toHaveBeenCalledWith(multiLocationSkill.id, [
      "/tmp/claude/fixture",
    ]);
    expect(useSkillStore.getState().skills[0].installedLocations).toEqual([
      "/tmp/codex/fixture",
    ]);
  });
});
