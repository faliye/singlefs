#!/usr/bin/env bash
# 同 local-repro.sh，但**显式传采样参数**，用来判定复读是不是采样默认值造成的。
#
#   local-repro-tuned.sh <标签> <提示文件> <N> <presence_penalty>
#
# 模型自带的 generation_config.json 只有 temperature/top_k/top_p，**没有任何抗复读项**。
# 这一条就是要证明：加上 presence_penalty 之后复读率降不降。
set -uo pipefail
TAG="$1"; PROMPT_FILE="$2"; N="${3:-5}"; PP="${4:-1.5}"; RP="${5:-}"
CENTER="${AI_CENTER_DIR:-$HOME/code/ai-center}"
URL="${AI_CENTER_URL:-http://127.0.0.1:8200/v1/chat/completions}"
OUT="${OUT_DIR:-$HOME/code/singlefs/research/results/corruption}"
mkdir -p "$OUT"
KEY="$(sed -n 's/^AI_CENTER_KEY_VSCODE_CHAT=//p' "$CENTER/.env.tenants" 2>/dev/null)"
[[ -n "$KEY" ]] || { echo "取不到 key" >&2; exit 2; }
PROMPT="$(cat "$PROMPT_FILE")"
[[ -n "$PROMPT" ]] || { echo "提示为空" >&2; exit 2; }
REQ="$(PROMPT="$PROMPT" PP="$PP" RP="$RP" python3 -c '
import json,os
body={"model":"local","presence_penalty":float(os.environ["PP"]),
      "messages":[{"role":"user","content":os.environ["PROMPT"]}]}
# repetition_penalty 是 vLLM 的扩展参数，惩罚的是**已出现过的 token 再次出现**，
# 比 presence_penalty 更对症紧邻复读。留空表示不传。
if os.environ.get("RP"): body["repetition_penalty"]=float(os.environ["RP"])
print(json.dumps(body))')"
for i in $(seq 1 "$N"); do
  raw="$OUT/$TAG-$i.raw.json"; txt="$OUT/$TAG-$i.txt"; rm -f "$raw" "$txt"
  curl -sS -m 1800 -H "X-Api-Key: $KEY" -H "Content-Type: application/json" \
       -d "$REQ" "$URL" -o "$raw" || { echo "$TAG-$i: curl 失败"; continue; }
  RAW="$raw" TXT="$txt" python3 - <<'PY'
import json, os, sys
d = json.loads(open(os.environ["RAW"], "rb").read().decode("utf-8"))
if "error" in d:
    sys.stderr.write("网关报错 %s\n" % json.dumps(d["error"], ensure_ascii=False)); sys.exit(3)
c = (d.get("choices") or [{}])[0].get("message", {}).get("content") or ""
open(os.environ["TXT"], "w", encoding="utf-8").write(c)
PY
  [[ -s "$txt" ]] || { echo "$TAG-$i: 正文为空"; continue; }
  echo "$TAG-$i pp=$PP rp=${RP:-无} $(python3 "$(dirname "$0")/corruption-check.py" "$txt" | head -1)"
done
