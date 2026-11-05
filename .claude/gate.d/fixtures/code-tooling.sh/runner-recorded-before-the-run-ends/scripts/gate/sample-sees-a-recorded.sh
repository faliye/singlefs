#!/usr/bin/env bash
# 样本：在样本自己的判定库（prune-server 里，按样本根 git common-dir 的命名空间：scripts/prune-server/prune_server_client.py 文件头「键空间」）里
# 找 a 那一条的判绿记录：admission.py 的 VerdictLibrary 那一份，族 unit-verdicts、种类 code-tooling.sh:research-script-selftests、
# 库目录 <git common-dir>/singlefs-unit-verdicts/<种类>、相对路径 items/<a 那一条命令原文的 sha256>.json；在而 verdict 是 green 退 0，别的退 1。
# 样本根的 .selftest-environment 设 SINGLEFS_THREAD_CAP=1，这一格每批一条、按表次序起：b 起的时候 a 已经跑完，整格还没收尾。
# 读库经客户端命令行 read（客户端文件头「命令行」），客户端在这一道所在的仓里（这一格给样本设的 CODE_TOOLING_SAMPLE_STAGE_REPOSITORY 指）：
# 用 python3 -m 起、模块所在目录经 PYTHONPATH 给，不经变量指的路径起脚本、不按路径载入模块，这一条自证的单元键算得出
# （样本根里没有这个模块，-m 起的照外部程序认，scripts/gate/lib_function_units.py 的 repository_module）。服务连不上客户端退 75，这里照原码退。
[[ "${1:-}" == --selftest ]] || exit 2
[[ -n "${CODE_TOOLING_SAMPLE_STAGE_REPOSITORY:-}" && -f "$CODE_TOOLING_SAMPLE_STAGE_REPOSITORY/scripts/prune-server/prune_server_client.py" ]] || {
  echo "  ✗ b 的自证判错：CODE_TOOLING_SAMPLE_STAGE_REPOSITORY 没指到带 scripts/prune-server/prune_server_client.py 的仓（code-tooling 的 research-script-selftests 格只给样本设它）"
  exit 1
}
sample_root="$(dirname "${BASH_SOURCE[0]}")/../.."
common_directory="$(git -C "$sample_root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)"
[[ -n "$common_directory" ]] || { echo "  ✗ b 的自证判错：样本根不在 git 工作树里（setup.sh 没 git init？），没有判定库"; exit 1; }
kind="code-tooling.sh:research-script-selftests"
item_digest="$(printf '%s' "bash scripts/gate/sample-pass-a.sh --selftest" | sha256sum)"
item="items/${item_digest%% *}.json"
if stored="$(PYTHONPATH="$CODE_TOOLING_SAMPLE_STAGE_REPOSITORY/scripts/prune-server" python3 -m prune_server_client read unit-verdicts "$kind" \
     "$common_directory/singlefs-unit-verdicts/$kind" "$item")"; then
  read_exit=0
else
  read_exit=$?
fi
if [[ "$read_exit" == 1 ]]; then
  echo "  ✗ b 的自证判错：a 已经跑完，它的判绿记录此刻不在判定库 $common_directory/singlefs-unit-verdicts/$kind 的 $item 里（整格收尾才记，中途被停 a 就白跑了）"
  exit 1
fi
if [[ "$read_exit" != 0 ]]; then
  echo "  ✗ b 的自证判错：客户端 read 退 $read_exit（读失败，见上面客户端打的那一行）"
  exit "$read_exit"
fi
verdict="$(python3 -c 'import json, sys; print(json.loads(sys.argv[1]).get("verdict", ""))' "$stored" 2>/dev/null)"
if [[ "$verdict" != green ]]; then
  echo "  ✗ b 的自证判错：a 那一条的记录在 $item，verdict 却是「$verdict」（要 green）"
  exit 1
fi
echo "  ✓ b 的自证 1 条都判对：a 的判绿记录此刻已在判定库里（$common_directory/singlefs-unit-verdicts/$kind 的 $item）"
