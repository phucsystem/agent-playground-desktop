# System Architecture

**Version:** 0.2.0 | **Updated:** 2026-03-19

## 1. High-Level Architecture

```mermaid
graph TB
    subgraph Desktop["Tauri v2 Desktop Shell (Rust)"]
        Window["System Window<br/>(Webview)"]
        Tray["System Tray<br/>+ Menu"]
        Bridge["JS Bridge<br/>(Initialization Script)"]
        Store["Local Store<br/>(JSON)"]
        Notif["Notification Engine<br/>(Plugin)"]
        Shortcuts["Global Shortcuts<br/>(Plugin)"]
        Updater["Auto-Updater<br/>(Plugin)"]
    end

    subgraph Frontend["Web App (Remote)"]
        Chat["Chat UI<br/>(Next.js)"]
        Realtime["Realtime Listener<br/>(Supabase)"]
    end

    subgraph Backend["Backend Services"]
        Supabase["Supabase<br/>(Auth + DB)"]
    end

    Window -->|Page Load| Chat
    Chat -->|Custom Events| Bridge
    Bridge -->|IPC invoke()| Desktop
    Bridge -->|JS→Rust Commands| Store
    Bridge -->|JS→Rust Commands| Notif
    Realtime -->|Message Events| Chat
    Chat -->|notify event| Bridge
    Bridge -->|notify_new_message| Notif
    Tray -->|toggle_window| Window
    Shortcuts -->|Global Hotkey| Window
    Updater -->|Check Releases| Backend
    Store -->|Persist Config| Desktop
    Supabase -->|Authenticate| Chat
```

**Key Characteristics:**
- Remote URL loading (no bundled web app)
- Event-driven notification bridge (web app → desktop)
- Single window with minimize-to-tray
- Persistent state via local JSON store
- Background auto-update checking

## 2. Data Flow Diagram

### 2.1 New Message Notification Flow

```mermaid
sequenceDiagram
    participant Supabase as Supabase Realtime
    participant WebApp as Web App (Next.js)
    participant JS as JS Bridge (bridge.js)
    participant Rust as Rust Backend
    participant OS as OS Notification

    Supabase->>WebApp: Message INSERT event
    WebApp->>WebApp: Process message
    WebApp->>JS: dispatch CustomEvent('tauri:new-message')
    JS->>JS: Check window.__TAURI__ available
    JS->>Rust: invoke('notify_new_message', payload)

    Rust->>Rust: Check notifications_enabled?
    Rust->>Rust: Check if window focused?
    Rust->>Rust: Truncate message (100 chars)
    Rust->>OS: Send native notification
    OS->>OS: Display banner

    Note over Rust: Store unread count
    Rust->>Rust: Update tray tooltip
```

**Latency targets:**
- Supabase → WebApp: <100ms
- WebApp dispatch: <50ms
- Bridge invoke: <50ms
- Notification display: <300ms
- **Total: <500ms**

### 2.2 Window Persistence Flow

```mermaid
sequenceDiagram
    participant User as User
    participant Window as Main Window
    participant Rust as Rust Backend
    participant Store as plugin-store (JSON)
    participant Disk as Disk

    User->>Window: Move / Resize window
    Window->>Rust: Event fired
    Rust->>Rust: Debounce (500ms)
    Rust->>Store: save_window_state()
    Store->>Disk: Write to JSON file

    User->>User: Quit app
    User->>Rust: Close event
    Rust->>Store: save_window_state()
    Rust->>Rust: Hide window, exit

    Disk->>Disk: (Time passes)

    User->>User: Relaunch app
    Rust->>Store: get_window_state()
    Store->>Disk: Read JSON file
    Store->>Rust: Return (x, y, width, height, maximized)
    Rust->>Window: Apply geometry
    Window->>Window: Restore position
```

**State elements persisted:**
- x, y: window position
- width, height: window size
- maximized: boolean flag

## 3. Component Architecture

### 3.1 Rust Backend Modules

```mermaid
graph LR
    Main["main.rs<br/>(Entry)"]
    Lib["lib.rs<br/>(Setup)"]
    Commands["commands.rs<br/>(IPC)"]
    Tray["tray.rs<br/>(Menu)"]
    Updater["updater.rs<br/>(Updates)"]

    Main -->|Initialize| Lib
    Lib -->|Register| Commands
    Lib -->|Initialize| Tray
    Lib -->|Start timer| Updater

    Commands -->|Read/Write| Store["plugin-store"]
    Commands -->|Dispatch| Notif["plugin-notification"]
    Tray -->|Call| Commands
    Updater -->|Check| GitHub["GitHub API"]
```

