# SoundCloud Desktop - Capability Table & Integration Plan

**Reference Project**: zxcloli666/SoundCloud-Desktop (MIT License)
**Our Project**: Minimal Tauri 2 + WebView2 SoundCloud Desktop App
**Date**: 2026-11-15

---

## Executive Summary

| Module | Priority | Complexity | Dependencies | Estimated Effort | Notes |
|--------|----------|------------|--------------|------------------|-------|
| **System Tray** | 🔴 Critical | Medium | `tauri-plugin-tray-icon` | 1-2 days | Foundational - enables minimize-to-tray |
| **Media Controls (SMTC)** | 🔴 Critical | Medium | `windows`, `tauri-plugin-global-shortcut` | 2-3 days | High user value - media keys, OS media UI |
| **Discord Rich Presence** | 🟡 High | Low | `discord-rich-presence` | 1-2 days | Standalone, optional feature |
| **Network Diagnostics** | 🟡 High | High | `tokio`, `reqwest`, custom logic | 3-5 days | Complex but valuable for troubleshooting |
| **AdBlock / Proxy** | 🟢 Medium | Very High | WebView2 COM, custom URI scheme | 5-7 days | Requires deep WebView2 integration |
| **System Notifications** | 🟢 Medium | Low | `tauri-plugin-notification` | 0.5-1 day | Simple, uses existing Tauri plugin |
| **Auto-Updater** | 🟢 Medium | Low | `tauri-plugin-updater` | 1-2 days | Requires infrastructure (GitHub Releases) |
| **Media Session / Wallpapers** | 🔵 Low | High | Complex audio routing | 5+ days | Low priority - core app is web-based |

---

## Detailed Module Analysis

### 1. System Tray (Critical - Foundation)

**Reference Implementation**: `app/tray.rs`, `app/popover.rs`, `app/visibility.rs`

| Feature | Reference Approach | Our Implementation |
|---------|-------------------|-------------------|
| Tray icon with menu | `tauri::menu::Menu` + `tauri::tray::TrayIcon` | Same - Tauri 2 native APIs |
| Left-click → mini player popover | Custom `WebviewWindow` (label: "popover") | Create minimal popover window |
| Right-click → context menu | Dynamic menu with states | Tauri 2 menu API |
| Minimize to tray (not close) | WindowEvent::CloseRequested handler | Same pattern |
| Tray state persistence | JSON in app data dir | Same |
| Auto-start minimized | Launch args detection | Same |

**Technical Requirements**:
```toml
# Cargo.toml additions
tauri-plugin-tray-icon = "2"  # If not in core
```

**Integration Points**:
- `tauri.conf.json`: Remove `"window"` config for main, create programmatically
- `main.rs setup()`: Add tray setup after window creation
- Window event handlers: Prevent close → hide instead

**Porting Complexity**: ⭐⭐⭐ (Medium)
- Tauri 2 tray API is stable
- Need to design popover HTML/CSS (or use native menu only initially)
- Reference has complex popover with mini-player - we can start simpler

---

### 2. Media Controls / SMTC (Critical - High User Value)

**Reference Implementation**: `audio/media_controls.rs`, `audio/state.rs`, `audio/tick.rs`

| Feature | Reference Approach | Our Implementation |
|---------|-------------------|-------------------|
| Windows SMTC integration | `windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager` | Same - Windows native APIs |
| Media key handling (play/pause/next/prev) | `tauri-plugin-global-shortcut` + SMTC | Same |
| Now Playing UI in OS | SMTC session properties | Same |
| Track metadata (title, artist, artwork) | Extracted from web page via JS | Need injection script |
| Playback position sync | Audio engine + SMTC timeline | Web-based playback position |

**Technical Requirements**:
```toml
# Cargo.toml additions
windows = { version = "0.62", features = [
    "Media_Control_GlobalSystemMediaTransportControlsSessionManager",
    "Media_Control_GlobalSystemMediaTransportControlsSession",
    "Media_Control_GlobalSystemMediaTransportControlsSessionPlaybackInfo",
    "Media_Control_GlobalSystemMediaTransportControlsSessionMediaProperties",
    "Media_Control_GlobalSystemMediaTransportControlsSessionTimelineProperties",
    "Media_Playback_PlaybackState",
    "Media_Playback_AutoRepeatMode",
    "Foundation_Collections_MapChangedEventArgs",
    "Foundation_Collections_IMap",
    "Foundation_IReference",
    "Foundation_TimeSpan",
    "Storage_Streams_RandomAccessStreamReference",
    "Storage_Streams_FileRandomAccessStream",
    "Win32_Foundation",
    "Win32_UI_Shell",
] }
tauri-plugin-global-shortcut = "2"
```

