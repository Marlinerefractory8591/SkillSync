import type { SkillMetadata } from "../types/skillsync";

/**
 * A usable upstream remote is the source of truth for whether an item can be
 * checked for updates. Local branches, tags, and detached HEAD SHAs are refs,
 * not proof that tracking is missing; Git remotes and GitHub release sources
 * can all update a package without a checked-out branch.
 */
export const hasUpdateSource = (skill: SkillMetadata): boolean =>
  typeof skill.remoteUrl === "string" && skill.remoteUrl.trim().length > 0;
