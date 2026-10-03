#!/usr/bin/env bash
# 崩溃放量入口 research/scripts/crash-amplification.sh 在仓拷贝上用假分项驱动、假清场跑几种形状，打每一趟的退出码、ITEM / TOTAL 行与成功句。
# 用法：entry-probe.sh <仓拷贝根>；不编、不跑 cargo、不连第二台（多机配置与显卡配置指到不在的文件）。复跑挂钟：几秒。
set -uo pipefail
root="$1"
work="$(mktemp -d "${TMPDIR:-/tmp}/entry-probe.XXXXXX")"
trap 'rm -rf -- "${work:?}"' EXIT
cat > "$work/driver.sh" <<'STUB'
#!/usr/bin/env bash
# 假分项驱动：形状按 SHAPE_<测试目标>（没给按 SHAPE）打 CRASH_AMPLIFICATION 行；expand_up_to 照交进来的
target="$1"; expand=""; previous=""
for argument in "$@"; do [[ "$previous" == --expand-up-to ]] && expand="$argument"; previous="$argument"; done
variable="SHAPE_$(printf '%s' "$target" | tr -c 'A-Za-z0-9_' '_')"; shape="${!variable:-${SHAPE:-clean}}"
base="mode=single_host_cpu share=0/1 cards=0 states=10 states_checked_here=7 blocks_checked_here=2 imported_blocks=0 unjudged_blocks=0"
case "$shape" in
  clean) echo "CRASH_AMPLIFICATION $base states_reused=3 red=0 expand_up_to=$expand exhaustive=true" ;;
  reddish) echo "CRASH_AMPLIFICATION $base states_reused=3 red=2 expand_up_to=$expand exhaustive=true" ;;
  device) echo "  ✗ 点名的卡用不了（假驱动）" >&2; exit 78 ;;
  red-then-device) echo "CRASH_AMPLIFICATION $base states_reused=3 red=2 expand_up_to=$expand exhaustive=true"; echo "GPU_CARD_NOT_USABLE index=1（假驱动）"; exit 78 ;;
  zero) echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=0 states_reused=0 blocks_checked_here=0 unjudged_blocks=0 red=0 expand_up_to=$expand exhaustive=true" ;;
  no-red-field) echo "CRASH_AMPLIFICATION $base states_reused=3 red_states=3 violations=3 expand_up_to=$expand exhaustive=true" ;;
  red-not-a-number) echo "CRASH_AMPLIFICATION $base states_reused=3 red=? expand_up_to=$expand exhaustive=true" ;;
  ignored) echo "CRASH_AMPLIFICATION $base states_reused=3 red=0 expand_up_to=$expand exhaustive=true ignored_points=4 states_skipped_by_ignore=900000" ;;
esac
exit 0
STUB
printf '#!/usr/bin/env bash\nexit 0\n' > "$work/quiesce.sh"
run() { # <标签> <环境…> -- <入口参数…>
  local label="$1"; shift
  local -a environment=()
  while [[ "$1" != -- ]]; do environment+=("$1"); shift; done; shift
  local output exit_code
  output="$(env -u GATE_NOT_RUN_FILE CRASH_AMPLIFICATION_ITEM_DRIVER="$work/driver.sh" CRASH_AMPLIFICATION_QUIESCE_STUB="$work/quiesce.sh" \
    SINGLEFS_MULTI_HOST_CONFIG="$work/no-multi-host.env" SINGLEFS_GPU_CARDS_CONFIG="$work/no-gpu-cards.env" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home" \
    ${environment[@]+"${environment[@]}"} bash "$root/research/scripts/crash-amplification.sh" "$@" 2>&1)"; exit_code=$?
  echo "### $label：退 $exit_code"
  grep -E '^CRASH_AMPLIFICATION_(ITEM|TOTAL) |✓|✗ [0-9]+ 个分项判红|✗ 分项 |本次未跑|驱动退 78' <<<"$output" | sed 's/^/    /'
}
run E0-header-control SHAPE_crash_amplification_fixed_script_through_rollback=reddish -- --item fixed-script-through-rollback
run E1-red-then-78 SHAPE_crash_amplification_fixed_script_through_rollback=reddish SHAPE_crash_amplification_fixed_script_through_unmount=device -- --item fixed-script-through-rollback --item fixed-script-through-unmount
run E2-red-line-and-78 SHAPE=red-then-device -- --item fixed-script-through-rollback
run E3-zero-states SHAPE=zero --
run E4-no-red-field SHAPE=no-red-field --
run E5-red-not-a-number SHAPE=red-not-a-number -- --item tree-split-streams
run E6-ignored-not-reported SHAPE=ignored --
