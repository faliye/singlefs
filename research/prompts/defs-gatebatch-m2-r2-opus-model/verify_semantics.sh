#!/usr/bin/env bash
# 核攻击格的前提在真工具上成立（不碰被判的重型用例）：libtest 的 --skip / --logfile 吃掉 --list、cfg 二选一与子模块同名、# [ignore]；
# strace -fo、flock -xw、/usr/bin/time -ao、systemd-run -qu / -E、env -S 真能那样用；cargo 认不认 --config 里的别名与带引号的 runner 键、
# 带解释器的 runner；include_str!(env!(…)) 走 [env]、::core::include_str!(::core::concat!(…)) 编得过、读的是别的测试目标的文件。
# 用法：bash verify_semantics.sh <草稿目录>；编译经 capped.sh 6，跑编出来的东西经 run-with-memory-cap.sh。
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../../.." && pwd)"
D="${1:?草稿目录}/semantics"
rm -rf -- "${D:?}"; mkdir -p "$D"; cd "$D" || exit 1
CAP="bash $REPO/research/scripts/capped.sh 6"
MC="bash $REPO/research/scripts/run-with-memory-cap.sh 2G"
cat > t.rs <<'RS'
#[test]
fn quick() {}
#[test]
#[ignore]
fn full_ignored() { println!("FULL_IGNORED_RAN"); }
#[cfg(debug_assertions)]
#[test]
#[ignore]
fn the_case() { println!("THE_CASE_DEBUG_IGNORED_RAN"); }
#[cfg(not(debug_assertions))]
#[test]
fn the_case() { println!("THE_CASE_RELEASE_UNIGNORED_RAN"); }
mod slow {
    #[test]
    #[ignore]
    fn other_case() {}
}
#[test]
# [ignore]
fn spaced_ignore() { println!("SPACED_RAN"); }
RS
cat > m.rs <<'RS'
mod slow {
    #[test]
    #[ignore]
    fn the_case() {}
}
#[test]
fn the_case() { println!("TOP_LEVEL_UNIGNORED_RAN"); }
RS
nice -n 19 $CAP rustc --edition 2021 --test -C debug-assertions=off -O t.rs -o t_release
nice -n 19 $CAP rustc --edition 2021 --test m.rs -o m_bin
for args in "--list" "--nocapture" "--include-ignored --skip --list --nocapture" "--include-ignored --exact full_ignored --logfile --list --nocapture"; do
  echo "== t_release（release、debug-assertions 关）$args"; $MC ./t_release $args 2>&1 | grep -E 'RAN|test result|: test'
done
echo "== m_bin --exact the_case"; $MC ./m_bin --exact the_case --nocapture 2>&1 | grep -E 'RAN|test result'
for c in "strace -fo s.txt true" "flock -xw 1 lockf true" "/usr/bin/time -ao t.txt true" "env -S 'echo ENV_S_RAN'" "systemd-run --user --scope -qu k1semantics$$ true"; do
  echo "== $c"; eval "$c"; echo "rc=$?"
done
echo "== systemd-run --scope -E 把变量带给里面那条命令"; systemd-run --user --scope -q -E K1_PROBE_VAR=seen env | grep K1_PROBE_VAR
mkdir -p crate/src crate/tests crate/.cargo
printf '[package]\nname = "semanticsprobe"\nversion = "0.0.0"\nedition = "2021"\n[workspace]\n' > crate/Cargo.toml
printf 'fn main() { println!("BINARY_RAN_DIRECTLY"); }\n' > crate/src/main.rs
printf '#!/usr/bin/env bash\necho "RUNNER_USED $*"\n' > crate/r.sh; chmod +x crate/r.sh
printf '[env]\nK1_FIXTURE = { value = "tests/oth%s", relative = true }\n' 'er_target.rs' > crate/.cargo/config.toml
cat > crate/tests/own_case.rs <<'RS'
const VIA_ENV: &str = include_str!(env!("K1_FIXTURE"));
const VIA_QUALIFIED: &str = ::core::include_str!(::core::concat!(env!("CARGO_MANIFEST_DIR"), "/tests/oth", "er_target.rs"));
#[test]
#[ignore]
fn the_case() { println!("ENV_LEN={} QUALIFIED_LEN={} SAME={}", VIA_ENV.len(), VIA_QUALIFIED.len(), VIA_ENV == VIA_QUALIFIED); }
RS
printf '#[test]\nfn other() { let marker = 1; assert_eq!(marker, 1); }\n' > crate/tests/other_target.rs
cd crate || exit 1
for key in 'runner' '"runner"'; do
  echo "== cargo --config target.<三元组>.${key}=<脚本> run"; nice -n 19 $CAP $MC cargo --config "target.x86_64-unknown-linux-gnu.${key}=\"$PWD/r.sh\"" run -q 2>&1 | tail -1
done
echo "== runner 前面带解释器（环境变量）"; CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="bash $PWD/r.sh" nice -n 19 $CAP $MC cargo run -q 2>&1 | tail -1
echo "== cargo --config alias.k1zz=\"version\" k1zz"; $MC cargo --config 'alias.k1zz="version"' k1zz 2>&1 | head -1
echo "== CARGO_ALIAS_K1ZZ=version cargo k1zz"; CARGO_ALIAS_K1ZZ=version $MC cargo k1zz 2>&1 | head -1
echo "== include_str!(env!(…)) 与 ::core::include_str!(::core::concat!(…))"; nice -n 19 $CAP $MC cargo test -q --test own_case -- --ignored --nocapture 2>&1 | grep -E 'LEN|error|test result'
