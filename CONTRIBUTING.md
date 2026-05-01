# Contributing to Fenrir

So you want to help build a game launcher. Nice. Here's everything you need to
know to get started without breaking things (or at least, breaking them in
interesting ways).

## Getting Started

1. Fork the repo and clone your fork
2. Create a branch from `dev` (not `main`):
   ```bash
   git checkout dev
   git checkout -b feat/your-feature
   ```
3. Make your changes
4. Push and open a PR against `dev`

## Development Setup

You'll need Rust stable and the usual cargo toolchain.

```bash
# Build everything
cargo build

# Run the test suite
cargo test --all

# Lint -- CI enforces zero warnings, so this needs to be clean
cargo clippy --all-targets -- -D warnings

# Format check
cargo fmt --all --check
```

All four of these must pass before your PR can be merged. CI runs them
automatically, but catching issues locally saves everyone a round-trip.

## Good First Contributions

Not sure where to start? These require no Rust knowledge:

- **Add a detection signature** -- if Fenrir doesn't detect a game you have,
  add a TOML signature for it. See the
  [Signatures Guide](docs/dev/signatures-guide.md). Open issues labeled
  `signature-needed` have specific requests.

- **Add or improve a tuning profile** -- if a game type needs specific Wine
  configuration that the current profiles don't cover, write one. See the
  [Profiles Guide](docs/dev/profiles-guide.md).

- **Test on real games** -- run `fenrir --verbose scan` on your library and
  report what gets misdetected or missed. This feeds directly into signature
  improvements.

Both signatures and profiles are plain TOML files. You don't need to understand
Rust to contribute them.

## Commit Conventions

We use [Conventional Commits](https://www.conventionalcommits.org/) with module
scopes. The format is:

```
<type>(<scope>): <description in imperative mood>
```

Types: `feat`, `fix`, `test`, `refactor`, `chore`, `docs`, `ci`

Scopes match the crate modules: `config`, `scanner`, `prefix`, `runtime`,
`launcher`, `db`, `cli`. Use them.

Examples:
```
feat(scanner): add Epic Games Store signature
fix(prefix): handle spaces in Wine binary path
test(launcher): cover proton command edge cases
docs: update commands reference
ci: add cargo audit step
```

## Branch Model

```
main       -- stable releases only
  dev      -- integration branch, PRs go here
    feat/* -- feature branches
    fix/*  -- bug fixes
```

## Pull Request Checklist

Before opening a PR, make sure:

- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets -- -D warnings` is clean
- [ ] `cargo fmt --all --check` is clean
- [ ] Commits follow the conventional format
- [ ] You've rebased on `dev` to avoid merge conflicts

PRs require at least one review and a green CI before merge.

## Code Style

- Follow `rustfmt.toml` (edition 2021, 100-char width, 4-space indent)
- Follow `clippy.toml` (relaxed args threshold at 8)
- Use `thiserror` for error types in the core library
- Use `Box<dyn Error>` in the CLI layer
- Write tests for new functionality -- inline `#[cfg(test)]` for unit tests,
  `crates/fenrir-core/tests/integration_test.rs` for end-to-end cases
- No comments explaining what code does; only add one when the why is
  non-obvious (a hidden constraint, a workaround, a subtle invariant)

## Reporting Issues

Found a bug? Open an issue with:

- What you did
- What you expected
- What actually happened
- Your distro, kernel version, and Wine/Proton version

If Fenrir misdetects or misses a game, include the output of:
```bash
fenrir --verbose scan --path /path/to/that/game/
```

## Conduct

Be direct and constructive. We don't have a formal code of conduct document,
but the basics apply: review code, not people; give specific feedback, not
vague complaints; if you disagree with a decision, explain why.
