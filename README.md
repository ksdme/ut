# ut

A fast, lightweight CLI utility toolkit for developers and IT professionals. `ut` provides a comprehensive set of commonly-used tools in a single binary, eliminating the need to install and remember multiple utilities or search for random websites to perform simple tasks.

## Installation

```bash
cargo install --git https://github.com/ksdme/ut.git
```

## Usage

```bash
ut <TOOL> [OPTIONS]
```

Run `ut --help` to see all available tools, or `ut <TOOL> --help` for specific tool documentation.

## Available Tools

```
├── Encoding
│   ├── base64      - Base64 encode/decode
│   └── url         - URL encode/decode
├── Hashing
│   └── hash        - Cryptographic hash digests (MD5, SHA-1, SHA-256, etc.)
├── Data Generation
│   ├── uuid        - Generate UUIDs (v1, v3, v4, v5)
│   ├── token       - Generate secure random tokens
│   ├── lorem       - Generate lorem ipsum text
│   └── random      - Generate random numbers
├── Text Processing
│   ├── case        - Convert text case formats
│   ├── pretty-print - Unescape newlines and tabs
│   └── diff        - Compare text with visual output
├── Development Tools
│   ├── calc        - Expression calculator
│   ├── json        - JSON builder and utilities
│   ├── regex       - Interactive regex tester
│   └── datetime    - Parse and convert datetimes
├── Web & Network
│   ├── http-status - HTTP status code lookup
│   ├── serve       - Local HTTP file server
│   └── qr          - Generate QR codes
├── Color & Design
│   └── color       - Convert color formats
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
```

#### `url`
URL encode and decode text.

```bash
ut url encode "hello world"
ut url decode "hello%20world"
```

### Hashing

#### `hash`
Generate cryptographic hash digests using various algorithms.
- Supports MD5, SHA-1, SHA-224, SHA-256, SHA-384, and SHA-512
- Can read from files or stdin

```bash
ut hash sha256 "hello world"
ut hash md5 - < file.txt
```

### Data Generation

#### `uuid`
Generate UUIDs in various versions.
- v1: Timestamp-based
- v3: Namespace + MD5 hash
- v4: Random
- v5: Namespace + SHA-1 hash

```bash
ut uuid v4
ut uuid v4 --count 5
ut uuid v5 --namespace DNS --name example.com
```

#### `token`
Generate cryptographically secure random tokens.
- Customizable length and character sets
- Uses OS-level secure randomness

```bash
ut token --length 32
ut token --no-symbols --length 64
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
- lowercase, UPPERCASE, camelCase, snake_case, Title Case, CONSTANT_CASE, Header-Case, Sentence case

```bash
ut case lower "Hello World"
ut case camel "hello_world"
ut case snake "HelloWorld"
```

#### `pretty-print`
Resolve escaped newlines and tab characters in text.

```bash
ut pretty-print "hello\nworld\ttab"
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

#### `calc`
Expression calculator with support for multiple number formats and mathematical functions.
- Supports arithmetic operations, exponentiation, functions (sin, cos, tan, log, exp, sqrt, abs, floor, ceil, round)
- Binary (0b), hexadecimal (0x), and decimal number formats
- Mathematical constants (pi, e)
- Results displayed in decimal, hex, and binary

```bash
ut calc "2 + 2 * 3"
ut calc "sin(pi / 2)"
ut calc "0xFF + 0b1010"
ut calc "sqrt(16) ^ 2"
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

#### `datetime`
Parse and convert datetimes between timezones.
- Support for ISO 8601 and custom format strings
- Convert between timezones
- "now" keyword for current time

```bash
ut datetime now
ut datetime "2025-10-04T15:30:00Z" --target-timezone "Asia/Tokyo"
ut datetime "October 04, 2025 03:30 PM" --source-timezone UTC --parse-format "MonthName Day2, Year4 Hour12:Minute2 AMPM"
```

### Web & Network

#### `http-status`
Look up HTTP status codes and their descriptions.

```bash
ut http-status 404
ut http-status  # List all status codes
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
```

### Color & Design

#### `color`
Convert colors between different formats.
- Supports hex, rgb, rgba, hsl, hwb, lab, lch, oklab, oklch
- Parses any CSS-compatible color format

```bash
ut color "#FF5733"
ut color "rgb(255, 87, 51)"
ut color "hsl(9, 100%, 60%)"
```

### Reference

#### `unicode`
Display Unicode symbol reference table.

```bash
ut unicode
```

## Features

- **Fast**: Built in Rust for optimal performance
- **Standalone**: Single binary with no runtime dependencies
- **Composable**: Tools work with stdin/stdout for easy piping
- **Secure**: Uses cryptographically secure random number generators where appropriate
- **Cross-platform**: Works on Linux, macOS, and Windows

## Development

```bash
# Run the project
cargo run -- <tool> [args]

# Format code
cargo fmt

# Run tests
cargo test
```

## License

[Add your license here]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
