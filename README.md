# `ut`

A fast, lightweight CLI **utility toolkit** for developers and IT professionals. `ut` provides a comprehensive set of commonly-used tools in a single binary, eliminating the need to install and remember multiple utilities or search for random websites to perform simple tasks.

## Installation

<details open>
<summary>Install on <b>macOS</b> or Linux via homebrew</summary>

```bash
brew install ksdme/tap/ut
```

</details>

<details open>
<summary>Install on <b>Linux</b> or macOS via shell script</summary>

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ksdme/ut/releases/latest/download/ut-installer.sh | sh
```

</details>

<details open>
<summary>Install prebuilt binaries on <b>Windows</b> via powershell</summary>

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/ksdme/ut/releases/latest/download/ut-installer.ps1 | iex"
```

</details>

<details>
<summary>Install from source</summary>

```bash
cargo install --git https://github.com/ksdme/ut.git
```

</details>


You can also download prebuilt binaries directly from the [releases page](https://github.com/ksdme/ut/releases/).

## Shell Completions

`ut` supports shell completions for bash, zsh, fish, nushell, elvish, and PowerShell. To enable tab completion:

<details>
<summary><b>Zsh</b></summary>

```bash
# Generate and save completions to your fpath
ut completions zsh > ~/.zsh/completions/_ut

# Or add directly to .zshrc (requires reload for each ut update)
echo 'eval "$(ut completions zsh)"' >> ~/.zshrc
```

</details>

<details>
<summary><b>Bash</b></summary>

```bash
ut completions bash > ~/.local/share/bash-completion/completions/ut
# Or add to .bashrc
echo 'eval "$(ut completions bash)"' >> ~/.bashrc
```

</details>

<details>
<summary><b>Fish</b></summary>

```bash
ut completions fish > ~/.config/fish/completions/ut.fish
```

</details>

<details>
<summary><b>Nushell</b></summary>

```bash
ut completions nushell > ~/.config/nushell/completions/ut.nu
# Or use the short alias
ut completions nu > ~/.config/nushell/completions/ut.nu
```

</details>

<details>
<summary><b>PowerShell</b></summary>

```powershell
ut completions powershell | Out-File -FilePath $PROFILE -Append
```

</details>

After setting up completions, restart your shell or source your configuration file.

## Available Tools

```
├── Encoding
│   ├── base64      - Base64 encode/decode
│   │   ├── encode
│   │   └── decode
│   └── url         - URL utilities
│       ├── parse
│       ├── encode
│       └── decode
├── Hashing & Security
│   ├── hash        - Cryptographic hash digests
│   │   ├── md5
│   │   ├── sha1
│   │   ├── sha224
│   │   ├── sha256
│   │   ├── sha384
│   │   └── sha512
│   ├── bcrypt      - Password hashing and verification
│   │   ├── hash
│   │   └── verify
│   └── jwt         - JWT (JSON Web Token) utilities
│       ├── encode
│       ├── decode
│       └── verify
├── Data Generation
│   ├── uuid        - Generate UUIDs
│   │   ├── v1
│   │   ├── v3
│   │   ├── v4
│   │   ├── v5
│   │   └── v7
│   ├── token (secret, password) - Generate secure random tokens
│   ├── lorem       - Generate lorem ipsum text
│   └── random      - Generate random numbers
├── Text Processing
│   ├── case        - Convert text case formats
│   │   ├── lower
│   │   ├── upper
│   │   ├── camel
│   │   ├── title
│   │   ├── constant
│   │   ├── header
│   │   ├── sentence
│   │   ├── snake
│   │   └── kebab
│   ├── pretty-print (pp) - Unescape newlines and tabs
│   └── diff        - Compare text with visual output
├── Development Tools
│   ├── calc (cal)  - Expression calculator
│   ├── json        - JSON builder and utilities
│   │   └── builder
│   ├── regex       - Interactive regex tester
│   ├── crontab     - Cron utilities
│   │   └── schedule
│   └── datetime (dt) - Parse and convert datetimes
├── Web & Network
│   ├── ip          - IP & CIDR utilities
│   │   └── cidr
│   │       └── describe
│   ├── http        - HTTP utilities
│   │   └── status
│   ├── proc        - Process and port utilities
│   │   ├── list
│   │   ├── name
│   │   ├── pid
│   │   ├── port
│   │   └── ports
│   ├── serve       - Local HTTP file server
│   └── qr          - Generate QR codes
├── Color & Design
│   └── color       - Color utilities
│       └── convert
└── Reference
    └── unicode     - Unicode symbol reference
```

### Encoding

#### `base64`
Encode and decode data using Base64 encoding.
- Supports both standard and URL-safe character sets
- Can read from files or stdin

```bash
ut base64 encode "hello world"
ut base64 decode "aGVsbG8gd29ybGQ="
ut base64 encode --urlsafe "hello world"
echo -n "hello world" | ut base64 encode -
```

#### `url`
URL encode, decode, and parse.

```bash
ut url parse "https://example.com:8080/path?key=value#section"
ut url encode "hello world"
ut url decode "hello%20world"
printf "hello world" | ut url encode -
```

### Hashing & Security

#### `hash`
Generate cryptographic hash digests using various algorithms.
- Supports MD5, SHA-1, SHA-224, SHA-256, SHA-384, and SHA-512
- Can read from files or stdin

```bash
ut hash sha256 "hello world"
ut hash md5 - < file.txt
echo -n "password" | ut hash sha256 -
```

#### `bcrypt`
Hash and verify passwords using the bcrypt algorithm.
- Secure password hashing with configurable cost factor
- Built-in salt generation for each hash
- Verification returns "valid" or "invalid"
- Cost factor range: 4-31 (default: 12, higher = more secure but slower)

```bash
# Hash a password with default cost (12)
ut bcrypt hash "mypassword"
echo -n "mypassword" | ut bcrypt hash -

# Wrong password verification
ut bcrypt verify "wrongpassword" '$2b$12$...'
# Output: invalid
```

#### `jwt`
JWT (JSON Web Token) utilities for encoding, decoding, and verifying tokens.
- Support for HMAC algorithms (HS256, HS384, HS512)
- Encode with custom claims (iss, sub, aud, exp)
- Decode without verification (inspect token)
- Verify with signature validation

```bash
# Encode a JWT with custom claims
ut jwt encode --payload '{"user":"alice"}' --secret "my-secret" --issuer "my-app" --expires-in 3600

# Decode a JWT without verification
ut jwt decode eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...

# Verify a JWT with signature validation
ut jwt verify TOKEN --secret "my-secret" --issuer "my-app"
```

### Data Generation

#### `uuid`
Generate UUIDs in various versions.
- v1: Timestamp-based
- v3: Namespace + MD5 hash
- v4: Random
- v5: Namespace + SHA-1 hash
- v7: Timestamp-based, sortable

```bash
ut uuid v4
ut uuid v4 --count 5
ut uuid v5 --namespace DNS --name example.com
ut uuid v7
ut uuid v7 --count 5
```

#### `token` (aliases: `secret`, `password`)
Generate cryptographically secure random tokens.
- Customizable length and character sets
- Uses OS-level secure randomness
- Can be used for passwords, API keys, session tokens, etc.

```bash
# Generate a 32-character token
ut token --length 32

# Generate a password (using alias)
ut password --length 16

# Generate without symbols
ut secret --no-symbols --length 64

# Generate without uppercase letters
ut token --no-uppercase --length 20
```

#### `lorem`
Generate lorem ipsum placeholder text.
- Customizable paragraph count and sentence structure

```bash
ut lorem --paragraphs 5
ut lorem --min-sentences 2 --max-sentences 6
```

#### `random`
Generate random numbers within a specified range.
- Supports decimal precision with step parameter
- Can generate multiple values at once

```bash
ut random --min 1 --max 100
ut random --min 0 --max 1 --step 0.01 --count 10
```

### Text Processing

#### `case`
Convert text between different case formats.
- lowercase, UPPERCASE, camelCase, snake_case, kebab-case, Title Case, CONSTANT_CASE, Header-Case, Sentence case

```bash
ut case lower "Hello World"
ut case camel "hello_world"
ut case snake "HelloWorld"
ut case kebab "HelloWorld"
echo -n "Hello :)" | ut case lower -
```

#### `pretty-print` (alias: `pp`)
Resolve escaped newlines and tab characters in text.

```bash
ut pretty-print "hello\nworld\ttab"
ut pp "hello\nworld\ttab"
```

#### `diff`
Compare text contents with visual diff output.
- Supports file comparison or interactive editing
- Color-coded character-level differences

```bash
ut diff -a file1.txt -b file2.txt
ut diff  # Opens editor for both inputs
```

### Development Tools

#### `calc` (alias: `cal`)
Expression calculator with support for multiple number formats and mathematical functions.
- Supports arithmetic operations, exponentiation, functions (sin, cos, tan, log, exp, sqrt, abs, floor, ceil, round)
- Binary (0b), hexadecimal (0x), and decimal number formats
- Mathematical constants (pi, e)
- Results displayed in decimal, hex, and binary

```bash
ut calc "2 + 2 * 3"
ut cal "sin(pi / 2)"
ut calc "0xFF + 0b1010"
ut calc "sqrt(16) ^ 2"
echo -n "2 + 2" | ut calc -
```

#### `json`
JSON utilities including a powerful JSON builder.
- Build complex JSON structures using dot notation
- Supports nested objects and arrays
- Array indexing and append operations

```bash
ut json builder a.b.c=hello a.b.d=world
ut json builder "user.name=John" "user.age=30" "user.tags[]=dev" "user.tags[]=rust"
ut json builder "items[0].id=1" "items[0].name=first" "items[1].id=2"
```

#### `regex`
Interactive regex tester with live highlighting.
- Real-time pattern matching visualization
- Multi-color highlighting for capture groups
- Load test strings from files

```bash
ut regex
ut regex --test sample.txt
```

#### `crontab`
Parse crontab expressions and show upcoming firing times.
- Support for traditional 5-field cron expressions
- Support for extended 6-field cron expressions (with seconds)
- Configurable number of results with -n / --count
- Configurable start time with -a / --after

```bash
ut crontab schedule "0 9 * * 1-5"
ut crontab schedule "0 0 * * *" --count 3
ut crontab schedule "0 9 * * 1-5" --after "2024-01-01T00:00:00Z"
echo -n "0 9 * * 1-5" | ut crontab schedule -
```

#### `datetime` (alias: `dt`)
Parse and convert datetimes between timezones.
- Support for ISO 8601 and custom format strings
- Convert between timezones
- "now" keyword for current time

```bash
ut datetime now
ut dt "2025-10-04T15:30:00Z" --target-timezone "Asia/Tokyo"
ut datetime "October 04, 2025 03:30 PM" --source-timezone UTC --parse-format "MonthName Day2, Year4 Hour12:Minute2 AMPM"
echo -n "2025-10-04T15:30:00Z" | ut datetime -
```

### Web & Network

#### `ip`
IP and CIDR utilities including a tool to describe CIDR blocks.

```bash
ut ip cidr describe 192.168.1.100/24
ut ip cidr describe 10.0.0.0/8
ut ip cidr describe 172.16.0.1/16
```

#### `http`
HTTP utilities including status code lookup.

```bash
ut http status 404
ut http status  # List all status codes
```

#### `proc`
Query running processes and find which process owns a given TCP port.
- List all running processes, optionally filtered by name
- Look up a specific process by PID
- Identify which process is listening on a port
- List all open TCP listening ports with their owning process
- Cross-platform: Linux (reads `/proc/net/tcp` directly), macOS (uses `lsof`), Windows (uses `netstat`)
- Processes owned by other users may require elevated privileges

```bash
# List all running processes
ut proc list

# Filter by name (case-insensitive substring match)
ut proc list --name node
ut proc name nginx

# Inspect a process by PID
ut proc pid 1234

# Find what's listening on port 3000
ut proc port 3000

# List every open TCP listening port with its owning process
ut proc ports
```

#### `serve`
Start a local HTTP file server.
- Customizable host and port
- Directory listing support
- Optional HTTP Basic authentication

```bash
ut serve --port 8080
ut serve --directory ./public --auth username:password
```

#### `qr`
Generate QR codes.
- Terminal display or save to PNG file

```bash
ut qr "https://example.com"
ut qr "Hello World" --output qrcode.png
echo -n "Hello World" | ut qr -
```

### Color & Design

#### `color`
Color utilities for working with different color formats.
- Supports hex, rgb, rgba, hsl, hwb, lab, lch, oklab, oklch
- Parses any CSS-compatible color format

```bash
ut color convert "#FF5733"
ut color convert "rgb(255, 87, 51)"
ut color convert "hsl(9, 100%, 60%)"
printf "#FF5733" | ut color convert -
```

### Reference

#### `unicode`
Display Unicode symbol reference table.

```bash
ut unicode
```

## Nice to have

- crontab explainer

## Development

```bash
# Run the project
cargo run -- <tool> [args]

# Format code
cargo fmt

# Run tests
cargo test
```
## Built with Claude Code

Parts of this project were built using [Claude Code](https://claude.com/claude-code), an AI-powered coding assistant, with human oversight and collaboration.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
