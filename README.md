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

## Configuration

flary authenticates with the Cloudflare API using credentials stored in your Wrangler config file (`~/.wrangler/config/default.toml`) or via environment variables.

### Environment variables

| Variable      | Description                        |
|---------------|------------------------------------|
| `CF_API_TOKEN`| Cloudflare API token               |
| `CF_API_KEY`  | Cloudflare API key (global)        |
| `CF_EMAIL`    | Cloudflare account email (required with `CF_API_KEY`) |

Environment variables take priority over the config file. You can also set `WRANGLER_HOME` to override the default config directory (`~/.wrangler`).

### Config file

Run `wrangler login` or create `~/.wrangler/config/default.toml` manually:

```toml
api_token = "your-cloudflare-api-token"
```

OAuth tokens from `wrangler login` are also supported:

```toml
oauth_token = "your-oauth-token"
```

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

## Supported record types

| Type   | Description                     |
|--------|---------------------------------|
| `A`    | IPv4 address                    |
| `AAAA` | IPv6 address                    |
| `CNAME`| Canonical name (alias)          |
| `MX`   | Mail exchange                   |
| `NS`   | Name server                     |
| `SRV`  | Service locator                 |
| `TXT`  | Text record                     |

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
