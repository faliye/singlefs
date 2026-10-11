#!/usr/bin/env bash
# 复跑：SINGLEFS_TOOLS=<仓根> bash rerun.sh <新工作目录>（冻结副本缺省 ~/.local/share/singlefs/work/m3-batch10z-ghi-code-r1/tree，SINGLEFS_FROZEN 可改）。
# 建两份冻结副本拷贝：repo（打 src/opus10zghi-arms.patch、放原型包与 K1-1 模型）、repo-m（原样，逐条打 scripts/mut.py 的改坏跑 harness 档 lib 单测）。
# 每件经 run-with-memory-cap.sh 与 capped.sh；末尾每份汇总一行「<名> 全同 / 不同」。挂钟本机约 12 分钟（b1、b2、b3 各约 3 分钟，编译另算）。
set -u
work=${1:?新工作目录}
here=$(cd "$(dirname "$0")" && pwd)
frozen=${SINGLEFS_FROZEN:-$HOME/.local/share/singlefs/work/m3-batch10z-ghi-code-r1/tree}
tools=${SINGLEFS_TOOLS:?要仓根}
cap() { nice -n 19 bash "$tools/scripts/hosts/run-with-memory-cap.sh" "$1" bash "$tools/scripts/hosts/capped.sh" "$2" "${@:3}"; }
mkdir -p "$work/runs" "$work/out"
rsync -a --exclude target --exclude .git "$frozen/" "$work/repo/"
rsync -a --exclude target --exclude .git "$frozen/" "$work/repo-m/"
(cd "$work/repo" && patch -p0 < "$here/src/opus10zghi-arms.patch") > "$work/patch.log" 2>&1 || { echo "打补丁失败"; exit 1; }
mkdir -p "$work/repo/crates/opus10zghi-probe"
cp -r "$here/src/opus10zghi-probe/." "$work/repo/crates/opus10zghi-probe/"
cp "$here/src/opus_k11_poll_race.rs" "$work/repo/crates/singlefs-harness/tests/"
(cd "$work/repo" && CARGO_TARGET_DIR="$work/target" cap 12G 15 cargo test --release --offline -p opus10zghi-probe --test opus_k21_chain --no-run) > "$work/build.log" 2>&1 || { echo "编译失败"; exit 1; }
bin=$(ls -t "$work/target/release/deps/" | grep '^opus_k21_chain-[0-9a-f]*$' | head -1)
B="$work/target/release/deps/$bin"
piece() { # 批号 件号 名 环境变量串
  local b=$1 i=$2 name=$3; shift 3
  { env "$@" cap 4G 1 "$B" --nocapture opus_sweep > "$work/runs/$b-$i-$name.log" 2>&1; echo "$?" > "$work/runs/$b-$i.rc"; } &
}
F='OPUS_FAULTS=cfgq,qroot,qlast,racct'
rm -f "$work"/runs/b1-*.rc; n=0
for arm in bing chain h2; do for seed in 11 12 13; do n=$((n+1)); piece b1 $n $arm-$seed OPUS10ZGHI_ARM=$arm OPUS_PREROOT=1 OPUS_STEPCHECK=1 OPUS_KS=3 OPUS_Q=s OPUS_R=n "$F" OPUS_ALPHABET=onsFb OPUS_ACTIONS=RAND20:$seed::20:34; done; done
wait
rm -f "$work"/runs/b2-*.rc; n=0
for arm in bing chain; do for cfg in k5 persist; do for seed in 21 22; do n=$((n+1))
  if [ $cfg = k5 ]; then K=5; PE=OPUS_DUMMY=1; else K=3; PE=OPUS_PERSIST=1; fi
  piece b2 $n $arm-$cfg-$seed $PE OPUS10ZGHI_ARM=$arm OPUS_PREROOT=1 OPUS_STEPCHECK=1 OPUS_KS=$K OPUS_Q=s OPUS_R=n "$F" OPUS_ALPHABET=onsFb OPUS_ACTIONS=RAND20:$seed::20:34
done; done; done
wait
rm -f "$work"/runs/b3-*.rc; n=0
for arm in bing exact h2; do for rk in o s n; do n=$((n+1)); piece b3 $n $arm-$rk OPUS10ZGHI_ARM=$arm OPUS_STEPCHECK=1 OPUS_KS=3 OPUS_Q=s OPUS_R=$rk OPUS_FAULTS=racct OPUS_ALPHABET=onsWFbu OPUS_ACTIONS=RAND30:31::3:12; done; done
wait
for b in b1:9 b2:8 b3:9; do got=$(ls "$work"/runs/${b%%:*}-*.rc 2>/dev/null | wc -l); [ "$got" = "${b##*:}" ] || echo "${b%%:*} 件数不对：$got"; done
echo "退出码不为 0 的件：$(grep -Lx 0 "$work"/runs/*.rc | wc -l)"
for b in b1 b2 b3; do python3 "$here/scripts/agg.py" "$work/runs/$b-*.log" > "$work/out/summary-$b.txt"; done
python3 "$here/scripts/iso.py" "$work/runs/b3-*.log" > "$work/out/iso-b3.txt"
(cd "$work/repo" && CARGO_TARGET_DIR="$work/target-h" cap 8G 15 cargo test --offline -p singlefs-harness --test opus_k11_poll_race -- --nocapture) > "$work/k11.log" 2>&1
grep ATTACK "$work/k11.log" > "$work/out/k11.txt"
Fl=crates/singlefs-harness/src/gpu_memory_ledger.rs; cp "$work/repo-m/$Fl" "$work/ledger.orig"
for m in M1-release-keeps-kept-leases M2-release-keeps-device-in-use M3-device-never-released M5-never-resume-unwind M6-poll-exit-returns-every-card; do
  cp "$work/ledger.orig" "$work/repo-m/$Fl"; python3 "$here/scripts/mut.py" "$work/repo-m/$Fl" $m
  (cd "$work/repo-m" && CARGO_TARGET_DIR="$work/target-m" cap 8G 8 cargo test --offline -p singlefs-harness --lib -- gpu_memory_ledger) > "$work/mut-$m.log" 2>&1
  echo "MUTANT $m $(grep -E '^test result' "$work/mut-$m.log" | head -1)"
done | sed 's/finished in [0-9.]*s//' > "$work/out/mutants.txt"
cp "$work/ledger.orig" "$work/repo-m/$Fl"
for f in "$work"/out/*.txt; do b=$(basename "$f")
  if [ "$b" = mutants.txt ]; then sed 's/finished in [0-9.]*s//' "$here/out/$b" | cmp -s - "$f" && echo "$b 全同" || echo "$b 不同"; else cmp -s "$f" "$here/out/$b" && echo "$b 全同" || echo "$b 不同"; fi
done
