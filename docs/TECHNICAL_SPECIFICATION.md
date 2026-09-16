# Szczegółowa Specyfikacja Techniczna: SkillSync

Niniejszy dokument definiuje specyfikację implementacyjną, struktury danych, protokoły IPC oraz konfigurację technologiczną aplikacji **SkillSync**.

---

## 1. Architektura Technologiczna i Zależności

### 1.1. Backend (Rust / Tauri v2 Core)

| Biblioteka / Crate | Wersja | Cel i Zastosowanie |
| :--- | :--- | :--- |
| `tauri` | `^2.1.0` | Szkielet aplikacji desktopowej, okna, menu, tray, mostek IPC. |
| `tauri-plugin-updater` | `^2.0.0` | Automatyczne, kryptograficznie weryfikowane aktualizacje aplikacji. |
| `tauri-plugin-notification` | `^2.0.0` | Natywne powiadomienia w systemach macOS i Windows. |
| `tauri-plugin-fs` | `^2.0.0` | Kontrolowany dostęp do operacji na systemie plików. |
| `tauri-plugin-shell` | `^2.0.0` | Bezpieczne otwieranie linków zewnętrznych oraz folderów w edytorach. |
| `git2` | `^0.19.0` | Natywny binding do `libgit2` — obsługa operacji Git bez subshelli. |
| `tokio` | `^1.38.0` | Asynchroniczny runtime, wielowątkowe kanały MPSC, `Semaphore`. |
| `serde` / `serde_json` | `^1.0.120` | Serializacja i deserializacja struktur danych do/z JSON. |
| `semver` | `^1.0.23` | Rygorystyczne parsowanie i porównywanie wersji wg SemVer 2.0.0. |
| `walkdir` | `^2.5.0` | Rekurencyjna, szybka eksploracja katalogów na dysku. |
| `notify` | `^6.1.1` | Reaktywny monitoring zmian w plikach oparty o zdarzenia jądra OS. |
| `flate2` / `tar` | `^1.0.30` | Tworzenie skompresowanych archiwów `.tar.gz` dla kopii zapasowych. |
| `directories-next` | `^2.0.0` | Standardowe, ujednolicone ścieżki systemowe (Application Support / AppData). |
| `chrono` | `^0.4.38` | Precyzyjna obsługa dat, czasu i znaczników stref czasowych UTC. |
| `thiserror` | `^1.0.63` | Definiowanie ustrukturyzowanych, typowanych błędów domenowych. |

### 1.2. Frontend (React 19 + TypeScript + Tailwind CSS)

| Pakiet | Wersja | Cel i Zastosowanie |
| :--- | :--- | :--- |
| `react` / `react-dom` | `^19.0.0` | Najnowsza biblioteka UI ze zoptymalizowanym silnikiem renderującym. |
| `typescript` | `^5.5.0` | Ścisła statyczna kontrola typów w całej warstwie prezentacji. |
| `vite` | `^5.3.0` | Błyskawiczny bundler i środowisko deweloperskie z HMR. |
| `tailwindcss` | `^4.0.0` | Elastyczny, wysokowydajny silnik stylizacji oparty o utility classes. |
| `@radix-ui/*` | `latest` | Dostępne, bezstylowe prymitywy UI zgodne ze standardem WAI-ARIA. |
| `zustand` | `^4.5.4` | Lekki, zoptymalizowany pod kątem renderowania menedżer stanu. |
| `immer` | `^10.1.1` | Niemutowalna, deklaratywna manipulacja stanem w store Zustand. |
| `framer-motion` | `^11.3.0` | Deklaratywne animacje oparte na fizyce sprężyn (spring physics). |
| `@tanstack/react-virtual`| `^3.8.0` | Wirtualizacja widoków dla płynnego renderowania 1000+ elementów. |
| `lucide-react` | `^0.400.0` | Nowoczesny, spójny wizualnie zestaw ikon SVG. |

---

## 2. Modele Danych w Języku Rust (Backend Structs)

Poniżej zdefiniowano kompletny zestaw struktur domenowych wykorzystywanych w backendzie Rust:

```rust
// src-tauri/src/models/skill.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentScope {
    Global,
    Claude,
    Cursor,
    Antigravity,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkillStatus {
    UpToDate,
    UpdateAvailable,
    ModifiedLocally,
    Corrupted,
    Updating,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub author: String,
    pub path: PathBuf,
    pub is_git_repo: bool,
    pub remote_url: Option<String>,
    pub branch_or_tag: Option<String>,
    pub agent_scope: AgentScope,
    pub status: SkillStatus,
    pub update_available: bool,
    pub changelog: Option<String>,
    pub dependencies: Vec<String>,
    pub permissions: Vec<String>,
    pub last_checked: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgressPayload {
    pub skill_id: String,
    pub skill_name: String,
    pub stage: UpdateStage,
    pub percentage: u8,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UpdateStage {
    Validating,
    BackingUp,
    Fetching,
    CheckingOut,
    Verifying,
    RollingBack,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSnapshot {
    pub snapshot_id: String,
    pub skill_id: String,
    pub created_at: DateTime<Utc>,
    pub backup_file_path: PathBuf,
    pub original_version: String,
}
```

### Konfiguracja Aplikacji (AppConfig)

```rust
// src-tauri/src/models/config.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub paths: PathsConfig,
    pub updates: UpdatesConfig,
    pub notifications: NotificationsConfig,
    pub appearance: AppearanceConfig,
    pub advanced: AdvancedConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitoredPath {
    pub id: String,
    pub path: PathBuf,
    pub scope: String,
    pub custom_label: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralConfig {
    pub language: String,
    pub launch_at_login: bool,
    pub minimize_to_tray: bool,
    pub check_app_updates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathsConfig {
    pub monitored: Vec<MonitoredPath>,
    pub default_install_directory: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatesConfig {
    pub auto_check_frequency: String, // "hourly" | "every_6_hours" | "daily" | "manual"
    pub auto_install: String,         // "ask" | "always" | "never"
    pub concurrency_limit: usize,     // 1 to 10
    pub backup_retention_days: u32,
    pub allow_prerelease: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationsConfig {
    pub enabled: bool,
    pub on_update_found: bool,
    pub on_update_success: bool,
    pub on_update_failure: bool,
    pub sound: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceConfig {
    pub theme: String, // "system" | "dark" | "light"
    pub accent_color: String,
    pub reduced_motion: bool,
    pub compact_view: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvancedConfig {
    pub log_level: String,
    pub git_timeout_seconds: u64,
    pub custom_git_binary: Option<PathBuf>,
    pub cache_ttl_minutes: u64,
}
```

---

## 3. Typy w Języku TypeScript (Frontend Contracts)

Poniżej przedstawiono definicje TypeScript odpowiadające 1:1 modelom z języka Rust:

```typescript
// src/types/skillsync.d.ts

export type AgentScope = 'global' | 'claude' | 'cursor' | 'antigravity' | { custom: string };

export type SkillStatus =
  | 'up_to_date'
  | 'update_available'
  | 'modified_locally'
  | 'corrupted'
  | 'updating'
  | { error: string };

export type UpdateStage =
  | 'validating'
  | 'backing_up'
  | 'fetching'
  | 'checking_out'
  | 'verifying'
  | 'rolling_back'
  | 'completed'
  | 'failed';

export interface SkillMetadata {
  id: string;
  name: string;
  description: string;
  currentVersion: string;
  latestVersion: string | null;
  author: string;
  path: string;
  isGitRepo: boolean;
  remoteUrl: string | null;
  branchOrTag: string | null;
  agentScope: AgentScope;
  status: SkillStatus;
  updateAvailable: boolean;
  changelog: string | null;
  dependencies: string[];
  permissions: string[];
  lastChecked: string;
}

export interface UpdateProgressPayload {
  skillId: string;
  skillName: string;
  stage: UpdateStage;
  percentage: number;
  message: string;
}

export interface MonitoredPath {
  id: string;
  path: string;
  scope: string;
  customLabel?: string;
  enabled: boolean;
}

export interface AppConfig {
  general: {
    language: string;
    launchAtLogin: boolean;
    minimizeToTray: boolean;
    checkAppUpdates: boolean;
  };
  paths: {
    monitored: MonitoredPath[];
    defaultInstallDirectory: string;
  };
  updates: {
    autoCheckFrequency: 'hourly' | 'every_6_hours' | 'daily' | 'manual';
    autoInstall: 'ask' | 'always' | 'never';
    concurrencyLimit: number;
    backupRetentionDays: number;
    allowPrerelease: boolean;
  };
  notifications: {
    enabled: boolean;
    onUpdateFound: boolean;
    onUpdateSuccess: boolean;
    onUpdateFailure: boolean;
    sound: boolean;
  };
  appearance: {
    theme: 'system' | 'dark' | 'light';
    accentColor: string;
    reducedMotion: boolean;
    compactView: boolean;
  };
  advanced: {
    logLevel: 'debug' | 'info' | 'warn' | 'error';
    gitTimeoutSeconds: number;
    customGitBinary: string | null;
    cacheTtlMinutes: number;
  };
}
```

