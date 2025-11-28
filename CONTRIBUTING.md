# Contributing to Pwnagotchi Rust

Thank you for your interest in contributing to the Pwnagotchi Rust port! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Git
- A Raspberry Pi with WiFi (for testing)
- Bettercap installed

### Development Setup

1. Clone the repository:

```bash
git clone https://github.com/jayofelony/pwnagotchi.git
cd pwnagotchi
git checkout rust
```

2. Build the project:

```bash
cargo build
```

3. Run tests:

```bash
cargo test --all
```

## Guidelines

### Contributing to the Codebase

- If you would like to contribute to the codebase **please raise an issue to propose the change**
- Do not mix feature changes or fixes with refactoring - it makes the code harder to review and means there is more for the maintainers (with limited time) to test
- If you have found a bug please raise an issue and fill out the whole template
- If the documentation can be improved / translated etc please raise an issue to discuss
- Please always provide a summary of what you changed, how you did it and how it can be tested

### Before Submitting

1. **Format your code**:

```bash
make fmt
# or
cargo fmt --all
```

2. **Run clippy**:

```bash
make clippy
# or
cargo clippy --all -- -D warnings
```

3. **Run tests**:

```bash
make test
# or
cargo test --all
```

### Commit Messages

Follow conventional commit format:

- `feat: add new feature`
- `fix: bug fix`
- `docs: documentation changes`
- `test: add tests`
- `refactor: code refactoring`
- `perf: performance improvements`
- `chore: maintenance tasks`

Example:

```
feat(mesh): add peer discovery timeout

Adds configurable timeout for mesh peer discovery to prevent
stale peers from remaining in the list.
```

### Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run all checks (format, clippy, tests)
5. Commit your changes with clear messages
6. Push to your fork
7. Open a Pull Request

## Code Style

### Rust Style

Follow the standard Rust style guide:

- Use `rustfmt` for formatting
- Follow `clippy` suggestions
- Use meaningful variable names
- Add documentation comments for public APIs
- Keep functions small and focused

### Documentation

All public APIs should have documentation:

```rust
/// Connects to the bettercap API server.
///
/// # Arguments
///
/// * `hostname` - The hostname or IP address
/// * `port` - The port number
///
/// # Returns
///
/// Returns a `Result` with the connected client or an error.
pub fn new(hostname: &str, port: u16) -> Result<Self> {
    // ...
}
```

## License

This project is licensed under the GPL3 License. By contributing, you agree that your contributions will be licensed under the same license.

#### Sign your work

The sign-off is a simple line at the end of the explanation for a patch. Your
signature certifies that you wrote the patch or otherwise have the right to pass
it on as an open-source patch. The rules are pretty simple: if you can certify
the below (from [developercertificate.org](http://developercertificate.org/)):

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.
1 Letterman Drive
Suite D4700
San Francisco, CA, 94129

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.

Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

Then you just add a line to every git commit message:

    Signed-off-by: Joe Smith <joe.smith@email.com>

If you set your `user.name` and `user.email` git configs, you can sign your
commit automatically with `git commit -s`.

- Please sign your commits with `git commit -s` so that commits are traceable.
