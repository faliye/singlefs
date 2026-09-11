#!/usr/bin/env bash
# 三方论证的「本地 LLM」那一腿。
#
#   ask-local.sh <prompt 文件>        提示从文件读，答案打到 stdout
#   cat p.md | ask-local.sh           也接 stdin
#
# 上游是 ~/code/ai-center 的 OpenAI 兼容网关（:8200，前置 vLLM）。
# 不传 max_tokens / thinking_token_budget —— 网关按两本账补值并按形状学习，
# 传一个偏小的值等于把自己按死在那个值上（ai-center kb/token-budget.md）。
set -uo pipefail

CENTER="${AI_CENTER_DIR:-$HOME/code/ai-center}"
URL="${AI_CENTER_URL:-http://127.0.0.1:8200/v1/chat/completions}"
TIMEOUT="${ASK_LOCAL_TIMEOUT:-900}"

KEY="$(sed -n 's/^AI_CENTER_KEY_VSCODE_CHAT=//p' "$CENTER/.env.tenants" 2>/dev/null)"
[[ -n "$KEY" ]] || { echo "ask-local: 取不到 AI_CENTER_KEY_VSCODE_CHAT（$CENTER/.env.tenants）" >&2; exit 2; }

PROMPT_PATH="${1:-}"
PROMPT="$(cat "${1:-/dev/stdin}")"
[[ -n "$PROMPT" ]] || { echo "ask-local: 提示为空" >&2; exit 2; }

# 测试缝：ASK_LOCAL_FAKE_TEXT 指向一份现成正文时跳过网关。
# fs-design.md 硬要求 2「每条分支必须能被测试强制进入」——判红那条分支此前进不去，
# 于是「判红时把证据留下来」这件事没有任何东西验得了。
if [[ -n "${ASK_LOCAL_FAKE_TEXT:-}" ]]; then
  RESP="$(FAKE="$ASK_LOCAL_FAKE_TEXT" python3 -c 'import json,os;print(json.dumps({"choices":[{"message":{"content":open(os.environ["FAKE"],encoding="utf-8").read()}}]}))')"
else
REQ="$(PROMPT="$PROMPT" python3 -c 'import json,os;print(json.dumps({"model":"local","messages":[{"role":"user","content":os.environ["PROMPT"]}]}))')"

RESP="$(curl -sS -m "$TIMEOUT" -H "X-Api-Key: $KEY" -H "Content-Type: application/json" -d "$REQ" "$URL")" \
  || { echo "ask-local: 请求失败（超时 ${TIMEOUT}s？）" >&2; exit 3; }
fi

# 读不到 ≠ 读到 0：空正文一律报错，不许当成「模型没话说」
TXT="$(mktemp)"
trap 'rm -f "$TXT"' EXIT
RESP="$RESP" TXT="$TXT" python3 -c '
import sys,json,os
raw=os.environ["RESP"]
try: d=json.loads(raw)
except Exception as e:
    sys.stderr.write("ask-local: 响应不是 JSON: %s\n"%e); sys.exit(3)
if isinstance(d,dict) and "error" in d:
    sys.stderr.write("ask-local: 网关报错 %s\n"%json.dumps(d["error"],ensure_ascii=False)); sys.exit(3)
try: m=d["choices"][0]["message"]
except Exception:
    sys.stderr.write("ask-local: 响应里没有 choices: %s\n"%raw[:400]); sys.exit(3)
c=(m.get("content") or "").strip()
if not c:
    # 字段名是 reasoning，不是 reasoning_content —— 写错的话这条诊断永远报 0 字，
    # 「正文为空」与「连思考都没有」就分不开了（2026-08-28 实测网关返回的键是 reasoning）
    sys.stderr.write("ask-local: 正文为空（thinking %d 字）——整轮作废，不当成 0\n"%len(m.get("reasoning") or ""))
    sys.exit(4)
open(os.environ["TXT"],"w",encoding="utf-8").write(c)
'
rc=${PIPESTATUS[0]:-$?}
[[ $rc -eq 0 ]] || exit $rc

