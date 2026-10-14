#!/usr/bin/env bash
# admission: always 直接执行只打此刻的登记值或跑自证，判的是此刻的登记文件与环境，上一次的结论不替这一次作保
# run-condition: none 被 source 的函数库；直接执行只读登记文件、在临时目录里造样本，除了 bash 与 python3 没有环境要求
# 每台默认重活线程数的 shell 取法：被 source 的函数库，只定义函数，不改 shell 选项、不设全局变量。
# 登记位是仓根的 .claude/heavy-thread-default（写法在它的文件头）；python 那一份是 scripts/hosts/heavy_threads.py 的 default_heavy_threads，
# 两份同一个判法，两边的自证拿同几份登记互相对拍。
#
#   source scripts/hosts/heavy-threads.sh
#   if ! threads="$(heavy_thread_default)"; then exit 2; fi    stdout 一行正整数；拒的时候 stderr 写 ✗ 与 →、返回 2、stdout 什么都不打
#   bash scripts/hosts/heavy-threads.sh --print                 打此刻的值，拒了退 2
#   bash scripts/hosts/heavy-threads.sh --selftest              自证
#
# 判法：
#   SINGLEFS_THREAD_CAP 设了（非空）：是正整数就打它（经 bash scripts/hosts/capped.sh <N> 起的命令里它就是 N），不是就拒；
#   没设：读这份文件往上两级那棵树的 .claude/heavy-thread-default，第一条非注释行（空行与 # 开头的行不算）要是
#     「<正整数><空格或制表符>#<理由>」（理由不空），打那个数；文件不在、读不了、没有非注释行、那一行不是这个形状，都拒。
#   拒了不退回按核数算。
#
# 弄坏开关 HEAVY_THREADS_BREAK=<项>（逗号分隔，只给自证证红；scripts/hosts/heavy_threads.py 认同样的名字），
# 打开后自证哪几格红（失败行「✗ heavy-threads.sh 自证：<格名>：…」）：
#   cap-unchecked                         SINGLEFS_THREAD_CAP 原样打出、不判正整数：「SINGLEFS_THREAD_CAP=0 拒」「SINGLEFS_THREAD_CAP=abc 拒」
#   missing-registration-uses-core-count  登记文件不在时照改前取本机核数：「登记文件不在拒」

heavy_threads_break_is_set() {
  [[ ",${HEAVY_THREADS_BREAK:-}," == *",$1,"* ]]
}

# heavy_thread_default：打每台默认重活线程数（判法在文件头）。
heavy_thread_default() {
  local cap="${SINGLEFS_THREAD_CAP:-}" directory registration line number=0 found=0 value=""
  local comment_or_blank=$'^[ \t]*(#|$)' registration_line=$'^[ \t]*([1-9][0-9]*)[ \t]+#[ \t]*[^ \t]'
  if [[ -n "$cap" ]]; then
    if [[ "$cap" =~ ^[1-9][0-9]*$ ]] || heavy_threads_break_is_set cap-unchecked; then
      printf '%s\n' "$cap"
      return 0
    fi
    echo "  ✗ SINGLEFS_THREAD_CAP=$cap 不是正整数，取不了每台默认重活线程数" >&2
    echo "     → 怎么办：设成正整数（经 bash scripts/hosts/capped.sh <N> 起就是 N），或 unset 它取 .claude/heavy-thread-default 的登记值" >&2
    return 2
  fi
  directory="${BASH_SOURCE[0]%/*}"
  if [[ "$directory" == "${BASH_SOURCE[0]}" ]]; then directory=.; fi
  registration="$directory/../../.claude/heavy-thread-default"
  if [[ ! -f "$registration" ]]; then
    if heavy_threads_break_is_set missing-registration-uses-core-count; then nproc; return 0; fi
    echo "  ✗ 每台默认重活线程数的登记文件 $registration 不在" >&2
    echo "     → 怎么办：在仓根建 .claude/heavy-thread-default，第一条非注释行写「31  # <理由>」（写法在仓里那一份的文件头）；临时要别的数设 SINGLEFS_THREAD_CAP 或经 capped.sh <N> 起，不退回按核数算" >&2
    return 2
  fi
  if [[ ! -r "$registration" ]]; then
    echo "  ✗ 每台默认重活线程数的登记文件 $registration 读不了" >&2
    echo "     → 怎么办：看它的权限，改成本用户可读再跑；不退回按核数算" >&2
    return 2
  fi
  while IFS= read -r line || [[ -n "$line" ]]; do
    number=$((number + 1))
    if [[ "$line" =~ $comment_or_blank ]]; then continue; fi
    found=1
    if [[ "$line" =~ $registration_line ]]; then value="${BASH_REMATCH[1]}"; fi
    break
  done < "$registration"
  if [[ -n "$value" ]]; then
    printf '%s\n' "$value"
    return 0
  fi
  if ((found)); then
    echo "  ✗ $registration 第 $number 行「$line」不是「<正整数>  # <理由>」" >&2
    echo "     → 怎么办：第一条非注释行写成「31  # <理由>」：数是正整数，数与 # 之间至少一个空格，# 后面写为什么是这个数" >&2
  else
    echo "  ✗ $registration 里没有非注释行，读不出每台默认重活线程数" >&2
    echo "     → 怎么办：加一行「31  # <理由>」：数是正整数，数与 # 之间至少一个空格，# 后面写为什么是这个数" >&2
  fi
  return 2
}

