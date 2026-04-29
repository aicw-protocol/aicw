# Contributing to AICW

Thank you for your interest in contributing to the AICW standard.

## Getting Started

1. Fork the repository
2. Clone your fork and create a new branch
3. Install prerequisites (see [README](README.md#prerequisites))
4. Run `npm install` and `anchor build` to verify setup

## Development Workflow

```bash
# Build the program
anchor build

# Run all tests
anchor test

# Run tests against an existing validator
anchor test --skip-local-validator
```

## Pull Requests

- One PR per feature or fix
- All tests must pass (`anchor test`)
- Keep changes focused — avoid unrelated modifications in the same PR
- Write clear commit messages describing *why*, not just *what*

## Code Style

- Rust: follow standard `rustfmt` formatting
- TypeScript: consistent with existing test patterns
- No commented-out code in PRs

## Reporting Issues

Use [GitHub Issues](https://github.com/aicw-protocol/aicw/issues) for bug reports and feature requests. Include:

- Steps to reproduce (for bugs)
- Expected vs actual behavior
- Solana CLI / Anchor CLI versions

## Security

If you discover a security vulnerability, **do not open a public issue**. See [SECURITY.md](SECURITY.md) for responsible disclosure instructions.
