#!/usr/bin/env bash
# Build the draft copy used by collide.sh: repo copy without target/.git, ask-local.sh gets a fake-mode delay
# (ASK_LOCAL_FAKE_DELAY, draft copy only), and a fake AI_CENTER_DIR with a dummy key so fake mode never touches the gateway.
# usage: bash setup.sh <draft dir>
set -euo pipefail
D=${1:?draft dir}
REPO=$(cd "$(dirname "$0")/../../.." && pwd)
mkdir -p "$D/fakecenter" "$D/collide"
rsync -a --exclude target --exclude .git "$REPO/" "$D/repo/"
printf 'AI_CENTER_KEY_VSCODE_CHAT=dummy\n' > "$D/fakecenter/.env.tenants"
cd "$D/repo/research/scripts"
python3 - <<'PY'
src = open("ask-local.sh", encoding="utf-8").read()
old = 'if [[ -n "${ASK_LOCAL_FAKE_TEXT:-}" ]]; then\n'
assert src.count(old) == 1
open("ask-local.sh.tmp", "x", encoding="utf-8").write(src.replace(old, old + '  sleep "${ASK_LOCAL_FAKE_DELAY:-0}"   # draft-copy only: simulate request latency\n'))
PY
chmod --reference=ask-local.sh ask-local.sh.tmp && mv ask-local.sh.tmp ask-local.sh
diff "$REPO/research/scripts/ask-local.sh" ask-local.sh || true
