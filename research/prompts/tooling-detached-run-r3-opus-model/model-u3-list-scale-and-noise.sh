#!/usr/bin/env bash
# 模型 U3：只跑 list（不起单元）。替身 systemctl：show 全答 inactive，list-units 列出 $NOISE 个别的会话的自证单元（selftest-<pid>-<格>）。
#   甲：登记 N 条（都已结束，各带 exit）时 list 的用时、输出行数与字节；乙：登记 0 条、别的会话自证在跑 25 个单元时 list 里「登记里没有」几行。
# 用法：bash model-u3-list-scale-and-noise.sh <detached-run.sh 的路径> <N…>
set -u
script="$1"; shift
work="$(mktemp -d "${TMPDIR:-/tmp}/opusr3-u3.XXXXXX")"; trap 'rm -rf -- "$work"' EXIT
mkdir -p "$work/bin"; real="$(command -v systemctl)"
cat > "$work/bin/systemctl" <<SHIM
#!/usr/bin/env bash
for a in "\$@"; do [[ "\$a" == show-environment ]] && exec $(printf '%q' "$real") "\$@"; done
case " \$* " in
  *" show "*) echo inactive ;;
  *" list-units "*) for i in \$(seq 1 \${NOISE:-0}); do echo "singlefs-detached-selftest-99999-c\$i.service loaded active running 替身"; done ;;
esac
exit 0
SHIM
chmod +x "$work/bin/systemctl"
for n in "$@"; do
  state="$work/state-$n"; mkdir -p "$state"
  for i in $(seq 1 "$n"); do
    r="$state/l$i"; mkdir -p "$r"
    echo "singlefs-detached-l$i.service" > "$r/unit"; echo 1790000000 > "$r/started"; echo 2026-10-01 > "$r/started-date"
    echo 1790000100 > "$r/ended"; echo 2026-10-01 > "$r/ended-date"; echo 0 > "$r/exit"; echo 1 > "$r/pid"; echo "sleep 1" > "$r/command"; echo "$r/log" > "$r/log"
  done
  t0=$(date +%s.%N)
  PATH="$work/bin:$PATH" NOISE=0 DETACHED_RUN_STATE_DIRECTORY="$state" bash "$script" list > "$work/out-$n" 2>&1
  t1=$(date +%s.%N)
  echo "甲 登记 $n 条：list 退 $?（最后一条命令），用时 $(echo "$t1 - $t0" | bc) 秒，输出 $(wc -l < "$work/out-$n") 行 $(wc -c < "$work/out-$n") 字节"
done
mkdir -p "$work/state-empty"
PATH="$work/bin:$PATH" NOISE=25 DETACHED_RUN_STATE_DIRECTORY="$work/state-empty" bash "$script" list > "$work/out-noise" 2>&1
echo "乙 登记 0 条、别的会话自证 25 个单元：「登记里没有」$(grep -c '登记里没有  单元' "$work/out-noise") 行；末行：$(tail -n 1 "$work/out-noise" | cut -c1-120)"
