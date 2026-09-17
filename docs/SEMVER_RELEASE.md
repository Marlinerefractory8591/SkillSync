# System Zarządzania Wersjami i Tagami (SemVer 2.0.0): SkillSync

Niniejszy dokument precyzuje zasady wersjonowania, strukturę tagów Git, reguły Conventional Commits oraz procedurę tworzenia oficjalnych wydań dla projektu **SkillSync**.

---

## 1. Zgodność z Semantic Versioning (SemVer 2.0.0)

Wszystkie wydania aplikacji SkillSync, powiązane biblioteki oraz formaty manifestów podlegają rygorystycznym regułom **SemVer 2.0.0**:

$$\mathbf{v\langle MAJOR\rangle.\langle MINOR\rangle.\langle PATCH\rangle[-prerelease][+build]}$$

### 1.1. Kryteria Zmiany Wersji

- **MAJOR (1.0.0 ➔ 2.0.0):**
  - Niekompatybilne zmiany w schemacie manifestu `skill.json` lub `SKILL.md`.
  - Modyfikacja sygnatur komend IPC zrywające kompatybilność wsteczną.
  - Zmiana struktury bazy danych / plików konfiguracyjnych wymagająca migracji bez opcji automatycznego fallbacku.
  - Usunięcie wsparcia dla wersji systemu operacyjnego (np. wycofanie starszych wersji macOS/Windows).
- **MINOR (1.1.0 ➔ 1.2.0):**
  - Dodanie nowego silnika autodetekcji (np. wsparcie dla nowego frameworka agentowego).
  - Wprowadzenie nowej funkcjonalności (np. masowe aktualizacje, harmonogram w tle, eksport do pliku lock).
  - Nowe widoki w UI lub nowe opcje w konfiguracji.
  - Znaczące optymalizacje wydajnościowe (np. przyspieszenie skanowania o 50%).
- **PATCH (1.2.0 ➔ 1.2.1):**
  - Poprawki błędów (bug fixes) w operacjach Git lub systemie powiadomień.
  - Drobne poprawki estetyczne i tekstowe w UI.
  - Aktualizacje zależności rozwiązujące wykryte podatności bezpieczeństwa.

---

## 2. Standard Commitów: Conventional Commits 1.0.0

Wszystkie commity trafiające do gałęzi `main` muszą być sformatowane zgodnie ze specyfikacją Conventional Commits:

$$\mathbf{\langle type\rangle(\langle scope\rangle): \langle description\rangle}$$

### Dopuszczalne Typy:

- `feat`: Nowa funkcjonalność dla użytkownika końcowego.
- `fix`: Naprawa błędu w istniejącej funkcjonalności.
- `perf`: Zmiana kodu poprawiająca wydajność (czas wykonania, pamięć RAM).
- `docs`: Zmiany w dokumentacji (README, docs, komentarze kodu).
- `refactor`: Zmiana w kodzie nienaprawiająca błędu ani niedodająca funkcjonalności.
- `test`: Dodanie lub zmiana testów automatycznych.
- `chore`: Zmiany w procesie budowania, narzędziach CI lub zależnościach.

### Przykłady Prawidłowych Commitów:

```bash
feat(orchestrator): zaimplementowano buforowanie operacji w kolejce tokio
fix(git): rozwiązano problem stanu detached HEAD przy przełączaniu tagów
perf(detector): przyspieszono skanowanie katalogów o 40% dzięki walkdir
docs(readme): rozszerzono sekcję FAQ o rozwiązywanie problemów z uprawnieniami
chore(tauri): zaktualizowano tauri do wersji 2.1.0
```

---

## 3. Format Tagu Git (Annotated Tag)

Tagi wydań muszą być tworzone jako **Annotated Tags** z prefiksem `v`:

```bash
git tag -a v1.2.0 -m "v1.2.0 - Masowe aktualizacje i powiadomienia"
```

### Stabilne i testowe tagi

- Stabilne wydanie ma format `vMAJOR.MINOR.PATCH`, np. `v1.0.0`.
- Wydanie przedpremierowe ma format `vMAJOR.MINOR.PATCH-rc.N`, `-beta.N` lub `-codex.N`, np. `v1.0.0-rc.1`.
- Zgodnie z SemVer wersja stabilna `1.9.6` jest **nowsza** od `1.9.6-codex.5`. SkillSync nie oferuje aktualizacji ze stabilnego wydania do odpowiadającego mu prerelease.
- Jeżeli lokalny manifest nie podaje wersji, aplikacja pokazuje „Nieznana wersja” i nie wyświetla automatycznej propozycji aktualizacji na podstawie samej różnicy tekstu.

### Przykładowy Opis Tagu Git:

```markdown
v1.2.0 - Masowe aktualizacje i powiadomienia

## Nowe funkcje

- [Feature] Masowa aktualizacja wszystkich skills jednym kliknięciem (skrót Cmd/Ctrl+U)
- [Feature] System powiadomień w systemach macOS i Windows o dostępnych aktualizacjach
- [Feature] Harmonogram automatycznych sprawdzeń w tle (co 1h, 6h, 24h)

## Poprawki błędów

- [Fix] Rozwiązano błąd timeoutu przy aktualizacji repozytoriów z wolnym łączem
- [Fix] Wyeliminowano crash podczas skanowania folderów z uprawnieniami tylko do odczytu
- [Fix] Poprawiono kontrast wskaźników statusu w trybie ciemnym

## Zmiany wewnętrzne & Wydajność

- [Perf] Wielowątkowe skanowanie dysku — 40% szybsza indeksacja dla 500+ skills
- [Chore] Aktualizacja silnika Tauri do wersji v2.1.0 i biblioteki git2-rs do v0.19.0

Breaking changes: Brak
Wymagana migracja konfiguracji: Nie
```

---

## 4. Automatyzacja Publikacji i Dystrybucja Aktualizacji (Tauri Updater)

W momencie wypchnięcia tagu do repozytorium (`git push origin v1.2.0`), pipeline CI/CD realizuje następujące kroki:

1. Kompilacja natywnych binarzy dla macOS i Windows (`x86_64-pc-windows-msvc`).
2. Podpisanie pakietów aktualizacji `.app.tar.gz` i `.msi` przy użyciu klucza prywatnego Minisign.
3. Wygenerowanie pliku metadanych aktualizatora `latest.json`:

```json
{
  "version": "v1.2.0",
  "notes": "Masowe aktualizacje i powiadomienia. Zobacz pełny CHANGELOG.md.",
  "pub_date": "2026-09-16T10:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHNraWxsc3luYyBzZWNyZXQga2V5C...",
      "url": "https://github.com/tomaszboloz/SkillSync/releases/download/v1.2.0/SkillSync.app.tar.gz"
    },
    "windows-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHNraWxsc3luYyBzZWNyZXQga2V5C...",
      "url": "https://github.com/tomaszboloz/SkillSync/releases/download/v1.2.0/SkillSync_x64_en-US.msi"
    }
  }
}
```

4. Opublikowanie GitHub Release z załączonymi instalatorami, podpisami, `latest.json`, sumami SHA-256 oraz opisem wydania.
