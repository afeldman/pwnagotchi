# Pwnagotchi Rust Architecture

This document describes the architecture of the Pwnagotchi Rust implementation.

## Overview

Pwnagotchi Rust is designed as a modular, async-first system using the Tokio runtime. The codebase is organized as a Cargo workspace with multiple crates, each responsible for a specific domain.

## Crate Organization

```
pwnagotchi/
├── pwnagotchi-core/         # Main agent logic
├── pwnagotchi-bettercap/    # Bettercap integration
├── pwnagotchi-automata/     # State machine & mood system
├── pwnagotchi-mesh/         # Peer-to-peer networking
├── pwnagotchi-ui/           # Display rendering
├── pwnagotchi-plugins/      # Plugin infrastructure
├── pwnagotchi-cli/          # CLI application
└── pwnagotchi-tools/        # Utility tools
```

## Core Components

### 1. Bettercap Client (`pwnagotchi-bettercap`)

**Purpose**: Communicate with the bettercap API server.

**Key Features**:

- HTTP REST API client using `reqwest`
- WebSocket event stream using `tokio-tungstenite`
- Session management
- Module control (start, stop, restart)

**Example**:

```rust
let client = BettercapClient::new("localhost", "http", 8081, "user", "pass")?;

// Run a command
client.run("wifi.recon on").await?;

// Get session info
let session = client.session().await?;

// Start event stream
let mut events = client.start_websocket().await?;
while let Some(event) = events.recv().await {
    // Process event
}
```

### 2. Automata (`pwnagotchi-automata`)

**Purpose**: Manage agent state and behavior through a mood system.

**Moods**:

- `Starting` - Initial startup
- `Ready` - Operational
- `Bored` - No activity for a while
- `Sad` - Extended inactivity
- `Angry` - Excessive inactivity
- `Excited` - High activity
- `Grateful` - Good peer support
- `Lonely` - No peer support
- `Rebooting` - System restart

**Epoch Tracking**:

```rust
pub struct Epoch {
    pub epoch: u64,              // Current epoch number
    pub inactive_for: u32,       // Epochs without activity
    pub active_for: u32,         // Epochs with activity
    pub num_handshakes: u32,     // Handshakes captured
    pub num_associations: u32,   // Association attempts
    pub num_deauths: u32,        // Deauth attempts
}
```

**State Transitions**:

```
Ready → Bored → Sad → Angry
  ↓       ↓      ↓      ↓
  ← Excited ← Grateful ←
```

### 3. Core Agent (`pwnagotchi-core`)

**Purpose**: Main agent logic orchestrating all components.

**Responsibilities**:

- WiFi monitoring via bettercap
- Access point and station tracking
- Handshake detection and storage
- Channel hopping
- Event processing
- Plugin coordination

**Flow**:

```
Start → Wait for Bettercap → Setup Events → Start Monitor Mode
  ↓
Event Loop:
  ├─ Process WiFi Events
  ├─ Update AP/Station Lists
  ├─ Detect Handshakes
  ├─ Update Automata State
  ├─ Trigger Plugins
  └─ Update Display
```

### 4. Mesh Networking (`pwnagotchi-mesh`)

**Purpose**: Enable peer-to-peer communication between pwnagotchi units.

**Components**:

- **Identity**: Ed25519 keypair for cryptographic identity
- **Advertisement**: Broadcast presence and stats
- **Peer Discovery**: Track nearby units
- **Signature Verification**: Ensure message authenticity

**Advertisement Format**:

```rust
pub struct MeshAdvertisement {
    pub fingerprint: String,     // First 8 bytes of public key
    pub name: String,            // Unit name
    pub identity: String,        // Full public key (hex)
    pub pwnd_run: u32,          // Handshakes this session
    pub pwnd_tot: u32,          // Total handshakes
    pub uptime: u64,            // Uptime in seconds
    pub version: String,        // Software version
    pub timestamp: i64,         // Unix timestamp
    pub signature: Vec<u8>,     // Ed25519 signature
}
```

### 5. UI System (`pwnagotchi-ui`)

**Purpose**: Render agent state to displays.

**Design**:

- Trait-based abstraction for different display types
- ViewState struct for current display data
- Mood-based face expressions
- Mock display for testing

**Display Trait**:

```rust
pub trait Display {
    fn init(&mut self) -> Result<()>;
    fn clear(&mut self) -> Result<()>;
    fn render(&mut self, state: &ViewState) -> Result<()>;
    fn update(&mut self) -> Result<()>;
}
```

### 6. Plugin System (`pwnagotchi-plugins`)

**Purpose**: Extensibility through async trait-based plugins.

**Hook Points**:

