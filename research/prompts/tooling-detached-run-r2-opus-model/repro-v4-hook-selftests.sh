#!/usr/bin/env bash
# 复现 V4 钩子那一半：三种错实现（h4 子 agent 闸只认 cp / ln、不认 install / mv；h5 重型闸不认 heredoc 写出的拷贝；
#   h7 子 agent 闸解相对路径不跟 cd），各在一份 .claude 拷贝里改上，跑那道钩子的 --selftest。不起单元。
#   control 是不改的拷贝（拷贝里跑本身对不对的对照）。期望：control 与 h4、h5、h7 都 ✓ ⇒ 这三种错实现自证放得过；
#   样本表 cases-f1.json 20 行对这三种也都照旧（hook-probes-v2.txt 的 V4 段已量过）。
# 用法：bash repro-v4-hook-selftests.sh <仓根>（主 agent 带前缀跑，例如 SINGLEFS_HEAVY_TESTS=commit）
set -u
repo="$1"; work="$(mktemp -d "${TMPDIR:-/tmp}/opusr2-rv4h.XXXXXX")"
for m in control h4 h5 h7; do
  mkdir -p "$work/$m/research"; rsync -a --exclude target --exclude .git "$repo/.claude" "$work/$m/"; rsync -a "$repo/research/scripts" "$work/$m/research/"
done
python3 - "$work" <<'PY'
import sys
w = sys.argv[1]
def patch(path, old, new):
    s = open(path, encoding="utf-8").read(); assert s.count(old) == 1, (path, old[:60]); open(path, "w", encoding="utf-8").write(s.replace(old, new, 1))
patch(f"{w}/h4/.claude/hooks/bash-command-detector.sh", 'DETACHED_RUN_COPYING_COMMANDS = {"cp", "install", "ln", "mv"}', 'DETACHED_RUN_COPYING_COMMANDS = {"cp", "ln"}')
patch(f"{w}/h5/.claude/hooks/heavy-test-guard.sh", "    if entry is None:\n        return None\n    if entry.copied_from is not None:",
      "    if entry is None or entry.copied_from is None:\n        return None\n    if entry.copied_from is not None:")
patch(f"{w}/h7/.claude/hooks/bash-command-detector.sh",
      "        resolved = shell_words.resolve_path(command.directory, os.path.expanduser(word)) if command.directory else None\n        recognized = shell_words.detached_run_script(word, command.directory)",
      "        resolved = shell_words.resolve_path(directory, os.path.expanduser(word)) if directory else None\n        recognized = shell_words.detached_run_script(word, directory)")
PY
for m in control h4 h7; do bash "$work/$m/.claude/hooks/bash-command-detector.sh" --selftest > "$work/$m-detector.out" 2>&1; echo "$m bash-command-detector --selftest 退 $?：$(tail -n 1 "$work/$m-detector.out" | cut -c1-60)"; done
for m in control h5; do bash "$work/$m/.claude/hooks/heavy-test-guard.sh" --selftest > "$work/$m-heavy.out" 2>&1; echo "$m heavy-test-guard --selftest 退 $?：$(tail -n 1 "$work/$m-heavy.out" | cut -c1-60)"; done
rm -rf -- "$work"
