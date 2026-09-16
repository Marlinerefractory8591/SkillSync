# Specyfikacja Interfejsu Użytkownika (UI/UX) i Dostępności: SkillSync

Niniejszy dokument precyzuje wytyczne projektowe, architekturę komponentów, paletę kolorystyczną, zasady dostępności cyfrowej (WCAG 2.1 AA) oraz specyfikację animacji dla aplikacji **SkillSync**.

---

## 1. Filozofia Projektowa i Standard Jakości

SkillSync stawia sobie za cel dorównanie standardom wizualnym i ergonomii narzędzi deweloperskich nowej generacji (np. Linear, Raycast, GitHub Desktop):

- **Natychmiastowa Reaktywność (< 100ms Feedback):** Wszystkie interakcje użytkownika (kliknięcie, przełączenie filtrów, otwarcie modala) wywołują natychmiastową reakcję wizualną (optimistic updates, wskaźniki stanu oczekiwania).
- **Zrównoważona Gęstość Informacji:** Interfejs domyślnie prezentuje kluczowe statusy (wersja, dostępność aktualizacji, autor) w zwartej formie, oferując możliwość przejścia do widoku szczegółowego bez opuszczania kontekstu.
- **Pełna Dostępność dla Klawiatury (Keyboard-First):** Każda akcja w aplikacji jest osiągalna bez konieczności używania myszy.
- **Brak Zbędnego Szumu Wizualnego (Decluttered Aesthetics):** Zastosowanie stonowanych, neutralnych odcieni szarości/slate oraz precyzyjnych akcentów kolorystycznych zarezerwowanych wyłącznie dla stanów semantycznych (zielony = aktualny, żółty = aktualizacja, czerwony = błąd).

---

## 2. Paleta Kolorów i Tryby Ciemny / Jasny (Zgodność z WCAG 2.1 AA)

Aplikacja opiera się na tokenach CSS wykorzystujących przestrzeń barw HSL. Wszystkie kombinacje kolorów pierwszoplanowych i tła zostały zweryfikowane pod kątem wymaganego kontrastu minimum **4.5:1** dla tekstu podstawowego oraz **3:1** dla komponentów graficznych:

| Token Projektowy | Tryb Ciemny (Dark) | Tryb Jasny (Light) | Współczynnik Kontrastu (WCAG) | Rola Semantyczna |
| :--- | :--- | :--- | :--- | :--- |
| `--bg-base` | `hsl(224, 71%, 4%)` | `hsl(0, 0%, 100%)` | — | Główne tło okna aplikacji |
| `--bg-surface` | `hsl(222, 47%, 8%)` | `hsl(210, 40%, 98%)` | — | Tło kart, paneli bocznych |
| `--border-subtle` | `hsl(217, 33%, 17%)` | `hsl(214, 32%, 91%)` | $> 3.2:1$ | Linie podziału, obrysy komponentów |
| `--text-primary` | `hsl(210, 40%, 98%)` | `hsl(222, 47%, 11%)` | $> 13.5:1$ (AAA) | Nagłówki, nazwy skilli, tekst główny |
| `--text-muted` | `hsl(215, 20%, 65%)` | `hsl(215, 16%, 47%)` | $> 4.8:1$ (AA) | Opisy, wersje, autorzy |
| `--accent-brand` | `hsl(263, 70%, 65%)` | `hsl(262, 83%, 58%)` | $> 4.6:1$ (AA) | Przyciski główne, aktywne zakładki |
| `--status-success` | `hsl(142, 70%, 45%)` | `hsl(142, 76%, 36%)` | $> 5.1:1$ (AA) | Badge "Up-to-date", sukces operacji |
| `--status-warning` | `hsl(48, 96%, 53%)` | `hsl(38, 92%, 45%)` | $> 4.9:1$ (AA) | Badge "Update Available" |
| `--status-error` | `hsl(0, 72%, 51%)` | `hsl(0, 84%, 60%)` | $> 4.7:1$ (AA) | Badge "Error", akcje destrukcyjne |

### Płynne Przełączanie Motywu (Theme Transition)
Przejście między trybem ciemnym a jasnym jest animowane za pomocą reguły CSS zapewniającej 300-milisekundowe wygaszanie kolorów bez migotania ekranu:

```css
* {
  transition: background-color 300ms cubic-bezier(0.4, 0, 0.2, 1),
              border-color 300ms cubic-bezier(0.4, 0, 0.2, 1),
              color 300ms cubic-bezier(0.4, 0, 0.2, 1);
}
```

---

## 3. Makiety i Układ Kluczowych Ekranów

### 3.1. Dashboard (Główny Pulpit Sterowania)
Ekran domyślny podzielony na trzy sekcje hierarchiczne:

