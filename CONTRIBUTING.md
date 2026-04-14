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

The full branching model, PR process, and release flow are documented internally
for team members.

## Pull Request Checklist

Before opening a PR, make sure:

- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all-targets -- -D warnings` is clean
- [ ] `cargo fmt --all --check` is clean
- [ ] Commits follow the conventional format
- [ ] You've rebased on `dev` to avoid merge conflicts

PRs require at least one review and a green CI before merge.

## Extending Fenrir

Fenrir is designed to be extended through TOML data files, not just code.
Two common contribution paths:

**Adding game detection signatures** -- If Fenrir doesn't detect a game type
you care about, you can add a signature pattern. See the
[Signatures Guide](docs/dev/signatures-guide.md) for how the detection system
works and how to write new patterns.

**Adding tuning profiles** -- If a game type needs specific Wine configuration
(DLL overrides, environment variables, etc.), you can create a profile. See the
[Profiles Guide](docs/dev/profiles-guide.md) for the profile format and
examples.

Both of these can be contributed without touching Rust code.

## Code Style

We keep it simple:

- Follow `rustfmt.toml` (edition 2021, 100 char width, 4-space tabs)
- Follow `clippy.toml` (relaxed args threshold at 8)
- Use `thiserror` for error types in the core library
- Use `Box<dyn Error>` in the CLI layer
- Write tests for new functionality

## Reporting Issues

Found a bug? Open an issue with:

- What you did
- What you expected
- What actually happened
- Your distro and Wine/Proton version (if relevant)

## Internal Documentation

Team members have access to additional internal documentation covering
architecture decisions, implementation plans, and workflow details. Ask the team
for access if you need it.