**Module responsibilities:**

| Module | LOC | Responsibility |
|--------|-----|-----------------|
| main.rs | 6 | Entry point, delegates to lib |
| lib.rs | 136 | App setup, window creation, bridge injection, event loop |
| commands.rs | 230 | 5 IPC commands, data struct definitions, store integration |
| tray.rs | 106 | System tray menu, visibility toggle, notification toggle |
| updater.rs | 98 | Background update checks, version comparison, download |

### 3.2 Frontend Stack

```mermaid
graph TB
    HTML["index.html<br/>(Loading Screen)"]
    Bridge["bridge.js<br/>(JS Bridge)"]
    RemoteApp["Remote Web App<br/>(Next.js)"]

    HTML -->|Load| RemoteApp
    HTML -->|Inject| Bridge
    Bridge -->|Polls| TAURI["window.__TAURI__"]
    Bridge -->|Listen| Events["Custom Events"]
    Events -->|Invoke| Rust["Tauri Commands"]
    RemoteApp -->|Dispatch| Events
```

**Frontend files:**
- **index.html** (37 lines): Loading spinner, light/dark mode CSS, minimal JS to hide on page load
- **bridge.js** (71 lines): Polls for __TAURI__, listens for 3 custom events, forwards to IPC

## 4. IPC Command Architecture

```mermaid
graph TB
    subgraph WebApp["Web App Layer"]
        JS["JS Event Handlers"]
    end

    subgraph Bridge["Bridge Layer"]
        Listen["Event Listeners"]
        Invoke["Invoke Tauri Commands"]
    end

    subgraph RustCommands["Rust Commands (commands.rs)"]
        Notify["notify_new_message<br/>(send OS notification)"]
        Badge["update_badge_count<br/>(update tray)"]
        Active["report_user_active<br/>(suppress notifications)"]
        GetCfg["get_app_config<br/>(read store)"]
        SetCfg["set_app_config<br/>(write + apply)"]
    end

    subgraph State["Persistent State (plugin-store)"]
        AppCfg["AppConfig<br/>JSON"]
        WinState["WindowState<br/>JSON"]
    end

    subgraph Plugins["Tauri Plugins"]
        Notif["notification<br/>plugin"]
        Store["store<br/>plugin"]
        AutoStart["autostart<br/>plugin"]
        Shortcut["global-shortcut<br/>plugin"]
        Update["updater<br/>plugin"]
    end

    JS -->|dispatch| Listen
    Listen -->|invoke| Invoke
    Invoke -->|→ notify| Notify
    Invoke -->|→ badge| Badge
    Invoke -->|→ active| Active
    Invoke -->|→ get/set| GetCfg
    Invoke -->|→ get/set| SetCfg

    Notify -->|send| Notif
    Badge -->|update| Notif
    GetCfg -->|read| AppCfg
    SetCfg -->|write| AppCfg
    SetCfg -->|apply| AutoStart
    SetCfg -->|apply| Shortcut
    SetCfg -->|apply| Update
```

**Command flow:** JS Event → JS Bridge → Tauri Invoke → Rust Command → Plugin API → Side effects

## 5. Plugin Dependency Map

```mermaid
graph TB
    Tauri["Tauri v2.0"]

    Tauri -->|provides| Tray["tray-icon<br/>(built-in)"]
    Tauri -->|provides| Notif["tauri-plugin-notification"]
    Tauri -->|provides| Store["tauri-plugin-store"]
    Tauri -->|provides| AutoStart["tauri-plugin-autostart"]
    Tauri -->|provides| Shortcut["tauri-plugin-global-shortcut"]
    Tauri -->|provides| Update["tauri-plugin-updater"]
    Tauri -->|provides| DeepLink["tauri-plugin-deep-link"]

    Notif -->|Used by| Commands["commands.rs"]
    Store -->|Used by| Commands
    AutoStart -->|Used by| Commands
    Shortcut -->|Used by| Commands
    Update -->|Used by| Updater["updater.rs"]
    DeepLink -->|Used by| Lib["lib.rs"]
    Tray -->|Used by| Tray2["tray.rs"]
```

