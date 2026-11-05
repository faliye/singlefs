# 桩的公共部分（纯 bash，不起子进程）：把"被真 bash 起来了"连同参数、当前目录、环境里的前缀、祖先里有没有内存包装 / 编译目录包装，记进日志。
# 日志路径与前缀不靠环境变量一路传（env -i 会清掉它们），沿进程祖先去找：
#   - 日志路径：最近一个祖先的环境里有 ORACLE_LOG 的
#   - 内存包装 / 编译目录包装：祖先（不含自己）的 argv[1] 是 run-with-memory-cap.sh / run-build-directory.sh（桩不 exec，留在祖先链上）
# 记录格式：一行一条，字段用制表符分，参数之间用 \x1f 连。
stub_log() {   # stub_log <名字> <参数…>
  local name="$1"; shift
  local args="" a p="$$" log="${ORACLE_LOG-}" memcap="" builddir="" w0 w1 kv guard=0 rest
  for a in "$@"; do args+="${a}"$'\x1f'; done
  while [ -n "$p" ] && [ "$p" -gt 1 ] && [ "$guard" -lt 40 ]; do
    guard=$((guard + 1))
    if [ "$p" != "$$" ] && [ -r "/proc/$p/cmdline" ]; then
      w0="" w1=""
      { IFS= read -r -d '' w0; IFS= read -r -d '' w1; } < "/proc/$p/cmdline" 2>/dev/null
      case "$w0 $w1" in
        *scripts/hosts/run-with-memory-cap.sh*) memcap=1 ;;
        *scripts/hosts/run-build-directory.sh*) builddir=1 ;;
      esac
    fi
    if [ -z "$log" ] && [ -r "/proc/$p/environ" ]; then
      while IFS= read -r -d '' kv; do
        case "$kv" in ORACLE_LOG=*) log="${kv#ORACLE_LOG=}"; break ;; esac
      done < "/proc/$p/environ" 2>/dev/null
    fi
    rest=""
    { read -r _ _ _ rest _; } < "/proc/$p/stat" 2>/dev/null
    p="$rest"
  done
  printf '%s\t%s\t%s\tHEAVY=%s\tMEMCAP=%s\tBUILDDIR=%s\tTARGET=%s\tPPID=%s\t%s\n' \
    "$name" "$PWD" "$$" "${SINGLEFS_HEAVY_TESTS-}" "$memcap" "$builddir" "${CARGO_TARGET_DIR-}" "$PPID" "$args" \
    >> "${log:-/dev/null}"
}
