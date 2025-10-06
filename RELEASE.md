# Release Process

This project uses [cargo-dist](https://github.com/axodotdev/cargo-dist) for automated releases.

## Creating a new release

1. **Update version in Cargo.toml**

   ```bash
   # Edit Cargo.toml and update the version number
   version = "0.1.0" → version = "0.2.0"
   ```

2. **Commit and tag**

   ```bash
   git add Cargo.toml
   git commit -m "chore: bump version to 0.2.0"
   git tag v0.2.0
   git push origin main
   git push origin v0.2.0
   ```

3. **cargo-dist will automatically:**
   - Build binaries for 5 platforms:
     - macOS (Intel: `x86_64-apple-darwin`)
     - macOS (ARM: `aarch64-apple-darwin`)
     - Linux (x86_64: `x86_64-unknown-linux-gnu`)
     - Linux (ARM64: `aarch64-unknown-linux-gnu`)
     - Windows (x64: `x86_64-pc-windows-msvc`)
   - Create installer scripts (shell, PowerShell)
   - Generate a GitHub Release with all artifacts
   - Attach `.tar.gz` and `.zip` archives

## Testing the release process

Before pushing a tag, you can test locally:

```bash
# See what will be built
cargo dist plan

# Build for current platform
cargo dist build
```

## Testing the binary

```bash
# Build and test the binary
cargo build --release
./target/release/ut --help
./target/release/ut base64 encode "test"
```

## Quick release script

You can automate with:

```bash
#!/bin/bash
VERSION=$1
if [ -z "$VERSION" ]; then
  echo "Usage: ./release.sh 0.2.0"
  exit 1
fi

# Update Cargo.toml
sed -i '' "s/version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Commit and tag
git add Cargo.toml
git commit -m "chore: bump version to $VERSION"
git tag "v$VERSION"
git push origin main
git push origin "v$VERSION"

echo "Release v$VERSION pushed. Check GitHub Actions for build status."
```

## Configuration

Release configuration is in `dist-workspace.toml`. To modify:

```bash
# Edit dist-workspace.toml, then regenerate CI workflow
cargo dist generate
```

Available options:

- `installers`: Add/remove installer types (shell, powershell, homebrew, npm, etc.)
- `targets`: Add/remove build platforms
- `ci`: Configure CI behavior (GitHub Actions)

See [cargo-dist documentation](https://opensource.axo.dev/cargo-dist/) for more options.
