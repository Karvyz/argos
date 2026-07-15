#!/usr/bin/env bash
#
# auto-sync.sh — One-way directory sync to remote host via inotify + rsync
#
# Monitors the current directory (recursively) for changes and syncs them
# to root@luwudynamics.home:/root/xgo.  The `target/` and `.git/`
# directories are excluded from both the watch and the transfer.
#
# Usage:
#   ./auto-sync.sh            # run in foreground
#   ./auto-sync.sh &          # run in background (use `kill %1` to stop)
#   nohup ./auto-sync.sh &    # survive terminal close
#

set -euo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
LOCAL_DIR="$(cd "$(dirname "$0")" && pwd)"
REMOTE_HOST="root@luwudynamics.home"
REMOTE_DIR="/root/argos"

# Directories to exclude from both inotifywait and rsync
EXCLUDE_PATTERNS=(
  "target/"
  ".git/"
)

# Debounce delay in seconds — changes arriving within this window are batched
DEBOUNCE_SECS=1

# SSH options
SSH_OPTS=(
  -o StrictHostKeyChecking=accept-new
  -o ConnectTimeout=5
)

# Rsync options
RSYNC_OPTS=(
  -avz
  --delete   # remove files on remote that don't exist locally
  --no-owner # don't try to preserve ownership (probably different UIDs)
  --no-group # same for group
  --no-perms # don't preserve permissions (use umask on remote)
  --exclude="target/"
  --exclude=".git/"
  --exclude=".gitignore"
)

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log() {
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

run_sync() {
  log "Syncing to ${REMOTE_HOST}:${REMOTE_DIR} …"
  rsync "${RSYNC_OPTS[@]}" -e "ssh ${SSH_OPTS[*]}" \
    "${LOCAL_DIR}/" \
    "${REMOTE_HOST}:${REMOTE_DIR}/"
  log "Sync complete."
}

# ---------------------------------------------------------------------------
# Pre-flight checks
# ---------------------------------------------------------------------------
if ! command -v inotifywait &>/dev/null; then
  echo "ERROR: 'inotifywait' not found. Install inotify-tools." >&2
  exit 1
fi

if ! command -v rsync &>/dev/null; then
  echo "ERROR: 'rsync' not found." >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Build the inotifywait exclude regex
# inotifywait only accepts a single --exclude flag (POSIX extended regex).
# ---------------------------------------------------------------------------
INOTIFY_EXCLUDE_REGEX=""
sep=""
for pat in "${EXCLUDE_PATTERNS[@]}"; do
  # Escape dots and slashes for regex
  escaped="$(printf '%s' "$pat" | sed 's/\./\\./g; s|/|/|g')"
  INOTIFY_EXCLUDE_REGEX+="${sep}${escaped}"
  sep="|"
done

# ---------------------------------------------------------------------------
# Initial sync
# ---------------------------------------------------------------------------
log "Starting auto-sync for ${LOCAL_DIR}"
log "Remote: ${REMOTE_HOST}:${REMOTE_DIR}"
log "Excluded patterns: ${EXCLUDE_PATTERNS[*]}"
log "---"
log "Running initial sync …"
run_sync

# ---------------------------------------------------------------------------
# File watcher loop
# ---------------------------------------------------------------------------
log "Watching for changes (debounce = ${DEBOUNCE_SECS}s) …"
log "---"

# inotifywait exits when the --monitor child is killed, so we run it in the
# background and read its output through a named pipe (or just a pipe).
#
# Events we care about:
#   modify, create, delete, move, attrib
#
# We use --monitor so it keeps running after each event.
inotifywait \
  --monitor \
  --recursive \
  --event modify \
  --event create \
  --event delete \
  --event move \
  --event attrib \
  --exclude "${INOTIFY_EXCLUDE_REGEX}" \
  --format '%w%f' \
  "${LOCAL_DIR}" |
  while IFS= read -r changed_file; do

    # Skip files inside excluded dirs (belt-and-suspenders)
    skip=0
    for pat in "${EXCLUDE_PATTERNS[@]}"; do
      if [[ "$changed_file" == *"${pat%/}"* ]]; then
        skip=1
        break
      fi
    done
    [[ $skip -eq 1 ]] && continue

    # Debounce: wait a bit, then sync.  While we wait, drain any subsequent
    # events so we batch them all into one sync.
    while true; do
      # Read any additional events arriving within the debounce window
      # If read succeeds (another event arrived), reset the timer.
      # If it times out, break out and sync.
      if ! read -t "$DEBOUNCE_SECS" -r extra_file; then
        break
      fi
      # Skip excluded paths among the extra events too
      skip=0
      for pat in "${EXCLUDE_PATTERNS[@]}"; do
        if [[ "$extra_file" == *"${pat%/}"* ]]; then
          skip=1
          break
        fi
      done
      [[ $skip -eq 1 ]] && continue

      # The extra event was legit — we stay in the inner loop waiting
      # for more, resetting the timer each time.
      :
    done

    run_sync
  done

