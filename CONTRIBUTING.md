# Contributing to Wololo

Thank you for your interest in contributing to Wololo! This document provides guidelines and information for contributors.

## Code of Conduct

By participating in this project, you agree to abide by our code of conduct:
- Be respectful and inclusive
- Focus on constructive feedback
- Help maintain a welcoming environment for all contributors

## Getting Started

### Prerequisites

- Rust 1.70.0 or higher
- Git
- A text editor or IDE of your choice

### Development Setup

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/your-username/wololo.git
   cd wololo
   ```
3. Add the original repository as upstream:
   ```bash
   git remote add upstream https://github.com/sharmashobhit/wololo.git
   ```
4. Create a new branch for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

### Building and Testing

1. Build the project:
   ```bash
   cargo build
   ```

2. Run the application:
   ```bash
   cargo run
   ```

3. Run tests:
   ```bash
   cargo test
   ```

4. Check code formatting:
   ```bash
   cargo fmt --check
   ```

5. Run clippy for linting:
   ```bash
   cargo clippy -- -D warnings
   ```

## Making Changes

### Commit Messages

Follow conventional commit format:
- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `style:` for code style changes
- `refactor:` for code refactoring
- `test:` for adding tests
- `chore:` for maintenance tasks

Example: `feat: add device status checking functionality`

### Code Style

- Use `cargo fmt` to format your code
- Follow Rust naming conventions
- Add documentation comments for public APIs
- Keep functions small and focused
- Use meaningful variable names

### Testing

- Write unit tests for new functionality
- Ensure all existing tests pass
- Test the web interface manually
- Test with different device configurations

## Submitting Changes

1. Push your changes to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

2. Create a pull request on GitHub with:
   - Clear description of changes
   - Reference any related issues
   - Screenshots for UI changes
   - Test results

### Pull Request Guidelines

- Keep PRs focused and atomic
- Include tests for new features
- Update documentation as needed
- Ensure CI passes
- Be responsive to feedback

## Project Structure

```
wololo/
├── src/
│   ├── main.rs          # Application entry point
│   ├── config.rs        # Configuration handling
│   └── routes.rs        # HTTP route handlers
├── frontend/
│   └── index.html       # Main HTML template
├── assets/              # Static assets
├── config.yaml         # Example configuration
└── Cargo.toml          # Dependencies
```

## Areas for Contribution

We welcome contributions in these areas:

### High Priority
- [ ] Device status checking (ping/online detection)
- [ ] Wake On LAN functionality implementation
- [ ] Error handling and user feedback
- [ ] Configuration validation

### Medium Priority
- [ ] Device grouping/categories
- [ ] Scheduled wake-ups
- [ ] REST API endpoints
- [ ] Device discovery

### Low Priority
- [ ] Dark mode theme
- [ ] Multiple network interface support
- [ ] Device history/logging
- [ ] Mobile app companion

## Architecture Notes

- **Backend**: Rust with Axum for HTTP handling
- **Frontend**: HTMX for dynamic interactions, Handlebars for templating
- **Configuration**: YAML-based device management
- **Styling**: Tailwind CSS for responsive design

## Dependencies

Key dependencies and their purposes:
- `axum`: Web framework
- `tokio`: Async runtime
- `handlebars`: Template engine
- `serde`: Serialization/deserialization
- `wol-rs`: Wake On LAN functionality
- `tower-http`: HTTP utilities

## Getting Help

- Open an issue for bug reports or feature requests
- Join discussions in existing issues
- Check the README for basic usage questions

## Security

- Report security vulnerabilities privately
- Don't commit sensitive information
- Follow secure coding practices

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.

Thank you for contributing to Wololo! 🎉