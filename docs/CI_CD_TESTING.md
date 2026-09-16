# Strategia CI/CD i Testowania Automatycznego: SkillSync

Dokument ten opisuje rygorystyczny proces zapewnienia jakości, strukturę testów automatycznych oraz konfigurację ciągłej integracji i wdrażania (CI/CD) dla aplikacji **SkillSync**.

---

## 1. Piramida Testów i Metodologia

Aby zagwarantować stabilność krytycznych operacji na systemie plików i operacji Git, system testowy zorganizowany jest w cztery warstwy:

```
                  ┌──────────────────────┐
                  │      E2E Tests       │   5% - Playwright + Tauri WebDriver
                  ├──────────────────────┤
                  │  Integration Tests   │  20% - Rust libgit2 + Mock GitHub
                  ├──────────────────────┤
                  │   UI Component Tests │  25% - Vitest + React Testing Library
                  ├──────────────────────┤
                  │   Unit Tests (Rust)  │  50% - Parsery, SemVer, Heurystyki
                  └──────────────────────┘
```

### 1.1. Testy Jednostkowe (Unit Tests - Rust & TypeScript)
- **Rust:** Testy parserów `skill.json`, nagłówków YAML w `SKILL.md`, funkcji porównywania SemVer, walidacji ścieżek i atomowości plików.
- **TypeScript:** Testy czystych funkcji formatujących, filtrów widoku Dashboard, middleware Immer w store Zustand.

### 1.2. Testy Integracyjne (Integration Tests)
- Tworzenie lokalnych, tymczasowych repozytoriów Git na dysku przy użyciu `tempfile::tempdir`.
- Symulacja operacji `fetch`, `checkout`, konfliktów merge, symulacja uszkodzonego manifestu i weryfikacja automatycznego rollbacku.
- Mockowanie odpowiedzi sieciowych GitHub API.

### 1.3. Testy Komponentów UI (Component Testing)
- Testowanie komponentów shadcn/ui przy użyciu Vitest i `@testing-library/react`.
- Snapshot testing dla stanów: pusta lista skilli, błąd ładowania, stan aktualizacji w toku, stan błędu.

### 1.4. Testy End-to-End (E2E)
- Wykorzystanie narzędzia `tauri-driver` w połączeniu z `Playwright`.
- Automatyczne uruchomienie skompilowanej binarki SkillSync w środowisku wirtualnym i weryfikacja kluczowych scenariuszy użytkownika:
  1. Wykrycie skilli w folderze testowym.
  2. Kliknięcie przycisku "Update All".
  3. Zmiana motywu w Settings na ciemny/jasny.
  4. Sprawdzenie powiadomienia systemowego.

### 1.5. Testy Wydajnościowe (Performance Benchmarks)
- **Skanowanie Dużej Skali:** Test skanowania 1,000 sztucznie wygenerowanych repozytoriów skilli — wymóg: czas skanowania $< 350\text{ ms}$.
- **Zużycie Pamięci:** Test obciążeniowy — pamięć RAM w stanie spoczynku $< 55\text{ MB}$, w trakcie masowej aktualizacji $< 120\text{ MB}$.

---

## 2. Wymagania Jakościowe dla Pull Requestów (Quality Gates)

Każda zmiana wprowadzana do gałęzi `main` musi bezwzględnie spełniać następujące kryteria:

- **100% Testów Passing:** Wszystkie testy jednostkowe, integracyjne i E2E muszą zakończyć się sukcesem.
- **Code Coverage > 80%:** Pokrycie kodu testami weryfikowane przez `tarpaulin` (dla Rusta) oraz `c8/vitest` (dla frontendu).
- **Zero Ostrzeżeń Linterów:**
  - Rust: `cargo clippy -- -D warnings`
  - Frontend: `eslint --max-warnings=0`
- **Formatowanie Kodu:** `cargo fmt --check` oraz `prettier --check`.
- **Audyt Bezpieczeństwa:** Brak znanych podatności w zależnościach (`cargo audit` oraz `npm audit --audit-level=high`).
- **Code Review:** Co najmniej jeden zatwierdzający review od doświadczonego inżyniera.

---

## 3. Matryca Pipeline'u CI/CD w GitHub Actions

Pipeline uruchamiany jest automatycznie dla każdego `push` do gałęzi `main` oraz dla każdego `pull_request`:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        GITHUB ACTIONS WORKFLOW                         │
├────────────────────────────────┬───────────────────────────────────────┤
│ Stage 1: Lint & Static Check   │ - Cargo fmt & Clippy                  │
│                                │ - ESLint & Prettier & TSC Check       │
├────────────────────────────────┼───────────────────────────────────────┤
│ Stage 2: Automated Tests       │ - Cargo test (Unit & Integration)     │
│                                │ - Vitest UI Component tests           │
├────────────────────────────────┼───────────────────────────────────────┤
│ Stage 3: Matrix Multi-Build    │ - macOS Runner: Universal .dmg        │
│                                │ - Windows Runner: x64 .msi & .exe     │
├────────────────────────────────┼───────────────────────────────────────┤
│ Stage 4: E2E Verification      │ - Playwright + Tauri WebDriver        │
├────────────────────────────────┼───────────────────────────────────────┤
│ Stage 5: Release & Publish     │ - Sign Binaries (Minisign)            │
│ (Tylko dla tagów v*.*.*)       │ - Generate latest.json updater spec   │
│                                │ - Create GitHub Release + Changelog   │
└────────────────────────────────┴───────────────────────────────────────┘
```
