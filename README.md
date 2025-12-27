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

## License

MIT
