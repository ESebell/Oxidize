# Oxidize - Workout Tracker

Tvåplattformsapp: PWA (Rust/Leptos/WASM) + native iOS (SwiftUI).
Delad backend via Supabase. UI-språk: Svenska.

---

## Web (PWA)

### Snabbstart

```bash
trunk serve          # Dev-server på localhost:8080
trunk build --release  # Produktion → dist/
```

Kräver: `cargo install trunk` och `rustup target add wasm32-unknown-unknown`

### Deploy

Manuell deploy till GitHub Pages via `gh-pages`-branchen:

```bash
trunk build --release
cp -f manifest.json sw.js icon-*.png dist/
git checkout gh-pages
cp dist/* .
# git rm gamla .js/.wasm/.css-filer, git add nya
git commit -m "Deploy: ..."
git push origin gh-pages
git checkout main
```

VIKTIGT: Bumpa `CACHE_NAME` i `sw.js` vid varje deploy,
annars serverar service workern cachad gammal version.

Det finns INGEN automatisk CI/CD — push till `main` deployer inte.

### Moduler

| Fil | Ansvar |
|-----|--------|
| `src/app.rs` | App-root + delade hjälpfunktioner |
| `src/pages/` | UI-komponenter, en fil per vy |
| `src/pages/auth.rs` | Login + Register |
| `src/pages/dashboard.rs` | Dashboard |
| `src/pages/workout.rs` | Workout + WorkoutActive |
| `src/pages/stats_page.rs` | Stats + WeightChart |
| `src/pages/settings.rs` | Settings |
| `src/pages/routine_builder.rs` | RoutineBuilder + AI-generering + Wger-sök |
| `src/supabase.rs` | Auth, molnsynk, Supabase REST API |
| `src/storage.rs` | localStorage, databasoperationer |
| `src/types.rs` | Alla datastrukturer |
| `src/stats.rs` | Statistik, muskelkarta, E1RM-beräkningar |
| `src/lib.rs` | WASM-entrypoint |

### Dataflöde

```
Supabase (källa)  →  localStorage (cache)  →  Leptos-signaler (UI)
```

- Vid appstart: token-refresh → reset sync status → sync_from_cloud() → mount UI
- Push: sessioner sparas direkt till Supabase vid workout-save
- Pull: molndata ersätter lokal cache (cloud-first)

### PWA

- Service worker: `sw.js` (cache-first, bumpa CACHE_NAME vid deploy!)
- Manifest: `manifest.json` (standalone, dark theme)

### localStorage-nycklar

- `oxidize_db_v2` - Hela databasen (sessions, vikter, kroppsvikt)
- `oxidize_auth_session` - JWT-token och user-info
- `oxidize_paused_workout` - Pausat pågående pass
- `oxidize_active_routine` - Cachad aktiv rutin
- `oxidize_sync_status` - "pending" / "success" / "failed"
- `oxidize_data_version` - Version-counter för reaktiv UI-uppdatering

### Konventioner (Web)

- 100% Rust, ingen JavaScript-källkod (förutom sw.js och inline i index.html)
- All styling i `style.css` (ren CSS, dark mode)
- Supabase-anrop via `web_sys::window().fetch()` (ingen HTTP-crate)
- Tidsformat: Unix-sekunder (i64) genomgående
- ID:n genereras med `js_sys::Date::now()` som millisekunder

---

## iOS App

### Snabbstart

```bash
cd ios
xcodegen generate                    # Generera .xcodeproj från project.yml
xcodebuild -scheme OxidizeApp \
  -destination 'platform=iOS Simulator,name=iPhone 17 Pro' build
```

Kräver: Xcode 16+, `brew install xcodegen`

### Simulator-kommandon

```bash
DEVICE=5038CE6D-42B6-4D5A-9F63-F8AD1B559960
xcrun simctl install $DEVICE <path-to-.app>
xcrun simctl launch $DEVICE com.oxidize.app
xcrun simctl terminate $DEVICE com.oxidize.app

# Se app-loggar:
xcrun simctl launch --console-pty $DEVICE com.oxidize.app
```

### Arkitektur

SwiftUI + MVVM med `@Observable` ViewModels. XcodeGen (`project.yml`) genererar Xcode-projektet.

### Kodstruktur (ios/OxidizeApp/)

