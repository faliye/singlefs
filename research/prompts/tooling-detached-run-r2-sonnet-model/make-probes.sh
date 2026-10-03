#!/usr/bin/env bash
# 重建 feed.py 用的探针（只拷文件、不执行被判脚本）。REPO 是装了冻结副本七份文件加 .claude/singlefs-ai-sop 的临时树。
set -euo pipefail
REPO=${1:?临时树根}; PROBE=${2:?探针目录}
mkdir -p "$PROBE"
cp "$REPO/research/scripts/detached-run.sh" "$PROBE/renamed-copy.sh"
sed 's/^# detached-run-script-signature:/# 签名行删掉了:/' "$REPO/research/scripts/detached-run.sh" > "$PROBE/nosig.sh"
sed 's/^# detached-run-script-signature:/# detached-run-script-signatur:/' "$REPO/research/scripts/detached-run.sh" > "$PROBE/onechar.sh"
ln -sf "$REPO/research/scripts/detached-run.sh" "$PROBE/link.sh"
python3 - "$REPO" "$PROBE" <<'PY'
import sys
t = open(sys.argv[1] + '/research/scripts/detached-run.sh').read().split('\n')
open(sys.argv[2] + '/sigafter4k.sh', 'w').write('\n'.join([t[0]] + ['# pad ' + 'x' * 90] * 50 + [t[1]] + t[2:]))
PY
cp "$PROBE/renamed-copy.sh" "$PROBE/CRLF.sh"; sed -i 's/$/\r/' "$PROBE/CRLF.sh"
