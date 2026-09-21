import type { SkillMetadata } from "../types/skillsync";

/**
 * A tag name (or the abbreviated SHA reported for a detached HEAD) is not a
 * tracking branch. Only a branch discovered from Git or selected explicitly
 * by the user removes an item from the "Without branch" queue.
 */
export const hasTrackingBranch = (skill: SkillMetadata): boolean =>
  [skill.branchOverride, skill.detectedBranch].some(
    (branch) => typeof branch === "string" && branch.trim().length > 0,
  );
