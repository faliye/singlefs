#!/usr/bin/env bash
# G5 模型：照改后 mutation-triage.md 第 5 步「照每条的 `✅ [抓到]`、`⏭  [无效]`、`❌ [没红]` 标记数」数 mutate.sh 的真实输出。
# 假 cargo 仿 mutate.sh --selftest 的写法（按被测源码里的 BEHAVIOR 演结局，不编译任何东西）；变异名取实验变异表常见的英文名，不与类别同名。
# 这个容器没有用户级 systemd（run-with-memory-cap.sh --check 退 251），所以设 RUN_WITH_MEMORY_CAP_BREAK=nocap 让包装直接跑假 cargo；只影响内存上限，不影响判结局。
# 用法：bash g5-mutate-markers.sh <草稿目录>
set -uo pipefail
REPO=/home/user/singlefs
SCRATCH=${1:?草稿目录}
work="$(mktemp -d -p "$SCRATCH" g5.XXXX)"
mkdir -p "$work/bin" "$work/research/bench/src/bin" "$work/research/mutations"
cat > "$work/bin/cargo" <<'FAKE'
#!/usr/bin/env bash
behavior="$(sed -n 's/.*BEHAVIOR: \([A-Z]*\).*/\1/p' bench/src/bin/fake_bench.rs | head -1)"
passing() { printf 'running 2 tests\ntest tests::adds ... ok\ntest tests::subtracts ... ok\n\ntest result: ok. 2 passed; 0 failed\n'; }
case "$behavior" in
  PASS) passing; exit 0 ;;
  RED) printf 'running 2 tests\ntest tests::adds ... FAILED\ntest tests::subtracts ... ok\n\ntest result: FAILED. 1 passed; 1 failed\n'; exit 101 ;;
  BROKEN) printf 'error[E0425]: cannot find value `missing` in this scope\nerror: could not compile `bench`\n'; exit 101 ;;
  CRASH) printf 'running 2 tests\nerror: test failed, to rerun pass `--bin fake-bench`\n\nCaused by:\n  process didn'"'"'t exit successfully: `target/release/deps/fake_bench-0123456789abcdef` (signal: 9, SIGKILL: kill)\n'; exit 101 ;;
  *) exit 2 ;;
esac
FAKE
chmod +x "$work/bin/cargo"
# 这个容器没装 rsync：只认 mutate.sh 那一种调用（rsync -a --exclude … 源/ 目的/），用 tar 按同样的排除拷
cat > "$work/bin/rsync" <<'SHIM'
#!/usr/bin/env bash
excludes=(); while [[ "$1" == -* ]]; do case "$1" in --exclude) excludes+=("--exclude=./$2"); shift 2 ;; *) shift ;; esac; done
mkdir -p "$2" && tar -C "$1" "${excludes[@]}" -cf - . | tar -C "$2" -xf -
SHIM
chmod +x "$work/bin/rsync"
printf '[workspace]\nmembers = ["bench"]\n' > "$work/research/Cargo.toml"
printf '[package]\nname = "bench"\n\n[[bin]]\nname = "fake-bench"\npath = "src/bin/fake_bench.rs"\n' > "$work/research/bench/Cargo.toml"
printf '// comment line\nfn main() {} // BEHAVIOR: PASS\n' > "$work/research/bench/src/bin/fake_bench.rs"
printf 'e999_drop_bound_check\tBEHAVIOR: PASS\tBEHAVIOR: RED\ne999_flip_comparison\tBEHAVIOR: PASS\tBEHAVIOR: RED\ne999_touch_comment\t// comment line\t// edited comment line\ne999_break_syntax\tBEHAVIOR: PASS\tBEHAVIOR: BROKEN\ne999_endless_loop\tBEHAVIOR: PASS\tBEHAVIOR: CRASH\n' \
  > "$work/research/mutations/e999_model.tsv"
( cd "$work/research" && env PATH="$work/bin:$PATH" RUN_WITH_MEMORY_CAP_BREAK=nocap MUTATE_TARGET_DIR="$work/target" MUTATE_TIMEOUT=20 \
    bash "$REPO/research/scripts/mutate.sh" fake-bench bench/src/bin/fake_bench.rs mutations/e999_model.tsv ) > "$work/out.txt" 2>&1
echo "mutate.sh 退出码=$?；输出 $(wc -l < "$work/out.txt") 行，其中标记行："
grep -E '^(✅|❌|⏭|💥|⚠️|⏱|🧱)' "$work/out.txt"
echo "收尾两行："; grep -E '^计数：|已还原' "$work/out.txt"
echo "── 照定义字面数（grep -cF）──"
for marker in '✅ [抓到]' '⏭  [无效]' '❌ [没红]'; do printf '%s → %s\n' "$marker" "$(grep -cF "$marker" "$work/out.txt")"; done
echo "── 按行首符号数（grep -c '^符号'）──"
for sign in '✅' '⏭' '❌' '💥' '⚠️' '⏱' '🧱'; do printf '%s → %s\n' "$sign" "$(grep -c "^$sign" "$work/out.txt")"; done
echo "变异表条数：$(grep -c . "$work/research/mutations/e999_model.tsv")"