# ── 直接执行：--print 与 --selftest ─────────────────────────────────────────────

# heavy_threads_selftest_case <样本树> <SINGLEFS_THREAD_CAP：值，或 -unset> <结果前缀>：在样本树里 source 那一份取值，
# 写 <前缀>.stdout / .stderr / .rc；同一棵树里那份 heavy_threads.py 的结果写 <前缀>.python（数或 refused）
heavy_threads_selftest_case() {
  local tree="$1" cap="$2" prefix="$3" environment=(env -u SINGLEFS_THREAD_CAP) status
  if [[ "$cap" != -unset ]]; then environment=(env "SINGLEFS_THREAD_CAP=$cap"); fi
  if "${environment[@]}" bash -c 'source "$1"; heavy_thread_default' heavy-threads-case "$tree/scripts/hosts/heavy-threads.sh" \
       >"$prefix.stdout" 2>"$prefix.stderr"; then status=0; else status=$?; fi
  printf '%s\n' "$status" > "$prefix.rc"
  "${environment[@]}" python3 -B -c '
import sys
sys.path.insert(0, sys.argv[1])
import heavy_threads
try:
    print(heavy_threads.default_heavy_threads())
except heavy_threads.HeavyThreadDefaultError:
    print("refused")
' "$tree/scripts/hosts" > "$prefix.python" 2>"$prefix.python-stderr"
}