**Key Challenge**: Reference has native audio engine (rodio). Our app uses WebView2 - SoundCloud handles playback in web page.

**Solution**: 
1. Inject JS to read player state from SoundCloud page
2. Send metadata to Rust via Tauri commands
3. Update SMTC session from Rust

**Porting Complexity**: ⭐⭐⭐⭐ (High)
- Requires WebView2 JS injection to read SoundCloud player state
- SMTC API is complex but well-documented
- Need to bridge web playback state → native SMTC

---

### 3. Discord Rich Presence (High - Standalone)

**Reference Implementation**: `discord/commands.rs`, `discord/mod.rs`

| Feature | Reference Approach | Our Implementation |
|---------|-------------------|-------------------|
| Discord IPC connection | `discord-rich-presence` crate | Same crate |
| Activity types (Listening) | `ActivityType::Listening` | Same |
| Track metadata display | Title, artist, artwork, progress | Same |
| Buttons (Listen on SoundCloud) | Activity buttons | Same |
| Party/group support | For listening rooms | Optional |
| Lyrics line display | Custom status text | Optional |

**Technical Requirements**:
```toml
# Cargo.toml additions
discord-rich-presence = "0.8"
```

**Integration Points**:
- Tauri commands: `discord_connect`, `discord_disconnect`, `discord_set_activity`, `discord_clear_activity`
- Settings UI: Enable/disable, configure display options
- Called from media controls when track changes

**Porting Complexity**: ⭐⭐ (Low)
- Crate handles all IPC complexity
- Just need to wire up Tauri commands and call from track change events
- Reference has comprehensive tests - good to port

---

### 4. Network Diagnostics (High - Complex)

**Reference Implementation**: `network/netcheck/` (19 modules!)

| Feature | Reference Approach | Our Implementation |
|---------|-------------------|-------------------|
| Multi-target health checks | Parallel HTTP/TCP checks to SC endpoints | Same concept |
| DNS over HTTPS (DoH) verification | Custom DoH client | Optional |
| Edge/relay detection | Custom routing logic | Simplified |
| Proxy/VPN detection | Environment analysis | Basic |
| Historical reports | Persistent storage + UI | Simplified |
| Auto-check scheduling | Exponential backoff | Basic periodic |

**Key Modules in Reference**:
- `model.rs` - Data structures (NetReport, Verdict, Target, etc.)
- `phases.rs` - Check execution phases
- `targets.rs` - Target definitions (API, WebSocket, CDN, etc.)
- `verdict.rs` - Health classification logic
- `store.rs` - Persistence
- `route.rs` - Route classification

**Technical Requirements**:
```toml
# Cargo.toml additions
reqwest = { version = "0.12", features = ["json", "rustls-tls", "stream"] }
tokio = { version = "1", features = ["full"] }
futures-util = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
trust-dns-resolver = "0.23"  # For DoH
```

**Porting Strategy**:
- Start with minimal: 3-4 key endpoints (api.soundcloud.com, ws.soundcloud.com, cdn)
- Basic verdict: Online / Degraded / Offline
- Store last 10 reports in JSON
- Settings UI: "Run Network Check" button + display last result

**Porting Complexity**: ⭐⭐⭐⭐⭐ (Very High)
- Reference is extremely comprehensive (2000+ lines)
- We need ~20% of functionality for 80% value
- Recommend: Build simplified version from scratch using reference as guide

---

### 5. AdBlock / Network Proxy (Medium - Very Complex)

**Reference Implementation**: `network/proxy.rs`, `network/proxy_server.rs`, `network/server.rs`, URI scheme `scproxy://`

| Feature | Reference Approach | Our Implementation |
|---------|-------------------|-------------------|
| Request interception | Custom URI scheme (`scproxy://`) + local HTTP proxy | Same |
| Ad blocking | Filter lists (EasyList) + custom rules | Same |
| Resource rewriting | Inject CSS/JS, modify responses | Complex |
| Wallpaper/CDN proxy | Local static server for assets | Optional |

