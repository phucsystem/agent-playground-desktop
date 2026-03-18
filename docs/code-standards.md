# Code Standards & Conventions

**Version:** 0.2.0 | **Applies to:** Rust backend, JavaScript frontend

## 1. Rust Backend Standards

### File Organization

Keep files under 200 lines for maintainability.

**Current structure (src-tauri/src/)**
- `main.rs`: Entry point only
- `lib.rs`: App setup, plugins, window events
- `commands.rs`: All IPC handlers and data structures
- `tray.rs`: System tray menu
- `updater.rs`: Auto-update checking

**When adding features:**
- New IPC command → add to commands.rs
- New tray menu item → add to tray.rs
- New update logic → add to updater.rs
- Complex command → extract into separate module if >50 lines

### Naming Conventions

| Category | Convention | Example |
|----------|-----------|---------|
| Modules | snake_case | `commands.rs`, `update_badge_count` |
| Functions | snake_case | `notify_new_message()`, `toggle_window()` |
| Structs | PascalCase | `AppConfig`, `WindowState`, `AppState` |
| Constants | SCREAMING_SNAKE_CASE | `UPDATE_CHECK_INTERVAL_SECS` |
| Variables | snake_case | `notification_enabled`, `window_state` |
| Type aliases | PascalCase | `Result<T>` (standard) |

### Error Handling

Use `Result<T>` type for fallible operations. Handle errors gracefully:

```rust
// Good: Log error, continue safely
match app_config.check_updates {
    Ok(enabled) if enabled => check_updates(),
    Err(e) => eprintln!("Failed to read config: {}", e),
    _ => {}
}

// Avoid: Unwrap on untrusted input
let notification_enabled = config.get("notifications_enabled").unwrap();
```

**Pattern:** Try-catch via Result, log on error, provide sensible default.

### Comments & Documentation

Minimal comments. Code should be self-documenting via clear naming.

**Add comments only for:**
- Non-obvious business logic (why, not what)
- Subtle timing issues (e.g., why polling for __TAURI__)
- Workarounds for platform quirks

```rust
// Good: Explains why (timing race)
// Poll for __TAURI__ availability - window initialization can race with script injection
for _ in 0..50 {
    if window.__TAURI__ {
        break;
    }
}

// Avoid: Explains what (code is clear)
// Check notifications_enabled flag
if app_config.notifications_enabled { ... }
```

### Module Exports

Use `pub fn` only for Tauri commands and exposed APIs. Keep internal functions private.

```rust
// src-tauri/src/commands.rs

#[tauri::command]
pub async fn notify_new_message(payload: NotifyPayload) -> Result<(), String> {
    // Tauri command - must be pub
}

fn filter_message_text(text: &str, max_len: usize) -> String {
    // Helper function - keep private
}
```

### Data Structures

Define structs with clear field types and defaults:

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub notifications_enabled: bool,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub global_shortcut: String,
    pub web_app_url: String,
    pub check_updates: bool,
    pub last_update_check: Option<String>,
}
```

**Rules:**
- Use `Option<T>` for nullable fields (not null strings)
- Derive `Serialize, Deserialize` for plugin-store compatibility
- Document fields with /// comments if complex

### Testing

No mandatory unit tests for v0.2. When adding tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_truncation() {
        let text = "a".repeat(150);
        let result = filter_message_text(&text, 100);
        assert_eq!(result.len(), 103); // 100 + "..."
    }
}
```

**Focus:** Critical logic (notification filtering, update checks), not boilerplate.

### Dependencies

**Current Cargo.toml constraints:**
- Tauri 2.x (no breaking changes)
- Minimize external crates (keep binary small)
- Pin major versions

Before adding a dependency:
1. Check if Tauri plugin exists (prefer plugins)
2. Consider code size impact
3. Review security (supply chain risk)

## 2. JavaScript Frontend Standards

### File Organization

**Current structure (src/)**
- `bridge.js`: JS bridge script (71 lines)
- `index.html`: Loading screen (37 lines)

**When adding JS:**
- Keep bridge.js under 100 lines
- Extract helpers into separate files only if >50 lines
- No framework (plain HTML/JS or Tauri API calls)

### Naming Conventions

| Category | Convention | Example |
|----------|-----------|---------|
| Functions | camelCase | `notifyNewMessage()`, `pollForTauri()` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_RETRIES`, `POLL_INTERVAL_MS` |
| Variables | camelCase | `messageText`, `conversationId` |
| CSS classes | kebab-case | `.spinner-overlay`, `.loading-text` |

### Tauri API Calls

Use `window.__TAURI__.core.invoke()` for command invocation:

```javascript
// Good: async/await
const config = await window.__TAURI__.core.invoke('get_app_config');

// Also OK: .then() chaining
window.__TAURI__.core.invoke('set_app_config', { notifications_enabled: false })
  .then(() => console.log('Config saved'))
  .catch(err => console.error('Error:', err));

// Avoid: Not checking __TAURI__ availability
window.__TAURI__.core.invoke(...); // Fails if not in Tauri context
```

### Event Handling

Dispatch custom events for Rust integration:

```javascript
// Good: Clear event naming
window.dispatchEvent(new CustomEvent('tauri:new-message', {
  detail: { sender_name: 'Alice', message_text: '...', conversation_id: '123', is_group: false }
}));

// Avoid: Inconsistent naming
window.dispatchEvent(new Event('notify-message'));
```

**Events must match bridge.js listeners** (see api-docs for full list).

### Error Handling

Wrap Tauri calls in try-catch when result matters:

```javascript
try {
  const config = await window.__TAURI__.core.invoke('get_app_config');
  console.log('Notifications:', config.notifications_enabled);
} catch (error) {
  console.error('Failed to load config:', error);
  // Use sensible default
  const config = { notifications_enabled: true };
}
```

### Comments & Documentation

Keep comments minimal. Use clear variable names.

```javascript
// Good: Self-documenting
const maxRetries = 50;
const pollIntervalMs = 100;
let attemptCount = 0;

