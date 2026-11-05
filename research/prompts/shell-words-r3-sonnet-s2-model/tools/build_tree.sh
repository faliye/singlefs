#!/usr/bin/env bash
# 造喂钩子与起真 bash 共用的那棵树：仓里凡是"名字会被钩子认"的脚本，在这棵树里换成桩（只记录自己被起、不干活）。
# 用法：build_tree.sh <repo> <stublib.sh 的绝对路径>
set -euo pipefail
R="$1"; LIB="$2"
mk_stub() {   # mk_stub <相对路径> <模式：log|wrap|wrapmem|wrapbuild|detached|quiesce>
  local rel="$1" mode="$2" f="$R/$1"
  mkdir -p "$(dirname "$f")"
  {
    echo '#!/usr/bin/env bash'
    echo "# STUB($mode)"
    echo "source '$LIB'"
    echo "stub_log '$rel' \"\$@\""
    case "$mode" in
      log) echo 'exit 0' ;;
      wrap) echo 'shift; exec "$@"' ;;
      wrapmem) echo 'shift; "$@"; exit $?' ;;
      wrapbuild) echo '"$@"; exit $?' ;;
      quiesce) echo 'while [ "$#" -gt 0 ] && [ "$1" != -- ]; do shift; done; shift; "$@"; exit $?' ;;
      detached) cat <<'EOS'
if [ "${1-}" = start ]; then
  while [ "$#" -gt 0 ] && [ "$1" != -- ]; do shift; done
  shift
  "$@"; exit $?
fi
exit 0
EOS
      ;;
    esac
  } > "$f"
  chmod +x "$f"
}
cd "$R"
# 先把仓里所有 .sh 换成"只记录"的桩（读它们正文的钩子读到的就是桩；钩子按名字判的脚本不读正文）
while IFS= read -r -d '' f; do
  case "$f" in .claude/hooks/*|.claude/scripts/preflight.sh|.claude/scripts/env.sh|.claude/singlefs-ai-sop/*) continue ;; esac
  mk_stub "$f" log
done < <(find scripts .claude/gate.d .claude/scripts -name '*.sh' -print0)
mk_stub scripts/hosts/capped.sh wrap
mk_stub scripts/hosts/run-with-memory-cap.sh wrapmem
mk_stub scripts/hosts/run-build-directory.sh wrapbuild
mk_stub scripts/hosts/detached-run.sh detached
mk_stub scripts/crash/gpu-quiesce-run.sh quiesce
for py in scripts/mutation/broad-mutation.py scripts/gate/verus-proof-check.py scripts/experiment-apparatus/e165-pin-red-examples.py scripts/agents/agent-watch.py; do
  mkdir -p "$(dirname "$py")"
  printf '#!/usr/bin/env python3\n# STUB\nimport os,sys\nopen(os.environ.get("ORACLE_LOG","/dev/null"),"a").write("\\t".join([%q,os.getcwd(),str(os.getpid()),"HEAVY="+os.environ.get("SINGLEFS_HEAVY_TESTS",""),"MEMCAP="+os.environ.get("FAKE_MEMCAP",""),"BUILDDIR=","TARGET=","PPID="+str(os.getppid()),"\\x1f".join(sys.argv[1:])+"\\x1f"])+"\\n")\n' "$py" > "$py"
  chmod +x "$py"
done
git init -q . 2>/dev/null || true
git add -A >/dev/null 2>&1; git -c user.email=a@b -c user.name=r3 commit -qm base >/dev/null 2>&1 || true
