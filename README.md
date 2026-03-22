# Shimmy - MCP Inspector

<p align="center">
  <img src="media/shimmy-icon-round.png" alt="Shimmy Logo" width="256" height="256">
</p>

Shimmy is a debugging and inspection tool for [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) servers. It intercepts traffic between MCP clients and servers, letting you see every request, response, and notification in real time.

## How It Works

Shimmy has two components:

- **Shimmy App** - A desktop GUI (built with Tauri + SvelteKit) that displays captured MCP traffic in a three-panel inspector view.
- **Shimmy CLI** - A command-line proxy that sits between an MCP client and server, forwarding all messages to the Shimmy App for inspection.

```
MCP Client <──> Shimmy CLI (proxy) <──> MCP Server
                     │
                     ▼
               Shimmy App (inspector UI)
```

The CLI intercepts all JSON-RPC messages and sends them to the app over HTTP (`127.0.0.1:13579`). The app displays them in real time with a timeline view, request details, and response details.

## Installation

Download the latest release for your platform from the [GitHub Releases](https://github.com/chihlee/shimmy/releases) page. Each release includes:

- **Shimmy App** - Desktop application installer (macOS, Linux, Windows)
- **Shimmy CLI** - Standalone binary

Alternatively, you can [build from source](#building-from-source).

## Getting Started

### 1. Launch the Shimmy App

Open the Shimmy desktop application. It starts an HTTP listener on port `13579` and waits for incoming connections from the CLI.

### 2. Run the CLI to inspect an MCP server

#### Stdio transport

For MCP servers that communicate over standard input/output (the most common case):

```bash
shimmy-cli stdio <server_command> [server_args...]
```

**Examples:**

```bash
# Inspect a Python MCP server
shimmy-cli stdio python my_server.py

# Inspect a Node.js MCP server with arguments
shimmy-cli stdio node server.js --port 3000

# Pass environment variables to the server
shimmy-cli -e API_KEY=sk-xxx -e DEBUG=true stdio python my_server.py
```

#### HTTP transport

For MCP servers that communicate over HTTP/SSE:

```bash
shimmy-cli http <url>
```

**Example:**

```bash
shimmy-cli http http://127.0.0.1:8080
```

### 3. Inspect traffic in the App

Once the CLI connects, the Shimmy App shows all MCP traffic in real time:

- **Timeline panel** (left) - A chronological list of all messages, color-coded by origin (client vs. server) and status (success, error, pending, notification).
- **Request panel** (center) - Full JSON payload of the selected request, plus any stderr output.
- **Response panel** (right) - The corresponding response JSON.

You can filter messages by method name using the search bar and switch between multiple active connections using the connection selector.

## CLI Reference

```
Usage: shimmy-cli [OPTIONS] <COMMAND>

Commands:
  stdio  Spawn the MCP server via standard input/output
  http   Spawn the MCP server over HTTP/SSE

Options:
  -v, --verbose  Enable verbose logging
  -h, --help     Print help
  -V, --version  Print version
```

### `shimmy-cli stdio`

```
Usage: shimmy-cli stdio [OPTIONS] <SERVER_COMMAND> [SERVER_ARGS]...

Arguments:
  <SERVER_COMMAND>  The executable command to run the MCP server (e.g., 'node', 'python')
  [SERVER_ARGS]...  Arguments and flags to pass to the server command

Options:
  -e, --env <KEY=VALUE>  Environment variables to pass to the server (format: KEY=VALUE)
```

### `shimmy-cli http`

```
Usage: shimmy-cli http <URL>

Arguments:
  <URL>  URL of the MCP server (e.g., http://127.0.0.1:8080)
```

## Using Shimmy with MCP Clients

To use Shimmy with an MCP client (such as Claude Desktop), replace your server command with `shimmy-cli` wrapping the original command. For example, if your MCP client config looks like:

```json
{
  "mcpServers": {
    "my-server": {
      "command": "python",
      "args": ["my_server.py"]
    }
  }
}
```

Change it to:

```json
{
  "mcpServers": {
    "my-server": {
      "command": "shimmy-cli",
      "args": ["stdio", "python", "my_server.py"]
    }
  }
}
```

Now all traffic between the client and server will be visible in the Shimmy App.

## Supported MCP Methods

Shimmy proxies and inspects MCP protocol messages. Below is the current coverage of MCP methods.

### Client-to-Server Requests

| Method | Status |
|---|---|
| `initialize` | Supported |
| `ping` | Supported |
| `tools/list` | Supported |
| `tools/call` | Supported |
| `resources/list` | Supported |
| `resources/read` | Supported |
| `prompts/list` | Supported |
| `prompts/get` | Not yet supported |
| `resources/templates/list` | Not yet supported |
| `resources/subscribe` | Not yet supported |
| `resources/unsubscribe` | Not yet supported |
| `completion/complete` | Not yet supported |
| `logging/setLevel` | Not yet supported |

### Client-to-Server Notifications

| Method | Status |
|---|---|
| `notifications/initialized` | Supported |
| `notifications/cancelled` | Not yet supported |
| `notifications/progress` | Not yet supported |
| `notifications/roots/list_changed` | Not yet supported |

### Server-to-Client Requests

| Method | Status |
|---|---|
| `ping` | Supported |
| `elicitation/create` | Supported |
| `sampling/createMessage` | Not yet supported |
| `roots/list` | Not yet supported |

### Server-to-Client Notifications

| Method | Status |
|---|---|
| `notifications/tools/list_changed` | Supported |
| `notifications/cancelled` | Not yet supported |
| `notifications/progress` | Not yet supported |
| `notifications/message` | Not yet supported |
| `notifications/resources/updated` | Not yet supported |
| `notifications/resources/list_changed` | Not yet supported |
| `notifications/prompts/list_changed` | Not yet supported |

## Building from Source

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Node.js](https://nodejs.org/) (for the frontend)
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

### Build the CLI

```bash
cargo build --release -p shimmy-cli
```

The binary will be at `target/release/shimmy-cli`.

### Build the App

```bash
cd tauri-app
npm install
npm run tauri build
```

## License

MIT