while (!window.__TAURI__ && attemptCount < maxRetries) {
  await new Promise(resolve => setTimeout(resolve, pollIntervalMs));
  attemptCount++;
}

// Avoid: Redundant comments
let x = 0; // Counter
```

### HTML & CSS

**index.html (loading screen)**
- Minimal HTML (no framework)
- Inline CSS for fast load
- Light/dark mode detection via `prefers-color-scheme`
- No external resources (stylesheets, fonts)

```html
<style>
  :root {
    color-scheme: light dark;
  }

  body {
    background-color: light-dark(#ffffff, #1e1e1e);
    color: light-dark(#000000, #ffffff);
  }
</style>
```

## 3. Shared Conventions

### String Formatting

**Rust:** Use `format!()` and fmt strings
```rust
let message = format!("Unread: {}", count);
```

**JavaScript:** Use template literals
```javascript
const message = `Unread: ${count}`;
```

### IPC Payload Structures

Define once in spec (API_SPEC.md), implement consistently:

```rust
// Rust
#[derive(Serialize, Deserialize)]
struct NotifyNewMessagePayload {
    sender_name: String,
    message_text: String,
    conversation_id: String,
    is_group: bool,
}

// JavaScript (matching)
const payload = {
  sender_name: "Alice",
  message_text: "Hello!",
  conversation_id: "123",
  is_group: false
};
```

**Rule:** Payload field names must match exactly (case-sensitive).

### Configuration Keys

Store keys in AppConfig struct (Rust) and in plugin-store JSON. Key names use snake_case:

```json
{
  "notifications_enabled": true,
  "auto_start": false,
  "minimize_to_tray": true,
  "global_shortcut": "CmdOrCtrl+Shift+A"
}
```

### Logging

**Rust:** Use `eprintln!()` or structured logging (future)
```rust
eprintln!("Update check failed: {}", error);
```

**JavaScript:** Use `console.log()` / `console.error()`
```javascript
console.error('Bridge initialization failed after 50 attempts');
```

**Rule:** Log errors, not happy paths. Avoid spam.

## 4. Platform-Specific Code

### macOS vs Windows

Use `#[cfg()]` for platform-specific Rust:

```rust
#[cfg(target_os = "macos")]
fn register_global_shortcut(shortcut: &str) -> Result<(), String> {
    // macOS specific code
}

#[cfg(target_os = "windows")]
fn register_global_shortcut(shortcut: &str) -> Result<(), String> {
    // Windows specific code
}
```

For JavaScript: use Tauri's `os` module to detect platform:

```javascript
import { os } from '@tauri-apps/api';

const platform = await os.type(); // "Darwin" | "Windows_NT" | "Linux"
```

## 5. Security Guidelines

### Input Validation

Always validate untrusted input:

```rust
// In notify_new_message command
if message_text.len() > 10000 {
    return Err("Message text too long".to_string());
}
if conversation_id.is_empty() {
    return Err("Conversation ID required".to_string());
}
```

### No Secrets in Code

Never commit:
- API keys
- Private signing keys
- Database credentials
- Authentication tokens

Use environment variables or .env files (already in .gitignore).

### HTTPS Enforcement

Remote URLs must use HTTPS. Validate in bridge:

```javascript
if (!new URL(remoteUrl).protocol.startsWith('https')) {
  throw new Error('Only HTTPS URLs allowed');
}
```

## 6. Linting & Formatting

**Rust:**
- Run `cargo fmt` before committing
- No clippy warnings (lint tool)

**JavaScript:**
- No enforced linter (flexibility)
- Keep code readable (2-space indent, meaningful names)

## 7. Testing Strategy

**Unit Tests (optional for v0.2)**
- Critical logic only (notification filtering, state restoration)
- No test framework required yet
- Use Rust's built-in `#[test]` macro

**Integration Tests**
- Manual testing on macOS Intel/Apple Silicon
- Manual testing on Windows x64
- Platform-specific: deep links, notifications, tray menu

**CI/CD**
- Build verification on PR merge
- Artifact upload to GitHub Releases
- Manual release approval (not automatic publish)

## 8. Code Review Checklist

Before submitting a PR:

- [ ] Code follows naming conventions (snake_case Rust, camelCase JS)
- [ ] Error handling uses Result<> (Rust) or try-catch (JS)
- [ ] No unwrap() on untrusted input (Rust)
- [ ] No console.log in production code (JS)
- [ ] Comments explain "why", not "what"
- [ ] IPC payloads match API_SPEC.md
- [ ] No credentials or secrets in code
- [ ] Tested on at least one platform
- [ ] File size <200 lines (consider splitting if larger)

## 9. File Size Targets

| File | Target | Current | Status |
|------|--------|---------|--------|
| lib.rs | <150 lines | 136 lines | Pass |
| commands.rs | <300 lines | 230 lines | Pass |
| tray.rs | <150 lines | 106 lines | Pass |
| updater.rs | <100 lines | 98 lines | Pass |
| bridge.js | <100 lines | 71 lines | Pass |

**Rule:** If file exceeds target, plan refactoring for next phase.

## 10. Dependency Updates

When updating Tauri or plugins:

1. Test on all platforms (macOS, Windows)
2. Check for breaking changes in plugin APIs
3. Update version in Cargo.toml
4. Run `cargo build` and test manually
5. Document breaking changes in commit message

**Don't:** Update dependencies without testing.

---

**Last Updated:** 2026-03-19
