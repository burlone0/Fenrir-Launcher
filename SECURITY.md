# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.2.x (latest) | Yes |
| 0.1.x | No |

Security fixes are applied to the latest release only. If you're on an older
version, update first.

## Reporting a Vulnerability

Please don't open a public GitHub issue for security vulnerabilities.

Report them privately by emailing the maintainers. You can find contact info
in the GitHub profiles of the repository owners. We'll respond within a few
days and work with you to understand and fix the issue before any public
disclosure.

What to include in your report:

- A description of the vulnerability and its potential impact
- Steps to reproduce it (command line, input files, environment)
- Any relevant logs or error output
- Your suggested fix, if you have one (not required)

## Scope

Fenrir is a local CLI tool that manages Wine prefixes and launches game
executables. Its attack surface is relatively small but worth taking seriously.

**In scope:**

- Arbitrary code execution via malicious TOML files (signatures, profiles,
  config) -- Fenrir loads these from user-controlled directories, but a
  malicious file dropped in `data/signatures/` or `~/.config/fenrir/` should
  not be able to run code during parsing
- Path traversal vulnerabilities in the scanner or cleanup module
- Checksum bypass in the runtime download/install path
- Privilege escalation (Fenrir should never require root and should never
  elevate itself)

**Out of scope:**

- Vulnerabilities in Wine/Proton itself
- Games doing bad things -- Fenrir launches executables you chose; what those
  executables do is not Fenrir's responsibility
- Issues that require an attacker to already have write access to your home
  directory -- at that point your system is already compromised

## What We Do With Reports

1. Acknowledge receipt within a few days
2. Reproduce and assess the issue
3. Develop and test a fix
4. Release the fix and credit you in the changelog (unless you prefer to
   stay anonymous)

We don't have a bug bounty program. What we can offer is a credit in the
release notes and the satisfaction of having made a small open-source project
a little more solid.