**How Reference Works**:
1. Registers `scproxy://` URI scheme handler in Tauri
2. Runs local HTTP proxy server on random port
3. Injects initialization script to rewrite `fetch`/`XMLHttpRequest`
4. Requests go through proxy → filter → forward → modify response

**Technical Requirements**:
```toml
# Cargo.toml additions
http = "1"
hyper = { version = "1", features = ["full", "http1", "http2"] }
hyper-util = { version = "0.1", features = ["full"] }
tokio-tungstenite = "0.21"  # If WebSocket proxy needed
```

**Major Challenge**: WebView2 on Windows doesn't support proxy configuration as easily as WebKit. The reference uses a custom URI scheme which requires:
- `--enable-features=NetworkService,NetworkServiceInProcess` flags
- Custom `WebResourceRequested` handler via WebView2 COM

**Alternative for Tauri 2 + WebView2**:
- Use WebView2's `add_WebResourceRequestedFilter` + `WebResourceRequested` event
- Modify request/response headers and body
- Much simpler than full proxy server

**Porting Complexity**: ⭐⭐⭐⭐⭐ (Very High)
- Deep WebView2 COM integration required
- Reference's proxy server is 500+ lines
- Recommendation: Defer to Phase 2, use uBlock Origin extension approach if possible

---

### 6. System Notifications (Medium - Simple)

**Reference Implementation**: Uses `tauri-plugin-notification` + custom wrapper

| Feature | Reference Approach | Our Implementation |
|---------|-------------------|-------------------|
| New track notifications | `tauri::notification::Notification` | Same |
| Custom actions (play/pause) | Notification buttons | Same |
| Persistent notifications | Windows toast API | Same |

**Technical Requirements**:
```toml
# Already in our Cargo.toml
tauri-plugin-notification = "2"
```

**Integration Points**:
- Tauri command: `send_notification(title, body, actions)`
- Called from media controls on track change
- Settings: Enable/disable, choose events

**Porting Complexity**: ⭐ (Trivial)
- 1-2 hours max
- Tauri plugin handles platform differences

---

### 7. Auto-Updater (Medium - Infrastructure Required)

**Reference Implementation**: `app/updater/mod.rs` using `tauri-plugin-updater`

| Feature | Reference Approach | Our Implementation |
|---------|-------------------|-------------------|
| Update check | GitHub Releases API | Same |
| Download + install | Built-in plugin logic | Same |
| Progress reporting | Channel-based streaming | Same |
| Signature verification | Built-in (Ed25519) | Same |
| Flatpak/Store detection | `install_kind` module | Not needed (Windows only) |

**Technical Requirements**:
```toml
# Cargo.toml additions
tauri-plugin-updater = "2"
```

**Infrastructure Needed**:
1. GitHub repository with Releases
2. Code signing certificate (Windows) - **Required for production**
3. Updater public key in `tauri.conf.json`
4. Build pipeline that signs and publishes

**Current Blockers**:
- No code signing cert → updater will fail on Windows
- No GitHub repo for releases
- **Recommendation**: Defer until distribution ready

**Porting Complexity**: ⭐⭐ (Low code, High infrastructure)
- Code is ~200 lines
- Infrastructure is the real cost

---

### 8. Media Session / Wallpapers / Advanced Audio (Low - Out of Scope)

**Reference Implementation**: `audio/` (entire module), `network/wallpapers.rs`, `chat/`

| Feature | Reference Approach | Verdict |
|---------|-------------------|---------|
| Native audio engine (rodio) | Full playback stack | ❌ Not needed - web player |
| Equalizer / effects | DSP on raw audio | ❌ Not applicable |
| Crossfade / gapless | Custom logic | ❌ Not applicable |
| Track caching / offline | Download + transcode (ffmpeg) | ❌ Out of scope |
| Lyrics / comments timeline | WebSocket + parsing | ❌ Out of scope |
| Chat / messaging | Full SC chat client | ❌ Out of scope |
| Wallpaper engine | Custom CDN + caching | ❌ Out of scope |
| OBS integration | WebSocket plugin | ❌ Out of scope |
| Last.fm / ListenBrainz | Scrobbling | 🟡 Maybe later |

---

## Integration Architecture

### Proposed Module Structure

