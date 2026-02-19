# Oxidize - Workout Tracker

## Snabbstart

```bash
# Kör lokalt
trunk serve          # Startar dev-server på localhost:8080

# Bygg för produktion
trunk build --release  # Output i dist/
```

Kräver: `cargo install trunk` och `rustup target add wasm32-unknown-unknown`

## Deploy

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

VIKTIGT: Bumpa `CACHE_NAME` i `sw.js` (t.ex. `oxidize-v37` → `v38`) vid varje deploy,
annars serverar service workern cachad gammal version till användare.

Det finns INGEN automatisk CI/CD — push till `main` deployer inte.

## Arkitektur

Rust + Leptos 0.6 (CSR) kompilerat till WebAssembly. PWA med offline-stöd.

### Moduler

| Fil | Ansvar |
|-----|--------|
| `src/app.rs` | App-root + delade hjälpfunktioner (format_time, format_date, format_weight, parse_target_range) |
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

### Navigering (AppView enum i types.rs)

`Login` → `Register` → `Dashboard` → `Workout(pass_name)` → `Stats` → `Settings` → `RoutineBuilder`

### Dataflöde

```
Supabase (källa)  →  localStorage (cache)  →  Leptos-signaler (UI)
```

- Vid appstart: token-refresh → reset sync status → sync_from_cloud() → mount UI
- Push: lokala sessioner som saknas i molnet laddas upp
- Pull: molndata ersätter lokal cache (cloud-first)

## Tech Stack

- **Framework:** Leptos 0.6 (reactive, CSR-only)
- **Kompilering:** Trunk → wasm32-unknown-unknown
- **Databas:** Supabase PostgreSQL (RLS per user_id)
- **Auth:** Supabase Auth (email/password, JWT)
- **AI:** Google Gemini 2.5 Flash (rutin-generator)
- **Hosting:** GitHub Pages (statisk PWA, gh-pages branch)
- **Språk i UI:** Svenska

## Supabase-tabeller

- `sessions` - Träningshistorik (exercises som JSONB)
- `last_weights` - Senaste vikt/reps per övning per user
- `bodyweight` - Vikhistorik
- `user_settings` - Display name, kroppsvikt
- `routines` - Sparade träningsrutiner (passes som JSONB)

Alla tabeller har RLS-policies som filtrerar på `user_id`.

## Viktiga mönster

- **Leptos-signaler:** `create_signal`, `create_effect`, `create_memo` för reaktivitet
- **Tre-tap-regeln:** Tryck övning → reps → vikt = ett set loggat
- **Supersets:** Två övningar med `is_superset=true` och korshänvisningar via `superset_with`
- **E1RM:** Brzycki-formel: `weight × (36 / (37 - reps))`
- **Styrketotal:** Summa E1RM för Big 4 (Knäböj, Marklyft, Bänkpress, Militärpress)
- **Veckovolym:** Set per muskelgrupp/vecka, optimalt 10-20 set (primära muskler)
- **Sync-polling:** Dashboard pollar `sync_status` var 200ms tills sync klar

## localStorage-nycklar

- `oxidize_db_v2` - Hela databasen (sessions, vikter, kroppsvikt)
- `oxidize_auth_session` - JWT-token och user-info
- `oxidize_paused_workout` - Pausat pågående pass
- `oxidize_active_routine` - Cachad aktiv rutin
- `oxidize_sync_status` - "pending" / "success" / "failed"
- `oxidize_data_version` - Version-counter för reaktiv UI-uppdatering

## PWA

- Service worker: `sw.js` (cache-first, bumpa CACHE_NAME vid deploy!)
- Manifest: `manifest.json` (standalone, dark theme)
- Post-build hook i Trunk.toml kopierar PWA-assets till dist/

## Konventioner

- Appen är 100% Rust, ingen JavaScript-källkod (förutom sw.js och inline i index.html)
- All styling i `style.css` (ren CSS, dark mode)
- Supabase-anrop görs via `web_sys::window().fetch()` (ingen HTTP-crate)
- Tidsformat: Unix-sekunder (i64) genomgående
- ID:n genereras med `js_sys::Date::now()` som millisekunder
