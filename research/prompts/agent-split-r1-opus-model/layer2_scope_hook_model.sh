#!/usr/bin/env bash
# 攻方腿模型：提案第五节第二层 `.claude/hooks/agent-write-scope.sh <路径模式…>` 还没写；这里按两种自然写法各实现一份，
# 用构造的 PreToolUse stdin JSON 喂它，看哪些输入判错。读 JSON 的那一行照抄仓里唯一现成的 hook
# （.claude/hooks/refuse-overwrite-untracked.sh 第 54–55 行：只取 tool_input.file_path，取不到就 exit 0）。
#   naive      ：去掉项目根前缀后拿 bash 的 [[ == 模式 ]] 比
#   normalized ：先 realpath -m（解 .. 与符号链接）再比
set -uo pipefail
work="${TMPDIR:-/tmp}/agent-split-r1-opus-layer2.$$"; rm -rf "${work:?}"; mkdir -p "$work/repo/.claude/kb" "$work/repo/crates/core/src" "$work/repo/research/prompts"
root="$work/repo"; echo 'fn a(){}' > "$root/crates/core/src/allocator.rs"
ln -s ../../crates/core/src "$root/.claude/kb/src-link"          # 仓里某处现成的符号链接
decide() { # $1 模式 naive|normalized；$2 stdin JSON；其余是路径模式；返回 0 放行、2 拒绝
  local mode="$1" input="$2"; shift 2
  local target; target="$(python3 -c 'import json,sys; data=json.loads(sys.argv[1]); print((data.get("tool_input") or {}).get("file_path",""))' "$input" 2>/dev/null)"
  [[ -n "$target" ]] || return 0
  [[ "$mode" == normalized ]] && target="$(realpath -m "$target")"
  local rel="${target#"$root"/}" pattern
  for pattern in "$@"; do [[ "$rel" == $pattern ]] && return 0; done
  return 2
}
run() { local label="$1" json="$2" want="$3"; shift 3
  for mode in naive normalized; do decide "$mode" "$json" "$@"; local rc=$?
    local verdict; if [[ $rc == 0 ]]; then verdict=放行; else verdict=拒绝; fi
    echo "$label [$mode] → $verdict（该：$want）"; done; }
kb='.claude/kb/*'
run "A Edit 带 .. 跳出 kb 改 crates" "{\"tool_name\":\"Edit\",\"tool_input\":{\"file_path\":\"$root/.claude/kb/../../crates/core/src/allocator.rs\"}}" 拒绝 "$kb"
run "B Write 经 kb 里的符号链接写 crates" "{\"tool_name\":\"Write\",\"tool_input\":{\"file_path\":\"$root/.claude/kb/src-link/allocator.rs\"}}" 拒绝 "$kb"
run "C NotebookEdit 写 crates（参数名是 notebook_path）" "{\"tool_name\":\"NotebookEdit\",\"tool_input\":{\"notebook_path\":\"$root/crates/core/src/x.ipynb\"}}" 拒绝 "$kb"
leg='research/prompts/*-sonnet-output.md'
run "D 正推腿的静态模式覆盖别的轮已冻结的产出" "{\"tool_name\":\"Write\",\"tool_input\":{\"file_path\":\"$root/research/prompts/c143-r3-sonnet-output.md\"}}" 拒绝 "$leg"
run "E 模式里的 * 跨目录：写进别的腿的模型目录" "{\"tool_name\":\"Write\",\"tool_input\":{\"file_path\":\"$root/research/prompts/r1-opus-model/x-sonnet-output.md\"}}" 拒绝 "$leg"
run "F 经符号链接的项目根进来的合法写" "{\"tool_name\":\"Write\",\"tool_input\":{\"file_path\":\"$work/link-to-repo/.claude/kb/a.md\"}}" 放行 "$kb"
ln -s "$root" "$work/link-to-repo"
run "F' 同上（链接建好之后）" "{\"tool_name\":\"Write\",\"tool_input\":{\"file_path\":\"$work/link-to-repo/.claude/kb/a.md\"}}" 放行 "$kb"
rm -rf "${work:?}"