```
src-tauri/
├── src/
│   ├── main.rs                 # Entry point, setup()
│   ├── commands.rs             # All Tauri commands (centralized)
│   ├── state.rs                # AppState with all module states
│   ├── modules/
│   │   ├── mod.rs              # Module registration
│   │   ├── tray.rs             # System tray + popover
│   │   ├── media_controls.rs   # SMTC + media keys
│   │   ├── discord.rs          # Discord Rich Presence
│   │   ├── netcheck.rs         # Network diagnostics
│   │   ├── notifications.rs    # System notifications
│   │   └── updater.rs          # Auto-updater (deferred)
│   ├── webview2/               # WebView2-specific helpers
│   │   ├── mod.rs
│   │   ├── popup_handler.rs    # OAuth popup handling
│   │   └── js_injection.rs     # Initialization scripts
│   └── settings/               # Settings commands
│       ├── mod.rs
│       └── commands.rs
├── Cargo.toml
└── tauri.conf.json
```

### State Management Pattern

```rust
// src/state.rs
use std::sync::Arc;
use tauri::Manager;

pub struct AppState {
    pub tray: Arc<tray::TrayState>,
    pub media_controls: Arc<media_controls::MediaControlsState>,
    pub discord: Arc<discord::DiscordState>,
    pub netcheck: Arc<netcheck::NetCheckState>,
    pub settings: Arc<settings::SettingsState>,
}

// In setup():
app.manage(AppState { ... });
```

### Tauri Command Organization

```rust
// src/commands.rs - All commands in one place for generate_handler!
#[tauri::command]
async fn tray_show_popover(app: AppHandle) { ... }

#[tauri::command]
async fn media_controls_play_pause(state: State<MediaControlsState>) { ... }

#[tauri::command]
async fn discord_connect(app: AppHandle, state: State<DiscordState>) { ... }

#[tauri::command]
async fn netcheck_run(state: State<NetCheckState>) -> NetReport { ... }

// main.rs setup():
.invoke_handler(tauri::generate_handler![
    // Window management
    window_minimize, window_maximize, window_close,
    // Settings
    settings_get, settings_save, settings_reset,
    // Tray
    tray_show_popover, tray_set_minimize_to_tray,
    // Media Controls
    media_controls_play_pause, media_controls_next, media_controls_prev,
    // Discord
    discord_connect, discord_disconnect, discord_set_activity,
    // NetCheck
    netcheck_run, netcheck_last,
    // Notifications
    notification_send,
])
```

---

## Phase Implementation Plan

### Phase 1: Foundation (Week 1)
**Goal**: System Tray + Minimize to Tray (enables all other features)

| Task | Effort | Dependencies |
|------|--------|--------------|
| Add `tauri-plugin-tray-icon` (if needed) | 0.5h | - |
| Create `modules/tray.rs` with TrayIcon + Menu | 4h | - |
| Implement minimize-to-tray on close | 2h | - |
| Add tray state persistence (position, minimized) | 2h | - |
| Basic context menu: Show, Settings, Quit | 1h | - |
| Test: App hides to tray, restores from tray | 1h | - |

**Deliverable**: App minimizes to tray, restores from tray icon

---

### Phase 2: Media Controls (Week 2)
**Goal**: Windows SMTC + Global Media Keys

| Task | Effort | Dependencies |
|------|--------|--------------|
| Add `windows` crate with SMTC features | 1h | Phase 1 |
| Create `modules/media_controls.rs` | 2h | Phase 1 |
| Implement SMTC session registration | 4h | - |
| Create JS injection to read SC player state | 4h | - |
| Bridge web player state → SMTC metadata | 3h | - |
| Register global shortcuts (MediaPlayPause, etc.) | 2h | - |
| Handle media key events → control web player | 3h | - |
| Test: OS media overlay shows track, keys work | 2h | - |

**Deliverable**: Media keys control SoundCloud, Windows shows Now Playing

---

### Phase 3: Discord Rich Presence (Week 2-3)
**Goal**: Show listening status on Discord

| Task | Effort | Dependencies |
|------|--------|--------------|
| Add `discord-rich-presence` crate | 0.5h | Phase 2 |
| Create `modules/discord.rs` (port from reference) | 3h | - |
| Add Tauri commands (connect, disconnect, set_activity) | 2h | - |
| Integrate with media_controls track change events | 2h | Phase 2 |
| Settings UI: Enable/disable, display options | 2h | - |
| Test: Discord shows "Listening to X on SoundCloud" | 1h | - |

