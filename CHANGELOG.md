# Changelog

All notable changes to **Arc** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] — 2026-06-13

Initial release.

### Added

- Natural language → shell command generation via the [Groq API](https://groq.com/) (`llama-3.3-70b-versatile`)
- Interactive confirmation prompt before any command runs (`Execute? [y/N]`)
- `arc set [key]` subcommand to store your Groq API key locally
- Config directory at `/usr/arc/` with key file at `/usr/arc/groq.key`
- Restrictive file permissions (`0600`) on the API key file (Unix)
- Built-in safety filter that blocks known destructive command patterns
- Strict system prompt: single command output only — no markdown, explanations, or code fences
- Single static Rust binary built with `clap`, `tokio`, and `reqwest`

### Notes

- Linux (or WSL) required for command execution via `sh -c`
- Groq API key must be configured before first use: `arc set gsk_...`

[0.1.0]: https://github.com/Arcnaboo/arc/releases/tag/v0.1.0
