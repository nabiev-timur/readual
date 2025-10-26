# readual

**Repository Assistant** - a utility for repository management that runs commands and scripts described in README.md file.

## 📋 Description

Readual is a set of utilities for processing and analyzing text files, especially Markdown documents. The project provides a CLI interface for analyzing document structure and executing commands described in README files.

### 🚀 Key Features

- 📄 Parse Markdown files and build heading hierarchy
- 🌳 Display document structure as a file tree
- ⚡ Execute commands described in README.md
- 🔍 Analyze repository structure
- 🎯 Support for various output formats

## 🛠️ Installation

### Requirements

- **Rust**: 1.70+ (recommended 1.75+)
- **Cargo**: latest stable version
- **Git**: for repository management

### Build from Source

```bash
# Clone repository
git clone https://github.com/your-username/readual.git
cd readual

# Build project
cargo build --release

# Install to system
cargo install --path creates/readual-cli
```

## 🏗️ Environment Setup

### Windows

#### PowerShell (recommended)

```bash
# Install Rust via rustup
Invoke-WebRequest -Uri "https://win.rustup.rs" -OutFile "rustup-init.exe"
.\rustup-init.exe -y

# Update PATH
$env:PATH += ";$env:USERPROFILE\.cargo\bin"

# Verify installation
rustc --version
cargo --version
```

#### Command Prompt

```bash
# Install via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload PATH
refreshenv

# Verify installation
rustc --version
cargo --version
```

### Linux (Ubuntu/Debian)

```bash
# Install dependencies
sudo apt update
sudo apt install -y curl build-essential

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Linux (CentOS/RHEL/Fedora)

```bash
# Install dependencies
sudo dnf groupinstall "Development Tools"
sudo dnf install curl

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### macOS

```bash
# Install via Homebrew (recommended)
brew install rust

# Or via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Docker

```bash
# Create Dockerfile
cat > Dockerfile << 'EOF'
FROM rust:1.75-slim

WORKDIR /app
COPY . .
RUN cargo build --release

CMD ["./target/release/readual", "--help"]
EOF

# Build image
docker build -t readual .

# Run container
docker run --rm -v $(pwd):/workspace -w /workspace readual info
```

## 🔨 Build

### Basic Build

```bash
# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Build only CLI utility
cargo build --bin readual

# Build all components
cargo build --workspace
```

### Optimized Build

```bash
# Build with optimizations
cargo build --release --target x86_64-unknown-linux-gnu

# Build for Windows
cargo build --release --target x86_64-pc-windows-msvc

# Build for macOS
cargo build --release --target x86_64-apple-darwin
```

### Cross-compilation

```bash
# Install target platforms
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-msvc
rustup target add x86_64-apple-darwin

# Build for Linux
cargo build --release --target x86_64-unknown-linux-gnu

# Build for Windows
cargo build --release --target x86_64-pc-windows-msvc

# Build for macOS
cargo build --release --target x86_64-apple-darwin
```

### Build with Additional Flags

```bash
# Build with debug information
cargo build --release --profile release-with-debug

# Build with LTO optimization
RUSTFLAGS="-C lto=fat" cargo build --release

# Build with size optimization
RUSTFLAGS="-C opt-level=s" cargo build --release
```

## 🧪 Test

### Run Tests

```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test -- --nocapture

# Run tests for specific package
cargo test -p readual-cli

# Run tests with coverage
cargo test --workspace
```

### CLI Testing

```bash
# Test info command
cargo run --bin readual info

# Test info command with verbose
cargo run --bin readual info --verbose

# Test run command
cargo run --bin readual run

# Test run command with dry-run
cargo run --bin readual run --dry

# Test run command with specific path
cargo run --bin readual run --path "Build::Basic Build::cargo build"
```

### Integration Tests

```bash
# Test README parsing
cargo test test_parse_readme

# Test command execution
cargo test test_command_execution

# Test various output formats
cargo test test_output_formats
```

### Cross-platform Testing

```bash
# Tests for Linux
cargo test --target x86_64-unknown-linux-gnu

# Tests for Windows
cargo test --target x86_64-pc-windows-msvc

# Tests for macOS
cargo test --target x86_64-apple-darwin
```

### Code Quality Checks

```bash
# Format code
cargo fmt

