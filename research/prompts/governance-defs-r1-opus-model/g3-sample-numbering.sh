#!/usr/bin/env bash
# G3 模型：照改后 three-way-local-attack.md 第 3、4 步的重定向写法起 ask-local.sh（用它自己的测试缝 ASK_LOCAL_FAKE_TEXT，不连网关），
# 看样本号 s<n> 已经存在时各写法的结果。每一例在独立的 Bash 子进程里起，与 Claude Code 每次 Bash 调用一样从 noclobber 关着开始。
# 用法：bash g3-sample-numbering.sh <草稿目录>
set -uo pipefail
REPO=/home/user/singlefs
SCRATCH=${1:?草稿目录}
base="$(mktemp -d -p "$SCRATCH" g3.XXXX)"
mkdir -p "$base/center" "$base/prompts"
printf 'AI_CENTER_KEY_VSCODE_CHAT=fake-key-for-model\n' > "$base/center/.env.tenants"
prompt="$base/prompts/rX-local-attack.md"
printf 'Answer each numbered item about the write range of the scribe and the mutation table.\n' > "$prompt"
clean1="$base/clean1.txt"; clean2="$base/clean2.txt"
printf '1. The scribe may write the table. What would refute it: a rejected edit.\n2. The mutation table has six fields. What would refute it: a seventh field.\n' > "$clean1"
printf '1. The runner appends one row. What would refute it: two rows appended.\n' > "$clean2"
run() { # <说明> <命令，在 $base 里跑>
  local rc
  ( cd "$base" && AI_CENTER_DIR="$base/center" bash -c "$2" ) 2> "$base/stderr.txt"; rc=$?
  printf '%s\n  命令：%s\n  退出码=%s；s1=%s；stderr 首行：%s\n' "$1" "$2" "$rc" \
    "$( [[ -e "$base/prompts/rX-local-attack-output-s1.md" ]] && sha256sum < "$base/prompts/rX-local-attack-output-s1.md" | cut -c1-16 || echo 不存在)" \
    "$(head -n 1 "$base/stderr.txt")"
}
ask="bash $REPO/research/scripts/ask-local.sh prompts/rX-local-attack.md"
echo "clean1 的 sha256 前 16 位：$(sha256sum < "$clean1" | cut -c1-16)；clean2：$(sha256sum < "$clean2" | cut -c1-16)"
run "H0 第一次调用，s1 新号（第 3 步字面写法）" "ASK_LOCAL_FAKE_TEXT=$clean1 $ask > prompts/rX-local-attack-output-s1.md"
run "H1 s1 已是一份干净样本，再用第 3 步字面写法落进 s1" "ASK_LOCAL_FAKE_TEXT=$clean2 $ask > prompts/rX-local-attack-output-s1.md"
run "H2 同上，但照共用约束「新建文件一律排他」加 noclobber" "set -o noclobber; ASK_LOCAL_FAKE_TEXT=$clean1 $ask > prompts/rX-local-attack-output-s1.md"
: > "$base/prompts/rX-local-attack-output-s2.md"
run "H3 判红留下的空 s2，不加 >|、直接 >（第 4 步说 > 写不进）" "ASK_LOCAL_FAKE_TEXT=$clean2 $ask > prompts/rX-local-attack-output-s2.md"
echo "  s2 此刻大小：$(stat -c %s "$base/prompts/rX-local-attack-output-s2.md") 字节"
mkdir -p "$base/nochecker"; cp "$REPO/research/scripts/ask-local.sh" "$base/nochecker/ask-local.sh"
run "H4 检测器找不到（退出码 6 那一支）" "ASK_LOCAL_FAKE_TEXT=$clean1 bash nochecker/ask-local.sh prompts/rX-local-attack.md > prompts/rX-local-attack-output-s3.md"
echo "  s3 大小：$(stat -c %s "$base/prompts/rX-local-attack-output-s3.md") 字节；作废副本：$(ls "$base/prompts" | grep -c void) 份"
