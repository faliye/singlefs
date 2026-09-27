#!/usr/bin/env bash
# 项目自己的脚本 source 这一份，不直接 source 规范副本里的 preflight.sh（.claude/singlefs-ai-sop/rules/preflight-discipline.md 的开头先判）。
# 规范副本 .claude/singlefs-ai-sop/ 被 .gitignore 挡着，git worktree 里没有它：提交时在「HEAD + 暂存区」的临时树里跑 54 号 --full
# 就是这样。直接 source 副本会落空，preflight 成了找不到的命令，紧跟着的 set -- 把参数整个清掉，脚本不带参数照跑，一声不响。
# 这一份随仓走，每棵树里都在：先用这棵树里的副本，没有就用主仓（git 的公共目录所在的那一份仓）里的，
# 两处都没有就定义一个当场判红的 preflight，不让脚本静默往下走。只定义函数、不改 shell 选项，变量用完就清掉。
project_preflight_shim_directory="${BASH_SOURCE[0]%/*}"
if [[ "$project_preflight_shim_directory" == "${BASH_SOURCE[0]}" ]]; then project_preflight_shim_directory=.; fi
project_preflight_tree_root="$(cd "$project_preflight_shim_directory/../.." && pwd -P)"
project_preflight_library="$project_preflight_tree_root/.claude/singlefs-ai-sop/scripts/preflight.sh"
if [[ ! -f "$project_preflight_library" ]]; then
  project_preflight_common_directory="$(git -C "$project_preflight_tree_root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null || true)"
  if [[ -n "$project_preflight_common_directory" ]]; then
    project_preflight_library="${project_preflight_common_directory%/*}/.claude/singlefs-ai-sop/scripts/preflight.sh"
  fi
fi
if [[ -f "$project_preflight_library" ]]; then
  # shellcheck source=../singlefs-ai-sop/scripts/preflight.sh
  source "$project_preflight_library"
else
  PROJECT_PREFLIGHT_MISSING_LIBRARY="$project_preflight_library"
  preflight() {
    echo "  ✗ $1：找不到规范副本里的 preflight.sh（这棵树与主仓里都没有：$PROJECT_PREFLIGHT_MISSING_LIBRARY），准入与运行条件一条都判不了，拒绝往下跑" >&2
    echo "     → 怎么办：规范副本 .claude/singlefs-ai-sop/ 不随 git 走；在主仓里装好它（bash .claude/singlefs-ai-sop/install.sh），临时树从主仓的副本里找得到。" >&2
    exit 1
  }
fi
unset project_preflight_shim_directory project_preflight_tree_root project_preflight_library project_preflight_common_directory