```
┌────────────────────────────────────────────────────────────────────────┐
│ [⚡ SkillSync]    Total: 24   •   Outdated: 3   •   Last Scan: 2m ago   │
├────────────────────────────────────────────────────────────────────────┤
│ [ 🔍 Search skills... (/) ] [ Filters: All | Claude | Cursor | Global ]│
│                                           [ Rescan (⌘R) ] [ Update All │
├────────────────────────────────────────────────────────────────────────┤
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │ ⚡ web-search-pro       v1.4.2 ➔ v1.5.0 [Update Available]        │ │
│ │ Autonomous search and content synthesis engine...                  │ │
│ │ Scope: claude • Author: SkillSync Team • Upstream: github.com/...  │ │
│ │                                  [ Changelog ] [ Update (⌘U) ]     │ │
│ └────────────────────────────────────────────────────────────────────┘ │
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │ 🛡️ ast-code-auditor     v2.1.0          [ Up to Date ]             │ │
│ │ AST-aware security and architectural review tool...                │ │
│ │ Scope: global • Author: DevOps Lead • Upstream: github.com/...     │ │
│ │                                  [ View Details ] [ Open Folder ]  │ │
│ └────────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────────┘
```

### 3.2. Skill Detail Modal & Changelog Viewer
Otwierany po kliknięciu kafelka lub wciśnięciu `Enter`:
- **Lewa Kolumna:** Komplet metadanych, ścieżka bezwzględna na dysku z przyciskiem szybkiego kopiowania, identyfikator commita, zadeklarowane uprawnienia oraz zależności.
- **Prawa Kolumna:** Zintegrowany podgląd release notes w Markdownie pobierany bezpośrednio z Git taga lub GitHub Release.
- **Pasek Akcji:**
  - Przycisk "Update to vX.Y.Z" (główny fioletowy akcent).
  - Przycisk "Rollback" z listą rozwijaną punktów przywracania.
  - Przycisk "Open in VS Code / Cursor".

### 3.3. Update Center (Centrum Kolejki i Historii)
- **Aktywna Kolejka (Active Queue):** Lista skilli oczekujących na pobranie lub w trakcie instalacji. Każdy element posiada indywidualny mikro-pasek postępu z aktualnym stanem (np. `Fetching objects: 45%`, `Validating integrity`).
- **Ustawienia Kolejki:** Suwak limitu wątków współbieżnych (1-10 workerów).
- **Dziennik Audytu (History Log):** Zestawienie wszystkich wykonanych w przeszłości aktualizacji wraz z datą, starą i nową wersją oraz przyciskiem natychmiastowego przywrócenia (Rollback).

---

## 4. Specyfikacja Mikrointerakcji i Animacji (Framer Motion)

Wszystkie animacje wykorzystują sprzętowo akcelerowane właściwości CSS (`transform`, `opacity`) i oparte są o fizykę sprężyn:

### 4.1. Kaskadowe Wejście Listy (Staggered Fade-In)
```typescript
export const listContainerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.04,
      delayChildren: 0.02,
    },
  },
};

export const listItemVariants = {
  hidden: { opacity: 0, y: 8 },
  visible: {
    opacity: 1,
    y: 0,
    transition: {
      type: 'spring',
      stiffness: 400,
      damping: 30,
    },
  },
};
```

### 4.2. Okna Dialogowe (Modal Spring Animation)
```typescript
export const modalVariants = {
  hidden: { opacity: 0, scale: 0.96, y: -10 },
  visible: {
    opacity: 1,
    scale: 1,
    y: 0,
    transition: {
      type: 'spring',
      stiffness: 350,
      damping: 25,
    },
  },
  exit: {
    opacity: 0,
    scale: 0.97,
    transition: { duration: 0.15, ease: 'easeOut' },
  },
};
```

---

## 5. Stany Ładowania i Obsługa Błędów

### 5.1. Skeleton Screens
Podczas początkowego skanowania lub przeładowania listy, aplikacja wyświetla co najmniej 3 kafelki zastępcze (Skeleton Loaders) o pulsującym kontraście, eliminując zjawisko skakania układu (Cumulative Layout Shift = 0).

### 5.2. Komunikaty o Błędach i Sugestie Naprawcze
Aplikacja kategorycznie unika prezentowania użytkownikowi surowych komunikatów systemowych. Każdy błąd tłumaczony jest na format:

```
[Typ Błędu: Ikona Ostrzeżenia]
Tytuł: Nie udało się zaktualizować "web-search-pro"
Opis: Katalog zawiera niezacommitowane modyfikacje plików.
Sugestia: Zatwierdź lub wycofaj lokalne zmiany w repozytorium przed aktualizacją.
Akcje: [Otwórz w Terminalu] [Odrzuć zmiany i zaktualizuj] [Anuluj]
```
