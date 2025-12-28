# 🔥 Oxidize

**Ultra-minimalist workout tracker** built with Rust, Leptos, and WebAssembly.

## Features

- 📱 **PWA** - Install on your phone, works offline
- ⚡ **Three-Tap Rule** - Log sets in seconds
- 📊 **Smart Stats** - E1RM, Power-to-Weight, Efficiency tracking
- 🔥 **Progressive Overload** - Visual indicators for progress
- 🗺️ **Muscle Heatmap** - See which muscles need attention
- ☁️ **Cloud Sync** - Supabase backup (optional)

## Tech Stack

- **Frontend**: Rust + Leptos (compiled to WebAssembly)
- **Styling**: Pure CSS, high-contrast dark mode
- **Storage**: localStorage + Supabase
- **Hosting**: GitHub Pages

## Ändra träningspass

Passen definieras i `routines.json`. Så här gör du för att ändra:

1. **Redigera `routines.json`** - lägg till/ta bort/ändra övningar
2. **Be AI:n synka** - skriv "synka storage.rs med routines.json"
3. **Klar!** - Rust-koden uppdateras automatiskt

### Övningstyper

```json
// Standard övning
{ "name": "Knäböj", "sets": 3, "reps_target": "5-8", "type": "standard" }

// Superset (två övningar som alternerar)
{
  "type": "superset",
  "pair": [
    { "name": "Leg Curls", "sets": 2, "reps_target": "12-15" },
    { "name": "Dips", "sets": 2, "reps_target": "AMRAP" }
  ]
}

// Finisher (bodyweight, visas sist)
{ "name": "Shoulder Taps", "sets": 3, "reps_target": "20", "is_bodyweight": true }
```

### Hints (tips som visas i appen)

```json
{ "name": "Hammercurls", "sets": 3, "reps_target": "10-12", "hint": "Lägg ihop båda hantlarnas vikt" }
```

## Development

```bash
# Install dependencies
cargo install trunk
rustup target add wasm32-unknown-unknown

# Run locally
trunk serve

# Build for production
trunk build
```

## Deploya till GitHub Pages

```bash
trunk build --release
# Kopiera assets och pusha till gh-pages branch
```

## License

MIT