**Deliverable**: Discord Rich Presence working with track metadata

---

### Phase 4: Network Diagnostics (Week 3-4)
**Goal**: Basic network health check in Settings

| Task | Effort | Dependencies |
|------|--------|--------------|
| Create simplified `modules/netcheck.rs` | 4h | - |
| Define 4-5 key targets (API, WS, CDN, auth) | 1h | - |
| Implement parallel health checks | 3h | - |
| Basic verdict logic (Online/Degraded/Offline) | 2h | - |
| Persist last 10 reports to JSON | 2h | - |
| Tauri commands: run_check, get_last | 1h | - |
| Settings UI: "Run Check" button + results display | 3h | - |
| Auto-check on startup (optional) | 1h | - |

**Deliverable**: Network diagnostics panel in Settings

---

### Phase 5: Notifications (Week 4)
**Goal**: System notifications for track changes

| Task | Effort | Dependencies |
|------|--------|--------------|
| Use existing `tauri-plugin-notification` | 0.5h | Phase 2 |
| Create `modules/notifications.rs` | 1h | - |
| Send notification on track change (from media_controls) | 1h | Phase 2 |
| Settings: Enable/disable, event types | 1h | - |
| Test: Windows toast appears on new track | 0.5h | - |

**Deliverable**: Optional track change notifications

---

### Phase 6: Settings UI Integration (Week 4-5)
**Goal**: Unified Settings window with all modules

| Task | Effort | Dependencies |
|------|--------|--------------|
| Extend `settings.html` with Toolkit sections | 3h | All phases |
| Add tabs: General, Window, Network, Discord, Notifications | 2h | - |
| Wire up all module settings commands | 2h | All phases |
| Add "Advanced" section for netcheck, diagnostics | 1h | Phase 4 |
| Test: All settings persist and apply | 2h | - |

**Deliverable**: Complete Settings UI with all features

---

### Phase 7: Polish & Release (Week 5+)
**Goal**: Production-ready build

| Task | Effort | Dependencies |
|------|--------|--------------|
| Code signing certificate setup | 4h | - |
| GitHub Actions: build, sign, release | 4h | - |
| `tauri-plugin-updater` integration | 4h | - |
| Installer testing (MSI/NSIS) | 2h | - |
| Icon / branding finalization | 2h | - |
| Documentation (README, CHANGELOG) | 2h | - |

---

## Risk Assessment & Mitigation

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| WebView2 COM API complexity (SMTC bridge) | High | High | Start with minimal JS injection, iterate |
| Windows SMTC API breaking changes | Medium | Low | Pin `windows` crate version, test on Win10/11 |
| Discord IPC connection issues | Low | Medium | Graceful degradation - disable if fails |
| Code signing certificate cost/time | High | High | Budget $200-400/year, start early |
| Tauri 2 API changes | Medium | Low | Pin Tauri version, test upgrades in branch |
| AdBlock/WebView2 proxy complexity | Very High | Very High | **Defer entirely** - not in MVP |

---

## License Compliance (MIT)

**Reference Project**: zxcloli666/SoundCloud-Desktop - MIT License

**Requirements**:
1. ✅ Include MIT license copy in our repo
2. ✅ Retain copyright notices in ported code
3. ✅ Add attribution in README/About dialog
4. ✅ No external infrastructure dependencies copied

**Attribution Template**:
```
This project includes code adapted from SoundCloud-Desktop 
by zxcloli666 (https://github.com/zxcloli666/SoundCloud-Desktop)
licensed under MIT License.
```

**Files needing attribution headers** (when porting):
- `modules/discord.rs` - heavily based on reference
- `modules/netcheck.rs` - structure inspired by reference
- `modules/tray.rs` - pattern from reference

---

## Next Steps

1. **Immediate**: Start Phase 1 - System Tray implementation
2. **Parallel**: Research WebView2 JS injection for player state reading
3. **Decision**: Confirm code signing certificate timeline
4. **Defer**: AdBlock, Updater infrastructure, Advanced audio features

---

*Generated: 2026-11-15 | For SoundCloud Desktop Tauri 2 Project*