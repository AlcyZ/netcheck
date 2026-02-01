# netcheck 🦀

A robust internet connectivity monitor written in Rust.

This tool monitors the stability of your internet connection by performing regular checks against multiple targets. It is designed to provide reliable logs even with unstable connections or sudden power outages.


## Features

- **Multi-Target Checking**: Validates connectivity against Google (Generate 204), Cloudflare (1.1.1.1), and Example.com.
- **Structured Logging**: Generates machine-readable JSON Lines (`.jsonl`) for easy post-analysis (e.g., using `jq` or Python).
- **Automated Rotation**: Creates a new log file daily to keep file sizes manageable.
- **Crash-Resistant**: Uses blocking I/O to minimize data loss during system crashes.

---

## Installation

Ensure you have the [Rust toolchain](https://rustup.rs/) installed.

```bash
# Clone the repository
git clone [https://github.com/YOUR_USERNAME/netcheck.git](https://github.com/YOUR_USERNAME/netcheck.git)
cd netcheck

# Build the project
cargo build --release
```

---

## Usage

Run the monitor directly from your terminal:

```bash
./target/release/netcheck
```

---

## Tech Stack

- **Async Runtime**: [Tokio](https://tokio.rs/)
- **Logging**: [Tracing](https://github.com/tokio-rs/tracing) & [Tracing-Appender](https://docs.rs/tracing-appender)
- **Serialization**: [Serde](https://serde.rs/) & [Serde_JSON](https://docs.rs/serde_json)
- **HTTP Client**: [Reqwest](https://docs.rs/reqwest/)

## License

This project is licensed under the [MIT License](LICENSE).

