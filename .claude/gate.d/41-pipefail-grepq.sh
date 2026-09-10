#!/usr/bin/env bash
# gate-stage: pipefail 下用 grep -q 收尾的管道（命中会被读成没命中）
#
# 判据：一个设了 `pipefail` 的脚本里，凡是 `… | grep -q…` 这种**以提前退出的 grep 收尾**的管道，
# 一律判红。理由是它的判决会随机翻向：`grep -q` 一命中就退出，前段还没写完就吃 SIGPIPE，
# 于是管道整体返回 141，而 `if` 把它读成「没命中」——**明明命中，判成没有**。
#
# ⚠️ **这条是实测出来的，不是理论**：2026-09-10 的整轮门禁里 `10-kb-rot.sh` 判红一次，
# 说某个实验的状态变动提交「没有动 decisions.md」；同一份输入空载连跑 23 次全绿，
# 两次输出里的提交号与相邻那一格逐字相同，**只有 `git show --stat … | grep -q` 这条管道的退出码翻了**。
# 复现机制：`set -uo pipefail; yes X | head -200000 | grep -q X` 稳定返回 141。
# 前段是 `git show` / `git diff` 时输出可以很大，机器越忙越容易撞上——
# 也就是说，**门禁越是在真干活的时候，越容易假红**。
#
# 出路是 `.claude/singlefs-ai-sop/rules/command-safety.md` 自己写的那条：
# 先把输出落到变量或文件，判完退出码再处理。
#
# 不管的两种：① 脚本没设 pipefail（管道退出码本来就只看最后一段）；
# ② `grep -q … <<<"$var"` 这种没有前段的形态。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"

mapfile -t files < <(
  find "$ROOT/.claude/gate.d" "$ROOT/.claude/scripts" "$ROOT/research/scripts" \
       -maxdepth 1 -name '*.sh' -type f 2>/dev/null | sort
)

checked=0
bad=()
for f in "${files[@]}"; do
  grep -qE '^[[:space:]]*set .*pipefail' "$f" || continue
  checked=$((checked + 1))
  while IFS= read -r hit; do
    bad+=("${f#"$ROOT"/}:$hit")
    # 只看非注释行：这条坑本身在好几个脚本里留着警告注释，
    # 把注释也算进来会让检查报出一堆它自己写的字（实测）。
  done < <(grep -nE '^[[:space:]]*[^#[:space:]].*\|[[:space:]]*grep[[:space:]]+(-[A-Za-z]*q[A-Za-z]*)' "$f" | cut -c1-120)
done

if ((${#bad[@]})); then
  echo "  ✗ pipefail 下有 ${#bad[@]} 条管道以提前退出的 grep -q 收尾，判决会随机翻向："
  printf '      %s\n' "${bad[@]}"                    # gate-lint:detail
  echo "  → 怎么办：把前段的输出先落到变量，再对变量判——"
  echo "            out=\$(前段 || true); if grep -q '模式' <<<\"\$out\"; then …"
  echo "            前段是 git show / git diff 时尤其要改：输出越大越容易吃 SIGPIPE。"
  exit 1
fi
echo "  ✓ 没有 pipefail 下以 grep -q 收尾的管道（检查了 $checked 个设了 pipefail 的脚本）"
