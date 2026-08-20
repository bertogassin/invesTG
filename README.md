# invesTG - Rust Telegram Bot with SQLite

Interactive Telegram bot built with Rust, teloxide, and SQLite for collecting investment votes by geography and category.

## Features

- 🤖 Async Telegram bot using `teloxide` framework
- 💾 SQLite database with pre-populated geography data
- 🗺️ Interactive flow: `/start` → Continent → Country → City → Category → Vote
- 🔐 Admin controls via environment variables
- 📊 Vote tracking and statistics

## Prerequisites

- Rust 1.70+ ([Install](https://rustup.rs/))
- A Telegram Bot Token from [@BotFather](https://t.me/botfather)

## Setup

1. **Clone and navigate to project:**
   ```bash
   git clone https://github.com/bertogassin/invesTG.git
   cd invesTG
   ```

2. **Create .env file:**
   ```bash
   cp .env.example .env
   ```

3. **Configure your bot token:**
   ```bash
   # Edit .env and add your Telegram bot token
   echo "BOT_TOKEN=your_actual_token_here" >> .env
   ```

## Build

```bash
cargo build --release
```

The compiled binary will be at `target/release/invesTG`.

## Run

```bash
./target/release/invesTG
```

Or directly with cargo:
```bash
cargo run --release
```

## Bot Commands

- `/start` - Begin the interactive selection flow
- `/cancel` - Cancel current operation and return to start
- `/stats` - View voting statistics (admin only)
- `/help` - Display available commands

## Interactive Flow

1. User sends `/start`
2. Bot displays continent buttons (inline keyboard)
3. User selects continent → shows countries
4. User selects country → shows cities
5. User selects city → shows categories
6. User selects category → can submit vote (1-5 stars)
7. Vote saved to SQLite database

## Database

Database is automatically initialized on first run with seed data:
- **5 Continents**: Africa, Asia, Europe, North America, South America
- **20+ Countries**: Distributed across continents
- **50+ Cities**: Distributed across countries
- **10+ Categories**: Various investment sectors
- **Votes Table**: Tracks all user votes with timestamps
- **User Sessions**: Maintains user state during navigation

## Project Structure

```
src/
├── main.rs          # Bot setup, command dispatcher
├── db.rs            # SQLite initialization, queries
├── handlers.rs      # Callback and message handlers
├── models.rs        # Data structures (Geography, Category, Vote)
└── utils.rs         # Helper functions
```

## Configuration

Edit `.env` to customize:

```env
BOT_TOKEN=your_bot_token          # Required: Telegram bot token
ADMIN_IDS=123,456,789             # Optional: Comma-separated admin user IDs
DATABASE_PATH=./invesTG.db        # Optional: Database file location
RUST_LOG=info                      # Optional: Log level (debug, info, warn, error)
```

## Development

### Run in debug mode:
```bash
cargo run
```

### Run tests:
```bash
cargo test
```

### Check code:
```bash
cargo clippy
cargo fmt --check
```

## Roadmap

- [ ] Add PostgreSQL support
- [ ] Docker containerization
- [ ] Deployment guides
- [ ] Vote analytics dashboard
- [ ] Export voting data to CSV

## License

MIT

## Support

For issues or questions, open a GitHub issue.
