# 样本：替身 preflight（gate-structure 格的 preflight-guarded 两份样本用）：摘掉 --force、其余参数原样放回，不判准入
preflight() {
  shift
  PREFLIGHT_ARGUMENTS=()
  local argument
  for argument in "$@"; do
    [[ "$argument" == --force ]] || PREFLIGHT_ARGUMENTS+=("$argument")
  done
}
