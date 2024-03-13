# Pizza Picker

We like pizza, and we like gambling. This is a Slack Bot that will give you a random pizza from Pizzabakeren when you
run `/spin`.

## How to use

Create a SlackBot integration and install it to your workspace. It should at least have the scopes `slash_commands` and
`chat:write`.

We use Slack Socket Mode to listen for events, so you need to enable it in your app settings. This will give you a Slack
App Token which you store in your environment as `SLACK_APP_TOKEN`.

If you have Rust/Cargo installed, you can clone the repository and run `cargo run`. If not, you can use the Dockerfile
to build a Docker image and run it.

```bash
# Build release mode
cargo build --release
./target/release/pizzapicker

# Cargo run
cargo run

# Docker
docker build -t pizzapicker:latest .
docker run -e SLACK_APP_TOKEN=<your_token> pizzapicker:latest
```

## License

MIT Licensed, do whatever you want with it.