| Mapp | Innehåll |
|------|----------|
| `Models/` | Datastrukturer: Exercise, Pass, SavedRoutine, Session, Database, AuthModels, GeminiModels, WgerModels, WorkoutState, StatsModels |
| `Views/Auth/` | LoginView, RegisterView |
| `Views/Dashboard/` | DashboardView, PausedWorkoutBanner, RecentSessionRow |
| `Views/Workout/` | WorkoutView, WorkoutActiveView, ExerciseScreen, RestScreen, FinishScreen, RepButtonGrid, WeightAdjuster, TimerExerciseView, WorkoutOverviewSheet |
| `Views/RoutineBuilder/` | RoutineBuilderView, PassEditorView, ExerciseRowView, AIWizardSheet, ExerciseSearchSheet, SupersetPickerSheet |
| `Views/Stats/` | StatsView, PowerScoreCard, WeeklyVolumeCard, ProgressionCard, BodyweightChartCard |
| `Views/Settings/` | SettingsView |
| `Services/` | SupabaseService, SyncService, StorageService, GeminiService, WgerService, HealthKitService, HapticService, StatsEngine |
| `ViewModels/` | AuthViewModel, DashboardViewModel, WorkoutViewModel, RoutineBuilderViewModel, StatsViewModel, SettingsViewModel |
| `Utilities/` | Theme, Extensions, Formatters, Constants |
| `Resources/Fonts/` | JetBrains Mono (Regular, Medium, SemiBold, Bold) |

### Dataflöde (iOS)

```
Supabase (källa)  →  UserDefaults/JSON (cache)  →  @Observable ViewModels (UI)
```

- Sync är **pull-only**: molndata ersätter lokal cache vid appstart
- Sessioner pushas direkt till Supabase vid workout-save (fire-and-forget)
- Ingen PUSH-fas i sync (undviker att raderade sessioner återuppstår)

### Integrationer

- **HealthKit**: Läser/skriver kroppsvikt, registrerar pass som `.traditionalStrengthTraining` med kalorier och RPE (1-5)
- **Gemini AI**: Rutingenerator via `app_config`-tabell i Supabase (API-nyckel)
- **Wger API**: Övningssök i rutinbyggaren
- **Haptics**: UIImpactFeedbackGenerator för knapptryck

### Design

- **Tema**: Mörk cyberpunk-estetik matchande PWA:n
- **Font**: JetBrains Mono (bundlad via Info.plist UIAppFonts)
- **Färger**: `#050505` bg, `#00ff88` accent (grön), `#ff6600` accent B (orange)
- **Tracking-centrering**: SwiftUI `.tracking(N)` lägger space efter sista tecknet → kompensera med `.padding(.leading, N)` på centrerad text
- **Modaler**: Egna overlay-modaler istället för system `.alert()` för konsekvent estetik

### Konventioner (iOS)

- `project.yml` (XcodeGen) är source of truth — redigera ALDRIG `.xcodeproj` manuellt
- Alla services är singletons: `ServiceName.shared`
- Codable-structs med `CodingKeys` + custom `init(from:)` för feltoleranta JSON-avkodningar
- Supabase-anrop via `URLSession` (ingen extern SDK)
- Alla vy-strängar på svenska

---

## Delad Backend (Supabase)

### Tabeller

- `sessions` - Träningshistorik (exercises som JSONB)
- `last_weights` - Senaste vikt/reps per övning per user
- `bodyweight` - Vikhistorik
- `user_settings` - Display name, kroppsvikt
- `routines` - Sparade träningsrutiner (passes som JSONB)
- `app_config` - Konfigurationsvärden (t.ex. `gemini_api_key`)

Alla tabeller har RLS-policies som filtrerar på `user_id`.

### Tech Stack

- **Backend:** Supabase PostgreSQL + Auth + RLS
- **AI:** Google Gemini 2.5 Flash (rutin-generator)
- **Web:** Rust + Leptos 0.6 (CSR) → WASM, GitHub Pages
- **iOS:** SwiftUI + MVVM + XcodeGen, HealthKit
- **UI-språk:** Svenska

## Viktiga mönster (båda plattformar)

- **Tre-tap-regeln:** Tryck övning → reps → vikt = ett set loggat
- **Supersets:** `is_superset=true` + `superset_with` korshänvisning på båda övningar
- **E1RM:** Brzycki-formel: `weight × (36 / (37 - reps))`
- **Styrketotal:** Summa E1RM för Big 4 (Squat, Deadlift, Bench Press, Overhead Press)
- **Veckovolym:** Set per muskelgrupp/vecka, optimalt 10-20 set
- **Övningsnamn:** Engelska i databasen, matchas mot muskelkarta