# Check code style
cargo clippy

# Check security
cargo audit

# Analyze dependencies
cargo tree
```

## 📦 Pack

### Create Packages

```bash
# Create source archive
cargo package

# Create documentation archive
cargo doc --no-deps
tar -czf readual-docs.tar.gz target/doc/

# Create release binaries archive
mkdir -p dist
cp target/release/readual.exe dist/ 2>/dev/null || cp target/release/readual dist/
tar -czf readual-linux-x64.tar.gz -C dist .
```

### Create Installer Packages

#### Windows (NSIS)

```bash
# Create NSIS script
cat > installer.nsi << 'EOF'
!define APPNAME "Readual"
!define COMPANYNAME "Readual Team"
!define DESCRIPTION "Repository Assistant"
!define VERSIONMAJOR 0
!define VERSIONMINOR 1
!define VERSIONBUILD 0

!include "MUI2.nsh"

Name "${APPNAME}"
OutFile "readual-installer.exe"
InstallDir "$PROGRAMFILES\${APPNAME}"

Section "install"
    SetOutPath $INSTDIR
    File "target\release\readual.exe"
    CreateShortCut "$DESKTOP\${APPNAME}.lnk" "$INSTDIR\readual.exe"
SectionEnd
EOF

# Compile installer
makensis installer.nsi
```

#### Linux (DEB)

```bash
# Create DEB package
mkdir -p readual_0.1.0/usr/bin
cp target/release/readual readual_0.1.0/usr/bin/
mkdir -p readual_0.1.0/DEBIAN

cat > readual_0.1.0/DEBIAN/control << 'EOF'
Package: readual
Version: 0.1.0
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Nabiev Timur <nabievtimurprogrammer@gmail.com>
Description: Repository Assistant
 A console utility for reading and processing text
EOF

dpkg-deb --build readual_0.1.0
```

#### macOS (PKG)

```bash
# Create PKG package
mkdir -p pkgroot/usr/local/bin
cp target/release/readual pkgroot/usr/local/bin/

pkgbuild --root pkgroot --identifier com.readual.cli --version 0.1.0 readual.pkg
```

### Create Docker Images

```bash
# Create development image
docker build -t readual:dev -f Dockerfile.dev .

# Create production image
docker build -t readual:latest -f Dockerfile.prod .

# Create multi-platform image
docker buildx build --platform linux/amd64,linux/arm64 -t readual:latest .
```

## 🚀 Deploy

### Local Deployment

```bash
# Install to system
cargo install --path creates/readual-cli

# Verify installation
readual --version
readual --help

# Create symbolic links
ln -s ~/.cargo/bin/readual /usr/local/bin/readual
```

### CI/CD Deployment

#### GitHub Actions

```bash
# Create workflow file
mkdir -p .github/workflows
cat > .github/workflows/ci.yml << 'EOF'
name: CI/CD Pipeline

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - name: Run tests
      run: cargo test --workspace
    - name: Run clippy
      run: cargo clippy -- -D warnings
    - name: Check formatting
      run: cargo fmt -- --check

  build:
    needs: test
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - name: Build
      run: cargo build --release
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: readual-${{ matrix.os }}
        path: target/release/
EOF
```

#### GitLab CI

```bash
# Create .gitlab-ci.yml
cat > .gitlab-ci.yml << 'EOF'
stages:
  - test
  - build
  - deploy

test:
  stage: test
  image: rust:1.75
  script:
    - cargo test --workspace
    - cargo clippy -- -D warnings
    - cargo fmt -- --check

build:
  stage: build
  image: rust:1.75
  script:
    - cargo build --release
  artifacts:
    paths:
      - target/release/
    expire_in: 1 week

deploy:
  stage: deploy
  script:
    - echo "Deploying to production..."
  only:
    - main
EOF
```

### Cloud Deployment

#### AWS Lambda

```bash
# Create Lambda function
cargo new readual-lambda --bin
cd readual-lambda

# Configure Cargo.toml for Lambda
cat >> Cargo.toml << 'EOF'
[package]
name = "readual-lambda"
version = "0.1.0"
edition = "2021"

[dependencies]
lambda_runtime = "0.7"
tokio = { version = "1", features = ["full"] }
EOF

