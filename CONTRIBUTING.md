# Contributing to Ripress

Thank you for your interest in contributing to Ripress! We welcome contributions from developers of all skill levels. This guide will help you get started with contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Contributing Guidelines](#contributing-guidelines)
- [Testing](#testing)
- [Documentation](#documentation)
- [Performance Considerations](#performance-considerations)
- [Submitting Changes](#submitting-changes)
- [Community](#community)

## Code of Conduct

By participating in this project, you agree to abide by our community guidelines. Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- **Rust 1.80+** - Latest stable version recommended
- **Git** - For version control
- **cargo** - Rust package manager (comes with Rust)

### Types of Contributions

We welcome various types of contributions:

- 🐛 **Bug Reports** - Help us identify and fix issues
- 💡 **Feature Requests** - Suggest new functionality
- 🔧 **Code Contributions** - Bug fixes, new features, optimizations
- 📚 **Documentation** - Improve guides, examples, and API docs
- 🧪 **Testing** - Add test coverage, benchmark improvements
- 🎨 **Examples** - Real-world usage examples and tutorials

## Development Setup

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:

   ```bash
   git clone https://github.com/YOUR_USERNAME/ripress.git
   cd ripress
   ```

3. **Add upstream remote**:

   ```bash
   git remote add upstream https://github.com/guru901/ripress.git
   ```

4. **Install dependencies**:

   ```bash
   cargo build
   ```

5. **Run tests** to ensure everything works:
   ```bash
   cargo test
   ```

## Project Structure

```
ripress/
├── src/
│   ├── app/              # Main App struct and routing
│   ├── req/              # Request handling (HttpRequest, query params, headers, etc.)
│   ├── res/              # Response handling (HttpResponse, status codes, headers, etc.)
│   ├── middlewares/      # Built-in middleware
│   ├── router/           # Router implementation
│   ├── helpers.rs        # Utility functions
│   ├── types.rs          # Type definitions and traits
│   └── lib.rs           # Library entry point
├── tests/              # Integration tests (includes Playwright tests)
├── docs/               # Documentation
├── scripts/             # Build and test scripts
└── Cargo.toml
```

### Key Components

- **`app/`** - Core application logic and server setup
- **`req/`** - Request handling, parsing, and data extraction
- **`res/`** - Response handling, status codes, and headers
- **`middlewares/`** - CORS, file upload, rate limiting, compression, etc.
- **`router/`** - Route matching and handler dispatch
- **`types.rs`** - Shared type definitions and traits
- **`helpers.rs`** - Utility functions and common operations

## Contributing Guidelines

### Code Style

- Follow **standard Rust formatting** (`cargo fmt`)
- Use **clippy** for linting (`cargo clippy`)
- Write **clear, descriptive variable names**
- Add **documentation comments** for public APIs
- Keep **functions focused** and reasonably sized

### Performance First

Ripress maintains **97% of Actix-Web performance**. When contributing:

- **Benchmark changes** that could affect performance
- **Avoid allocations** in hot paths where possible
- **Use `&str` over `String`** when appropriate
- **Profile before optimizing** - measure, don't guess
- **Consider async overhead** - sometimes sync is faster

### Error Handling

- Use **`Result<T, E>`** for fallible operations
- Create **meaningful error types** with context
- **Don't panic** in library code (except for truly exceptional cases)
- Provide **helpful error messages** for debugging

### API Design Principles

- **Express.js familiarity** - Keep the API intuitive for JS developers
- **Type safety** - Leverage Rust's type system for correctness
- **Composability** - Allow middleware and handlers to be easily combined
- **Backward compatibility** - Avoid breaking changes in minor versions

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run benchmarks
cargo bench
```

### Writing Tests

- **Unit tests** - Place alongside the code being tested
- **Integration tests** - Place in `tests/` directory
- **Examples** - Ensure all examples in `examples/` compile and run

### Test Guidelines

- Test both **happy path and error cases**
- Use **descriptive test names** that explain what's being tested
- **Mock external dependencies** where appropriate
- **Test performance-critical paths** with benchmarks

## Documentation

### API Documentation

- Use **`///` doc comments** for public APIs
- Include **examples** in documentation
- Document **panics**, **errors**, and **safety** considerations
- Run `cargo doc --open` to preview documentation

### Guides and Tutorials

- Keep documentation **up-to-date** with code changes
- Use **real-world examples** that developers can relate to
- Include **common pitfalls** and how to avoid them
- Test **all code examples** to ensure they work

## Performance Considerations

### Benchmarking

Performance testing is done through integration tests and manual benchmarking:

```bash
# Run performance tests
cargo test --release

# Run the test script for comprehensive testing
./scripts/test.sh
```

### Performance Guidelines

- **Measure first** - Profile before making performance claims
- **Avoid premature optimization** - Focus on correctness first
- **Consider real-world usage** - Micro-benchmarks can be misleading
- **Document performance characteristics** - Help users make informed decisions

## Submitting Changes

### Before Submitting

1. **Update your branch**:

   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run the full test suite**:

   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

3. **Update documentation** if needed
4. **Add tests** for new functionality

### Pull Request Process

1. **Create a feature branch**:

   ```bash
   git checkout -b feature/my-awesome-feature
   ```

2. **Make your changes** following the guidelines above

3. **Commit with clear messages**:

   ```bash
   git commit -m "Add support for custom error handlers

   - Implement ErrorHandler trait
   - Add middleware integration
   - Include comprehensive tests
   - Update documentation with examples"
   ```

4. **Push to your fork**:

   ```bash
   git push origin feature/my-awesome-feature
   ```

5. **Open a Pull Request** on GitHub

### Pull Request Guidelines

- **Clear title** describing the change
- **Detailed description** explaining the motivation and implementation
- **Link to relevant issues** if applicable
- **Include breaking changes** in the description if any
- **Request review** from maintainers

### Review Process

- Maintainers will review your PR within **1-2 business days**
- Address **feedback constructively** and update your branch
- **Squash commits** if requested before merging
- **Celebrate** when your contribution is merged! 🎉

## Community

### Getting Help

- **GitHub Issues** - For bugs, feature requests, and questions
- **GitHub Discussions** - For general questions and community chat

### Reporting Issues

When reporting bugs, please include:

- **Rust version** (`rustc --version`)
- **Ripress version**
- **Minimal reproduction case**
- **Expected vs actual behavior**
- **Error messages** and stack traces

### Feature Requests

When suggesting features:

- **Explain the use case** - Why is this needed?
- **Provide examples** - How would it be used?
- **Consider alternatives** - Are there existing solutions?
- **Think about API design** - How should it integrate?

## Recognition

Contributors are recognized in:

- **Changelog** - Contributions credited in release notes
- **GitHub** - Contributor graphs and commit history

## Questions?

Don't hesitate to ask questions! We're here to help:

- Open an issue for **technical questions**
- Use discussions for **general questions**

Thank you for contributing to Ripress! Together, we're building the future of Rust web development. 🚀
