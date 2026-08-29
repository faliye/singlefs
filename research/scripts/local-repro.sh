#!/usr/bin/env bash
# 复现本地腿的字词损坏：同一份提示跑 N 轮，**保留原始 HTTP 响应**。
#
#   local-repro.sh <标签> <提示文件> <N>
#
# 与 ask-local.sh 的唯一区别：它把 curl 的原始字节存下来。
# 这是为了分辨**损坏在模型那一侧，还是在我们的解包那一侧**——
# 只看 ask-local.sh 的输出，这两者长得一模一样。
set -uo pipefail
TAG="$1"; PROMPT_FILE="$2"; N="${3:-5}"
CENTER="${AI_CENTER_DIR:-$HOME/code/ai-center}"
URL="${AI_CENTER_URL:-http://127.0.0.1:8200/v1/chat/completions}"
OUT="${OUT_DIR:-$HOME/code/singlefs/research/results/corruption}"
mkdir -p "$OUT"

KEY="$(sed -n 's/^AI_CENTER_KEY_VSCODE_CHAT=//p' "$CENTER/.env.tenants" 2>/dev/null)"
[[ -n "$KEY" ]] || { echo "local-repro: 取不到 key" >&2; exit 2; }
PROMPT="$(cat "$PROMPT_FILE")"
[[ -n "$PROMPT" ]] || { echo "local-repro: 提示为空" >&2; exit 2; }
REQ="$(PROMPT="$PROMPT" python3 -c 'import json,os;print(json.dumps({"model":"local","messages":[{"role":"user","content":os.environ["PROMPT"]}]}))')"

for i in $(seq 1 "$N"); do
  raw="$OUT/$TAG-$i.raw.json"
  txt="$OUT/$TAG-$i.txt"
  rm -f "$raw" "$txt"
  curl -sS -m 1800 -H "X-Api-Key: $KEY" -H "Content-Type: application/json" \
       -d "$REQ" "$URL" -o "$raw"
  rc=$?
  if [[ $rc -ne 0 ]]; then echo "$TAG-$i: curl 失败 rc=$rc"; continue; fi
  # 原始字节里的替换字符：EF BF BD。在这里数，才知道是不是模型发出来的。
  raw_fffd=$(grep -c $'\xef\xbf\xbd' "$raw" 2>/dev/null || true)
  RAW="$raw" TXT="$txt" python3 - <<'PY'
import json, os, sys
raw = open(os.environ["RAW"], "rb").read()
try:
    d = json.loads(raw.decode("utf-8"))
except Exception as e:
    sys.stderr.write("解析失败: %s\n" % e); sys.exit(3)
c = (d.get("choices") or [{}])[0].get("message", {}).get("content") or ""
open(os.environ["TXT"], "w", encoding="utf-8").write(c)
PY
  [[ -s "$txt" ]] || { echo "$TAG-$i: 正文为空"; continue; }
  echo "$TAG-$i raw_fffd_lines=$raw_fffd $(python3 "$(dirname "$0")/corruption-check.py" "$txt" | head -1)"
done
