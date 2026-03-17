#!/bin/bash
# Patches remote-access.json with AGENT_PLAYGROUND_URL for production builds.
# Called by tauri.conf.json beforeBuildCommand.

CAPABILITY_FILE="src-tauri/capabilities/remote-access.json"

if [ -z "$AGENT_PLAYGROUND_URL" ]; then
  echo "[patch-remote-url] AGENT_PLAYGROUND_URL not set, keeping localhost only"
  exit 0
fi

DOMAIN=$(echo "$AGENT_PLAYGROUND_URL" | sed 's|https\?://||' | sed 's|/.*||')

echo "[patch-remote-url] Adding $DOMAIN to remote-access.json"

# Use node for reliable JSON manipulation
node -e "
const fs = require('fs');
const cap = JSON.parse(fs.readFileSync('$CAPABILITY_FILE', 'utf8'));
const prodUrl = process.env.AGENT_PLAYGROUND_URL.replace(/\/$/, '') + '/*';
if (!cap.remote.urls.includes(prodUrl)) {
  cap.remote.urls.push(prodUrl);
}
fs.writeFileSync('$CAPABILITY_FILE', JSON.stringify(cap, null, 2) + '\n');
console.log('[patch-remote-url] Updated urls:', cap.remote.urls);
"
