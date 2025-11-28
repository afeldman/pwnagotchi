# Python Plugin Development Guide

## Overview

Pwnagotchi supports Python plugins through PyO3 integration. This allows you to write plugins in Python that are loaded and executed by the Rust core.

## Setup

### Install Development Dependencies

```bash
make py-install
```

This installs:

- **mypy** - Static type checker
- **ruff** - Fast Python linter (replaces flake8, isort, etc.)
- **black** - Code formatter
- **pytest** - Testing framework

## Creating a Plugin

### 1. Basic Structure

Create a new file in the `plugins/` directory:

```python
"""My custom plugin."""

from typing import Any, Dict
from pwnagotchi_plugin import Plugin


class MyPlugin(Plugin):
    """Example plugin description."""

    def __init__(self) -> None:
        """Initialize the plugin."""
        pass

    def name(self) -> str:
        """Return plugin name."""
        return "my_plugin"

    def version(self) -> str:
        """Return plugin version."""
        return "1.0.0"

    def description(self) -> str:
        """Return plugin description."""
        return "Does something cool"

    async def on_handshake(
        self,
        filename: str,
        access_point: Dict[str, Any],
        station: Dict[str, Any],
    ) -> None:
        """Handle handshake capture."""
        print(f"Captured: {filename}")
```

### 2. Available Hooks

All hooks are optional except `name()`, `version()`, and `description()`:

- **Lifecycle**: `on_loaded()`, `on_unload()`, `on_ready()`, `on_starting()`, `on_rebooting()`
- **Activity**: `on_handshake()`, `on_association()`, `on_deauthentication()`
- **State**: `on_epoch()`, `on_mood_change()`, `on_wifi_update()`
- **Network**: `on_peer_detected()`, `on_peer_lost()`, `on_internet_available()`
- **Events**: `on_bettercap_event()`, `on_channel_hop()`

### 3. Type Hints

Always use type hints for better IDE support and static analysis:

```python
from typing import Any, Dict, List

async def on_wifi_update(self, access_points: List[Dict[str, Any]]) -> None:
    """Process WiFi update."""
    for ap in access_points:
        essid: str = ap["essid"]
        channel: int = ap["channel"]
        rssi: int = ap["rssi"]
```

## Development Workflow

### Format Code

```bash
make py-format
```

This runs:

- **black** - Formats code to PEP 8 style
- **ruff** - Fixes auto-fixable lint issues

### Check Code Quality

```bash
make py-lint
```

Runs **ruff** to check for:

- Code style issues
- Potential bugs
- Security issues
- Best practice violations

### Type Check

```bash
make py-type
```

Runs **mypy** in strict mode to check:

- Type annotations
- Type consistency
- Missing return types

### Run Tests

```bash
make py-test
```

Runs **pytest** on all tests in `tests/` directory.

### Run All Checks

```bash
make py-all
```

Formats, lints, type-checks, and tests in one command.

## Testing

Create tests in `tests/test_*.py`:

```python
"""Tests for my plugin."""

import pytest
from plugins.my_plugin import MyPlugin


@pytest.mark.asyncio
async def test_basic() -> None:
    """Test basic plugin functionality."""
    plugin = MyPlugin()

    assert plugin.name() == "my_plugin"
    assert plugin.version() == "1.0.0"


@pytest.mark.asyncio
async def test_handshake() -> None:
    """Test handshake handling."""
    plugin = MyPlugin()

    ap = {"bssid": "aa:bb:cc:dd:ee:ff", "essid": "Test", "channel": 6, "rssi": -45}
    sta = {"mac": "11:22:33:44:55:66", "rssi": -60}

    await plugin.on_handshake("/tmp/test.pcap", ap, sta)
```

## Static Analysis Configuration

### Ruff (pyproject.toml)

Ruff is configured with strict rules in `pyproject.toml`:

- Code style (PEP 8)
- Security checks (bandit)
- Bug detection
- Modern Python idioms
- Import sorting

### Mypy (pyproject.toml)

Mypy is configured in strict mode:

- All functions must have type hints
- No implicit `Optional`
- No `Any` without explicit annotation
- Check untyped definitions

### Black (pyproject.toml)

Black formatting:

- Line length: 100
- Target: Python 3.10+

## Example Plugins

### Handshake Logger

See `plugins/handshake_logger.py` for a complete example that:

- Counts handshakes
- Logs to file
- Tracks mood changes

### Stats Display

See `plugins/stats_display.py` for a complete example that:

- Tracks WiFi statistics
- Displays periodic summaries
- Monitors channel hopping

## Loading Plugins

Plugins are loaded automatically from the `plugins/` directory by the Rust core:

```rust
use pwnagotchi_py::PythonPlugin;

let plugin = PythonPlugin::from_file("plugins/my_plugin.py", "MyPlugin").await?;
```

## Best Practices

1. **Always use type hints** - Enables static analysis and better IDE support
2. **Handle errors gracefully** - Don't crash the agent
3. **Use async/await** - All hooks are async
4. **Log with context** - Prefix logs with plugin name: `[my_plugin]`
5. **Test your plugin** - Write tests before deploying
6. **Follow PEP 8** - Use `make py-format` to auto-format
7. **Document your code** - Use docstrings for all functions
8. **Keep plugins focused** - One plugin, one purpose

## Troubleshooting

### Import Errors

Make sure `pwnagotchi_plugin.py` is in your Python path or in the same directory.

### Type Errors

Run `make py-type` to see detailed type errors and fix them.

### Lint Errors

Run `make py-lint` and fix issues. Use `make py-format` to auto-fix many issues.

### Plugin Not Loading

Check:

1. Plugin file is in `plugins/` directory
2. Class name matches what's specified in config
3. Plugin inherits from `Plugin` base class
4. Required methods (`name`, `version`, `description`) are implemented

## CI/CD Integration

Add to your CI pipeline:

```yaml
- name: Check Python plugins
  run: |
    pip install -r requirements-dev.txt
    make py-all
```

This ensures all plugins pass linting, type checking, and tests before deployment.