# 判红那一轮的正文必须留下来：tooling.md 的坏样本表就是靠这些文件撑着，
# 而 $TXT 是 mktemp、退出时被 trap 删掉 —— 此前作废轮的原样输出全靠调用方记得重定向
# stdout，一次没记得就没了（2026-09-07 实测丢了一份）。
save_void() {
  local dir base n
  [[ -z "${VOID_SAVED:-}" ]] || return 0   # 两个检测器都判红时只留一份
  if [[ -n "${PROMPT_PATH:-}" && -f "$PROMPT_PATH" ]]; then
    dir="$(dirname "$PROMPT_PATH")"; base="$(basename "$PROMPT_PATH" .md)"; base="${base%-prompt}"
  else
    dir="${TMPDIR:-/tmp}"; base="ask-local-$(date +%Y%m%d-%H%M%S)"
  fi
  n=1
  while [[ -e "$dir/$base-output-void$n.md" ]]; do n=$((n+1)); done
  cp "$TXT" "$dir/$base-output-void$n.md"
  VOID_SAVED=1
  echo "ask-local: 作废轮的原样输出已留存 $dir/$base-output-void$n.md" >&2
}

# ── 字词损坏闸 ───────────────────────────────────────────────────────────
# 本地腿是 4-bit 量化模型，中文输出会退化性复读（实测中文 4/5 判红、英文 0/5）。
# **这条以前只是规则里的一句提醒，提醒句拦不住手敲命令**——做成会拒绝的检查才拦得住
# （singlefs-ai-sop/rules/show-me-test.md）。
# ⚠️ **两个检测器，查的是不同的损坏类**，缺一个就有一整类漏过去：
#   corruption-check.py  复读（多吐了）+ 成对标记落单（整段掉了）
#   oov-check.py         拼接（两个词粘死：`configurationing` `batchinggroup`）
# 实测：一轮含 `batchinggroup`×2 的输出被前者判绿并当作干净证据用进了 D23 轴二论证。
for CHECK in "$(dirname "$0")/corruption-check.py" "$(dirname "$0")/oov-check.py"; do
if [[ -x "$CHECK" || -f "$CHECK" ]]; then
  VERDICT="$(python3 "$CHECK" "$TXT" "$1" 2>&1)"; crc=$?
  case $crc in
    0) : ;;  # 判绿
    1)
      echo "ask-local: 输出判定为字词损坏 —— 按 .claude/rules/three-way-inference.md 这一轮作废" >&2
      echo "$VERDICT" >&2
      save_void
      echo "下一步：改用英文提示重跑（实测英文 0/5 损坏），或设 ASK_LOCAL_ALLOW_CORRUPT=1 强行采用" >&2
      [[ "${ASK_LOCAL_ALLOW_CORRUPT:-0}" == "1" ]] || exit 5
      ;;
    *)
      # **「检测器自己崩了」不等于「判红」，更不等于「判绿」。**
      # 混在一起的话，一个装不上的检测器会把每一轮都判成损坏，
      # 而修好它的人只会去调提示词 —— 报「没做」才指得出真正的下一步。
      echo "ask-local: 字词损坏检查**没跑成**（退出码 $crc），不是通过也不是判红" >&2
      echo "$VERDICT" >&2
      ;;
  esac
else
  echo "ask-local: 找不到 $CHECK —— **没做**字词损坏检查，不是通过了" >&2
fi
done

# 正文放到最后才打：此前 python 里先 print 再跑损坏闸，判红那一轮的正文已经进了调用方的重定向文件，
# 一份作废输出顶着 `-output-s1.md` 这种合法名字落了盘（2026-09-12 实测，与 void 副本只差一个换行）。
# 判红且没设 ASK_LOCAL_ALLOW_CORRUPT 时上面已经 exit 5，走不到这里。
cat "$TXT"; echo   # 与原先 print(c) 一样补一个换行
