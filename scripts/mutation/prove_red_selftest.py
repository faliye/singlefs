"""prove-red.sh 拆出的一份。
自证：样本表与样本结局、假 cargo、各组格函数（并行、判定库、选路、卡租约、编译目录）与 selftest 本体。只被入口的 --selftest 起，import core。
入口 prove-red.sh 的内联 python 按同目录 import 这一份；拆出的两份只向前 import（core → selftest）。
"""
import os, re, shlex, shutil, signal, subprocess, tempfile, time
from prove_red_core import (BROKEN, CLOSURE_ROOT_FOR_SELFTEST, COPY_LOCK, DROP_LAST_LANE_FOR_SELFTEST, LANES_VARIABLE, LIBRARY_ROOT_FOR_SELFTEST,
    MAIN_ROOT, RESTORE_JOURNAL_NAME, ROWS, ROW_STATE, SCRIPT_DIRECTORY, SCRIPT_KEY, SENTINEL_LABEL, TRIP_BUILD_DIRECTORIES,
    TRIP_BUILD_DIRECTORY_PREFIX, new_trip_build_directory, planned_lanes, prove, prove_from_arguments, rebuild_problem,
    remove_trip_build_directories_when_stopped, table_rows, verdict_of, write_text)

# 自证的样本表：checker 那一行是 验证包的（--lib），verification-test 那一行是验证包只点名集成测试目标的，noexpected 那一行必须红的测试名空着（不成形）
SELFTEST_TABLE = ("# 样本\n"
                  "caught\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "untargeted\tcrates/demo/src/lib.rs\t{ 42 }\t{ 43 }\t-p demo -- answer\tanswer_is_42\n"
                  "layer\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --test demo_crash_point_replay -- answer\tanswer_is_42\n"
                  "panics\tcrates/demo/src/lib.rs\t{ 42 }\t{ 45 }\t-p demo --lib -- answer\tanswer_panics\n"
                  "broken\tcrates/demo/src/lib.rs\t{ 42 }\t{ 46 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "elsewhere\tcrates/demo/src/lib.rs\t{ 42 }\t{ 47 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "linkfail\tcrates/demo/src/lib.rs\t{ 42 }\t{ 52 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "gpu\tcrates/demo/src/lib.rs\t{ 42 }\t{ 48 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "checker\tcrates/singlefs-verification/src/lib.rs\t{ 42 }\t{ 41 }\t-p singlefs-verification --lib -- answer\tanswer_is_42\n"
                  "verification-test\tcrates/singlefs-verification/src/lib.rs\t{ 42 }\t{ 40 }\t-p singlefs-verification --test verification_target -- answer\tanswer_is_42\n"
                  "redbase\tcrates/redbase/src/lib.rs\t{ 50 }\t{ 51 }\t-p redbase --lib -- answer\tanswer_is_42\n"
                  "release-gpu\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --features gpu --lib -- answer\tanswer_is_42\n"
                  "release-crash\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --test demo_crash_stream -- answer\tanswer_is_42\n"
                  "noexpected\tcrates/demo/src/lib.rs\t{ 42 }\t{ 49 }\t-p demo --lib -- answer\t\n"
                  "five-fields\tcrates/demo/src/lib.rs\t{ 42 }\t{ 49 }\t-p demo --lib -- answer\n")

# 判法逐行相同那一格的样本：(说明, 退出码, 输出, 点名)；prove-red 自己的判法与 verdict_of_run 判得不一样的几种都在里面
SELFTEST_VERDICT_SAMPLES = (
    ("点名的红了、退 101", 101, "test tests::answer_is_42 ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed\n", "answer_is_42"),
    ("should panic 的红了", 101, "test tests::answer_panics - should panic ... FAILED\ntest result: FAILED. 0 passed; 1 failed\n", "answer_panics"),
    ("点名省掉模块路径、两个模块各红一个", 101, "test a::answer_is_42 ... FAILED\ntest b::answer_is_42 ... FAILED\ntest result: FAILED. 0 passed; 2 failed\n", "answer_is_42"),
    ("退出码 0 而输出里有 FAILED", 0, "test tests::answer_is_42 ... FAILED\n", "answer_is_42"),
    ("编不过", 101, "error[E0308]: mismatched types\nerror: could not compile `demo`\n", "answer_is_42"),
    ("编过了、失败在链接", 101, "error: linking with `cc` failed: exit status: 1\n  = note: rust-lld: error: undefined hidden symbol: anon.0.llvm.1\n"
                               "error: could not compile `demo` (lib test) due to 1 previous error\n", "answer_is_42"),
    ("点名的绿、别的红", 101, "test tests::answer_is_42 ... ok\ntest tests::other ... FAILED\ntest result: FAILED. 1 passed; 1 failed\nerror: test failed\n", "answer_is_42"),
    ("点名的绿、打了 NOT_RUN", 0, "test tests::answer_is_42 ... ok\n---- tests::answer_is_42 stdout ----\nNOT_RUN_NO_GPU_ADAPTER test=tests::answer_is_42 reason=没卡\n"
                                   "test result: ok. 1 passed; 0 failed\n", "answer_is_42"),
    ("点名的红了、它那一段里有 GPU_DEVICE_LOST 行（test=main）", 101, "test tests::answer_is_42 ... FAILED\n\nfailures:\n\n---- tests::answer_is_42 stdout ----\n"
                                                                  "GPU_DEVICE_LOST test=main card=NVIDIA GeForce RTX 5090 reason=Xid 31\n\nfailures:\n    tests::answer_is_42\n\n"
                                                                  "test result: FAILED. 0 passed; 1 failed\n", "answer_is_42"),
    ("点名的红了、它那一段里有 GPU_DEVICE_LOST 行（test=unnamed-thread）", 101, "test tests::answer_is_42 ... FAILED\n\nfailures:\n\n---- tests::answer_is_42 stdout ----\n"
                                                                            "GPU_DEVICE_LOST test=unnamed-thread card=NVIDIA GeForce RTX 5090 reason=Xid 31\n\nfailures:\n    tests::answer_is_42\n\n"
                                                                            "test result: FAILED. 0 passed; 1 failed\n", "answer_is_42"),
    ("点名的红了、它那一段里有 GPU_DEVICE_LOST 行（test=点名的全名）", 101, "test tests::answer_is_42 ... FAILED\n\nfailures:\n\n---- tests::answer_is_42 stdout ----\n"
                                                                       "GPU_DEVICE_LOST test=tests::answer_is_42 card=NVIDIA GeForce RTX 5090 reason=Xid 31\n\nfailures:\n    tests::answer_is_42\n\n"
                                                                       "test result: FAILED. 0 passed; 1 failed\n", "answer_is_42"),
    ("点名的红了、GPU_DEVICE_LOST 行在别的用例的那一段里", 101, "test tests::answer_is_42 ... FAILED\ntest tests::other ... FAILED\n\nfailures:\n\n---- tests::answer_is_42 stdout ----\n"
                                                           "assertion failed\n\n---- tests::other stdout ----\n"
                                                           "GPU_DEVICE_LOST test=tests::answer_is_42 card=NVIDIA GeForce RTX 5090 reason=Xid 31\n\nfailures:\n    tests::answer_is_42\n    tests::other\n\n"
                                                           "test result: FAILED. 0 passed; 2 failed\n", "answer_is_42"),
    ("点名的红了、GPU_DEVICE_LOST 行在所有段之外（实时打印的部分）", 101, "GPU_DEVICE_LOST test=wgpu-callback card=NVIDIA GeForce RTX 5090 reason=Xid 31\n"
                                                                 "test tests::answer_is_42 ... FAILED\n\nfailures:\n\n---- tests::answer_is_42 stdout ----\n"
                                                                 "assertion failed\n\nfailures:\n    tests::answer_is_42\n\ntest result: FAILED. 0 passed; 1 failed\n", "answer_is_42"),
    ("点名的红了、GPU_DEVICE_LOST 行在所有段之外，别的用例的那一段里另有一行", 101, "GPU_DEVICE_LOST test=wgpu-callback card=NVIDIA GeForce RTX 5090 reason=Xid 31\n"
                                                                        "test tests::answer_is_42 ... FAILED\ntest tests::other ... FAILED\n\nfailures:\n\n---- tests::answer_is_42 stdout ----\n"
                                                                        "assertion failed\n\n---- tests::other stdout ----\n"
                                                                        "GPU_DEVICE_LOST test=tests::other card=NVIDIA GeForce RTX 5090 reason=Xid 31\n\nfailures:\n    tests::answer_is_42\n    tests::other\n\n"
                                                                        "test result: FAILED. 0 passed; 2 failed\n", "answer_is_42"),
)


def path_included_package_failure(work):
    """自证一格（文件头「判法与次序」⑤，弄坏开关 path-included-package-ignored）：被改文件在 crates/owner/build.rs，经 #[path] 编进 crates/includer
    （includer 的测试目标 dep-info 列着它，相对工作区根、带 ..，真 cargo 的写法）；点名的是 -p includer，这一趟输出里只有「Compiling includer」、
    没有「Compiling owner」：判重编过；includer 也没有 Compiling 行的（只有别的包）仍判没重编。→ 失败说明，过了交 None。"""
    path_copy = os.path.join(work, "path-copy")
    write_text(os.path.join(path_copy, "Cargo.toml"), '[workspace]\nmembers = ["crates/*"]\n')
    for package in ("owner", "includer"):
        write_text(os.path.join(path_copy, "crates", package, "Cargo.toml"), f'[package]\nname = "{package}"\n')
        write_text(os.path.join(path_copy, "crates", package, "src", "lib.rs"), "fn answer() -> u32 { 42 }\n")
    write_text(os.path.join(path_copy, "crates", "owner", "build.rs"), "fn main() {}\n")
    path_target = os.path.join(work, "path-target")
    write_text(os.path.join(path_target, "debug", "deps", "includer-0123.d"),
               "target/debug/deps/includer-0123: crates/includer/src/lib.rs crates/includer/src/../../owner/build.rs\n\n"
               "crates/includer/src/lib.rs:\ncrates/includer/src/../../owner/build.rs:\n")
    write_text(os.path.join(path_target, "debug", "deps", "build_script_build-4567.d"),
               "target/debug/build/owner-4567/build_script_build-4567: crates/owner/build.rs\n\ncrates/owner/build.rs:\n")
    path_row = ROWS.Row(1, "included", "crates/owner/build.rs", "main", "mine", "-p includer --lib -- answer", "answer_is_42", ())
    included_green = rebuild_problem(path_copy, path_row, 101, "   Compiling includer v0.1.0 (/x/crates/includer)\ntest tests::answer_is_42 ... FAILED\n", "log", path_target)
    included_red = rebuild_problem(path_copy, path_row, 101, "   Compiling unrelated v0.1.0 (/x/crates/unrelated)\ntest tests::answer_is_42 ... FAILED\n", "log", path_target)
    if included_green is not None or included_red is None or "没重编" not in included_red:
        return (f"被改文件经 #[path] 编进点名的包（dep-info 列着它）：有点名的那个包的 Compiling 行就判重编过、没有就仍判没重编，"
                f"实际有 Compiling 行的判 {included_green!r}、没有的判 {included_red!r}")
    return None


def process_gone(pid, seconds=5):
    """自证自己起的进程号 pid 在 seconds 秒内没了（或只剩僵尸）就交 True。"""
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        try:
            with open(f"/proc/{pid}/stat", encoding="utf-8") as handle:
                if handle.read().rsplit(")", 1)[1].split()[0] == "Z":
                    return True
        except (FileNotFoundError, ProcessLookupError):
            return True
        time.sleep(0.05)
    return False


def selftest_copy(work, name="copy"):
    copy = os.path.join(work, name)
    write_text(os.path.join(copy, "Cargo.toml"), '[workspace]\nmembers = ["crates/*"]\n')
    for package, answer in (("demo", 42), ("singlefs-verification", 42), ("redbase", 50)):
        write_text(os.path.join(copy, "crates", package, "Cargo.toml"), f'[package]\nname = "{package}"\n')
        write_text(os.path.join(copy, "crates", package, "src", "lib.rs"), f"fn answer() -> u32 {{ {answer} }}\n")
    write_text(os.path.join(copy, "crates", "mutations.tsv"), SELFTEST_TABLE)
    return copy


def reuse_states(lines):
    """prove 交的输出行 → {变异名: 现判 / 复用 / 强制现判 / 拒算}：认每条判定那一行末尾的标（文件头「判定库与复用」）。"""
    states = {}
    for line in lines:
        name, tab, _rest = (line[2:] if line.startswith("✗ ") else line).partition("\t")
        found = ROW_STATE.search(line)
        if tab and found:
            states[name] = found.group(1)
    return states


# 判定库那几格的样本（文件头「判定库与复用」）：demo 一个包两条测试、两条变异；假 cargo 照源码里那两段原文在不在判两条测试的红绿
REUSE_SOURCE = ("pub fn answer() -> u32 { 42 }\n"
                "pub fn other() -> u32 {\n"
                "    let base = 7;\n"
                "    base\n"
                "}\n"
                "\n"
                "#[cfg(test)]\n"
                "mod tests {\n"
                "    #[test]\n"
                "    fn answer_is_42() { assert_eq!(super::answer(), 42); }\n"
                "    #[test]\n"
                "    fn other_is_7() { assert_eq!(super::other(), 7); }\n"
                "}\n")
REUSE_TABLE = ("# 判定库那几格的样本\n"
               "answer-41\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --lib -- answer_is_42\tanswer_is_42\n"
               "other-8\tcrates/demo/src/lib.rs\tlet base = 7;\tlet base = 8;\t-p demo --lib -- other_is_7\tother_is_7\n")
# 判定库那几格的假 cargo：每起一次往 @LOG@ 记一行参数；`--` 之后头一个不以 - 起头的词是过滤词（照 libtest 按全名子串选）；
# answer_is_42 在源码里有「{ 42 }」时 ok（PROVE_RED_SELFTEST_TEST_WEAKENED 设了一律 ok：演点名的测试被改弱、同一把键判出没红），other_is_7 在有「let base = 7;」时 ok
REUSE_FAKE_CARGO = r'''#!/usr/bin/env bash
printf '%s\n' "$*" >> @LOG@
filter=''; separated=''
for argument in "$@"; do
  if [[ -n "$separated" && -z "$filter" && "$argument" != -* ]]; then filter="$argument"; fi
  if [[ "$argument" == -- ]]; then separated=1; fi
done
source_file=crates/demo/src/lib.rs
echo "   Compiling demo v0.1.0 ($PWD/crates/demo)"
passed=0; failed=0; results=''
for name in answer_is_42 other_is_7; do
  if [[ -n "$filter" && "tests::$name" != *"$filter"* ]]; then continue; fi
  status=FAILED
  if [[ "$name" == answer_is_42 ]] && { grep -qF '{ 42 }' "$source_file" || [[ -n "${PROVE_RED_SELFTEST_TEST_WEAKENED:-}" ]]; }; then status=ok; fi
  if [[ "$name" == other_is_7 ]] && grep -qF 'let base = 7;' "$source_file"; then status=ok; fi
  results+="test tests::$name ... $status"$'\n'
  if [[ "$status" == ok ]]; then passed=$((passed + 1)); else failed=$((failed + 1)); fi
done
printf 'running %s tests\n%s\n' "$((passed + failed))" "$results"
if (( failed )); then printf 'test result: FAILED. %s passed; %s failed; 0 ignored\n' "$passed" "$failed"; exit 101; fi
printf 'test result: ok. %s passed; 0 failed; 0 ignored\n' "$passed"
'''


# 并行那几格的样本（文件头「判法与次序」的「并行」一条）：demo 一个包五个源文件；假 cargo 照全部源文件里的数判，不点名具体的文件
PARALLEL_SOURCES = {"lib.rs": "fn answer() -> u32 { 42 }\nfn other() -> u32 { 7 }\n", "beta.rs": "fn beta() -> u32 { 42 }\n", "gamma.rs": "fn gamma() -> u32 { 42 }\n",
                    "delta.rs": "fn delta() -> u32 { 42 }\n", "epsilon.rs": "fn epsilon() -> u32 { 42 }\n"}
PARALLEL_TABLE = ("# 并行那几格的样本\n"
                  "p-elsewhere\tcrates/demo/src/beta.rs\t{ 42 }\t{ 47 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "p-caught\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --lib -- answer_caught\tanswer_is_42\n"
                  "p-same-file\tcrates/demo/src/lib.rs\t{ 7 }\t{ 8 }\t-p demo --lib -- answer_same\tanswer_is_42\n"
                  "p-broken\tcrates/demo/src/gamma.rs\t{ 42 }\t{ 46 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "p-notrun\tcrates/demo/src/delta.rs\t{ 42 }\t{ 52 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "p-stale\tcrates/demo/src/epsilon.rs\t{ 42 }\t{ 53 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "p-beta-caught\tcrates/demo/src/beta.rs\t{ 42 }\t{ 41 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "p-base-red\tcrates/demo/src/beta.rs\t{ 42 }\t{ 41 }\t-p demo --lib -- baseline_red_filter\tanswer_is_42\n")
# 点名次序；每一行要判成的那个开头（「并行判定与串行逐条相同」那一格另核串行、并行两边都是它）
PARALLEL_EXPECTED = (("p-caught", "p-caught\t抓到\t"), ("p-elsewhere", "✗ p-elsewhere\t没红\t"), ("p-same-file", "p-same-file\t抓到\t"), ("p-broken", "✗ p-broken\t无效\t"),
                     ("p-notrun", "- 本次未跑 p-notrun："), ("p-stale", "✗ p-stale\t没重编\t"))
# 并行那几格的假 cargo：每起一次往 @LOG@ 记一行「工作目录、编译目录、编译并行度、参数」。改坏的那一趟（源文件里有替换文）设了 PROVE_RED_SELFTEST_RENDEZVOUS 就在那里等齐
# PROVE_RED_SELFTEST_RENDEZVOUS_COUNT 路才往下走，等齐记 met、等不齐（30 秒）记 gave-up，之后的不再等；等齐之后读源文件定结局。41（p-caught）多睡 1 秒，让点名在前的跑完得晚：
# 8、41 点名的红；47 点名的绿、别的红；46 编不过；52 点名的绿、打 NOT_RUN；53 全绿而不打 Compiling（没重编）。参数里带 baseline_red_filter 的基线红。
# dep-info 文件名带工作目录的上一层，几路共用编译目录时各写各的（共用编译目录那条开关只红「每路各一份」那一格）
PARALLEL_FAKE_CARGO = r'''#!/usr/bin/env bash
printf '%s\t%s\t%s\t%s\n' "$PWD" "$CARGO_TARGET_DIR" "$CARGO_BUILD_JOBS" "$*" >> @LOG@
sources="$(cat crates/demo/src/*.rs)"
case "$sources" in
  *'{ 41 }'*|*'{ 8 }'*|*'{ 46 }'*|*'{ 47 }'*|*'{ 52 }'*|*'{ 53 }'*)
    rendezvous="${PROVE_RED_SELFTEST_RENDEZVOUS:-}"
    if [[ -n "$rendezvous" ]]; then
      : > "$rendezvous/arrived-$$"
      if [[ ! -e "$rendezvous/gave-up" ]]; then
        for _ in {1..600}; do
          arrived=("$rendezvous"/arrived-*)
          if (( ${#arrived[@]} >= PROVE_RED_SELFTEST_RENDEZVOUS_COUNT )); then : > "$rendezvous/met"; break; fi
          sleep 0.05
        done
        [[ -e "$rendezvous/met" ]] || : > "$rendezvous/gave-up"
      fi
      sources="$(cat crates/demo/src/*.rs)"
    fi
    ;;
esac
green='running 1 test\ntest tests::answer_is_42 ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored\n'
case "$sources" in *'{ 53 }'*) ;; *) echo "   Compiling demo v0.1.0 ($PWD/crates/demo)" ;; esac
mkdir -p "$CARGO_TARGET_DIR/debug/deps"
printf '%s: %s\n' "$CARGO_TARGET_DIR/debug/deps/demo" "$(printf '%s ' "$PWD"/crates/demo/src/*.rs)" > "$CARGO_TARGET_DIR/debug/deps/demo-$(basename "$(dirname "$PWD")").d"
if [[ "$*" == *baseline_red_filter* ]]; then printf 'test tests::answer_is_42 ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored\n'; exit 101; fi
case "$sources" in
  *'{ 46 }'*) printf 'error[E0308]: mismatched types\nerror: could not compile `demo` (lib test) due to 1 previous error\n'; exit 101 ;;
  *'{ 41 }'*) sleep 1; printf 'test tests::answer_is_42 ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored\n'; exit 101 ;;
  *'{ 8 }'*) printf 'test tests::answer_is_42 ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored\n'; exit 101 ;;
  *'{ 47 }'*) printf 'running 2 tests\ntest tests::answer_is_42 ... ok\ntest tests::other ... FAILED\n\ntest result: FAILED. 1 passed; 1 failed; 0 ignored\n'; exit 101 ;;
  *'{ 52 }'*) printf 'test tests::answer_is_42 ... ok\n---- tests::answer_is_42 stdout ----\nNOT_RUN_NO_GPU_ADAPTER test=tests::answer_is_42 reason=没卡\ntest result: ok. 1 passed; 0 failed\n'; exit 0 ;;
esac
printf "$green"
'''


def parallel_cells(work):
    """并行那十二格（文件头「判法与次序」的「并行」一条）：→ (格数, [失败说明])。同一份副本、同一份假 cargo，点名六行（五个文件）串行（PROVE_RED_LANES=1，就在副本上）与并行（等齐五路）各跑一趟，
    比判定、看各路的工作目录与编译目录；另跑丢掉一路的、一路基线红的、路数写错的，分路判法直接调。样本里没有点名的 #[test]，行键算不出：每趟都现判，不进判定库。"""
    copy = selftest_copy(work, "parallel")
    sources = {name: os.path.join(copy, "crates", "demo", "src", name) for name in PARALLEL_SOURCES}
    for name, text in PARALLEL_SOURCES.items():
        write_text(sources[name], text)
    write_text(os.path.join(copy, "crates", "mutations.tsv"), PARALLEL_TABLE)
    log = os.path.join(work, "parallel-cargo-calls.txt")
    cargo = os.path.join(work, "bin", "parallel-cargo")
    write_text(cargo, PARALLEL_FAKE_CARGO.replace("@LOG@", shlex.quote(log)), executable=True)
    parallel_jobs = "16"
    lanes_base = os.path.join(work, "parallel-lanes-base")
    names = [name for name, _start in PARALLEL_EXPECTED]
    failures = []

    def trial_in_lanes(chosen, lanes, rendezvous=None):
        """一趟证红：(退出码, 输出行, 假 cargo 每次的 [工作目录, 编译目录, 编译并行度, 参数])。"""
        settings = {LANES_VARIABLE: lanes, "CARGO_BUILD_JOBS": parallel_jobs, "CARGO_TARGET_DIR": lanes_base, "PROVE_RED_SELFTEST_RENDEZVOUS": rendezvous,
                    "PROVE_RED_SELFTEST_RENDEZVOUS_COUNT": str(len(PARALLEL_SOURCES)) if rendezvous else None}
        saved = {name: os.environ.get(name) for name in settings}
        for name, value in settings.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
        if os.path.exists(log):
            os.unlink(log)
        try:
            code, lines = prove(copy, "demo", chosen, cargo=cargo)
        finally:
            for name, value in saved.items():
                if value is None:
                    os.environ.pop(name, None)
                else:
                    os.environ[name] = value
        calls = [line.split("\t") for line in open(log, encoding="utf-8").read().splitlines()] if os.path.exists(log) else []
        return code, lines, calls

    def judged_lines(lines):
        """比串行与并行时不看的：「- 并行」那一行、随机种子的复用哨兵行与日志路径。"""
        return [re.sub(r"（日志 [^）]*）", "（日志）", line) for line in lines if not line.startswith("- 并行") and "复用哨兵" not in line]

    rendezvous = os.path.join(work, "parallel-rendezvous")
    os.makedirs(rendezvous)
    serial_code, serial_lines, _serial_calls = trial_in_lanes(names, "1")
    code, lines, calls = trial_in_lanes(names, "8", rendezvous)
    expected_missing = [start for _name, start in PARALLEL_EXPECTED if not any(line.startswith(start) for line in lines)
                        or not any(line.startswith(start) for line in serial_lines)]
    if code != serial_code or code != 1 or judged_lines(lines) != judged_lines(serial_lines) or expected_missing:
        failures.append(f"并行判定与串行逐条相同：两边都退 1、逐行相同、六行各判成写明的结局，实际串行退 {serial_code}、并行退 {code}、缺 {expected_missing}；串行 {judged_lines(serial_lines)}；并行 {judged_lines(lines)}")
    if not os.path.exists(os.path.join(rendezvous, "met")):
        failures.append(f"并行同时起：互不撞文件的五路应当同时在施加变异（假 cargo 等齐五路），实际没等齐，进程起了 {len(calls)} 次，输出 {lines}")
    plan = next((line for line in lines if line.startswith("- 并行：开 ")), "")
    planned = re.match(r"- 并行：开 (\d+) 路.*各路的工作副本与编译目录在 (.+)$", plan)
    lane_count = int(planned.group(1)) if planned else 0
    lanes_root = planned.group(2) if planned else ""
    pairs = {(call[0], call[1]) for call in calls}
    if (lane_count != len({call[0] for call in calls}) or lane_count != len({call[1] for call in calls}) or lane_count != len(pairs) or lane_count < 2
            or any(call[0] == os.path.realpath(copy) or not call[0].startswith(lanes_root) or not call[1].startswith(lanes_root) for call in calls)):
        failures.append(f"并行每路各一份工作副本与编译目录：报 {lane_count} 路，工作目录与编译目录一一对应、都在各路的目录里、不是 --copy 那一份，实际 {sorted(pairs)}，那一行 {plan!r}")
    wanted_jobs = str(max(1, int(parallel_jobs) // max(1, lane_count)))
    if {call[2] for call in calls} != {wanted_jobs}:
        failures.append(f"并行编译并行度：总数 {parallel_jobs}、{lane_count} 路，每路 {wanted_jobs}，实际 {sorted({call[2] for call in calls})}")
    unchanged = all(open(sources[name], encoding="utf-8").read() == text for name, text in PARALLEL_SOURCES.items())
    if not unchanged or not lanes_root or os.path.exists(lanes_root):
        failures.append(f"并行收尾：--copy 那份副本的源码不变、各路的目录删掉，实际源码{'没变' if unchanged else '变了'}、目录 {lanes_root!r} {'还在' if lanes_root and os.path.exists(lanes_root) else '不在'}")
    same_file_directories = {call[0] for call in calls if "answer_caught" in call[3] or "answer_same" in call[3]}
    if len(same_file_directories) != 1:
        failures.append(f"并行撞文件的行同一路：p-caught 与 p-same-file 改同一个文件，应当在同一份工作副本里跑，实际 {sorted(same_file_directories)}")
    if (code != 1 or not any(line.startswith("p-caught\t抓到\t") for line in lines) or not any(line.startswith("✗ p-elsewhere\t没红\t") for line in lines)
            or not any(line.startswith("✗ 点名 6 条") for line in lines)):
        failures.append(f"并行单路红不吃掉：头一路都抓到、别的路有没红，整次退 1、汇总句报红、逐行照列，实际退 {code} {lines}")
    verdict_order = [name for line in lines for name, start in PARALLEL_EXPECTED if line.startswith(start)]
    if verdict_order != names:
        failures.append(f"并行输出按点名次序：判定行的次序应当是 {names}（p-caught 跑完得最晚也排头一个），实际 {verdict_order}")
    # 收回数：自证放一个值进 DROP_LAST_LANE_FOR_SELFTEST，变异那一段丢掉最后派出的那一路
    DROP_LAST_LANE_FOR_SELFTEST.append(True)
    try:
        dropped_code, dropped_lines, _calls = trial_in_lanes(["p-caught", "p-beta-caught"], "8")
    finally:
        DROP_LAST_LANE_FOR_SELFTEST.clear()
    if dropped_code != 1 or not any("派出 2 路、只收回 1 路" in line for line in dropped_lines):
        failures.append(f"并行收回数核对：派出 2 路、收回 1 路，整次退 1、点名两个数，实际退 {dropped_code} {dropped_lines}")
    fake_lanes = [ROWS.Row(number, f"row-{number}", f"crates/demo/src/{path}", "a", "b", arguments, "answer_is_42", ())
                  for number, (path, arguments) in enumerate((("one.rs", "-p demo --features gpu --lib"), ("two.rs", "-p demo --lib"), ("three.rs", "-p demo --features gpu --lib"), ("four.rs", "-p demo --lib")), 1)]
    card_lanes = [[row.name for row in lane] for lane in planned_lanes(fake_lanes, 8)]
    ceiling_lanes = planned_lanes(fake_lanes, 2)
    if card_lanes != [["row-1", "row-3"], ["row-2"], ["row-4"]] or len(ceiling_lanes) != 2 or sum(len(lane) for lane in ceiling_lanes) != 4:
        failures.append(f"并行分路：要卡的两行在同一路、不要卡的各一路，上限 2 时开 2 路、四行都在，实际 {card_lanes}、上限 2 时 {[[row.name for row in lane] for lane in ceiling_lanes]}")
    refused_code, refused_lines, refused_calls = trial_in_lanes(["p-caught", "p-beta-caught"], "abc")
    if refused_code != 2 or refused_calls or not any(LANES_VARIABLE in line for line in refused_lines):
        failures.append(f"并行路数写错：{LANES_VARIABLE}=abc 整次拒绝、退 2、点名这个变量、cargo 一次都不起，实际退 {refused_code}、cargo 起了 {len(refused_calls)} 次 {refused_lines}")
    # 一路基线红（p-base-red 那一路的参数带 baseline_red_filter，别的路基线绿）：两边都退 2、一条变异都不施加（假 cargo 每一次都带 --show-output）、输出逐行相同
    baseline_names = ["p-caught", "p-base-red"]
    serial_baseline = trial_in_lanes(baseline_names, "1")
    parallel_baseline = trial_in_lanes(baseline_names, "8")
    applied = [call for each in (serial_baseline, parallel_baseline) for call in each[2] if "--show-output" not in call[3]]
    if (serial_baseline[0] != 2 or parallel_baseline[0] != 2 or applied or judged_lines(serial_baseline[1]) != judged_lines(parallel_baseline[1])
            or not any("基线（没改源码）就红" in line for line in parallel_baseline[1])):
        failures.append(f"并行基线：有一路基线红，串行、并行都退 2、一条变异都不施加、输出逐行相同，实际退 {serial_baseline[0]}、{parallel_baseline[0]}，施加了 {applied}；串行 {serial_baseline[1]}；并行 {parallel_baseline[1]}")
    return 12, failures


def library_cells(work):
    """判定库那六格（文件头「判定库与复用」，弄坏开关 library-unread、sentinel-unplanned、red-not-recorded、narrowed-recorded、narrowed-sentinel-drawn）：→ (格数, [失败说明])。
    一份副本（与别的格同一份闭包，demo 的源码与变异表换成 REUSE_SOURCE、REUSE_TABLE）、一份假 cargo、库根是自证开头 git init 的那一棵临时仓：
    头一趟（库是空的）、同一份输入第二趟、加一个没人调的函数之后一趟、改 other 函数体之后一趟，照 lib_reuse_checks.py 判复用三问、打规范输出；
    第二趟另核复用哨兵抽一条、强制现判一条；加函数那一趟哨兵取全部（不强制现判）另核 cargo 一次都不起；
    再挑一个种子让强制现判落在 answer-41、假 cargo 演点名的测试被改弱：那一趟判没红、报判定不稳、退 1、照行键记下没红，下一趟 answer-41 现判（上次没判成抓到）。
    缩过域那两格：改两条变异各自被改坏的函数之后缩域跑一趟（两条现判、不入库、判定与汇总句写缩过域），不缩域再跑一趟（两条仍现判），
    再缩域跑一趟（库里有了全域下判的抓到：两条都复用、不抽哨兵、假 cargo 一次都不起）。"""
    copy = selftest_copy(work, "reuse")
    source = os.path.join(copy, "crates", "demo", "src", "lib.rs")
    write_text(source, REUSE_SOURCE)
    write_text(os.path.join(copy, "crates", "mutations.tsv"), REUSE_TABLE)
    log = os.path.join(work, "reuse-cargo-arguments.txt")
    cargo = os.path.join(work, "bin", "reuse-cargo")
    write_text(cargo, REUSE_FAKE_CARGO.replace("@LOG@", shlex.quote(log)), executable=True)
    names = ["answer-41", "other-8"]
    items = {row.name: ROWS.sentinel_item(row) for row in ROWS.read_table(os.path.join(copy, "crates", "mutations.tsv"))[0]}
    count_variable, seed_variable = ROWS.reuse_checks.SENTINEL_COUNT_VARIABLE, ROWS.reuse_checks.SENTINEL_SEED_VARIABLE
    narrowing_variables = tuple(ROWS.admission_module().CRASH_CASE_NARROWING_VARIABLES)
    failures = []

    def trial_with_sentinels(count, seed=None, weakened=False, narrowing=None):
        """一趟证红：(退出码, 输出行, {变异名: 结局}, 假 cargo 起了几次)。count、seed 是这一趟复用哨兵的条数与种子（seed 不给就不设）；
        narrowing 给了是这一趟缩域：{缩域变量: 值}（文件头「判定库与复用」的「缩过域的一趟」），不给两个缩域变量都不设。"""
        saved = {name: os.environ.get(name) for name in (count_variable, seed_variable, "PROVE_RED_SELFTEST_TEST_WEAKENED") + narrowing_variables}
        os.environ[count_variable] = str(count)
        for name in (seed_variable, "PROVE_RED_SELFTEST_TEST_WEAKENED") + narrowing_variables:
            os.environ.pop(name, None)
        if seed is not None:
            os.environ[seed_variable] = str(seed)
        if weakened:
            os.environ["PROVE_RED_SELFTEST_TEST_WEAKENED"] = "1"
        os.environ.update(narrowing or {})
        if os.path.exists(log):
            os.unlink(log)
        try:
            code, lines = prove(copy, "demo", names, cargo=cargo)
        finally:
            for name, value in saved.items():
                if value is None:
                    os.environ.pop(name, None)
                else:
                    os.environ[name] = value
        calls = open(log, encoding="utf-8").read().splitlines() if os.path.exists(log) else []
        return code, lines, reuse_states(lines), len(calls)

    def replace_once(old, new):
        with open(source, encoding="utf-8") as handle:
            text = handle.read()
        write_text(source, text.replace(old, new, 1))

    first = trial_with_sentinels(1, 5)
    second = trial_with_sentinels(1, 5)
    replace_once("\n#[cfg(test)]", "pub fn spare() -> u32 { 5 }\n\n#[cfg(test)]")   # 闭包外：没人调的函数
    unrelated = trial_with_sentinels(1000)
    replace_once("    base\n}", "    base + 0\n}")   # 闭包内：other-8 改坏的那个函数（other_is_7 的闭包里有它），answer-41 那一条的闭包里没有
    unit = trial_with_sentinels(1, 5)
    runs = (first, second, unrelated, unit)
    passed, problems = ROWS.reuse_checks.report_three_questions(SCRIPT_KEY, first[2], second[2], unrelated[2], unit[2], ["other-8"])
    if not passed or any(each[0] != 0 for each in runs):
        failures.append(f"reuse-three-questions：四趟退 {[each[0] for each in runs]}、结局 {[each[2] for each in runs]}；{'；'.join(problems)}；末一趟输出 {unit[1]}")
    if (second[0] != 0 or sorted(second[2].values()) != ["复用", "强制现判"]
            or not any(line.strip().startswith("· 复用哨兵（证红）：种子 5") for line in second[1])):
        failures.append(f"复用哨兵：同一份输入的第二趟从键相同的两条里抽一条哨兵、强制现判另一条（种子 5、哨兵 1 条），哨兵复用、强制现判与库里同为抓到、退 0，"
                        f"实际退 {second[0]}、结局 {second[2]}、输出 {second[1]}")
    if (unrelated[0] != 0 or unrelated[3] != 0 or unrelated[2] != {"answer-41": "复用", "other-8": "复用"}
            or not any("现判 0 条、复用 2 条、拒算 0 条" in line for line in unrelated[1])
            or not any("现判 2 条、复用 0 条、拒算 0 条" in line for line in first[1])):
        failures.append(f"键相同的都复用：加一个没人调的函数之后两条都复用、假 cargo 一次都不起、成功句报现判 0 条、复用 2 条、拒算 0 条（头一趟报现判 2 条、复用 0 条），"
                        f"实际退 {unrelated[0]}、假 cargo 起了 {unrelated[3]} 次、结局 {unrelated[2]}、输出 {unrelated[1]}；头一趟输出 {first[1]}")

    def forced_under(seed):
        planned = ROWS.reuse_checks.ReuseSentinel(SCRIPT_KEY, SENTINEL_LABEL, count=1, seed=seed)
        planned.plan({items[name]: "抓到" for name in names}, printer=lambda _line: None)
        return planned.forced_item

    seed = next((candidate for candidate in range(256) if forced_under(candidate) == items["answer-41"]), None)
    weakened = trial_with_sentinels(1, seed, weakened=True) if seed is not None else (None, [], {}, 0)
    after = trial_with_sentinels(1000)
    rejudged = [line for line in after[1] if line.startswith("answer-41\t")]
    if (seed is None or weakened[0] != 1 or weakened[2].get("answer-41") != "强制现判"
            or not any(line.startswith("✗ answer-41\t没红\t") for line in weakened[1]) or not any("判定不稳：" in line for line in weakened[1])
            or after[0] != 0 or after[2] != {"answer-41": "现判", "other-8": "复用"} or not any("上次没判成抓到" in line for line in rejudged)):
        failures.append(f"记红撤绿：强制现判落在 answer-41、点名的测试被改弱的那一趟判没红、报判定不稳、退 1，没红照行键记进库，下一趟 answer-41 现判（上次没判成抓到）、other-8 复用、退 0，"
                        f"实际种子 {seed}、那一趟退 {weakened[0]}、结局 {weakened[2]}、输出 {weakened[1]}；下一趟退 {after[0]}、结局 {after[2]}、输出 {after[1]}")

    # 缩过域的一趟（文件头「判定库与复用」的「缩过域的一趟」）：先改两条变异各自被改坏的函数（行键都变、库里没有），缩域跑一趟、再不缩域跑一趟，
    # 再缩域跑一趟（库里有了全域下判的抓到，复用哨兵取 1 条、强制现判 1 条）
    replace_once("pub fn answer() -> u32 { 42 }", "pub fn answer() -> u32 { let _unused = 0; { 42 } }")
    replace_once("    base + 0\n}", "    base + 0 + 0\n}")
    narrowed = trial_with_sentinels(1000, narrowing={"SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO": "3"})
    plain = trial_with_sentinels(1000)
    narrowed_reuse = trial_with_sentinels(1, 5, narrowing={"SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO": "5"})
    narrowed_rows = [line for line in narrowed[1] if line.startswith(("answer-41\t", "other-8\t"))]
    if (narrowed[0] != 0 or narrowed[2] != {"answer-41": "现判", "other-8": "现判"} or len(narrowed_rows) != 2
            or not all("；缩过域（SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=3）、不算门禁通过、没入库（现判：" in line for line in narrowed_rows)
            or not any(line.startswith("✓ 点名 2 条：") and "；缩过域（SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=3）、不算门禁通过、现判的没入库" in line for line in narrowed[1])
            or plain[0] != 0 or plain[2] != {"answer-41": "现判", "other-8": "现判"} or any("缩过域" in line for line in plain[1])):
        failures.append(f"缩过域·现判不入库：设了 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=3 的一趟两条都现判、都抓到、退 0，两行判定与汇总句写缩过域（变量=值）、不算门禁通过、没入库；"
                        f"同一份输入不设变量再跑一趟两条仍现判、不复用，实际缩域那一趟退 {narrowed[0]}、结局 {narrowed[2]}、输出 {narrowed[1]}；"
                        f"不设变量那一趟退 {plain[0]}、结局 {plain[2]}、输出 {plain[1]}")
    if (narrowed_reuse[0] != 0 or narrowed_reuse[2] != {"answer-41": "复用", "other-8": "复用"} or narrowed_reuse[3] != 0
            or not any("缩过域（SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5）：强制现判出的是缩域下的结论" in line and "不抽哨兵" in line for line in narrowed_reuse[1])
            or not any(line.startswith("✓ 点名 2 条：") and "；缩过域（SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5）、不算门禁通过" in line for line in narrowed_reuse[1])):
        failures.append(f"缩过域·复用全域判定：库里有了全域下判的抓到之后设 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5（复用哨兵取 1 条、强制现判 1 条）再跑：两条都复用、"
                        f"假 cargo 一次都不起、打一行不抽哨兵与缩过域的原因、汇总句写缩过域、退 0，实际退 {narrowed_reuse[0]}、假 cargo 起了 {narrowed_reuse[3]} 次、"
                        f"结局 {narrowed_reuse[2]}、输出 {narrowed_reuse[1]}")
    return 6, failures


# 不领卡那一格的样本表（文件头「不领卡」）：cards-plain 不带 gpu 特性，cards-gpu 带
NO_CARD_LEASE_TABLE = ("# 样本\n"
                       "cards-plain\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --lib -- answer\tanswer_is_42\n"
                       "cards-gpu\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --features gpu --lib -- answer\tanswer_is_42\n")


GPU_ROUTE_TABLE = ("# 开 GPU 补 gpu 特性那一格的样本：两行都不写特性，route-41 点名的测试调到 answer（这一格把它当崩溃判段的选路函数），plain-8 点名的测试调不到\n"
                   "route-41\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --lib -- answer_is_42\tanswer_is_42\n"
                   "plain-8\tcrates/demo/src/lib.rs\tlet base = 7;\tlet base = 8;\t-p demo --lib -- other_is_7\tother_is_7\n")


def gpu_route_cells(work):
    """开 GPU 补 gpu 特性那一格（文件头「判法与次序」的「开 GPU 补 gpu 特性」一条，弄坏开关 gpu-route-feature-unadded）：→ (格数, [失败说明])。
    副本的 demo 换成 REUSE_SOURCE、表换成 GPU_ROUTE_TABLE，假 cargo 同判定库那几格（每起一次记一行参数）；crates-mutation-rows.py 的 GPU_ROUTE_UNITS 临时换成 demo 的 answer。
    多机配置 ENABLE_GPU=1（SINGLEFS_MULTI_HOST_CONFIG 指这一格自己写的一份）：route-41 起的每一次 cargo（基线与变异）`--` 之前带 --features gpu 与 --release，
    plain-8 一次都不带，输出点名补了 route-41、不点 plain-8，两行都抓到、退 0；配置换成 ENABLE_GPU=0 再证一趟：route-41 起的 cargo 不带 --features。"""
    copy = selftest_copy(work, "gpu-route")
    write_text(os.path.join(copy, "crates", "demo", "src", "lib.rs"), REUSE_SOURCE)
    write_text(os.path.join(copy, "crates", "mutations.tsv"), GPU_ROUTE_TABLE)
    log = os.path.join(work, "gpu-route-cargo-arguments.txt")
    cargo = os.path.join(work, "bin", "gpu-route-cargo")
    write_text(cargo, REUSE_FAKE_CARGO.replace("@LOG@", shlex.quote(log)), executable=True)
    configurations = {}
    for state, value in (("on", "1"), ("off", "0")):
        configurations[state] = os.path.join(work, f"gpu-route-{state}", "multi-host.env")
        write_text(configurations[state], f"# 自证用的多机配置：GPU {'开' if value == '1' else '关'}\nENABLE_GPU={value}\n")
    saved_units, saved_configuration = ROWS.GPU_ROUTE_UNITS[0], os.environ.get("SINGLEFS_MULTI_HOST_CONFIG")
    trials = {}
    try:
        ROWS.GPU_ROUTE_UNITS[0] = (("demo", "crates/demo/src/lib.rs", "answer"),)
        for state in ("on", "off"):
            os.environ["SINGLEFS_MULTI_HOST_CONFIG"] = configurations[state]
            # 两条点名测试的函数体每一趟各带一句这一格、这一趟独有的断言说明：行键跟着变，判定库里别的格（与上一趟）判过的同键判定复用不到，
            # 这一趟两行都现判、都起 cargo（复用到了就一次 cargo 都不起，这一格核不到参数）
            write_text(os.path.join(copy, "crates", "demo", "src", "lib.rs"),
                       REUSE_SOURCE.replace("assert_eq!(super::answer(), 42);", f'assert_eq!(super::answer(), 42, "gpu route cell {state}");')
                                   .replace("assert_eq!(super::other(), 7);", f'assert_eq!(super::other(), 7, "gpu route cell {state}");'))
            if os.path.exists(log):
                os.unlink(log)
            code, lines = prove(copy, "demo", ["route-41", "plain-8"], cargo=cargo)
            calls = open(log, encoding="utf-8").read().splitlines() if os.path.exists(log) else []
            trials[state] = (code, lines, [call.split(" -- ")[0].split() for call in calls if "answer_is_42" in call],
                             [call.split(" -- ")[0].split() for call in calls if "other_is_7" in call])
    finally:
        ROWS.GPU_ROUTE_UNITS[0] = saved_units
        if saved_configuration is None:
            os.environ.pop("SINGLEFS_MULTI_HOST_CONFIG", None)
        else:
            os.environ["SINGLEFS_MULTI_HOST_CONFIG"] = saved_configuration
    code, lines, route_calls, plain_calls = trials["on"]
    off_code, off_lines, off_route_calls, _off_plain_calls = trials["off"]
    failures = []
    if (code != 0 or len(route_calls) < 2 or not plain_calls
            or any(words[words.index("--features") + 1:words.index("--features") + 2] != ["gpu"] if "--features" in words else True for words in route_calls)
            or any(words.count("--release") != 1 for words in route_calls)
            or any("--features" in words or "--release" in words for words in plain_calls)
            or not any(line.startswith("- route-41：cargo 参数补上 --features gpu 起（多机配置 ") for line in lines)
            or any(line.startswith("- plain-8：") for line in lines)
            or not any(line.startswith("route-41\t抓到\t") for line in lines) or not any(line.startswith("plain-8\t抓到\t") for line in lines)
            or off_code != 0 or not off_route_calls or any("--features" in words for words in off_route_calls)
            or any(line.startswith("- route-41：cargo 参数补上 --features") for line in off_lines)):
        failures.append(f"开 GPU 补 gpu 特性：行不写特性、配置 ENABLE_GPU=1、点名测试够得着选路函数，起的每一次 cargo 带 --features gpu 与 --release、输出点名补了那一行，"
                        f"够不着的那一行不带；ENABLE_GPU=0 时不带。实际 开：退 {code}、route-41 的 cargo {route_calls}、plain-8 的 cargo {plain_calls}、输出 {lines}；"
                        f"关：退 {off_code}、route-41 的 cargo {off_route_calls}、输出 {off_lines}")
    return 1, failures


def no_card_lease_cells(work, fake_cargo):
    """不领卡那一格（文件头「不领卡」）：→ (格数, [失败说明])。包着 fake_cargo 的假 cargo 把每次起 cargo 的参数、环境里的卡串与 VK_DRIVER_FILES 记进一份事件日志；
    环境里设 GPU_CARDS_BY_PRIORITY=FAKE-CARD-0。cards-plain 与 cards-gpu 各证一趟：每趟起两条 cargo（基线、变异），都不带 --no-run（不另编）、
    收到的卡串都是 FAKE-CARD-0（原样交）、都没有藏卡的 VK_DRIVER_FILES，照常抓到、退 0。弄坏开关 gpu-rows-compiled-apart、gpu-rows-cards-dropped 下 cards-gpu 那一趟判错。"""
    copy = selftest_copy(work, "no-card-lease")
    write_text(os.path.join(copy, "crates", "mutations.tsv"), NO_CARD_LEASE_TABLE)
    events = os.path.join(work, "no-card-lease-events.txt")
    logging_cargo = os.path.join(work, "bin", "no-card-lease-cargo")
    write_text(logging_cargo,
               "#!/usr/bin/env bash\n"
               f"printf 'cargo %s cards=%s vk=%s\\n' \"$*\" \"${{GPU_CARDS_BY_PRIORITY-none}}\" \"${{VK_DRIVER_FILES-unset}}\" >> {shlex.quote(events)}\n"
               f"exec {shlex.quote(fake_cargo)} \"$@\"\n", executable=True)
    failures = []

    def trial(chosen):
        """→ (退出码, 输出行, 事件日志的行)。GPU_CARDS_BY_PRIORITY 只在这一趟里设。"""
        saved = os.environ.get("GPU_CARDS_BY_PRIORITY")
        os.environ["GPU_CARDS_BY_PRIORITY"] = "FAKE-CARD-0"
        if os.path.exists(events):
            os.unlink(events)
        try:
            code, lines = prove(copy, "demo", chosen, cargo=logging_cargo)
        finally:
            if saved is None:
                os.environ.pop("GPU_CARDS_BY_PRIORITY", None)
            else:
                os.environ["GPU_CARDS_BY_PRIORITY"] = saved
        return code, lines, [line for line in open(events).read().splitlines() if line.startswith("cargo")] if os.path.exists(events) else []

    for name in ("cards-plain", "cards-gpu"):
        code, lines, calls = trial([name])
        if (code != 0 or len(calls) != 2 or any(" --no-run" in call for call in calls)
                or any("cards=FAKE-CARD-0 " not in call or not call.endswith("vk=unset") for call in calls)
                or not any(line.startswith(f"{name}\t抓到\t") for line in lines)):
            failures.append(f"不领卡：{name} 起两条 cargo（基线、变异），都不带 --no-run、收到的卡串都是环境里给的 FAKE-CARD-0、都没有藏卡的 VK_DRIVER_FILES，照常抓到、退 0，"
                            f"实际退 {code}、cargo {calls}、输出 {lines}")
    return 1, failures


def trip_build_directory_cells(work, fake_cargo, target_log):
    """编译目录归这一趟三格（文件头「判法与次序」的「编译目录归这一趟」）：→ (格数, [失败说明])。调用方没给 CARGO_TARGET_DIR（这里取下来）。
    ① 每一趟新建：副本里留着别的命令编出的 target（里面 demo 的产物比源码新，沿用就判没重编），连起两趟证红：每一趟的 cargo 看到的编译目录恰好一个、都不是 <副本>/target、
       两趟互不相同，照常抓到、退 0，副本里那份旧 target 原样留着（不是这一趟建的，不用、不删）；
    ② 跑完删：上面两趟与基线红（退 2）那一趟的编译目录，跑完一个都不在了；
    ③ 被打断删：子进程里建一份、装好处理函数、睡着，点名发它 TERM，那份编译目录删了、进程按默认处理停（被 TERM 杀死）。
    弄坏开关 copy-target-reused 下 ① ② 判错，trip-target-kept 下 ② 判错，trip-target-kept-on-stop 下 ③ 判错。"""
    failures = []
    copy = selftest_copy(work, "trip-copy")
    old_target = os.path.join(copy, "target")
    old_artifact = os.path.join(old_target, "fake-build", "demo.rs")
    write_text(old_artifact, "fn answer() -> u32 { 42 }\n")
    write_text(os.path.join(old_target, "debug", "deps", "demo.d"), f"{old_artifact}: {os.path.realpath(copy)}/crates/demo/src/lib.rs\n")
    tomorrow = time.time() + 86400
    os.utime(old_artifact, (tomorrow, tomorrow))
    saved_target = os.environ.pop("CARGO_TARGET_DIR", None)
    try:
        trips = []
        for chosen, crate in ((["caught"], "demo"), (["caught"], "demo"), (["redbase"], "redbase")):
            if os.path.exists(target_log):
                os.unlink(target_log)
            code, lines = prove(copy, crate, chosen, cargo=fake_cargo)
            trips.append((code, lines, set(open(target_log).read().splitlines()) if os.path.exists(target_log) else set()))
        first, second, red = trips
        if (first[0] != 0 or second[0] != 0 or not any("\t抓到\t" in line for line in first[1] + second[1])
                or len(first[2]) != 1 or len(second[2]) != 1 or first[2] == second[2] or old_target in first[2] | second[2] or not os.path.exists(old_artifact)
                or not all(directory.startswith(os.path.join(os.path.realpath(tempfile.gettempdir()), TRIP_BUILD_DIRECTORY_PREFIX)) for directory in first[2] | second[2])):
            failures.append(f"编译目录归这一趟·每一趟新建：副本里留着别的命令编出的 target（产物比源码新）时，没给 CARGO_TARGET_DIR 的每一趟都自己建一份新的，不沿用 <副本>/target、"
                            f"两趟互不相同，照常抓到、退 0，旧 target 原样留着；实际两趟退 {first[0]}、{second[0]}，cargo 看到的编译目录 {sorted(first[2])}、{sorted(second[2])}，"
                            f"旧产物还在 {os.path.exists(old_artifact)}，输出 {first[1] + second[1]}")
        left = sorted(directory for directory in first[2] | second[2] | red[2] if os.path.exists(directory))
        if red[0] != 2 or not red[2] or left:
            failures.append(f"编译目录归这一趟·跑完删：成功两趟与基线红那一趟（退 2）跑完，这一趟自己建的编译目录一个都不留；实际基线红那一趟退 {red[0]}，"
                            f"cargo 看到过 {sorted(first[2] | second[2] | red[2])}，跑完还在的 {left}")
    finally:
        for directory in list(TRIP_BUILD_DIRECTORIES):
            shutil.rmtree(directory, ignore_errors=True)
            TRIP_BUILD_DIRECTORIES.remove(directory)
        if saved_target is not None:
            os.environ["CARGO_TARGET_DIR"] = saved_target
    ready = os.path.join(work, "trip-stop-ready")
    child = os.fork()
    if child == 0:
        try:
            remove_trip_build_directories_when_stopped()
            held_directory = new_trip_build_directory()
            write_text(os.path.join(held_directory, "artifact"), "x")
            write_text(ready, held_directory)
            time.sleep(60)
        finally:
            os._exit(0)
    deadline = time.monotonic() + 30
    while not os.path.exists(ready) and time.monotonic() < deadline:
        time.sleep(0.05)
    os.kill(child, signal.SIGTERM if os.path.exists(ready) else signal.SIGKILL)
    _pid, status = os.waitpid(child, 0)
    held = open(ready).read() if os.path.exists(ready) else ""
    if not held or os.path.exists(held) or not os.WIFSIGNALED(status) or os.WTERMSIG(status) != signal.SIGTERM:
        failures.append(f"编译目录归这一趟·被打断删：这一趟自己建了编译目录的进程收到 TERM，处理函数删了它再按默认处理停（被 TERM 杀死）；"
                        f"实际 就绪 {bool(held)}、编译目录 {held} 还在 {bool(held) and os.path.exists(held)}、退出状态 {status}")
        if held:
            shutil.rmtree(held, ignore_errors=True)
    return 3, failures


# 变异那一趟连哪一处那一格的探针（假 cargo 在 PROVE_RED_SELFTEST_SERVICE_LOG 设了时调它，参数是 baseline 或 mutated）：照收到的连哪一处往服务里写一条
# （演引擎写崩溃判定：键 $PROVE_RED_SELFTEST_KEY_PREFIX 接「<参数>-<进程号>」），再往 $PROVE_RED_SELFTEST_SERVICE_LOG 追加一行
# 「<参数><TAB><SINGLEFS_PRUNE_SERVER_SOCKET><TAB><PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET><TAB><这一刻那个 socket 在不在：1 或 0>」；客户端从 $PROVE_RED_SELFTEST_CLIENT_DIRECTORY 载入
SERVICE_PROBE_PROGRAM = '''import os, sys
sys.path.insert(0, os.environ["PROVE_RED_SELFTEST_CLIENT_DIRECTORY"])
import prune_server_client
kind = sys.argv[1]
socket_path = os.environ.get("SINGLEFS_PRUNE_SERVER_SOCKET", "")
prune_server_client.process_client().write_many([((os.environ["PROVE_RED_SELFTEST_KEY_PREFIX"] + f"{kind}-{os.getpid()}").encode(), b"red")])
with open(os.environ["PROVE_RED_SELFTEST_SERVICE_LOG"], "a", encoding="utf-8") as service_log:
    service_log.write(f"{kind}\\t{socket_path}\\t{os.environ.get('PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET', '')}\\t{int(os.path.exists(socket_path))}\\n")
'''


def selftest():
    # 读写判定库：只在临时 prune-server 底下跑（文件头用法一段；scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service）
    client = ROWS.mutation_units.Library.client()
    service_problem = client.selftest_service_problem()
    if service_problem:
        print(f"  ✗ prove-red 自证没跑：{service_problem}")  # gate-lint:detail
        print(f"  → 怎么办：{client.SELFTEST_SERVICE_REMEDY}：python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- "
              "bash scripts/mutation/prove-red.sh --selftest")
        return 2
    work = tempfile.mkdtemp(prefix="prove-red-selftest-")
    # Rust 词法单元的磁盘缓存放这一趟自己的临时目录，不碰真仓那一份
    cache_variables = {"FUNCTION_UNITS_LEX_CACHE_DIRECTORY": os.path.join(work, "rust-lex-cache")}
    saved_caches = {name: os.environ.get(name) for name in cache_variables}
    os.environ.update(cache_variables)
    # 调用方环境里的卡串不进各格：不领卡那一格自己设
    saved_card_variables = {name: os.environ.pop(name, None) for name in ("GPU_CARDS_BY_PRIORITY",)}
    # 调用方给的 CARGO_TARGET_DIR 不进各格（多机可用：服务在第二台给每趟一份编译目录、本机不给）：假 cargo 照「编译目录 + 包名」放假产物、按修改时刻判新不新鲜，
    #   几份假副本共用一个编译目录时，后建的副本编出的改坏产物比 copy 那份的源码新、顶掉它，之后 copy 上的基线就红，结局随派到哪台变。
    #   各格照不设算（每一趟自己建一份新的编译目录、跑完删）；要共用编译目录的那几格自己设、自己换回。弄坏开关 inherited-target-kept 照用调用方的，没给就替它设一份共用的
    inherited_target = os.environ.pop("CARGO_TARGET_DIR", None)
    if BROKEN == "inherited-target-kept":
        os.environ["CARGO_TARGET_DIR"] = inherited_target or os.path.join(work, "inherited-target")
    try:
        copy = selftest_copy(work)
        CLOSURE_ROOT_FOR_SELFTEST.append(selftest_copy(work, "closure-main"))   # 各格的假副本照这棵假主工作区核缺不缺（闭包只有 Cargo.toml 与 crates）
        # 判定库的库根（文件头「判定库与复用」）：一棵自己 git init 的临时仓，库在临时服务里另占一份命名空间、开跑时是空的
        library_root = os.path.join(work, "library-root")
        os.makedirs(library_root)
        initialized = subprocess.run(["git", "init", "-q", library_root], capture_output=True, text=True, stdin=subprocess.DEVNULL)
        if initialized.returncode != 0:
            print(f"  ✗ 自检：判定库的库根 {library_root} git init 退 {initialized.returncode}：{(initialized.stdout + initialized.stderr).strip()}")  # gate-lint:detail
            print("  → 看 git 在不在 PATH 上、临时目录可不可写；判定库那几格要一棵 git 工作树当库根")
            return 1
        LIBRARY_ROOT_FOR_SELFTEST.append(library_root)
        fake_cargo = os.path.join(work, "bin", "cargo")
        target_log = os.path.join(work, "target-directories.txt")
        cargo_log = os.path.join(work, "cargo-arguments.txt")
        incremental_log = os.path.join(work, "cargo-release-incremental.txt")
        service_log = os.path.join(work, "cargo-service.txt")
        service_probe = os.path.join(work, "service-probe.py")
        write_text(service_probe, SERVICE_PROBE_PROGRAM)
        lock_log = os.path.join(work, "copy-lock.txt")
        # 假 cargo 照源码里那个数打 libtest / cargo 的原样形态：42 基线全绿；45 是 #[should_panic] 的用例没 panic
        # （结果行在名字与「 ... 」之间多一段「 - should panic」，cargo 收尾还打一行 error: test failed）；46 替换之后编不过；
        # 47 点名的测试绿、别的测试红（同样带 error: test failed 与 test result: 行）；别的数照旧只打点名的那一行 FAILED。
        # 读哪份源码看 -p：-p <包> 读 crates/<包>/src/lib.rs（redbase 那个包的源码是 50，基线就红）。
        # 每起一次先往 target_log 记一行它看到的 CARGO_TARGET_DIR（编译目录那两格核它），往 cargo_log 记一行参数（验证包那两格核它起没起），
        # 往 incremental_log 记一行「参数<TAB>它看到的 CARGO_PROFILE_RELEASE_INCREMENTAL」（基线不带 release 增量覆盖那一格核它）
        write_text(fake_cargo,
                   "#!/usr/bin/env bash\n"
                   f"printf '%s\\n' \"${{CARGO_TARGET_DIR-<未设>}}\" >> {shlex.quote(target_log)}\n"
                   f"printf '%s\\n' \"$*\" >> {shlex.quote(cargo_log)}\n"
                   f"printf '%s\\t%s\\n' \"$*\" \"${{CARGO_PROFILE_RELEASE_INCREMENTAL-<未设>}}\" >> {shlex.quote(incremental_log)}\n"
                   # 副本锁那一格：起的时候副本根（当前目录）上的锁被人拿着记 held，没有锁文件或拿得到记 free
                   f"if [[ -e {shlex.quote(COPY_LOCK.COPY_LOCK_NAME)} ]] && ! flock -n {shlex.quote(COPY_LOCK.COPY_LOCK_NAME)} true 2>/dev/null; then echo held; else echo free; fi >> {shlex.quote(lock_log)}\n"
                   "echo \"stamp=${PROVE_RED_SELFTEST_STAMP-none}\"\n"   # 日志目录那一格靠它认出每个基线日志是哪一趟写的
                   "package=demo; previous=''\n"
                   "for argument in \"$@\"; do [[ \"$previous\" == -p ]] && package=\"$argument\"; previous=\"$argument\"; done\n"
                   "source_file=\"crates/$package/src/lib.rs\"\n"
                   # 照 cargo 按 mtime 判新不新鲜：编译目录里这个包的产物不在、或源码比它新，才重编（拷一份当产物、照真 cargo 打 Compiling 行）；
                   # 测试跑的是产物，不是此刻的源码（编译目录里留着比源码新的旧产物时，测试跑在旧产物上）
                   "build_directory=\"${CARGO_TARGET_DIR:-target}/fake-build\"; mkdir -p \"$build_directory\"; artifact=\"$build_directory/$package.rs\"\n"
                   "if [[ ! -e \"$artifact\" || \"$source_file\" -nt \"$artifact\" ]]; then cp \"$source_file\" \"$artifact\"; echo \"   Compiling $package v0.1.0 ($PWD/crates/$package)\"\n"
                   # 照 rustc 写 dep-info（<编译目录>/debug/deps/<包>.d 列出读过的源文件）：没重编那一格靠它认出这个文件是这一趟编译读的
                   "  mkdir -p \"${CARGO_TARGET_DIR:-target}/debug/deps\"; echo \"$artifact: $PWD/$source_file\" > \"${CARGO_TARGET_DIR:-target}/debug/deps/$package.d\"; fi\n"
                   # 变异那一趟连哪一处的那一格（PROVE_RED_SELFTEST_SERVICE_LOG 设了才做）：产物里是没改坏的「{ 42 }」就是基线、不然是变异，照收到的连哪一处往服务里写一条、记一行
                   f"if [[ -n \"${{PROVE_RED_SELFTEST_SERVICE_LOG-}}\" ]]; then if grep -q '{{ 42 }}' \"$artifact\"; then run_kind=baseline; else run_kind=mutated; fi; "
                   f"python3 {shlex.quote(service_probe)} \"$run_kind\" || exit 99; fi\n"
                   # --no-run（弄坏开关 gpu-rows-compiled-apart 下那一条只编不跑的）：编过了只打 Executable 行退 0，替换之后编不过（46）照打编译错误退 101
                   "if [[ \" $* \" == *' --no-run '* ]]; then\n"
                   "  case \"$(cat \"$artifact\")\" in\n"
                   "  *'{ 46 }'*) printf 'error[E0308]: mismatched types\\n --> crates/demo/src/lib.rs:1:22\\n\\n"
                   "error: could not compile \\x60demo\\x60 (lib test) due to 1 previous error\\n'; exit 101 ;;\n"
                   "  esac\n"
                   "  echo '  Executable unittests src/lib.rs'; exit 0\n"
                   "fi\n"
                   "case \"$(cat \"$artifact\")\" in\n"
                   "*'{ 42 }'*) printf 'running 2 tests\\ntest tests::answer_is_42 ... ok\\ntest tests::answer_panics - should panic ... ok\\n\\n"
                   "test result: ok. 2 passed; 0 failed; 0 ignored\\n'; exit 0 ;;\n"
                   "*'{ 45 }'*) printf 'running 2 tests\\ntest tests::answer_is_42 ... ok\\ntest tests::answer_panics - should panic ... FAILED\\n\\n"
                   "failures:\\n\\n---- tests::answer_panics stdout ----\\nnote: test did not panic as expected at crates/demo/src/lib.rs:9:5\\n\\n"
                   "failures:\\n    tests::answer_panics\\n\\ntest result: FAILED. 1 passed; 1 failed; 0 ignored\\n\\n"
                   "error: test failed, to rerun pass \\x60-p demo --lib\\x60\\n'; exit 101 ;;\n"
                   "*'{ 46 }'*) printf 'error[E0308]: mismatched types\\n --> crates/demo/src/lib.rs:1:22\\n\\n"
                   "error: could not compile \\x60demo\\x60 (lib test) due to 1 previous error\\n'; exit 101 ;;\n"
                   "*'{ 47 }'*) printf 'running 2 tests\\ntest tests::answer_is_42 ... ok\\ntest tests::answer_panics - should panic ... FAILED\\n\\n"
                   "failures:\\n    tests::answer_panics\\n\\ntest result: FAILED. 1 passed; 1 failed; 0 ignored\\n\\n"
                   "error: test failed, to rerun pass \\x60-p demo --lib\\x60\\n'; exit 101 ;;\n"
                   # 52：编过了、失败在链接（头一条变异链接不过的样子：没有 error[E<码>]，有 error: linking with 与 rust-lld 的报错）
                   "*'{ 52 }'*) printf 'error: linking with \\x60cc\\x60 failed: exit status: 1\\n  = note: rust-lld: error: undefined hidden symbol: anon.0123456789abcdef.0.llvm.1\\n\\n"
                   "error: could not compile \\x60demo\\x60 (lib test) due to 1 previous error\\n'; exit 101 ;;\n"
                   "*) echo 'test tests::answer_is_42 ... FAILED'; exit 101 ;;\n"
                   "esac\n", executable=True)
        os.environ["PROVE_RED_LOG_DIRECTORY"] = os.path.join(work, "logs")
        failures = []
        checked_cases = 0

        def cargo_calls():
            calls = open(cargo_log).read().splitlines() if os.path.exists(cargo_log) else []
            if os.path.exists(cargo_log):
                os.unlink(cargo_log)
            return calls

        code, lines = prove(copy, "demo", ["caught"], cargo=fake_cargo)
        checked_cases += 1
        if code != 0 or not any("\t抓到\t" in line for line in lines):
            failures.append(f"会红的变异应当抓到、退 0，实际 {code} {lines}")
        # 拒算（文件头「判定库与复用」）：样本里没有点名的那条 #[test]，行键算不出：照跑、打一行为什么、那一行标（拒算）、成功句报拒算 1 条
        checked_cases += 1
        if (not any(line.startswith("- 拒算 caught：行键算不出") for line in lines) or reuse_states(lines).get("caught") != "拒算"
                or not any("现判 0 条、复用 0 条、拒算 1 条" in line for line in lines)):
            failures.append(f"拒算：行键算不出的行照跑、打一行为什么、那一行标（拒算）、成功句报现判 0 条、复用 0 条、拒算 1 条，实际 {lines}")
        checked_cases += 1
        if open(os.path.join(copy, "crates", "demo", "src", "lib.rs")).read() != "fn answer() -> u32 { 42 }\n":
            failures.append("跑完没把源码写回原样")
        # 基线起的 cargo 不带门禁加的 release 增量覆盖（文件头「判定库与复用」的基线一条）：调用方设 CARGO_PROFILE_RELEASE_INCREMENTAL=false，一趟 caught 跑下来
        #   基线那一次收到 false（调用方的值）、变异那一次收到 true（门禁加的，顶掉调用方的）。弄坏开关 baseline-incremental-kept 下基线收到 true
        if os.path.exists(incremental_log):
            os.unlink(incremental_log)
        saved_incremental = os.environ.get("CARGO_PROFILE_RELEASE_INCREMENTAL")
        os.environ["CARGO_PROFILE_RELEASE_INCREMENTAL"] = "false"
        try:
            incremental_code, incremental_lines = prove(copy, "demo", ["caught"], cargo=fake_cargo)
        finally:
            if saved_incremental is None:
                os.environ.pop("CARGO_PROFILE_RELEASE_INCREMENTAL", None)
            else:
                os.environ["CARGO_PROFILE_RELEASE_INCREMENTAL"] = saved_incremental
        incremental_runs = [line.split("\t")[-1] for line in open(incremental_log).read().splitlines()] if os.path.exists(incremental_log) else []
        checked_cases += 1
        if incremental_code != 0 or len(incremental_runs) < 2 or incremental_runs[0] != "false" or incremental_runs[-1] != "true":
            failures.append(f"基线起的 cargo 不带门禁加的 release 增量覆盖：调用方设 false，基线那一次应当收到 false、变异那一次收到门禁加的 true，"
                            f"实际退 {incremental_code}，每次 cargo 收到的 {incremental_runs}；{incremental_lines}")
        # 变异那一趟连的不是调用方的服务（文件头「判定库与复用」的变异一条）：假 cargo 照收到的连哪一处往服务里写一条、记一行（SERVICE_PROBE_PROGRAM）。基线连的是这一趟自证所在的那一份
        #   （演调用方的服务：socket 与临时服务的标记都是外面给的，写的键落在它上面）；变异那一次连的是它自己的临时服务（socket 与标记相同、不是自证所在的那一份、跑的时候在、跑完不在了），
        #   写的键不落在自证所在的那一份上。弄坏开关 mutated-run-on-real-service 下变异那一次连的是自证所在的那一份
        outer_socket = os.environ.get(client.SOCKET_VARIABLE, "")
        outer_mark = os.environ.get(client.TEMPORARY_SERVICE_VARIABLE, "")
        service_key_prefix = f"prove-red-selftest/mutated-run-crash-verdicts/{os.getpid()}-{os.urandom(4).hex()}/"
        service_variables = {"PROVE_RED_SELFTEST_SERVICE_LOG": service_log, "PROVE_RED_SELFTEST_KEY_PREFIX": service_key_prefix,
                             "PROVE_RED_SELFTEST_CLIENT_DIRECTORY": os.path.dirname(os.path.realpath(ROWS.PRUNE_SERVER_TEMPORARY_SERVICE_SCRIPT))}
        os.environ.update(service_variables)
        try:
            service_code, service_lines = prove(copy, "demo", ["caught"], cargo=fake_cargo)
        finally:
            for name in service_variables:
                os.environ.pop(name, None)
        service_seen = [line.split("\t") for line in open(service_log).read().splitlines()] if os.path.exists(service_log) else []
        baseline_service = [fields for fields in service_seen if fields[0] == "baseline"]
        mutated_service_runs = [fields for fields in service_seen if fields[0] == "mutated"]
        service_written = {key.decode()[len(service_key_prefix):] for key, _value in client.process_client().scan(service_key_prefix.encode())}
        checked_cases += 1
        if (service_code != 0 or not baseline_service or not mutated_service_runs or not outer_socket
                or not all(fields[1] == outer_socket and fields[2] == outer_mark and fields[3] == "1" for fields in baseline_service)
                or not all(fields[1] and fields[1] != outer_socket and fields[1] == fields[2] and fields[3] == "1" and not os.path.exists(fields[1]) for fields in mutated_service_runs)
                or not any(name.startswith("baseline-") for name in service_written) or any(name.startswith("mutated-") for name in service_written)):
            failures.append(f"变异那一趟连的不是调用方的服务：基线连的是自证所在的那一份（socket 与临时服务的标记都是外面给的）、写的键落在它上面；变异那一次连它自己的临时服务"
                            f"（socket 与标记相同、不是自证所在的那一份、跑的时候在、跑完不在了），写的键不落在自证所在的那一份上；"
                            f"实际退 {service_code}，基线 {baseline_service}、变异 {mutated_service_runs}（自证所在的那一份 {outer_socket}、标记 {outer_mark}），"
                            f"自证所在的那一份按前缀有 {sorted(service_written)}；{service_lines}")
        code, lines = prove(copy, "demo", ["caught", "untargeted"], cargo=fake_cargo)
        checked_cases += 1
        if code != 2 or not any("没挑目标" in line for line in lines):
            failures.append(f"不挑目标的变异行应当整次拒绝、退 2，实际 {code} {lines}")
        code, lines = prove(copy, "demo", ["layer"], cargo=fake_cargo)
        checked_cases += 1
        if code != 0 or not any(line.startswith("layer\t抓到\t") for line in lines):
            failures.append(f"--test 目标名带 crash_point_replay 而包不是 验证包：照跑、判抓到、退 0（只按包判），实际 {code} {lines}")
        code, lines = prove(copy, "demo", ["panics"], cargo=fake_cargo)
        checked_cases += 1
        if code != 0 or not any(line.startswith("panics\t抓到\t") for line in lines):
            failures.append(f"#[should_panic] 的用例红了（结果行带「 - should panic」、收尾有 error: test failed）应当判抓到、退 0，实际 {code} {lines}")
        code, lines = prove(copy, "demo", ["broken"], cargo=fake_cargo)
        checked_cases += 1
        if code != 1 or not any(line.startswith("✗ broken\t无效\t") for line in lines):
            failures.append(f"替换之后编不过（error[E…] 与 could not compile）应当判无效、退 1，实际 {code} {lines}")
        code, lines = prove(copy, "demo", ["elsewhere"], cargo=fake_cargo)
        checked_cases += 1
        if code != 1 or not any(line.startswith("✗ elsewhere\t没红\t") for line in lines):
            failures.append(f"编过了、别的测试红而点名的没红（有 error: test failed 也有 test result:）应当判没红、不判无效，实际 {code} {lines}")
        # 链接失败（文件头「判法与次序」⑤）：替换之后编过了、失败在链接（有 error: linking with 与 rust-lld、没有 error[E<码>]）：单列一档「链接失败」、退 1、写出路（删编译目录重跑），不判无效
        code, lines = prove(copy, "demo", ["linkfail"], cargo=fake_cargo)
        checked_cases += 1
        if (code != 1 or not any(line.startswith("✗ linkfail\t链接失败\t") and "删掉这一趟用的编译目录" in line for line in lines)
                or any("\t无效\t" in line for line in lines)):
            failures.append(f"链接失败：替换之后编过了、失败在链接（有 error: linking with 与 rust-lld、没有 error[E<码>]）应当单列「链接失败」、写出路、退 1，不判无效，实际 {code} {lines}")
        code, lines = prove(copy, "redbase", ["redbase"], cargo=fake_cargo)
        checked_cases += 1
        if code != 2 or not any("基线（没改源码）就红" in line for line in lines) or any(line.startswith("redbase\t抓到\t") for line in lines):
            failures.append(f"基线（没改源码）就红：停下、退 2、一条变异都不记抓到，实际 {code} {lines}")
        # 起 cargo 之前判 RocksDB 预编库的取值：另一份副本里放一份依赖 rocksdb 的 验证包清单、进程环境里四个取值是空的，假 cargo 不带 entry 认的标记
        #   ⇒ 整次拒绝、退 2、假 cargo 一次都没起（别的格副本里没有这份清单，entry 报不判）
        rocksdb_copy = selftest_copy(work, "rocksdb")
        write_text(os.path.join(rocksdb_copy, "crates", "singlefs-verification", "Cargo.toml"),
                   '[package]\nname = "singlefs-verification"\nversion = "0.0.0"\n[dependencies]\nrocksdb = { version = "=0.25.0", optional = true }\n')
        saved_values = {name: os.environ.get(name) for name in ("ROCKSDB_LIB_DIR", "ROCKSDB_STATIC", "SNAPPY_LIB_DIR", "SNAPPY_STATIC")}
        os.environ.update({name: "" for name in saved_values})
        try:
            cargo_calls()
            code, lines = prove(rocksdb_copy, "demo", ["caught"], cargo=fake_cargo)
            calls = cargo_calls()
        finally:
            for name, value in saved_values.items():
                if value is None:
                    os.environ.pop(name, None)
                else:
                    os.environ[name] = value
        checked_cases += 1
        if code != 2 or calls or not any("rocksdb-prebuilt entry：" in line for line in lines):
            failures.append(f"副本里的 验证包依赖 rocksdb 而进程环境里没有预编库的取值：应当在起 cargo 之前整次拒绝、退 2、假 cargo 一次都不起，实际退 {code}、假 cargo 起了 {len(calls)} 次 {lines}")
        # 主工作区拿临时目录里一棵同样形态的树（有 crates/mutations.tsv）顶替、经 main_root 交给 prove：自证从哪棵树起（仓里、仓副本、符号链接拼的草稿树）都判得出这一格
        fake_main = selftest_copy(work, "main")
        code, lines = prove(fake_main, "demo", ["caught"], cargo=fake_cargo, main_root=fake_main)
        checked_cases += 1
        if code != 2 or not any("主工作区" in line for line in lines):
            failures.append(f"--copy 指主工作区应当以「主工作区」为由拒绝，实际 {code} {lines}")
        # 副本缺不缺那两格（文件头「判法与次序」①，弄坏开关 copy-closure-unchecked）：闭包照另一棵假主工作区算（它的测试代码 include_str! 读仓根的 litmus/never.litmus），
        #   缺 litmus/ 的副本整次拒绝、退 2、点名 litmus/never.litmus、出路指到 minimal-copy.py、cargo 一次都不起；带上 litmus/ 的副本照跑、判抓到、退 0
        litmus_main = selftest_copy(work, "litmus-main")
        write_text(os.path.join(litmus_main, "crates", "demo", "tests", "litmus_files.rs"), 'const NEVER: &str = include_str!("../../../litmus/never.litmus");\n')
        write_text(os.path.join(litmus_main, "litmus", "never.litmus"), "C never\n")
        short_copy = selftest_copy(work, "litmus-short")
        full_copy = selftest_copy(work, "litmus-full")
        write_text(os.path.join(full_copy, "litmus", "never.litmus"), "C never\n")
        cargo_calls()
        code, lines = prove(short_copy, "demo", ["caught"], cargo=fake_cargo, closure_root=litmus_main)
        calls = cargo_calls()
        checked_cases += 1
        if (code != 2 or calls or not any("litmus/never.litmus" in line for line in lines)
                or not any(line.startswith("→") and "minimal-copy.py" in line for line in lines)):
            failures.append(f"副本缺闭包里的条目（litmus/never.litmus）：整次拒绝、退 2、点名它、出路指到 minimal-copy.py、cargo 一次都不起，"
                            f"实际退 {code}、cargo 起了 {len(calls)} 次 {lines}")
        code, lines = prove(full_copy, "demo", ["caught"], cargo=fake_cargo, closure_root=litmus_main)
        cargo_calls()
        checked_cases += 1
        if code != 0 or not any("\t抓到\t" in line for line in lines):
            failures.append(f"副本带全闭包里的条目（连 litmus/never.litmus）：照跑、判抓到、退 0，实际 {code} {lines}")
        # 验证包的行照跑（能不能起由项目代码闸与统一任务服务判，脚本不看环境变量另判）
        cargo_calls()
        code, lines = prove(copy, "singlefs-verification", ["checker"], cargo=fake_cargo)
        calls = cargo_calls()
        checked_cases += 1
        if code != 0 or not any(line.startswith("checker\t抓到\t") for line in lines) or not any("-p singlefs-verification" in call for call in calls):
            failures.append(f"验证包的行照跑、判抓到、退 0，实际退 {code}、cargo 的参数 {calls} {lines}")
        # 验证包只点名集成测试目标的行那一格（文件头「判法与次序」的「验证包的测试目标不编装置二进制」一条，弄坏开关 no-default-features-unadded）：
        #   点名 --test 的 verification-test 与点名 --lib 的 checker 两行：verification-test 起的每一次 cargo（基线与变异）`--` 之前带 --no-default-features、只带一个，
        #   checker 一次都不带；输出点名补了 verification-test、不点 checker；两行都抓到、退 0
        cargo_calls()
        code, lines = prove(copy, "singlefs-verification", ["verification-test", "checker"], cargo=fake_cargo)
        verification_sides = [words[:words.index("--")] if "--" in words else words for words in (call.split() for call in cargo_calls())]
        target_sides = [words for words in verification_sides if "verification_target" in words]
        library_sides = [words for words in verification_sides if "--lib" in words]
        checked_cases += 1
        if (code != 0 or len(target_sides) < 2 or any(words.count("--no-default-features") != 1 for words in target_sides)
                or len(library_sides) < 2 or any("--no-default-features" in words for words in library_sides)
                or not any(line.startswith("- verification-test：cargo 参数补上 --no-default-features 起") for line in lines)
                or any(line.startswith("- checker：") for line in lines)
                or not any(line.startswith("verification-test\t抓到\t") for line in lines) or not any(line.startswith("checker\t抓到\t") for line in lines)):
            failures.append(f"验证包只点名集成测试目标的行：起的每一次 cargo（基线与变异）`--` 之前带一个 --no-default-features，点名 --lib 的行一次都不带，"
                            f"输出点名补了 verification-test、不点 checker，两行都抓到、退 0，实际退 {code}、点名 --test 的 {target_sides}、点名 --lib 的 {library_sides} {lines}")
        # 重测试目标那一格（文件头「判法与次序」的「重测试目标」一条，弄坏开关 release-unforced）：登记表换成这一格自己造的一份（只登记 demo 的集成测试目标
        #   demo_crash_stream），点名带 gpu 特性的、点名登记目标的与两样都不中的 caught 三行：前两行起的每一次 cargo（基线与变异）`--` 之前带 --release、只带一个，
        #   caught 那两次不带；输出逐行点名补了哪两行、不点 caught；三行都抓到、退 0
        release_registry = os.path.join(work, "release-registry")
        write_text(os.path.join(release_registry, ".claude", "gate.d", "stage-inputs.tsv"),
                   "crash-case:demo-stream\t-\ttest=demo:demo_crash_stream:answer_is_42 count-line=DEMO\t# 自证样本\n")
        cargo_calls()
        code, lines = prove(copy, "demo", ["caught", "release-gpu", "release-crash"], cargo=fake_cargo, registry_root=release_registry)
        release_calls = cargo_calls()
        cargo_sides = [words[:words.index("--")] if "--" in words else words for words in (call.split() for call in release_calls)]
        hit_sides = [words for words in cargo_sides if "gpu" in words or "demo_crash_stream" in words]
        plain_sides = [words for words in cargo_sides if "gpu" not in words and "demo_crash_stream" not in words]
        checked_cases += 1
        if (code != 0 or len(hit_sides) != 4 or any(words.count("--release") != 1 for words in hit_sides)
                or len(plain_sides) != 2 or any("--release" in words for words in plain_sides)
                or not any(line.startswith("- release-gpu：cargo 参数补上 --release 起（gpu 特性 gpu）") for line in lines)
                or not any(line.startswith("- release-crash：cargo 参数补上 --release 起（crash-case 目标 demo:demo_crash_stream") for line in lines)
                or any(line.startswith("- caught：") for line in lines)):
            failures.append(f"命中重测试目标的行（gpu 特性、crash-case 目标）起的每一次 cargo 都带 --release、只带一个，不命中的不带，输出点名补了哪两行，"
                            f"实际退 {code}、cargo 的参数 {release_calls} {lines}")
        # 切表两格：点名的行不成形整次拒绝；切出来的行与 read_table 逐行相同
        code, lines = prove(copy, "demo", ["noexpected"], cargo=fake_cargo)
        checked_cases += 1
        if code != 2 or not any("不成形" in line and "必须红的测试名" in line for line in lines):
            failures.append(f"点名的行不成形（必须红的测试名空着）：整次拒绝、写明哪一段空着、退 2，实际 {code} {lines}")
        ours = sorted(table_rows(copy)[0])
        theirs = sorted({row.name for row in ROWS.read_table(os.path.join(copy, "crates", "mutations.tsv"))[0]})
        checked_cases += 1
        if ours != theirs:
            failures.append(f"切表与 read_table 逐行相同：这里切出 {ours}，read_table 切出 {theirs}")
        # 判法那一格：同一组样本，这里判的档与 verdict_of_run 判的逐行相同
        differing = []
        for description, sample_code, sample_output, expected in SELFTEST_VERDICT_SAMPLES:
            row = ROWS.Row(1, "sample", "crates/demo/src/lib.rs", "a", "b", "-p demo --lib", expected, ())
            ours = verdict_of(row, sample_code, sample_output)[0]
            theirs = ROWS.verdict_of_run(row, sample_code, sample_output, environment=os.environ)[0]
            if ours != theirs:
                differing.append(f"{description}：这里判 {ours}，verdict_of_run 判 {theirs}")
        checked_cases += 1
        if differing:
            failures.append(f"判法与 verdict_of_run 对同一组样本逐行相同（{len(SELFTEST_VERDICT_SAMPLES)} 条），对不上的：{'；'.join(differing)}")
        # 副本锁那一格（文件头「判法与次序」⓪）：证红跑的时候副本根上拿着锁（假 cargo 每次起都试拿一次，记 held / free）、跑完放掉、锁文件不留；
        #   锁被别人占着（这里自己先拿着，演另一趟证红）时整次拒绝、退 2、cargo 一次都不起
        if os.path.exists(lock_log):
            os.unlink(lock_log)
        lock_wrong = []
        code, lines = prove(copy, "demo", ["caught"], cargo=fake_cargo)
        seen = open(lock_log).read().split() if os.path.exists(lock_log) else []
        if code != 0 or not seen or set(seen) != {"held"}:
            lock_wrong.append(f"跑的时候假 cargo 每次都应当看到锁被拿着（held），实际 {seen}、退 {code}")
        if os.path.exists(os.path.join(copy, COPY_LOCK.COPY_LOCK_NAME)):
            lock_wrong.append("跑完锁文件还在")
        if os.path.exists(os.path.join(copy, COPY_LOCK.COPY_OWNER_NAME)):
            lock_wrong.append("跑完持有者记录还在")
        cargo_calls()
        held = COPY_LOCK.hold_copy_lock(copy)
        try:
            code, lines = prove(copy, "demo", ["caught"], cargo=fake_cargo)
            calls = cargo_calls()
        finally:
            if held is not None:
                COPY_LOCK.release_copy_lock(copy, held)
        if held is None or code != 2 or calls or not any("副本上有证红在跑" in line for line in lines):
            lock_wrong.append(f"锁被占着时应当拒绝、退 2、cargo 一次都不起，实际退 {code}、cargo 起了 {len(calls)} 次 {lines}（自证{'拿到了' if held is not None else '没拿到'}锁）")
        checked_cases += 1
        if lock_wrong:
            failures.append("证红跑的时候副本根上拿着副本锁、跑完放掉；锁被占着时拒绝、退 2、cargo 一次都不起：" + "；".join(lock_wrong))
        # 跨机那一格（apply-writer-patch.py 文件头「副本锁」的跨机一段，弄坏开关 APPLY_WRITER_PATCH_BREAK=copy-owner-ignored）：副本根上有别的机器的持有者记录、
        #   刚刷新过（flock 跨机不一定互斥，这里自己不拿 flock 演那种情形）：整次拒绝、退 2、cargo 一次都不起；记录早过过期线就接手照跑，跑完记录不留
        owner_path = os.path.join(copy, COPY_LOCK.COPY_OWNER_NAME)
        remote_wrong = []
        write_text(owner_path, "other-machine-selftest 4242 selftest-token\n")
        cargo_calls()
        code, lines = prove(copy, "demo", ["caught"], cargo=fake_cargo)
        calls = cargo_calls()
        if code != 2 or calls or not any("上已有一趟证红在跑" in line for line in lines):   # 拒绝句是「副本 <路径> 上已有一趟证红在跑」，路径夹在中间
            remote_wrong.append(f"别的机器上的证红占着（持有者记录刚刷新过）时应当拒绝、退 2、cargo 一次都不起，实际退 {code}、cargo 起了 {len(calls)} 次 {lines}")
        stale_at = time.time() - 3 * COPY_LOCK.COPY_OWNER_STALE_SECONDS
        os.utime(owner_path, (stale_at, stale_at))
        code, lines = prove(copy, "demo", ["caught"], cargo=fake_cargo)
        cargo_calls()
        if code != 0 or os.path.exists(owner_path):
            remote_wrong.append(f"别的机器上的持有者记录早过过期线时应当接手照跑、退 0、跑完记录不留，实际退 {code}、记录{'还在' if os.path.exists(owner_path) else '已删'} {lines}")
        if os.path.exists(owner_path):
            os.unlink(owner_path)
        checked_cases += 1
        if remote_wrong:
            failures.append("别的机器上的证红占着副本时拒绝、过期的接手：" + "；".join(remote_wrong))
        # 日志目录那一格（文件头「判法与次序」，弄坏开关 shared-log-directory）：同一份副本上先后起两趟（假 cargo 在输出里打 stamp=<这一趟的标记>），
        #   每趟的基线日志写在自己的目录里：两份基线日志各带自己那趟的标记，后一趟没有把前一趟的盖掉
        log_wrong = []
        logs_root = os.environ["PROVE_RED_LOG_DIRECTORY"]
        shutil.rmtree(logs_root, ignore_errors=True)
        run_stamps = ("stamp-sparrow", "stamp-heron")
        for run_stamp in run_stamps:
            os.environ["PROVE_RED_SELFTEST_STAMP"] = run_stamp
            code, lines = prove(copy, "demo", ["caught"], cargo=fake_cargo)
            if code != 0:
                log_wrong.append(f"标记 {run_stamp} 那一趟应当抓到、退 0，实际 {code} {lines}")
        os.environ.pop("PROVE_RED_SELFTEST_STAMP", None)
        cargo_calls()
        baseline_stamps = sorted(re.search(r"stamp=(\S+)", open(os.path.join(directory, "baseline.log"), encoding="utf-8").read()).group(1)
                                 for directory, _subdirectories, names in os.walk(logs_root) if "baseline.log" in names)
        checked_cases += 1
        if baseline_stamps != sorted(run_stamps):
            log_wrong.append(f"两趟各写自己的基线日志，日志目录下读回的标记应当是 {sorted(run_stamps)}，实际 {baseline_stamps}")
        if log_wrong:
            failures.append("同一份副本上先后起两趟，每趟读的是自己的基线日志：" + "；".join(log_wrong))
        # 被杀那一格（文件头「判法与次序」的「还原日志」一条，弄坏开关 no-restore-journal）：另一份副本上，假 cargo 在源码是改坏的样子时记下自己的进程号、睡着不退；
        #   这一格 fork 出的子进程在那份副本上跑 prove，看到记号就对这个子进程发 SIGKILL（finally 不跑，源码留在改坏的样子，与撞内存上限被杀同形），
        #   再停掉睡着的那个假 cargo（两个都是这一格自己起的进程号）。同一份副本上再起一趟：源码要被写回、报写回的那一行、基线照常跑
        #   （假 cargo 起了带 --show-output 的那一次）、不以命中 0 次退 2、还原日志删掉
        killed_copy = selftest_copy(work, "killed")
        killed_source = os.path.join(killed_copy, "crates", "demo", "src", "lib.rs")
        killed_marker = os.path.join(work, "killed-cargo-pid")
        killed_cargo = os.path.join(work, "bin", "killed-cargo")
        write_text(killed_cargo,
                   "#!/usr/bin/env bash\n"
                   f"if grep -q '{{ 42 }}' crates/demo/src/lib.rs; then exec {shlex.quote(fake_cargo)} \"$@\"; fi\n"
                   f"echo \"$$\" > {shlex.quote(killed_marker + '.tmp')} && mv {shlex.quote(killed_marker + '.tmp')} {shlex.quote(killed_marker)}\n"
                   "exec sleep 120\n", executable=True)
        killed_wrong = []
        child = os.fork()
        if child == 0:
            try:
                prove(killed_copy, "demo", ["caught"], cargo=killed_cargo)
            finally:
                os._exit(0)
        deadline = time.monotonic() + 60
        finished_alone = False
        while not os.path.exists(killed_marker) and time.monotonic() < deadline:
            if os.waitpid(child, os.WNOHANG)[0]:
                finished_alone = True
                break
            time.sleep(0.05)
        if not finished_alone:
            os.kill(child, signal.SIGKILL)
            os.waitpid(child, 0)
        sleeper = open(killed_marker).read().strip() if os.path.exists(killed_marker) else ""
        if sleeper.isdigit():
            try:
                os.kill(int(sleeper), signal.SIGKILL)
            except ProcessLookupError:
                pass
            if not process_gone(int(sleeper)):
                killed_wrong.append(f"睡着的假 cargo（进程 {sleeper}）停不掉")
        if finished_alone or not sleeper.isdigit() or open(killed_source).read() == "fn answer() -> u32 { 42 }\n":
            killed_wrong.append(f"没造出被杀的样子（子进程自己跑完了：{finished_alone}，假 cargo 记的进程号 {sleeper!r}，源码此刻 {open(killed_source).read()!r}）")
        else:
            cargo_calls()
            code, lines = prove(killed_copy, "demo", ["caught"], cargo=fake_cargo)
            calls = cargo_calls()
            if open(killed_source).read() != "fn answer() -> u32 { 42 }\n":
                killed_wrong.append(f"源码没写回，此刻 {open(killed_source).read()!r}")
            if not any(line.startswith("- 上一趟没还原的变异已写回：caught") for line in lines):
                killed_wrong.append("没报写回的那一行")
            if code == 2 or not any("--show-output" in call for call in calls):
                killed_wrong.append(f"基线没照常跑（退 {code}、cargo 的参数 {calls}）")
            if os.path.exists(os.path.join(killed_copy, RESTORE_JOURNAL_NAME)):
                killed_wrong.append("跑完还原日志还在")
            if killed_wrong:
                killed_wrong.append(f"输出 {lines}")
        checked_cases += 1
        if killed_wrong:
            failures.append("被杀之后同一份副本上下一趟起来：先照还原日志把源码写回、报写回的那一行、基线照常跑、还原日志删掉：" + "；".join(killed_wrong))
        # 留着替换文那一格（文件头「判法与次序」的「先核原文」一条，弄坏开关 old-text-checked-late）：副本里 demo 的源码已是 caught 那一行的替换文（41），
        #   没有还原日志可照：基线之前以原文命中 0 次整次拒绝、出路点到还原日志与干净副本、退 2、假 cargo 一次都不起
        leftover_copy = selftest_copy(work, "leftover")
        write_text(os.path.join(leftover_copy, "crates", "demo", "src", "lib.rs"), "fn answer() -> u32 { 41 }\n")
        cargo_calls()
        code, lines = prove(leftover_copy, "demo", ["caught"], cargo=fake_cargo)
        calls = cargo_calls()
        checked_cases += 1
        if code != 2 or calls or not any("命中 0 次" in line and "还原日志" in line and "干净副本" in line for line in lines):
            failures.append(f"副本里留着一条替换文而没有还原日志：基线之前以原文命中 0 次整次拒绝、退 2、cargo 一次都不起，实际退 {code}、cargo 起了 {len(calls)} 次 {lines}")
        # 没重编那一格（文件头「判法与次序」⑤）：调用方设的 CARGO_TARGET_DIR 是一份共用编译目录，里面 demo 的产物（没改过的源码）比施加变异后的源码新
        #   （另一份副本后编的那种，修改时刻取明天）：假 cargo 不重编、测试跑在旧产物上全绿；要判没重编、退 1，不判没红
        shared_target = os.path.join(work, "shared-target")
        stale_artifact = os.path.join(shared_target, "fake-build", "demo.rs")
        write_text(stale_artifact, "fn answer() -> u32 { 42 }\n")
        # 上一次在这份编译目录里编这个副本时留下的 dep-info：lib.rs 是编译读过的文件
        write_text(os.path.join(shared_target, "debug", "deps", "demo.d"), f"{stale_artifact}: {os.path.realpath(copy)}/crates/demo/src/lib.rs\n")
        tomorrow = time.time() + 86400
        os.utime(stale_artifact, (tomorrow, tomorrow))
        saved_target = os.environ.get("CARGO_TARGET_DIR")
        os.environ["CARGO_TARGET_DIR"] = shared_target
        try:
            code, lines = prove(copy, "demo", ["caught"], cargo=fake_cargo)
        finally:
            os.environ.pop("CARGO_TARGET_DIR", None)
            if saved_target is not None:
                os.environ["CARGO_TARGET_DIR"] = saved_target
        checked_cases += 1
        if code != 1 or not any(line.startswith("✗ caught\t没重编\t") and "Compiling demo" in line for line in lines):
            failures.append(f"编译目录里的产物比施加变异后的源码新（共用编译目录）：那一条判没重编、退 1，不判没红，实际退 {code} {lines}")
        # 编译目录归这一趟三格（文件头「判法与次序」的「编译目录归这一趟」）：每一趟新建、跑完删、被打断删
        trip_count, trip_failures = trip_build_directory_cells(work, fake_cargo, target_log)
        checked_cases += trip_count
        failures += trip_failures
        checked_cases += 1
        path_included_failure = path_included_package_failure(work)
        if path_included_failure:
            failures.append(path_included_failure)
        # 判定库那四格（文件头「判定库与复用」）：复用三问、复用哨兵、键相同的都复用、记红撤绿
        library_count, library_failures = library_cells(work)
        checked_cases += library_count
        failures += library_failures
        # 开 GPU 补 gpu 特性那一格（文件头「判法与次序」的「开 GPU 补 gpu 特性」一条）
        route_count, route_failures = gpu_route_cells(work)
        checked_cases += route_count
        failures += route_failures
        # 并行那十二格（文件头「判法与次序」的「并行」一条）
        parallel_count, parallel_failures = parallel_cells(work)
        checked_cases += parallel_count
        failures += parallel_failures
        # 不领卡那一格（文件头「不领卡」）
        no_card_lease_count, no_card_lease_failures = no_card_lease_cells(work, fake_cargo)
        checked_cases += no_card_lease_count
        failures += no_card_lease_failures
        # 编译目录三格：开头那几行不碰位置参数（只给 --selftest 也走得过去）；默认值在解析完 --copy 之后定，次序不限；调用方设了的不覆盖
        caller_target = os.environ.pop("CARGO_TARGET_DIR", None)
        try:
            probe = subprocess.run(["bash", os.path.join(SCRIPT_DIRECTORY, "prove-red.sh"), "--selftest"], capture_output=True, text=True,
                                   env=dict(os.environ, PROVE_RED_SELFTEST_PROLOGUE="1"))
            checked_cases += 1
            if probe.returncode != 0 or "PROVE_RED_PROLOGUE CARGO_TARGET_DIR=<未设>" not in probe.stdout.splitlines():
                failures.append(f"不设 CARGO_TARGET_DIR、只给 --selftest：开头那几行应当走得过去、不替调用方设编译目录（子进程打 CARGO_TARGET_DIR=<未设>、退 0），"
                                f"实际退 {probe.returncode}，输出 {(probe.stdout + probe.stderr).strip()!r}")
            # 编译并行度：没设 CARGO_BUILD_JOBS 取登记文件第一条非注释行的数（SINGLEFS_THREAD_CAP 也没设），设了的照它
            with open(os.path.join(MAIN_ROOT, ".claude", "heavy-thread-default"), encoding="utf-8") as registry:
                registered_jobs = next((line.split()[0] for line in registry if line.strip() and not line.lstrip().startswith("#")), "<登记里没有数>")
            for preset, want in ((None, registered_jobs), ("3", "3")):
                environment = {name: value for name, value in os.environ.items() if name not in ("CARGO_BUILD_JOBS", "SINGLEFS_THREAD_CAP")}
                environment["PROVE_RED_SELFTEST_PROLOGUE"] = "1"
                if preset is not None:
                    environment["CARGO_BUILD_JOBS"] = preset
                jobs_probe = subprocess.run(["bash", os.path.join(SCRIPT_DIRECTORY, "prove-red.sh"), "--selftest"], capture_output=True, text=True, env=environment)
                checked_cases += 1
                if jobs_probe.returncode != 0 or f"PROVE_RED_PROLOGUE CARGO_BUILD_JOBS={want}" not in jobs_probe.stdout.splitlines():
                    failures.append(f"{'不设' if preset is None else '调用方设了'} CARGO_BUILD_JOBS：开头那几行应当{'取登记的重活默认并行度 ' if preset is None else '照它的 '}{want}（子进程打 CARGO_BUILD_JOBS={want}），"
                                    f"实际退 {jobs_probe.returncode}，输出 {(jobs_probe.stdout + jobs_probe.stderr).strip()!r}")
            if os.path.exists(target_log):
                os.unlink(target_log)
            os.environ["CARGO_TARGET_DIR"] = os.path.join(work, "caller-target")
            code, lines = prove_from_arguments(["--copy", copy, "demo", "caught"], cargo=fake_cargo)
            seen = open(target_log).read().splitlines() if os.path.exists(target_log) else []
            checked_cases += 1
            if code != 0 or not seen or set(seen) != {os.path.join(work, "caller-target")}:
                failures.append(f"调用方设了 CARGO_TARGET_DIR：应当照它的、不覆盖，实际退 {code}，cargo 看到的 {seen}")
        finally:
            os.environ.pop("CARGO_TARGET_DIR", None)
            if caller_target is not None:
                os.environ["CARGO_TARGET_DIR"] = caller_target
        for failure in failures:
            print(f"  ✗ 自检：{failure}")  # gate-lint:detail
        if failures:
            print("  → 看 prove() / table_rows() / refusal_for() / verdict_of() / judged_line() 的判法（判结局在 crates-mutation-rows.py 的 verdict_of_run，"
                  "还原日志与先核原文看 restore_unrestored_mutation() / write_restore_journal() / old_text_misses()，"
                  "补 --no-default-features 看 default_features_off_rows()（判法在 crates-mutation-rows.py 的 default_features_off_row），补 --release 看 release_forced_rows()（判法在 lib_release_only_tests.py），补 gpu 特性看 library_plan() 调 gpu_route_rows 那一段（判法在 crates-mutation-rows.py），"
                  "结果行在 lib_libtest_result_lines.py），编译目录那三格看 prove_from_arguments() / cargo_environment() 与开头 python3 之前那几行，编译目录归这一趟那三格看 prove() 里 new_trip_build_directory() / remove_trip_build_directory() 与 stopped_by_signal()；链接失败看 judged_line()（判法在 crates-mutation-rows.py 的 verdict_of_run）；"
                  "判定库那几格看 library_plan() / planned_sentinel() / record_verdict() 与 prove_on_restored_sources() 里复用、交哨兵那一段（行键与复用判据在 crates-mutation-rows.py）；"
                  "并行那几格看 planned_lanes() / prepared_lanes() / run_in_lanes() / lanes_collected_problem() 与 prove_on_restored_sources() 里 judged_on_lanes()；"
                  "不领卡那一格看 run_cargo()；"
                  "PROVE_RED_BREAK 或 LIBTEST_RESULT_LINES_BREAK 设着的话这里本来就该红")
            return 1
        print(f"  ✓ prove-red 自检通过（判法样本 {len(SELFTEST_VERDICT_SAMPLES)} 条）：会红的抓到并写回源码，should_panic 红了抓到，编不过判无效，别的测试红判没红，基线红停下退 2，"
              f"不挑目标的整次拒绝，目标名带 crash_point_replay 而包不是 验证包的照跑，主工作区拒绝，缺闭包里条目的副本整次拒绝、拷全的照跑，"
              f"验证包的行照跑，验证包只点名集成测试目标的行补 --no-default-features 起而点名 --lib 的不补，命中重测试目标的行补 --release 起而不命中的不补，开 GPU 时点名测试走崩溃判段的行补 gpu 特性起，不成形的行整次拒绝，切表与 read_table 相同，"
              f"判法与 verdict_of_run 对 {len(SELFTEST_VERDICT_SAMPLES)} 条样本逐行相同，跑的时候副本根上拿着副本锁、跑完放掉、锁被占着时拒绝，"
              f"施加之后被杀的下一趟先照还原日志写回源码再跑基线，留着替换文而没有还原日志的基线之前就拒绝，"
              f"每一趟自己建编译目录、不沿用副本里别的命令留下的 target、跑完删（成功、红、被 TERM 停都删），替换之后失败在链接的单列链接失败、不判无效，"
              f"共用编译目录里的产物比施加变异后的源码新时判没重编，被改文件经 #[path] 编进点名的包时认那个包的 Compiling 行，"
              f"不设 CARGO_TARGET_DIR 只给 --selftest 走得过开头，调用方设了的不覆盖，"
              f"行键算不出的照跑、报拒算，判定按行键进判定库：复用三问都过、复用哨兵抽一条强制现判一条、键相同的都复用时 cargo 一次都不起、强制现判判成没红的记红撤绿，"
              f"互不撞文件的行并行：判定与串行逐条相同、输出按点名次序、每路各一份工作副本与编译目录、单路红不吃掉、收回数核对、一路基线红一条变异都不施加，"
              f"不领卡：带不带 gpu 特性的行都照原样起、不另编、环境里给的卡串原样交、不藏卡（查了 {checked_cases} 种）")
        return 0
    finally:
        CLOSURE_ROOT_FOR_SELFTEST.clear()
        LIBRARY_ROOT_FOR_SELFTEST.clear()
        for name, value in {**saved_caches, **saved_card_variables}.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
        shutil.rmtree(work, ignore_errors=True)
        os.environ.pop("CARGO_TARGET_DIR", None)
        if inherited_target is not None:
            os.environ["CARGO_TARGET_DIR"] = inherited_target