- `on_loaded()` - Plugin initialization
- `on_ready()` - Agent ready
- `on_handshake()` - Handshake captured
- `on_epoch()` - Epoch completed
- `on_wifi_update()` - WiFi list updated
- `on_channel_hop()` - Channel changed
- `on_peer_detected()` - Mesh peer found
- And 15+ more...

**Example Plugin**:

```rust
#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my-plugin" }

    async fn on_handshake(
        &mut self,
        filename: &str,
        ap: &AccessPoint,
        station: &Station,
    ) {
        println!("Captured: {} - {}", ap.essid, filename);
    }
}
```

## Data Flow

### Event Processing

```
Bettercap → WebSocket → Event Channel → Agent
                                          ↓
                                    Parse Event
                                          ↓
                                    Update State
                                          ↓
                           ┌──────────────┼──────────────┐
                           ↓              ↓              ↓
                      Update APs    Track Handshake  Update UI
                           ↓              ↓              ↓
                    Trigger Plugins  Save to Disk   Refresh Display
```

### Configuration Flow

```
config.toml → TOML Parser → AgentConfig
                                ↓
                    ┌───────────┼───────────┐
                    ↓           ↓           ↓
              MainConfig  BettercapConfig  PersonalityConfig
                    ↓           ↓           ↓
                Agent      Client      Automata
```

## Async Architecture

### Runtime: Tokio

All async operations use the Tokio runtime:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let mut agent = Agent::new(config)?;
    agent.start().await?;
    Ok(())
}
```

### Concurrency Model

- **Single-threaded event loop**: Main agent runs in one task
- **Parallel I/O**: Multiple WebSocket/HTTP requests concurrently
- **Channel-based communication**: `mpsc` channels for event passing
- **Async plugins**: All plugin hooks are async

### Task Spawning

```rust
// Event stream in separate task
tokio::spawn(async move {
    while let Some(event) = events.recv().await {
        process_event(event).await;
    }
});

// Main agent loop
loop {
    recon().await?;
    automata.next_epoch();
    update_display().await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

## Error Handling

### Strategy

- `anyhow::Result<T>` for application errors
- `thiserror` for custom error types
- Context propagation with `.context()`
- Graceful degradation where possible

### Example

```rust
pub async fn connect(&self) -> Result<()> {
    let session = self.session().await
        .context("Failed to get bettercap session")?;

    if !session.active {
        anyhow::bail!("Bettercap session is not active");
    }

    Ok(())
}
```

## Memory Safety

### Guarantees

- **No null pointers**: Option<T> instead
- **No data races**: Send + Sync traits
- **No use-after-free**: Ownership and borrowing
- **No buffer overflows**: Bounds checking

### Example

```rust
// Python equivalent might have race conditions
// Rust ensures thread safety at compile time
pub struct Agent {
    access_points: HashMap<String, AccessPoint>,  // Owned
    automata: Automata,                           // Owned
    bettercap: BettercapClient,                   // Owned
}

// Only one mutable reference at a time
fn update(&mut self) {
    self.access_points.clear();  // Exclusive access
}
```

## Performance Considerations

### Zero-Cost Abstractions

- Traits are monomorphized (no vtable overhead)
- Inline optimizations
- No garbage collector pauses

### Async Performance

- Non-blocking I/O
- Minimal context switching
- Efficient task scheduling

### Memory Usage

- Stack allocation where possible
- Minimal heap allocations
- Efficient data structures (HashMap, Vec)

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_epoch_tracking() {
        let mut epoch = Epoch::new();
        epoch.track_handshake();
        assert!(epoch.any_activity());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_bettercap_connection() {
    let client = BettercapClient::new(/* ... */)?;
    let session = client.session().await?;
    assert!(session.active);
}
```

### Mock Objects

```rust
pub struct MockDisplay {
    state: ViewState,
}

impl Display for MockDisplay {
    fn render(&mut self, state: &ViewState) -> Result<()> {
        self.state = state.clone();
        Ok(())
    }
}
```

## Future Enhancements

### Planned

1. **GPIO Display Drivers**: Real e-ink display support
2. **Web UI**: Browser-based interface
3. **Advanced Mesh**: Multi-hop routing
4. **AI Integration**: Reinforcement learning (optional)
5. **Cross-compilation**: Pre-built ARM binaries

### Under Consideration

1. **Alternative Backends**: Direct WiFi control without bettercap
2. **Plugin Hot-reloading**: Dynamic plugin loading
3. **Distributed Mode**: Multiple units coordinated
4. **Cloud Sync**: Optional backup to cloud storage

## References

- [Tokio Documentation](https://tokio.rs/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [Bettercap API](https://www.bettercap.org/api/)
- [Original Pwnagotchi](https://pwnagotchi.org/)
