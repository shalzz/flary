# flary

[![Crates.io](https://img.shields.io/crates/d/flary)](https://crates.io/crates/flary)
[![Crates.io](https://img.shields.io/crates/v/flary)](https://crates.io/crates/flary)
[![Crates.io](https://img.shields.io/crates/l/flary)](https://crates.io/crates/flary)

A CLI for managing Domains and DNS records on Cloudflare.

## Installation

### From crates.io

```bash
cargo install flary
```

### From source

```bash
git clone https://github.com/shalzz/flary
cd flary
cargo install --path .
```

## Authentication

flary reads credentials from the following sources, in order of priority:

1. **Environment variables** (highest priority)
2. **Local `wrangler.toml`** in the current directory
3. **Global wrangler config** at `~/.wrangler/config/default.toml`

### Environment variables

| Variable | Description |
|---|---|
| `CF_API_TOKEN` | Cloudflare API token (preferred) |
| `CF_API_KEY` | Global API key (requires `CF_EMAIL`) |
| `CF_EMAIL` | Account email address |

### Wrangler config

flary reads `api_token` or `oauth_token` from wrangler's config files, preferring `api_token` when both are present. The global config path can be overridden with `WRANGLER_HOME`.

Example wrangler config:

```toml
oauth_token = "your-oauth-token"
refresh_token = "your-refresh-token"
expiration_time = "2026-01-01T00:00:00Z"
scopes = ["zone:read", "dns_records:read", "dns_records:edit"]
```

### OAuth login

If you don't have a token, use the built-in OAuth flow:

```sh
flary config auth
```

This opens a browser for Cloudflare consent with DNS read/write scopes (`zone:read`, `dns_records:read`, `dns_records:edit`). After authorizing, the token is saved to `~/.wrangler/config/default.toml` and is compatible with wrangler.

## Shell completion

```bash
flary completions bash > ~/.local/share/bash-completion/completions/flary
```

## Commands

```
flary config auth                  # Authenticate via OAuth
flary domains ls                   # List all domains
flary dns ls <domain>              # List DNS records for a domain
flary dns add <domain> <name> <type> <value> [--proxied] [--ttl N] [--priority N]
flary dns update <id> <domain> <name> <type> <value> [--proxied] [--ttl N]
flary dns rm <id> <domain> [--yes]
```

### DNS record types

A, AAAA, CNAME, TXT, MX, SRV, NS

## Usage

### List domains

```bash
flary domains ls
```

Lists all domains (zones) on your Cloudflare account.

### List DNS records

```bash
flary dns ls example.com
```

Lists all DNS records for a domain.

### Add a DNS record

```bash
flary dns add <domain> <name> <type> <value> [OPTIONS]
```

| Argument  | Description                                         |
|-----------|-----------------------------------------------------|
| `domain`  | The domain to add the record to                     |
| `name`    | Record name (e.g. `www`, `@` for root)              |
| `type`    | Record type: `A`, `AAAA`, `CNAME`, `TXT`, `MX`, `SRV`, `NS` |
| `value`   | Record value                                        |

| Option         | Description                                         |
|----------------|-----------------------------------------------------|
| `--proxied`    | Enable Cloudflare proxy (orange cloud)              |
| `--ttl <sec>`  | Time to live in seconds (`1` = automatic, default)  |
| `--priority`   | Priority for MX/SRV records                         |

#### Examples

Add an A record:

```bash
flary dns add example.com www A 192.0.2.1
```

Add a proxied CNAME record:

```bash
flary dns add example.com blog CNAME my-app.pages.dev --proxied
```

Add an MX record with priority:

```bash
flary dns add example.com @ MX mail.example.com --priority 10
```

Add a TXT record:

```bash
flary dns add example.com @ TXT "v=spf1 include:_spf.google.com ~all"
```

### Update a DNS record

```bash
flary dns update <id> <domain> <name> <type> <value> [OPTIONS]
```

Requires the record ID (obtainable from `dns ls`). The `--proxied` and `--ttl` flags work the same as `dns add`.

#### Example

Update an A record's IP address:

```bash
flary dns update abc123 example.com www A 192.0.2.2
```

Update and enable proxying:

```bash
flary dns update abc123 example.com www A 192.0.2.2 --proxied --ttl 300
```

### Remove a DNS record

```bash
flary dns rm <id> <domain> [OPTIONS]
```

Prompts for confirmation before deleting. Use `--yes` to skip the prompt.

#### Examples

Interactive delete:

```bash
flary dns rm abc123 example.com
```

Skip confirmation:

```bash
flary dns rm abc123 example.com --yes
```

## Development

```bash
# Build
cargo build

# Run tests (unit + integration)
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- domains ls
```

## License

MIT OR Apache-2.0