heavy_threads_selftest() {
  local here scratch tree index=0 checked=0 failures=0 label registration cap want got rc python_got real_value printed
  here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
  scratch="$(mktemp -d)" || { echo "  ✗ heavy-threads.sh 自证：建不了临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限"; return 1; }
  trap 'rm -rf "${scratch:?}"' EXIT
  heavy_threads_selftest_fail() {
    failures=$((failures + 1))
    echo "  ✗ heavy-threads.sh 自证：$1：$2"
    echo "     → 怎么办：照文件头「判法」改 heavy_thread_default（与 scripts/hosts/heavy_threads.py 的 default_heavy_threads 同一个判法）；HEAVY_THREADS_BREAK 设着的话这里本来就该红"
  }
  # 每格：格名、登记文件的内容（MISSING 是不建这份文件；\n 换行）、SINGLEFS_THREAD_CAP（-unset 是不设）、期望（数，或 refused）
  local cells=(
    "登记 7、带理由|7  # 样本理由|-unset|7"
    "登记前有注释行与空行、只认第一条非注释行|# 注释\n\n7  # 样本理由\n8  # 第二条不认|-unset|7"
    "登记文件不在拒|MISSING|-unset|refused"
    "写坏：不是数拒|abc  # 样本理由|-unset|refused"
    "写坏：0 拒|0  # 样本理由|-unset|refused"
    "写坏：没写理由拒|7|-unset|refused"
    "写坏：数与 # 之间没有空白拒|7# 样本理由|-unset|refused"
    "写坏：只有注释拒|# 只有注释|-unset|refused"
    "SINGLEFS_THREAD_CAP=5 盖过登记的 7|7  # 样本理由|5|5"
    "SINGLEFS_THREAD_CAP=5、登记文件不在也照它|MISSING|5|5"
    "SINGLEFS_THREAD_CAP=0 拒|7  # 样本理由|0|refused"
    "SINGLEFS_THREAD_CAP=abc 拒|7  # 样本理由|abc|refused"
    "SINGLEFS_THREAD_CAP 设成空串当没设|7  # 样本理由||7"
  )
  local cell
  for cell in "${cells[@]}"; do
    IFS='|' read -r label registration cap want <<<"$cell"
    index=$((index + 1))
    tree="$scratch/tree-$index"
    mkdir -p "$tree/scripts/hosts" "$tree/.claude"
    cp "$here/heavy-threads.sh" "$here/heavy_threads.py" "$tree/scripts/hosts/"
    if [[ "$registration" != MISSING ]]; then printf '%b\n' "$registration" > "$tree/.claude/heavy-thread-default"; fi
    heavy_threads_selftest_case "$tree" "$cap" "$scratch/case-$index"
    checked=$((checked + 1))
    got="$(<"$scratch/case-$index.stdout")"
    rc="$(<"$scratch/case-$index.rc")"
    if [[ "$want" == refused ]]; then
      if [[ "$rc" != 2 || -n "$got" ]] || ! grep -q '✗' "$scratch/case-$index.stderr" || ! grep -q '→' "$scratch/case-$index.stderr"; then
        heavy_threads_selftest_fail "$label" "应当拒（返回 2、stdout 空、stderr 有 ✗ 与 →），实际返回 $rc、stdout「$got」"
      fi
    elif [[ "$rc" != 0 || "$got" != "$want" ]]; then
      heavy_threads_selftest_fail "$label" "应当打 $want，实际返回 $rc、stdout「$got」、stderr「$(head -n 1 "$scratch/case-$index.stderr")」"
    fi
    checked=$((checked + 1))
    python_got="$(<"$scratch/case-$index.python")"
    if [[ "$want" == refused ]]; then
      [[ "$python_got" == refused ]] || heavy_threads_selftest_fail "$label（与 heavy_threads.py 对拍）" "shell 拒了，python 交「$python_got」"
    elif [[ "$python_got" != "$got" ]]; then
      heavy_threads_selftest_fail "$label（与 heavy_threads.py 对拍）" "shell 打「$got」，python 交「$python_got」（$(head -n 1 "$scratch/case-$index.python-stderr")）"
    fi
  done
  # 本仓那一份：登记读得出，--print 与 source 之后调函数同值，与 heavy_threads.py 同值
  checked=$((checked + 1))
  if real_value="$(env -u SINGLEFS_THREAD_CAP bash -c 'source "$1"; heavy_thread_default' heavy-threads-real "$here/heavy-threads.sh" 2>"$scratch/real.stderr")" \
       && [[ "$real_value" =~ ^[1-9][0-9]*$ ]]; then
    printed="$(env -u SINGLEFS_THREAD_CAP bash "$here/heavy-threads.sh" --print 2>/dev/null)" || printed="（退了非 0）"
    [[ "$printed" == "$real_value" ]] || heavy_threads_selftest_fail "本仓的登记：--print 与函数同值" "函数打 $real_value，--print 打「$printed」"
    python_got="$(env -u SINGLEFS_THREAD_CAP python3 -B -c 'import sys; sys.path.insert(0, sys.argv[1]); import heavy_threads; print(heavy_threads.default_heavy_threads())' "$here" 2>&1)"
    [[ "$python_got" == "$real_value" ]] || heavy_threads_selftest_fail "本仓的登记：与 heavy_threads.py 同值" "shell 打 $real_value，python 交「$python_got」"
  else
    heavy_threads_selftest_fail "本仓的登记读得出" "$(head -n 2 "$scratch/real.stderr" | tr '\n' ' ')"
  fi
  if ((failures)); then
    echo "  ✗ heavy-threads.sh 自证 $failures 处不对（查了 $checked 项）"
    echo "     → 怎么办：照上面逐条改；HEAVY_THREADS_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ heavy-threads.sh 自证通过（查了 $checked 项：${#cells[@]} 份样本登记各在 shell 里取一次、与 heavy_threads.py 对拍一次，本仓那一份读得出、--print 与 python 同值）"
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  set -uo pipefail
  source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
  preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
  case "${1:-}" in
    --selftest) heavy_threads_selftest; exit $? ;;
    --print) heavy_thread_default; exit $? ;;
    *)
      echo "  ✗ 认不出的用法：$*"
      echo "     → 怎么办：source 它再调 heavy_thread_default；直接执行只认 --print（打此刻的值）与 --selftest"
      exit 2 ;;
  esac
fi