## 6. Configuration & Capability Architecture

```mermaid
graph LR
    Conf["tauri.conf.json"]
    Default["capabilities/default.json<br/>(local perms)"]
    Remote["capabilities/remote-access.json<br/>(web app perms)"]
    BuildScript["scripts/patch-remote-url.sh<br/>(build-time)"]
    Env["AGENT_PLAYGROUND_URL<br/>(env var)"]

    Conf -->|Window config| App["Tauri App"]
    Default -->|Local permissions| App
    Remote -->|Remote content permissions| App

    Env -->|URL| BuildScript
    BuildScript -->|Injects| Remote

    Remote -->|Whitelist| URLs["localhost:3000 (dev)<br/>https://* (prod)"]
```

**Key principle:** Production URL patched at build time to avoid hardcoding credentials.

## 7. State Machine Diagrams

### 7.1 Window Visibility State

```mermaid
stateDiagram-v2
    [*] --> Visible: App launch

    Visible --> Hidden: Close button
    Visible --> Hidden: Global shortcut
    Visible --> Hidden: Tray menu

    Hidden --> Visible: Global shortcut
    Hidden --> Visible: Tray "Show"
    Hidden --> Visible: Notification click

    Visible --> [*]: Quit (tray menu)
    Hidden --> [*]: Quit (tray menu)
```

### 7.2 Update Check State Machine

```mermaid
stateDiagram-v2
    [*] --> Idle

    Idle -->|App startup| Checking: Start check timer
    Idle -->|User clicks "Check Updates"| Checking: Manual check

    Checking -->|Success| Available: New version found
    Checking -->|No update| Idle: Same version
    Checking -->|Network error| Idle: Log error, retry later

    Available -->|User "Install Now"| Downloading: Start download
    Available -->|User "Later"| Idle: Dismiss dialog

    Downloading -->|Success| Installed: Restart app
    Downloading -->|Failure| Idle: Log error

    Installed --> [*]: App restarts
```

## 8. Notification Architecture

```mermaid
graph TB
    subgraph Desktop["Tauri Desktop"]
        Cmd["notify_new_message<br/>command"]
        Logic["Check config<br/>Check window focus<br/>Format message"]
        Payload["Build notification<br/>title, body, etc"]
    end

    subgraph Plugins["Tauri Plugins"]
        Notif["tauri-plugin-notification<br/>sendNotification()"]
    end

    subgraph OS["Operating System"]
        macOS["macOS:<br/>NSNotificationCenter"]
        Windows["Windows:<br/>WinToast API"]
        Linux["Linux:<br/>libnotify"]
    end

    subgraph Display["User Display"]
        Banner["Notification<br/>Banner"]
    end

    Cmd -->|Parse| Logic
    Logic -->|→ truncate| Payload
    Payload -->|invoke| Notif
    Notif -->|call| macOS
    Notif -->|call| Windows
    Notif -->|call| Linux
    macOS -->|display| Banner
    Windows -->|display| Banner
    Linux -->|display| Banner
```

**Notification lifecycle:**
1. Web app detects new message via Supabase Realtime
2. Dispatches `tauri:new-message` custom event
3. JS bridge invokes `notify_new_message` command
4. Rust checks: enabled? window focused?
5. Formats (truncate to 100 chars, add "...")
6. Sends to OS via plugin
7. OS displays banner
8. User clicks → navigates webview to conversation

## 9. File I/O Diagram

```mermaid
graph TB
    Rust["Rust Backend"]

    Rust -->|Via plugin-store| Store["Local JSON Files"]

    Store -->|AppConfig| AppCfgFile[".../AppConfig.json<br/>notifications_enabled<br/>auto_start<br/>global_shortcut<br/>web_app_url<br/>etc"]
    Store -->|WindowState| WinStateFile[".../WindowState.json<br/>x, y<br/>width, height<br/>maximized"]

    AppCfgFile -->|On startup| Rust
    WinStateFile -->|On startup| Rust

    Rust -->|On save| AppCfgFile
    Rust -->|On window close| WinStateFile

    Disk["Disk Storage<br/>(platform-specific)"]
    AppCfgFile -->|Persisted| Disk
    WinStateFile -->|Persisted| Disk
```

**Storage locations (platform-specific):**
- macOS: ~/Library/Application Support/com.agent-playground.desktop/
- Windows: %AppData%/com.agent-playground.desktop/
- Linux: ~/.config/com.agent-playground.desktop/