# Build for Lambda
cargo build --release --target x86_64-unknown-linux-musl
```

#### Docker Hub

```bash
# Login to Docker Hub
docker login

# Tag image
docker tag readual:latest yourusername/readual:latest
docker tag readual:latest yourusername/readual:0.1.0

# Push to registry
docker push yourusername/readual:latest
docker push yourusername/readual:0.1.0
```

## 📖 Usage

### CLI Utility

```bash
# Show help
readual --help

# Show version
readual --version

# Analyze document structure
readual info

# Analyze with detailed information
readual info --verbose

# Analyze with various formats
readual info --format tree
readual info --format clean
readual info --format sections
readual info --format list
```

### Command Execution

```bash
# Show all available commands
readual run

# Execute specific command
readual run --path "Build::Basic Build::cargo build"

# Dry run (show what would be executed)
readual run --path "Test::Run Tests::cargo test" --dry

# Verbose mode
readual run --verbose
```

### MD Parser

```bash
# Parse file
readual-md --file README.md

# Parse with structure output
readual-md --file README.md --format tree

# Parse with level filtering
readual-md --file README.md --min-level 2
```

## 🏗️ Project Structure

### Components

- **`readual-cli`** - main CLI utility
- **`readual-md`** - Markdown file parser
- **`readual-command-info`** - information analysis command
- **`readual-command-run`** - script execution command

### Architecture

#### Modules

- **`parser`** - file parsing and command extraction
- **`analyzer`** - document structure analysis
- **`formatter`** - output formatting
- **`executor`** - command execution

#### Configuration

- Parser settings
- Formatting options
- Output parameters
- Environment variables

## 📝 Examples

### Basic Usage

```rust
use readual_md::{DocumentHierarchy, parse_markdown_file};

// Parse file
let hierarchy = parse_markdown_file("README.md")?;

// Display structure
for heading in &hierarchy.headings {
    println!("Level {}: {}", heading.level, heading.text);
}
```

### Advanced Features

```rust
use readual_command_run::{parse_commands_from_readme, execute_command};

// Extract commands from README
let commands = parse_commands_from_readme()?;

// Execute command
execute_command("cargo build", false)?;
```

### CI/CD Integration

```yaml
# .github/workflows/readual.yml
name: Readual Analysis
on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Install Readual
      run: cargo install --path creates/readual-cli
    - name: Analyze Repository
      run: readual info --verbose
    - name: Run Commands
      run: readual run --path "Test::Run Tests::cargo test"
```

## 🤝 Contributing

### How to Help

1. Fork the repository
2. Create a branch for new feature (`git checkout -b feature/amazing-feature`)
3. Make changes and commit them (`git commit -m 'Add amazing feature'`)
4. Push the branch (`git push origin feature/amazing-feature`)
5. Create a Pull Request

### Code Standards

- Use `cargo fmt` for formatting
- Run `cargo clippy` for style checking
- Cover new code with tests
- Follow Rust API Guidelines
- Document public APIs

### Development Process

```bash
# Create new branch
git checkout -b feature/new-feature

# Development with testing
cargo test
cargo clippy
cargo fmt

# Commit changes
git add .
git commit -m "feat: add new feature"

# Push to repository
git push origin feature/new-feature
```

## 📋 Changelog

### v0.1.0 (2024-01-XX)

#### ✨ New Features
- Basic CLI utility with `info` and `run` commands
- Markdown file parsing and heading hierarchy building
- Document structure display as file tree
- Command execution described in README.md
- Support for various output formats (tree, clean, sections, list)

#### 🐛 Fixes
- Improved error handling for file parsing
- Fixed nested structure display in tree format

#### 📚 Documentation
- Added detailed README with instructions for different platforms
- Created usage examples
- Added CI/CD integration documentation

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👥 Contacts

- **Author**: Nabiev Timur
- **Email**: nabievtimurprogrammer@gmail.com
- **GitHub**: [@your-username](https://github.com/your-username)
- **Project**: [readual](https://github.com/your-username/readual)

## 🙏 Acknowledgments

- Rust team for excellent programming language
- Cargo community for build tools
- All project contributors

---

**Made with ❤️ using Rust**