---

## 4. Specyfikacja Komend IPC (Tauri Invocations)

Komunikacja pomiędzy React 19 a Rustem odbywa się poprzez silnie typowane komendy IPC:

| Nazwa Komendy IPC | Argumenty Wejściowe | Typ Zwracany | Opis Funkcjonalny |
| :--- | :--- | :--- | :--- |
| `scan_skills` | `{ forceRefresh: boolean }` | `Promise<SkillMetadata[]>` | Skanuje monitorowane ścieżki i zwraca listę skilli. |
| `check_single_update` | `{ skillId: string }` | `Promise<SkillMetadata>` | Odpytuje remote o najnowszy tag dla pojedynczego skilla. |
| `update_single_skill` | `{ skillId: string, targetVersion?: string }` | `Promise<SkillMetadata>` | Wykonuje 7-etapową atomową aktualizację wskazanego skilla. |
| `batch_update_skills` | `{ skillIds: string[] }` | `Promise<BatchUpdateResult>` | Uruchamia aktualizację masową w kolejce ze współbieżnością. |
| `rollback_skill` | `{ skillId: string, snapshotId?: string }` | `Promise<boolean>` | Przywraca katalog skilla z kopii zapasowej. |
| `get_config` | `void` | `Promise<AppConfig>` | Pobiera aktualny plik konfiguracyjny z walidacją. |
| `save_config` | `{ config: AppConfig }` | `Promise<void>` | Atomowo zapisuje nową konfigurację na dysku. |
| `open_in_editor` | `{ path: string, editor?: string }` | `Promise<void>` | Otwiera folder skilla w VS Code, Cursorze lub Finderze. |
| `get_backups_list` | `{ skillId: string }` | `Promise<BackupSnapshot[]>` | Zwraca listę dostępnych punktów przywracania. |

---

## 5. Hierarchia Błędów i Kody Wyjątków (Error Handling)

Do modelowania błędów domenowych w backendzie wykorzystywany jest crate `thiserror`:

```rust
// src-tauri/src/errors.rs
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "details")]
pub enum SkillSyncError {
    #[error("Błąd systemu plików: {0}")]
    FileSystem(String),

    #[error("Błąd operacji Git ({code}): {message}")]
    Git { code: i32, message: String },

    #[error("Brak uprawnień do katalogu: {path}")]
    PermissionDenied { path: String },

    #[error("Katalog roboczy posiada niezacommitowane zmiany")]
    WorktreeDirty,

    #[error("Niepoprawny format manifestu: {0}")]
    InvalidManifest(String),

    #[error("Weryfikacja integralności po aktualizacji nie powiodła się: {0}")]
    IntegrityCheckFailed(String),

    #[error("Błąd przywracania kopii zapasowej: {0}")]
    RollbackFailed(String),

    #[error("Przekroczono limit czasu operacji sieciowej ({0}s)")]
    NetworkTimeout(u64),

    #[error("Błąd konfiguracji: {0}")]
    Config(String),
}
```

Błędy te są automatycznie serializowane do obiektów JSON i przekazywane do interfejsu użytkownika, gdzie system powiadomień Toast wyświetla precyzyjne, zrozumiałe dla użytkownika wskazówki naprawcze (np. „Nadaj uprawnienia do zapisu komendą chmod”, „Zatwierdź lokalne zmiany przed aktualizacją”).
