# Coval CLI

Command-line interface for the [Coval](https://coval.dev) AI evaluation platform.

## Installation

### Homebrew (macOS/Linux)

```bash
brew install coval-ai/tap/coval
```

### Cargo

```bash
cargo install coval
```

### Binary

Download pre-built binaries from [Releases](https://github.com/coval-ai/cli/releases).

## Quick Start

```bash
# Authenticate
coval login

# List your agents
coval agents list

# Launch an evaluation run
coval runs launch \
  --agent-id <agent_id> \
  --persona-id <persona_id> \
  --test-set-id <test_set_id>

# Check run status
coval runs get <run_id>

# List simulations for a run
coval simulations list --run-id <run_id>
```

## Commands

| Command | Description |
|---------|-------------|
| `coval login` | Authenticate with Coval |
| `coval whoami` | Show current authentication |
| `coval agents` | Manage AI agent configurations |
| `coval runs` | Launch and manage evaluation runs |
| `coval simulations` | View individual simulation results |
| `coval test-sets` | Manage test set collections |
| `coval test-cases` | Manage individual test cases |
| `coval personas` | Manage simulated personas |
| `coval metrics` | Manage evaluation metrics |
| `coval traces` | Configure and validate Coval tracing |
| `coval mutations` | Test agent variations with config overrides |
| `coval api-keys` | Manage API keys |
| `coval run-templates` | Save reusable evaluation configurations |
| `coval scheduled-runs` | Schedule recurring evaluation runs |
| `coval dashboards` | Manage dashboards and widgets |
| `coval config` | Manage CLI configuration |

### Common Flags

| Flag | Description |
|------|-------------|
| `--format json` | Output as JSON (default: table) |
| `--api-key` | Override API key |
| `--help` | Show help |

## Examples

### Setup Traces

```bash
# LiveKit Agents or Pipecat
coval traces setup

# Force generic mode for other Python voice agents
coval traces setup --framework generic
```

`coval traces setup` is currently framework-aware for `LiveKit Agents` and `Pipecat` only.
Other Python voice agents are not currently validated and may require manual instrumentation,
even when using `--framework generic`.

### Launch a Run

```bash
# Basic run
coval runs launch \
  --agent-id abc123 \
  --persona-id xyz789 \
  --test-set-id ts123456

# With options
coval runs launch \
  --agent-id abc123 \
  --persona-id xyz789 \
  --test-set-id ts123456 \
  --iterations 3 \
  --concurrency 5 \
  --name "Regression Test"
```

### Create Resources

```bash
# Create a voice agent
coval agents create \
  --name "Support Agent" \
  --type voice \
  --phone-number "+15551234567"

# Create a test set
coval test-sets create \
  --name "Customer Support Scenarios" \
  --type SCENARIO

# Create a test case
coval test-cases create \
  --test-set-id ts123456 \
  --input "I need help with my order"
```

### JSON Output for Scripting

```bash
# Get run as JSON
coval runs get abc123 --format json | jq '.status'

# List agents as JSON
coval agents list --format json | jq '.[].id'
```

## Configuration

Config file: `~/.config/coval/config.toml`

```toml
api_key = "sk_..."
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `COVAL_API_KEY` | API key (overrides config file) |

## License

MIT - see [LICENSE](LICENSE)
