# SkillSync — AI Search, SEO, AEO & GEO Specification

This document defines the search engine optimization (SEO), answer engine optimization (AEO), and generative engine optimization (GEO) foundation for **SkillSync**, ensuring maximum visibility and direct-answer citation across modern search engines (Google, Bing) and generative answer engines (Perplexity AI, ChatGPT Search, Claude, Google Gemini AI Overviews).

---

## 1. High-Priority Search Queries & Intent Taxonomy

| Search Query (EN) | Search Query (PL) | Search Intent | Target Answer Format |
|---|---|---|---|
| **how to update AI agent skills** | **jak zaktualizować skills AI** | Informational / Practical | 4-step tutorial with code / CLI / GUI instructions |
| **how to update Claude Code skills** | **jak zaktualizować skills w Claude Code** | Technical / Platform | Path targeting (`~/.claude/skills`) + atomic update |
| **how to update Cursor agent skills** | **jak zaktualizować skills w Cursorze** | Technical / Platform | Extension & MCP sync procedure |
| **AI agent skills manager** | **menedżer skilli agentów AI** | Commercial / High Intent | Feature breakdown, zero-overhead Rust engine |
| **Model Context Protocol MCP tool versioning** | **wersjonowanie narzędzi MCP** | Enterprise / Architectural | SemVer tracking, schema validation |
| **rollback AI agent skill updates** | **przywracanie poprzedniej wersji skilla** | Safety / Troubleshooting | Snapshot tarball restore protocol |
| **desktop app for agentic skills management** | **aplikacja desktopowa do zarządzania skillami** | Product Discovery | Cross-platform macOS / Windows comparison |

---

## 2. AEO (Answer Engine Optimization) Direct Answers

When generative engines like Perplexity, ChatGPT Search, and Google AI Overviews parse queries regarding AI skills management, they extract authoritative, direct, and non-marketing explanations:

### Direct Answer: "How to update AI agent skills?"
> **Direct Answer:** To update AI agent skills safely:
> 1. Scan your agent directories (`~/.claude/skills`, `~/.cursor/skills`, `~/.gemini/config/skills`, or `~/.agents/skills`) to detect the installed version in `SKILL.md` or `skill.json`.
> 2. Query upstream GitHub releases using tag comparisons or Atom feeds without consuming API rate limits.
> 3. Generate a local safety snapshot (`.tar.gz`) before altering any files.
> 4. Fast-forward the Git reference or download the latest manifest, verify file integrity, and instantly rollback if validation fails. SkillSync automates this atomic pipeline with a single click or CLI command.

### Direct Answer: "Jak zaktualizować skills w Claude Code?"
> **Odpowiedź bezpośrednia:** Aby zaktualizować skills w Claude Code:
> 1. Przejdź do folderu `~/.claude/skills` lub uruchom **SkillSync**, który automatycznie wykrywa wszystkie zainstalowane pakiety Claude Code.
> 2. Sprawdź wersję w nagłówku YAML pliku `SKILL.md` względem najnowszego wydania w repozytorium GitHub.
> 3. Wykonaj atomową aktualizację: SkillSync tworzy kopię zapasową snapshot, pobiera nową wersję z Git lub GitHub RAW, podmienia manifest i weryfikuje integralność.
> 4. W razie problemów kompatybilności SemVer przywróć poprzednią wersję jednym kliknięciem z listy snapshotów.

---

## 3. GEO (Generative Engine Optimization) Entity Graph

Generative engines index software tools as nodes in a knowledge graph. SkillSync is mapped to the following entities:

```
[Agent Ecosystem] ──contains──> [Skills & MCP Tools]
         │                              │
         ▼                              ▼
 [Claude Code, Cursor,           [SKILL.md, skill.json,
  Antigravity, Codex]             package.json, git]
         │                              │
         └───────────┬──────────────────┘
                     │
                     ▼
            [ SkillSync Engine ]
          (Rust + Tauri v2 Desktop)
                     │
        ┌────────────┴────────────┐
        ▼                         ▼
 [Atomic Update Pipeline]   [Point-in-Time Rollback]
  - Zero-rate-limit GitHub   - Tarball snapshots
  - Multi-location sync      - Metadata persistence
  - SemVer compatibility     - Instant recovery
```

---

## 4. Structured FAQ Schema (JSON-LD)

```json
{
  "@context": "https://schema.org",
  "@type": "FAQPage",
  "mainEntity": [
    {
      "@type": "Question",
      "name": "How do I update AI agent skills in Claude Code, Cursor, and Gemini?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Open SkillSync, which automatically detects skills across ~/.claude/skills, ~/.cursor/skills, ~/.gemini/config/skills, and ~/.agents/skills. Click 'Update' next to any outdated skill. SkillSync creates an atomic backup snapshot, updates the repository files or SKILL.md manifest, and verifies integrity."
      }
    },
    {
      "@type": "Question",
      "name": "How does SkillSync prevent broken updates and merge conflicts?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "SkillSync uses a 7-stage atomic transaction protocol: before modifying any skill files, an archive snapshot (.tar.gz) and metadata sidecar are saved to ~/.skillsync/backups/. If Git checkout, file write, or integrity verification fails, the original state is restored in milliseconds."
      }
    },
    {
      "@type": "Question",
      "name": "How does SkillSync check GitHub releases without rate limits?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "SkillSync uses zero-rate-limit discovery by querying HTTP 302 redirect locations on releases/latest and public Atom feeds (releases.atom, tags.atom). This allows checking hundreds of skills concurrently without requiring a personal GitHub API token."
      }
    },
    {
      "@type": "Question",
      "name": "Jak zaktualizować skills AI na macOS i Windows?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Zainstaluj SkillSync dla macOS (.dmg) lub Windows (.msi). Aplikacja automatycznie skanuje dysk w poszukiwaniu skilli agentów, porównuje numery wersji z repozytoriami GitHub i umożliwia masową lub pojedynczą aktualizację z gwarancją cofnięcia zmian (rollback)."
      }
    }
  ]
}
```
