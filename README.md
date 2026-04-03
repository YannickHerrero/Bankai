<div align="center">

# Bankai

A terminal UI client for [AniList](https://anilist.co) — track your anime and manga from the command line.

Built with Rust, [ratatui](https://github.com/ratatui/ratatui), and the AniList GraphQL API.

</div>

---

## Features

- **Dashboard** — View your currently watching list, upcoming episode schedule, and recent activity
- **Search** — Browse the AniList catalog, view details, and add anime/manga to your list
- **Statistics** — Score distribution, top genres, format breakdown, and watch time overview
- **Page Selector** — Quick navigation popup with fuzzy search (press `Space`)
- **Persistent Sessions** — Automatically saves and restores your login between sessions
- **Async UI** — All API calls run in the background without blocking the interface

## Screenshots

*Coming soon*

## Installation

### From source

```bash
git clone https://github.com/YannickHerrero/Bankai.git
cd Bankai
cargo build --release
```

The binary will be at `target/release/bankai`.

### With cargo install

```bash
cargo install --git https://github.com/YannickHerrero/Bankai.git
```

### Requirements

- Rust (edition 2024)

## Setup

Bankai requires an AniList API application to authenticate.

1. Go to [AniList Developer Settings](https://anilist.co/settings/developer) and create a new application
2. Set the environment variables:
   ```bash
   export BANKAI_CLIENT_ID="your_client_id"
   export BANKAI_CLIENT_SECRET="your_client_secret"
   ```
   Or create a `.env` file in the project directory:
   ```
   BANKAI_CLIENT_ID=your_client_id
   BANKAI_CLIENT_SECRET=your_client_secret
   ```
3. Run the application:
   ```bash
   cargo run
   ```
   Or if installed:
   ```bash
   bankai
   ```

Your authentication token is saved to `~/.config/bankai/config.toml` so you only need to log in once.

## Keyboard Controls

### General

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Space` | Open page selector |
| `Esc` | Cancel / Go back |
| `Enter` | Confirm / Select |

### Navigation

| Key | Action |
|-----|--------|
| `j` / `k` or `Up` / `Down` | Navigate lists |
| `Shift+h/j/k/l` or `Shift+Arrows` | Move between panels |

### Search

| Key | Action |
|-----|--------|
| `/` | Focus search input |
| `Tab` or `h` / `l` | Switch between Anime and Manga |
| `a` | Add or remove media from your list |

## Tech Stack

- **[ratatui](https://github.com/ratatui/ratatui)** — Terminal UI framework
- **[crossterm](https://github.com/crossterm-rs/crossterm)** — Cross-platform terminal control
- **[tokio](https://github.com/tokio-rs/tokio)** — Async runtime
- **[reqwest](https://github.com/seanmonstar/reqwest)** — HTTP client for the AniList GraphQL API

## Project Structure

```
src/
├── main.rs          # Event loop and message handling
├── app.rs           # Application state
├── api.rs           # AniList GraphQL client
├── auth.rs          # OAuth authentication flow
├── token.rs         # Token persistence
└── ui/
    ├── mod.rs           # Rendering dispatcher
    ├── login.rs         # Login screen
    ├── page_selector.rs # Fuzzy page selector popup
    └── pages/
        ├── dashboard.rs # Currently watching, calendar, activity
        ├── search.rs    # Search and manage media
        └── stats.rs     # Statistics visualizations
```

## Contributing

All contributions are appreciated! Whether it's bug reports, feature suggestions, or pull requests — feel free to get involved.

## License

This project is not yet licensed. See [Choose a License](https://choosealicense.com/) if you'd like to add one.