## 10. Release & Update Architecture

```mermaid
graph LR
    Code["Code on main"]
    GH["GitHub Actions<br/>(release.yml)"]
    Build["Build macOS + Windows"]
    Sign["Sign binaries<br/>(Ed25519)"]
    Release["GitHub Releases"]
    Latest["latest.json<br/>(version info)"]

    Code -->|PR merge| GH
    GH -->|Trigger| Build
    Build -->|Artifacts| Sign
    Sign -->|Upload| Release
    Release -->|Generate| Latest

    Desktop["Desktop App<br/>(running)"]
    Check["Check for updates<br/>(every 6h)"]
    Fetch["Fetch latest.json"]
    Compare["Compare versions"]
    Dialog["Show dialog<br/>(if new)"]
    Download["Download & install<br/>(if user agrees)"]
    Restart["Restart app"]

    Desktop -->|On startup| Check
    Check -->|GET| Fetch
    Fetch -->|Parse| Compare
    Compare -->|Version > current| Dialog
    Dialog -->|Install Now| Download
    Download -->|Verify sig| Restart
```

## 11. Security Architecture

```mermaid
graph TB
    CSP["Content Security<br/>Policy<br/>(null in config)"]
    Caps["Tauri Capabilities<br/>(ACL)"]
    Remote["remote-access.json<br/>(URL whitelist)"]
    TauriAPI["Tauri API<br/>Access Control"]

    CSP -->|Allow| HTTPS["HTTPS only"]
    Caps -->|Restrict| Permissions["Specific permissions"]
    Remote -->|Whitelist| URLs["localhost:3000<br/>https://*"]

    HTTPS -->|Blocks| LocalFile["Local files"]
    HTTPS -->|Blocks| HTTP["Unencrypted HTTP"]

    Permissions -->|Allow| Notify["Notifications"]
    Permissions -->|Allow| Store["Local store"]
    Permissions -->|Allow| Shortcuts["Shortcuts"]
    Permissions -->|Deny| FileAccess["Direct file access"]

    URLs -->|Allow| DevServer["Dev server"]
    URLs -->|Allow| ProdServer["Prod server (patched)"]
    URLs -->|Deny| MaliciousSites["Arbitrary domains"]

    TauriAPI -->|Verify| Caps
    TauriAPI -->|Verify| Remote
```

**Defense layers:**
1. Tauri capabilities (ACL) restrict which commands JS can invoke
2. remote-access.json whitelist restricts which URLs get API access
3. HTTPS enforcement (CSP null allows all, but relies on HTTPS requirement)
4. Ed25519 signature verification for updates

## 12. Deployment Architecture

```mermaid
graph TB
    LocalDev["Developer<br/>Local Machine"]
    MainBranch["GitHub main<br/>Branch"]
    GitHub["GitHub Actions<br/>CI/CD"]

    LocalDev -->|git push| MainBranch
    MainBranch -->|PR merge| GitHub

    GitHub -->|Build| MacOS["macOS<br/>Universal Binary<br/>(arm64 + x86_64)"]
    GitHub -->|Build| Windows["Windows<br/>x86_64<br/>Binary"]

    MacOS -->|Sign + Package| DMG["Agent Playground.dmg"]
    Windows -->|Sign + Package| MSI["Agent Playground.msi"]

    DMG -->|Upload| Releases["GitHub Releases<br/>v0.2.0 (draft)"]
    MSI -->|Upload| Releases

    Releases -->|Manual| Publish["Publish Release<br/>(to GitHub)"]

    Publish -->|Advertise| LatestJSON["latest.json<br/>(update feed)"]

    Users["End Users"]
    LatestJSON -->|Check| Users
    Releases -->|Download| Users
```

**Key workflow:**
1. Developer pushes code to main
2. GitHub Actions automatically builds macOS + Windows
3. Artifacts uploaded to GitHub Releases (draft)
4. Manual publish converts draft to public
5. End users see update via latest.json feed

---

**Architecture Principles:**
- **Separation of concerns:** Rust (system) vs JS (UI)
- **Event-driven:** Custom events connect web app to desktop
- **Graceful degradation:** Config errors logged, sensible defaults used
- **Minimal persistence:** Only state that can't be recomputed (geometry, preferences)
- **Platform-native:** Leverage OS APIs (notifications, tray, shortcuts), not custom solutions
