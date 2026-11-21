#!/usr/bin/env bash
# 合入后验证 merge-and-verify.sh 的自证：selftest（入口在 --selftest 时调它）。
# 被 source，只定义函数、不改 shell 选项；读写入口脚本的全局变量（同一个 shell）。

selftest() {
  local scratch fake_cargo_dir failures=0 checked=0 out rc expected_lints expected_lint_count check_case check_case_text combination clippy_expected clippy_received clippy_unreported
  local baseline_calls flagged_calls flagged_clippy_count
  local -a expected_combinations=()
  local service_problem
  # 逐项与复用那几格经 admission.py 写判定库（文件头 --selftest 那一行）：不是临时 prune-server 就一格都不跑，不写真服务
  if ! service_problem="$(python3 -c 'import sys; sys.path.insert(0, sys.argv[1]); import prune_server_client; print(prune_server_client.selftest_service_problem())' "$(dirname "${BASH_SOURCE[0]}")/../prune-server" 2>&1)" || [[ -n "$service_problem" ]]; then
    echo "  ✗ merge-and-verify.sh 自证没跑：${service_problem:-读不了 scripts/prune-server/prune_server_client.py}"
    echo "     → 怎么办：照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起这一份自证（带 --server-binary 就不编）"
    return 2
  fi
  unset MERGE_AND_VERIFY_KEEP_COPY   # 留现场那一格自己设；调用方设了不能带进别的格
  unset TASK_SERVER_TASK TASK_SERVER_ATTEMPT   # 上一次尝试那几格自己设；经服务起这一份自证时环境里有，不能带进别的格
  unset SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO   # 缩崩溃域的格自己设；调用方环境里有的不能带进别的格
  # 测试那一步：别的格一条 cargo test（它们核假 cargo 收到的那一条），只有「按测试目标并行起」那一格不设它（文件头 test 一段）
  export MERGE_AND_VERIFY_TEST_TARGETS=serial
  scratch="$(mktemp -d)"; fake_cargo_dir="$scratch/bin"; mkdir -p "$fake_cargo_dir" "$scratch/patch-a" "$scratch/patch-b"
  # 各格没设 CARGO_TARGET_DIR 时编译目录建在 TMPDIR 下（文件头「这一趟的编译目录」）：TMPDIR 指进 scratch，留下的随 scratch 一起删
  export TMPDIR="$scratch/tmp"; mkdir -p "$TMPDIR"
  # 槽位（文件头「槽位」）：别的格照一趟一份那一种跑（这份登记 slots=0）；slots 是 0 也照样读工作根与这一台的编译根、清多出来的槽与槽已不在的编译目录，
  #   这里把两样连同登记一起指进 scratch 里空着的那一份（三样要一起给，文件头「槽位」），不碰真的工作根与编译根；「槽位：」那几格各自换成有槽的登记、各给一个工作根与编译根
  #   （不给自证的登记与编译根那几格不给这两样，另把数据根与多机配置换成这一格假的，见那几格）
  printf 'slots=0\nslot-target-limit-gib=0\n' > "$scratch/slot-table-none.conf"
  export MERGE_AND_VERIFY_SLOT_TABLE="$scratch/slot-table-none.conf" MERGE_AND_VERIFY_SLOT_BUILD_ROOT="$scratch/slot-build-none" SINGLEFS_WORK_ROOT="$scratch/work-root-none"
  # 这一趟建的编译目录在哪：取它开跑时打的那一行（「这一趟的编译目录：<路径>（这里建的，收尾删）」）
  run_target_of() { sed -n 's/^  这一趟的编译目录：\(.*\)（这里建的，收尾删）$/\1/p' "$1" 2>/dev/null | head -1; }
  # patch-a 改的是 demo 里的一份数据文件（不是 .rs：重编核对不核它，测试照挑 demo）
  printf -- '--- a/crates/demo/src/table.txt\n+++ b/crates/demo/src/table.txt\n@@ -1 +1 @@\n-1\n+2\n' > "$scratch/patch-a/crates.patch"
  # 验证包那一步（文件头「验证包那一步」）：各格都换上假的那一道，它照真的那一道的写法答 --list-items、打按条复用与崩溃状态块那几行
  cat > "$scratch/fake-verification-step.sh" <<'FAKE'
#!/usr/bin/env bash
# 假的 门禁 crash-point-replay：--list-items 照真的那一道的写法列登记的崩溃枚举用例（测试目标 setting；FAKE_VERIFICATION_CASES，没设是 demo-case）与挂特性的两项（库、gpu_pair.rs），退 FAKE_VERIFICATION_LIST_EXIT（没设是 0）；
#   别的起法照真的那一道给点名的每条崩溃枚举用例打一行判绿（FAKE_VERIFICATION_EXIT 不是 0 时判红），再打一行按条复用、一行崩溃状态块与一行逐格，FAKE_VERIFICATION_EXIT 不是 0 时另打一条 FAILED 与点名的每项一行 exit=1 的 ITEM_DONE，退它（没设是 0）。
#   每次的参数记进 FAKE_VERIFICATION_CALLS，不是 --list-items 的那几次把展开深度的两个变量（放量与崩溃枚举用例读的、对拍读的）记进 FAKE_VERIFICATION_ENVIRONMENT（设了那个文件的话）：
#   两个同值记那个值（都没设记 unset），不同值记「amplification=<值> judge=<值>」。
[[ -z "${FAKE_VERIFICATION_CALLS:-}" ]] || printf '%s\n' "$*" >> "$FAKE_VERIFICATION_CALLS"
# FAKE_VERIFICATION_CARDS 设了：每次另记一行「<参数><TAB>cards=<收到的 GPU_CARDS_BY_PRIORITY>」（没有的写 none；「卡：」那两格看交给门禁的卡是不是照原样）
[[ -z "${FAKE_VERIFICATION_CARDS:-}" ]] || printf '%s\tcards=%s\n' "$*" "${GPU_CARDS_BY_PRIORITY:-none}" >> "$FAKE_VERIFICATION_CARDS"
# 照真的那一道写它的产品 GPU 包那一格守哪个包：合入后验证照这一行现读（文件头「验证包那一步」）
PRODUCT_GPU_PACKAGE_NAME="singlefs-background-gpu-tests"
if [[ "$1" == --list-items ]]; then
  for listed_case in ${FAKE_VERIFICATION_CASES:-demo-case}; do printf 'crash-case:%s\tsinglefs-verification:setting:case_function\n' "$listed_case"; done
  printf 'feature-gated:lib\t--lib\n'
  printf 'feature-gated:gpu_pair.rs\twhole-file gpu\nfeature-gated:split_pair/main.rs\twhole-file gpu\n'
  exit "${FAKE_VERIFICATION_LIST_EXIT:-0}"
fi
if [[ -n "${FAKE_VERIFICATION_ENVIRONMENT:-}" ]]; then
  amplification_value="${SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO:-unset}"; judge_value="${SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO:-unset}"
  # 两个变量同值才记那个值（没设记 unset），不同值记「amplification=… judge=…」：各格比的是一个值，只交了一个的就对不上
  if [[ "$amplification_value" == "$judge_value" ]]; then printf '%s\n' "$amplification_value" >> "$FAKE_VERIFICATION_ENVIRONMENT"
  else printf 'amplification=%s judge=%s\n' "$amplification_value" "$judge_value" >> "$FAKE_VERIFICATION_ENVIRONMENT"; fi
fi
[[ -z "${FAKE_VERIFICATION_SLEEP:-}" ]] || sleep "$FAKE_VERIFICATION_SLEEP"
previous=""
for word in "$@"; do
  if [[ "$previous" == --item && "$word" == crash-case:* ]]; then
    if [[ "${FAKE_VERIFICATION_EXIT:-0}" != 0 ]]; then echo "  ✗ $word 判红：cargo test 退非 0（假的那一道）"; else echo "  ✓ $word 判绿（假的那一道）"; fi
  fi
  previous="$word"
done
echo "  · 按条复用：验证包 2 条编得进的测试函数，现判 0、复用 2、拒算 0；这一组特性下不编 0 条"
echo "  ✓ 崩溃状态块：现判 3、复用 5、拒算 0、本次未跑 0"
echo "  · 逐格：crash-case-replay：绿；feature-gated-tests：无对象可判"
if [[ "${FAKE_VERIFICATION_EXIT:-0}" != 0 ]]; then
  echo "test case_function ... FAILED"
  previous=""
  # FAKE_VERIFICATION_RED_ITEM 设了：exit=1 的 ITEM_DONE 只打它一行（点名的项里没有它：红的项认不出那一格用）
  if [[ -n "${FAKE_VERIFICATION_RED_ITEM:-}" ]]; then echo "ITEM_DONE item=$FAKE_VERIFICATION_RED_ITEM seconds=1 judged=1 reused=0 exit=1"
  else for word in "$@"; do [[ "$previous" == --item ]] && echo "ITEM_DONE item=$word seconds=1 judged=1 reused=0 exit=1"; previous="$word"; done; fi
fi
exit "${FAKE_VERIFICATION_EXIT:-0}"
FAKE
  export MERGE_AND_VERIFY_VERIFICATION_STAGE="$scratch/fake-verification-step.sh"
  unset MERGE_AND_VERIFY_VERIFICATION_ROOT FAKE_VERIFICATION_CALLS FAKE_VERIFICATION_EXIT FAKE_VERIFICATION_ENVIRONMENT FAKE_VERIFICATION_LIST_EXIT FAKE_VERIFICATION_SLEEP FAKE_CARGO_ENVIRONMENT FAKE_CARGO_TEST_SLEEP MERGE_AND_VERIFY_PROGRESS_INTERVAL MERGE_AND_VERIFY_SELFTEST_FORKED_FINISH
  unset FAKE_VERIFICATION_CASES FAKE_VERIFICATION_CARDS FAKE_VERIFICATION_RED_ITEM FAKE_CARGO_DEBUGINFO FAKE_CARGO_BUILD_COMPILING_DROPPED
  # build 那一条关掉调试信息那一格核 test 与 clippy 没收到 CARGO_PROFILE_DEV_DEBUG：调用方环境里有的不带进各格
  unset CARGO_PROFILE_DEV_DEBUG
  # 卡（文件头「显卡」）：调用方环境里的卡不带进各格（「卡：」那两格自己设）
  unset GPU_CARDS_BY_PRIORITY
  # 崩溃域上限（文件头「崩溃域上限」）：样本树里没有那份库，换成这份脚本所在仓里那一份（判的根仍是样本树）；运行记录放临时家目录。
  #   write_case_record <家目录> <这一趟的标签> <合表项> <展开深度或 full> <用例名> <态数>：造入口每趟写的 .scope（一趟一个目录）。
  #   默认家目录里样本树登记的几条用例都有整个域的记录、各 100 态，不超上限；要别的局面的格在起之前换 SINGLEFS_CRASH_AMPLIFICATION_HOME
  export MERGE_AND_VERIFY_CRASH_DOMAIN_LIBRARY="$here/../crash/lib_crash_point_replay_items.py"
  export SINGLEFS_CRASH_AMPLIFICATION_HOME="$scratch/crash-home"
  write_case_record() {
    mkdir -p "$1/runs/crash-point-replay-$2"
    printf 'expand_up_to=%s\ncase_states=%s:%s\n' "$4" "$5" "$6" > "$1/runs/crash-point-replay-$2/enumeration.$3.$5.log.scope"
  }
  write_case_record "$scratch/crash-home" whole-domain demo full demo-case 100
  write_case_record "$scratch/crash-home" whole-domain other full other-case 100
  write_case_record "$scratch/crash-home" whole-domain third full third-case 100
  write_case_record "$scratch/crash-home" whole-domain edited full edited-case 100
  # write_crash_domain_tables <树> <用例名…>：样本树里合表、stage-inputs.tsv（每条用例一项，项名是用例名去掉 -case，都在测试目标 setting 里）与引擎展开上限常量（30）
  write_crash_domain_tables() {
    local domain_case
    mkdir -p "$1/.claude/gate.d" "$1/crates/singlefs-verification/src"
    printf 'pub const EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION: usize = 30;\n' > "$1/crates/singlefs-verification/src/crash_amplification.rs"
    : > "$1/.claude/gate.d/crash-point-replay-items.tsv"; : > "$1/.claude/gate.d/stage-inputs.tsv"
    for domain_case in "${@:2}"; do
      printf '%s\tcrash-case:%s\t-\n' "${domain_case%-case}" "$domain_case" >> "$1/.claude/gate.d/crash-point-replay-items.tsv"
      printf 'crash-case:%s\t-\ttest=singlefs-verification:setting:case_function\n' "$domain_case" >> "$1/.claude/gate.d/stage-inputs.tsv"
    done
  }
  # 只有变异行、没有 crates.patch 的补丁目录（--skip-apply 时不核打没打）：变异行点名 demo 的库目标，--skip-apply 的各格拿它挑测试目标（文件头「挑测试目标」）
  mkdir -p "$scratch/patch-demo-lib"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'demo 的库（自证样本）' crates/demo/src/lib.rs 'pub fn f() {}' 'pub fn f() { }' '-p demo --lib' f > "$scratch/patch-demo-lib/mutations-append.tsv"
  # patch-targets 改 demo 的库里一份数据文件与测试目标 alpha 的根文件（「按测试目标并行起」那一格用）
  mkdir -p "$scratch/patch-targets"
  printf -- '--- a/crates/demo/src/table.txt\n+++ b/crates/demo/src/table.txt\n@@ -1 +1 @@\n-1\n+2\n--- a/crates/demo/tests/alpha.rs\n+++ b/crates/demo/tests/alpha.rs\n@@ -1 +1 @@\n-1\n+2\n' \
    > "$scratch/patch-targets/crates.patch"
  # 自证自己经 task-server 派到第二台时环境里带着第二台那一趟的几样（scripts/hosts/multi-host-run.sh 文件头 ⑤「带到第二台的环境」：本台的多机配置、起活那一方的回读握手、
  #   这一趟的本地目录）：各格照本机跑，只有「跑在第二台上」那一格自己设那几样；多机配置不带进来（逐项与复用那几格起 admission.py，
  #   本台那一份的路径不漏进它们起的子进程）。两台对称，没有认台的角色变量
  unset SINGLEFS_MULTI_HOST_CONFIG GPU_QUIESCE_ORIGIN_READ_BACK MULTI_HOST_RUN_PEER_SCRATCH
  cat > "$scratch/fake-apply.py" <<'FAKE'
import sys
if "--root" in sys.argv:
    print("root=" + sys.argv[sys.argv.index("--root") + 1])
sys.exit(2 if sys.argv[1].endswith("patch-b") else 0)
FAKE
  # 假 cargo 照真 cargo 在标准错误打这一趟编的包的行：test 与 build 打 -p 点名的包的「Compiling <包名> v…」（不带 -p 的 build 打工作区每个成员的），
  #   clippy 不带 -p、打工作区每个成员的「Checking <包名> v…」：快照里每个文件都比编译目录新，真 cargo 照编、照打；重编核对（lib_rebuilt_packages.py check）读的就是这几行
  cat > "$fake_cargo_dir/cargo" <<'FAKE'
#!/usr/bin/env bash
[[ -z "${FAKE_CARGO_CALLS:-}" ]] || printf '%s\n' "$*" >> "$FAKE_CARGO_CALLS"
# FAKE_CARGO_DEBUGINFO 设了：每条另记一行「<子命令><TAB><收到的 CARGO_PROFILE_DEV_DEBUG，没有写 unset>」（build 那一条关掉调试信息那一格看）
[[ -z "${FAKE_CARGO_DEBUGINFO:-}" ]] || printf '%s\t%s\n' "$1" "${CARGO_PROFILE_DEV_DEBUG-unset}" >> "$FAKE_CARGO_DEBUGINFO"
# FAKE_CARGO_ENVIRONMENT 设了：每条 cargo test 另记一行「<展开深度的值><TAB><参数>」；两个展开深度变量同值记那个值（都没设记 unset），不同值记「amplification=<值> judge=<值>」
if [[ -n "${FAKE_CARGO_ENVIRONMENT:-}" && "$1" == test ]]; then
  amplification_value="${SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO:-unset}"; judge_value="${SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO:-unset}"
  if [[ "$amplification_value" == "$judge_value" ]]; then received_value="$amplification_value"; else received_value="amplification=$amplification_value judge=$judge_value"; fi
  printf '%s\t%s\n' "$received_value" "$*" >> "$FAKE_CARGO_ENVIRONMENT"
fi
case "$1" in
  metadata) cat "${FAKE_CARGO_METADATA:?假 cargo 没给 FAKE_CARGO_METADATA}"; exit 0 ;;
  test) previous=""
        for word in "$@"; do [[ "$previous" == -p ]] && echo "   Compiling $word v0.1.0 (/selftest/$word)" >&2; previous="$word"; done
        if [[ " $* " == *" --no-run "* ]]; then cat "${FAKE_CARGO_ARTIFACTS:?假 cargo 没给 FAKE_CARGO_ARTIFACTS}"; exit 0; fi
        [[ "${FAKE_CARGO_MODE:-}" == "empty" ]] && exit 0
        [[ -z "${FAKE_CARGO_TEST_SLEEP:-}" ]] || sleep "$FAKE_CARGO_TEST_SLEEP"
        printf 'running 2 tests\ntest tests::adds ... ok\ntest tests::subtracts_in_%s ... FAILED\n\ntest result: FAILED. 1 passed; 1 failed\n' "${FAKE_CARGO_MODE:-x}"; [[ "${FAKE_CARGO_MODE:-}" == "green" ]] && exit 0 || exit 101 ;;
  fmt) exit 0 ;;
  build) # 带 -p 的打点名的那几个包；不带 -p 的打工作区每个成员（当前目录是快照根：成员是 crates/ 下各个 Cargo.toml 里 [package] 的 name）。
         # FAKE_CARGO_BUILD_COMPILING_DROPPED 设了一行都不打（「重编核对读 clippy 的 Checking 行」那一格单看 clippy 那一趟的行）
         [[ -z "${FAKE_CARGO_BUILD_COMPILING_DROPPED:-}" ]] || exit 0
         previous=""; named=0
         for word in "$@"; do [[ "$previous" == -p ]] && { echo "   Compiling $word v0.1.0 (/selftest/$word)" >&2; named=1; }; previous="$word"; done
         ((named)) && exit 0
         for manifest in crates/*/Cargo.toml; do
           name="$(sed -n 's/^name = "\(.*\)"$/\1/p' "$manifest" 2>/dev/null | head -n 1)"; [[ -z "$name" ]] || echo "   Compiling $name v0.1.0 (/selftest/$name)" >&2
         done
         exit 0 ;;
  clippy) # 先照真 cargo 给工作区每个成员打一行「Checking <包名> v…」（当前目录是快照根：成员是 crates/ 下各个 Cargo.toml 里 [package] 的 name）；
          # FAKE_CARGO_CLIPPY_RED 设了：特性组合（--all-targets 与 -- 之间的词，没有就是 default）等于它的那一趟打一条告警、退 101
          for manifest in crates/*/Cargo.toml; do
            name="$(sed -n 's/^name = "\(.*\)"$/\1/p' "$manifest" 2>/dev/null | head -n 1)"; [[ -z "$name" ]] || echo "    Checking $name v0.1.0 (/selftest/$name)" >&2
          done
          all="$*"; combination="${all#clippy --offline --all-targets}"; combination="${combination%% -- *}"; combination="${combination# }"
          if [[ -n "${FAKE_CARGO_CLIPPY_RED:-}" && "${combination:-default}" == "$FAKE_CARGO_CLIPPY_RED" ]]; then
            echo "error: function \`only_used_under_a_feature\` is never used"; exit 101
          fi
          exit 0 ;;
esac
FAKE
  [[ "${FAKE_CARGO_MODE:-}" == "" ]] || true
  chmod +x "$fake_cargo_dir/cargo"
  # 快照从临时树拷（MERGE_AND_VERIFY_TREE），不碰本仓的源文件与编译目录
  mkdir -p "$scratch/tree/crates/demo/src"; printf 'pub fn f() {}\n' > "$scratch/tree/crates/demo/src/lib.rs"
  # 样本工作区两个成员：demo 与 验证包；补丁 patch-a 只改到 demo 的库，cargo test 只该起 demo --lib
  write_workspace() {   # $1 树，$2 验证包那个成员的包名
    mkdir -p "$1/crates/demo" "$1/crates/singlefs-verification"
    printf '[workspace]\nmembers = ["crates/demo", "crates/singlefs-verification"]\n' > "$1/Cargo.toml"
    printf '[package]\nname = "demo"\n' > "$1/crates/demo/Cargo.toml"
    printf '[package]\nname = "%s"\n' "$2" > "$1/crates/singlefs-verification/Cargo.toml"
  }
  write_workspace "$scratch/tree" singlefs-verification
  run_case() { PATH="$fake_cargo_dir:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-$1" \
    MERGE_AND_VERIFY_TREE="$scratch/tree" env -u CARGO_TARGET_DIR bash "$0" "${@:2}" > "$out" 2>&1; rc=$?; }
  # 不带 --skip-apply（文件头「先验后合」）：红的归不到项（假 cargo 不打段头），只重跑的是整批再验的合入后验证（不带 --skip-apply、带 patch-a），
  #   点这一趟跑过的 demo:src/lib.rs（文件头「只重跑的命令点名」）
  out="$scratch/out1.txt"; run_case one "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || ! grep -q 'subtracts_in_x' "$out" || ! grep -qF "scripts/gate/merge-and-verify.sh --item demo:src/lib.rs $scratch/patch-a" "$out" \
     || ! grep -qF "跑什么=合入后验证这一趟跑过的 demo:src/lib.rs; 范围=1 项;" "$out"; then
    echo "  ✗ 自检：一条测试红时应当退 1、列出那条测试与只重跑它的命令，实际退出码 $rc：$(tail -4 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 每一步的日志出两份（run_step 经 scripts/hosts/two-logs.sh）：test.log 是全量，test.problems.log 带行号收那条 FAILED，红的出路点名错误警告日志的路径
  checked=$((checked+1))
  if [[ ! -f "$scratch/logs-one/test.log" ]] || ! grep -qE '^[0-9]+:test tests::subtracts_in_x \.\.\. FAILED$' "$scratch/logs-one/test.problems.log" 2>/dev/null \
     || ! grep -qF "错误警告日志：$scratch/logs-one/test.problems.log" "$out"; then
    echo "  ✗ 自检：一条测试红时每一步的日志应当出两份（test.log 全量；test.problems.log 带行号收那条 FAILED），红的出路点名错误警告日志的路径，实际日志目录里有 $(ls "$scratch/logs-one" 2>/dev/null | tr '\n' ' ')：$(tail -3 "$scratch/logs-one/test.problems.log" 2>&1 | tr '\n' '|')"; failures=1; fi   # gate-lint:detail
  # 打补丁之后 mutation-tables 那一格（文件头「门禁 code-source-discipline 的 mutation-tables 格」）：报本次未跑、打出另交的整条命令，汇总不说判过
  checked=$((checked+1))
  if ! grep -qF '· code-source-discipline 的 mutation-tables 格本次未跑' "$scratch/out1.txt" \
     || ! grep -qF "task-server/task submit --cwd $scratch/tree -- bash .claude/gate.d/code-source-discipline.sh --check mutation-tables $scratch/tree" "$scratch/out1.txt" \
     || grep -qF 'code-source-discipline 红 0 个' "$scratch/out1.txt"; then
    echo "  ✗ 自检：打补丁之后应当报 code-source-discipline 的 mutation-tables 格本次未跑、打出补丁打进主工作区之后另交的整条命令，实际：$(grep -E 'mutation-tables|汇总' "$scratch/out1.txt" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  out="$scratch/out2.txt"; run_case two "$scratch/patch-a" "$scratch/patch-b"; checked=$((checked+1))
  if [[ $rc -ne 2 ]] || ! grep -q '补丁核不过' "$out" || ! grep -qF '以主工作区此刻建一棵新段树' "$out" || ! grep -qF '这一份退给实现员以新段树为基线重做' "$out" \
     || ! grep -qF -- 'python3 scripts/hosts/minimal-copy.py <仓根> <新段树> --with-test-inputs --with-gate-tables --with scripts=<为什么> --with .claude/scripts=<为什么> --with .claude/singlefs-ai-sop=<为什么>' "$out"; then
    echo "  ✗ 自检：apply 退 2 时应当整批停、退 2，出路写段树那一句（建新段树那一条与 .claude/main-agent.md 写段第 2 条同一条），实际 $rc：$(tail -2 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 打补丁核不过的是只含拒算表的补丁目录（主 agent 自己从主工作区做的）：整批停、退 2，出路不指实现员与段树，写主 agent 从主工作区此刻重做这一份（.claude/skills/merge-verify/SKILL.md「合入后验证之后判新出现的拒算」第 1 条）
  mkdir -p "$scratch/refused-only/patch-b"
  printf 'diff --git a/.claude/gate.d/refused-tests.tsv b/.claude/gate.d/refused-tests.tsv\n--- a/.claude/gate.d/refused-tests.tsv\n+++ b/.claude/gate.d/refused-tests.tsv\n@@ -1 +1 @@\n-x\n+y\n' > "$scratch/refused-only/patch-b/crates.patch"
  out="$scratch/out-refused-only.txt"; run_case refused-only "$scratch/patch-a" "$scratch/refused-only/patch-b"; checked=$((checked+1))
  if [[ $rc -ne 2 ]] || ! grep -qF '只含拒算表 .claude/gate.d/refused-tests.tsv 的补丁目录' "$out" || ! grep -qF '主 agent 从主工作区此刻重做这一份（.claude/skills/merge-verify/SKILL.md「合入后验证之后判新出现的拒算」第 1 条' "$out" \
     || grep -qF '这一份退给实现员以新段树为基线重做' "$out"; then
    echo "  ✗ 自检：打补丁核不过的是只含拒算表的补丁目录时应当整批停、退 2，出路写主 agent 从主工作区此刻重做这一份、指到 .claude/skills/merge-verify/SKILL.md「合入后验证之后判新出现的拒算」第 1 条，不指实现员与段树，实际退出码 $rc：$(tail -2 "$out" | tr '\n' ' ' | clip_text 500)"; failures=1; fi   # gate-lint:detail
  # 快照（文件头「快照」）测试红（假 cargo 这时还报一条 FAILED）：退 1，自己拷的快照与建的编译目录照样删掉
  out="$scratch/out-copy-red.txt"; run_case copy-red "$scratch/patch-a"; checked=$((checked+1))
  local copy_red_target; copy_red_target="$(run_target_of "$out")"
  if [[ $rc -ne 1 ]] || [[ -e "$scratch/logs-copy-red/repo" || -z "$copy_red_target" || -e "$copy_red_target" ]] || ! grep -q '删了快照' "$out"; then
    echo "  ✗ 自检：快照：测试红时也应当删掉快照与编译目录，实际退出码 $rc，日志目录里还有 $(ls "$scratch/logs-copy-red" 2>/dev/null | tr '\n' ' ')：$(grep -E '✗|删了|拷好快照' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 日志目录指进了要验的树里面：快照会把自己拷进自己，退 2、什么都不拷（这一处拒绝不配弄坏开关：放开它 rsync 会往快照里一层层拷快照）
  out="$scratch/out-snapshot-inside.txt"; : > "$scratch/calls-snapshot-inside.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" FAKE_CARGO_CALLS="$scratch/calls-snapshot-inside.txt" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/tree/logs-inside" \
    MERGE_AND_VERIFY_TREE="$scratch/tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  if [[ $rc -ne 2 || -s "$scratch/calls-snapshot-inside.txt" || -e "$scratch/tree/logs-inside/repo" ]] || ! grep -qF '快照会把自己拷进自己' "$out"; then
    echo "  ✗ 自检：日志目录在要验的树里面时应当退 2、什么都不拷，实际退出码 $rc：$(grep -E '✗|拷好快照' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  rm -rf -- "${scratch:?}/tree/logs-inside"
  # 起 cargo 之前判 RocksDB 预编库的取值（文件头「RocksDB 预编库的取值」）：样本工作区的 验证包依赖 rocksdb、进程环境里四个取值是空的，
  #   PATH 上的假 cargo 不带 entry 认的标记 ⇒ 退 1、假 cargo 一次都没起、输出里有 rocksdb-prebuilt entry 的红句（别的格样本工作区的 验证包不依赖 rocksdb，entry 报不判）
  local rocksdb_tree="$scratch/tree-rocksdb"
  mkdir -p "$rocksdb_tree/crates/demo/src"; printf 'pub fn f() {}\n' > "$rocksdb_tree/crates/demo/src/lib.rs"
  write_workspace "$rocksdb_tree" singlefs-verification
  printf '[dependencies]\nrocksdb = { version = "=0.25.0", optional = true }\n' >> "$rocksdb_tree/crates/singlefs-verification/Cargo.toml"
  out="$scratch/out-rocksdb.txt"; : > "$scratch/calls-rocksdb.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-rocksdb.txt" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-rocksdb" MERGE_AND_VERIFY_TREE="$rocksdb_tree" ROCKSDB_LIB_DIR= ROCKSDB_STATIC= SNAPPY_LIB_DIR= SNAPPY_STATIC= \
    env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  if [[ $rc -ne 1 || -s "$scratch/calls-rocksdb.txt" ]] || ! grep -qF 'rocksdb-prebuilt entry：' "$out"; then
    echo "  ✗ 自检：起 cargo 之前 RocksDB 预编库的取值缺了（验证包依赖 rocksdb、进程环境里没有取值）应当退 1、一条 cargo 都不起，实际退出码 $rc、假 cargo 收到 $(wc -l < "$scratch/calls-rocksdb.txt") 条：$(tail -3 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 假 cargo 全绿的一趟：把 FAILED 行换掉
  sed -i.bak 's/ \.\.\. FAILED/ ... ok/; s/FAILED\. 1 passed; 1 failed/ok. 2 passed; 0 failed/' "$fake_cargo_dir/cargo" && rm -f "$fake_cargo_dir/cargo.bak"
  out="$scratch/out3.txt"; FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-three.txt" FAKE_CARGO_DEBUGINFO="$scratch/debuginfo-three.txt" run_case three "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q '测试全绿' "$out"; then
    echo "  ✗ 自检：全绿时应当退 0 并说全绿，实际 $rc：$(tail -3 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 快照全绿：补丁打进快照（apply 收到 --root <日志目录>/repo），收尾快照与编译目录都删掉、打一行删了什么
  out="$scratch/out-copy-green.txt"; FAKE_CARGO_MODE=green run_case copy-green "$scratch/patch-a"; checked=$((checked+1))
  local copy_green_target; copy_green_target="$(run_target_of "$out")"
  if [[ $rc -ne 0 ]] || [[ -e "$scratch/logs-copy-green/repo" || -z "$copy_green_target" || -e "$copy_green_target" ]] || ! grep -q '删了快照' "$out" \
     || ! grep -qxF "root=$scratch/logs-copy-green/repo" "$scratch/logs-copy-green/apply-patch-a.log" 2>/dev/null; then
    echo "  ✗ 自检：快照：全绿时应当在快照里打补丁、收尾删掉快照与编译目录，实际退出码 $rc，日志目录里还有 $(ls "$scratch/logs-copy-green" 2>/dev/null | tr '\n' ' ')，apply 收到 $(cat "$scratch/logs-copy-green/apply-patch-a.log" 2>/dev/null)：$(grep -E '✗|删了|拷好快照' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # MERGE_AND_VERIFY_KEEP_COPY=1：留着快照与编译目录、打出路径；源树里那份 lib.rs 的修改时刻先拨回 2020 年初，留下的快照里那一份要是拷的那一刻（不保留修改时刻）
  touch -d @1577836800 "$scratch/tree/crates/demo/src/lib.rs"
  out="$scratch/out-copy-kept.txt"; FAKE_CARGO_MODE=green MERGE_AND_VERIFY_KEEP_COPY=1 run_case copy-kept "$scratch/patch-a"; checked=$((checked+1))
  local copy_kept_target kept_mtime; copy_kept_target="$(run_target_of "$out")"
  kept_mtime="$(stat -c %Y "$scratch/logs-copy-kept/repo/crates/demo/src/lib.rs" 2>/dev/null)"
  if [[ $rc -ne 0 ]] || [[ ! -d "$scratch/logs-copy-kept/repo" || -z "$copy_kept_target" || ! -d "$copy_kept_target" ]] \
     || ! grep -qF "留着快照 $scratch/logs-copy-kept/repo 与编译目录 $copy_kept_target" "$out" || [[ ! "$kept_mtime" =~ ^[0-9]+$ ]] || ((kept_mtime <= 1577836800)); then
    echo "  ✗ 自检：快照：MERGE_AND_VERIFY_KEEP_COPY=1 时应当留着快照与编译目录、打出路径，快照里的修改时刻是拷的那一刻，实际退出码 $rc，快照里 lib.rs 的 mtime「${kept_mtime:-取不到}」（源树的是 1577836800）：$(grep -E '✗|留着|删了' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 快照根 git init（文件头「快照」）：读上面留着的那一份快照，它的 git common-dir 是 <快照>/.git
  local kept_copy_common=""; checked=$((checked+1))
  kept_copy_common="$(git -C "$scratch/logs-copy-kept/repo" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || kept_copy_common=""
  if [[ "$kept_copy_common" != "$(cd "$scratch/logs-copy-kept" 2>/dev/null && pwd -P)/repo/.git" ]]; then
    echo "  ✗ 自检：快照：快照根应当 git init（留下的快照 $scratch/logs-copy-kept/repo 的 git common-dir 该是它自己的 .git），实际取到「${kept_copy_common:-取不到}」"; failures=1; fi   # gate-lint:detail
  # 归属记录（文件头「快照」）：留着快照的那一趟在日志目录写 .owner（三行 session、created、purpose，会话号取 WORK_DIRECTORY_OWNER_SESSION），不留快照的那一趟不写
  out="$scratch/out-copy-owner-kept.txt"; FAKE_CARGO_MODE=green MERGE_AND_VERIFY_KEEP_COPY=1 WORK_DIRECTORY_OWNER_SESSION=selftest-owner-session run_case copy-owner-kept "$scratch/patch-a"; checked=$((checked+1))
  out="$scratch/out-copy-owner-dropped.txt"; FAKE_CARGO_MODE=green WORK_DIRECTORY_OWNER_SESSION=selftest-owner-session run_case copy-owner-dropped "$scratch/patch-a"
  local owner_record owner_line_count
  owner_record="$(cat "$scratch/logs-copy-owner-kept/.owner" 2>/dev/null)"; owner_line_count="$(grep -c . <<<"$owner_record")"
  if [[ "$owner_line_count" -ne 3 || "$(sed -n 1p <<<"$owner_record")" != "session: selftest-owner-session" || "$(sed -n 2p <<<"$owner_record")" != "created: $(TZ=Asia/Tokyo date +%F)" \
     || "$(sed -n 3p <<<"$owner_record")" != "purpose: 合入后验证留下的快照 $scratch/logs-copy-owner-kept/repo"* || -e "$scratch/logs-copy-owner-dropped/.owner" ]]; then
    echo "  ✗ 自检：快照：留着快照的那一趟应当在日志目录写归属记录 .owner（三行 session、created、purpose，会话号取 WORK_DIRECTORY_OWNER_SESSION），不留快照的那一趟不写，实际留着的那一趟写的是「$(tr '\n' '|' <<<"$owner_record")」，不留的那一趟日志目录里有 $(ls -A "$scratch/logs-copy-owner-dropped" 2>/dev/null | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 上一次尝试留下的快照（文件头「上一次尝试留下的快照」）：任务 7001 与任务 7002 的尝试 1 都带 MERGE_AND_VERIFY_KEEP_COPY=1 留着快照；
  #   任务 7001 的尝试 2 开跑时删掉它自己尝试 1 留下的（日志目录与记着的任务号、尝试号留着），不碰任务 7002 的；再起一趟尝试 2（尝试号相同）、一趟只有尝试号没有任务号的，都不删前面留下的；
  #   尝试 3 开跑时有进程开着尝试 2 那一棵快照里的文件：那一棵不删、打一行写明，没人开着的另一棵照删；进程放手之后再起一趟尝试 3，把那一棵也删掉
  local retried_logs="$scratch/logs-retried" other_task_logs="$scratch/logs-other-task" retry_logs="$scratch/logs-retry" same_attempt_logs="$scratch/logs-same-attempt"
  local snapshot_holder holder_wait same_attempt_rc held_file="$scratch/logs-retry/repo/held-by-selftest"
  out="$scratch/out-retried.txt"; TASK_SERVER_TASK=7001 TASK_SERVER_ATTEMPT=1 FAKE_CARGO_MODE=green MERGE_AND_VERIFY_KEEP_COPY=1 run_case retried "$scratch/patch-a"
  out="$scratch/out-other-task.txt"; TASK_SERVER_TASK=7002 TASK_SERVER_ATTEMPT=1 FAKE_CARGO_MODE=green MERGE_AND_VERIFY_KEEP_COPY=1 run_case other-task "$scratch/patch-a"
  out="$scratch/out-retry.txt"; TASK_SERVER_TASK=7001 TASK_SERVER_ATTEMPT=2 FAKE_CARGO_MODE=green MERGE_AND_VERIFY_KEEP_COPY=1 run_case retry "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 0 || -e "$retried_logs/repo" || ! -f "$retried_logs/snapshot.log" || ! -d "$retry_logs/repo" \
        || "$(cat "$retried_logs/task-attempt" 2>/dev/null)" != "7001 1" || "$(cat "$retry_logs/task-attempt" 2>/dev/null)" != "7001 2" ]] \
     || ! grep -qF "删了上一次尝试留下的快照 $retried_logs/repo（共 " "$out"; then
    echo "  ✗ 自检：上一次尝试：尝试 2 应当删掉同一任务尝试 1 留下的快照（带 MERGE_AND_VERIFY_KEEP_COPY=1 留的也删）、日志目录留着、打一行「删了上一次尝试留下的快照」、两个日志目录里各记着任务号与尝试号，实际退出码 $rc，尝试 1 的快照$([[ -e "$retried_logs/repo" ]] && echo 还在 || echo 没了)，记着「$(cat "$retried_logs/task-attempt" 2>/dev/null)」「$(cat "$retry_logs/task-attempt" 2>/dev/null)」：$(grep -E '✗|删了|没删' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-same-attempt.txt"; TASK_SERVER_TASK=7001 TASK_SERVER_ATTEMPT=2 FAKE_CARGO_MODE=green MERGE_AND_VERIFY_KEEP_COPY=1 run_case same-attempt "$scratch/patch-a"; same_attempt_rc=$rc
  out="$scratch/out-hand-run.txt"; TASK_SERVER_ATTEMPT=5 FAKE_CARGO_MODE=green run_case hand-run "$scratch/patch-a"; checked=$((checked+1))
  if [[ $same_attempt_rc -ne 0 || $rc -ne 0 || ! -d "$other_task_logs/repo" || ! -d "$retry_logs/repo" || ! -d "$same_attempt_logs/repo" ]] \
     || grep -qF '上一次尝试留下的快照' "$scratch/out-same-attempt.txt" "$scratch/out-hand-run.txt"; then
    echo "  ✗ 自检：上一次尝试：别的任务号的快照、同一任务号尝试号不小于这一趟的快照、没有任务号（手跑）时都不碰，实际尝试号相同那一趟退 $same_attempt_rc、只有尝试号那一趟退 $rc，任务 7002 的快照$([[ -d "$other_task_logs/repo" ]] && echo 在 || echo 没了)，尝试 2 的两棵$([[ -d "$retry_logs/repo" ]] && echo 在 || echo 没了)、$([[ -d "$same_attempt_logs/repo" ]] && echo 在 || echo 没了)：$(grep -hE '✗|删了上一次|没删上一次' "$scratch/out-same-attempt.txt" "$scratch/out-hand-run.txt" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  sleep 30 > "$held_file" 2>&1 < /dev/null & snapshot_holder=$!
  for holder_wait in $(seq 1 100); do [[ -e "$held_file" ]] && break; sleep 0.05; done
  out="$scratch/out-later-retry.txt"; TASK_SERVER_TASK=7001 TASK_SERVER_ATTEMPT=3 FAKE_CARGO_MODE=green run_case later-retry "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 0 || ! -d "$retry_logs/repo" || -e "$same_attempt_logs/repo" ]] \
     || ! grep -qF "没删上一次尝试留下的快照 $retry_logs/repo（任务 7001 的尝试 2）：本机有进程开着里面的文件：进程 $snapshot_holder（sleep 30）" "$out"; then
    echo "  ✗ 自检：上一次尝试：有进程开着上一次快照里的文件时应当不删它、打一行写明开着的进程，没人开着的另一棵照删，这一趟照常跑完，实际退出码 $rc，被开着的那棵$([[ -d "$retry_logs/repo" ]] && echo 在 || echo 没了)，没人开着的那棵$([[ -d "$same_attempt_logs/repo" ]] && echo 在 || echo 没了)：$(grep -E '✗|删了上一次|没删上一次' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  kill "$snapshot_holder" 2>/dev/null; wait "$snapshot_holder" 2>/dev/null
  out="$scratch/out-retry-after-release.txt"; TASK_SERVER_TASK=7001 TASK_SERVER_ATTEMPT=3 FAKE_CARGO_MODE=green run_case retry-after-release "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 0 || -e "$retry_logs/repo" || -e "$same_attempt_logs/repo" || ! -d "$retry_logs" ]]; then
    echo "  ✗ 自检：上一次尝试：进程放手之后再起一趟尝试 3 应当把还留着的快照删掉、日志目录留着，实际退出码 $rc，尝试 2 的快照$([[ -e "$retry_logs/repo" ]] && echo 还在 || echo 没了)：$(grep -E '✗|删了上一次|没删上一次' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 与 check.sh 同一套判据：lint 表在这里另取一遍（sed 取那一段、grep 数条数），与脚本里 read_check_sh 不是同一段代码
  expected_lints="$(sed -n '/^CODE_DISCIPLINE_LINTS=(/,/^)/p' "$CHECK_SH" | grep -oE -- '-[DWAF] clippy::[a-z0-9_]+' | tr '\n' ' ')"
  expected_lints="${expected_lints% }"
  expected_lint_count="$(sed -n '/^CODE_DISCIPLINE_LINTS=(/,/^)/p' "$CHECK_SH" | grep -cE -- '-[DWAF] clippy::')"
  # 特性组合同样在这里另取一遍（sed 取行首那一行、grep 抠双引号里的词），步名照文件头的规则另算一遍
  while IFS= read -r combination; do expected_combinations+=("$combination"); done \
    < <(sed -n 's/^CLIPPY_FEATURE_COMBINATIONS=(\(.*\))[[:space:]]*$/\1/p' "$CHECK_SH" | grep -oE '"[^"]*"' | tr -d '"')
  checked=$((checked+1))
  clippy_received="$(grep '^clippy' "$scratch/calls-three.txt")"
  if [[ "$expected_lint_count" -lt 1 || -z "$clippy_received" ]] || grep -vqE -- " -- -D warnings $expected_lints\$" <<< "$clippy_received"; then
    echo "  ✗ 自检：clippy 应当带上 check.sh 的 lint 表里每一条（$CHECK_SH 里数到 $expected_lint_count 条，每个特性组合那一条都带），假 cargo 收到的是：$(tr '\n' '|' <<< "$clippy_received")"; failures=1; fi   # gate-lint:detail
  # clippy 照 check.sh 的特性组合逐个跑：每个组合一条、次序同 check.sh，default 不带特性参数（只比 -- 之前那一段，lint 表归上一格）
  checked=$((checked+1))
  clippy_expected=""
  for combination in ${expected_combinations[@]+"${expected_combinations[@]}"}; do
    clippy_expected+="clippy --offline --all-targets$([[ "$combination" == default ]] || printf ' %s' "$combination")|"
  done
  clippy_received="$(sed 's/ -- .*//' <<< "$clippy_received" | tr '\n' '|')"
  if ((${#expected_combinations[@]} < 2)) || [[ "$clippy_received" != "$clippy_expected" ]]; then
    echo "  ✗ 自检：clippy 应当照 check.sh 的特性组合逐个跑（$CHECK_SH 里读到 ${#expected_combinations[@]} 个：${expected_combinations[*]-}；要收到 $clippy_expected），假 cargo 收到的是：$clippy_received"; failures=1; fi   # gate-lint:detail
  # 一个特性组合红（假 cargo 在 default 那一趟打告警、退 101）：整步红、退 1、点名 default，别的组合照跑
  out="$scratch/out-clippy-red.txt"; checked=$((checked+1))
  FAKE_CARGO_MODE=green FAKE_CARGO_CLIPPY_RED=default FAKE_CARGO_CALLS="$scratch/calls-clippy-red.txt" run_case clippy-red --skip-apply
  if [[ $rc -ne 1 ]] || ! grep -qF '✗ clippy 特性组合 default 红' "$out" || ! grep -qF "✗ clippy 红：${#expected_combinations[@]} 个特性组合里 1 个红（default）" "$out" \
     || grep -qF '✓ clippy 过' "$out" || [[ "$(grep -c '^clippy' "$scratch/calls-clippy-red.txt" 2>/dev/null)" != "${#expected_combinations[@]}" ]] \
     || ! grep -qF 'only_used_under_a_feature' "$scratch/logs-clippy-red/clippy-default.log" 2>/dev/null; then
    echo "  ✗ 自检：clippy 一个特性组合红时应当整步红、退 1、点名红的那个组合（default，日志 clippy-default.log），别的组合照跑，实际退出码 $rc，假 cargo 收到 $(grep -c '^clippy' "$scratch/calls-clippy-red.txt" 2>/dev/null) 条 clippy：$(grep -E 'clippy' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 要验的树里有 check.sh 认的特性组合登记（上游 lib.sh 的 CLIPPY_FEATURE_COMBINATIONS_FILE，此刻是 .claude/clippy-feature-combinations）：退 1、不编
  mkdir -p "$scratch/tree/.claude" && printf -- '--all-features  # 自证样本\n' > "$scratch/tree/.claude/clippy-feature-combinations"
  out="$scratch/out-clippy-registered.txt"; checked=$((checked+1))
  FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-clippy-registered.txt" run_case clippy-registered --skip-apply
  rm -rf -- "${scratch:?}/tree/.claude"
  if [[ $rc -ne 1 ]] || ! grep -qF '要验的树里有 check.sh 认的特性组合登记' "$out" || [[ -s "$scratch/calls-clippy-registered.txt" ]]; then
    echo "  ✗ 自检：要验的树里有特性组合登记时应当退 1、不编，实际退出码 $rc，假 cargo 收到 $(cat "$scratch/calls-clippy-registered.txt" 2>/dev/null | wc -l) 条：$(grep -E '✗|!' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  checked=$((checked+1))
  # build 那一步（文件头 build 一段）：check.sh 的构建那一条只点补丁 patch-a 改到的 demo（样本工作区里没有包依赖它），成功句写出编了哪几个包
  if [[ "$(grep '^build' "$scratch/calls-three.txt" | tr '\n' '|')" != "build --offline --all-targets -p demo|" ]] || ! grep -qF '✓ build 过（编了 1 个包：demo）' "$scratch/out3.txt"; then
    echo "  ✗ 自检：应当跑 cargo build --offline --all-targets -p demo（check.sh 的构建那一条，只点补丁改到的 demo），假 cargo 收到的是：$(tr '\n' '|' < "$scratch/calls-three.txt")；build 那一句：$(grep -E 'build' "$scratch/out3.txt" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # build 那一条关掉调试信息（文件头 build 一段）：同一趟（out3）假 cargo 把每条收到的 CARGO_PROFILE_DEV_DEBUG 记进 debuginfo-three.txt，
  #   build 那一条恰好一行、收到 0；test、fmt、clippy 那几条都没收到（记 unset），test 与 clippy 都要有记下的行
  local debuginfo_record="$scratch/debuginfo-three.txt" debuginfo_build debuginfo_leaked
  checked=$((checked+1))
  debuginfo_build="$(grep -c $'^build\t0$' "$debuginfo_record" 2>/dev/null)"
  debuginfo_leaked="$(grep -v $'^build\t' "$debuginfo_record" 2>/dev/null | grep -vc $'\tunset$')"
  if [[ "${debuginfo_build:-0}" -ne 1 || "${debuginfo_leaked:-0}" -ne 0 ]] || ! grep -q $'^test\tunset$' "$debuginfo_record" || ! grep -q $'^clippy\tunset$' "$debuginfo_record"; then
    echo "  ✗ 自检：build 那一条的环境里应当收到 CARGO_PROFILE_DEV_DEBUG=0（关掉调试信息），test、fmt、clippy 那几条不收到，假 cargo 记下的是：$(tr '\t\n' '=|' < "$debuginfo_record" 2>/dev/null)"; failures=1; fi   # gate-lint:detail
  checked=$((checked+1))
  if [[ "$(grep '^test' "$scratch/calls-three.txt" | tr '\n' '|')" != "test --offline --no-fail-fast -p demo --lib|" ]]; then
    echo "  ✗ 自检：cargo test 应当只起补丁改到的 demo 的库目标（-p demo --lib），假 cargo 收到的是：$(grep '^test' "$scratch/calls-three.txt" | tr '\n' '|')"; failures=1; fi   # gate-lint:detail
  # 测试那一步按测试目标并行起（文件头 test 一段）：这一格不设 MERGE_AND_VERIFY_TEST_TARGETS，假 cargo 的 metadata 与 --no-run 打录好的两个测试目标（demo 的 lib 与 test alpha）；
  # 全绿那一趟两个目标各起一条、汇总数到 2 个测试二进制 4 条测试，红的那一趟两条 FAILED 都列出来、退 1
  printf '%s\n' '{"packages":[{"id":"demo-id","name":"demo","targets":[{"kind":["lib"],"name":"demo","doctest":false},{"kind":["test"],"name":"alpha","doctest":false}]}],"workspace_members":["demo-id"]}' \
    > "$scratch/fake-metadata.json"
  printf '%s\n' '{"reason":"compiler-artifact","package_id":"demo-id","target":{"kind":["lib"],"name":"demo"},"profile":{"test":true},"executable":"/selftest/demo"}' \
    '{"reason":"compiler-artifact","package_id":"demo-id","target":{"kind":["test"],"name":"alpha"},"profile":{"test":true},"executable":"/selftest/alpha"}' > "$scratch/fake-artifacts.jsonl"
  mkdir -p "$scratch/tree/crates/demo/tests"; printf 'fn main() {}\n' > "$scratch/tree/crates/demo/tests/alpha.rs"   # 这两格之后删掉：别的格的 demo 只有库目标
  out="$scratch/out-targets.txt"; MERGE_AND_VERIFY_TEST_TARGETS= FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-targets.txt" \
    FAKE_CARGO_METADATA="$scratch/fake-metadata.json" FAKE_CARGO_ARTIFACTS="$scratch/fake-artifacts.jsonl" run_case targets "$scratch/patch-targets"; checked=$((checked+1))
  local targets_green_rc=$rc targets_red_out="$scratch/out-targets-red.txt" targets_red_rc
  # 红的那一趟要假 cargo 照最早那一份打一条 FAILED（全绿那一格之前已把它换成全 ok），跑完换回全 ok
  sed -i.bak 's/subtracts_in_%s \.\.\. ok/subtracts_in_%s ... FAILED/; s/ok\. 2 passed; 0 failed/FAILED. 1 passed; 1 failed/' "$fake_cargo_dir/cargo" && rm -f "$fake_cargo_dir/cargo.bak"
  out="$targets_red_out"; MERGE_AND_VERIFY_TEST_TARGETS= FAKE_CARGO_CALLS="$scratch/calls-targets-red.txt" \
    FAKE_CARGO_METADATA="$scratch/fake-metadata.json" FAKE_CARGO_ARTIFACTS="$scratch/fake-artifacts.jsonl" run_case targets-red "$scratch/patch-targets"; targets_red_rc=$rc
  rm -rf -- "${scratch:?}/tree/crates/demo/tests"
  sed -i.bak 's/ \.\.\. FAILED/ ... ok/; s/FAILED\. 1 passed; 1 failed/ok. 2 passed; 0 failed/' "$fake_cargo_dir/cargo" && rm -f "$fake_cargo_dir/cargo.bak"
  out="$scratch/out-targets.txt"
  if [[ $targets_green_rc -ne 0 ]] || ! grep -qxF -- "test --offline --no-fail-fast --lib --test alpha -p demo --no-run --message-format=json-render-diagnostics" "$scratch/calls-targets.txt" \
     || ! grep -qxF -- "test --offline --no-fail-fast -p demo --lib" < "$scratch/calls-targets.txt" || ! grep -qxF -- "test --offline --no-fail-fast -p demo --test alpha" < "$scratch/calls-targets.txt" \
     || ! grep -q '✓ 测试全绿（2 个测试二进制，共 4 条测试' "$out" || ! grep -q '^TEST_TARGETS_PARALLEL dispatched=2 collected=2 ' "$scratch/logs-targets/test.log" \
     || [[ $targets_red_rc -ne 1 ]] || ! grep -q '红的测试 2 条' "$targets_red_out"; then
    echo "  ✗ 自检：测试那一步应当经 test-targets-parallel.py 按测试目标并行起（先 --no-run 编、两个目标各一条、汇总 2 个测试二进制 4 条测试；红的那一趟列出 2 条、退 1），假 cargo 收到的是：$(grep '^test' "$scratch/calls-targets.txt" 2>/dev/null | tr '\n' '|')；全绿那一趟：$(grep -E '✓ 测试|✗' "$out" | head -2 | tr '\n' ' ')；红的那一趟退 $targets_red_rc：$(grep -E '红的测试|✓ 测试' "$targets_red_out" | head -1)"; failures=1; fi   # gate-lint:detail
  # 测试那一步的 cargo test 在 -- 之后什么都不带（文件头 test 一段）：一条 cargo test 那一种（全绿那一格）与按测试目标并行起那一种（上一格）收到的命令里都没有 -- 之后的参数，验证包照旧不进，成功句不报标 #[ignore] 的条数
  checked=$((checked+1))
  if grep -qE '^test .* -- ' "$scratch/calls-three.txt" "$scratch/calls-targets.txt" \
     || grep -qE '^test .*-p singlefs-verification' "$scratch/calls-three.txt" "$scratch/calls-targets.txt" \
     || ! grep -qF '共 4 条测试；跑了 2 项（1 个包）' "$scratch/out-targets.txt"; then
    echo "  ✗ 自检：测试那一步的 cargo test 在 -- 之后应当什么都不带（一条 cargo test 与按测试目标并行起两种都一样）、验证包照旧不进、成功句不报标 #[ignore] 的条数，假 cargo 收到的是：$(grep -h '^test' "$scratch/calls-three.txt" "$scratch/calls-targets.txt" 2>/dev/null | tr '\n' '|')；成功句：$(grep '✓ 测试' "$scratch/out-targets.txt" | head -1)"; failures=1; fi   # gate-lint:detail
  # 挑测试目标（文件头「挑测试目标」）：样本工作区成员 demo、alpha、beta（dev 依赖 alpha）、delta（dev 依赖 demo）、验证包与产品 GPU 包（依赖验证包）；
  #   alpha 有测试目标 case_a、beta 有 case，验证包有库与 setting、gpu_pair 两个测试目标，产品 GPU 包的 real_card 写了 required-features
  local narrow_tree="$scratch/narrow-tree" narrow_member setting_directory
  mkdir -p "$narrow_tree"
  printf '[workspace]\nmembers = ["crates/demo", "crates/alpha", "crates/beta", "crates/delta", "crates/singlefs-verification", "crates/singlefs-background-gpu-tests"]\n' > "$narrow_tree/Cargo.toml"
  for narrow_member in demo alpha beta delta singlefs-verification singlefs-background-gpu-tests; do
    mkdir -p "$narrow_tree/crates/$narrow_member/src" "$narrow_tree/crates/$narrow_member/tests"; printf 'pub fn f() {}\n' > "$narrow_tree/crates/$narrow_member/src/lib.rs"
    printf '[package]\nname = "%s"\nedition = "2021"\n' "$narrow_member" > "$narrow_tree/crates/$narrow_member/Cargo.toml"   # 写明 2021：没写 edition 的 cargo 按 2015 推自动认的目标
  done
  printf '\n[dev-dependencies]\nalpha = { path = "../alpha" }\n' >> "$narrow_tree/crates/beta/Cargo.toml"
  printf '\n[dev-dependencies]\ndemo = { path = "../demo" }\n' >> "$narrow_tree/crates/delta/Cargo.toml"
  printf '\n[[test]]\nname = "real_card"\nrequired-features = ["tests-on-a-real-gpu"]\n' >> "$narrow_tree/crates/singlefs-background-gpu-tests/Cargo.toml"
  printf '\n[dependencies]\nsinglefs-verification = { path = "../singlefs-verification" }\n' >> "$narrow_tree/crates/singlefs-background-gpu-tests/Cargo.toml"   # 产品 GPU 包依赖验证包
  printf 'fn main() {}\n' > "$narrow_tree/crates/alpha/tests/case_a.rs"
  printf 'fn main() {}\n' > "$narrow_tree/crates/singlefs-verification/tests/setting.rs"
  write_crash_domain_tables "$narrow_tree" demo-case other-case third-case
  narrow_patch() {   # $1 补丁目录，其余是改到的路径；树里那几份写成打过之后的内容（--skip-apply 先核补丁打过没有）
    mkdir -p "$1"; local narrow_path; : > "$1/crates.patch"
    for narrow_path in "${@:2}"; do printf -- '--- a/%s\n+++ b/%s\n@@ -1 +1 @@\n-1\n+2\n' "$narrow_path" "$narrow_path" >> "$1/crates.patch"
      mkdir -p "$(dirname "$narrow_tree/$narrow_path")"; printf '2\n' > "$narrow_tree/$narrow_path"; done
  }
  narrow_case() {   # $1 标签，其余是参数
    FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-narrow-$1.txt" PATH="$fake_cargo_dir:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" \
      MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-narrow-$1" MERGE_AND_VERIFY_TREE="$narrow_tree" env -u CARGO_TARGET_DIR bash "$0" "${@:2}" > "$out" 2>&1; rc=$?
  }
  narrow_tests() { grep '^test' "$scratch/calls-narrow-$1.txt" 2>/dev/null | tr '\n' '|'; }   # <标签>：假 cargo 收到的测试命令，| 连起来
  # 补丁改到 alpha 的库（src 下的数据文件）与 beta 的测试目标 case：只起这两个目标，各一条（不带上依赖 alpha 的 beta 的别的目标、也不跑 demo、delta）
  narrow_patch "$scratch/narrow-two" crates/alpha/src/table.txt crates/beta/tests/case.rs
  out="$scratch/out-narrow-two.txt"; narrow_case two --skip-apply "$scratch/narrow-two"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests two)" != "test --offline --no-fail-fast -p alpha --lib|test --offline --no-fail-fast -p beta --test case|" ]] \
     || ! grep -qF '核过 1 份补丁都已打在树上' "$out" || ! grep -q '✓ 测试全绿.*跑了 2 项（2 个包）' "$out"; then
    echo "  ✗ 自检：补丁改到 alpha 的库与 beta 的测试目标 case：测试那一步只该起 alpha --lib 与 beta --test case 两条，实际退出码 $rc，假 cargo 收到 $(narrow_tests two)：$(grep -E '✓ 测试|✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 补丁改到 alpha 的 src/lib.rs、变异行点名 alpha 的测试目标 case_a（带筛选词）：只起 alpha 一条 --lib --test case_a，依赖 alpha 的 beta 一个目标都不起
  narrow_patch "$scratch/narrow-alpha" crates/alpha/src/lib.rs
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'alpha 的 case_a（自证样本）' crates/alpha/src/lib.rs 2 3 '-p alpha --test case_a -- a_filter' a_filter > "$scratch/narrow-alpha/mutations-append.tsv"
  out="$scratch/out-narrow-alpha.txt"; narrow_case alpha --skip-apply "$scratch/narrow-alpha"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests alpha)" != "test --offline --no-fail-fast -p alpha --lib --test case_a|" ]] || ! grep -qF '筛选词 a_filter' "$out"; then
    echo "  ✗ 自检：补丁改到 alpha 的 src/lib.rs、变异行点名 alpha 的 case_a：测试那一步只该起 alpha --lib 与 --test case_a，不起依赖 alpha 的 beta，实际退出码 $rc，假 cargo 收到 $(narrow_tests alpha)：$(grep -E '✓ 测试|筛选词|✗' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 包的 Cargo.toml 只改 [[test]] / [[bin]] 段（文件头「挑测试目标」包的 Cargo.toml 那一条）：补丁目录里是 alpha 的 Cargo.toml 旧侧到新侧的真补丁（--skip-apply 先核补丁打过没有，要倒得掉），
  #   树里的 Cargo.toml 写成新侧，每格跑完还原；alpha 的 Cargo.toml 平常只有 [package] 那三行（alpha_package_lines，写明 edition 2021），旧侧、新侧在它后面接各格的段。
  #   新加写了 path 的 [[test]] → 那个目标；新加没写 path 的 [[bin]] → 缺省路径 src/bin/<名>.rs 那个目标；新加写了 required-features 的 [[test]] → 要特性、测试那一步不跑；
  #   删掉 [[test]] 段 → 报一句、不跑；同改 [dependencies] 或只动注释 → 照旧判不出、退 3；
  #   [[bin]] 段有差别 → 只挑那个 bin，同包的集成测试 case_a（tests/case_a.rs，自动认的）不挑；
  #   新加一段与自动认的 case_a 同名的 [[test]]、path 指到别处 → 自动认的 case_a 被顶掉，判不出、退 3、原因点名 case_a 与 tests/case_a.rs；
  #   新加一段 [[bin]] 名字是 tool-cli、path 指到自动认的 src/bin/tool.rs（照 singlefs-unit-graph 包 [[bin]] unit-graph 那种写法）→ 根文件还有目标认、不算顶掉，只挑 --bin tool-cli
  local alpha_package_lines=$'[package]\nname = "alpha"\nedition = "2021"\n'
  manifest_patch() {   # <补丁目录> <旧侧接在 [package] 后面的段> <新侧接在 [package] 后面的段> [<两侧的 [package] 那几行，缺省是 alpha_package_lines>]
    local sides="$scratch/manifest-sides" package_lines="${4-$alpha_package_lines}"
    rm -rf -- "${sides:?}"; mkdir -p "$sides/old/crates/alpha" "$sides/new/crates/alpha" "$1"
    printf '%s%s' "$package_lines" "$2" > "$sides/old/crates/alpha/Cargo.toml"
    printf '%s%s' "$package_lines" "$3" > "$sides/new/crates/alpha/Cargo.toml"
    cp -- "$sides/new/crates/alpha/Cargo.toml" "$narrow_tree/crates/alpha/Cargo.toml"
    { printf 'diff --git a/crates/alpha/Cargo.toml b/crates/alpha/Cargo.toml\n--- a/crates/alpha/Cargo.toml\n+++ b/crates/alpha/Cargo.toml\n'
      (cd "$sides" && git diff --no-index --no-color --no-ext-diff --no-prefix old new | tail -n +5); } > "$1/crates.patch"
  }
  manifest_restore() {
    printf '%s' "$alpha_package_lines" > "$narrow_tree/crates/alpha/Cargo.toml"
    rm -rf -- "${narrow_tree:?}/crates/alpha/checks" "${narrow_tree:?}/crates/alpha/src/bin"
  }
  mkdir -p "$narrow_tree/crates/alpha/checks" "$narrow_tree/crates/alpha/src/bin"
  printf 'fn main() {}\n' > "$narrow_tree/crates/alpha/checks/extra.rs"; printf 'fn main() {}\n' > "$narrow_tree/crates/alpha/checks/gated.rs"; printf 'fn main() {}\n' > "$narrow_tree/crates/alpha/src/bin/tool.rs"
  manifest_patch "$scratch/narrow-manifest-test" '' $'\n[[test]]\nname = "extra"\npath = "checks/extra.rs"\n'
  out="$scratch/out-narrow-manifest-test.txt"; narrow_case manifest-test --skip-apply "$scratch/narrow-manifest-test"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests manifest-test)" != "test --offline --no-fail-fast -p alpha --test extra|" ]] || ! grep -qF '补丁改到 crates/alpha/Cargo.toml（[[test]] 段 extra：新加）' "$out" || grep -qF '判不出' "$out"; then
    echo "  ✗ 自检：挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段写了 path 的 [[test]]：只该起它点名的目标 --test extra、不退 3，实际退出码 $rc，假 cargo 收到 $(narrow_tests manifest-test)：$(grep -E '判不出|补丁改到|✗' "$out" | head -3 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  manifest_patch "$scratch/narrow-manifest-bin" '' $'\n[[bin]]\nname = "tool"\n'
  out="$scratch/out-narrow-manifest-bin.txt"; narrow_case manifest-bin --skip-apply "$scratch/narrow-manifest-bin"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests manifest-bin)" != "test --offline --no-fail-fast -p alpha --bin tool|" ]] || ! grep -qF '补丁改到 crates/alpha/Cargo.toml（[[bin]] 段 tool：新加）' "$out" || grep -qF '判不出' "$out"; then
    echo "  ✗ 自检：挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段没写 path 的 [[bin]]：该照缺省路径 src/bin/tool.rs 只落到 --bin tool（同包的集成测试 case_a 不挑）、不退 3，实际退出码 $rc，假 cargo 收到 $(narrow_tests manifest-bin)：$(grep -E '判不出|补丁改到|✗' "$out" | head -3 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  # 新加一段 [[bin]] tool-cli，path 指到自动认的 src/bin/tool.rs（名字不同）：旧侧自动认的 tool 的根文件在新侧还是 tool-cli 的根文件，不算顶掉，只起 --bin tool-cli
  manifest_patch "$scratch/narrow-manifest-renamed" '' $'\n[[bin]]\nname = "tool-cli"\npath = "src/bin/tool.rs"\n'
  out="$scratch/out-narrow-manifest-renamed.txt"; narrow_case manifest-renamed --skip-apply "$scratch/narrow-manifest-renamed"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests manifest-renamed)" != "test --offline --no-fail-fast -p alpha --bin tool-cli|" ]] || ! grep -qF '补丁改到 crates/alpha/Cargo.toml（[[bin]] 段 tool-cli：新加）' "$out" \
     || grep -qF '判不出' "$out" || grep -qF '顶掉' "$out"; then
    echo "  ✗ 自检：挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段 path 指到自动认的 src/bin/tool.rs、名字不同的 [[bin]]：根文件还有目标认、不算顶掉，只该起 --bin tool-cli、不退 3，实际退出码 $rc，假 cargo 收到 $(narrow_tests manifest-renamed)：$(grep -E '判不出|顶掉|补丁改到|✗' "$out" | head -3 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  # 新加一段与自动认的测试目标 case_a 同名的 [[test]]（path 指到 checks/extra.rs）：cargo 从此不认 tests/case_a.rs，判不出、退 3、原因点名被顶掉的 case_a
  manifest_patch "$scratch/narrow-manifest-shadow" '' $'\n[[test]]\nname = "case_a"\npath = "checks/extra.rs"\n'
  out="$scratch/out-narrow-manifest-shadow.txt"; narrow_case manifest-shadow --skip-apply "$scratch/narrow-manifest-shadow"; checked=$((checked+1))
  if [[ $rc -ne 3 || -n "$(narrow_tests manifest-shadow)" ]] || ! grep -qF '补丁改到 alpha 包里的 Cargo.toml，判不出改到哪个测试目标' "$out" \
     || ! grep -qF '自动认的目标被顶掉，cargo test 从此不跑它的根文件、一句不报：test 目标 case_a（自动认的，根文件 tests/case_a.rs）改后同名的 [[test]] 段指到 checks/extra.rs' "$out"; then
    echo "  ✗ 自检：判不出测试目标：补丁在 alpha 的 Cargo.toml 里新加一段与自动认的 case_a 同名的 [[test]]：测试那一步不跑、退 3，原因点名被顶掉的 case_a（tests/case_a.rs），实际退出码 $rc，假 cargo 收到 $(narrow_tests manifest-shadow)：$(grep -E '判不出|顶掉|✗' "$out" | head -2 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  manifest_patch "$scratch/narrow-manifest-gated" '' $'\n[[test]]\nname = "gated"\npath = "checks/gated.rs"\nrequired-features = ["gate-on"]\n'
  out="$scratch/out-narrow-manifest-gated.txt"; narrow_case manifest-gated --skip-apply "$scratch/narrow-manifest-gated"; checked=$((checked+1))
  if [[ $rc -ne 0 || -n "$(narrow_tests manifest-gated)" ]] || ! grep -qF 'alpha 的 gated 要特性 gate-on（补丁改到 crates/alpha/Cargo.toml（[[test]] 段 gated：新加））：测试那一步不跑' "$out" || grep -qF '判不出' "$out"; then
    echo "  ✗ 自检：挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段写了 required-features 的 [[test]]：记要特性、测试那一步不跑它、不退 3，实际退出码 $rc，假 cargo 收到 $(narrow_tests manifest-gated)：$(grep -E '判不出|要特性|✗' "$out" | head -3 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  manifest_patch "$scratch/narrow-manifest-removed" $'\n[[test]]\nname = "extra"\npath = "checks/extra.rs"\n' ''
  out="$scratch/out-narrow-manifest-removed.txt"; narrow_case manifest-removed --skip-apply "$scratch/narrow-manifest-removed"; checked=$((checked+1))
  if [[ $rc -ne 0 || -n "$(narrow_tests manifest-removed)" ]] || ! grep -qF '补丁改到 crates/alpha/Cargo.toml（[[test]] 段 extra：删掉）：它的根文件 checks/extra.rs 现在不是 cargo test 会跑的目标' "$out" || grep -qF '判不出' "$out"; then
    echo "  ✗ 自检：挑测试目标：补丁只在 alpha 的 Cargo.toml 里删掉一段 [[test]]：报一句、测试那一步不跑、不退 3，实际退出码 $rc，假 cargo 收到 $(narrow_tests manifest-removed)：$(grep -E '判不出|删掉|✗' "$out" | head -3 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  manifest_patch "$scratch/narrow-manifest-mixed" '' $'\n[[test]]\nname = "extra"\npath = "checks/extra.rs"\n\n[dependencies]\ndemo = { path = "../demo" }\n'
  out="$scratch/out-narrow-manifest-mixed.txt"; narrow_case manifest-mixed --skip-apply "$scratch/narrow-manifest-mixed"; checked=$((checked+1))
  if [[ $rc -ne 3 || -n "$(narrow_tests manifest-mixed)" ]] || ! grep -qF '补丁改到 alpha 包里的 Cargo.toml，判不出改到哪个测试目标' "$out" || ! grep -qF '包的 Cargo.toml 只改 [[test]] / [[bin]] 段才认得出：[dependencies] 段有改动' "$out"; then
    echo "  ✗ 自检：判不出测试目标：补丁在 alpha 的 Cargo.toml 里新加 [[test]] 段、同时改了 [dependencies]：测试那一步不跑、退 3，原因点名 [dependencies]，实际退出码 $rc，假 cargo 收到 $(narrow_tests manifest-mixed)：$(grep -E '判不出|✗' "$out" | head -2 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  manifest_patch "$scratch/narrow-manifest-comment" '' $'# 只加了一行注释\n'
  out="$scratch/out-narrow-manifest-comment.txt"; narrow_case manifest-comment --skip-apply "$scratch/narrow-manifest-comment"; checked=$((checked+1))
  if [[ $rc -ne 3 || -n "$(narrow_tests manifest-comment)" ]] || ! grep -qF '包的 Cargo.toml 只改 [[test]] / [[bin]] 段才认得出：没有一个 [[test]] / [[bin]] 段有差别' "$out"; then
    echo "  ✗ 自检：判不出测试目标：补丁只在 alpha 的 Cargo.toml 里加了一行注释：测试那一步不跑、退 3，原因写没有 [[test]] / [[bin]] 段有差别，实际退出码 $rc，假 cargo 收到 $(narrow_tests manifest-comment)：$(grep -E '判不出|✗' "$out" | head -2 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  # bin 只按改动挑：改了一段 [[bin]]（tool 加 test = true）只挑 --bin tool；补丁只改 bin 的根文件 src/bin/tool.rs（Cargo.toml 没有段，tool 是自动认的）只挑 --bin tool；
  #   两格都不挑同包的库与集成测试 case_a
  manifest_patch "$scratch/narrow-manifest-bin-changed" $'\n[[bin]]\nname = "tool"\n' $'\n[[bin]]\nname = "tool"\ntest = true\n'
  out="$scratch/out-narrow-manifest-bin-changed.txt"; narrow_case manifest-bin-changed --skip-apply "$scratch/narrow-manifest-bin-changed"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests manifest-bin-changed)" != "test --offline --no-fail-fast -p alpha --bin tool|" ]] || ! grep -qF '补丁改到 crates/alpha/Cargo.toml（[[bin]] 段 tool：改了）' "$out" || grep -qF '判不出' "$out"; then
    echo "  ✗ 自检：挑测试目标：补丁只在 alpha 的 Cargo.toml 里改了一段 [[bin]]（tool 加了 test = true）：只该起 --bin tool（同包的集成测试 case_a 不挑）、不退 3，实际退出码 $rc，假 cargo 收到 $(narrow_tests manifest-bin-changed)：$(grep -E '判不出|补丁改到|✗' "$out" | head -3 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  printf '%s' "$alpha_package_lines" > "$narrow_tree/crates/alpha/Cargo.toml"
  narrow_patch "$scratch/narrow-bin-root" crates/alpha/src/bin/tool.rs
  out="$scratch/out-narrow-bin-root.txt"; narrow_case bin-root --skip-apply "$scratch/narrow-bin-root"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests bin-root)" != "test --offline --no-fail-fast -p alpha --bin tool|" ]] || ! grep -qF '补丁改到 crates/alpha/src/bin/tool.rs' "$out" || grep -qF '判不出' "$out"; then
    echo "  ✗ 自检：挑测试目标：补丁只改 alpha 的 bin 根文件 src/bin/tool.rs：只该起 --bin tool（同包的库与集成测试 case_a 不挑）、不退 3，实际退出码 $rc，假 cargo 收到 $(narrow_tests bin-root)：$(grep -E '判不出|补丁改到|✗' "$out" | head -3 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  # 段写的 path 先归一：新加 [[test]] case_a、path 写成 ./tests/case_a.rs，归一之后就是自动认的 case_a 那份文件（cargo 认的也是它），不算顶掉，只挑 --test case_a
  manifest_patch "$scratch/narrow-manifest-dot-path" '' $'\n[[test]]\nname = "case_a"\npath = "./tests/case_a.rs"\n'
  out="$scratch/out-narrow-manifest-dot-path.txt"; narrow_case manifest-dot-path --skip-apply "$scratch/narrow-manifest-dot-path"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests manifest-dot-path)" != "test --offline --no-fail-fast -p alpha --test case_a|" ]] || ! grep -qF '补丁改到 crates/alpha/Cargo.toml（[[test]] 段 case_a：新加）' "$out" \
     || grep -qF '判不出' "$out" || grep -qF '顶掉' "$out"; then
    echo "  ✗ 自检：挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加一段 path 写成 ./tests/case_a.rs 的 [[test]] case_a：归一之后就是自动认的那份文件、不算顶掉，只该起 --test case_a、不退 3，实际退出码 $rc，假 cargo 收到 $(narrow_tests manifest-dot-path)：$(grep -E '判不出|顶掉|补丁改到|✗' "$out" | head -3 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  # 目录形根文件（rerun-failed-tests.py 的 default_root_paths，目标清单与两侧目标集合共用）：tests/d/main.rs 与 src/bin/probe/main.rs；
  #   新加两段没写 path 的 [[bin]] probe、[[test]] d → 落到这两个目标；段留着、补丁只改两份根文件的正文与同目录的子模块 part.rs → 同样只挑这两个；两格都不退 3
  mkdir -p "$narrow_tree/crates/alpha/tests/d" "$narrow_tree/crates/alpha/src/bin/probe"
  printf 'fn main() {}\n' > "$narrow_tree/crates/alpha/tests/d/main.rs"; printf 'fn main() {}\n' > "$narrow_tree/crates/alpha/src/bin/probe/main.rs"
  manifest_patch "$scratch/narrow-manifest-directory-form" '' $'\n[[bin]]\nname = "probe"\n\n[[test]]\nname = "d"\nharness = true\n'
  out="$scratch/out-narrow-manifest-directory-form.txt"; narrow_case manifest-directory-form --skip-apply "$scratch/narrow-manifest-directory-form"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests manifest-directory-form)" != "test --offline --no-fail-fast -p alpha --bin probe --test d|" ]] || ! grep -qF '补丁改到 crates/alpha/Cargo.toml（[[test]] 段 d：新加）' "$out" \
     || ! grep -qF '补丁改到 crates/alpha/Cargo.toml（[[bin]] 段 probe：新加）' "$out" || grep -qF '判不出' "$out" || grep -qF '顶掉' "$out"; then
    echo "  ✗ 自检：挑测试目标：补丁只在 alpha 的 Cargo.toml 里新加两段没写 path 的目录形目标（[[bin]] probe、[[test]] d，根文件 src/bin/probe/main.rs、tests/d/main.rs）：只该起 --bin probe --test d、不退 3，实际退出码 $rc，假 cargo 收到 $(narrow_tests manifest-directory-form)：$(grep -E '判不出|顶掉|补丁改到|✗' "$out" | head -3 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  # 树里的 Cargo.toml 留着上一格的新侧（两段都在）
  narrow_patch "$scratch/narrow-directory-form-body" crates/alpha/tests/d/main.rs crates/alpha/src/bin/probe/main.rs crates/alpha/tests/d/part.rs crates/alpha/src/bin/probe/part.rs
  out="$scratch/out-narrow-directory-form-body.txt"; narrow_case directory-form-body --skip-apply "$scratch/narrow-directory-form-body"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests directory-form-body)" != "test --offline --no-fail-fast -p alpha --bin probe --test d|" ]] || ! grep -qF '补丁改到 crates/alpha/tests/d/main.rs' "$out" \
     || ! grep -qF '补丁改到 crates/alpha/src/bin/probe/main.rs' "$out" || ! grep -qF '补丁改到 crates/alpha/tests/d/part.rs' "$out" || ! grep -qF '补丁改到 crates/alpha/src/bin/probe/part.rs' "$out" || grep -qF '判不出' "$out"; then
    echo "  ✗ 自检：挑测试目标：段已写在 Cargo.toml 里、补丁只改目录形根文件 tests/d/main.rs 与 src/bin/probe/main.rs 的正文、与它们同目录的子模块 part.rs（谁也没 mod 它）：只该起 --bin probe --test d、不退 3，实际退出码 $rc，假 cargo 收到 $(narrow_tests directory-form-body)：$(grep -E '判不出|补丁改到|✗' "$out" | head -3 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  rm -rf -- "${narrow_tree:?}/crates/alpha/tests/d"
  # 没写 edition 的包 cargo 按 2015 算：写了一段 [[test]]、autotests 又没写，cargo 不再自动认别的测试目标。没写 edition 的 alpha 新加一段写了 path 的 [[test]] extra：
  #   自动认的 case_a 从此不跑，判不出、退 3，原因点名 case_a 与 edition 2015
  manifest_patch "$scratch/narrow-manifest-edition" '' $'\n[[test]]\nname = "extra"\npath = "checks/extra.rs"\n' $'[package]\nname = "alpha"\n'
  out="$scratch/out-narrow-manifest-edition.txt"; narrow_case manifest-edition --skip-apply "$scratch/narrow-manifest-edition"; checked=$((checked+1))
  if [[ $rc -ne 3 || -n "$(narrow_tests manifest-edition)" ]] || ! grep -qF '补丁改到 alpha 包里的 Cargo.toml，判不出改到哪个测试目标' "$out" \
     || ! grep -qF '自动认的目标被顶掉，cargo test 从此不跑它的根文件、一句不报：test 目标 case_a（自动认的，根文件 tests/case_a.rs）改后它的根文件不再是任何 [[test]] 目标的根文件（包的 edition 照 cargo 算是 2015，写了 [[test]] 段就不再自动认同一种目标）' "$out"; then
    echo "  ✗ 自检：判不出测试目标：没写 edition 的 alpha（cargo 按 2015 算）在 Cargo.toml 里新加一段写了 path 的 [[test]] extra：cargo 从此不再自动认 case_a，测试那一步不跑、退 3，原因点名被顶掉的 case_a 与 edition 2015，实际退出码 $rc，假 cargo 收到 $(narrow_tests manifest-edition)：$(grep -E '判不出|顶掉|✗' "$out" | head -2 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  manifest_restore
  # 先验后合（文件头「先验后合」）：不带 --skip-apply、留着快照（MERGE_AND_VERIFY_KEEP_COPY=1）的那几趟，补丁照真打的样子打进快照（applying-apply.py：git apply 那份 crates.patch、
  #   变异表按名字把 mutations-replacements.tsv 那一行的前六段（改后的行；后六段是前像，这里不比对）换进去、删掉 mutations-delete.txt 那一行第一段的名字、追加 mutations-append.tsv 的行；打进主工作区那一种（不带 --root）自证不起）。样本树 merge-tree：工作区 demo、alpha、beta（dev 依赖 alpha）、delta（dev 依赖 demo）与验证包，
  #   几份 merge_*.txt 与 docs/merge-notes.txt 的内容是 1，变异表里一行 merge-kept-row；fresh_merge_tree 在每组格之前重建
  cat > "$scratch/applying-apply.py" <<'FAKE'
import os
import subprocess
import sys
directory = sys.argv[1]
root = sys.argv[sys.argv.index("--root") + 1] if "--root" in sys.argv else None
print("root=" + (root or "-"))
if root is None:
    sys.exit(0)
patch = os.path.join(directory, "crates.patch")
if os.path.isfile(patch) and os.path.getsize(patch) > 0 and subprocess.run(["git", "-C", root, "apply", patch]).returncode != 0:
    sys.exit(2)
table_path = os.path.join(root, "crates", "mutations.tsv")
replaced_path, deleted_path = os.path.join(directory, "mutations-replacements.tsv"), os.path.join(directory, "mutations-delete.txt")
if os.path.isfile(replaced_path) or os.path.isfile(deleted_path):
    replacing, deleting = {}, set()
    if os.path.isfile(replaced_path):
        with open(replaced_path, encoding="utf-8") as source:
            replacing = {line.split("\t", 1)[0]: "\t".join(line.split("\t")[:6]) for line in source.read().split("\n") if line.strip()}
    if os.path.isfile(deleted_path):
        with open(deleted_path, encoding="utf-8") as source:
            deleting = {line.split("\t", 1)[0] for line in source.read().split("\n") if line.strip()}
    with open(table_path, encoding="utf-8") as table:
        rows = [line for line in table.read().split("\n") if line]
    rows = [replacing.get(line.split("\t", 1)[0], line) for line in rows if line.split("\t", 1)[0] not in deleting]
    with open(table_path, "w", encoding="utf-8") as table:
        table.write("".join(line + "\n" for line in rows))
appended = os.path.join(directory, "mutations-append.tsv")
if os.path.isfile(appended):
    with open(appended, encoding="utf-8") as source, open(os.path.join(root, "crates", "mutations.tsv"), "a", encoding="utf-8") as table:
        table.write(source.read())
sys.exit(0)
FAKE
  local merge_tree="$scratch/merge-tree" merge_member merge_logs merge_patch_dir merge_apply_text merge_apply_at merge_summary_at merge_check_line merge_check_at merge_rows_line recheck_same_rc formal_case
  local merge_check_rc=missing merge_changed_rc=missing merge_added_rc=missing merge_demo_rc=missing merge_delta_rc=missing merge_rows_same_rc=missing merge_rows_changed_rc=missing
  fresh_merge_tree() {   # <树>：删掉重建
    rm -rf -- "${1:?}"; mkdir -p "$1/docs"
    printf '[workspace]\nmembers = ["crates/demo", "crates/alpha", "crates/beta", "crates/delta", "crates/singlefs-verification"]\n' > "$1/Cargo.toml"
    for merge_member in demo alpha beta delta singlefs-verification; do
      mkdir -p "$1/crates/$merge_member/src" "$1/crates/$merge_member/tests"; printf 'pub fn f() {}\n' > "$1/crates/$merge_member/src/lib.rs"
      printf '[package]\nname = "%s"\n' "$merge_member" > "$1/crates/$merge_member/Cargo.toml"
    done
    printf '\n[dev-dependencies]\nalpha = { path = "../alpha" }\n' >> "$1/crates/beta/Cargo.toml"
    printf '\n[dev-dependencies]\ndemo = { path = "../demo" }\n' >> "$1/crates/delta/Cargo.toml"
    printf 'fn main() {}\n' > "$1/crates/alpha/tests/case_a.rs"
    printf '1\n' > "$1/crates/alpha/src/merge_table.txt"; printf '1\n' > "$1/crates/beta/src/merge_extra.txt"
    printf '1\n' > "$1/crates/demo/src/merge_demo.txt"; printf '1\n' > "$1/crates/delta/src/merge_delta.txt"; printf '1\n' > "$1/docs/merge-notes.txt"
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'merge-kept-row' crates/alpha/src/lib.rs 'pub fn f() {}' 'pub fn f() { }' '-p alpha --lib' f > "$1/crates/mutations.tsv"
  }
  merge_case() {   # $1 标签，其余是参数：样本树 merge-tree、留着快照（MERGE_CASE_KEEP_COPY=0 的那一趟不留）、照真打的样子打补丁
    FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-merge-$1.txt" PATH="$fake_cargo_dir:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/applying-apply.py" MERGE_AND_VERIFY_KEEP_COPY="${MERGE_CASE_KEEP_COPY:-1}" \
      MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-merge-$1" MERGE_AND_VERIFY_TREE="$merge_tree" env -u CARGO_TARGET_DIR bash "$0" "${@:2}" > "$out" 2>&1; rc=$?
  }
  merge_patch_text() { printf -- '--- a/%s\n+++ b/%s\n@@ -1 +1 @@\n-%s\n+%s\n' "$1" "$1" "$2" "$3"; }   # <路径> <被替换的行> <替换成的行>
  mkdir -p "$scratch/merge-head" "$scratch/merge-stacked" "$scratch/merge-demo" "$scratch/merge-delta" "$scratch/merge-rows" "$scratch/merge-untargeted"
  merge_patch_text crates/alpha/src/merge_table.txt 1 2 > "$scratch/merge-head/crates.patch"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'merge-head-row' crates/alpha/src/lib.rs 'pub fn f() {}' 'pub fn f() { }' '-p alpha --lib' f > "$scratch/merge-head/mutations-append.tsv"
  { merge_patch_text crates/alpha/src/merge_table.txt 2 3; merge_patch_text crates/beta/src/merge_extra.txt 1 2; } > "$scratch/merge-stacked/crates.patch"
  merge_patch_text crates/demo/src/merge_demo.txt 1 2 > "$scratch/merge-demo/crates.patch"
  merge_patch_text crates/delta/src/merge_delta.txt 1 2 > "$scratch/merge-delta/crates.patch"
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' 'merge-kept-row' crates/alpha/src/lib.rs 'pub fn f() {}' 'pub fn f() {  }' '-p alpha --lib' f \
    'merge-kept-row' crates/alpha/src/lib.rs 'pub fn f() {}' 'pub fn f() { }' '-p alpha --lib' f > "$scratch/merge-rows/mutations-replacements.tsv"
  merge_patch_text docs/merge-notes.txt 1 2 > "$scratch/merge-untargeted/crates.patch"
  # 验绿了：汇总行之后打第 2 步在快照上起的几条（--cwd 与根都是 <日志目录>/repo），再打合进主工作区那一段：一条 apply-writer-patch.py <日志目录>/merge-patch，不逐份打、不留底
  fresh_merge_tree "$merge_tree"
  out="$scratch/out-merge-in-green.txt"; merge_case in-green "$scratch/merge-head" "$scratch/merge-stacked"; checked=$((checked+1))
  merge_logs="$scratch/logs-merge-in-green"
  merge_apply_text="$(printf '%q ' "$root/task-server/task" submit -- python3 "$root/scripts/gate/apply-writer-patch.py" "$merge_logs/merge-patch")"
  merge_summary_at="$(grep -n '^  汇总：' "$out" | head -n 1 | cut -d: -f1)"
  merge_apply_at="$(grep -nxF "    ${merge_apply_text% }" "$out" | head -n 1 | cut -d: -f1)"
  if [[ $rc -ne 0 || -z "$merge_summary_at" || -z "$merge_apply_at" ]] || ((merge_summary_at > merge_apply_at)) || grep -qF 'copy-originals.py copy' "$out" \
     || grep -qF "apply-writer-patch.py $scratch/merge-head" "$out" || grep -qF "apply-writer-patch.py $scratch/merge-stacked" "$out" \
     || ! grep -qF "task-server/task submit --cwd $merge_logs/repo -- python3 scripts/gate/refused-test-registry.py list $merge_logs/repo" "$out" \
     || ! grep -qF "task-server/task submit --cwd $merge_logs/repo -- python3 scripts/crash/crash-case-check.py --only placement,modules,provenance,must-red $merge_logs/repo" "$out" \
     || ! grep -qF '「merge-head-row」' "$out"; then
    echo "  ✗ 自检：先验后合：验绿了应当打第 2 步在快照上起的几条与合进主工作区的整条命令（汇总行之后、快照上的 crash-case-check 与 refused-test-registry list 与新变异行名，再一条 apply-writer-patch.py <日志目录>/merge-patch，不逐份打、不留底），实际退出码 $rc，行号：汇总 ${merge_summary_at:-无}、合的那一条 ${merge_apply_at:-无}：$(grep -E '合进主工作区|apply-writer-patch|快照' "$out" | head -4 | tr '\n' ' ' | clip_text 600)"; failures=1; fi   # gate-lint:detail
  # 合进主工作区的那一份补丁目录：两份补丁合起来的样子（merge_table 1→3、merge_extra 1→2），基线是开跑那一刻快照里的原样，变异行按名字比出追加的那一行，打得上一棵没动过的样本树
  checked=$((checked+1)); merge_patch_dir="$merge_logs/merge-patch"
  fresh_merge_tree "$scratch/merge-pristine"
  if [[ ! -s "$merge_patch_dir/crates.patch" ]] || ! grep -qx '+3' "$merge_patch_dir/crates.patch" || ! grep -qF 'b/crates/beta/src/merge_extra.txt' "$merge_patch_dir/crates.patch" \
     || [[ "$(cat "$merge_patch_dir/baseline/crates/alpha/src/merge_table.txt" 2>/dev/null)" != 1 ]] || ! grep -q $'^merge-head-row\t' "$merge_patch_dir/mutations-append.tsv" 2>/dev/null \
     || ! git -C "$scratch/merge-pristine" apply --check "$merge_patch_dir/crates.patch" > "$scratch/merge-pristine-check.txt" 2>&1; then
    echo "  ✗ 自检：先验后合：合进主工作区的那一份补丁目录应当是几份补丁合起来的样子（merge_table 1→3、merge_extra 1→2，基线是开跑那一刻的快照、变异行按名字比出、打得上没动过的树），实际 merge-patch 里有 $(ls "$merge_patch_dir" 2>/dev/null | tr '\n' ' ')：$(cat -- "$scratch/merge-pristine-check.txt" 2>/dev/null | tr '\n' ' ' | clip_text 300)"; failures=1; fi   # gate-lint:detail
  # 合进主工作区那一段的头一条核主工作区自开跑以来没变（文件头「开跑那一刻的指纹」）：在汇总行与合的那一条之间，指到这一趟的指纹，主工作区没动时它退 0
  merge_check_line="$(grep -m 1 '^    find ' "$out" | sed 's/^    //')"
  merge_check_at="$(grep -n -m 1 '^    find ' "$out" | cut -d: -f1)"
  checked=$((checked+1))
  if [[ -n "$merge_check_line" ]]; then bash -c "$merge_check_line" > "$scratch/merge-check-unchanged.txt" 2>&1; merge_check_rc=$?; fi
  if [[ -z "$merge_check_line" || "$merge_check_rc" != 0 || -z "$merge_summary_at" || -z "$merge_apply_at" ]] || ((merge_check_at < merge_summary_at || merge_check_at > merge_apply_at)) \
     || [[ "$merge_check_line" != *"$merge_logs/main-tree-at-start.sha256"* ]]; then
    echo "  ✗ 自检：先验后合：合进主工作区那一段应当先打核主工作区自开跑以来没变的那一条，主工作区没动时它退 0，实际行号 ${merge_check_at:-无}（汇总 ${merge_summary_at:-无}、合的那一条 ${merge_apply_at:-无}），退 ${merge_check_rc}：$(cat -- "$scratch/merge-check-unchanged.txt" 2>/dev/null | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  # 同包里别的文件（alpha 的 tests/case_a.rs，补丁没改它）开跑之后改了、beta 的 src 里加了一份文件：核主工作区没变的那一条两趟都退非 0
  checked=$((checked+1))
  if [[ -n "$merge_check_line" ]]; then
    printf '// 开跑之后改的\n' >> "$merge_tree/crates/alpha/tests/case_a.rs"
    bash -c "$merge_check_line" > "$scratch/merge-check-changed.txt" 2>&1; merge_changed_rc=$?
    printf 'fn main() {}\n' > "$merge_tree/crates/alpha/tests/case_a.rs"
    printf 'pub fn added_after_start() {}\n' > "$merge_tree/crates/beta/src/added_after_start.rs"
    bash -c "$merge_check_line" > "$scratch/merge-check-added.txt" 2>&1; merge_added_rc=$?
    rm -f -- "$merge_tree/crates/beta/src/added_after_start.rs"
  fi
  if [[ "$merge_changed_rc" == missing || "$merge_changed_rc" == 0 || "$merge_added_rc" == missing || "$merge_added_rc" == 0 ]]; then
    echo "  ✗ 自检：先验后合：同包里别的文件开跑之后改了、加了，核主工作区没变的那一条应当退非 0，实际改了的那一趟退 ${merge_changed_rc}、加了的那一趟退 ${merge_added_rc}"; failures=1; fi   # gate-lint:detail
  # 依赖关系上的包（文件头「开跑那一刻的指纹」）：只改 demo 的那一趟，依赖 demo 的 delta 开跑之后改了；只改 delta 的那一趟，delta 依赖的 demo 开跑之后改了：核那一条都退非 0
  fresh_merge_tree "$merge_tree"
  out="$scratch/out-merge-demo.txt"; merge_case demo "$scratch/merge-demo"
  merge_check_line="$(grep -m 1 '^    find ' "$out" | sed 's/^    //')"
  if [[ -n "$merge_check_line" ]]; then
    printf '// 开跑之后改的\n' >> "$merge_tree/crates/delta/src/lib.rs"
    bash -c "$merge_check_line" > "$scratch/merge-check-demo.txt" 2>&1; merge_demo_rc=$?
  fi
  fresh_merge_tree "$merge_tree"
  out="$scratch/out-merge-delta.txt"; merge_case delta "$scratch/merge-delta"
  merge_check_line="$(grep -m 1 '^    find ' "$out" | sed 's/^    //')"
  if [[ -n "$merge_check_line" ]]; then
    printf '// 开跑之后改的\n' >> "$merge_tree/crates/demo/src/lib.rs"
    bash -c "$merge_check_line" > "$scratch/merge-check-delta.txt" 2>&1; merge_delta_rc=$?
  fi
  checked=$((checked+1))
  if [[ "$merge_demo_rc" == missing || "$merge_demo_rc" == 0 || "$merge_delta_rc" == missing || "$merge_delta_rc" == 0 ]]; then
    echo "  ✗ 自检：先验后合：依赖关系上的包开跑之后改了，核主工作区没变的那一条应当退非 0（只改 demo 的那一趟改 delta、只改 delta 的那一趟改 demo），实际 ${merge_demo_rc}、${merge_delta_rc}；两趟的合进主工作区那一段：只改 demo 的「$(grep -E '合进主工作区|没有改动|一份都没有|^    find ' "$scratch/out-merge-demo.txt" 2>/dev/null | head -3 | clip_text 120 | tr '\n' ' ')」、只改 delta 的「$(grep -E '合进主工作区|没有改动|一份都没有|^    find ' "$scratch/out-merge-delta.txt" 2>/dev/null | head -3 | clip_text 120 | tr '\n' ' ')」"; failures=1; fi   # gate-lint:detail
  # 这一批要换掉的变异行（merge-rows 换 merge-kept-row）：开跑之后表里只追加别的会话的行，核变异行的那一条退 0；那一行被改了退非 0
  fresh_merge_tree "$merge_tree"
  out="$scratch/out-merge-rows.txt"; merge_case rows "$scratch/merge-rows"
  merge_rows_line="$(grep -m 1 '^    LC_ALL=C grep -xF' "$out" | sed 's/^    //')"
  if [[ -n "$merge_rows_line" ]]; then
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'other-session-row' crates/demo/src/lib.rs 'pub fn f() {}' 'pub fn f() { }' '-p demo --lib' f >> "$merge_tree/crates/mutations.tsv"
    bash -c "$merge_rows_line" > "$scratch/merge-rows-same.txt" 2>&1; merge_rows_same_rc=$?
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'merge-kept-row' crates/alpha/src/lib.rs 'pub fn f() {}' 'pub fn f() {   }' '-p alpha --lib' f > "$merge_tree/crates/mutations.tsv"
    bash -c "$merge_rows_line" > "$scratch/merge-rows-changed.txt" 2>&1; merge_rows_changed_rc=$?
  fi
  checked=$((checked+1))
  if [[ "$merge_rows_same_rc" != 0 || "$merge_rows_changed_rc" == missing || "$merge_rows_changed_rc" == 0 ]]; then
    echo "  ✗ 自检：先验后合：这一批要换掉的变异行开跑之后被改了，核变异行的那一条应当退非 0（只追加了别的行的那一趟退 0），实际 ${merge_rows_same_rc}、${merge_rows_changed_rc}：$(grep -E '变异行|✗' "$out" | head -2 | tr '\n' ' ' | clip_text 300)"; failures=1; fi   # gate-lint:detail
  # 判不出测试目标（补丁改到不落在包里的 docs/merge-notes.txt，退 3）：照样打合进主工作区的命令，先写一句退 3 那一支
  fresh_merge_tree "$merge_tree"
  out="$scratch/out-merge-untargeted.txt"; merge_case untargeted "$scratch/merge-untargeted"; checked=$((checked+1))
  if [[ $rc -ne 3 ]] || ! grep -qF "apply-writer-patch.py $scratch/logs-merge-untargeted/merge-patch" "$out" || ! grep -qF '这一趟退 3' "$out"; then
    echo "  ✗ 自检：先验后合：判不出测试目标（退 3）的那一趟也应当打合进主工作区的命令、写明退 3 那一支，实际退出码 $rc：$(grep -E '退 3|合进主工作区|apply-writer-patch' "$out" | head -3 | tr '\n' ' ' | clip_text 400)；补丁目录里有「$(ls "$scratch/logs-merge-untargeted/merge-patch" 2>/dev/null | tr '\n' ' ')」，动到的路径「$(tr '\n' ' ' < "$scratch/logs-merge-untargeted/$MERGE_BASELINE_PATHS_NAME" 2>/dev/null)」，样本树里 docs/merge-notes.txt 是「$(tr '\n' ' ' < "$merge_tree/docs/merge-notes.txt" 2>/dev/null)」"; failures=1; fi   # gate-lint:detail
  # 变异锚点核（文件头「先验后合」变异锚点核）：样本树 merge-tree 里放 ANCHOR_CHECK_SAMPLE_MARK 标记（自证里只在带标记的树上做），static-check 换成 fake-anchor-static-check.py
  #   （照变异表逐行数原文在文件里命中几次，不是恰好 1 次的打一行 ANCHOR，与 crates-mutation-rows.py static-check 那一行同形；apply-writer-patch.py 的 static_check_report 与摆树都是真的）。
  #   anchor-break：表里别批的一行 merge-data-row 锚在 alpha 的 merge_table.txt 里的原文 1，补丁把那个 1 改成 2：判红、退 1，点名行名、文件与命中 0 次，不打合进主工作区的命令；
  #   anchor-pre-broken：表里另有一行 merge-old-broken-row 补丁之前锚点就断了（原文在 alpha 的 lib.rs 里命中 0 次），补丁（merge-head）与它无关：不判红、退 0，只报补丁前就红的 1 处
  local merge_anchor_apply_text
  cat > "$scratch/fake-anchor-static-check.py" <<'FAKE'
import os
import sys
tree = sys.argv[2]
table = os.path.join(tree, "crates", "mutations.tsv")
rows = []
if os.path.isfile(table):
    with open(table, encoding="utf-8") as handle:
        rows = [line.split("\t") for line in handle.read().split("\n") if line.strip() and not line.startswith("#")]
print("COPIED\tcrates")
red = 0
for number, columns in enumerate(rows, 1):
    path = os.path.join(tree, columns[1])
    text = ""
    if os.path.isfile(path):
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
    count = text.count(columns[2])
    if count != 1:
        red += 1
        print(f"ANCHOR\t{number}\t{columns[0]}\t原文在 {columns[1]} 里命中 {count} 次，要恰好 1 次")
print(f"CHECKED\t{len(rows)}\t1\t0")
sys.exit(1 if red else 0)
FAKE
  export MERGE_AND_VERIFY_STATIC_CHECK="$scratch/fake-anchor-static-check.py"
  mkdir -p "$scratch/merge-anchor-break"
  merge_patch_text crates/alpha/src/merge_table.txt 1 2 > "$scratch/merge-anchor-break/crates.patch"
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'merge-data-row' crates/alpha/src/merge_table.txt 1 9 '-p alpha --lib' f >> "$merge_tree/crates/mutations.tsv"
  out="$scratch/out-merge-anchor-break.txt"; merge_case anchor-break "$scratch/merge-anchor-break"; checked=$((checked+1))
  merge_anchor_apply_text="$(printf '%q ' "$root/task-server/task" submit -- python3 "$root/scripts/gate/apply-writer-patch.py" "$scratch/logs-merge-anchor-break/merge-patch")"
  if [[ $rc -ne 1 ]] || ! grep -qE 'ANCHOR [0-9]+ merge-data-row 原文在 crates/alpha/src/merge_table\.txt 里命中 0 次' "$out" || grep -qF 'merge-kept-row 原文在' "$out" \
     || grep -qxF "    ${merge_anchor_apply_text% }" "$out" || ! grep -qF 'mutations-delete.txt' "$out"; then
    echo "  ✗ 自检：变异锚点核：补丁弄断别批变异行的锚点应当判红、退 1，点名行名 merge-data-row、文件 crates/alpha/src/merge_table.txt 与命中 0 次（不点名没弄断的 merge-kept-row），不打合进主工作区的命令，出路写锚着的代码被整段删掉的改写进 mutations-delete.txt，实际退出码 $rc：$(grep -E '变异锚点核|ANCHOR|合进主工作区' "$out" | head -4 | tr '\n' ' ' | clip_text 500)"; failures=1; fi   # gate-lint:detail
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'merge-old-broken-row' crates/alpha/src/lib.rs 'pub fn missing() {}' 'pub fn missing() { }' '-p alpha --lib' f >> "$merge_tree/crates/mutations.tsv"
  out="$scratch/out-merge-anchor-pre-broken.txt"; merge_case anchor-pre-broken "$scratch/merge-head"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -qF '变异锚点核过' "$out" || ! grep -qF '补丁前就红的 1 处' "$out" || grep -qE 'ANCHOR [0-9]+ merge-old-broken-row' "$out"; then
    echo "  ✗ 自检：变异锚点核：补丁之前就失效的变异行不算这一批弄断的，不相干的补丁应当照放行、退 0、报变异锚点核过与补丁前就红的 1 处、不点名 merge-old-broken-row，实际退出码 $rc：$(grep -E '变异锚点核|ANCHOR|补丁前就红' "$out" | head -4 | tr '\n' ' ' | clip_text 500)"; failures=1; fi   # gate-lint:detail
  # 变异锚点核的几格（样本树都放 ANCHOR_CHECK_SAMPLE_MARK，static-check 是上面那份 fake-anchor-static-check.py；MERGE_AND_VERIFY_BREAK 的开关名在文件头开关表）：
  #   own-only、own-with-code：这一批自己追加的变异行锚点不中（只改变异表、没有 crates.patch；连代码一起改），判红、退 1、点名那一行
  #   created：补丁只新建一份文件、不碰变异行，核过、退 0
  #   release-shift：补丁之前就红的 RELEASE 行只因这一批删了它前面一行、行号前移，不算新红：退 0、报补丁前就红的 1 处
  #   skip-apply、skip-apply-own-only：带 --skip-apply 的那一趟（补丁已写在样本树上）补丁弄断别批变异行的锚点、自己追加的行锚点不中，照样判红、退 1
  #   skip-apply-replacement、skip-apply-irreversible、skip-apply-no-directories：带 --skip-apply 的那一趟补丁目录只有一行替换（改后的行锚点不中）判红、追加的行之后被改过倒不回记红、没给补丁目录打一行本次未核
  #   many-reds：新红多过标准输出列的行数，日志里全有、标准输出写明一共几处；no-keep：没留快照的那一趟出路写带 MERGE_AND_VERIFY_KEEP_COPY=1 重交
  local anchor_case anchor_label anchor_total anchor_index anchor_static_saved anchor_original anchor_replaced dry_printed dry_rc dry_out dry_case dry_label dry_edited dry_package
  # 合入前的 --dry-run 的几格（dry-concurrent、dry-mirror、dry-own-mutonly、dry-own-with-code、dry-own-release、dry-register、dry-same-file；文件头「先验后合」合入前的 --dry-run）：
  #   不带 --skip-apply 的一趟照常验绿、打出合进主工作区的命令，之后「开跑之后别的会话」改了样本树，
  #   合入前的 --dry-run 那一条（merge_dry_run 照它的样子跑：apply-writer-patch.py 的 apply 在样本树上 --dry-run，static-check 换成假的）应当在合入那一刻的样本树上拦住新弄断的、放行不受影响的
  #   dry-run-driver.py：python3 <apply-writer-patch.py> <补丁目录> <树> <static-check 的假脚本>，在树上照 apply-writer-patch.py <补丁目录> --dry-run 的样子跑一遍（核补丁与变异表、套上补丁的临时树与没打补丁的树各跑一次 static-check），打它的输出、退它的退出码；
  #   fake-registered-static-check.py：照变异表逐行数原文命中几次（不是恰好 1 次的打 ANCHOR），另照被判那棵树的 .claude/gate.d/stage-inputs.tsv 里 crash-case: 行的 test=<包>:<测试目标>:… 给第五段点名那个包与 --test 目标、没写 --release 的行打 RELEASE
  cat > "$scratch/dry-run-driver.py" <<'FAKE'
import importlib.util
import sys
specification = importlib.util.spec_from_file_location("apply_writer_patch_for_dry_run", sys.argv[1])
apply_module = importlib.util.module_from_spec(specification)
specification.loader.exec_module(apply_module)
patch_directory, tree, replacement_command = sys.argv[2:5]
code, lines = apply_module.apply(patch_directory, tree, True, ["true"], static_check=lambda checked_tree: [sys.executable or "python3", replacement_command, "static-check", checked_tree])
print("\n".join(lines))
sys.exit(code)
FAKE
  cat > "$scratch/fake-registered-static-check.py" <<'FAKE'
import os
import re
import sys
tree = sys.argv[2]


def read(path):
    try:
        with open(path, encoding="utf-8") as handle:
            return handle.read()
    except OSError:
        return ""


registered = set(re.findall(r"test=([^:\s]+):([^:\s]+):", read(os.path.join(tree, ".claude", "gate.d", "stage-inputs.tsv"))))
rows = [line.split("\t") for line in read(os.path.join(tree, "crates", "mutations.tsv")).split("\n") if line.strip() and not line.startswith("#")]
print("COPIED\tcrates")
red = 0
for number, columns in enumerate(rows, 1):
    count = read(os.path.join(tree, columns[1])).count(columns[2])
    if count != 1:
        red += 1
        print(f"ANCHOR\t{number}\t{columns[0]}\t原文在 {columns[1]} 里命中 {count} 次，要恰好 1 次")
    words = columns[4].split() if len(columns) > 4 else []
    package = words[words.index("-p") + 1] if "-p" in words[:-1] else None
    target = words[words.index("--test") + 1] if "--test" in words[:-1] else None
    if (package, target) in registered and "--release" not in words:
        red += 1
        print(f"RELEASE\t{number}\t{columns[0]}\tcrash-case 目标 {package}:{target}（--test 点名；登记 crash-case）")
print(f"CHECKED\t{len(rows)}\t1\t0")
sys.exit(1 if red else 0)
FAKE
  merge_dry_run() {   # $1 标签（merge_case 的）、$2 static-check 的假脚本：$out 里合入前的 --dry-run 那一整条打出来了、排在合的那一条之前的 dry_printed 记 1，再在样本树此刻的样子上跑一遍，退出码进 dry_rc、输出进 dry_out
    local dry_text apply_text dry_at apply_at
    dry_text="$(printf '%q ' "$root/task-server/task" submit -- python3 "$root/scripts/gate/apply-writer-patch.py" --dry-run "$scratch/logs-merge-$1/merge-patch")"
    apply_text="$(printf '%q ' "$root/task-server/task" submit -- python3 "$root/scripts/gate/apply-writer-patch.py" "$scratch/logs-merge-$1/merge-patch")"
    dry_at="$(grep -nxF "    ${dry_text% }" "$out" | head -n 1 | cut -d: -f1)"; apply_at="$(grep -nxF "    ${apply_text% }" "$out" | head -n 1 | cut -d: -f1)"
    dry_printed=0
    if [[ -n "$dry_at" && -n "$apply_at" ]] && ((dry_at < apply_at)); then dry_printed=1; fi
    dry_out="$scratch/dry-run-$1.txt"
    git init -q "$merge_tree" > "$scratch/dry-run-git-init-$1.txt" 2>&1   # 主工作区是 git 工作树：apply-writer-patch.py 在 git 仓上判换上到一半的暂存目录与对象库，样本树也照这样
    # 弄坏开关 dry-run-baseline-on-live-root、dry-run-outside-files-not-copied：经 APPLY_WRITER_PATCH_BREAK 交给 apply-writer-patch.py 同名的两条（只给这一条驱动）
    (
      case "${MERGE_AND_VERIFY_BREAK:-}" in
        dry-run-baseline-on-live-root) export APPLY_WRITER_PATCH_BREAK=static-check-baseline-on-live-root ;;
        dry-run-outside-files-not-copied) export APPLY_WRITER_PATCH_BREAK=static-check-outside-files-not-copied ;;
      esac
      python3 "$scratch/dry-run-driver.py" "$APPLY_WRITER_PATCH_LIBRARY" "$scratch/logs-merge-$1/merge-patch" "$merge_tree" "$2"
    ) > "$dry_out" 2>&1; dry_rc=$?
  }
  dry_run_blocked() {   # $1 标签、$2 格名、$3 变异名、$4 红的种类（ANCHOR 或 RELEASE）、$5 static-check 的假脚本、$6 格名后面接在「应当拦住」之前的说明（可空）：--dry-run 退 2、点名 $3 的 $4 行
    merge_dry_run "$1" "$5"
    if [[ $rc -ne 0 || $dry_printed -ne 1 ]]; then
      echo "  ✗ 自检：合入前的 --dry-run：$2${6:-}应当拦住：这一趟应当退 0、合进主工作区那一段里打出合入前的 --dry-run 且排在合的那一条之前，实际退 $rc、打出并排在前面的记 $dry_printed：$(grep -E '合进主工作区|--dry-run|✗' "$out" | head -n 3 | tr '\n' ' ' | clip_text 300)"; failures=1   # gate-lint:detail
    elif [[ $dry_rc -ne 2 ]] || ! grep -qE "$4 [0-9]+ $3 " "$dry_out"; then
      echo "  ✗ 自检：合入前的 --dry-run：$2${6:-}应当拦住：开跑之后别的会话改了样本树，合进去 $3 的 $4 行会红，--dry-run 应当退 2 并点名它，实际退 $dry_rc：$(head -n 4 "$dry_out" | tr '\n' ' ' | clip_text 300)"; failures=1   # gate-lint:detail
    fi
  }
  # 别的会话开跑之后往主工作区变异表追加一行、锚在这一批改到的文件（alpha 的 merge_table.txt）的原文上，这一批把那个原文改没了
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  out="$scratch/out-merge-dry-concurrent.txt"; merge_case dry-concurrent "$scratch/merge-anchor-break"; checked=$((checked+1))
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'other-session-row' crates/alpha/src/merge_table.txt 1 9 '-p alpha --lib' f >> "$merge_tree/crates/mutations.tsv"
  dry_run_blocked dry-concurrent '别的会话开跑之后追加的、锚在这一批改到的文件上的变异行' other-session-row ANCHOR "$scratch/fake-anchor-static-check.py"
  # 镜像方向：这一批只追加一行变异行（只改变异表、没有 crates.patch），锚着的 alpha 的 merge_table.txt 开跑之后被别的会话改了
  mkdir -p "$scratch/merge-anchor-mirror"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'mirror-own-row' crates/alpha/src/merge_table.txt 1 9 '-p alpha --lib' f > "$scratch/merge-anchor-mirror/mutations-append.tsv"
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  out="$scratch/out-merge-dry-mirror.txt"; merge_case dry-mirror "$scratch/merge-anchor-mirror"; checked=$((checked+1))
  printf '2\n' > "$merge_tree/crates/alpha/src/merge_table.txt"
  dry_run_blocked dry-mirror '这一批只追加一行变异行、锚着的文件开跑之后被别的会话改了' mirror-own-row ANCHOR "$scratch/fake-anchor-static-check.py"
  # 这一批自己追加的行锚在补丁没改到的文件上，开跑之后别的会话改了那份文件：mutonly 只有 mutations-append.tsv（锚在 beta 的 merge_extra.txt），
  #   with-code 另改 alpha 的 merge_table.txt、追加的行锚在 demo 的 merge_demo.txt（demo 与 alpha 没有依赖关系，指纹那一条也罩不到）
  for dry_case in "mutonly:只改变异表" "with-code:连代码一起改"; do
    dry_label="${dry_case#*:}"; dry_case="${dry_case%%:*}"
    fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
    rm -rf -- "${scratch:?}/merge-dry-own-$dry_case"; mkdir -p "$scratch/merge-dry-own-$dry_case"
    if [[ "$dry_case" == mutonly ]]; then
      dry_edited=crates/beta/src/merge_extra.txt; dry_package=beta
    else
      dry_edited=crates/demo/src/merge_demo.txt; dry_package=demo
      merge_patch_text crates/alpha/src/merge_table.txt 1 2 > "$scratch/merge-dry-own-$dry_case/crates.patch"
    fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "dry-own-$dry_case-row" "$dry_edited" 1 9 "-p $dry_package --lib" f > "$scratch/merge-dry-own-$dry_case/mutations-append.tsv"
    out="$scratch/out-merge-dry-own-$dry_case.txt"; merge_case "dry-own-$dry_case" "$scratch/merge-dry-own-$dry_case"; checked=$((checked+1))
    printf '2\n' > "$merge_tree/$dry_edited"
    dry_run_blocked "dry-own-$dry_case" '这一批自己追加的行锚在补丁没改到的文件上、开跑之后别的会话改了那份文件' "dry-own-$dry_case-row" ANCHOR "$scratch/fake-anchor-static-check.py" "（$dry_label）"
  done
  # 登记表：样本树另有一份空的 .claude/gate.d/stage-inputs.tsv 与 beta 的 tests/case_b.rs，两样都带一处 alpha 的代码改动（有测试目标可跑）。
  #   dry-own-release：这一批追加一行点名 beta 的 case_b、没写 --release，开跑之后别的会话把 case_b 登记成崩溃枚举用例；
  #   dry-register：这一批把 case_b 登记成崩溃枚举用例，开跑之后别的会话追加一行点名 case_b、锚在补丁没改到的文件上
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  mkdir -p "$merge_tree/.claude/gate.d"; : > "$merge_tree/.claude/gate.d/stage-inputs.tsv"; printf 'fn main() {}\n' > "$merge_tree/crates/beta/tests/case_b.rs"
  rm -rf -- "${scratch:?}/merge-dry-own-release"; mkdir -p "$scratch/merge-dry-own-release"
  merge_patch_text crates/alpha/src/merge_table.txt 1 2 > "$scratch/merge-dry-own-release/crates.patch"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' dry-own-release-row crates/beta/src/merge_extra.txt 1 9 '-p beta --test case_b' f > "$scratch/merge-dry-own-release/mutations-append.tsv"
  out="$scratch/out-merge-dry-own-release.txt"; merge_case dry-own-release "$scratch/merge-dry-own-release"; checked=$((checked+1))
  printf 'crash-case:dry-case\t-\ttest=beta:case_b:case_function\n' >> "$merge_tree/.claude/gate.d/stage-inputs.tsv"
  dry_run_blocked dry-own-release '这一批追加的行点名的测试目标开跑之后被别的会话登记成崩溃枚举用例' dry-own-release-row RELEASE "$scratch/fake-registered-static-check.py"
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  mkdir -p "$merge_tree/.claude/gate.d"; : > "$merge_tree/.claude/gate.d/stage-inputs.tsv"; printf 'fn main() {}\n' > "$merge_tree/crates/beta/tests/case_b.rs"
  rm -rf -- "${scratch:?}/merge-dry-register"; mkdir -p "$scratch/merge-dry-register"
  printf 'diff --git a/crates/alpha/src/merge_table.txt b/crates/alpha/src/merge_table.txt\n--- a/crates/alpha/src/merge_table.txt\n+++ b/crates/alpha/src/merge_table.txt\n@@ -1 +1 @@\n-1\n+2\ndiff --git a/.claude/gate.d/stage-inputs.tsv b/.claude/gate.d/stage-inputs.tsv\n--- a/.claude/gate.d/stage-inputs.tsv\n+++ b/.claude/gate.d/stage-inputs.tsv\n@@ -0,0 +1 @@\n+crash-case:dry-case\t-\ttest=beta:case_b:case_function\n' > "$scratch/merge-dry-register/crates.patch"
  out="$scratch/out-merge-dry-register.txt"; merge_case dry-register "$scratch/merge-dry-register"; checked=$((checked+1))
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' dry-other-release-row crates/beta/src/merge_extra.txt 1 9 '-p beta --test case_b' f >> "$merge_tree/crates/mutations.tsv"
  dry_run_blocked dry-register '这一批登记崩溃枚举用例、开跑之后别的会话追加了点名它的变异行' dry-other-release-row RELEASE "$scratch/fake-registered-static-check.py"
  # 同一份文件里不受影响的追加：表里别批两行锚在 merge_table.txt 的原文 1 上，这一批在 1 后面加一行 3（两行照样恰好命中 1 次），开跑之后别的会话又追加一行锚在同一份文件上：放行
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' dry-zz-row crates/alpha/src/merge_table.txt 1 9 '-p alpha --lib' f dry-aa-row crates/alpha/src/merge_table.txt 1 8 '-p alpha --lib' f >> "$merge_tree/crates/mutations.tsv"
  rm -rf -- "${scratch:?}/merge-dry-same-file"; mkdir -p "$scratch/merge-dry-same-file"
  printf -- '--- a/crates/alpha/src/merge_table.txt\n+++ b/crates/alpha/src/merge_table.txt\n@@ -1 +1,2 @@\n 1\n+3\n' > "$scratch/merge-dry-same-file/crates.patch"
  out="$scratch/out-merge-dry-same-file.txt"; merge_case dry-same-file "$scratch/merge-dry-same-file"; checked=$((checked+1))
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' dry-other-same-file-row crates/alpha/src/merge_table.txt 1 7 '-p alpha --lib' f >> "$merge_tree/crates/mutations.tsv"
  merge_dry_run dry-same-file "$scratch/fake-anchor-static-check.py"
  if [[ $rc -ne 0 || $dry_printed -ne 1 ]]; then
    echo "  ✗ 自检：合入前的 --dry-run：同一份文件里不受影响的追加应当放行：这一趟应当退 0、合进主工作区那一段里打出合入前的 --dry-run 且排在合的那一条之前，实际退 $rc、打出并排在前面的记 $dry_printed：$(grep -E '合进主工作区|--dry-run|✗' "$out" | head -n 3 | tr '\n' ' ' | clip_text 300)"; failures=1   # gate-lint:detail
  elif [[ $dry_rc -ne 0 ]] || ! grep -qF '核过了（--dry-run' "$dry_out"; then
    echo "  ✗ 自检：合入前的 --dry-run：同一份文件里不受影响的追加应当放行：这一批没有弄断任何一行的锚点，--dry-run 应当退 0、报核过了，实际退 $dry_rc：$(head -n 4 "$dry_out" | tr '\n' ' ' | clip_text 300)"; failures=1   # gate-lint:detail
  fi
  # 合入前的 --dry-run 与合进主工作区那一段的 ①到⑤（merge_in_commands；文件头「合入前的 --dry-run」）：样本都照打出的次序做——先跑 --dry-run 之前打出的 ①，再在样本树上跑 --dry-run（merge_dry_run），
  #   「开跑之后别的会话」改了样本树之后，跑 --dry-run 之后、合的那一条之前打出的只读核（③ 与 ④）；打出的核逐条在样本树此刻的样子上起一遍（日志与退出码都看）。这一段的格：
  #   window-fingerprint：--dry-run 跑着的时候别的会话改了这一批改到的包里的别的文件（alpha 的 tests/case_a.rs）：④ 核主工作区没变的那一条排在 --dry-run 之后、此刻退非 0
  #   window-table-append：--dry-run 跑完之后别的会话往变异表追加一行锚在这一批改到的 merge_table.txt 的原文 1 上（这一批把它改成 2）：③ 照 ① 重算再比、退非 0
  #   window-unchanged：主工作区没变：① 记下、③ 比对、④ 核主工作区没变都退 0（上面两格退非 0 不是因为这几条本来就红）；这一趟的输出另判三格：
  #     redo-loop：合的那一条退 2 等完（开工快照、同一个仓有别的在换）与三方合并冲突、核与换之间被改了的出路都写从 ① 重做到 ⑤
  #     dry-run-way-out-outcomes：合入前的 --dry-run 那一条的出路写判不了、没做成、不是 2 的结局（被停、撞内存上限）怎么办，写整批重交与从 ① 重做
  #     dry-run-way-out-cost：出路写明代价（约 400–740 秒、内存峰值约 15 GiB）
  #   dry-transient：别的会话开跑之后追加了一行锚在这一批改到的 merge_table.txt 的原文 1 上（这一批把它改成 2）；--dry-run 跑补丁之后那一棵的 static-check 时别的会话临时把主工作区的 merge_table.txt 改成 5，
  #     没打补丁那一棵跑完又改回 1：没打补丁那一棵是同一刻拷出的树，这一行补丁之前命中 1 次、补丁之后 0 次，--dry-run 应当退 2 点名它
  #   dry-skew：表里一行补丁之前就红的变异行（原文 7 在 beta 的 merge_extra.txt 里命中 0 次）；--dry-run 跑补丁之后那一棵的 static-check 时别的会话把它的原文重取成 1（修好了）：
  #     补丁之前那一棵是同一刻拷出的树，这一行在两棵树里都红，不是这一批弄断的，--dry-run 应当退 0
  #   dry-outside：合并补丁带拷贝范围之外的文件段（拒算表 .claude/gate.d/refused-tests.tsv 一行）：--dry-run 照常判、退 0 报核过了，不退没做成
  #   dry-transient-static-check.py：照 dry-transient-plan.txt（模式、主工作区树根、改哪份文件、改成什么，四行）在第一趟判的时候临时改主工作区那一份、第二趟判完改回，别的与 fake-anchor-static-check.py 同
  local window_case window_label window_edit window_dry_text window_apply_text window_dry_at window_apply_at window_before_rc window_find_rc window_awk_rc window_other_rc window_rc window_lines window_one window_two window_range window_word window_missing
  cat > "$scratch/dry-transient-static-check.py" <<'FAKE'
import os
import sys
tree = sys.argv[2]
here = os.path.dirname(os.path.abspath(__file__))
plan_path = os.path.join(here, "dry-transient-plan.txt")
state = os.path.join(here, "dry-transient-state.txt")


def read(path):
    with open(path, encoding="utf-8") as handle:
        return handle.read()


def write(path, text):
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


plan = read(plan_path).split("\n") if os.path.isfile(plan_path) else []
mode, root, relative, text = (plan + ["", "", "", ""])[:4]
target = os.path.join(root, relative) if root else ""
first = bool(mode) and not os.path.exists(state)
if first:
    kept = read(target)
    write(state, kept)
    if mode == "file":
        write(target, text + "\n")
    else:
        name, _, fixed = text.partition("|")
        rewritten = []
        for line in kept.split("\n"):
            columns = line.split("\t")
            rewritten.append("\t".join(columns[:2] + [fixed] + columns[3:]) if len(columns) > 2 and columns[0] == name else line)
        write(target, "\n".join(rewritten))
table = os.path.join(tree, "crates", "mutations.tsv")
rows = []
if os.path.isfile(table):
    rows = [line.split("\t") for line in read(table).split("\n") if line.strip() and not line.startswith("#")]
print("COPIED\tcrates")
red = 0
for number, columns in enumerate(rows, 1):
    path = os.path.join(tree, columns[1])
    count = read(path).count(columns[2]) if os.path.isfile(path) else 0
    if count != 1:
        red += 1
        print(f"ANCHOR\t{number}\t{columns[0]}\t原文在 {columns[1]} 里命中 {count} 次，要恰好 1 次")
print(f"CHECKED\t{len(rows)}\t1\t0")
if mode and not first and os.path.exists(state):
    write(target, read(state))
    os.remove(state)
sys.exit(1 if red else 0)
FAKE
  for window_case in "window-fingerprint:package-file" "window-table-append:table-row" "window-unchanged:none"; do
    window_label="${window_case%%:*}"; window_edit="${window_case#*:}"
    fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
    out="$scratch/out-merge-$window_label.txt"; merge_case "$window_label" "$scratch/merge-anchor-break"; checked=$((checked+1))
    window_dry_text="$(printf '%q ' "$root/task-server/task" submit -- python3 "$root/scripts/gate/apply-writer-patch.py" --dry-run "$scratch/logs-merge-$window_label/merge-patch")"
    window_apply_text="$(printf '%q ' "$root/task-server/task" submit -- python3 "$root/scripts/gate/apply-writer-patch.py" "$scratch/logs-merge-$window_label/merge-patch")"
    window_dry_at="$(grep -nxF "    ${window_dry_text% }" "$out" | head -n 1 | cut -d: -f1)"; window_apply_at="$(grep -nxF "    ${window_apply_text% }" "$out" | head -n 1 | cut -d: -f1)"
    if [[ $rc -ne 0 || -z "$window_dry_at" || -z "$window_apply_at" ]] || ((window_dry_at > window_apply_at)); then
      echo "  ✗ 自检：合入前的 --dry-run 之后的核（$window_label）：没走到：这一趟应当退 0、打出 ② 合入前的 --dry-run 与 ⑤ 合的那一条且 ② 在前，实际退 $rc、② 在第 ${window_dry_at:-无} 行、⑤ 在第 ${window_apply_at:-无} 行：$(grep -E '合进主工作区|--dry-run|✗' "$out" | head -n 3 | tr '\n' ' ' | clip_text 300)"; failures=1   # gate-lint:detail
      continue
    fi
    window_before_rc=missing; window_find_rc=missing; window_awk_rc=missing; window_other_rc=0; window_lines=0
    merge_check_line="$(awk -v to="$window_dry_at" 'NR < to && /^    LC_ALL=C awk / { sub(/^    /, ""); print; exit }' "$out")"
    if [[ -n "$merge_check_line" ]]; then bash -c "$merge_check_line" < /dev/null > "$scratch/window-$window_label-before.txt" 2>&1; window_before_rc=$?; fi
    merge_dry_run "$window_label" "$scratch/fake-anchor-static-check.py"
    case "$window_edit" in
      package-file) printf '// 合入前的 --dry-run 跑着的时候别的会话改的\n' >> "$merge_tree/crates/alpha/tests/case_a.rs" ;;
      table-row) printf '%s\t%s\t%s\t%s\t%s\t%s\n' window-table-row crates/alpha/src/merge_table.txt 1 9 '-p alpha --lib' f >> "$merge_tree/crates/mutations.tsv" ;;
    esac
    while IFS= read -r merge_check_line; do
      window_lines=$((window_lines+1))
      bash -c "$merge_check_line" < /dev/null > "$scratch/window-$window_label-after-$window_lines.txt" 2>&1; window_rc=$?
      case "$merge_check_line" in
        'find '*) window_find_rc=$window_rc ;;
        'LC_ALL=C awk '*) window_awk_rc=$window_rc ;;
        *) ((window_rc == 0)) || window_other_rc=$window_rc ;;
      esac
    done < <(awk -v from="$window_dry_at" -v to="$window_apply_at" 'NR > from && NR < to && /^    (find |LC_ALL=C awk |LC_ALL=C grep )/ { sub(/^    /, ""); print }' "$out")
    case "$window_label" in
      window-fingerprint)
        if [[ "$window_find_rc" == missing || "$window_find_rc" == 0 ]]; then
          echo "  ✗ 自检：合入前的 --dry-run 之后的核（window-fingerprint）：--dry-run 跑着的时候别的会话改了这一批改到的包里的文件，--dry-run 之后、合的那一条之前应当有 ④ 核主工作区没变的那一条、且它退非 0，实际它退 $window_find_rc（没打出来写 missing），② 之后到 ⑤ 之前共核了 $window_lines 条"; failures=1   # gate-lint:detail
        fi ;;
      window-table-append)
        if [[ "$window_awk_rc" == missing || "$window_awk_rc" == 0 ]]; then
          echo "  ✗ 自检：合入前的 --dry-run 之后的核（window-table-append）：--dry-run 跑完之后别的会话往变异表追加了锚在这一批改到的文件上的行，--dry-run 之后、合的那一条之前应当有 ③ 照 ① 重算再比的那一条、且它退非 0，实际它退 $window_awk_rc（没打出来写 missing），② 之后到 ⑤ 之前共核了 $window_lines 条"; failures=1   # gate-lint:detail
        fi ;;
      window-unchanged)
        if [[ "$window_before_rc" != 0 || "$window_find_rc" != 0 || "$window_awk_rc" != 0 || "$window_other_rc" != 0 ]]; then
          echo "  ✗ 自检：合入前的 --dry-run 之后的核（window-unchanged）：主工作区没变时 ① 记下、③ 比对、④ 核主工作区没变都应当退 0，实际 ① 退 $window_before_rc、③ 退 $window_awk_rc、④ 核主工作区没变退 $window_find_rc、④ 别的核退非 0 的是 $window_other_rc（没打出来写 missing）"; failures=1   # gate-lint:detail
        fi
        window_one="$(awk -v from="$window_apply_at" 'NR > from' "$out" | grep -F '开着的三方轮开工快照' | head -n 1)"
        window_two="$(awk -v from="$window_apply_at" 'NR > from' "$out" | grep -F '三方合并冲突' | head -n 1)"
        if [[ "$window_one" != *'从 ① 重做到 ⑤'* || "$window_two" != *'从 ① 重做到 ⑤'* ]]; then
          echo "  ✗ 自检：合入前的 --dry-run 之后的核（redo-loop）：合的那一条退 2 等完（开工快照、同一个仓有别的在换）与三方合并冲突、核与换之间被改了的出路都应当写从 ① 重做到 ⑤，实际等完那一句「$(clip_text 160 <<<"$window_one")」、冲突那一句「$(clip_text 160 <<<"$window_two")」"; failures=1   # gate-lint:detail
        fi
        window_range="$(awk -v from="$window_dry_at" 'NR > from { if ($0 ~ /^    ③ /) exit; print }' "$out")"
        window_missing=""
        for window_word in 没做成 判不了 被停 撞内存上限 '再交同一条 --dry-run' '从 ① 重做' 整批重交; do
          grep -qF -- "$window_word" <<<"$window_range" || window_missing+="「$window_word」"
        done
        if [[ -n "$window_missing" ]]; then
          echo "  ✗ 自检：合入前的 --dry-run 之后的核（dry-run-way-out-outcomes）：合入前的 --dry-run 那一条的出路应当分开写判不了、没做成、不是 2 的结局（被停、撞内存上限）怎么办，写整批重交与从 ① 重做，实际缺 $window_missing：$(clip_text 200 <<<"$window_range")"; failures=1   # gate-lint:detail
        fi
        if ! grep -qF '400–740' <<<"$window_range" || ! grep -qF '15 GiB' <<<"$window_range"; then
          echo "  ✗ 自检：合入前的 --dry-run 之后的核（dry-run-way-out-cost）：合入前的 --dry-run 那一条之后应当写代价（约 400–740 秒、内存峰值约 15 GiB），实际那一段是：$(clip_text 200 <<<"$window_range")"; failures=1   # gate-lint:detail
        fi ;;
    esac
  done
  # dry-transient：别的会话开跑之后追加的行锚在这一批改到的文件上；--dry-run 跑着时那份文件被临时改成 5、又改回
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  out="$scratch/out-merge-dry-transient.txt"; merge_case dry-transient "$scratch/merge-anchor-break"; checked=$((checked+1))
  if [[ $rc -ne 0 ]]; then
    echo "  ✗ 自检：合入前的 --dry-run：两棵树同一刻拷出（dry-transient）没走到：这一趟应当退 0，实际退 $rc：$(grep -E '变异锚点核|✗' "$out" | head -n 3 | tr '\n' ' ' | clip_text 300)"; failures=1   # gate-lint:detail
  else
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' dry-transient-row crates/alpha/src/merge_table.txt 1 9 '-p alpha --lib' f >> "$merge_tree/crates/mutations.tsv"
    printf 'file\n%s\ncrates/alpha/src/merge_table.txt\n5\n' "$merge_tree" > "$scratch/dry-transient-plan.txt"
    merge_dry_run dry-transient "$scratch/dry-transient-static-check.py"
    rm -f -- "$scratch/dry-transient-plan.txt" "$scratch/dry-transient-state.txt"
    if [[ $dry_rc -ne 2 ]] || ! grep -qE 'ANCHOR [0-9]+ dry-transient-row ' "$dry_out"; then
      echo "  ✗ 自检：合入前的 --dry-run：两棵树同一刻拷出（dry-transient）：别的会话追加的、锚在这一批改到的文件上的行被这一批弄断了，--dry-run 跑着的时候那份文件被临时改过又改回，补丁之前那一棵是同一刻拷出的树、这一行在它上面命中 1 次，--dry-run 应当退 2 点名 dry-transient-row，实际退 $dry_rc：$(head -n 4 "$dry_out" | tr '\n' ' ' | clip_text 300)"; failures=1   # gate-lint:detail
    fi
  fi
  # dry-skew：补丁之前就红的行在 --dry-run 跑着的时候被别的会话修好
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  out="$scratch/out-merge-dry-skew.txt"; merge_case dry-skew "$scratch/merge-anchor-break"; checked=$((checked+1))
  if [[ $rc -ne 0 ]]; then
    echo "  ✗ 自检：合入前的 --dry-run：补丁之前就红的行不算这一批弄断的（dry-skew）没走到：这一趟应当退 0，实际退 $rc：$(grep -E '变异锚点核|✗' "$out" | head -n 3 | tr '\n' ' ' | clip_text 300)"; failures=1   # gate-lint:detail
  else
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' dry-skew-prebroken-row crates/beta/src/merge_extra.txt 7 9 '-p beta --lib' f >> "$merge_tree/crates/mutations.tsv"
    printf 'table\n%s\ncrates/mutations.tsv\ndry-skew-prebroken-row|1\n' "$merge_tree" > "$scratch/dry-transient-plan.txt"
    merge_dry_run dry-skew "$scratch/dry-transient-static-check.py"
    rm -f -- "$scratch/dry-transient-plan.txt" "$scratch/dry-transient-state.txt"
    if [[ $dry_rc -ne 0 ]] || grep -qE 'ANCHOR [0-9]+ dry-skew-prebroken-row ' "$dry_out"; then
      echo "  ✗ 自检：合入前的 --dry-run：补丁之前就红的行不算这一批弄断的（dry-skew）：dry-skew-prebroken-row 补丁之前就红（原文 7 命中 0 次）、--dry-run 跑着的时候被别的会话修好，补丁之前那一棵是同一刻拷出的树、它在两棵树里都红，--dry-run 应当退 0、不点名它，实际退 $dry_rc：$(head -n 4 "$dry_out" | tr '\n' ' ' | clip_text 300)"; failures=1   # gate-lint:detail
    fi
  fi
  # dry-outside：合并补丁带拷贝范围之外的文件段（拒算表一行）
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  mkdir -p "$merge_tree/.claude/gate.d"; printf 'refused-original\n' > "$merge_tree/.claude/gate.d/refused-tests.tsv"
  rm -rf -- "${scratch:?}/merge-dry-outside"; mkdir -p "$scratch/merge-dry-outside"
  { printf 'diff --git a/crates/alpha/src/merge_table.txt b/crates/alpha/src/merge_table.txt\n'; merge_patch_text crates/alpha/src/merge_table.txt 1 2
    printf 'diff --git a/.claude/gate.d/refused-tests.tsv b/.claude/gate.d/refused-tests.tsv\n--- a/.claude/gate.d/refused-tests.tsv\n+++ b/.claude/gate.d/refused-tests.tsv\n@@ -1 +1,2 @@\n refused-original\n+refused-added\n'; } > "$scratch/merge-dry-outside/crates.patch"
  out="$scratch/out-merge-dry-outside.txt"; merge_case dry-outside "$scratch/merge-dry-outside"; checked=$((checked+1))
  merge_dry_run dry-outside "$scratch/fake-anchor-static-check.py"
  if [[ $rc -ne 0 || $dry_printed -ne 1 ]]; then
    echo "  ✗ 自检：合入前的 --dry-run：合并补丁带拷贝范围外的文件段（拒算表一行）时应当照常判：这一趟应当退 0、合进主工作区那一段里打出合入前的 --dry-run 且排在合的那一条之前，实际退 $rc、打出并排在前面的记 $dry_printed：$(grep -E '合进主工作区|--dry-run|✗' "$out" | head -n 3 | tr '\n' ' ' | clip_text 300)"; failures=1   # gate-lint:detail
  elif [[ $dry_rc -ne 0 ]] || ! grep -qF '核过了（--dry-run' "$dry_out" || grep -qF '没做成' "$dry_out"; then
    echo "  ✗ 自检：合入前的 --dry-run：合并补丁带拷贝范围外的文件段（拒算表一行）时应当照常判：--dry-run 应当退 0、报核过了、不退没做成（补丁改到的文件也拷进两棵临时树），实际退 $dry_rc：$(head -n 4 "$dry_out" | tr '\n' ' ' | clip_text 300)"; failures=1   # gate-lint:detail
  fi
  mkdir -p "$scratch/merge-anchor-own-only" "$scratch/merge-anchor-own-with-code" "$scratch/merge-anchor-created" "$scratch/merge-anchor-release-shift"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'merge-own-broken-row' crates/alpha/src/lib.rs 'pub fn nowhere() {}' 'pub fn nowhere() { }' '-p alpha --lib' f > "$scratch/merge-anchor-own-only/mutations-append.tsv"
  cp -- "$scratch/merge-anchor-own-only/mutations-append.tsv" "$scratch/merge-anchor-own-with-code/mutations-append.tsv"
  merge_patch_text crates/alpha/src/merge_table.txt 1 2 > "$scratch/merge-anchor-own-with-code/crates.patch"
  for anchor_case in "own-only:只改变异表" "own-with-code:连代码一起改"; do
    anchor_label="${anchor_case#*:}"; anchor_case="${anchor_case%%:*}"
    fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
    out="$scratch/out-merge-anchor-$anchor_case.txt"; merge_case "anchor-$anchor_case" "$scratch/merge-anchor-$anchor_case"; checked=$((checked+1))
    if [[ $rc -ne 1 ]] || ! grep -qE 'ANCHOR [0-9]+ merge-own-broken-row 原文在 crates/alpha/src/lib\.rs 里命中 0 次' "$out"; then
      echo "  ✗ 自检：变异锚点核：自己追加的行锚点不中（$anchor_label），应当判红、退 1、点名 merge-own-broken-row，实际退出码 $rc：$(grep -E '变异锚点核|ANCHOR' "$out" | head -n 3 | tr '\n' ' ' | clip_text 300)"; failures=1   # gate-lint:detail
    fi
  done
  printf 'diff --git a/crates/alpha/src/anchor_created.txt b/crates/alpha/src/anchor_created.txt\nnew file mode 100644\n--- /dev/null\n+++ b/crates/alpha/src/anchor_created.txt\n@@ -0,0 +1 @@\n+fresh\n' > "$scratch/merge-anchor-created/crates.patch"
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  out="$scratch/out-merge-anchor-created.txt"; merge_case anchor-created "$scratch/merge-anchor-created"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -qF '变异锚点核过' "$out"; then
    echo "  ✗ 自检：变异锚点核：补丁只新建一份文件、不碰变异行，应当过、这一趟退 0，实际退出码 $rc：$(grep -E '变异锚点核|套不上|没做成' "$out" | head -n 3 | tr '\n' ' ' | clip_text 300)"; failures=1   # gate-lint:detail
  fi
  cat > "$scratch/anchor-release-static-check.py" <<'FAKE'
import os
import sys
tree = sys.argv[2]
table = os.path.join(tree, "crates", "mutations.tsv")
lines = []
if os.path.isfile(table):
    with open(table, encoding="utf-8") as handle:
        lines = handle.read().split("\n")
print("COPIED\tcrates")
red = rows = 0
for number, line in enumerate(lines, 1):
    if not line.strip() or line.startswith("#"):
        continue
    rows += 1
    columns = line.split("\t")
    path = os.path.join(tree, columns[1])
    text = ""
    if os.path.isfile(path):
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
    count = text.count(columns[2])
    if count != 1:
        red += 1
        print(f"ANCHOR\t{number}\t{columns[0]}\t原文在 {columns[1]} 里命中 {count} 次，要恰好 1 次")
    if len(columns) > 5 and columns[5] == "release_red":
        red += 1
        print(f"RELEASE\t{number}\t{columns[0]}\tcrash-case 目标 beta:case_b（--test 点名；登记 crash-case:anchor）")
print(f"CHECKED\t{rows}\t1\t0")
sys.exit(1 if red else 0)
FAKE
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'merge-row-to-delete' crates/demo/src/lib.rs 'pub fn f() {}' 'pub fn f() { }' '-p demo --lib' f \
    'merge-release-red-row' crates/beta/src/lib.rs 'pub fn f() {}' 'pub fn f() { }' '-p beta --lib' release_red >> "$merge_tree/crates/mutations.tsv"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'merge-row-to-delete' crates/demo/src/lib.rs 'pub fn f() {}' 'pub fn f() { }' '-p demo --lib' f > "$scratch/merge-anchor-release-shift/mutations-delete.txt"
  anchor_static_saved="$MERGE_AND_VERIFY_STATIC_CHECK"; export MERGE_AND_VERIFY_STATIC_CHECK="$scratch/anchor-release-static-check.py"
  out="$scratch/out-merge-anchor-release-shift.txt"; merge_case anchor-release-shift "$scratch/merge-anchor-release-shift"; checked=$((checked+1))
  export MERGE_AND_VERIFY_STATIC_CHECK="$anchor_static_saved"
  if [[ $rc -ne 0 ]] || ! grep -qF '补丁前就红的 1 处' "$out" || grep -qE 'RELEASE [0-9]+ merge-release-red-row' "$out"; then
    echo "  ✗ 自检：变异锚点核：补丁之前就红的 RELEASE 行只因这一批删了它前面一行、行号前移，不算这一批带来的新红，应当退 0、报补丁前就红的 1 处、不点名 merge-release-red-row，实际退出码 $rc：$(grep -E '变异锚点核|RELEASE|补丁前就红' "$out" | head -n 4 | tr '\n' ' ' | clip_text 400)"; failures=1   # gate-lint:detail
  fi
  # 带 --skip-apply 的那一趟：补丁（merge_table.txt 1→2 / 追加的变异行）已写在样本树上，补丁之前的树由倒打摆出
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  printf '2\n' > "$merge_tree/crates/alpha/src/merge_table.txt"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'merge-data-row' crates/alpha/src/merge_table.txt 1 9 '-p alpha --lib' f >> "$merge_tree/crates/mutations.tsv"
  out="$scratch/out-merge-anchor-skip-apply.txt"; merge_case anchor-skip-apply --skip-apply "$scratch/merge-anchor-break"; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || ! grep -qE 'ANCHOR [0-9]+ merge-data-row 原文在 crates/alpha/src/merge_table\.txt 里命中 0 次' "$out" || grep -qF 'merge-kept-row 原文在' "$out"; then
    echo "  ✗ 自检：变异锚点核：带 --skip-apply 的那一趟补丁弄断别批变异行的锚点应当判红、退 1、点名 merge-data-row（不点名没弄断的 merge-kept-row），实际退出码 $rc：$(grep -E '变异锚点核|ANCHOR' "$out" | head -n 4 | tr '\n' ' ' | clip_text 400)"; failures=1   # gate-lint:detail
  fi
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  cat -- "$scratch/merge-anchor-own-only/mutations-append.tsv" >> "$merge_tree/crates/mutations.tsv"
  out="$scratch/out-merge-anchor-skip-apply-own-only.txt"; merge_case anchor-skip-apply-own-only --skip-apply "$scratch/merge-anchor-own-only"; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || ! grep -qE 'ANCHOR [0-9]+ merge-own-broken-row 原文在 crates/alpha/src/lib\.rs 里命中 0 次' "$out"; then
    echo "  ✗ 自检：变异锚点核：带 --skip-apply 的那一趟补丁自己追加的行锚点不中应当判红、退 1、点名 merge-own-broken-row，实际退出码 $rc：$(grep -E '变异锚点核|ANCHOR' "$out" | head -n 4 | tr '\n' ' ' | clip_text 400)"; failures=1   # gate-lint:detail
  fi
  # 带 --skip-apply 的那一趟补丁目录只有一行替换：样本树的变异表里 merge-kept-row 已是替换成的那一行（原文 gone 在 alpha 的 lib.rs 里命中 0 次），补丁目录的 mutations-replacements.tsv 是改后的行接前像：倒回前像再比，改后的行是新红
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  anchor_original="$(printf '%s\t%s\t%s\t%s\t%s\t%s' merge-kept-row crates/alpha/src/lib.rs 'pub fn f() {}' 'pub fn f() { }' '-p alpha --lib' f)"
  anchor_replaced="$(printf '%s\t%s\t%s\t%s\t%s\t%s' merge-kept-row crates/alpha/src/lib.rs 'pub fn gone() {}' 'pub fn gone() { }' '-p alpha --lib' f)"
  printf '%s\n' "$anchor_replaced" > "$merge_tree/crates/mutations.tsv"
  rm -rf -- "${scratch:?}/merge-anchor-skip-replace"; mkdir -p "$scratch/merge-anchor-skip-replace"
  printf '%s\t%s\n' "$anchor_replaced" "$anchor_original" > "$scratch/merge-anchor-skip-replace/mutations-replacements.tsv"
  out="$scratch/out-merge-anchor-skip-replace.txt"; merge_case anchor-skip-replace --skip-apply "$scratch/merge-anchor-skip-replace"; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || ! grep -qE 'ANCHOR [0-9]+ merge-kept-row 原文在 crates/alpha/src/lib\.rs 里命中 0 次' "$out"; then
    echo "  ✗ 自检：变异锚点核：带 --skip-apply 的那一趟补丁把变异行换成锚点不中的一行应当判红：应当退 1、点名 merge-kept-row 与命中 0 次，实际退出码 $rc：$(grep -E '变异锚点核|ANCHOR|倒不回' "$out" | head -n 4 | tr '\n' ' ' | clip_text 400)"; failures=1   # gate-lint:detail
  fi
  # 带 --skip-apply 的那一趟变异表倒不回补丁之前：补丁目录 merge-anchor-own-only 追加的 merge-own-broken-row 在样本树的表里已被改成锚在 beta 的 merge_extra.txt 的另一行，找不到追加的原样
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'merge-own-broken-row' crates/beta/src/merge_extra.txt 1 9 '-p beta --lib' f >> "$merge_tree/crates/mutations.tsv"
  out="$scratch/out-merge-anchor-skip-irreversible.txt"; merge_case anchor-skip-irreversible --skip-apply "$scratch/merge-anchor-own-only"; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || ! grep -qF '变异锚点核没做成' "$out"; then
    echo "  ✗ 自检：变异锚点核：带 --skip-apply 的那一趟变异表倒不回补丁之前应当记红：应当退 1、写变异锚点核没做成，实际退出码 $rc：$(grep -E '变异锚点核|倒不回' "$out" | head -n 3 | tr '\n' ' ' | clip_text 300)"; failures=1   # gate-lint:detail
  fi
  # 带 --skip-apply 一个补丁目录都不给：没有补丁就摆不出补丁之前的树，打一行变异锚点核本次未核
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  out="$scratch/out-merge-anchor-skip-none.txt"; merge_case anchor-skip-none --skip-apply; checked=$((checked+1))
  if ! grep -qF '变异锚点核本次未核' "$out"; then
    echo "  ✗ 自检：变异锚点核：带 --skip-apply 没给补丁目录应当写本次未核：这一趟退 $rc，变异锚点核一句都没打（没核，也没写本次未核）：$(grep -E '变异锚点核|补丁打没打' "$out" | head -n 2 | tr '\n' ' ' | clip_text 200)"; failures=1   # gate-lint:detail
  fi
  # 本次未核不记红、写明为什么：样本树不放 ANCHOR_CHECK_SAMPLE_MARK（自证里这一步整个不做）的那一趟与放了标记（打本次未核）的那一趟，退出码相同，后一趟的那一行写明摆不出补丁之前的树
  local skip_none_plain_rc
  fresh_merge_tree "$merge_tree"
  out="$scratch/out-merge-anchor-skip-none-plain.txt"; merge_case anchor-skip-none-plain --skip-apply; skip_none_plain_rc=$rc
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  out="$scratch/out-merge-anchor-skip-none-marked.txt"; merge_case anchor-skip-none-marked --skip-apply; checked=$((checked+1))
  if [[ $skip_none_plain_rc -ne $rc ]] || ! grep -qF '摆不出补丁之前的树' "$out"; then
    echo "  ✗ 自检：变异锚点核：带 --skip-apply 没给补丁目录的本次未核应当不记红、写明为什么（摆不出补丁之前的树）：不放标记（不进这一步）那一趟退 $skip_none_plain_rc，放了标记（打本次未核）那一趟退 $rc：$(grep -F '本次未核' "$out" | head -n 2 | tr '\n' ' ' | clip_text 300)"; failures=1   # gate-lint:detail
  fi
  # 新红多过标准输出列的行数：表里别批 45 行都锚在 merge_table.txt 里的原文 1 上，补丁把它改成 2
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  for (( anchor_index = 1; anchor_index <= 45; anchor_index++ )); do
    printf 'merge-many-row-%02d\t%s\t%s\t%s\t%s\t%s\n' "$anchor_index" crates/alpha/src/merge_table.txt 1 9 '-p alpha --lib' f >> "$merge_tree/crates/mutations.tsv"
  done
  out="$scratch/out-merge-anchor-many.txt"; merge_case anchor-many "$scratch/merge-anchor-break"; checked=$((checked+1))
  anchor_total="$(grep -c 'merge-many-row-' "$scratch/logs-merge-anchor-many/anchor-check.log")"
  if [[ $rc -ne 1 ]] || ((anchor_total != 45)) || ! grep -qF '新红一共 45 处' "$out" || (( $(grep -c 'merge-many-row-' "$out") >= 45 )); then
    echo "  ✗ 自检：变异锚点核：新红多过标准输出列的行数时，日志里应当全有（45 处）、标准输出写明一共几处（新红一共 45 处）并只列前面几行，实际退出码 $rc、日志里 $anchor_total 处：$(grep -E '新红一共|✗ 变异锚点核' "$out" | head -n 2 | tr '\n' ' ' | clip_text 300)"; failures=1   # gate-lint:detail
  fi
  # 没留快照的那一趟：出路写带 MERGE_AND_VERIFY_KEEP_COPY=1 整条重交
  fresh_merge_tree "$merge_tree"; : > "$merge_tree/$ANCHOR_CHECK_SAMPLE_MARK"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'merge-data-row' crates/alpha/src/merge_table.txt 1 9 '-p alpha --lib' f >> "$merge_tree/crates/mutations.tsv"
  out="$scratch/out-merge-anchor-no-keep.txt"; MERGE_CASE_KEEP_COPY=0 merge_case anchor-no-keep "$scratch/merge-anchor-break"; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || ! grep -qF '这一趟没留快照（没设 MERGE_AND_VERIFY_KEEP_COPY=1）' "$out" || ! grep -qF 'env MERGE_AND_VERIFY_KEEP_COPY=1 整条重交' "$out"; then
    echo "  ✗ 自检：变异锚点核：没留快照的那一趟出路应当写带 MERGE_AND_VERIFY_KEEP_COPY=1 整条重交，实际退出码 $rc：$(grep -E '变异锚点核|怎么办|没留快照' "$out" | head -n 4 | tr '\n' ' ' | clip_text 400)"; failures=1   # gate-lint:detail
  fi
  # 登记表（文件头「挑测试目标」登记表那一条；样本与判法在 lib_merge_verify_formal_tables.py 的 sample、judge），四种补丁各配一份加 alpha 数据文件的 crates.patch，不带 --skip-apply 跑：
  #   registered 改 verus-pins.tsv 登记的必红世界表（alpha 那一张三行：改了一行的文件、末尾新加一行，另两行没动）：不落测试目标、不退 3、报一句，汇总行之后只点改过与新加的那两行的 formal-verification 一条（不点没动的两行、不点没改到的 beta 那张）；
  #   deleted 同一张表只删一行：不落测试目标、不退 3、报一句没有新加或改过的行，汇总行之后没有 formal-verification；
  #   unregistered 改包里一份没登记的 .tsv：照旧判不出、退 3、不交 formal-verification；plain 改拒算表（崩溃枚举用例、放量分项与拒算测试那一类登记表）：不落测试目标、不退 3、报一句
  for formal_case in registered deleted unregistered plain; do
    fresh_merge_tree "$merge_tree"; python3 "$here/lib_merge_verify_formal_tables.py" sample "$merge_tree" "$scratch/merge-formal-$formal_case" "$formal_case"
    out="$scratch/out-merge-formal-$formal_case.txt"; merge_case "formal-$formal_case" "$scratch/merge-formal-$formal_case"; checked=$((checked+1))
    python3 "$here/lib_merge_verify_formal_tables.py" judge "$formal_case" "$rc" "$out" "$scratch/logs-merge-formal-$formal_case" || failures=1
  done
  # 合进主工作区的那一份补丁目录算得出大小与修改时刻都相同的改动（MERGE_PATCH_PROGRAM 的 place_after；git 的 stat 缓存不能把它漏掉）：直接喂那一段一份样本，
  #   基线与要验的树里 docs/stat.txt 都是一个字节（1 与 2）、修改时刻同在 2020 年初（比临时仓的索引老得多，git 不当「刚写的」重读内容）；样本树里的 .merge-patch-stat-sample 标记
  #   让弄坏开关 merge-patch-stat-cache 只在这一格生效
  local stat_tree="$scratch/stat-tree" stat_baseline="$scratch/stat-baseline" stat_summary
  mkdir -p "$stat_tree/docs" "$stat_baseline/docs"; : > "$stat_tree/.merge-patch-stat-sample"; printf 'docs/stat.txt\n' > "$scratch/stat-paths"
  printf '1\n' > "$stat_baseline/docs/stat.txt"; printf '2\n' > "$stat_tree/docs/stat.txt"
  touch -d @1577836800 "$stat_baseline/docs/stat.txt" "$stat_tree/docs/stat.txt"
  stat_summary="$(python3 - "$REBUILT" "$stat_tree" "$stat_baseline" "$scratch/stat-paths" "$scratch/stat-out" <<<"$MERGE_PATCH_PROGRAM" 2>&1 | tail -n 1)"
  checked=$((checked+1))
  if [[ "$stat_summary" != "合起来改 1 份文件"* ]] || ! grep -qx '+2' "$scratch/stat-out/crates.patch" 2>/dev/null; then
    echo "  ✗ 自检：先验后合：合进主工作区的那一份补丁目录里大小与修改时刻都相同的改动不能被 git 的 stat 缓存漏掉（基线 1、改后 2，同为一个字节、同一个修改时刻），应当算出改了 1 份文件、crates.patch 里有 +2，实际末行「$stat_summary」，补丁目录里有 $(ls "$scratch/stat-out" 2>/dev/null | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 合进主工作区的那一份补丁目录里的变异行替换与删除带前像（MERGE_PATCH_PROGRAM；格式在 apply-writer-patch.py 文件头「补丁目录里的文件」）：直接喂那一段一份样本，
  #   基线的变异表 kept、changed、gone 三行，快照此刻 changed 改了第四段、gone 删了、新加 added；替换那一行应当是改后的行接基线里那一行，删除那一行应当是基线里那一行
  local preimage_tree="$scratch/preimage-tree" preimage_baseline="$scratch/preimage-baseline" preimage_summary
  mkdir -p "$preimage_tree/crates" "$preimage_baseline/crates"; printf 'crates/mutations.tsv\n' > "$scratch/preimage-paths"
  sample_mutation_row() { printf '%s\t%s\t%s\t%s\t%s\t%s' "$1" crates/alpha/src/lib.rs 'pub fn f() {}' "$2" '-p alpha --lib' f; }   # <名字> <第四段>：一行六段，不带行尾换行符
  printf '%s\n' "$(sample_mutation_row kept 'pub fn f() { }')" "$(sample_mutation_row changed 'pub fn f() {  }')" "$(sample_mutation_row gone 'pub fn f() {   }')" > "$preimage_baseline/crates/mutations.tsv"
  printf '%s\n' "$(sample_mutation_row kept 'pub fn f() { }')" "$(sample_mutation_row changed 'pub fn f() { 1 }')" "$(sample_mutation_row added 'pub fn f() { 2 }')" > "$preimage_tree/crates/mutations.tsv"
  preimage_summary="$(python3 - "$REBUILT" "$preimage_tree" "$preimage_baseline" "$scratch/preimage-paths" "$scratch/preimage-out" <<<"$MERGE_PATCH_PROGRAM" 2>&1 | tail -n 1)"
  checked=$((checked+1))
  if [[ "$preimage_summary" != *'变异行追加 1、换 1、删 1' ]] \
     || [[ "$(cat "$scratch/preimage-out/mutations-replacements.tsv" 2>/dev/null)" != "$(sample_mutation_row changed 'pub fn f() { 1 }')"$'\t'"$(sample_mutation_row changed 'pub fn f() {  }')" ]] \
     || [[ "$(cat "$scratch/preimage-out/mutations-delete.txt" 2>/dev/null)" != "$(sample_mutation_row gone 'pub fn f() {   }')" ]]; then
    echo "  ✗ 自检：先验后合：合进主工作区的那一份补丁目录里的变异行替换与删除应当带前像（替换一行是改后的行接基线里那一行、删除一行是基线里那一行），实际末行「$preimage_summary」，替换「$(head -c 300 "$scratch/preimage-out/mutations-replacements.tsv" 2>/dev/null | tr '\t\n' '|/')」，删除「$(head -c 300 "$scratch/preimage-out/mutations-delete.txt" 2>/dev/null | tr '\t\n' '|/')」"; failures=1; fi   # gate-lint:detail
  # 打出的变异行名是照补丁次序合完之后还在的行（文件头「先验后合」）：头一份追加 merge-gone-row 与 merge-new-row，后一份的 mutations-delete.txt 删掉 merge-gone-row，只该列 merge-new-row
  mkdir -p "$scratch/merge-names-add" "$scratch/merge-names-delete"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'merge-gone-row' crates/alpha/src/lib.rs 'pub fn f() {}' 'pub fn f() { }' '-p alpha --lib' f \
    'merge-new-row' crates/alpha/src/lib.rs 'pub fn f() {}' 'pub fn f() {  }' '-p alpha --lib' f > "$scratch/merge-names-add/mutations-append.tsv"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'merge-gone-row' crates/alpha/src/lib.rs 'pub fn f() {}' 'pub fn f() { }' '-p alpha --lib' f > "$scratch/merge-names-delete/mutations-delete.txt"
  fresh_merge_tree "$merge_tree"
  out="$scratch/out-merge-names.txt"; merge_case names "$scratch/merge-names-add" "$scratch/merge-names-delete"; checked=$((checked+1))
  # 只判打变异行名的那一行：挑测试目标那一段的「变异行「名」点名」里本来就有头一份补丁追加过的 merge-gone-row，整份输出里找它必中
  local merge_names_line
  merge_names_line="$(grep -F '这一批新写、改写的变异行 ' "$out" | head -n 1)"
  if [[ "$merge_names_line" != *'这一批新写、改写的变异行 1 行'* || "$merge_names_line" != *'「merge-new-row」'* || "$merge_names_line" == *'「merge-gone-row」'* ]]; then
    echo "  ✗ 自检：先验后合：打出的变异行名应当是照补丁次序合完之后还在的行（头一份追加 merge-gone-row 与 merge-new-row、后一份删掉 merge-gone-row，只列 merge-new-row），实际退出码 $rc：${merge_names_line:0:300}"; failures=1; fi   # gate-lint:detail
  # 没留快照（run_case three：不带 --skip-apply、没设 MERGE_AND_VERIFY_KEEP_COPY=1、验绿）：不打合进主工作区的命令，写一句没留快照、带它重交
  checked=$((checked+1))
  if grep -qF 'apply-writer-patch.py' "$scratch/out3.txt" || ! grep -qF '这一趟没留快照（没设 MERGE_AND_VERIFY_KEEP_COPY=1）' "$scratch/out3.txt"; then
    echo "  ✗ 自检：先验后合：没留快照的那一趟验绿了不该打合进主工作区的命令，实际：$(grep -E '没留快照|合进主工作区|apply-writer-patch' "$scratch/out3.txt" | head -3 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  # --fingerprint-of（文件头「开跑那一刻的指纹」）：头一趟之后主工作区没动，带它的重跑退 0；alpha 的 tests/case_a.rs 开跑之后改了，带它的重跑退 2、一个补丁都不打、一条 cargo 都不起
  fresh_merge_tree "$merge_tree"
  out="$scratch/out-merge-recheck-base.txt"; merge_case recheck-base "$scratch/merge-head" "$scratch/merge-stacked"
  out="$scratch/out-merge-recheck-same.txt"; merge_case recheck-same --item alpha:src/lib.rs "$scratch/merge-head" "$scratch/merge-stacked" --fingerprint-of "$scratch/logs-merge-recheck-base"
  recheck_same_rc=$rc
  printf '// 开跑之后改的\n' >> "$merge_tree/crates/alpha/tests/case_a.rs"
  out="$scratch/out-merge-recheck-changed.txt"; merge_case recheck-changed --item alpha:src/lib.rs "$scratch/merge-head" "$scratch/merge-stacked" --fingerprint-of "$scratch/logs-merge-recheck-base"
  checked=$((checked+1))
  if [[ $recheck_same_rc -ne 0 || $rc -ne 2 || -s "$scratch/calls-merge-recheck-changed.txt" || -e "$scratch/logs-merge-recheck-changed/apply-merge-head.log" ]] || ! grep -qF '开跑以来主工作区变了' "$out"; then
    echo "  ✗ 自检：先验后合：带 --fingerprint-of 的重跑，头一趟开跑之后主工作区变了应当退 2、一个补丁都不打（没变的那一趟退 0），实际 ${recheck_same_rc}、${rc}：$(grep -E '开跑以来|✗' "$out" | head -2 | tr '\n' ' ' | clip_text 300)"; failures=1; fi   # gate-lint:detail
  # 带 --skip-apply 的那一趟（narrow-two 那一格，补丁已在要验的树上）验绿了不打合进主工作区的命令
  checked=$((checked+1))
  if grep -qF '合进主工作区' "$scratch/out-narrow-two.txt" || grep -qF 'apply-writer-patch.py' "$scratch/out-narrow-two.txt"; then
    echo "  ✗ 自检：先验后合：带 --skip-apply 的那一趟验绿了不该打合进主工作区的命令，实际：$(grep -E '合进主工作区|apply-writer-patch' "$scratch/out-narrow-two.txt" | head -3 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  # 判不出测试目标的四种（文件头「挑测试目标」）：测试那一步不跑、别的步照跑、整条退 3、逐条报原因与整包归提交时的出路；
  #   build 照文件头 build 一段挑包：没给补丁目录、只改 Cargo.lock、只改不落在包里的 docs/notes.txt 没有包，报 build 本次未跑、假 cargo 收不到 build；
  #   改到 alpha 包里的文件的只点 alpha 与 dev 依赖它的 beta（不点 demo、delta、验证包与产品 GPU 包）
  narrow_patch "$scratch/narrow-lock" Cargo.lock
  narrow_patch "$scratch/narrow-loose" docs/notes.txt
  narrow_patch "$scratch/narrow-unmapped" crates/alpha/tests/fixture.txt
  local refusal_label refusal_text refusal_build refusal_build_received
  local -A refusal_rc=()
  for refusal_label in none lock loose unmapped; do
    out="$scratch/out-narrow-refuse-$refusal_label.txt"; checked=$((checked+1)); refusal_build=""
    case "$refusal_label" in
      none) refusal_text='没给补丁目录'; narrow_case refuse-none --skip-apply ;;
      lock) refusal_text='补丁改到工作区根的 Cargo.lock'; narrow_case refuse-lock --skip-apply "$scratch/narrow-lock" ;;
      loose) refusal_text='补丁改到不落在任何包里的 docs/notes.txt'; narrow_case refuse-loose --skip-apply "$scratch/narrow-loose" ;;
      unmapped) refusal_text='补丁改到 alpha 包里的 tests/fixture.txt，判不出改到哪个测试目标'; refusal_build='build --offline --all-targets -p alpha -p beta|'
         narrow_case refuse-unmapped --skip-apply "$scratch/narrow-unmapped" ;;
    esac
    refusal_rc[$refusal_label]=$rc
    refusal_build_received="$(grep '^build' "$scratch/calls-narrow-refuse-$refusal_label.txt" 2>/dev/null | tr '\n' '|')"
    if [[ $rc -ne 3 || -n "$(narrow_tests "refuse-$refusal_label")" || "$refusal_build_received" != "$refusal_build" ]] || ! grep -qF -- "$refusal_text" "$out" || ! grep -qF '整包归提交时的整轮门禁；提交之外要整包跑先弹窗问用户' "$out" \
       || { [[ -z "$refusal_build" ]] && ! grep -qF '  - build 本次未跑：' "$out"; }; then
      echo "  ✗ 自检：判不出测试目标：$refusal_text 时测试那一步不跑、别的步照跑（build 要收到「${refusal_build:-不起，报 build 本次未跑}」）、退 3，实际退出码 $rc，假 cargo 收到的测试命令 $(narrow_tests "refuse-$refusal_label")、build 命令「$refusal_build_received」：$(grep -E '判不出|✗|build' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  done
  # 判不出测试目标的出路写明包从哪取（文件头「挑测试目标」判不出的这几种）：工作区根的 Cargo.lock、不落在任何包里的 docs/notes.txt 没有包可点，原因里写明整包跑的范围由用户来选；
  #   落在包里的 alpha 的 tests/fixture.txt 点名了 alpha 包；三种的出路里都有「用户同意整包跑的，点上面逐条点名的包」那一句。
  #   三格都退 3 才核这一格（没退 3 的那一格自己已经红了）
  checked=$((checked+1))
  if [[ "${refusal_rc[lock]:-}" == 3 && "${refusal_rc[loose]:-}" == 3 && "${refusal_rc[unmapped]:-}" == 3 ]] \
     && { ! grep -qF '补丁改到工作区根的 Cargo.lock（改的是整个工作区的编法，判不出改到哪个测试目标；没有包可点，整包跑的范围交用户定）' "$scratch/out-narrow-refuse-lock.txt" \
          || ! grep -qF '补丁改到不落在任何包里的 docs/notes.txt（测试运行时可能读它，判不出归哪个测试目标；没有包可点，整包跑的范围交用户定）' "$scratch/out-narrow-refuse-loose.txt" \
          || ! grep -qF '用户同意整包跑的，点上面逐条点名的包' "$scratch/out-narrow-refuse-lock.txt" || ! grep -qF '用户同意整包跑的，点上面逐条点名的包' "$scratch/out-narrow-refuse-unmapped.txt"; }; then
    echo "  ✗ 自检：判不出测试目标的出路应当写明点哪几个包：工作区根的文件与不落在任何包里的路径写明没有包可点、整包跑的范围交用户定，出路里有「用户同意整包跑的，点上面逐条点名的包」那一句，实际改 Cargo.lock 的那一趟「$(grep -E '补丁改到工作区根|用户同意整包跑的' "$scratch/out-narrow-refuse-lock.txt" | head -2 | clip_text 300 | tr '\n' '|')」、改 docs/notes.txt 的那一趟「$(grep -E '补丁改到不落在任何包里' "$scratch/out-narrow-refuse-loose.txt" | head -1 | clip_text 300)」"; failures=1; fi   # gate-lint:detail
  rm -rf -- "${narrow_tree:?}/docs" "${narrow_tree:?}/crates/alpha/tests/fixture.txt"
  # 不变量文档（文件头「挑测试目标」不变量文档那一条）：alpha 的 invariants/05-sample.md。
  #   beta 的 mentions_invariants.rs 只在注释里提到它，不算读：不退 3、测试那一步不跑（弄坏开关 document-comments-counted、document-refused 下红）；
  #   再放 beta 的 reads_invariants.rs 按路径读它：只起这一个目标（document-readers-unscanned、document-refused 下红）；
  #   只有 alpha 的 build.rs 读它：落不到测试目标，退 3（document-readers-unscanned、document-refused 下红）
  printf '// 判据见 invariants/05-sample.md\nfn main() {}\n' > "$narrow_tree/crates/beta/tests/mentions_invariants.rs"
  narrow_patch "$scratch/narrow-document" crates/alpha/invariants/05-sample.md
  out="$scratch/out-narrow-document-unread.txt"; narrow_case document-unread --skip-apply "$scratch/narrow-document"; checked=$((checked+1))
  if [[ $rc -ne 0 || -n "$(narrow_tests document-unread)" ]] || ! grep -qF '没有一处读它，不影响测试目标' "$out" || ! grep -qF '测试本次未跑' "$out"; then
    echo "  ✗ 自检：不变量文档：没有代码读它（注释里提到不算）：不退 3、测试那一步不跑，实际退出码 $rc，假 cargo 收到 $(narrow_tests document-unread)：$(grep -E '判不出|不影响|✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  printf 'fn read() -> String { std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../alpha/invariants/05-sample.md")).unwrap() }\nfn main() { read(); }\n' > "$narrow_tree/crates/beta/tests/reads_invariants.rs"
  out="$scratch/out-narrow-document-read.txt"; narrow_case document-read --skip-apply "$scratch/narrow-document"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests document-read)" != "test --offline --no-fail-fast -p beta --test reads_invariants|" ]] || ! grep -qF 'beta 的 tests/reads_invariants.rs 读它' "$out"; then
    echo "  ✗ 自检：不变量文档：beta 的测试目标 reads_invariants 读它：只该起这一个目标，实际退出码 $rc，假 cargo 收到 $(narrow_tests document-read)：$(grep -E '判不出|读它|✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  rm -f -- "$narrow_tree/crates/beta/tests/reads_invariants.rs" "$narrow_tree/crates/beta/tests/mentions_invariants.rs"
  printf 'const DOCUMENT: &str = "invariants/05-sample.md";\nfn main() { let _ = DOCUMENT; }\n' > "$narrow_tree/crates/alpha/build.rs"
  out="$scratch/out-narrow-document-build-script.txt"; narrow_case document-build-script --skip-apply "$scratch/narrow-document"; checked=$((checked+1))
  if [[ $rc -ne 3 || -n "$(narrow_tests document-build-script)" ]] || ! grep -qF '读它的 alpha 的 build.rs 落不到测试目标' "$out"; then
    echo "  ✗ 自检：判不出测试目标：不变量文档只被 alpha 的 build.rs 读：测试那一步不跑、退 3，实际退出码 $rc，假 cargo 收到 $(narrow_tests document-build-script)：$(grep -E '判不出|落不到|✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  rm -rf -- "${narrow_tree:?}/crates/alpha/build.rs" "${narrow_tree:?}/crates/alpha/invariants"
  # 两个 bin 经 #[path] 共用一个模块（文件头「挑测试目标」mod / #[path] 那一条）：first、second 都写 #[path = "shared_dump/mod.rs"] mod dump;，共用模块里再 mod inner;，other 不引它；
  #   补丁只改 inner.rs：测试那一步只起 --bin first 与 --bin second（不起库、不起 other），挑出的项点名补丁改到 inner.rs。
  #   模块名 dump 与目录名 shared_dump 故意不同：#[path] 当没写时 mod dump; 找不到文件，判不出、退 3，这一格红（弄坏开关 path-attribute-ignored）
  local narrow_bin
  mkdir -p "$narrow_tree/crates/alpha/src/bin/shared_dump"
  for narrow_bin in first second; do printf '#[path = "shared_dump/mod.rs"]\nmod dump;\nfn main() {}\n' > "$narrow_tree/crates/alpha/src/bin/$narrow_bin.rs"; done
  printf 'fn main() {}\n' > "$narrow_tree/crates/alpha/src/bin/other.rs"
  printf 'mod inner;\n' > "$narrow_tree/crates/alpha/src/bin/shared_dump/mod.rs"
  narrow_patch "$scratch/narrow-shared-module" crates/alpha/src/bin/shared_dump/inner.rs
  out="$scratch/out-narrow-shared-module.txt"; narrow_case shared-module --skip-apply "$scratch/narrow-shared-module"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests shared-module)" != "test --offline --no-fail-fast -p alpha --bin first --bin second|" ]] \
     || ! grep -qF '补丁改到 crates/alpha/src/bin/shared_dump/inner.rs' "$out" || grep -qF '判不出' "$out"; then
    echo "  ✗ 自检：挑测试目标：两个 bin 经 #[path] 共用一个模块、补丁只改共用模块里 mod 出来的 inner.rs：测试那一步只该起 alpha 的 --bin first 与 --bin second，实际退出码 $rc，假 cargo 收到 $(narrow_tests shared-module)：$(grep -E '判不出|挑测试目标|✗' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  rm -rf -- "${narrow_tree:?}/crates/alpha/src/bin"
  # tests/<子目录>/ 下的文件（文件头「挑测试目标」tests/<子目录>/ 那一条）：alpha 的 tests/states/ 没有 main.rs，mod.rs 只写 mod inner;；
  #   by_path_one、by_path_two 经 #[path = "states/table.rs"] 引 table.rs，by_module_one、by_module_two 写 mod states;（经 mod.rs 走到 inner.rs），case_a 不引它；
  #   orphan.rs 谁都不引。整目录那一条（tests/ 顶层里 mod states 或 #[path = "states/…"]）落到 by_ 起头的四个目标
  local narrow_test
  mkdir -p "$narrow_tree/crates/alpha/tests/states"
  for narrow_test in by_path_one by_path_two; do printf '#[path = "states/table.rs"]\nmod table;\nfn main() {}\n' > "$narrow_tree/crates/alpha/tests/$narrow_test.rs"; done
  for narrow_test in by_module_one by_module_two; do printf 'mod states;\nfn main() {}\n' > "$narrow_tree/crates/alpha/tests/$narrow_test.rs"; done
  printf 'mod inner;\n' > "$narrow_tree/crates/alpha/tests/states/mod.rs"
  narrow_patch "$scratch/narrow-subdirectory-path" crates/alpha/tests/states/table.rs
  narrow_patch "$scratch/narrow-subdirectory-module" crates/alpha/tests/states/inner.rs
  narrow_patch "$scratch/narrow-subdirectory-orphan" crates/alpha/tests/states/orphan.rs
  # 只被两个目标经 #[path] 引、mod.rs 不 mod 它：只挑这两个
  out="$scratch/out-narrow-subdirectory-path.txt"; narrow_case subdirectory-path --skip-apply "$scratch/narrow-subdirectory-path"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests subdirectory-path)" != "test --offline --no-fail-fast -p alpha --test by_path_one --test by_path_two|" ]] || grep -qF '退回整目录' "$out"; then
    echo "  ✗ 自检：挑测试目标：子目录里的文件只被两个目标经 #[path] 引、mod.rs 不 mod 它：测试那一步只该起 alpha 的 --test by_path_one 与 --test by_path_two，实际退出码 $rc，假 cargo 收到 $(narrow_tests subdirectory-path)：$(grep -E '判不出|退回整目录|✗' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 经 mod.rs 被引：挑 mod 这个子目录的那两个（经 #[path] 只引 table.rs 的那两个不挑）
  out="$scratch/out-narrow-subdirectory-module.txt"; narrow_case subdirectory-module --skip-apply "$scratch/narrow-subdirectory-module"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests subdirectory-module)" != "test --offline --no-fail-fast -p alpha --test by_module_one --test by_module_two|" ]] || grep -qF '退回整目录' "$out"; then
    echo "  ✗ 自检：挑测试目标：子目录里的文件经 mod.rs 被引：测试那一步只该起 alpha 的 --test by_module_one 与 --test by_module_two，实际退出码 $rc，假 cargo 收到 $(narrow_tests subdirectory-module)：$(grep -E '判不出|退回整目录|✗' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 谁都走不到：退回整目录（by_ 起头的四个），打一行说明
  out="$scratch/out-narrow-subdirectory-orphan.txt"; narrow_case subdirectory-orphan --skip-apply "$scratch/narrow-subdirectory-orphan"; checked=$((checked+1))
  if [[ $rc -ne 0 || "$(narrow_tests subdirectory-orphan)" != "test --offline --no-fail-fast -p alpha --test by_module_one --test by_module_two --test by_path_one --test by_path_two|" ]] \
     || ! grep -qF '没有一个经 mod / #[path] 走得到 tests/states/orphan.rs，退回整目录' "$out"; then
    echo "  ✗ 自检：挑测试目标：子目录里的文件谁都走不到：应当退回整目录、起 alpha 的 by_ 起头的四个目标并打一行退回整目录的说明，实际退出码 $rc，假 cargo 收到 $(narrow_tests subdirectory-orphan)：$(grep -E '判不出|退回整目录|✗' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  rm -rf -- "${narrow_tree:?}/crates/alpha/tests/states" "$narrow_tree/crates/alpha/tests/by_path_one.rs" "$narrow_tree/crates/alpha/tests/by_path_two.rs" \
    "$narrow_tree/crates/alpha/tests/by_module_one.rs" "$narrow_tree/crates/alpha/tests/by_module_two.rs"
  # --all-packages 不是认得的参数（不留整包跑的选项）：给了就退 2、点名它、一样都不做
  out="$scratch/out-narrow-all-packages.txt"; narrow_case all-packages --skip-apply --all-packages "$scratch/narrow-two"; checked=$((checked+1))
  if [[ $rc -ne 2 || -s "$scratch/calls-narrow-all-packages.txt" ]] || ! grep -qF '认不出参数 --all-packages' "$out"; then
    echo "  ✗ 自检：给了 --all-packages（不认的参数）应当退 2、点名它、一样都不做，实际退出码 $rc，假 cargo 收到 $(grep -c . "$scratch/calls-narrow-all-packages.txt" 2>/dev/null) 条"; failures=1; fi   # gate-lint:detail
  # --skip-apply 给的补丁没打在树上（树里那一份还是补丁前的 1）：退 1、点名那份补丁、不编
  narrow_patch "$scratch/narrow-unapplied" crates/alpha/src/unapplied.txt
  printf '1\n' > "$narrow_tree/crates/alpha/src/unapplied.txt"
  # 同一局面原命令点了 --item：出路里拿新补丁目录再验的那一条带同一个 --item（文件头「只重跑的命令点名」）；输出由下面「重跑命令点名」那一格逐条判
  out="$scratch/out-narrow-unapplied-named.txt"; narrow_case unapplied-named --skip-apply --item alpha:src/lib.rs "$scratch/narrow-unapplied"
  out="$scratch/out-narrow-unapplied.txt"; narrow_case unapplied --skip-apply "$scratch/narrow-unapplied"; checked=$((checked+1))
  rm -f "$narrow_tree/crates/alpha/src/unapplied.txt"
  if [[ $rc -ne 1 ]] || ! grep -qF "没打在要验的树" "$out" || ! grep -qF "$scratch/narrow-unapplied" "$out" || [[ -s "$scratch/calls-narrow-unapplied.txt" ]]; then
    echo "  ✗ 自检：--skip-apply 给的补丁没打在树上：应当退 1、点名那份补丁、不编，实际退出码 $rc，假 cargo 收到 $(cat "$scratch/calls-narrow-unapplied.txt" 2>/dev/null | wc -l) 条：$(grep -E '✗|核过|本次未核' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 同一格的输出再核出路（文件头「--skip-apply 先核补丁打过没有」）：倒不掉时写了按此刻树现算补丁的整条命令（patch-of-the-current-tree.py，--out 在这一趟的日志目录下）与拿它 --skip-apply 的整条命令
  #   （弄坏开关 way-out-without-current-tree-patch 下出路里没有这两条，这一格红）
  checked=$((checked+1))
  if ! grep -qF "python3 $root/scripts/gate/patch-of-the-current-tree.py --tree $narrow_tree --out $scratch/logs-narrow-unapplied/current-tree-patch $scratch/narrow-unapplied" "$out" \
     || ! grep -qF "merge-and-verify.sh --skip-apply $scratch/logs-narrow-unapplied/current-tree-patch" "$out"; then
    echo "  ✗ 自检：--skip-apply 核不过的出路里应当写按此刻树现算补丁的整条命令与拿它 --skip-apply 的整条命令：$(grep -E '→' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  # --skip-apply 给的两份补丁改同一行（先打的 1 改成 2，后打的 2 改成 3，树里是两份都打过之后的 3），照打的次序给：临时副本里倒着逐份倒得掉，判都已打上、退 0、树不动
  #   （弄坏开关 reverse-check-each-alone 下先打的那一份单独反着核不过，这一格红）
  mkdir -p "$scratch/narrow-stacked-first" "$scratch/narrow-stacked-second"
  printf -- '--- a/crates/alpha/src/stacked.txt\n+++ b/crates/alpha/src/stacked.txt\n@@ -1 +1 @@\n-1\n+2\n' > "$scratch/narrow-stacked-first/crates.patch"
  printf -- '--- a/crates/alpha/src/stacked.txt\n+++ b/crates/alpha/src/stacked.txt\n@@ -1 +1 @@\n-2\n+3\n' > "$scratch/narrow-stacked-second/crates.patch"
  printf '3\n' > "$narrow_tree/crates/alpha/src/stacked.txt"
  out="$scratch/out-narrow-stacked.txt"; narrow_case stacked --skip-apply "$scratch/narrow-stacked-first" "$scratch/narrow-stacked-second"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -qF '核过 2 份补丁都已打在树上（临时副本里照给的次序倒着逐份 git apply --reverse' "$out" || [[ "$(cat "$narrow_tree/crates/alpha/src/stacked.txt")" != 3 ]]; then
    echo "  ✗ 自检：--skip-apply 给的两份补丁改同一行、照打的次序给：应当判都已打上、退 0、要验的树不动，实际退出码 $rc，树里那一份是「$(cat "$narrow_tree/crates/alpha/src/stacked.txt" 2>/dev/null)」：$(grep -E '✗|核过|本次未核' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  rm -f "$narrow_tree/crates/alpha/src/stacked.txt"
  # 临时目录不认保留属性（假 cp 见 -p、-a、--preserve 带权限就照那种文件系统报 Operation not supported）：--skip-apply 照样核得过、退 0
  #   （弄坏开关 reverse-check-copy-preserves 下带 -p 拷，这一格红）
  mkdir -p "$scratch/fake-cp-no-preserve"; real_cp="$(command -v cp)"
  printf '#!/usr/bin/env bash\nfor a in "$@"; do case "$a" in -p|-a|-*p*|--preserve|--preserve=*mode*|--preserve=all|--archive) [[ "$a" == --no-preserve* || "$a" == --parents ]] && continue; echo "cp: preserving permissions for ‘${@: -1}’: Operation not supported" >&2; exit 1;; esac; done\nexec %q "$@"\n' "$real_cp" > "$scratch/fake-cp-no-preserve/cp"
  chmod +x "$scratch/fake-cp-no-preserve/cp"
  narrow_patch "$scratch/narrow-no-preserve" crates/alpha/src/no-preserve.txt
  out="$scratch/out-narrow-no-preserve.txt"; PATH="$scratch/fake-cp-no-preserve:$PATH" narrow_case no-preserve --skip-apply "$scratch/narrow-no-preserve"; checked=$((checked+1))
  rm -f "$narrow_tree/crates/alpha/src/no-preserve.txt"
  if [[ $rc -ne 0 ]] || ! grep -qF '核过 1 份补丁都已打在树上' "$out"; then
    echo "  ✗ 自检：临时目录不认保留属性时 --skip-apply 照样核得过补丁已打上、退 0，实际退出码 $rc：$(grep -E '✗|核过|拷不进' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # check.sh 读不出 lint 表、clippy 那一行没带 lint 表、fmt 那一条换成认不出的包装、test 那一行带了收窄目标的标志、缺省特性组合只剩一个（照它 CHECK_BREAK=single-clippy-pass 的写法）、
  # 循环改成只取一个组合：退 1，一个补丁都不打、不编
  sed '/^CODE_DISCIPLINE_LINTS=(/,/^)/d' "$CHECK_SH" > "$scratch/check-no-table.sh"
  sed 's/ "\${CODE_DISCIPLINE_LINTS\[@\]}" \\$/ \\/' "$CHECK_SH" > "$scratch/check-clippy-lints-dropped.sh"
  sed 's/^\(run_cargo\|cargo\) fmt /"${cargo_command[@]}" fmt /' "$CHECK_SH" > "$scratch/check-fmt-unknown-wrapper.sh"
  sed 's/\(cargo test --all\) 2>&1/\1 --lib 2>\&1/' "$CHECK_SH" > "$scratch/check-test-narrowed.sh"
  sed 's/^CLIPPY_FEATURE_COMBINATIONS=(.*)[[:space:]]*$/CLIPPY_FEATURE_COMBINATIONS=("--all-features")/' "$CHECK_SH" > "$scratch/check-single-combination.sh"
  sed 's/for clippy_combination in "\${CLIPPY_FEATURE_COMBINATIONS\[@\]}"; do/for clippy_combination in "${CLIPPY_FEATURE_COMBINATIONS[@]:0:1}"; do/' "$CHECK_SH" > "$scratch/check-combination-loop-one.sh"
  for check_case in no-table clippy-lints-dropped fmt-unknown-wrapper test-narrowed single-combination combination-loop-one; do
    case "$check_case" in
      no-table) check_case_text='里读不出 lint 表' ;;
      clippy-lints-dropped) check_case_text='的 clippy 那一行没带 lint 表' ;;
      fmt-unknown-wrapper) check_case_text='的 fmt 那一条换成认不出的包装' ;;
      test-narrowed) check_case_text='的 test 那一行带了收窄目标的标志 --lib ' ;;   # 结尾留一个空格：拼上「时应当」后 --lib 与「时」之间要有空格，与文件头点名的原文一致
      single-combination) check_case_text='的缺省特性组合只剩一个' ;;
      combination-loop-one) check_case_text='的 clippy 循环改成只取一个组合' ;;
    esac
    out="$scratch/out-check-$check_case.txt"; checked=$((checked+1))
    if cmp -s "$CHECK_SH" "$scratch/check-$check_case.sh"; then   # 样本造不出（check.sh 里没有要改的那一行）：这一格不算判过
      echo "  ✗ 自检：check.sh ${check_case_text}时应当退 1、不编，样本没造出来（sed 在 $CHECK_SH 里一处都没改到）"; failures=1; continue; fi   # gate-lint:detail
    FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-check-$check_case.txt" MERGE_AND_VERIFY_CHECK_SH="$scratch/check-$check_case.sh" run_case "check-$check_case" --skip-apply
    if [[ $rc -ne 1 ]] || ! grep -q 'check.sh 读不出、或与这里对不上' "$out" || [[ -s "$scratch/calls-check-$check_case.txt" ]]; then
      echo "  ✗ 自检：check.sh ${check_case_text}时应当退 1、不编，实际退出码 $rc，假 cargo 收到 $(cat "$scratch/calls-check-$check_case.txt" 2>/dev/null | wc -l) 条：$(grep -E '✗|!' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  done
  # cargo 命令现读（文件头「cargo 命令现读」）：check.sh 只换包装、或给命令加标志时，不判红，照读到的写法跑。基线是真 check.sh 同一趟收到的假 cargo 命令；
  #   这三趟都给补丁 patch-a（改到 demo，四条 cargo 命令都起得来：build 只在补丁改到包时才起，文件头 build 一段）
  out="$scratch/out-read-baseline.txt"; checked=$((checked+1))
  FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-read-baseline.txt" run_case read-baseline "$scratch/patch-a"
  baseline_calls="$(sort "$scratch/calls-read-baseline.txt" 2>/dev/null)"
  if [[ $rc -ne 0 || -z "$baseline_calls" ]]; then
    echo "  ✗ 自检：cargo 命令现读：真 check.sh 同一趟应当退 0、假 cargo 收到命令（后面两格拿它当基线），实际退出码 $rc：$(grep -E '✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 包装：命令位置上写成 run_cargo <子命令>（check.sh 此刻不是这样就改成这样，已经是这样就不变），收到的命令要与基线逐条相同
  sed 's/^\([[:space:]]*\(if \)\?\)\(run_cargo\|cargo\) \(fmt\|clippy\|build\|test\) /\1run_cargo \4 /' "$CHECK_SH" > "$scratch/check-wrapper-run-cargo.sh"
  out="$scratch/out-wrapper-run-cargo.txt"; checked=$((checked+1))
  if [[ "$(grep -cE '^[[:space:]]*(if[[:space:]]+)?run_cargo[[:space:]]+(fmt|clippy|build|test)[[:space:]]' "$scratch/check-wrapper-run-cargo.sh")" != 4 ]]; then
    echo "  ✗ 自检：check.sh 的 cargo 命令包了 run_cargo 时应当照读、收到的命令与真 check.sh 相同，样本没造出来（sed 之后 $CHECK_SH 里命令位置上的 run_cargo <子命令> 不是 4 条）"; failures=1   # gate-lint:detail
  else
    FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-wrapper-run-cargo.txt" MERGE_AND_VERIFY_CHECK_SH="$scratch/check-wrapper-run-cargo.sh" run_case wrapper-run-cargo "$scratch/patch-a"
    if [[ $rc -ne 0 || "$(sort "$scratch/calls-wrapper-run-cargo.txt" 2>/dev/null)" != "$baseline_calls" ]]; then
      echo "  ✗ 自检：check.sh 的 cargo 命令包了 run_cargo 时应当照读、收到的命令与真 check.sh 相同，实际退出码 $rc，假 cargo 收到 $(wc -l < "$scratch/calls-wrapper-run-cargo.txt" 2>/dev/null) 条（基线 $(wc -l <<< "$baseline_calls") 条）：$(grep -E '✗|!' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  fi
  # 加标志：四条命令各加一个标志（fmt --verbose、clippy -W clippy::pedantic、build --locked、test --locked），收到的命令要带上它们（读来的，不是这里写死的）
  sed -e 's/cargo fmt --all -- --check/cargo fmt --all --verbose -- --check/' \
      -e 's/-- -D warnings "\${CODE_DISCIPLINE_LINTS\[@\]}"/-- -D warnings -W clippy::pedantic "${CODE_DISCIPLINE_LINTS[@]}"/' \
      -e 's/cargo build --all-targets /cargo build --all-targets --locked /' \
      -e 's/\(cargo test --all\) 2>&1/\1 --locked 2>\&1/' "$CHECK_SH" > "$scratch/check-flags-added.sh"
  out="$scratch/out-flags-added.txt"; checked=$((checked+1))
  if [[ "$(diff "$CHECK_SH" "$scratch/check-flags-added.sh" | grep -c '^>')" != 4 ]]; then
    echo "  ✗ 自检：check.sh 的四条 cargo 命令各加一个标志时应当照读、收到的命令带上它们，样本没造出来（sed 在 $CHECK_SH 里没改到四行）"; failures=1   # gate-lint:detail
  else
    FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-flags-added.txt" MERGE_AND_VERIFY_CHECK_SH="$scratch/check-flags-added.sh" run_case flags-added "$scratch/patch-a"
    flagged_calls="$(cat "$scratch/calls-flags-added.txt")"
    flagged_clippy_count="$(grep -cF -- ' -- -D warnings -W clippy::pedantic -D ' <<< "$flagged_calls")"
    if [[ $rc -ne 0 ]] || ! grep -qxF -- "fmt --all --verbose -- --check" <<< "$flagged_calls" || ! grep -qxF -- "build --offline --all-targets --locked -p demo" <<< "$flagged_calls" \
       || ! grep -qxF -- "test --offline --no-fail-fast --locked -p demo --lib" <<< "$flagged_calls" || [[ "$flagged_clippy_count" != "${#expected_combinations[@]}" ]]; then
      echo "  ✗ 自检：check.sh 的四条 cargo 命令各加一个标志时应当照读、收到的命令带上它们（fmt --verbose、build --locked、test --locked、每个组合的 clippy 都带 -W clippy::pedantic），实际退出码 $rc，clippy 带上的有 $flagged_clippy_count 条（要 ${#expected_combinations[@]} 条），假 cargo 收到：$(tr '\n' '|' <<< "$flagged_calls")"; failures=1; fi   # gate-lint:detail
  fi
  # 工作区成员里没有 singlefs-verification（样本里那个成员改了名）：认不出验证包，退 1、不编
  write_workspace "$scratch/tree-renamed" singlefs-verification-renamed
  mkdir -p "$scratch/tree-renamed/crates/demo/src"; printf 'pub fn f() {}\n' > "$scratch/tree-renamed/crates/demo/src/lib.rs"
  out="$scratch/out-renamed.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-renamed.txt" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-renamed" MERGE_AND_VERIFY_TREE="$scratch/tree-renamed" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  if [[ $rc -ne 1 ]] || ! grep -q '认不出它就排不掉登记的崩溃枚举用例' "$out" || grep -q '^test' "$scratch/calls-renamed.txt" 2>/dev/null; then
    echo "  ✗ 自检：工作区成员里没有 singlefs-verification 时应当退 1、不编，实际退出码 $rc：$(grep '^test' "$scratch/calls-renamed.txt" 2>/dev/null | tr '\n' ' ')$(grep -E '✗' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  # cargo test 退 0 而一条测试都没跑（假 cargo 什么都不打）：退 1
  out="$scratch/out-empty.txt"; checked=$((checked+1)); FAKE_CARGO_MODE=empty run_case empty --skip-apply "$scratch/patch-demo-lib"
  if [[ $rc -ne 1 ]] || ! grep -q '一条测试都没跑' "$out"; then
    echo "  ✗ 自检：cargo test 退 0 却一条测试都没跑时应当退 1，实际退出码 $rc：$(grep -E '✓ 测试|✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 没设日志目录：日志落在工作根下（SINGLEFS_WORK_ROOT 指进临时目录）
  out="$scratch/out4.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" FAKE_CARGO_MODE=green MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" SINGLEFS_WORK_ROOT="$scratch/work-root" \
    MERGE_AND_VERIFY_TREE="$scratch/tree" env -u MERGE_AND_VERIFY_LOG_DIRECTORY -u CARGO_TARGET_DIR bash "$0" "$scratch/patch-a" > "$out" 2>&1; rc=$?
  if [[ $rc -ne 0 ]] || ! compgen -G "$scratch/work-root/merge-and-verify/*/test.log" > /dev/null; then
    echo "  ✗ 自检：没设日志目录时日志应当落在工作根下（$scratch/work-root/merge-and-verify/<日期>-<进程号>/），实际退出码 $rc：$(grep '日志在' "$out" | tail -1)"; failures=1; fi   # gate-lint:detail
  # 换了源码却没重编（调用方给的编译目录 <树>/target 两趟共用，收尾不删；快照不拷它）：临时树先验一趟（编过），再从源树 rsync -a 铺进新的 walk.rs（mtime 比编译目录里的 dep-info 旧，取 2020 年初的 epoch 秒）；
  # 假 cargo 照 mtime 判新不新鲜（source-stamp.py --write-selftest-cargo 写的那一份）：快照里的修改时刻是拷的那一刻（文件头「快照」），walk.rs 照新内容重编；
  # 快照带着源树的旧 mtime（弄坏开关 snapshot-keeps-times）就认 walk.rs 已编、那一条 FAILED
  local stale_tree="$scratch/stale-tree" stale_source="$scratch/stale-source" stale_bin="$scratch/stale-bin"
  mkdir -p "$stale_tree/crates/demo/src" "$stale_source/crates/demo/src" "$stale_bin"
  write_workspace "$stale_tree" singlefs-verification
  python3 "$STAMP" --write-selftest-cargo "$stale_bin/cargo"
  printf 'pub mod walk;\n' > "$stale_tree/crates/demo/src/lib.rs"
  printf 'pub fn walk() -> u32 { 1 }\n' > "$stale_tree/crates/demo/src/walk.rs"
  touch -d @1577836800 "$stale_tree/crates/demo/src/lib.rs" "$stale_tree/crates/demo/src/walk.rs"
  cp --preserve=timestamps "$stale_tree/crates/demo/src/lib.rs" "$stale_source/crates/demo/src/lib.rs"
  printf 'pub fn walk() -> u32 { 22 }\n' > "$stale_source/crates/demo/src/walk.rs"
  touch -d @1577836900 "$stale_source/crates/demo/src/walk.rs"
  stale_case() { PATH="$stale_bin:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-$1" \
    MERGE_AND_VERIFY_TREE="$stale_tree" env CARGO_TARGET_DIR="$stale_tree/target" bash "$0" --skip-apply "$scratch/patch-demo-lib" > "$out" 2>&1; rc=$?; }
  out="$scratch/out5-first.txt"; stale_case five-first
  rsync -a "$stale_source/" "$stale_tree/"
  out="$scratch/out5.txt"; stale_case five; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q 'Compiling crates/demo/src/walk.rs' "$scratch/logs-five/test.log" 2>/dev/null || ! grep -q '测试全绿' "$out"; then
    echo "  ✗ 自检：往编过的副本里 rsync -a 铺新内容（旧 mtime）之后再验，应当重编改过的 walk.rs、测试全绿，实际退出码 $rc：$(grep -E 'Fresh|Compiling|FAILED' "$scratch/logs-five/test.log" 2>/dev/null | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 改过的包重编了没有（文件头；调用方给的编译目录 <树>/target 几趟共用，收尾不删）：假 cargo 按包、按 mtime 判新不新鲜，重编时照真 cargo 打「Compiling <包名> v…」（lib_rebuilt_packages.py --write-selftest-cargo）；
  # 补丁目录 rebuilt-patch 的 crates.patch 改的是 crates/demo/src/walk.rs（树里已经是改过的样子，--skip-apply 只验不打）
  local rebuilt_tree="$scratch/rebuilt-tree" rebuilt_bin="$scratch/rebuilt-bin" rebuilt_patch="$scratch/rebuilt-patch"
  mkdir -p "$rebuilt_tree/crates/demo/src" "$rebuilt_bin" "$rebuilt_patch"
  write_workspace "$rebuilt_tree" singlefs-verification
  printf 'pub mod walk;\n' > "$rebuilt_tree/crates/demo/src/lib.rs"
  printf 'pub fn walk() -> u32 { 22 }\n' > "$rebuilt_tree/crates/demo/src/walk.rs"
  printf 'diff --git a/crates/demo/src/walk.rs b/crates/demo/src/walk.rs\n--- a/crates/demo/src/walk.rs\n+++ b/crates/demo/src/walk.rs\n@@ -1 +1 @@\n-pub fn walk() -> u32 { 1 }\n+pub fn walk() -> u32 { 22 }\n' \
    > "$rebuilt_patch/crates.patch"
  python3 "$REBUILT" --write-selftest-cargo "$rebuilt_bin/cargo"
  rebuilt_case() { PATH="$rebuilt_bin:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-$1" \
    MERGE_AND_VERIFY_TREE="$rebuilt_tree" env CARGO_TARGET_DIR="$rebuilt_tree/target" bash "$0" --skip-apply "$rebuilt_patch" > "$out" 2>&1; rc=$?; }
  # 绿样本：头一次编（编译目录是空的），demo 有 Compiling 行
  out="$scratch/out-rebuilt-first.txt"; rebuilt_case rebuilt-first; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q '改过的包这一趟都重编了（核了 1 个包：demo）' "$out"; then
    echo "  ✗ 自检：头一次编、补丁改过的 demo 有 Compiling 行：应当退 0、报都重编了，实际退出码 $rc：$(grep -E '✗|✓|重编' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 编过之后再验同一批补丁：快照里的修改时刻是拷的那一刻，比上一趟的 dep-info 新，demo 照样重编（弄坏开关 snapshot-keeps-times 下快照带着源树的旧 mtime，demo 不重编）
  out="$scratch/out-rebuilt-rerun.txt"; rebuilt_case rebuilt-rerun; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q '改过的包这一趟都重编了（核了 1 个包：demo）' "$out"; then
    echo "  ✗ 自检：编过之后 --skip-apply 再验同一批补丁：改过的 demo 应当重编、退 0，实际退出码 $rc：$(grep -E '✗|✓|重编|刷到此刻' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 红样本：编译目录里 demo 的产物比快照的拷入时刻还新（别的趟共用这个编译目录、后编的那种）；
  # 验证包那个包的产物是上一趟编的、比快照旧，照常重编（有一行别的 Compiling），demo 没有 Compiling 行，测试照样全绿
  touch -d '+1 day' "$rebuilt_tree/target/debug/.fake/demo.d"
  touch "$rebuilt_tree/crates/singlefs-verification/Cargo.toml"
  out="$scratch/out-rebuilt-stale.txt"; rebuilt_case rebuilt-stale; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || ! grep -q '改过的包这一趟没重编：demo' "$out" || ! grep -q 'Compiling singlefs-verification' "$scratch/logs-rebuilt-stale/test.log" 2>/dev/null; then
    echo "  ✗ 自检：编译目录里的产物比源码新（共用编译目录）、补丁改过的 demo 没有 Compiling 行：应当退 1、点名 demo，实际退出码 $rc：$(grep -E '✗|✓|重编' "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 跑完运行目录里不剩它建的编译目录与快照（文件头「这一趟的编译目录」「快照」）：没设 CARGO_TARGET_DIR 的三种结局——全绿退 0（out3 那一趟）、测试红退 1（out1 那一趟）、
  # 被 TERM 停（假 cargo test 睡 2 秒，时间线里出现它开始之后给脚本发 TERM，脚本等假 cargo 退出后走 EXIT trap，退 143）——它开跑时打的编译目录都不在、输出里有删了它那一行；
  # 调用方设了 CARGO_TARGET_DIR 的（上面重编核对那几趟）照它编、收尾不删
  local term_pid term_rc term_attempt removal_problems="" three_target one_target term_target
  three_target="$(run_target_of "$scratch/out3.txt")"; one_target="$(run_target_of "$scratch/out1.txt")"
  [[ -n "$three_target" && ! -e "$three_target" ]] && grep -qF "删了这一趟的编译目录 $three_target" "$scratch/out3.txt" || removal_problems+="全绿那一趟（out3）还剩 $(ls -d "${three_target:-没打编译目录那一行}" 2>/dev/null || echo 没剩、但没打删了那一行)；"
  [[ -n "$one_target" && ! -e "$one_target" ]] && grep -qF "删了这一趟的编译目录 $one_target" "$scratch/out1.txt" || removal_problems+="测试红那一趟（out1）还剩 $(ls -d "${one_target:-没打编译目录那一行}" 2>/dev/null || echo 没剩、但没打删了那一行)；"
  PATH="$rebuilt_bin:$PATH" FAKE_CARGO_TIMELINE="$scratch/term-timeline.txt" FAKE_CARGO_TAG=T FAKE_CARGO_SLEEP=2 MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-term" MERGE_AND_VERIFY_TREE="$rebuilt_tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply "$rebuilt_patch" > "$scratch/out-term.txt" 2>&1 &
  term_pid=$!
  for term_attempt in $(seq 1 200); do grep -q '^start T ' "$scratch/term-timeline.txt" 2>/dev/null && break; kill -0 "$term_pid" 2>/dev/null || break; sleep 0.05; done
  kill -TERM "$term_pid" 2>/dev/null
  if wait "$term_pid"; then term_rc=0; else term_rc=$?; fi
  term_target="$(run_target_of "$scratch/out-term.txt")"
  ((term_rc == 143)) && [[ -n "$term_target" && ! -e "$term_target" ]] && grep -qF "删了这一趟的编译目录 $term_target" "$scratch/out-term.txt" \
    || removal_problems+="被 TERM 停的那一趟退 $term_rc（要 143），还剩 $(ls -d "${term_target:-没打编译目录那一行}" 2>/dev/null || echo 没剩)：$(tail -2 "$scratch/out-term.txt" | tr '\n' ' ')；"
  [[ -d "$rebuilt_tree/target/debug" ]] && ! grep -q '删了这一趟的编译目录' "$scratch/out-rebuilt-first.txt" || removal_problems+="调用方设的 CARGO_TARGET_DIR 收尾被删了或打了删了那一行；"
  local snapshot_label
  for snapshot_label in three one term; do
    [[ ! -e "$scratch/logs-$snapshot_label/repo" ]] || removal_problems+="$snapshot_label 那一趟的快照 $scratch/logs-$snapshot_label/repo 还在；"
  done
  checked=$((checked+1))
  if [[ -n "$removal_problems" ]]; then
    echo "  ✗ 自检：跑完运行目录里不剩它建的编译目录与快照（正常、出错、被 TERM 三种结局都删它开跑时打的那一份与快照，调用方设的 CARGO_TARGET_DIR 不删），实际：$removal_problems"; failures=1; fi   # gate-lint:detail
  # 这一趟的编译目录里还有进程开着文件（假 cargo test 留下一个后台 sleep 开着 <编译目录>/held，进程号写进 holder-pid）：不删、报出那个进程、退 1；看完只停那一个进程号
  local holder_bin="$scratch/holder-bin" holder_pid
  mkdir -p "$holder_bin"
  printf '#!/usr/bin/env bash\nif [[ "$1" == test ]]; then mkdir -p "$CARGO_TARGET_DIR"; sleep 30 > "$CARGO_TARGET_DIR/held" 2>&1 < /dev/null & echo "$!" > "$FAKE_HOLDER_PID"; fi\nexec python3 %q "$@"\n' "$rebuilt_bin/cargo" > "$holder_bin/cargo"
  chmod +x "$holder_bin/cargo"
  out="$scratch/out-holder.txt"; checked=$((checked+1))
  PATH="$holder_bin:$PATH" FAKE_HOLDER_PID="$scratch/holder-pid" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-holder" MERGE_AND_VERIFY_TREE="$rebuilt_tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply "$rebuilt_patch" > "$out" 2>&1; rc=$?
  holder_pid="$(cat "$scratch/holder-pid" 2>/dev/null)"
  local holder_target; holder_target="$(run_target_of "$out")"
  if [[ $rc -ne 1 || -z "$holder_target" || ! -d "$holder_target" ]] || ! grep -q "还有 1 个进程开着里面的文件" "$out" || ! grep -q "进程 ${holder_pid:-没有}（sleep 30）" "$out"; then
    echo "  ✗ 自检：这一趟的编译目录里还有进程开着文件：应当不删、报出那个进程、退 1，实际退出码 $rc，编译目录$([[ -n "$holder_target" && -d "$holder_target" ]] && echo 还在 || echo 删了)：$(grep -E '✗|删了|进程' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  [[ ! "$holder_pid" =~ ^[0-9]+$ ]] || kill "$holder_pid" 2>/dev/null
  # 编译目录不在工作根下（文件头「这一趟的编译目录」）：不给日志目录、工作根指进 scratch，编译目录要是 TMPDIR 下这一趟新建的那一份，日志目录照旧建在工作根下
  local placement_target placement_logs physical_temporary physical_work_root
  mkdir -p "$scratch/work-root"
  physical_temporary="$(cd "$TMPDIR" && pwd -P)"; physical_work_root="$(cd "$scratch/work-root" && pwd -P)"
  out="$scratch/out-placement.txt"; checked=$((checked+1))
  PATH="$rebuilt_bin:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" SINGLEFS_WORK_ROOT="$scratch/work-root" MERGE_AND_VERIFY_TREE="$rebuilt_tree" \
    env -u CARGO_TARGET_DIR -u MERGE_AND_VERIFY_LOG_DIRECTORY bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  placement_target="$(run_target_of "$out")"
  placement_logs="$(ls -d "$scratch/work-root/merge-and-verify/"*/ 2>/dev/null | head -1)"
  if [[ -z "$placement_target" || "$placement_target" == "$scratch/work-root/"* || "$placement_target" == "$physical_work_root/"* \
        || "$placement_target" != "$physical_temporary/"* || -z "$placement_logs" ]]; then
    echo "  ✗ 自检：编译目录不在工作根下：应当建在 TMPDIR（$physical_temporary）下这一趟新建的一份、日志目录照旧在工作根下，实际退出码 $rc，编译目录「$placement_target」，日志目录「$placement_logs」"; failures=1; fi   # gate-lint:detail
  # 收尾时前面一步写满盘（MERGE_AND_VERIFY_SELFTEST_FULL_DISK 演：删快照那一步写不进去、整条收尾停在那里）：编译目录在那之前删过，照删
  local full_disk_target
  out="$scratch/out-full-disk.txt"; checked=$((checked+1))
  PATH="$rebuilt_bin:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_SELFTEST_FULL_DISK=1 \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-full-disk" MERGE_AND_VERIFY_TREE="$rebuilt_tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply > "$out" 2>&1; rc=$?
  full_disk_target="$(run_target_of "$out")"
  if [[ $rc -ne 1 || -z "$full_disk_target" || -e "$full_disk_target" ]] || ! grep -qF 'No space left on device' "$out"; then
    echo "  ✗ 自检：收尾时前面一步写满盘（用开关演）编译目录照删：应当退 1、编译目录不在，实际退出码 $rc，编译目录「$full_disk_target」$([[ -n "$full_disk_target" && -e "$full_disk_target" ]] && echo 还在 || echo 不在)"; failures=1; fi   # gate-lint:detail
  # 槽位（文件头「槽位」）：样本树 slot-template（demo 的 lib.rs、walk.rs、notes.txt，验证包的 setting.rs），每格拷一份当要验的树、各给一个工作根与这一台的编译根
  #   （MERGE_AND_VERIFY_SLOT_BUILD_ROOT），照 rebuilt-patch 那份补丁 --skip-apply 验；假 cargo 是 rebuilt-bin 那一份（按包、按 mtime 判新不新鲜）。
  #   这几格不认工作根的弄坏开关（env -u SINGLEFS_WORK_ROOT_BREAK）：工作根是格自己给的，只判槽
  local slot_template="$scratch/slot-template" slot_table_two="$scratch/slot-table-two.conf" slot_table_one="$scratch/slot-table-one.conf" slot_table_trim="$scratch/slot-table-trim.conf"
  local slot_taken slot_target slot_problems slot_held_pid slot_held_rc slot_attempt slot_root_physical slot_build_physical
  mkdir -p "$slot_template/crates/demo/src" "$slot_template/crates/singlefs-verification/src"
  write_workspace "$slot_template" singlefs-verification
  printf 'pub mod walk;\n' > "$slot_template/crates/demo/src/lib.rs"
  printf 'pub fn walk() -> u32 { 22 }\n' > "$slot_template/crates/demo/src/walk.rs"
  printf 'notes\n' > "$slot_template/crates/demo/notes.txt"
  printf 'pub fn setting() -> u32 { 1 }\n' > "$slot_template/crates/singlefs-verification/src/setting.rs"
  printf 'slots=2\nslot-target-limit-gib=1000\n' > "$slot_table_two"
  printf 'slots=1\nslot-target-limit-gib=1000\n' > "$slot_table_one"
  printf 'slots=2\nslot-target-limit-gib=0\n' > "$slot_table_trim"
  slot_tree() { cp -r -- "$slot_template" "$scratch/slot-tree-$1"; }   # <格名>：这一格要验的树
  slot_case() {   # <格名> <这一趟的标签> <槽位登记> [交给 env 的赋值…]：在这一格的工作根与编译根上验一趟，输出 out-slot-<标签>.txt、日志 logs-slot-<标签>；调用方环境里的 CARGO_INCREMENTAL 不带进来
    out="$scratch/out-slot-$2.txt"
    PATH="$rebuilt_bin:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-slot-$2" MERGE_AND_VERIFY_TREE="$scratch/slot-tree-$1" \
      SINGLEFS_WORK_ROOT="$scratch/slot-work-$1" MERGE_AND_VERIFY_SLOT_TABLE="$3" MERGE_AND_VERIFY_SLOT_BUILD_ROOT="$scratch/slot-build-$1" \
      env -u CARGO_TARGET_DIR -u CARGO_INCREMENTAL -u SINGLEFS_WORK_ROOT_BREAK "${@:4}" bash "$0" --skip-apply "$rebuilt_patch" > "$out" 2>&1; rc=$?
  }
  slot_of() { head -n 1 -- "$scratch/logs-slot-$1/slot" 2>/dev/null; }   # <标签>：那一趟占的槽（没占槽是空）
  slot_target_of() { sed -n 's/^  这一槽的编译目录：\(.*\)（在这一台本地.*$/\1/p' "$scratch/out-slot-$1.txt" 2>/dev/null | head -1; }   # <标签>：那一趟打的这一槽的编译目录
  slot_lock_free() { python3 -c 'import fcntl, sys; handle = open(sys.argv[1], "a"); fcntl.lockf(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)' "$1/lock" 2>/dev/null; }   # <槽>：锁拿得到退 0
  # 槽位登记写坏（slots 不是非负整数）：退 1、点名那一项，一个补丁都不打、日志目录都不建
  printf 'slots=two\nslot-target-limit-gib=1\n' > "$scratch/slot-table-broken.conf"
  slot_tree table; checked=$((checked+1))
  slot_case table table "$scratch/slot-table-broken.conf"
  if [[ $rc -ne 1 || -e "$scratch/logs-slot-table" ]] || ! grep -qF 'slots 的值「two」不是非负整数' "$out"; then
    echo "  ✗ 自检：槽位：槽位登记写坏（slots 不是非负整数）时应当退 1、点名那一项、一个补丁都不打，实际退出码 $rc，日志目录$([[ -e "$scratch/logs-slot-table" ]] && echo 建了 || echo 没建)：$(grep -E '✗|不占槽' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 占槽、放槽：头一趟新开一槽，快照在 <工作根>/merge-and-verify/slot.<随机>/repo、已 git init，编译目录在这一台的编译根下的 merge-and-verify-slots/<槽名>/target、编过
  #   （假 cargo 写了 demo 的 dep-info），收尾放了锁、没记成留着，日志目录记下这一槽
  slot_tree hold; checked=$((checked+1)); slot_problems=""
  slot_case hold hold "$slot_table_two"
  slot_taken="$(slot_of hold)"; slot_target="$(slot_target_of hold)"
  mkdir -p "$scratch/slot-work-hold/merge-and-verify" "$scratch/slot-build-hold"
  slot_root_physical="$(cd "$scratch/slot-work-hold/merge-and-verify" && pwd -P)"; slot_build_physical="$(cd "$scratch/slot-build-hold" && pwd -P)"
  [[ $rc -eq 0 ]] || slot_problems+="退出码 $rc；"
  [[ -n "$slot_taken" && "$(dirname -- "$slot_taken")" == "$slot_root_physical" && "$(basename -- "$slot_taken")" == slot.* ]] || slot_problems+="占的槽「$slot_taken」不在 $slot_root_physical 下一层；"
  [[ -n "$slot_taken" && -f "$slot_taken/repo/crates/demo/src/walk.rs" && -d "$slot_taken/repo/.git" ]] || slot_problems+="槽里的快照没拷全或没 git init；"
  [[ -n "$slot_target" && "$slot_target" == "$slot_build_physical/merge-and-verify-slots/$(basename -- "${slot_taken:-没占槽}")/target" && -f "$slot_target/debug/.fake/demo.d" ]] \
    || slot_problems+="编译目录「$slot_target」不是这一台编译根下这一槽的那一份，或没编过；"
  [[ -z "$slot_taken" ]] || slot_lock_free "$slot_taken" || slot_problems+="收尾之后锁还占着；"
  [[ -z "$slot_taken" || ! -e "$slot_taken/kept" ]] || slot_problems+="没带 MERGE_AND_VERIFY_KEEP_COPY=1 却记成了留着；"
  grep -qF "放了槽 ${slot_taken:-没占槽}" "$out" || slot_problems+="没有放了槽那一行；"
  if [[ -n "$slot_problems" ]]; then
    echo "  ✗ 自检：槽位：占槽、放槽，槽的编译目录在这一台的编译根下：应当新开一槽、快照在槽里、编译目录在 $slot_build_physical 下、收尾放锁，实际：$slot_problems"; failures=1; fi   # gate-lint:detail
  # 留着的槽：登记一槽；带 MERGE_AND_VERIFY_KEEP_COPY=1 的那一趟收尾记成留着（kept、槽里的 .owner），打出放槽那一条；下一趟没空槽（唯一那一槽留着），
  #   不占槽、照一趟一份跑完退 0，留着的那一槽原样；放了槽（删 kept、.owner）再起一趟就接着用它
  slot_tree kept; checked=$((checked+1)); slot_problems=""
  slot_case kept kept-first "$slot_table_one" MERGE_AND_VERIFY_KEEP_COPY=1 WORK_DIRECTORY_OWNER_SESSION=slot-selftest-session
  slot_taken="$(slot_of kept-first)"
  [[ $rc -eq 0 && -n "$slot_taken" && -f "$slot_taken/kept" ]] || slot_problems+="留着快照的那一趟退 $rc、占的槽「$slot_taken」$([[ -n "$slot_taken" && -f "$slot_taken/kept" ]] || echo 没记成留着)；"
  [[ -n "$slot_taken" && "$(sed -n 1p "$slot_taken/.owner" 2>/dev/null)" == "session: slot-selftest-session" ]] || slot_problems+="留着的槽里没写归属记录 .owner；"
  grep -qF "→ 用完放槽：rm -rf -- '${slot_taken:-没占槽}/kept' '${slot_taken:-没占槽}/.owner' '${slot_taken:-没占槽}/.deliverables' '${slot_taken:-没占槽}/repo/target'" "$out" \
    || slot_problems+="没打出放槽那一条（四样：kept、.owner、.deliverables、repo/target）；"
  slot_case kept kept-busy "$slot_table_one"
  if [[ $rc -ne 0 || -n "$(slot_of kept-busy)" || -z "$slot_taken" || ! -f "$slot_taken/kept" ]] || ! grep -q '^  ! 这一趟不占槽：' "$out"; then
    slot_problems+="没空槽的那一趟退 $rc、占的槽「$(slot_of kept-busy)」、不占槽那一行 $(grep -c '^  ! 这一趟不占槽：' "$out") 行，留着的那一槽$([[ -n "$slot_taken" && -f "$slot_taken/kept" ]] && echo 还留着 || echo 不留着了)；"
  fi
  [[ -z "$slot_taken" ]] || rm -f -- "$slot_taken/kept" "$slot_taken/.owner"
  slot_case kept kept-back "$slot_table_one"
  [[ $rc -eq 0 && -n "$slot_taken" && "$(slot_of kept-back)" == "$slot_taken" ]] || slot_problems+="放了槽之后那一趟退 $rc、占的槽「$(slot_of kept-back)」不是放了的那一槽；"
  if [[ -n "$slot_problems" ]]; then
    echo "  ✗ 自检：槽位：留着的槽不给别的趟、没空槽时这一趟照一趟一份、放了槽再接着用：实际 $slot_problems"; failures=1; fi   # gate-lint:detail
  # 同一槽先后两趟：头一趟新开一槽、两个包都编；源树里只改 demo 的 lib.rs 再起一趟：还是那一槽，只有 demo 有 Compiling 行，验证包没有，重编核对核了 demo 一个包
  slot_tree incremental; checked=$((checked+1)); slot_problems=""
  slot_case incremental incremental-first "$slot_table_two"
  slot_taken="$(slot_of incremental-first)"
  [[ $rc -eq 0 && -n "$slot_taken" ]] || slot_problems+="头一趟退 $rc、占的槽「$slot_taken」；"
  printf '// edited\n' >> "$scratch/slot-tree-incremental/crates/demo/src/lib.rs"
  slot_case incremental incremental-again "$slot_table_two"
  [[ $rc -eq 0 && -n "$slot_taken" && "$(slot_of incremental-again)" == "$slot_taken" ]] || slot_problems+="再起的那一趟退 $rc、占的槽「$(slot_of incremental-again)」；"
  grep -q 'Compiling demo ' "$scratch/logs-slot-incremental-again/test.log" 2>/dev/null || slot_problems+="改过的 demo 没有 Compiling 行；"
  ! grep -q 'Compiling singlefs-verification ' "$scratch/logs-slot-incremental-again/test.log" 2>/dev/null || slot_problems+="没改的验证包也重编了；"
  grep -qF '改过的包这一趟都重编了（核了 1 个包：demo）' "$out" || slot_problems+="重编核对没报核了 demo 一个包：$(grep -E '重编' "$out" | head -2 | tr '\n' ' ')；"
  if [[ -n "$slot_problems" ]]; then
    echo "  ✗ 自检：槽位：同一槽先后两趟，下一趟只重编改过的包：实际 $slot_problems"; failures=1; fi   # gate-lint:detail
  # 并发两趟（文件头「槽位」的占哪一槽）：登记两槽；先起的那一趟（假 cargo 测试睡 3 秒，放进它自己的进程组）占着一槽、跑到测试那一步时再起一趟，
  #   后起的那一趟新开另一槽，两趟都退 0
  slot_tree concurrent; checked=$((checked+1)); slot_problems=""
  PATH="$rebuilt_bin:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-slot-concurrent-held" \
    MERGE_AND_VERIFY_TREE="$scratch/slot-tree-concurrent" SINGLEFS_WORK_ROOT="$scratch/slot-work-concurrent" MERGE_AND_VERIFY_SLOT_TABLE="$slot_table_two" \
    MERGE_AND_VERIFY_SLOT_BUILD_ROOT="$scratch/slot-build-concurrent" FAKE_CARGO_TIMELINE="$scratch/slot-concurrent-timeline.txt" FAKE_CARGO_TAG=held FAKE_CARGO_SLEEP=3 \
    setsid env -u CARGO_TARGET_DIR -u SINGLEFS_WORK_ROOT_BREAK bash "$0" --skip-apply "$rebuilt_patch" > "$scratch/out-slot-concurrent-held.txt" 2>&1 &
  slot_held_pid=$!
  for slot_attempt in $(seq 1 300); do grep -q '^start held ' "$scratch/slot-concurrent-timeline.txt" 2>/dev/null && break; kill -0 "$slot_held_pid" 2>/dev/null || break; sleep 0.1; done
  grep -q '^start held ' "$scratch/slot-concurrent-timeline.txt" 2>/dev/null || slot_problems+="先起的那一趟 $slot_attempt 轮里一直没跑到测试那一步；"
  slot_case concurrent concurrent-other "$slot_table_two"
  if wait "$slot_held_pid"; then slot_held_rc=0; else slot_held_rc=$?; fi
  [[ $slot_held_rc -eq 0 && $rc -eq 0 ]] || slot_problems+="先起的那一趟退 $slot_held_rc、后起的那一趟退 $rc；"
  [[ -n "$(slot_of concurrent-held)" && -n "$(slot_of concurrent-other)" && "$(slot_of concurrent-held)" != "$(slot_of concurrent-other)" ]] \
    || slot_problems+="两趟占的槽「$(slot_of concurrent-held)」「$(slot_of concurrent-other)」不是各占一槽；"
  if [[ -n "$slot_problems" ]]; then
    echo "  ✗ 自检：槽位：并发两趟各占一槽：实际 $slot_problems"; failures=1; fi   # gate-lint:detail
  # 改了、删了的文件（文件头「槽位」的拷进槽）：头一趟之后源树里改验证包的 setting.rs（大小不变）、删掉 demo 的 notes.txt，再起一趟：还是那一槽，
  #   槽里没有 notes.txt，验证包有 Compiling 行，重编核对核了它
  slot_tree sync; checked=$((checked+1)); slot_problems=""
  slot_case sync sync-first "$slot_table_two"
  slot_taken="$(slot_of sync-first)"
  printf 'pub fn setting() -> u32 { 2 }\n' > "$scratch/slot-tree-sync/crates/singlefs-verification/src/setting.rs"
  rm -f -- "$scratch/slot-tree-sync/crates/demo/notes.txt"
  slot_case sync sync-again "$slot_table_two"
  [[ $rc -eq 0 && -n "$slot_taken" && "$(slot_of sync-again)" == "$slot_taken" ]] || slot_problems+="退出码 $rc、两趟占的槽「$slot_taken」「$(slot_of sync-again)」；"
  [[ -n "$slot_taken" && ! -e "$slot_taken/repo/crates/demo/notes.txt" && -f "$slot_taken/repo/crates/demo/src/lib.rs" ]] \
    || slot_problems+="槽里$([[ -n "$slot_taken" && -e "$slot_taken/repo/crates/demo/notes.txt" ]] && echo 还留着源树里删掉的 notes.txt || echo 的快照不全)；"
  grep -q 'Compiling singlefs-verification ' "$scratch/logs-slot-sync-again/test.log" 2>/dev/null || slot_problems+="改过的验证包没有 Compiling 行；"
  grep -qF '改过的包这一趟都重编了（核了 1 个包：singlefs-verification）' "$out" || slot_problems+="重编核对没报核了验证包一个包：$(grep -E '重编' "$out" | head -2 | tr '\n' ' ')；"
  if [[ -n "$slot_problems" ]]; then
    echo "  ✗ 自检：槽位：源树里改了、删了的文件在槽里照改照删，改过的包必重编：实际 $slot_problems"; failures=1; fi   # gate-lint:detail
  # 槽的编译目录里的产物比改过的源码新（文件头「改过的包都重编了没有」槽位那一种）：头一趟之后把这一槽编译目录里 demo 的 dep-info 拨到明天、源树里改 demo 的 lib.rs，
  #   再起一趟：假 cargo 认 demo 已编、不打 Compiling demo，重编核对照拷进槽时写了的 lib.rs 核 demo，判红退 1
  slot_tree stale; checked=$((checked+1))
  slot_case stale stale-first "$slot_table_two"
  slot_target="$(slot_target_of stale-first)"
  [[ -z "$slot_target" ]] || touch -d '+1 day' "$slot_target/debug/.fake/demo.d"
  printf '// edited\n' >> "$scratch/slot-tree-stale/crates/demo/src/lib.rs"
  slot_case stale stale-again "$slot_table_two"
  if [[ $rc -ne 1 ]] || ! grep -q '改过的包这一趟没重编：demo' "$out"; then
    echo "  ✗ 自检：槽位：槽的编译目录里的产物比改过的源码新、改过的 demo 没有 Compiling 行：应当退 1、点名 demo，实际退出码 $rc：$(grep -E '✗|✓|重编' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 占盘（文件头「槽位」的占盘）：登记的上限 0 GiB；这一台编译根下先放一份槽已不在的编译目录 slot.gone/target。跑完：那一份删了，
  #   这一槽的编译目录收尾超上限、删了，打清了那一行
  slot_tree trim; checked=$((checked+1)); slot_problems=""
  mkdir -p "$scratch/slot-build-trim/merge-and-verify-slots/slot.gone/target/debug" && printf 'left\n' > "$scratch/slot-build-trim/merge-and-verify-slots/slot.gone/target/debug/left"
  slot_case trim trim "$slot_table_trim"
  slot_target="$(slot_target_of trim)"
  [[ $rc -eq 0 && -n "$slot_target" && ! -e "$slot_target" ]] || slot_problems+="退出码 $rc，这一槽的编译目录「$slot_target」$([[ -n "$slot_target" && -e "$slot_target" ]] && echo 还在 || echo 不在)；"
  [[ ! -e "$scratch/slot-build-trim/merge-and-verify-slots/slot.gone" ]] || slot_problems+="槽已不在的那一份编译目录还在；"
  grep -q '^  清了这一槽的编译目录 ' "$out" || slot_problems+="没有清了那一行；"
  if [[ -n "$slot_problems" ]]; then
    echo "  ✗ 自检：槽位：槽的编译目录超上限就清，槽没了的编译目录跟着删：实际 $slot_problems"; failures=1; fi   # gate-lint:detail
  # 撞上限重来（文件头「上一次尝试留下的快照」槽位那一种）：槽那一层先有两槽，slot.retry-old 最近一次占用在 2020 年、slot.retry-new 在此刻；
  #   任务 7301 尝试 1 的日志目录记着它占的是 slot.retry-old。尝试 2 要接着用 slot.retry-old（不挑最近占过的 slot.retry-new）
  slot_tree retry; checked=$((checked+1))
  mkdir -p "$scratch/slot-work-retry/merge-and-verify/slot.retry-old" "$scratch/slot-work-retry/merge-and-verify/slot.retry-new" "$scratch/logs-slot-retry-one"
  : > "$scratch/slot-work-retry/merge-and-verify/slot.retry-old/used"; touch -d @1577836800 "$scratch/slot-work-retry/merge-and-verify/slot.retry-old/used"
  : > "$scratch/slot-work-retry/merge-and-verify/slot.retry-new/used"
  printf '7301 1\n' > "$scratch/logs-slot-retry-one/task-attempt"
  printf '%s\n' "$scratch/slot-work-retry/merge-and-verify/slot.retry-old" > "$scratch/logs-slot-retry-one/slot"
  slot_case retry retry-two "$slot_table_two" TASK_SERVER_TASK=7301 TASK_SERVER_ATTEMPT=2
  if [[ $rc -ne 0 || "$(basename -- "$(slot_of retry-two)")" != slot.retry-old ]] || ! grep -qF '同一任务上一次尝试占过的那一槽' "$out"; then
    echo "  ✗ 自检：上一次尝试：撞上限重来的尝试 2 接着用尝试 1 记下的那一槽：应当占 slot.retry-old，实际退出码 $rc，占的是「$(slot_of retry-two)」"; failures=1; fi   # gate-lint:detail
  # 多出来的槽不占（文件头「槽位」的占哪一槽）：槽那一层先有三槽，used 从新到旧是 slot.surplus-top、slot.surplus-middle、slot.surplus-bottom，登记一槽；
  #   同时起三个挑槽的进程（SLOT_HOLDER_PROGRAM 本身，起它们的「shell」是这里起的一个 sleep，各自放进自己的进程组）：只有一个占到槽、占的是 slot.surplus-top，
  #   另两个答没占到。停那个 sleep 之后占着的那一个自己退（每半秒核一次起它的 shell 还在不在）
  local slot_surplus_root="$scratch/slot-surplus/merge-and-verify" slot_surplus_physical slot_parent_pid slot_picker slot_answered slot_held_count
  local -a slot_picker_pids=()
  checked=$((checked+1)); slot_problems=""
  mkdir -p "$slot_surplus_root/slot.surplus-top" "$slot_surplus_root/slot.surplus-middle" "$slot_surplus_root/slot.surplus-bottom"
  slot_surplus_physical="$(cd "$slot_surplus_root" && pwd -P)"
  : > "$slot_surplus_root/slot.surplus-top/used"
  : > "$slot_surplus_root/slot.surplus-middle/used"; touch -d '-1 day' "$slot_surplus_root/slot.surplus-middle/used"
  : > "$slot_surplus_root/slot.surplus-bottom/used"; touch -d '-2 day' "$slot_surplus_root/slot.surplus-bottom/used"
  setsid sleep 120 > /dev/null 2>&1 &
  slot_parent_pid=$!
  for slot_picker in one two three; do
    setsid python3 -c "$SLOT_HOLDER_PROGRAM" "$slot_surplus_root" 1 "" "$slot_parent_pid" "${MERGE_AND_VERIFY_BREAK:-}" take > "$scratch/slot-surplus-picker-$slot_picker.txt" 2>&1 &
    slot_picker_pids+=("$!")
  done
  for slot_attempt in $(seq 1 300); do
    slot_answered="$(cat "$scratch"/slot-surplus-picker-*.txt 2>/dev/null | grep -c '^SLOT \(held\|none\) ')"
    [[ "$slot_answered" -ge 3 ]] && break
    sleep 0.1
  done
  slot_held_count="$(cat "$scratch"/slot-surplus-picker-*.txt 2>/dev/null | grep -c '^SLOT held ')"
  [[ "$slot_answered" -ge 3 ]] || slot_problems+="$slot_attempt 轮里三个挑槽进程只答了 $slot_answered 个；"
  [[ "$slot_held_count" -eq 1 ]] || slot_problems+="占到槽的有 $slot_held_count 个；"
  grep -q "^SLOT held $slot_surplus_physical/slot.surplus-top"$'\t' "$scratch"/slot-surplus-picker-*.txt 2>/dev/null || slot_problems+="slot.surplus-top 没被占；"
  kill "$slot_parent_pid" 2>/dev/null; wait "$slot_parent_pid" 2>/dev/null
  wait "${slot_picker_pids[@]}" 2>/dev/null
  if [[ -n "$slot_problems" ]]; then
    echo "  ✗ 自检：槽位：调低上限之后多出来的槽不占：已有三槽、登记一槽、同时三个挑槽进程应当只有一个占到槽、占的是 slot.surplus-top，实际 $slot_problems答话：$(grep -h '^SLOT \(held\|none\)' "$scratch"/slot-surplus-picker-*.txt 2>/dev/null | cut -c1-120 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 多出来的槽（文件头「槽位」的多出来的槽）：登记 slots=0；槽那一层先有一个空着的槽 slot.idle 与一个留着的槽 slot.kept-back（kept、repo/ 都在），
  #   这一台编译根下两槽各有一份编译目录。跑一趟：slot.idle 与它的编译目录都删了；留着的那一槽与它的编译目录原样，打一行跳过它
  local slot_zero_root="$scratch/slot-work-surplus-zero/merge-and-verify" slot_zero_build="$scratch/slot-build-surplus-zero/merge-and-verify-slots" slot_zero_physical
  slot_tree surplus-zero; checked=$((checked+1)); slot_problems=""
  mkdir -p "$slot_zero_root/slot.idle/repo" "$slot_zero_root/slot.kept-back/repo" "$slot_zero_build/slot.idle/target/debug" "$slot_zero_build/slot.kept-back/target/debug"
  slot_zero_physical="$(cd "$slot_zero_root" && pwd -P)"
  : > "$slot_zero_root/slot.idle/used"; : > "$slot_zero_root/slot.kept-back/used"
  printf 'log: %s\ncreated: %s\n' "$scratch/logs-slot-earlier" "$(TZ=Asia/Tokyo date +%F)" > "$slot_zero_root/slot.kept-back/kept"
  printf 'left\n' > "$slot_zero_build/slot.idle/target/debug/left"; printf 'left\n' > "$slot_zero_build/slot.kept-back/target/debug/left"
  printf 'slots=0\nslot-target-limit-gib=1000\n' > "$scratch/slot-table-zero.conf"
  slot_case surplus-zero surplus-zero "$scratch/slot-table-zero.conf"
  [[ ! -e "$slot_zero_root/slot.idle" ]] || slot_problems+="空着的 slot.idle 还在；"
  [[ ! -e "$slot_zero_build/slot.idle" ]] || slot_problems+="slot.idle 在这一台的编译目录还在；"
  [[ -f "$slot_zero_root/slot.kept-back/kept" && -f "$slot_zero_build/slot.kept-back/target/debug/left" ]] || slot_problems+="留着的 slot.kept-back 或它在这一台的编译目录被动了；"
  grep -qF "! 多出来的槽 $slot_zero_physical/slot.kept-back 留着" "$out" || slot_problems+="没有跳过留着的那一槽那一行；"
  if [[ -n "$slot_problems" ]]; then
    echo "  ✗ 自检：槽位：slots 是 0 时清多出来的槽与它的编译目录，留着的槽不动：实际 $slot_problems$(grep -E '多出来的槽|槽已不在|不占槽' "$out" | head -4 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 拷进槽（文件头「槽位」的拷进槽）：槽那一层先有一槽 slot.leftover，快照下留着上一趟的编译产物 repo/target/release/tool 与 repo/scripts/__pycache__/x.pyc，
  #   另有 repo/.git/keep；要验的树里有 scripts/tool.py，登记两槽。跑一趟：占的是 slot.leftover，两样编译产物都没了，.git 下那一份照留
  local slot_left_repository="$scratch/slot-work-leftover/merge-and-verify/slot.leftover/repo"
  slot_tree leftover; checked=$((checked+1)); slot_problems=""
  mkdir -p "$scratch/slot-tree-leftover/scripts" "$slot_left_repository/target/release" "$slot_left_repository/scripts/__pycache__" "$slot_left_repository/.git"
  printf 'print(1)\n' > "$scratch/slot-tree-leftover/scripts/tool.py"
  printf 'tool\n' > "$slot_left_repository/target/release/tool"; printf 'pyc\n' > "$slot_left_repository/scripts/__pycache__/x.pyc"; printf 'keep\n' > "$slot_left_repository/.git/keep"
  : > "$scratch/slot-work-leftover/merge-and-verify/slot.leftover/used"
  slot_case leftover leftover "$slot_table_two"
  [[ "$(basename -- "$(slot_of leftover)")" == slot.leftover ]] || slot_problems+="占的槽是「$(slot_of leftover)」，不是 slot.leftover；"
  [[ ! -e "$slot_left_repository/target" ]] || slot_problems+="repo/target 还在；"
  [[ ! -e "$slot_left_repository/scripts/__pycache__" ]] || slot_problems+="repo/scripts/__pycache__ 还在；"
  [[ -f "$slot_left_repository/.git/keep" && -f "$slot_left_repository/scripts/tool.py" ]] || slot_problems+=".git/keep 或拷进来的 scripts/tool.py 不在；"
  if [[ -n "$slot_problems" ]]; then
    echo "  ✗ 自检：槽位：拷进槽之前删掉槽里上一趟留下的编译产物：应当占 slot.leftover、删掉 repo/target 与 repo/scripts/__pycache__、留着 .git，实际退出码 $rc，$slot_problems"; failures=1; fi   # gate-lint:detail
  # 槽位汇总（文件头「槽位」的槽位汇总）：登记三槽，槽那一层三槽都留着（kept、repo/、.owner 都在）。跑一趟：不占槽、照一趟一份跑，
  #   收尾的槽位汇总写这一趟不占槽，下面逐个列三槽（槽名、会话、放槽那一条）；三槽的 kept 原样（只报，不收）
  local slot_kept_root="$scratch/slot-work-all-kept/merge-and-verify" slot_kept_name
  slot_tree all-kept; checked=$((checked+1)); slot_problems=""
  printf 'slots=3\nslot-target-limit-gib=1000\n' > "$scratch/slot-table-three.conf"
  for slot_kept_name in slot.kept-one slot.kept-two slot.kept-three; do
    mkdir -p "$slot_kept_root/$slot_kept_name/repo"
    : > "$slot_kept_root/$slot_kept_name/used"
    printf 'log: %s\ncreated: %s\n' "$scratch/logs-slot-earlier" "$(TZ=Asia/Tokyo date +%F)" > "$slot_kept_root/$slot_kept_name/kept"
    printf 'session: slot-kept-session\ncreated: %s\npurpose: 自证造的留着的槽\n' "$(TZ=Asia/Tokyo date +%F)" > "$slot_kept_root/$slot_kept_name/.owner"
  done
  slot_case all-kept all-kept "$scratch/slot-table-three.conf"
  [[ -z "$(slot_of all-kept)" ]] || slot_problems+="占了槽「$(slot_of all-kept)」；"
  grep -q '^  槽位汇总：这一趟不占槽（登记的 3 槽都有了' "$out" || slot_problems+="槽位汇总没写这一趟不占槽；"
  for slot_kept_name in slot.kept-one slot.kept-two slot.kept-three; do
    grep -q "^    $slot_kept_name：留了 0 天.*会话 slot-kept-session；放槽：rm -rf -- '.*/$slot_kept_name/kept'" "$out" || slot_problems+="槽位汇总没列 $slot_kept_name；"
    [[ -f "$slot_kept_root/$slot_kept_name/kept" ]] || slot_problems+="$slot_kept_name 的 kept 没了；"
  done
  if [[ -n "$slot_problems" ]]; then
    echo "  ✗ 自检：槽位：三槽都留着时槽位汇总里有不占槽那一行与三个留着的槽：实际 $slot_problems$(grep -E '槽位汇总|不占槽' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 第二台那一层给的编译目录（文件头「槽位」的没占到槽的）：环境里有 MULTI_HOST_RUN_PEER_SCRATCH、CARGO_TARGET_DIR 是它下面的 target（multi-host-run.sh 派到第二台的那一种），
  #   登记两槽：不占槽，那一行说是第二台那一层给的编译目录
  slot_tree peer; checked=$((checked+1))
  mkdir -p "$scratch/slot-peer-scratch/target"
  slot_case peer peer "$slot_table_two" MULTI_HOST_RUN_PEER_SCRATCH="$scratch/slot-peer-scratch" CARGO_TARGET_DIR="$scratch/slot-peer-scratch/target"
  if [[ -n "$(slot_of peer)" ]] || ! grep -qF '这一趟不占槽：第二台那一层（scripts/hosts/multi-host-run.sh）给了编译目录' "$out"; then
    echo "  ✗ 自检：槽位：第二台那一层给了编译目录时不占槽，那一行说是第二台那一层给的：实际占的槽「$(slot_of peer)」：$(grep -E '不占槽|占了槽' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  # 增量编译（文件头「槽位」的增量编译）：rebuilt-bin 那一份假 cargo 前面套一层，记下每条 cargo 收到的 CARGO_INCREMENTAL（「<子命令><TAB><值，没设写 unset>」）
  local slot_incremental_bin="$scratch/slot-incremental-bin" slot_incremental_target slot_incremental_seen
  mkdir -p "$slot_incremental_bin"
  cat > "$slot_incremental_bin/cargo" <<'FAKE'
#!/usr/bin/env bash
printf '%s\t%s\n' "$1" "${CARGO_INCREMENTAL-unset}" >> "${FAKE_CARGO_INCREMENTAL_RECORD:?假 cargo 没给 FAKE_CARGO_INCREMENTAL_RECORD}"
exec "${FAKE_CARGO_INCREMENTAL_NEXT:?假 cargo 没给 FAKE_CARGO_INCREMENTAL_NEXT}/cargo" "$@"
FAKE
  chmod +x "$slot_incremental_bin/cargo"
  cargo_incremental_seen() { [[ -s "$1" ]] && cut -f2 -- "$1" | sort -u | tr '\n' ' ' | sed 's/ $//'; }   # <记录>：收到的值去重、空格连起来；一条记录都没有打空
  # 占槽的那一趟：槽那一层先有一槽 slot.incremental，它在这一台的编译目录里先放 debug/incremental/x 与 debug/keep，登记两槽。跑完：占的是 slot.incremental，
  #   每条 cargo 都收到 0，debug/incremental 删了、打了删它那一行，debug/keep 照留。另起一趟不占槽的（登记 slots=0，一趟一份那一种）：每条 cargo 都没收到 CARGO_INCREMENTAL
  slot_tree incremental-off; slot_tree incremental-none; checked=$((checked+1)); slot_problems=""
  slot_incremental_target="$scratch/slot-build-incremental-off/merge-and-verify-slots/slot.incremental/target"
  mkdir -p "$scratch/slot-work-incremental-off/merge-and-verify/slot.incremental" "$slot_incremental_target/debug/incremental"
  : > "$scratch/slot-work-incremental-off/merge-and-verify/slot.incremental/used"
  printf 'x\n' > "$slot_incremental_target/debug/incremental/x"; printf 'keep\n' > "$slot_incremental_target/debug/keep"
  slot_case incremental-off incremental-off "$slot_table_two" PATH="$slot_incremental_bin:$rebuilt_bin:$PATH" \
    FAKE_CARGO_INCREMENTAL_NEXT="$rebuilt_bin" FAKE_CARGO_INCREMENTAL_RECORD="$scratch/slot-incremental-off-cargo.txt"
  [[ $rc -eq 0 && "$(basename -- "$(slot_of incremental-off)")" == slot.incremental ]] || slot_problems+="占槽那一趟退出码 $rc、占的槽「$(slot_of incremental-off)」不是 slot.incremental；"
  slot_incremental_seen="$(cargo_incremental_seen "$scratch/slot-incremental-off-cargo.txt")"
  [[ "$slot_incremental_seen" == 0 ]] || slot_problems+="占槽那一趟 cargo 收到的 CARGO_INCREMENTAL 是「$slot_incremental_seen」，不是每条都是 0；"
  [[ ! -e "$slot_incremental_target/debug/incremental" ]] || slot_problems+="这一槽编译目录里的 debug/incremental 还在；"
  [[ -f "$slot_incremental_target/debug/keep" ]] || slot_problems+="这一槽编译目录里别的产物 debug/keep 也删了；"
  grep -q '^  删了这一槽编译目录里的增量编译缓存 .*/slot\.incremental/target/debug/incremental（' "$out" || slot_problems+="没有删了增量编译缓存那一行；"
  slot_case incremental-none incremental-none "$scratch/slot-table-none.conf" PATH="$slot_incremental_bin:$rebuilt_bin:$PATH" \
    FAKE_CARGO_INCREMENTAL_NEXT="$rebuilt_bin" FAKE_CARGO_INCREMENTAL_RECORD="$scratch/slot-incremental-none-cargo.txt"
  [[ $rc -eq 0 && -z "$(slot_of incremental-none)" ]] || slot_problems+="不占槽那一趟退出码 $rc、占的槽「$(slot_of incremental-none)」；"
  slot_incremental_seen="$(cargo_incremental_seen "$scratch/slot-incremental-none-cargo.txt")"
  [[ "$slot_incremental_seen" == unset ]] || slot_problems+="不占槽那一趟 cargo 收到的 CARGO_INCREMENTAL 是「$slot_incremental_seen」，应当每条都没设；"
  if [[ -n "$slot_problems" ]]; then
    echo "  ✗ 自检：槽位：占到槽的那一趟每条 cargo 收到 CARGO_INCREMENTAL=0、编之前删掉这一槽编译目录里的 incremental，不占槽的那一趟不设：实际 $slot_problems"; failures=1; fi   # gate-lint:detail
  # 调用方设了 CARGO_INCREMENTAL=1：槽那一层与这一台的编译目录照上一格先放好。跑完：占的是 slot.incremental，每条 cargo 都收到 1（不盖），
  #   debug/incremental/x 照留，打了调用方设了的那一行
  slot_tree incremental-caller; checked=$((checked+1)); slot_problems=""
  slot_incremental_target="$scratch/slot-build-incremental-caller/merge-and-verify-slots/slot.incremental/target"
  mkdir -p "$scratch/slot-work-incremental-caller/merge-and-verify/slot.incremental" "$slot_incremental_target/debug/incremental"
  : > "$scratch/slot-work-incremental-caller/merge-and-verify/slot.incremental/used"
  printf 'x\n' > "$slot_incremental_target/debug/incremental/x"
  slot_case incremental-caller incremental-caller "$slot_table_two" CARGO_INCREMENTAL=1 PATH="$slot_incremental_bin:$rebuilt_bin:$PATH" \
    FAKE_CARGO_INCREMENTAL_NEXT="$rebuilt_bin" FAKE_CARGO_INCREMENTAL_RECORD="$scratch/slot-incremental-caller-cargo.txt"
  [[ $rc -eq 0 && "$(basename -- "$(slot_of incremental-caller)")" == slot.incremental ]] || slot_problems+="退出码 $rc、占的槽「$(slot_of incremental-caller)」不是 slot.incremental；"
  slot_incremental_seen="$(cargo_incremental_seen "$scratch/slot-incremental-caller-cargo.txt")"
  [[ "$slot_incremental_seen" == 1 ]] || slot_problems+="cargo 收到的 CARGO_INCREMENTAL 是「$slot_incremental_seen」，不是调用方设的 1；"
  [[ -f "$slot_incremental_target/debug/incremental/x" ]] || slot_problems+="这一槽编译目录里的 debug/incremental/x 删了；"
  grep -qF '增量编译：调用方设了 CARGO_INCREMENTAL=1，照它编' "$out" || slot_problems+="没有调用方设了 CARGO_INCREMENTAL 那一行；"
  if [[ -n "$slot_problems" ]]; then
    echo "  ✗ 自检：槽位：调用方设了 CARGO_INCREMENTAL 的照它、不盖，这一槽编译目录里的 incremental 不删：实际 $slot_problems"; failures=1; fi   # gate-lint:detail
  # 不给自证的登记与编译根（文件头「槽位」的槽位登记）：这几格直接调 choose_slot，环境里去掉 MERGE_AND_VERIFY_SLOT_TABLE 与 MERGE_AND_VERIFY_SLOT_BUILD_ROOT，
  #   这一台缺省的工作根与编译根换成这一格自己的假的：SINGLEFS_MULTI_HOST_CONFIG 指到假多机配置（127.0.0.1 是这一台、192.0.2.81 是第二台，同 scripts/hosts/multi_host_addresses.py
  #   的自证；这一台的 BUILD_ROOT_BY_HOST 指到假编译根，SHARED_DATA_ROOT 指到假数据根，缺省的工作根是它下面的 work），SINGLEFS_DATA_ROOT 缺省也指到假数据根。调之前在同一份环境里
  #   照 slot_build_root、slot_default_work_root 与 lib_work_root.py 各取一遍：编译根不是这一格的假编译根、缺省的工作根或这一趟的工作根不在这一格的目录下就不调
  #   （不碰真的工作根与编译根），这一格判红。
  #   缺省的工作根下的槽那一层先放两槽：空着的 slot.idle（最近占过）与留着的 slot.kept（kept、repo/ 都在，一天前占过），假编译根下两槽各有一份编译目录
  slot_default_roots() {   # <格名>：在 slot-default-<格名>/ 下建这一格的假数据根、假编译根、假多机配置、要验的树、日志目录与另一个空的工作根
    local base="$scratch/slot-default-$1" name
    mkdir -p "$base/data/work/merge-and-verify" "$base/tree" "$base/logs" "$base/other-work"
    for name in slot.idle slot.kept; do
      mkdir -p "$base/data/work/merge-and-verify/$name/repo" "$base/build/merge-and-verify-slots/$name/target/debug"
      : > "$base/data/work/merge-and-verify/$name/used"
      printf 'artifact\n' > "$base/build/merge-and-verify-slots/$name/target/debug/artifact"
    done
    touch -d '-1 day' "$base/data/work/merge-and-verify/slot.kept/used"
    printf 'log: %s\ncreated: %s\n' "$base/logs" "$(TZ=Asia/Tokyo date +%F)" > "$base/data/work/merge-and-verify/slot.kept/kept"
    printf '%s\n' "BUILD_ROOT_BY_HOST=127.0.0.1=$base/build 192.0.2.81=/selftest/build-root-of-peer" \
      "CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/cargo-of-this 192.0.2.81=/selftest/cargo-of-peer" "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=2G 192.0.2.81=2G" \
      "QUIESCE_STOP_COMMAND=" "QUIESCE_STOPPED_CHECK_COMMAND=" "QUIESCE_START_COMMAND=" "QUIESCE_STARTED_CHECK_COMMAND=" "ENABLE_ACROSS_MACHINES=1" "ENABLE_GPU=0" \
      "NFS_SHARED_ROOTS=$base" "SHARED_DATA_ROOT=$base/data" "HOSTS=127.0.0.1 192.0.2.81" > "$base/multi-host.env"
  }
  slot_default_choose() {   # <格名> <交给 SINGLEFS_WORK_ROOT 的工作根，空串是不设> <登记几槽> [交给 SINGLEFS_DATA_ROOT 的数据根，缺省是这一格的假数据根]：
    #   在这一格的假环境里调一次 choose_slot、放槽（子 shell 里，改的全局变量不带出来）；占的槽写进 took、没占的为什么写进 fallback，没调的为什么写进 refused，choose_slot 打的行进 choose.out
    local base="$scratch/slot-default-$1" base_physical
    base_physical="$(realpath -m -- "$base")"
    (
      export SINGLEFS_DATA_ROOT="${4:-$base/data}" SINGLEFS_MULTI_HOST_CONFIG="$base/multi-host.env"
      if [[ -n "$2" ]]; then export SINGLEFS_WORK_ROOT="$2"; else unset SINGLEFS_WORK_ROOT; fi
      unset MERGE_AND_VERIFY_SLOT_TABLE MERGE_AND_VERIFY_SLOT_BUILD_ROOT CARGO_TARGET_DIR CARGO_INCREMENTAL SINGLEFS_WORK_ROOT_BREAK MULTI_HOST_RUN_PEER_SCRATCH
      seen_build="$(slot_build_root)"; seen_default="$(slot_default_work_root)"; seen_work="$(python3 "$root/.claude/hooks/lib_work_root.py" 2>&1)"
      if [[ -z "$seen_build" || -z "$seen_default" || -z "$seen_work" || "$(realpath -m -- "$seen_build")" != "$(realpath -m -- "$base/build")" \
            || "$(realpath -m -- "$seen_default")/" != "$base_physical/"* || "$(realpath -m -- "$seen_work")/" != "$base_physical/"* ]]; then
        printf '这一台的编译根取到「%s」、缺省的工作根取到「%s」、这一趟的工作根取到「%s」，不是这一格的假的，没调 choose_slot\n' "$seen_build" "$seen_default" "$seen_work" > "$base/refused"; exit 0
      fi
      SLOT_LIMIT="$3"; SLOT_TARGET_LIMIT_GIB=1000; SOURCE_TREE="$base/tree"
      SLOT_DIRECTORY=""; SLOT_TARGET=""; SLOT_FALLBACK_REASON=""; SLOT_ROOT_DIRECTORY=""; SLOT_HOLDER_PID=""; SLOT_HOLDER_FD=""
      choose_slot "$base/logs"
      printf '%s\n' "$SLOT_DIRECTORY" > "$base/took"; printf '%s\n' "$SLOT_FALLBACK_REASON" > "$base/fallback"
      release_slot
    ) > "$base/choose.out" 2>&1
  }
  # 只给另一个工作根：SINGLEFS_WORK_ROOT 指到这一格另一个空的工作根 other-work。不占也不清：没占槽、为什么那一句是工作根不是这一台缺省的那一个，
  #   缺省工作根下的两槽与它们在假编译根下的编译目录都在，other-work 下没开槽。登记三槽与登记 0 槽（只清、不占的那一支）各一格
  slot_other_work_root_case() {   # <格名> <登记几槽> <失败行里的格名>
    local base="$scratch/slot-default-$1" name
    checked=$((checked+1)); slot_problems=""
    slot_default_roots "$1"
    slot_default_choose "$1" "$base/other-work" "$2"
    if [[ -f "$base/refused" ]]; then slot_problems+="$(cat "$base/refused")；"
    else
      [[ -z "$(cat "$base/took" 2>/dev/null)" ]] || slot_problems+="占了槽「$(cat "$base/took")」；"
      grep -q '不是这一台缺省的那一个.*不占也不清' "$base/fallback" 2>/dev/null \
        || slot_problems+="没占槽的为什么是「$(cat "$base/fallback" 2>/dev/null)」，不是工作根不是这一台缺省的那一个；"
      for name in slot.idle slot.kept; do
        [[ -f "$base/build/merge-and-verify-slots/$name/target/debug/artifact" ]] || slot_problems+="缺省工作根下 $name 在这一台的编译目录删了；"
        [[ -d "$base/data/work/merge-and-verify/$name/repo" ]] || slot_problems+="缺省工作根下的 $name 删了；"
      done
      ! compgen -G "$base/other-work/merge-and-verify/slot.*" > /dev/null || slot_problems+="另一个工作根下开了槽；"
    fi
    if [[ -n "$slot_problems" ]]; then
      echo "  ✗ 自检：$3：实际 $slot_problems$(grep -E '删了|占了槽|多出来的槽' "$base/choose.out" 2>/dev/null | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  }
  slot_other_work_root_case other-root-with-slots 3 "槽位：登记三槽、只给另一个工作根而不给自证的登记与编译根时不占也不清，原工作根下各槽的编译目录都在"
  slot_other_work_root_case other-root-zero-slots 0 "槽位：登记 0 槽、只给另一个工作根而不给自证的登记与编译根时不清，原工作根下各槽的编译目录都在"
  # 工作根就是这一台缺省的那一个：SINGLEFS_WORK_ROOT 给一个指到缺省工作根的符号链接（按真路径比，算同一个），登记三槽：照常占槽，
  #   占的是缺省工作根下那一层里的槽，为什么那一句是空的，留着的 slot.kept 与它在这一台的编译目录原样
  local slot_default_base="$scratch/slot-default-same-root" slot_default_physical
  checked=$((checked+1)); slot_problems=""
  slot_default_roots same-root
  ln -s -- "$slot_default_base/data/work" "$slot_default_base/work-alias"
  slot_default_physical="$(cd "$slot_default_base/data/work/merge-and-verify" && pwd -P)"
  slot_default_choose same-root "$slot_default_base/work-alias" 3
  if [[ -f "$slot_default_base/refused" ]]; then slot_problems+="$(cat "$slot_default_base/refused")；"
  else
    slot_taken="$(cat "$slot_default_base/took" 2>/dev/null)"
    [[ -n "$slot_taken" && "$(dirname -- "$slot_taken")" == "$slot_default_physical" ]] || slot_problems+="占的槽「$slot_taken」不在缺省工作根的槽那一层 $slot_default_physical 下；"
    [[ -z "$(cat "$slot_default_base/fallback" 2>/dev/null)" ]] || slot_problems+="没占槽的为什么是「$(cat "$slot_default_base/fallback")」；"
    [[ -f "$slot_default_base/data/work/merge-and-verify/slot.kept/kept" && -f "$slot_default_base/build/merge-and-verify-slots/slot.kept/target/debug/artifact" ]] \
      || slot_problems+="留着的 slot.kept 或它在这一台的编译目录被动了；"
  fi
  if [[ -n "$slot_problems" ]]; then
    echo "  ✗ 自检：槽位：不给自证的登记与编译根、工作根就是这一台缺省的那一个时照常占槽：实际 $slot_problems$(grep -E '不占槽|占了槽|删了' "$slot_default_base/choose.out" 2>/dev/null | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 换了 SINGLEFS_DATA_ROOT（文件头「槽位」：缺省的工作根只照多机配置的 SHARED_DATA_ROOT 取、与编译根同源，不看 SINGLEFS_DATA_ROOT），登记三槽，每种情形各一份假环境：
  #   连同数据根一起换：SINGLEFS_DATA_ROOT 指到这一格另一个数据根 other-data，SINGLEFS_WORK_ROOT 给它下面的 work 与不设（工作根就是它下面的 work）两种，
  #     都不占也不清：没占槽、为什么那一句是工作根不是这一台缺省的那一个，缺省工作根下的两槽与它们在假编译根下的编译目录都在，other-data 下没开槽；
  #   只换数据根：SINGLEFS_DATA_ROOT 指到 other-data、SINGLEFS_WORK_ROOT 给配置里 SHARED_DATA_ROOT 下的 work（统一任务服务给的那一种），照常占槽：
  #     占的是缺省工作根下那一层里的槽，为什么那一句是空的，留着的 slot.kept 与它在这一台的编译目录原样
  local slot_data_root_case slot_data_root_base slot_data_root_work slot_data_root_physical slot_data_root_slot slot_data_root_case_problems
  checked=$((checked+1)); slot_problems=""
  for slot_data_root_case in data-root-moved-with-work data-root-moved-work-unset; do
    slot_data_root_base="$scratch/slot-default-$slot_data_root_case"; slot_data_root_case_problems=""
    slot_default_roots "$slot_data_root_case"
    slot_data_root_work="$slot_data_root_base/other-data/work"
    [[ "$slot_data_root_case" == data-root-moved-with-work ]] || slot_data_root_work=""
    slot_default_choose "$slot_data_root_case" "$slot_data_root_work" 3 "$slot_data_root_base/other-data"
    if [[ -f "$slot_data_root_base/refused" ]]; then slot_problems+="$slot_data_root_case：$(cat "$slot_data_root_base/refused")；"; continue; fi
    [[ -z "$(cat "$slot_data_root_base/took" 2>/dev/null)" ]] || slot_data_root_case_problems+="占了槽「$(cat "$slot_data_root_base/took")」；"
    grep -q '不是这一台缺省的那一个.*不占也不清' "$slot_data_root_base/fallback" 2>/dev/null \
      || slot_data_root_case_problems+="没占槽的为什么是「$(cat "$slot_data_root_base/fallback" 2>/dev/null)」，不是工作根不是这一台缺省的那一个；"
    for slot_data_root_slot in slot.idle slot.kept; do
      [[ -f "$slot_data_root_base/build/merge-and-verify-slots/$slot_data_root_slot/target/debug/artifact" ]] \
        || slot_data_root_case_problems+="缺省工作根下 $slot_data_root_slot 在这一台的编译目录删了；"
      [[ -d "$slot_data_root_base/data/work/merge-and-verify/$slot_data_root_slot/repo" ]] || slot_data_root_case_problems+="缺省工作根下的 $slot_data_root_slot 删了；"
    done
    ! compgen -G "$slot_data_root_base/other-data/work/merge-and-verify/slot.*" > /dev/null || slot_data_root_case_problems+="另一个数据根下开了槽；"
    [[ -z "$slot_data_root_case_problems" ]] \
      || slot_problems+="$slot_data_root_case：$slot_data_root_case_problems（$(grep -E '删了|占了槽' "$slot_data_root_base/choose.out" 2>/dev/null | head -2 | tr '\n' ' ')）；"
  done
  slot_data_root_base="$scratch/slot-default-data-root-only-moved"
  slot_default_roots data-root-only-moved
  slot_data_root_physical="$(cd "$slot_data_root_base/data/work/merge-and-verify" && pwd -P)"
  slot_default_choose data-root-only-moved "$slot_data_root_base/data/work" 3 "$slot_data_root_base/other-data"
  if [[ -f "$slot_data_root_base/refused" ]]; then slot_problems+="data-root-only-moved：$(cat "$slot_data_root_base/refused")；"
  else
    slot_taken="$(cat "$slot_data_root_base/took" 2>/dev/null)"
    [[ -n "$slot_taken" && "$(dirname -- "$slot_taken")" == "$slot_data_root_physical" ]] \
      || slot_problems+="data-root-only-moved：占的槽「$slot_taken」不在缺省工作根的槽那一层 $slot_data_root_physical 下；"
    [[ -z "$(cat "$slot_data_root_base/fallback" 2>/dev/null)" ]] || slot_problems+="data-root-only-moved：没占槽的为什么是「$(cat "$slot_data_root_base/fallback")」；"
    [[ -f "$slot_data_root_base/data/work/merge-and-verify/slot.kept/kept" && -f "$slot_data_root_base/build/merge-and-verify-slots/slot.kept/target/debug/artifact" ]] \
      || slot_problems+="data-root-only-moved：留着的 slot.kept 或它在这一台的编译目录被动了；"
  fi
  if [[ -n "$slot_problems" ]]; then
    echo "  ✗ 自检：槽位：连同 SINGLEFS_DATA_ROOT 一起换了工作根时不占也不清、原工作根下各槽的编译目录都在，只换 SINGLEFS_DATA_ROOT 时照常占槽：实际 $slot_problems"; failures=1; fi   # gate-lint:detail
  # 认不出的选项（文件头「上限、线程、机器」）：给了 --never-used-option 就退 2、点名它，一样都不做（假 cargo 一条都没收到）
  out="$scratch/out-unknown-option.txt"; : > "$scratch/calls-unknown-option.txt"; checked=$((checked+1))
  PATH="$fake_cargo_dir:$PATH" FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-unknown-option.txt" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" \
    MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-unknown-option" MERGE_AND_VERIFY_TREE="$scratch/tree" env -u CARGO_TARGET_DIR bash "$0" --skip-apply --never-used-option > "$out" 2>&1; rc=$?
  if [[ $rc -ne 2 ]] || [[ -s "$scratch/calls-unknown-option.txt" ]] || ! grep -qF '认不出参数 --never-used-option' "$out"; then
    echo "  ✗ 自检：给了认不出的选项 --never-used-option 应当退 2、点名它，一样都不做，实际退出码 $rc，假 cargo 收到 $(grep -c . "$scratch/calls-unknown-option.txt" 2>/dev/null) 条：$(grep -E '✗|→' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 验证包那一步（文件头「验证包那一步」）：假的那一道答 --list-items（登记的崩溃枚举用例 demo-case 在测试目标 setting 里，挂特性的是库与 gpu_pair.rs），
  #   别的起法打按条复用与崩溃状态块那几行；合入的树是挑测试目标那几格的 narrow-tree，那一道收到的参数记进 verification_calls
  local verification_calls="$scratch/verification-step-calls.txt" verification_environment="$scratch/verification-step-environment.txt"
  verification_starts() { grep -c -- "^$1" "$verification_calls" 2>/dev/null; }   # <参数开头>：那一道被这样起了几次
  # 补丁只改 demo 一个函数（库里一份数据文件）、变异行点名验证包的 setting：测试那一步只起 demo --lib 与验证包 --release --test setting，
  #   验证包那一步只起一次 --list-items 与一次按块那一路（--check crash-case-replay --crash-cases-only --item crash-case:demo-case），不起门禁的整格
  narrow_patch "$scratch/narrow-core" crates/demo/src/table.txt
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' '验证包的 setting（自证样本）' crates/demo/src/lib.rs 2 3 '-p singlefs-verification --release --test setting -- case_filter --exact' case_filter \
    > "$scratch/narrow-core/mutations-append.tsv"
  out="$scratch/out-verification-core.txt"; : > "$verification_calls"; checked=$((checked+1))
  FAKE_VERIFICATION_CALLS="$verification_calls" narrow_case verification-core --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 0 || "$(narrow_tests verification-core)" != "test --offline --no-fail-fast -p demo --lib|test --offline --no-fail-fast --release --no-default-features -p singlefs-verification --test setting|" ]] \
     || [[ "$(grep -c . "$verification_calls")" -ne 2 || "$(verification_starts '--list-items ')" -ne 1 ]] \
     || [[ "$(verification_starts '--check crash-case-replay --crash-cases-only --item crash-case:demo-case ')" -ne 1 ]] \
     || grep -qF 'crash-case-replay,' "$verification_calls" || ! grep -qF '✓ 验证包那一步（登记的崩溃枚举用例）过' "$out"; then
    echo "  ✗ 自检：验证包那一步：补丁只改 demo 一个函数、变异行点名验证包的 setting，应当只起 demo --lib、验证包 --release --test setting 与按块那一路的 crash-case:demo-case，不起整格，实际退出码 $rc，假 cargo 收到 $(narrow_tests verification-core)，那一道收到 $(tr '\n' '|' < "$verification_calls")：$(grep -E '验证包那一步|✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 进度行（文件头「进度行」）：间隔缩到 1 秒（MERGE_AND_VERIFY_PROGRESS_INTERVAL）、假 cargo 的测试与假的那一道（--list-items 之外）各跑 3 秒，
  #   测试那两条（demo --lib、验证包 --test setting）与验证包那一步的崩溃枚举用例那一次，标准输出里各要有不止一行「· 进度：<日志名> 已跑 N 秒，日志 M 行」，整条照旧退 0
  out="$scratch/out-progress.txt"; : > "$verification_calls"; checked=$((checked+1))
  MERGE_AND_VERIFY_PROGRESS_INTERVAL=1 FAKE_CARGO_TEST_SLEEP=3 FAKE_VERIFICATION_SLEEP=3 FAKE_VERIFICATION_CALLS="$verification_calls" narrow_case progress --skip-apply "$scratch/narrow-core"
  local progress_short=0 progress_label
  for progress_label in test-1 test-2 verification-crash-cases; do
    [[ "$(grep -cE "^  · 进度：$progress_label 已跑 [0-9]+ 秒，日志 [0-9]+ 行$" "$out")" -ge 2 ]] || progress_short=$((progress_short + 1))
  done
  if [[ $rc -ne 0 ]] || ((progress_short)); then
    echo "  ✗ 自检：进度行：子进程跑超过一个间隔时标准输出应当有进度行（间隔缩到 1 秒、测试与验证包那一步的门禁各跑 3 秒：test-1、test-2、verification-crash-cases 各至少两行），整条照旧退 0，实际退出码 $rc，进度行不够的有 $progress_short 项：$(grep -E '进度|✗' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 收尾只在起这一趟的那个 shell 里做（文件头「快照」末尾那一段）：run_step 放后台的进度进程是 fork 出来的子 shell，fork 窗口里收到 stop_progress_ticker 的 TERM 时
  #   照主 shell 的 trap 退出、跑继承来的 EXIT trap。撞不撞得上看调度，这里用 MERGE_AND_VERIFY_SELFTEST_FORKED_FINISH 让每一步的进度进程起头就调一次 finish 演这一种。
  #   快照与编译目录要留到这一趟自己收尾才删：「删了快照」至多收尾那一行，收尾删编译目录时它还在（没有「这一趟的编译目录 … 没建，不用删」那一行）。
  #   只看这两样：别的开关改的是收尾删不删、验证包那一步怎么走，这两行在它们底下照旧对。弄坏开关 finish-in-forked-shell 下测试那一步的进度进程就删了快照与编译目录，
  #   之后每一步的进度进程各打一行删了快照，收尾删编译目录时它已经不在，这一格红
  out="$scratch/out-forked-finish.txt"; checked=$((checked+1))
  MERGE_AND_VERIFY_SELFTEST_FORKED_FINISH=1 narrow_case forked-finish --skip-apply "$scratch/narrow-core"
  if [[ "$(grep -c '^  删了快照 ' "$out")" -gt 1 ]] || grep -q '^  这一趟的编译目录 .* 没建，不用删$' "$out"; then
    echo "  ✗ 自检：快照：fork 出来的子 shell 跑了继承来的 EXIT trap 时不该删这一趟的快照与编译目录（收尾只在起这一趟的那个 shell 里做：删了快照至多收尾那一行、收尾删编译目录时它还在），实际退出码 $rc，删了快照 $(grep -c '^  删了快照 ' "$out") 行：$(grep -E '删了|没建，不用删|崩溃域' "$out" | head -4 | tr '\n' ' ' | clip_text 600)"; failures=1; fi   # gate-lint:detail
  # 补丁改到验证包的库与挂特性的 gpu_pair（平铺的）、split_pair（目录式的 tests/split_pair/main.rs）：测试那一步验证包一条 --release --lib --test gpu_pair --test split_pair，
  #   验证包那一步只交 feature-gated:lib、feature-gated:gpu_pair.rs 与 feature-gated:split_pair/main.rs（不起按块那一路）
  narrow_patch "$scratch/narrow-features" crates/singlefs-verification/src/table.txt crates/singlefs-verification/tests/gpu_pair.rs crates/singlefs-verification/tests/split_pair/main.rs
  out="$scratch/out-verification-features.txt"; : > "$verification_calls"; checked=$((checked+1))
  FAKE_VERIFICATION_CALLS="$verification_calls" narrow_case verification-features --skip-apply "$scratch/narrow-features"
  if [[ $rc -ne 0 || "$(narrow_tests verification-features)" != "test --offline --no-fail-fast --release --no-default-features -p singlefs-verification --lib --test gpu_pair --test split_pair|" ]] \
     || [[ "$(grep -c . "$verification_calls")" -ne 2 || "$(verification_starts '--check feature-gated-tests --item feature-gated:lib --item feature-gated:gpu_pair.rs --item feature-gated:split_pair/main.rs ')" -ne 1 ]] \
     || ! grep -qF '✓ 验证包那一步（挂特性的测试文件）过' "$out"; then
    echo "  ✗ 自检：验证包那一步：补丁改到验证包的库与挂特性的 gpu_pair（平铺的）、split_pair（目录式的），应当只交 feature-gated:lib、feature-gated:gpu_pair.rs 与 feature-gated:split_pair/main.rs，实际退出码 $rc，假 cargo 收到 $(narrow_tests verification-features)，那一道收到 $(tr '\n' '|' < "$verification_calls")"; failures=1; fi   # gate-lint:detail
  # 按块那一路红（假的那一道打 case_function FAILED 与 crash-case:demo-case 的 ITEM_DONE exit=1）：退 1、列出红的测试函数，出路只重跑红的那一项（种类 items、范围 2 项：1 格加 1 项）
  out="$scratch/out-verification-red.txt"; checked=$((checked+1))
  FAKE_VERIFICATION_EXIT=1 narrow_case verification-red --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 1 ]] || ! grep -qF '✗ 验证包那一步（登记的崩溃枚举用例）红' "$out" || ! grep -qx '      case_function' "$out" \
     || ! grep -qF "SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crash-point-replay.sh 的 crash-case-replay 格点名的 crash-case:demo-case; 范围=2 项（1 格加点名的 1 项）;" "$out" \
     || ! grep -qF -- "--check crash-case-replay --crash-cases-only --item crash-case:demo-case" "$out" || ! grep -qF "task-server/task submit -- bash" "$out" \
     || grep -qF 'run-with-memory-cap.sh' "$out" || ! grep -qF '并进写段的下一批' "$out"; then
    echo "  ✗ 自检：验证包那一步：按块那一路判红时应当退 1、列出红的测试函数与只重跑红的那一项的命令，实际退出码 $rc：$(grep -E '验证包那一步|^      |→' "$out" | head -4 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 红的项认不出（假的那一道把 exit=1 的 ITEM_DONE 写成这一次没点的 crash-case:not-named）：只重跑的门禁点这一次的全部项（文件头「只重跑的命令点名」）；
  #   这一趟的输出由下面「重跑命令点名」那一格逐条判
  out="$scratch/out-verification-red-unknown-item.txt"
  FAKE_VERIFICATION_EXIT=1 FAKE_VERIFICATION_RED_ITEM=crash-case:not-named narrow_case verification-red-unknown-item --skip-apply "$scratch/narrow-core"
  # 补丁目录 narrow-product-gpu（改产品 GPU 包的真卡对拍 real_card）在这里先建：下面不带 --skip-apply 的先验后合那一格要拿它当补丁目录，不带 --skip-apply 的那一趟补丁目录不在就退 2；后面产品 GPU 包那几格也用它
  narrow_patch "$scratch/narrow-product-gpu" crates/singlefs-background-gpu-tests/tests/real_card.rs
  # 先验后合（文件头「先验后合」）：不带 --skip-apply 的那一趟验证包那一步红，只重跑红的是合入后验证（不带 --skip-apply）、不是起门禁的那一条：
  #   按块那一路红的点这一趟跑到的验证包目标 singlefs-verification:tests/setting.rs；产品 GPU 包的真卡对拍红的（那一格不收点名）不点项、submit 带 --whole-package-confirmed、
  #   范围写补丁份数，给过 --crash-expand-up-to 6 的照带
  local merge_red_rc merge_red_out="$scratch/out-verification-merge-red.txt" merge_red_gpu_out="$scratch/out-verification-merge-red-gpu.txt"
  out="$merge_red_out"; FAKE_VERIFICATION_EXIT=1 narrow_case verification-merge-red "$scratch/narrow-core"; merge_red_rc=$rc
  out="$merge_red_gpu_out"; FAKE_VERIFICATION_EXIT=1 narrow_case verification-merge-red-gpu --crash-expand-up-to 6 "$scratch/narrow-product-gpu"; checked=$((checked+1))
  if [[ $merge_red_rc -ne 1 || $rc -ne 1 ]] \
     || ! grep -qF "scripts/gate/merge-and-verify.sh --item singlefs-verification:tests/setting.rs $scratch/narrow-core" "$merge_red_out" \
     || ! grep -qF "跑什么=合入后验证点名的 singlefs-verification:tests/setting.rs; 范围=1 项;" "$merge_red_out" \
     || ! grep -qF "scripts/gate/merge-and-verify.sh --crash-expand-up-to 6 $scratch/narrow-product-gpu" "$merge_red_gpu_out" \
     || ! grep -qF "task-server/task submit --whole-package-confirmed $WHOLE_PACKAGE_CONFIRMATION_PLACEHOLDER -- env MERGE_AND_VERIFY_KEEP_COPY=1 bash" "$merge_red_gpu_out" \
     || ! grep -qF "跑什么=合入后验证 1 份补丁; 范围=1 份补丁;" "$merge_red_gpu_out" \
     || grep -qF "submit -- bash $scratch/fake-verification-step.sh" "$merge_red_out" "$merge_red_gpu_out" \
     || grep -qF "submit -- env SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6 " "$merge_red_gpu_out"; then
    echo "  ✗ 自检：先验后合：验证包那一步红了，只重跑红的应当是不带 --skip-apply 的合入后验证（按块那一路红点 --item singlefs-verification:tests/setting.rs，真卡对拍红不点项、submit 带 --whole-package-confirmed、带 --crash-expand-up-to 6），不是起门禁的那一条，实际退出码 $merge_red_rc $rc：$(grep -hE '→ 怎么办：FAILED' "$merge_red_out" "$merge_red_gpu_out" | clip_text 300 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 数红的测试函数（verification_red_test_names）：「test <名> ... FAILED」与「FAILED <名>」两种写法一起数、按名去重，汇总行「FAILED. N passed」不算
  checked=$((checked+1))
  printf '    test a_red ... FAILED
      FAILED a_red
      FAILED tests::b_red
test result: FAILED. 6 passed; 1 failed
    红的测试目标 x（汇总行 FAILED. 6 passed; 1 failed）：
' > "$scratch/red-names.log"
  out="$(verification_red_test_names "$scratch/red-names.log" | tr '
' ' ')"
  if [[ "$out" != "a_red tests::b_red " ]]; then
    echo "  ✗ 自检：验证包那一步：日志里两种写法的红应当一起数、按名去重（a_red、tests::b_red 两条），实际：$out"; failures=1; fi   # gate-lint:detail
  # 补丁只改 alpha、beta（跑到的目标里没有验证包的，也没有要特性的）：报本次未跑与原因，那一道一次都不起
  out="$scratch/out-verification-unaffected.txt"; : > "$verification_calls"; checked=$((checked+1))
  FAKE_VERIFICATION_CALLS="$verification_calls" narrow_case verification-unaffected --skip-apply "$scratch/narrow-two"
  if [[ $rc -ne 0 || -s "$verification_calls" ]] || ! grep -qF '  - 验证包那一步本次未跑：' "$out"; then
    echo "  ✗ 自检：验证包那一步：补丁只改到 alpha、beta 时应当报本次未跑、不起那一道，实际退出码 $rc，那一道被起 $(grep -c . "$verification_calls") 次：$(grep -E '验证包' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 补丁改到产品 GPU 包写了 required-features 的真卡对拍 real_card：测试那一步不跑它，验证包那一步起一次 --check product-gpu-real-card-tests（那一格不收点名）。
  #   这一趟假 cargo 的 build 不打 Compiling 行（FAKE_CARGO_BUILD_COMPILING_DROPPED）：下面「重编核对读 clippy 的 Checking 行」那一格单看 clippy 那一趟的行
  out="$scratch/out-verification-product-gpu.txt"; : > "$verification_calls"; checked=$((checked+1))
  FAKE_CARGO_BUILD_COMPILING_DROPPED=1 FAKE_VERIFICATION_CALLS="$verification_calls" narrow_case verification-product-gpu --skip-apply "$scratch/narrow-product-gpu"
  if [[ $rc -ne 0 || -n "$(narrow_tests verification-product-gpu)" || "$(grep -c . "$verification_calls")" -ne 1 || "$(verification_starts '--check product-gpu-real-card-tests ')" -ne 1 ]] \
     || ! grep -qF '产品 GPU 包的真卡对拍：singlefs-background-gpu-tests 的 real_card' "$out"; then
    echo "  ✗ 自检：验证包那一步：补丁改到产品 GPU 包的真卡对拍 real_card 时应当起 --check product-gpu-real-card-tests、测试那一步不跑它，实际退出码 $rc，假 cargo 收到 $(narrow_tests verification-product-gpu)，那一道收到 $(tr '\n' '|' < "$verification_calls")"; failures=1; fi   # gate-lint:detail
  # 重编核对也读 clippy 的「Checking」行（文件头「改过的包都重编了没有」）：补丁只改产品 GPU 包的真卡对拍 real_card，测试那一步不跑它、日志里没有这个包的 Compiling 行，
  #   这一趟 build 也不打 Compiling 行（上一格的 FAKE_CARGO_BUILD_COMPILING_DROPPED），它重编了没有靠 clippy 那一趟的 Checking 行证明：重编核对判过、报都重编了并点名这个包
  checked=$((checked+1))
  if [[ -f "$scratch/logs-narrow-verification-product-gpu/test.log" ]] && grep -qF 'Compiling singlefs-background-gpu-tests' "$scratch/logs-narrow-verification-product-gpu/test.log" \
     || grep -qF 'Compiling singlefs-background-gpu-tests' "$scratch/logs-narrow-verification-product-gpu/build.log" 2>/dev/null \
     || ! grep -qF '改过的包这一趟都重编了（核了 1 个包：singlefs-background-gpu-tests）' "$scratch/out-verification-product-gpu.txt"; then
    echo "  ✗ 自检：重编核对读 clippy 的 Checking 行：补丁只改产品 GPU 包的真卡对拍 real_card（测试那一步不跑它、build 那一趟不打它的 Compiling 行）时应当报改过的包都重编了、点名 singlefs-background-gpu-tests，实际：$(grep -E '重编|✗' "$scratch/out-verification-product-gpu.txt" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # build 那一步只点改到的包与传递依赖它们的包（文件头 build 一段）：narrow-tree 里产品 GPU 包依赖验证包。
  #   补丁只改验证包（verification-features 那一趟）：build 点验证包与依赖它的产品 GPU 包，成功句写出这两个包；
  #   补丁只改产品 GPU 包（verification-product-gpu 那一趟）：build 只点它，不点它依赖的验证包
  local build_features_calls build_product_calls
  build_features_calls="$(grep '^build' "$scratch/calls-narrow-verification-features.txt" 2>/dev/null | tr '\n' '|')"
  build_product_calls="$(grep '^build' "$scratch/calls-narrow-verification-product-gpu.txt" 2>/dev/null | tr '\n' '|')"
  checked=$((checked+1))
  if [[ "$build_features_calls" != "build --offline --all-targets -p singlefs-background-gpu-tests -p singlefs-verification|" ]] \
     || ! grep -qF '✓ build 过（编了 2 个包：singlefs-background-gpu-tests singlefs-verification）' "$scratch/out-verification-features.txt" \
     || [[ "$build_product_calls" != "build --offline --all-targets -p singlefs-background-gpu-tests|" ]]; then
    echo "  ✗ 自检：build 那一步：补丁只改验证包时应当只点验证包与依赖它的产品 GPU 包（-p singlefs-background-gpu-tests -p singlefs-verification）、成功句写出这两个包，只改产品 GPU 包时只点它（不点它依赖的验证包），实际只改验证包那一趟收到「$build_features_calls」、build 那一句「$(grep -F 'build' "$scratch/out-verification-features.txt" | head -1)」，只改产品 GPU 包那一趟收到「$build_product_calls」"; failures=1; fi   # gate-lint:detail
  # build 那一步本次未跑（文件头 build 一段）：补丁目录只有变异行（mutations-append.tsv 点名 alpha 的库）、没有 crates.patch：build 不起、报本次未跑、不报 build 过
  #   （只核 build 这一步：测试照挑 alpha --lib，挑法与退出码归挑测试目标那几格；只改不落在包里的文件那一种是上面判不出测试目标里改 docs/notes.txt 的那一格）
  mkdir -p "$scratch/narrow-mutations-only"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'alpha 的库（自证样本）' crates/alpha/src/lib.rs 'pub fn f() {}' 'pub fn f() { }' '-p alpha --lib' f > "$scratch/narrow-mutations-only/mutations-append.tsv"
  out="$scratch/out-narrow-build-mutations-only.txt"; checked=$((checked+1))
  narrow_case build-mutations-only --skip-apply "$scratch/narrow-mutations-only"
  if grep -q '^build' "$scratch/calls-narrow-build-mutations-only.txt" 2>/dev/null || ! grep -qF '  - build 本次未跑：这一批补丁没改到工作区里的包' "$out" || grep -qF '✓ build 过' "$out"; then
    echo "  ✗ 自检：build 那一步：补丁只改变异表时 build 应当不起、报 build 本次未跑，实际退出码 $rc，假 cargo 收到的 build 命令「$(grep '^build' "$scratch/calls-narrow-build-mutations-only.txt" 2>/dev/null | tr '\n' '|')」：$(grep -E 'build|✗' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # build 那一步认 { workspace = true } 的依赖（文件头 build 一段）：样本工作区 core、user、bystander，树根 [workspace.dependencies] 写 core = { path = "crates/core" }，
  #   user 的依赖写 core = { workspace = true }，bystander 不依赖谁；补丁只改 core：build-packages 那一段算出的包应当是 core 与 user（不含 bystander）
  local inherit_tree="$scratch/workspace-inherit-tree" inherit_member inherit_rc=0
  mkdir -p "$scratch/workspace-inherit-patch" "$inherit_tree"
  printf '[workspace]\nmembers = ["crates/core", "crates/user", "crates/bystander"]\n\n[workspace.dependencies]\ncore = { path = "crates/core" }\n' > "$inherit_tree/Cargo.toml"
  for inherit_member in core user bystander; do
    mkdir -p "$inherit_tree/crates/$inherit_member/src"; printf 'pub fn f() {}\n' > "$inherit_tree/crates/$inherit_member/src/lib.rs"
    printf '[package]\nname = "%s"\n' "$inherit_member" > "$inherit_tree/crates/$inherit_member/Cargo.toml"
  done
  printf '\n[dependencies]\ncore = { workspace = true }\n' >> "$inherit_tree/crates/user/Cargo.toml"
  printf -- '--- a/crates/core/src/lib.rs\n+++ b/crates/core/src/lib.rs\n@@ -1 +1 @@\n-pub fn f() {}\n+pub fn f() { }\n' > "$scratch/workspace-inherit-patch/crates.patch"
  out="$scratch/out-workspace-inherit.txt"; checked=$((checked+1))
  python3 - "$REBUILT" "$inherit_tree" build-packages "$scratch/workspace-inherit-patch" <<<"$MAIN_TREE_FINGERPRINT_PROGRAM" > "$out" 2>&1 || inherit_rc=$?
  if [[ $inherit_rc -ne 0 || "$(tr '\n' ' ' < "$out")" != "core user " ]]; then
    echo "  ✗ 自检：build 那一步：依赖方写 { workspace = true }（user 的 core = { workspace = true }，树根 [workspace.dependencies] 的 core 写了 path）、补丁只改 core 时，算出的包应当是 core 与 user，实际退出码 $inherit_rc，算出「$(tr '\n' ' ' < "$out" | clip_text 300)」"; failures=1; fi   # gate-lint:detail
  # 开跑那一刻的指纹同样认 { workspace = true }（文件头「开跑那一刻的指纹」，与 build 共用 path_dependencies）：同一棵样本工作区，补丁只改 user：
  #   record 那一段记下的起点应当有 user 与它依赖的 core 两个包目录，没有 bystander（起法与真跑那一处同：record <补丁目录>）
  local fingerprint_inherit="$scratch/workspace-inherit-fingerprint" fingerprint_inherit_rc=0
  mkdir -p "$fingerprint_inherit/patch"
  printf -- '--- a/crates/user/src/lib.rs\n+++ b/crates/user/src/lib.rs\n@@ -1 +1 @@\n-pub fn f() {}\n+pub fn f() { }\n' > "$fingerprint_inherit/patch/crates.patch"
  out="$scratch/out-workspace-inherit-fingerprint.txt"; checked=$((checked+1))
  python3 - "$REBUILT" "$inherit_tree" "$fingerprint_inherit/fingerprint" "$fingerprint_inherit/roots" "$fingerprint_inherit/rows" record "$fingerprint_inherit/patch" \
    <<<"$MAIN_TREE_FINGERPRINT_PROGRAM" > "$out" 2>&1 || fingerprint_inherit_rc=$?
  if [[ $fingerprint_inherit_rc -ne 0 ]] || ! grep -qxF "$inherit_tree/crates/core" "$fingerprint_inherit/roots" 2>/dev/null \
     || ! grep -qxF "$inherit_tree/crates/user" "$fingerprint_inherit/roots" 2>/dev/null || grep -qF bystander "$fingerprint_inherit/roots" 2>/dev/null; then
    echo "  ✗ 自检：开跑那一刻的指纹：依赖方写 { workspace = true }（user 的 core = { workspace = true }，树根 [workspace.dependencies] 的 core 写了 path）、补丁只改 user 时，起点应当有 user 与它依赖的 core、没有 bystander，实际退出码 $fingerprint_inherit_rc，起点「$(tr '\n' ' ' < "$fingerprint_inherit/roots" 2>/dev/null | clip_text 400)」：$(tail -n 1 "$out" | clip_text 200)"; failures=1; fi   # gate-lint:detail
  # build 那一步认树根 [patch.<源>] 里写了 path 的项（文件头 build 一段）：样本工作区 wcore、user、bystander，树根 [patch.crates-io] 写 wcore = { path = "crates/wcore" }，
  #   user 的依赖写成版本串 wcore = "0.1"（registry 来源，经 patch 换成工作区里的 wcore），bystander 不依赖谁；补丁只改 wcore：算出的包应当是 user 与 wcore（不含 bystander）
  local patch_tree="$scratch/patch-override-tree" patch_member patch_rc=0
  mkdir -p "$scratch/patch-override-patch" "$patch_tree"
  printf '[workspace]\nmembers = ["crates/wcore", "crates/user", "crates/bystander"]\n\n[patch.crates-io]\nwcore = { path = "crates/wcore" }\n' > "$patch_tree/Cargo.toml"
  for patch_member in wcore user bystander; do
    mkdir -p "$patch_tree/crates/$patch_member/src"; printf 'pub fn f() {}\n' > "$patch_tree/crates/$patch_member/src/lib.rs"
    printf '[package]\nname = "%s"\nedition = "2021"\n' "$patch_member" > "$patch_tree/crates/$patch_member/Cargo.toml"
  done
  printf '\n[dependencies]\nwcore = "0.1"\n' >> "$patch_tree/crates/user/Cargo.toml"
  printf -- '--- a/crates/wcore/src/lib.rs\n+++ b/crates/wcore/src/lib.rs\n@@ -1 +1 @@\n-pub fn f() {}\n+pub fn f() { }\n' > "$scratch/patch-override-patch/crates.patch"
  out="$scratch/out-patch-override.txt"; checked=$((checked+1))
  python3 - "$REBUILT" "$patch_tree" build-packages "$scratch/patch-override-patch" <<<"$MAIN_TREE_FINGERPRINT_PROGRAM" > "$out" 2>&1 || patch_rc=$?
  if [[ $patch_rc -ne 0 || "$(tr '\n' ' ' < "$out")" != "user wcore " ]]; then
    echo "  ✗ 自检：build 那一步：registry 依赖经树根 [patch.crates-io] 换成工作区里的包（user 写 wcore = \"0.1\"，树根 [patch.crates-io] 的 wcore 写了 path）、补丁只改 wcore 时，算出的包应当是 user 与 wcore，实际退出码 $patch_rc，算出「$(tr '\n' ' ' < "$out" | clip_text 300)」"; failures=1; fi   # gate-lint:detail
  # 同一份补丁、带 --skip-apply，真卡对拍那一格红：只重跑的门禁不点项（那一格不收点名）、submit 带 --whole-package-confirmed；输出由下面「重跑命令点名」那一格逐条判
  out="$scratch/out-verification-product-gpu-red.txt"
  FAKE_VERIFICATION_EXIT=1 narrow_case verification-product-gpu-red --skip-apply "$scratch/narrow-product-gpu"
  # 卡（文件头「显卡」）：假的那一道把每次收到的 GPU_CARDS_BY_PRIORITY 记进 card_cards（FAKE_VERIFICATION_CARDS），每行以「<TAB>cards=<值>」收尾
  local card_cards="$scratch/card-steps-cards.txt" card_lines card_matched
  # 环境里没有卡：补丁改到挂 gpu 特性的 gpu_pair 与产品 GPU 包的真卡对拍，变异行点名验证包的 setting（崩溃枚举用例）：
  #   那一道起的每一次（--list-items、崩溃枚举用例、挂特性的文件、真卡对拍）都不带卡串（这一层不领卡、不挑卡、不设卡）
  narrow_patch "$scratch/narrow-card-steps" crates/singlefs-verification/tests/gpu_pair.rs crates/singlefs-background-gpu-tests/tests/real_card.rs
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' '验证包的 setting（自证样本）' crates/demo/src/lib.rs 2 3 '-p singlefs-verification --release --test setting -- case_filter --exact' case_filter \
    > "$scratch/narrow-card-steps/mutations-append.tsv"
  out="$scratch/out-card-to-the-gate.txt"; : > "$card_cards"; checked=$((checked+1))
  FAKE_VERIFICATION_CARDS="$card_cards" narrow_case card-to-the-gate --skip-apply --crash-expand-up-to 6 "$scratch/narrow-card-steps"
  card_lines="$(grep -c . "$card_cards" 2>/dev/null)"
  card_matched="$(grep -c -- $'\tcards=none$' "$card_cards" 2>/dev/null)"
  if [[ $rc -ne 0 || "${card_lines:-0}" -eq 0 || "${card_matched:-0}" -ne "${card_lines:-0}" ]]; then
    echo "  ✗ 自检：卡：环境里没有卡时交给那一道的每一次应当都不带卡串（这一层不领卡、不挑卡、不设卡），实际退出码 $rc，那一道起了 ${card_lines:-0} 次、对得上的 ${card_matched:-0} 次：$(tr '\t\n' ' |' < "$card_cards")"; failures=1; fi   # gate-lint:detail
  # 环境里交的卡照传：环境里有 GPU_CARDS_BY_PRIORITY：补丁只改到产品 GPU 包的真卡对拍，那一道收到的是同一串卡（门禁照它原样交给 cargo）
  out="$scratch/out-card-from-the-start.txt"; : > "$card_cards"; checked=$((checked+1))
  GPU_CARDS_BY_PRIORITY=selftest-host:0:8G FAKE_VERIFICATION_CARDS="$card_cards" narrow_case card-from-the-start --skip-apply "$scratch/narrow-product-gpu"
  card_lines="$(grep -c . "$card_cards" 2>/dev/null)"
  card_matched="$(grep -c -- $'\tcards=selftest-host:0:8G$' "$card_cards" 2>/dev/null)"
  if [[ $rc -ne 0 || "${card_lines:-0}" -eq 0 || "${card_matched:-0}" -ne "${card_lines:-0}" ]]; then
    echo "  ✗ 自检：卡：环境里交的卡照传（环境里有 GPU_CARDS_BY_PRIORITY=selftest-host:0:8G）应当照原样交给那一道的每一次，实际退出码 $rc，那一道起了 ${card_lines:-0} 次、对得上的 ${card_matched:-0} 次：$(tr '\t\n' ' |' < "$card_cards")"; failures=1; fi   # gate-lint:detail
  # 那一道退 3 而交给它的根不是样本根（这里是快照）：判红
  out="$scratch/out-verification-exit-three.txt"; checked=$((checked+1))
  FAKE_VERIFICATION_EXIT=3 narrow_case verification-exit-three --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 1 ]] || ! grep -qF '退 3：退 3 只在样本根上算过，交给它的根不是样本根' "$out"; then
    echo "  ✗ 自检：验证包那一步：那一道退 3 而根不是它的样本根时应当判红、退 1，实际退出码 $rc：$(grep -E '验证包' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 那一道的 --list-items 退 1：读不出登记的用例与挂特性的文件，判红
  out="$scratch/out-verification-list-red.txt"; checked=$((checked+1))
  FAKE_VERIFICATION_LIST_EXIT=1 narrow_case verification-list-red --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 1 ]] || ! grep -qF -- '--list-items 退 1' "$out"; then
    echo "  ✗ 自检：验证包那一步：那一道 --list-items 退 1 时应当判红、退 1，实际退出码 $rc：$(grep -E '验证包' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 跑在第二台上（task-server 经 multi-host-run.sh --peer 派来的那一趟：带着起活那一方的回读握手与这一趟的本地目录，两台对称、没有认台的变量）：照起那一道、报过、退 0
  out="$scratch/out-verification-peer.txt"; : > "$verification_calls"; checked=$((checked+1))
  GPU_QUIESCE_ORIGIN_READ_BACK=1 MULTI_HOST_RUN_PEER_SCRATCH="$scratch/peer-scratch" FAKE_VERIFICATION_CALLS="$verification_calls" narrow_case verification-peer --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 0 || "$(verification_starts '--check ')" -ne 1 ]] || ! grep -qF '✓ 验证包那一步（登记的崩溃枚举用例）过' "$out"; then
    echo "  ✗ 自检：验证包那一步：跑在第二台上时应当照起那一道、报过、退 0，实际退出码 $rc，那一道被起 $(grep -c . "$verification_calls") 次：$(grep -E '验证包|→' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 缩崩溃域（文件头「验证包那一步」）：--crash-expand-up-to 6 把 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 与 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO 都设成 6 交给那一道（假的那一道两个同值才记那个值，
  #   只交了一个记「amplification=… judge=…」，下面各格比的 6、4、5、unset 因此两个变量都核）、汇总写缩过域、不算门禁通过、把崩溃状态块那一行的现判与复用写进汇总；
  #   不给就是整个域：那一道收到的是 unset、汇总不写缩过域；环境里已设的变量当作给了同一个值；值不是非负整数退 2、一样都不做；缩过域的那一趟红了，只重跑的命令带同一个展开深度的两个变量
  #   （弄坏开关 expand-up-to-not-passed、judge-expand-variable-not-passed、scaled-run-unlabelled、rerun-command-unscaled、expand-up-to-unchecked 下对应的格红）
  out="$scratch/out-verification-scaled.txt"; : > "$verification_environment"; checked=$((checked+1))
  FAKE_VERIFICATION_ENVIRONMENT="$verification_environment" narrow_case verification-scaled --skip-apply --crash-expand-up-to 6 "$scratch/narrow-core"
  if [[ $rc -ne 0 ]] || [[ "$(cat "$verification_environment")" != 6 ]] || ! grep -q '汇总：.*缩过域、不算门禁通过（SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6，只往小缩）' "$out" \
     || ! grep -q '汇总：.*崩溃状态块 现判 3、复用 5、拒算 0' "$out"; then
    echo "  ✗ 自检：验证包那一步：--crash-expand-up-to 6 应当把 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 与 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO 都设成 6 交给那一道、汇总写缩过域、不算门禁通过、把崩溃状态块的现判与复用写进汇总，实际退出码 $rc，那一道收到的变量「$(tr '\n' ' ' < "$verification_environment")」：$(grep '汇总：' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-verification-unscaled.txt"; : > "$verification_environment"; checked=$((checked+1))
  FAKE_VERIFICATION_ENVIRONMENT="$verification_environment" narrow_case verification-unscaled --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 0 ]] || [[ "$(cat "$verification_environment")" != unset ]] || grep -qF '缩过域' "$out"; then
    echo "  ✗ 自检：验证包那一步：不给 --crash-expand-up-to 时那一道收到的变量应当是 unset、汇总不写缩过域，实际退出码 $rc，那一道收到的变量「$(tr '\n' ' ' < "$verification_environment")」：$(grep '汇总：' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-verification-scaled-environment.txt"; : > "$verification_environment"; checked=$((checked+1))
  SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=4 FAKE_VERIFICATION_ENVIRONMENT="$verification_environment" narrow_case verification-scaled-environment --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 0 ]] || [[ "$(cat "$verification_environment")" != 4 ]] || ! grep -q '汇总：.*缩过域、不算门禁通过（SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=4，只往小缩）' "$out"; then
    echo "  ✗ 自检：验证包那一步：环境里已设展开深度变量时当作给了同一个值（那一道的两个变量都收到 4、汇总写缩过域），实际退出码 $rc，那一道收到的变量「$(tr '\n' ' ' < "$verification_environment")」：$(grep '汇总：' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-verification-bad-argument.txt"; checked=$((checked+1))
  narrow_case verification-bad-argument --skip-apply --crash-expand-up-to abc "$scratch/narrow-core"
  if [[ $rc -ne 2 ]] || [[ -s "$scratch/calls-narrow-verification-bad-argument.txt" ]] || ! grep -qF '不是非负整数' "$out"; then
    echo "  ✗ 自检：--crash-expand-up-to 不是非负整数时应当退 2、一样都不做（参数写 abc），实际退出码 $rc，假 cargo 收到 $(grep -c . "$scratch/calls-narrow-verification-bad-argument.txt" 2>/dev/null) 条：$(grep -E '✗' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-verification-bad-environment.txt"; checked=$((checked+1))
  SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=abc narrow_case verification-bad-environment --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 2 ]] || [[ -s "$scratch/calls-narrow-verification-bad-environment.txt" ]] || ! grep -qF '不是非负整数' "$out"; then
    echo "  ✗ 自检：--crash-expand-up-to 不是非负整数时应当退 2、一样都不做（环境里的变量写 abc），实际退出码 $rc，假 cargo 收到 $(grep -c . "$scratch/calls-narrow-verification-bad-environment.txt" 2>/dev/null) 条：$(grep -E '✗' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  out="$scratch/out-verification-missing-value.txt"; checked=$((checked+1))
  narrow_case verification-missing-value --skip-apply "$scratch/narrow-core" --crash-expand-up-to
  if [[ $rc -ne 2 ]] || [[ -s "$scratch/calls-narrow-verification-missing-value.txt" ]] || ! grep -qF '后面要跟一个非负整数' "$out"; then
    echo "  ✗ 自检：--crash-expand-up-to 后面没有值时应当退 2、一样都不做，实际退出码 $rc，假 cargo 收到 $(grep -c . "$scratch/calls-narrow-verification-missing-value.txt" 2>/dev/null) 条：$(grep -E '✗' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  # 只重跑的命令带不带两个变量，认 submit 的 -- 之后紧跟 env 两个变量：submit 与 -- 之间带不带 --whole-package-confirmed（不点项的那几条带）归「验证包那一步：按块那一路判红时…」与重跑命令点名那几格判，这里两种都认
  local paired_rerun_submit="task-server/task submit (--whole-package-confirmed $WHOLE_PACKAGE_CONFIRMATION_PLACEHOLDER )?-- env SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=6 bash"
  out="$scratch/out-verification-scaled-red.txt"; checked=$((checked+1))
  FAKE_VERIFICATION_EXIT=1 narrow_case verification-scaled-red --skip-apply --crash-expand-up-to 6 "$scratch/narrow-core"
  if [[ $rc -ne 1 ]] || ! grep -qE -- "$paired_rerun_submit" "$out" || ! grep -q '汇总：.*缩过域、不算门禁通过' "$out"; then
    echo "  ✗ 自检：验证包那一步：缩过域的那一趟红了，只重跑的命令应当带同一个展开深度的两个变量（task-server/task submit -- env SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=6 bash …）、汇总写缩过域，实际退出码 $rc：$(grep -E '→ 怎么办|汇总：' "$out" | head -2 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 缩崩溃域四处都交两个变量（文件头「验证包那一步」）：那一道（假的那一道记的值）、验证包目标的 cargo test（假 cargo 记的值）、算键的请求（交给 test-plan 的参数里 variable: 两项）、红了之后只重跑的命令；
  #   参数给的 6 一趟、环境里已设的 6 一趟（来历三种里估算回落的由下面崩溃域那几格核假的那一道记的值）。弄坏开关 judge-expand-variable-not-passed 下这一格红。
  #   算键的请求只在验证包有测试属性时才起（TESTED_PACKAGES），所以这一格用 narrow-tree 的一份拷贝：验证包的 setting.rs 写一条 #[test]、样本档目录钉死复用哨兵、git init，
  #   起 test-plan 要的 cargo -V 与 rustc -vV 由 pairing_bin 里的两个假的答，cargo 别的子命令转给假 cargo
  local pairing_cargo_environment="$scratch/cargo-environment-pairing.txt" pairing_plan pairing_origin pairing_problems
  local pairing_tree="$scratch/pairing-tree" pairing_bin="$scratch/bin-pairing"
  mkdir -p "$pairing_bin" && cp -r -- "$narrow_tree" "$pairing_tree" && git init -q "$pairing_tree" && mkdir -p "$pairing_tree/.unit-verdict-sample" \
    && printf '[package]\nname = "singlefs-verification"\nversion = "0.1.0"\nedition = "2021"\n' > "$pairing_tree/crates/singlefs-verification/Cargo.toml" \
    && printf '#[test]\nfn pairing_case() {}\n' > "$pairing_tree/crates/singlefs-verification/tests/setting.rs" \
    || { echo "  ✗ 自检：缩崩溃域：建不成算键请求要的样本树 $pairing_tree"; failures=1; }   # gate-lint:detail
  printf '#!/usr/bin/env bash\n[[ "$1" != -V ]] || { echo "cargo 0.0.0-merge-and-verify-selftest"; exit 0; }\nexec "%s/cargo" "$@"\n' "$fake_cargo_dir" > "$pairing_bin/cargo"
  printf '#!/usr/bin/env bash\ncase "${1:-}" in\n  -V) echo "rustc 0.0.0-merge-and-verify-selftest" ;;\n  -vV) printf "%%s\\n" "rustc 0.0.0-merge-and-verify-selftest" "host: x86_64-unknown-linux-gnu" "release: 0.0.0" ;;\nesac\nexit 0\n' > "$pairing_bin/rustc"
  chmod +x "$pairing_bin/cargo" "$pairing_bin/rustc"
  pairing_case() {   # <来历>：参数给的与环境里已设的两种起法，样本树是 pairing-tree，别的与 narrow_case 同
    local origin="$1"
    local -a pairing_arguments=(--skip-apply)
    [[ "$origin" != argument ]] || pairing_arguments+=(--crash-expand-up-to 6)
    ( [[ "$origin" != environment ]] || export SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6
      FAKE_VERIFICATION_EXIT=1 FAKE_CARGO_ENVIRONMENT="$pairing_cargo_environment" FAKE_VERIFICATION_ENVIRONMENT="$verification_environment" \
      FAKE_CARGO_MODE=green FAKE_CARGO_CALLS="$scratch/calls-narrow-verification-pairing-$origin.txt" PATH="$pairing_bin:$fake_cargo_dir:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" \
      MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-narrow-verification-pairing-$origin" MERGE_AND_VERIFY_TREE="$pairing_tree" env -u CARGO_TARGET_DIR bash "$0" "${pairing_arguments[@]}" "$scratch/narrow-core" ) > "$out" 2>&1; rc=$?
  }
  for pairing_origin in argument environment; do
    out="$scratch/out-verification-pairing-$pairing_origin.txt"; : > "$verification_environment"; : > "$pairing_cargo_environment"; checked=$((checked+1))
    pairing_case "$pairing_origin"
    pairing_plan="$scratch/logs-narrow-verification-pairing-$pairing_origin/test-plan.singlefs-verification.arguments"; pairing_problems=""
    [[ -f "$pairing_plan" ]] || pairing_problems+="验证包没起 test-plan（$pairing_plan 不在；输出里的按条复用与 ✗ 行「$(grep -E '按条复用|✗' "$out" | head -3 | clip_text 200 | tr '\n' '|')」）；"
    [[ "$(head -n 1 "$verification_environment")" == 6 ]] || pairing_problems+="那一道收到「$(head -n 1 "$verification_environment")」；"
    grep -q -- $'^6\ttest .*-p singlefs-verification ' "$pairing_cargo_environment" || pairing_problems+="验证包的 cargo test 收到「$(grep -- '-p singlefs-verification ' "$pairing_cargo_environment" | cut -f1 | tr '\n' '|')」；"
    grep -qxF 'variable:SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6' "$pairing_plan" 2>/dev/null && grep -qxF 'variable:SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=6' "$pairing_plan" 2>/dev/null \
      || pairing_problems+="算键的请求里 variable: 是「$(grep '^variable:' "$pairing_plan" 2>/dev/null | tr '\n' '|')」；"
    grep -qE -- "$paired_rerun_submit" "$out" || pairing_problems+="只重跑的命令里没有两个变量；"
    if [[ $rc -ne 1 || -n "$pairing_problems" ]]; then
      echo "  ✗ 自检：缩崩溃域（来历：$pairing_origin）：那一道、验证包的 cargo test、算键的请求、只重跑的命令都应当带 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 与 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO 两个、值同为 6，实际退出码 $rc：$pairing_problems"; failures=1; fi   # gate-lint:detail
  done
  # 崩溃域上限（文件头「崩溃域上限」）：判的根是 narrow-tree（合表三项各一条用例）、估算库是真的那一份、读库程序经假 cargo（没打 CRASH_LIBRARY_ITEM，读不到块账，估数走上一趟真跑的记录）。
  #   点名的格都以「崩溃域：」起头；各格照 narrow-core 起（点名 demo-case 一条），门禁收到的变量看 verification_environment 的最前面一行（whole-package-tests 下后面还跟一次 feature-gated:lib）
  write_case_record "$scratch/home-narrow" whole-domain demo full demo-case 300000
  write_case_record "$scratch/home-narrow" narrowed demo 6 demo-case 225
  write_case_record "$scratch/home-no-depth" whole-domain demo full demo-case 300000
  write_case_record "$scratch/home-only-narrowed" narrowed demo 6 demo-case 225
  write_case_record "$scratch/home-given" whole-domain demo full demo-case 300000
  write_case_record "$scratch/home-given" narrowed demo 4 demo-case 8
  # domain_fell_back <标签>：这一趟（$out、$rc）缩到小值接着跑：退 0、那一道收到小值、打这个值的来历、别的步照跑、汇总写缩过域：对了交 0
  domain_fell_back() {
    [[ $rc -eq 0 && "$(head -n 1 "$verification_environment")" == "$CRASH_DOMAIN_FALLBACK_DEPTH" && -n "$(narrow_tests "$1")" ]] \
      && grep -qF -- "展开深度上限取 $CRASH_DOMAIN_FALLBACK_DEPTH 接着跑（这个值是估不出时合入后验证固定缩到的深度，等同给 --crash-expand-up-to $CRASH_DOMAIN_FALLBACK_DEPTH" "$out" \
      && grep -q "汇总：.*缩过域、不算门禁通过（SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=$CRASH_DOMAIN_FALLBACK_DEPTH，只往小缩）" "$out"
  }
  # 不超上限（默认家目录：上一趟整个域 100 态）：照整个域跑，不写缩过域，打放行那一句
  out="$scratch/out-crash-domain-within.txt"; : > "$verification_environment"; checked=$((checked+1))
  FAKE_VERIFICATION_ENVIRONMENT="$verification_environment" narrow_case crash-domain-within --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 0 || "$(head -n 1 "$verification_environment")" != unset ]] || grep -qF '缩过域' "$out" || ! grep -qF '点名的 1 项上一趟不缩域真跑共 100 态，全部现判也不超上限，放行' "$out"; then
    echo "  ✗ 自检：崩溃域：不超上限应当照整个域跑（那一道收到 unset）、不写缩过域、打放行那一句，实际退出码 $rc，那一道收到「$(tr '\n' '|' < "$verification_environment")」：$(grep -E '崩溃域|✗' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 超上限且算得出 n（上一趟整个域 300000 态，缩到 6 的那一趟 225 态）：用 6 接着跑、打一行说明；读库程序只点名这一趟点名的项与它的测试目标文件
  out="$scratch/out-crash-domain-narrowed.txt"; : > "$verification_environment"; checked=$((checked+1))
  SINGLEFS_CRASH_AMPLIFICATION_HOME="$scratch/home-narrow" FAKE_VERIFICATION_ENVIRONMENT="$verification_environment" narrow_case crash-domain-narrowed --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 0 || "$(head -n 1 "$verification_environment")" != 6 ]] || ! grep -qF '缩到展开深度 6 接着跑（缩过域、不算门禁通过）' "$out" \
     || ! grep -qxF 'run --release --quiet --offline -p singlefs-verification --bin crash_library_reader -- items --item demo crates/singlefs-verification/tests/setting.rs' "$scratch/calls-narrow-crash-domain-narrowed.txt"; then
    echo "  ✗ 自检：崩溃域：超上限且算得出 n 应当用这个 n 缩域接着跑、打一行说明、读库程序只点名这一项，实际退出码 $rc，那一道收到「$(tr '\n' '|' < "$verification_environment")」，假 cargo 收到 $(grep -c '^run ' "$scratch/calls-narrow-crash-domain-narrowed.txt") 条 run：$(grep -E '崩溃域|✗' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 同一趟的汇总
  checked=$((checked+1))
  if ! grep -q '汇总：.*缩过域、不算门禁通过（SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6，只往小缩）' "$out"; then
    echo "  ✗ 自检：崩溃域：缩域接着跑的那一趟汇总应当写缩过域、不算门禁通过，实际：$(grep '汇总：' "$out" | head -1)"; failures=1; fi   # gate-lint:detail
  # 验证包的 setting 是目录式测试目标（tests/setting/main.rs 连同它 mod 声明的 steps.rs）：同样超上限缩域，读库程序点名 main.rs 与 steps.rs（估算那一段给 Subject 带仓根，
  #   弄坏开关 crash-domain-subject-without-root 下落回 tests/setting.rs）；这一格之后把 tests/setting.rs 摆回去
  setting_directory="$narrow_tree/crates/singlefs-verification/tests/setting"; mkdir -p "$setting_directory"; mv -- "$setting_directory.rs" "$setting_directory/main.rs"
  printf 'mod steps;\n' >> "$setting_directory/main.rs"; printf 'fn steps() {}\n' > "$setting_directory/steps.rs"
  out="$scratch/out-crash-domain-directory-target.txt"; : > "$verification_environment"; checked=$((checked+1))
  SINGLEFS_CRASH_AMPLIFICATION_HOME="$scratch/home-narrow" FAKE_VERIFICATION_ENVIRONMENT="$verification_environment" narrow_case crash-domain-directory-target --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 0 ]] || ! grep -qxF 'run --release --quiet --offline -p singlefs-verification --bin crash_library_reader -- items --item demo crates/singlefs-verification/tests/setting/main.rs crates/singlefs-verification/tests/setting/steps.rs' "$scratch/calls-narrow-crash-domain-directory-target.txt"; then
    echo "  ✗ 自检：崩溃域：验证包的 setting 是目录式测试目标，读库程序应当点名它的 main.rs 与 steps.rs，实际退出码 $rc，假 cargo 收到 $(grep '^run ' "$scratch/calls-narrow-crash-domain-directory-target.txt" | tr '\n' '|' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  rm -rf -- "${setting_directory:?}"; printf 'fn main() {}\n' > "$setting_directory.rs"
  # 估不出（一条运行记录都没有）：缩到小值接着跑、打这个值的来历、汇总写缩过域
  out="$scratch/out-crash-domain-unknown.txt"; : > "$verification_environment"; checked=$((checked+1))
  SINGLEFS_CRASH_AMPLIFICATION_HOME="$scratch/home-empty" FAKE_VERIFICATION_ENVIRONMENT="$verification_environment" narrow_case crash-domain-unknown --skip-apply "$scratch/narrow-core"
  if ! domain_fell_back crash-domain-unknown || ! grep -qF '要现判多少崩溃状态估不出' "$out"; then
    echo "  ✗ 自检：崩溃域：估不出应当缩到小值 $CRASH_DOMAIN_FALLBACK_DEPTH 接着跑、打这个值的来历、汇总写缩过域、别的步照跑，实际退出码 $rc，那一道收到「$(tr '\n' '|' < "$verification_environment")」：$(grep -E '崩溃域|验证包那一步|汇总' "$out" | head -4 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 超上限而算不出 n（只有整个域那一趟的记录）：同样缩到小值接着跑
  out="$scratch/out-crash-domain-no-depth.txt"; : > "$verification_environment"; checked=$((checked+1))
  SINGLEFS_CRASH_AMPLIFICATION_HOME="$scratch/home-no-depth" FAKE_VERIFICATION_ENVIRONMENT="$verification_environment" narrow_case crash-domain-no-depth --skip-apply "$scratch/narrow-core"
  if ! domain_fell_back crash-domain-no-depth || ! grep -qF '要现判约 300000 态，超过上限 200000 态，又算不出能压到上限以内的 n' "$out"; then
    echo "  ✗ 自检：崩溃域：超上限而算不出 n 应当同样缩到小值 $CRASH_DOMAIN_FALLBACK_DEPTH 接着跑，实际退出码 $rc，那一道收到「$(tr '\n' '|' < "$verification_environment")」：$(grep -E '崩溃域|验证包那一步|汇总' "$out" | head -4 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 估不出而有缩域记录（只有缩到 6 的那一趟，整个域算不出）：算得出 n 的照算出的
  out="$scratch/out-crash-domain-only-narrowed.txt"; : > "$verification_environment"; checked=$((checked+1))
  SINGLEFS_CRASH_AMPLIFICATION_HOME="$scratch/home-only-narrowed" FAKE_VERIFICATION_ENVIRONMENT="$verification_environment" narrow_case crash-domain-only-narrowed --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 0 || "$(head -n 1 "$verification_environment")" != 6 ]] || ! grep -qF '要现判多少崩溃状态估不出' "$out" || ! grep -qF '缩到展开深度 6 接着跑（缩过域、不算门禁通过）' "$out" \
     || ! grep -q '汇总：.*缩过域、不算门禁通过（SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=6，只往小缩）' "$out"; then
    echo "  ✗ 自检：崩溃域：估不出而有缩域记录时应当照算得出的 n（6）缩域接着跑、打一行说明、汇总写缩过域，实际退出码 $rc，那一道收到「$(tr '\n' '|' < "$verification_environment")」：$(grep -E '崩溃域|验证包那一步|汇总' "$out" | head -4 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 点名的用例不在合表里：算估不出，同样缩到小值接着跑
  out="$scratch/out-crash-domain-unplaced.txt"; : > "$verification_environment"; checked=$((checked+1))
  FAKE_VERIFICATION_CASES='demo-case not-in-the-table-case' FAKE_VERIFICATION_ENVIRONMENT="$verification_environment" narrow_case crash-domain-unplaced --skip-apply "$scratch/narrow-core"
  if ! domain_fell_back crash-domain-unplaced || ! grep -qF '不在合表里，估不出要现判多少态（crash-case:not-in-the-table-case' "$out"; then
    echo "  ✗ 自检：崩溃域：点名的用例不在合表里应当算估不出、缩到小值 $CRASH_DOMAIN_FALLBACK_DEPTH 接着跑，实际退出码 $rc，那一道收到「$(tr '\n' '|' < "$verification_environment")」：$(grep -E '崩溃域|验证包那一步|汇总' "$out" | head -4 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 给了值（参数 6，环境里的 5）：照给的，不估（记录里算得出的是 4）
  out="$scratch/out-crash-domain-given.txt"; : > "$verification_environment"; checked=$((checked+1))
  SINGLEFS_CRASH_AMPLIFICATION_HOME="$scratch/home-given" FAKE_VERIFICATION_ENVIRONMENT="$verification_environment" narrow_case crash-domain-given --skip-apply --crash-expand-up-to 6 "$scratch/narrow-core"
  local given_argument_exit="$rc" given_argument_received
  given_argument_received="$(head -n 1 "$verification_environment")"
  : > "$verification_environment"
  SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5 SINGLEFS_CRASH_AMPLIFICATION_HOME="$scratch/home-given" FAKE_VERIFICATION_ENVIRONMENT="$verification_environment" narrow_case crash-domain-given-environment --skip-apply "$scratch/narrow-core"
  if [[ "$given_argument_exit" -ne 0 || "$given_argument_received" != 6 || $rc -ne 0 || "$(head -n 1 "$verification_environment")" != 5 || -e "$scratch/logs-narrow-crash-domain-given/verification-crash-domain.log" ]]; then
    echo "  ✗ 自检：崩溃域：给了值应当照给的、不估（参数 6 那一趟退 $given_argument_exit、那一道收到「$given_argument_received」，环境里 5 那一趟退 $rc、那一道收到「$(tr '\n' '|' < "$verification_environment")」）"; failures=1; fi   # gate-lint:detail
  # 只数这一趟点名的用例：demo 那一项另有没点名的用例与放量分项（都没有记录），这一步不起，不算
  cp -- "$narrow_tree/.claude/gate.d/crash-point-replay-items.tsv" "$scratch/items-kept.tsv"
  printf 'demo\tcrash-case:demo-case crash-case:demo-case-not-named\tdemo-flow\n' > "$narrow_tree/.claude/gate.d/crash-point-replay-items.tsv"
  out="$scratch/out-crash-domain-named-only.txt"; checked=$((checked+1))
  narrow_case crash-domain-named-only --skip-apply "$scratch/narrow-core"
  cp -- "$scratch/items-kept.tsv" "$narrow_tree/.claude/gate.d/crash-point-replay-items.tsv"
  if [[ $rc -ne 0 ]] || ! grep -qF '点名的 1 项上一趟不缩域真跑共 100 态，全部现判也不超上限，放行' "$out"; then
    echo "  ✗ 自检：崩溃域：只数这一趟点名的用例（放量分项与没点名的用例不算），实际退出码 $rc：$(grep -E '崩溃域|✗' "$out" | head -3 | tr '\n' ' ')；估算那一步的判定「$(cat -- "$scratch/logs-narrow-crash-domain-named-only/verification-crash-domain.decision" 2>/dev/null | tr '\n' '|' | clip_text 400)」，它的日志「$(cat -- "$scratch/logs-narrow-crash-domain-named-only/verification-crash-domain.log" 2>/dev/null | tr '\n' '|' | clip_text 300)」；本进程的 SINGLEFS_CRASH_AMPLIFICATION_HOME「${SINGLEFS_CRASH_AMPLIFICATION_HOME:-未设}」，运行记录「$(ls -l --time-style=full-iso "$scratch/crash-home/runs/crash-point-replay-whole-domain" 2>&1 | tail -n +2 | awk '{ print $6, $7, $9 }' | tr '\n' '|')」，这一趟输出里估算相关的行「$(grep -nE '指定范围|崩溃枚举用例|估不出|缩到|✗|!' "$out" | head -6 | clip_text 240 | tr '\n' '|')」"; failures=1; fi   # gate-lint:detail
  # 估算库起不来：判红、验证包那一步不起，不当估不出
  printf 'raise SystemExit(3)\n' > "$scratch/broken-crash-domain-library.py"
  out="$scratch/out-crash-domain-broken-library.txt"; : > "$verification_calls"; checked=$((checked+1))
  MERGE_AND_VERIFY_CRASH_DOMAIN_LIBRARY="$scratch/broken-crash-domain-library.py" FAKE_VERIFICATION_CALLS="$verification_calls" narrow_case crash-domain-broken-library --skip-apply "$scratch/narrow-core"
  if [[ $rc -ne 1 || "$(verification_starts '--check ')" -ne 0 ]] || ! grep -qF '估崩溃域的那一步退 3' "$out"; then
    echo "  ✗ 自检：崩溃域：估算库起不来应当判红、退 1、不起那一道，实际退出码 $rc，那一道被起 $(grep -c . "$verification_calls") 次：$(grep -E '验证包那一步|→' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 跑着的时候主工作区（要验的树）里那份源码被改（文件头「快照」）：假 cargo 的 test 一开跑就把要验的树里的 lib.rs 与那一道门禁改掉，
  #   再照它此刻所在目录里的 lib.rs 报一条测试；补丁目录只有变异行（点名 demo 的库与验证包的 setting），验证包那一步照默认起快照里那一份门禁
  #   （这一格不设 MERGE_AND_VERIFY_VERIFICATION_STAGE），那一份答 --list-items（setting 里登记了一条崩溃枚举用例）、按块那一路报它收到的根里 lib.rs 的内容
  #   （当成复用键算的是哪一份源码）。测试的结论与复用的键都要照开跑时那一份（source_at_start）；
  #   弄坏开关 verify-in-main-tree 下测试编的是改后的那一份、验证包那一步起的是改后的那一份门禁，这一格红
  local edited_tree="$scratch/edited-tree" edited_bin="$scratch/edited-bin"
  mkdir -p "$edited_tree/crates/demo/src" "$edited_tree/crates/singlefs-verification/tests" "$edited_tree/.claude/gate.d" "$edited_bin" "$scratch/edited-patch"
  write_workspace "$edited_tree" singlefs-verification
  write_crash_domain_tables "$edited_tree" edited-case
  printf 'source_at_start\n' > "$edited_tree/crates/demo/src/lib.rs"
  printf 'fn main() {}\n' > "$edited_tree/crates/singlefs-verification/tests/setting.rs"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'demo 的库（自证样本）' crates/demo/src/lib.rs a b '-p demo --lib' f '验证包的 setting（自证样本）' crates/demo/src/lib.rs a b '-p singlefs-verification --test setting' f \
    > "$scratch/edited-patch/mutations-append.tsv"
  cat > "$edited_tree/.claude/gate.d/crash-point-replay.sh" <<'FAKE'
#!/usr/bin/env bash
# 开跑时那一份假门禁：--list-items 列 setting 里登记的一条崩溃枚举用例；按块那一路报它收到的根（最后一个参数）里 lib.rs 的内容
if [[ "$1" == --list-items ]]; then printf 'crash-case:edited-case\tsinglefs-verification:setting:case_function\n'; exit 0; fi
echo "  ✓ 崩溃状态块：现判 1、复用 0、拒算 0（键照 $(cat "${@: -1}/crates/demo/src/lib.rs") 算，开跑时那一份门禁）"
FAKE
  cat > "$edited_bin/cargo" <<'FAKE'
#!/usr/bin/env bash
# 假 cargo：test 一开跑就改要验的树（FAKE_EDITED_TREE）里的 lib.rs 与那一道门禁，再照此刻所在目录里的 lib.rs 报一条测试；别的子命令退 0
[[ "$1" == test ]] || exit 0
printf 'source_edited_mid_run\n' > "$FAKE_EDITED_TREE/crates/demo/src/lib.rs"
printf '%s\n' '#!/usr/bin/env bash' 'echo "  ✓ 崩溃状态块：现判 1、复用 0、拒算 0（键照主工作区改后的那一份门禁算）"' > "$FAKE_EDITED_TREE/.claude/gate.d/crash-point-replay.sh"
printf 'running 1 test\ntest tests::compiled_%s ... ok\n\ntest result: ok. 1 passed; 0 failed\n' "$(cat crates/demo/src/lib.rs)"
FAKE
  chmod +x "$edited_bin/cargo"
  out="$scratch/out-edited.txt"; checked=$((checked+1))
  PATH="$edited_bin:$PATH" FAKE_EDITED_TREE="$edited_tree" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-edited" \
    MERGE_AND_VERIFY_TREE="$edited_tree" env -u CARGO_TARGET_DIR -u MERGE_AND_VERIFY_VERIFICATION_STAGE bash "$0" --skip-apply "$scratch/edited-patch" > "$out" 2>&1; rc=$?
  if [[ $rc -ne 0 || "$(cat "$edited_tree/crates/demo/src/lib.rs" 2>/dev/null)" != source_edited_mid_run ]] \
     || ! grep -qxF 'test tests::compiled_source_at_start ... ok' "$scratch/logs-edited/test.log" 2>/dev/null \
     || ! grep -qF '键照 source_at_start 算，开跑时那一份门禁' "$scratch/logs-edited/"verification-*.log 2>/dev/null; then
    echo "  ✗ 自检：快照：跑着的时候主工作区里那份源码被改，结论与复用键应当照开跑时那一份算（测试编 source_at_start、验证包那一步起快照里那一份门禁、键照 source_at_start 算），实际退出码 $rc，测试：$(grep '^test tests::' "$scratch/logs-edited/test.log" 2>/dev/null | tr '\n' ' ')，验证包那一步：$(cat "$scratch/logs-edited/"verification-*.log 2>/dev/null | grep '崩溃状态块' | head -1)"; failures=1; fi   # gate-lint:detail
  # ── 逐项点名与按条复用（文件头「逐项点名与按条复用」），点名的格都以「逐项与复用：」起头 ─────────
  #   样本工作区的 alpha 有三条测试函数（库里 tests::plain、tests::uses_helper，tests/beta.rs 里 beta_case），uses_helper 调 helper、spare 谁都不调；
  #   假 cargo 照收到的 --lib / --test / --doc 与 -- 之后的 --exact --skip 打 alpha 的几段（段头、结果行、汇总行带 finished in 0.00s），FAKE_UNITS_RED 点名的那一条打 FAILED。
  #   样本树下有 .unit-verdict-sample/（admission.py 不抽复用哨兵：条数钉死）。每一格各拷一份样本树、各自 git init（判定库在临时服务里按库目录分命名空间，几格互不串）；
  #   同一格里的几趟用同一个日志目录（快照每趟落在同一条路径上，判定库是同一份）
  local units_template="$scratch/units-template" units_bin="$scratch/bin-units" units_calls="" units_logs whole_done_line named_done_line affected_name reuse_check_output reuse_check_rc
  local keys_tree units_round
  mkdir -p "$units_template/crates/alpha/src" "$units_template/crates/alpha/tests" "$units_template/crates/singlefs-verification" "$units_template/.unit-verdict-sample" "$units_bin"
  printf '[workspace]\nmembers = ["crates/alpha", "crates/singlefs-verification"]\n' > "$units_template/Cargo.toml"
  printf '[package]\nname = "alpha"\nversion = "0.1.0"\nedition = "2021"\n' > "$units_template/crates/alpha/Cargo.toml"
  printf '[package]\nname = "singlefs-verification"\nversion = "0.1.0"\nedition = "2021"\n' > "$units_template/crates/singlefs-verification/Cargo.toml"
  cat > "$units_template/crates/alpha/src/lib.rs" <<'RUST'
pub fn helper() -> u32 {
    1
}

pub fn spare() -> u32 {
    7
}

#[cfg(test)]
mod tests {
    #[test]
    fn uses_helper() {
        assert!(super::helper() > 0);
    }

    #[test]
    fn plain() {}
}
RUST
  printf '#[test]\nfn beta_case() {}\n' > "$units_template/crates/alpha/tests/beta.rs"
  # 每一趟都带的补丁目录：只有变异行，点名 alpha 的库、测试目标 beta 与文档测试（挑测试目标照它挑出这三项，文件头「挑测试目标」）
  local units_patch="$scratch/units-patch"
  mkdir -p "$units_patch"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'alpha 的库（自证样本）' crates/alpha/src/lib.rs 1 2 '-p alpha --lib -- tests' uses_helper \
    'alpha 的 beta（自证样本）' crates/alpha/tests/beta.rs a b '-p alpha --test beta -- beta_case' beta_case 'alpha 的文档测试（自证样本）' crates/alpha/src/lib.rs 1 2 '-p alpha --doc' doc \
    > "$units_patch/mutations-append.tsv"
  cat > "$units_bin/cargo" <<'FAKE'
#!/usr/bin/env bash
# 假 cargo（按条复用那几格）：-V 打版本；test 照收到的 --lib / --test <名> / --doc（一个都没点就是全部）与 -- 之后的 --exact --skip <名> 打 alpha 的几段，
# FAKE_UNITS_RED=<libtest 全名> 那一条打 FAILED、退 101；别的子命令退 0。每次起都把收到的参数记进 FAKE_CARGO_CALLS（设了的话）
[[ -z "${FAKE_CARGO_CALLS:-}" ]] || printf '%s\n' "$*" >> "$FAKE_CARGO_CALLS"
case "$1" in
  -V) echo "cargo 0.0.0-merge-and-verify-selftest"; exit 0 ;;
  test) ;;
  *) exit 0 ;;
esac
lib=0; doc=0; tests=" "; skips=" "; selected=0; after=0; previous=""
for word in "$@"; do
  if ((after)); then [[ "$previous" == --skip ]] && skips+="$word "; previous="$word"; continue; fi
  case "$word" in
    --) after=1 ;;
    --lib) lib=1; selected=1 ;;
    --doc) doc=1; selected=1 ;;
  esac
  [[ "$previous" == --test ]] && { tests+="$word "; selected=1; }
  previous="$word"
done
((selected)) || { lib=1; doc=1; tests=" beta "; }
red=0
section() {   # <段头> <测试名…>
  local header="$1" name ran=0 filtered=0 failed=0
  shift
  printf '     %s\n\n' "$header"
  for name in "$@"; do if [[ "$skips" == *" $name "* ]]; then filtered=$((filtered + 1)); else ran=$((ran + 1)); fi; done
  echo "running $ran tests"
  for name in "$@"; do
    [[ "$skips" != *" $name "* ]] || continue
    if [[ "$name" == "${FAKE_UNITS_RED:-}" ]]; then echo "test $name ... FAILED"; failed=$((failed + 1)); red=1; else echo "test $name ... ok"; fi
  done
  printf '\ntest result: %s. %d passed; %d failed; 0 ignored; 0 measured; %d filtered out; finished in 0.00s\n\n' "$( ((failed)) && echo FAILED || echo ok)" $((ran - failed)) "$failed" "$filtered"
}
((lib == 0)) || section "Running unittests src/lib.rs (target/debug/deps/alpha-0123456789abcdef)" tests::plain tests::uses_helper
[[ "$tests" != *" beta "* ]] || section "Running tests/beta.rs (target/debug/deps/beta-0123456789abcdef)" beta_case
((doc == 0)) || section "Doc-tests alpha"
exit $((red ? 101 : 0))
FAKE
  cat > "$units_bin/rustc" <<'FAKE'
#!/usr/bin/env bash
# 假 rustc（按条复用那几格）：-V、-vV 打版本与 host（admission.py 的单元键要工具链那几行），别的退 0
case "${1:-}" in
  -V) echo "rustc 0.0.0-merge-and-verify-selftest" ;;
  -vV) printf '%s\n' "rustc 0.0.0-merge-and-verify-selftest" "host: x86_64-unknown-linux-gnu" "release: 0.0.0" ;;
esac
exit 0
FAKE
  chmod +x "$units_bin/cargo" "$units_bin/rustc"
  units_tree() { cp -r -- "$units_template" "$scratch/$1" && git init -q "$scratch/$1"; }   # <格名>：拷一份样本树、git init
  units_case() {   # <样本树> <日志目录名> <参数…>：起一趟（参数后面接上 units_patch），输出进 $out、退出码进 rc、假 cargo 收到的参数进 $units_calls
    local units_tree_root="$1" units_logs_name="$2"
    shift 2
    PATH="$units_bin:$PATH" FAKE_CARGO_CALLS="$units_calls" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/$units_logs_name" \
      MERGE_AND_VERIFY_TREE="$units_tree_root" env -u CARGO_TARGET_DIR bash "$0" "$@" "$units_patch" > "$out" 2>&1; rc=$?
  }
  # 列项：--list-items 逐行打「项<TAB>挑目标的参数」、退 0，不编不跑（假 cargo 一次都没起）、不建日志目录
  units_tree units-list
  out="$scratch/out-units-list.txt"; units_calls="$scratch/calls-units-list.txt"; : > "$units_calls"; checked=$((checked+1))
  units_case "$scratch/units-list" logs-units-list --list-items
  if [[ $rc -ne 0 || -s "$units_calls" || -e "$scratch/logs-units-list" ]] || ! grep -qxF $'alpha:src/lib.rs\t--lib' "$out" || ! grep -qxF $'alpha:tests/beta.rs\t--test beta' "$out" \
     || ! grep -qxF $'alpha:doc\t--doc' "$out" || [[ "$(grep -c . "$out")" -ne 3 ]]; then
    echo "  ✗ 自检：逐项与复用：--list-items 逐行列这一趟的项（alpha:src/lib.rs、alpha:tests/beta.rs、alpha:doc 三行）、退 0、假 cargo 一次都没起、不建日志目录，实际退出码 $rc，假 cargo 收到 $(grep -c . "$units_calls") 条：$(tr '\n' '|' < "$out" | clip_text 300)"; failures=1; fi   # gate-lint:detail
  # 列项在打上这一批补丁的临时快照上（文件头「逐项点名与按条复用」--list-items）：补丁新建 alpha 的测试目标 tests/gamma.rs、变异行点名 --test gamma；
  #   打补丁换成真打进 --root 的假 apply（git apply），不带 --skip-apply：列得出 alpha:tests/gamma.rs、退 0，样本树本身没有 gamma.rs（一个字节不动），
  #   假 cargo 一次都没起、不建日志目录、TMPDIR 下不留列项的临时目录。没打补丁的树上 gamma 不在，变异行点名它判不出、整条退 3
  local new_target_patch="$scratch/units-new-target-patch" new_target_leftover
  units_tree units-list-new-target
  mkdir -p "$new_target_patch"
  printf 'diff --git a/crates/alpha/tests/gamma.rs b/crates/alpha/tests/gamma.rs\nnew file mode 100644\n--- /dev/null\n+++ b/crates/alpha/tests/gamma.rs\n@@ -0,0 +1,2 @@\n+#[test]\n+fn gamma_case() {}\n' \
    > "$new_target_patch/crates.patch"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'alpha 的 gamma（自证样本）' crates/alpha/tests/gamma.rs 'fn gamma_case() {}' 'fn gamma_case() { }' '-p alpha --test gamma -- gamma_case' gamma_case \
    > "$new_target_patch/mutations-append.tsv"
  cat > "$scratch/fake-apply-into-root.py" <<'FAKE'
import subprocess, sys
# 假 apply-writer-patch.py（列项在临时快照上那一格）：把 <补丁目录>/crates.patch 真打进 --root 那一棵（git apply），打得上退 0、打不上退 2
root = sys.argv[sys.argv.index("--root") + 1]
sys.exit(0 if subprocess.run(["git", "-C", root, "apply", sys.argv[1] + "/crates.patch"]).returncode == 0 else 2)
FAKE
  out="$scratch/out-units-list-new-target.txt"; units_calls="$scratch/calls-units-list-new-target.txt"; : > "$units_calls"; checked=$((checked+1))
  PATH="$units_bin:$PATH" FAKE_CARGO_CALLS="$units_calls" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply-into-root.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-units-list-new-target" \
    MERGE_AND_VERIFY_TREE="$scratch/units-list-new-target" env -u CARGO_TARGET_DIR bash "$0" --list-items "$new_target_patch" > "$out" 2>&1; rc=$?
  new_target_leftover="$(find "$TMPDIR" -maxdepth 1 -name 'merge-and-verify-list-items.*' | head -n 1)"
  if [[ $rc -ne 0 || -s "$units_calls" || -e "$scratch/logs-units-list-new-target" || -e "$scratch/units-list-new-target/crates/alpha/tests/gamma.rs" || -n "$new_target_leftover" ]] \
     || ! grep -qxF $'alpha:tests/gamma.rs\t--test gamma' "$out"; then
    echo "  ✗ 自检：逐项与复用：--list-items 应当在打上这一批补丁的临时快照上列（补丁新建的 alpha:tests/gamma.rs 列得出、退 0、样本树不动、不留临时目录），实际退出码 $rc，假 cargo 收到 $(grep -c . "$units_calls") 条，留下的临时目录「${new_target_leftover:-无}」：$(tr '\n' '|' < "$out" | clip_text 300)"; failures=1; fi   # gate-lint:detail
  # 项写错：退 2、列出这一趟可点的项、一条测试都不起
  out="$scratch/out-units-unknown.txt"; units_calls="$scratch/calls-units-unknown.txt"; : > "$units_calls"; checked=$((checked+1))
  units_case "$scratch/units-list" logs-units-unknown --skip-apply --item alpha:tests/missing.rs
  if [[ $rc -ne 2 ]] || grep -q '^test ' "$units_calls" || ! grep -qF -- '--item 点名的项不在这一趟可点的项里：alpha:tests/missing.rs' "$out" || ! grep -qxF '       alpha:tests/beta.rs' "$out"; then
    echo "  ✗ 自检：逐项与复用：--item 写错的项应当退 2、列出可点的项、一条测试都不起，实际退出码 $rc，假 cargo 收到的测试命令 $(grep -c '^test ' "$units_calls") 条：$(grep -E '✗|alpha:' "$out" | head -4 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 列项遇到落不到测试目标的（文件头「逐项点名与按条复用」--list-items）：补丁目录两行变异，一行点名 alpha 的测试目标 beta（落得到）、一行只点包 alpha（-p alpha，
  #   没点测试目标，落不到）；--skip-apply 照样本树此刻的样子挑：落得到的 alpha:tests/beta.rs 照列（项恰好这一行）、落不到的逐条报「✗ 判不出测试目标：…」、退 3，
  #   假 cargo 一次都没起、不建日志目录。弄坏开关 list-items-refusal-drops-items 下一项都不列，这一格红
  local partial_patch="$scratch/units-partial-patch"
  mkdir -p "$partial_patch"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'alpha 的 beta（自证样本）' crates/alpha/tests/beta.rs a b '-p alpha --test beta -- beta_case' beta_case \
    'alpha 整包（自证样本）' crates/alpha/src/lib.rs 1 2 '-p alpha' uses_helper > "$partial_patch/mutations-append.tsv"
  out="$scratch/out-units-list-partial.txt"; units_calls="$scratch/calls-units-list-partial.txt"; : > "$units_calls"; checked=$((checked+1))
  PATH="$units_bin:$PATH" FAKE_CARGO_CALLS="$units_calls" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-units-list-partial" \
    MERGE_AND_VERIFY_TREE="$scratch/units-list" env -u CARGO_TARGET_DIR bash "$0" --list-items --skip-apply "$partial_patch" > "$out" 2>&1; rc=$?
  if [[ $rc -ne 3 || -s "$units_calls" || -e "$scratch/logs-units-list-partial" ]] || ! grep -qxF $'alpha:tests/beta.rs\t--test beta' "$out" \
     || [[ "$(grep -c '^alpha:' "$out")" -ne 1 ]] || ! grep -qF '✗ 判不出测试目标：变异行「alpha 整包（自证样本）」' "$out" || ! grep -qF '只点了包 alpha、没点测试目标' "$out"; then
    echo "  ✗ 自检：逐项与复用：--list-items 有判不出测试目标的照列落得到的项（alpha:tests/beta.rs 一行）、逐条报判不出的（只点了包 alpha 那一行变异）、退 3、假 cargo 一次都没起、不建日志目录，实际退出码 $rc，假 cargo 收到 $(grep -c . "$units_calls") 条：$(tr '\n' '|' < "$out" | clip_text 300)"; failures=1; fi   # gate-lint:detail
  # 判不出的留给提交时整轮门禁（文件头「判不出的留给提交时整轮门禁」），点名的格都以「判不出的留给提交时整轮门禁：」起头。样本树 merge-tree、补丁两份：merge-head（落到 alpha 的库，--item alpha:src/lib.rs）与
  #   merge-untargeted（改不落在包里的 docs/merge-notes.txt，判不出）；不带 --skip-apply、留着快照：
  #   带选项、点了项（left）：退 0、假 cargo 只收到 alpha 的库那一条测试、逐条打「留给提交时整轮门禁」（汇总行之前一遍、之后一遍，共两遍）、汇总行写「有 1 处留给提交时整轮门禁」、
  #   不写测试那一步没跑与退 3、照常打合进主工作区的命令；不带选项（left-off）：照旧退 3、一条测试都不起、出路里有带这个选项重交那一句；
  #   带选项而没点项（left-noitem）：照旧退 3、一条测试都不起、出路里补一句不生效、不打「留给提交时整轮门禁」。
  #   弄坏开关 unresolved-option-ignored、unresolved-lines-dropped、unresolved-summary-dropped 红在第一格，unresolved-without-option 红在第二格，unresolved-without-item 红在第三格
  local left_summary
  fresh_merge_tree "$merge_tree"
  out="$scratch/out-merge-left.txt"; merge_case left --item alpha:src/lib.rs "$UNRESOLVED_OPTION" "$scratch/merge-head" "$scratch/merge-untargeted"; checked=$((checked+1))
  left_summary="$(grep '^  汇总：' "$out")"
  if [[ $rc -ne 0 || "$left_summary" != *'有 1 处留给提交时整轮门禁'* ]] || [[ "$(grep -cF '      留给提交时整轮门禁：补丁改到不落在任何包里的 docs/merge-notes.txt' "$out")" -ne 2 ]] \
     || grep -qF '测试那一步与验证包那一步没跑' "$out" || grep -qF '这一趟退 3' "$out" || ! grep -qF "apply-writer-patch.py $scratch/logs-merge-left/merge-patch" "$out" \
     || [[ "$(grep -c '^test ' "$scratch/calls-merge-left.txt" 2>/dev/null)" -ne 1 ]] || ! grep -qE '^test .* -p alpha --lib( |$)' "$scratch/calls-merge-left.txt" 2>/dev/null; then
    echo "  ✗ 自检：判不出的留给提交时整轮门禁：带 $UNRESOLVED_OPTION、点了 --item、有判不出测试目标的改动，应当只跑点名的项（alpha 的库）、逐条打「留给提交时整轮门禁」（汇总行前后各一遍）、汇总行写有 1 处留给提交时整轮门禁、打合进主工作区的命令、退 0，实际退出码 $rc，假 cargo 收到的测试命令 $(grep '^test ' "$scratch/calls-merge-left.txt" 2>/dev/null | tr '\n' '|')，汇总「$left_summary」，留给提交时的行 $(grep -cF '      留给提交时整轮门禁：' "$out") 条：$(grep -E '判不出|✗|退 3' "$out" | head -3 | tr '\n' ' ' | clip_text 300)"; failures=1; fi   # gate-lint:detail
  fresh_merge_tree "$merge_tree"
  out="$scratch/out-merge-left-off.txt"; merge_case left-off --item alpha:src/lib.rs "$scratch/merge-head" "$scratch/merge-untargeted"; checked=$((checked+1))
  if [[ $rc -ne 3 ]] || grep -q '^test ' "$scratch/calls-merge-left-off.txt" 2>/dev/null || grep -qF '留给提交时整轮门禁：' "$out" || ! grep -qF '测试那一步与验证包那一步没跑：1 处判不出改到哪个测试目标' "$out" \
     || ! grep -qF "加 $UNRESOLVED_OPTION 整条重交" "$out"; then
    echo "  ✗ 自检：判不出的留给提交时整轮门禁：不带 $UNRESOLVED_OPTION 时判不出测试目标的照旧退 3、测试那一步不跑、不打「留给提交时整轮门禁」、出路里写带这个选项重交，实际退出码 $rc，假 cargo 收到的测试命令 $(grep '^test ' "$scratch/calls-merge-left-off.txt" 2>/dev/null | tr '\n' '|')：$(grep -E '判不出|✗|退 3|重交' "$out" | head -3 | tr '\n' ' ' | clip_text 300)"; failures=1; fi   # gate-lint:detail
  fresh_merge_tree "$merge_tree"
  out="$scratch/out-merge-left-noitem.txt"; merge_case left-noitem "$UNRESOLVED_OPTION" "$scratch/merge-head" "$scratch/merge-untargeted"; checked=$((checked+1))
  if [[ $rc -ne 3 ]] || grep -q '^test ' "$scratch/calls-merge-left-noitem.txt" 2>/dev/null || grep -qF '留给提交时整轮门禁：' "$out" || ! grep -qF "带了 $UNRESOLVED_OPTION 但没点 --item，不生效" "$out"; then
    echo "  ✗ 自检：判不出的留给提交时整轮门禁：带了 $UNRESOLVED_OPTION 而没点 --item 时不生效、判不出测试目标的照旧退 3、测试那一步不跑、出路里补一句不生效，实际退出码 $rc，假 cargo 收到的测试命令 $(grep '^test ' "$scratch/calls-merge-left-noitem.txt" 2>/dev/null | tr '\n' '|')：$(grep -E '判不出|✗|不生效' "$out" | head -3 | tr '\n' ' ' | clip_text 300)"; failures=1; fi   # gate-lint:detail
  # 带选项、点了项的那一趟测试红（样本树 units-left-red，补丁 partial：alpha 的 beta 落得到、alpha 整包判不出，--skip-apply，FAKE_UNITS_RED=beta_case）：退 1、
  #   只重跑红的那一条合入后验证带上 --unresolved-to-commit-gate（不带的话判不出的那一处又退 3、点名的项一条不跑）。弄坏开关 unresolved-option-ignored、unresolved-option-dropped-from-rerun 红在这一格
  units_tree units-left-red; units_calls="$scratch/calls-units-left-red.txt"; : > "$units_calls"
  out="$scratch/out-units-left-red.txt"; checked=$((checked+1))
  PATH="$units_bin:$PATH" FAKE_CARGO_CALLS="$units_calls" FAKE_UNITS_RED=beta_case MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-units-left-red" \
    MERGE_AND_VERIFY_TREE="$scratch/units-left-red" env -u CARGO_TARGET_DIR bash "$0" --skip-apply --item alpha:tests/beta.rs "$UNRESOLVED_OPTION" "$partial_patch" > "$out" 2>&1; rc=$?
  if [[ $rc -ne 1 ]] || ! grep -qF "scripts/gate/merge-and-verify.sh --skip-apply $UNRESOLVED_OPTION --item alpha:tests/beta.rs $partial_patch" "$out"; then
    echo "  ✗ 自检：判不出的留给提交时整轮门禁：带 $UNRESOLVED_OPTION、点了项的那一趟测试红了应当退 1、只重跑红的那一条合入后验证带上 $UNRESOLVED_OPTION，实际退出码 $rc：$(grep -E '→ 怎么办|判不出|✗' "$out" | head -3 | tr '\n' ' ' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  # 复用三问（判法 scripts/gate/lib_reuse_checks.py three-questions）：同一份输入跑四趟——头一趟、同一份输入、改闭包外的 spare 之后、改闭包里的 helper 之后；
  #   每趟的逐条结局取 test-functions.tsv（名字与现判 / 复用），affected 是 uses_helper（只有它的闭包含 helper）。头两趟另核每项一行 ITEM_DONE
  units_tree units-reuse; units_calls="$scratch/calls-units-reuse.txt"; : > "$units_calls"
  local -a reuse_rounds_rc=()
  for units_round in first second unrelated unit-change; do
    case "$units_round" in
      unrelated) sed -i 's/^    7$/    8/' "$scratch/units-reuse/crates/alpha/src/lib.rs" ;;
      unit-change) sed -i 's/^    1$/    2/' "$scratch/units-reuse/crates/alpha/src/lib.rs" ;;
    esac
    out="$scratch/out-units-reuse-$units_round.txt"
    units_case "$scratch/units-reuse" logs-units-reuse --skip-apply
    reuse_rounds_rc+=("$rc")
    cut -f1,3 "$scratch/logs-units-reuse/test-functions.tsv" > "$scratch/units-reuse-$units_round.tsv" 2>/dev/null || : > "$scratch/units-reuse-$units_round.tsv"
  done
  affected_name="$(awk -F'\t' 'NF >= 7 && $6 == "tests::uses_helper" { print $1; exit }' "$scratch/logs-units-reuse/test-plan.alpha.out" 2>/dev/null)"
  checked=$((checked+1))
  reuse_check_rc=0
  reuse_check_output="$(python3 "$here/lib_reuse_checks.py" three-questions --script scripts/gate/merge-and-verify.sh --first "$scratch/units-reuse-first.tsv" \
    --second "$scratch/units-reuse-second.tsv" --unrelated "$scratch/units-reuse-unrelated.tsv" --unit-change "$scratch/units-reuse-unit-change.tsv" --affected "${affected_name:-tests::uses_helper}" 2>&1)" || reuse_check_rc=$?
  grep '^REUSE_SELFTEST ' <<<"$reuse_check_output"
  if [[ "${reuse_rounds_rc[*]}" != "0 0 0 0" || $reuse_check_rc -ne 0 || -z "$affected_name" ]]; then
    echo "  ✗ 自检：逐项与复用：reuse-three-questions（四趟退出码 ${reuse_rounds_rc[*]}、affected「${affected_name:-取不到}」；lib_reuse_checks.py three-questions 退 $reuse_check_rc：$(grep -E '✗|REUSE_SELFTEST' <<<"$reuse_check_output" | tr '\n' ' ' | clip_text 400)）"; failures=1; fi   # gate-lint:detail
  checked=$((checked+1))
  if ! grep -qxF 'ITEM_DONE item=alpha:src/lib.rs seconds=0 judged=2 reused=0 exit=0' "$scratch/out-units-reuse-first.txt" \
     || ! grep -qxF 'ITEM_DONE item=alpha:tests/beta.rs seconds=0 judged=1 reused=0 exit=0' "$scratch/out-units-reuse-first.txt" \
     || ! grep -qxF 'ITEM_DONE item=alpha:doc seconds=0 judged=1 reused=0 exit=0' "$scratch/out-units-reuse-first.txt" \
     || ! grep -qxF 'ITEM_DONE item=alpha:src/lib.rs seconds=0 judged=0 reused=2 exit=0' "$scratch/out-units-reuse-second.txt" \
     || ! grep -qF '按条复用：现判 0、复用 3、拒算 0' "$scratch/out-units-reuse-second.txt"; then
    echo "  ✗ 自检：逐项与复用：整趟每项一行 ITEM_DONE（头一趟三项各现判、第二趟库里那两条复用），成功句报「按条复用：现判、复用、拒算」，实际头一趟 $(grep '^ITEM_DONE ' "$scratch/out-units-reuse-first.txt" | tr '\n' '|')，第二趟 $(grep -E '^ITEM_DONE |按条复用' "$scratch/out-units-reuse-second.txt" | tr '\n' '|' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  # 点名单跑一项与整趟里同一项判定相同：两份新拷的样本树（各自一份库）一趟整趟、一趟只点 alpha:tests/beta.rs；点名那一趟只起 --test beta、不跑库与文档测试，
  #   它的 ITEM_DONE 与整趟里那一项的逐字相同，beta_case 的结局相同
  units_tree units-whole; units_tree units-named
  out="$scratch/out-units-whole.txt"; units_calls="$scratch/calls-units-whole.txt"; : > "$units_calls"
  units_case "$scratch/units-whole" logs-units-whole --skip-apply
  whole_done_line="$(grep '^ITEM_DONE item=alpha:tests/beta.rs ' "$out")"
  out="$scratch/out-units-named.txt"; units_calls="$scratch/calls-units-named.txt"; : > "$units_calls"; checked=$((checked+1))
  units_case "$scratch/units-named" logs-units-named --skip-apply --item alpha:tests/beta.rs
  named_done_line="$(grep '^ITEM_DONE item=alpha:tests/beta.rs ' "$out")"
  if [[ $rc -ne 0 || -z "$named_done_line" || "$named_done_line" != "$whole_done_line" ]] || [[ "$(grep -c '^test ' "$units_calls")" -ne 1 ]] \
     || ! grep -qE '^test --offline --no-fail-fast -p alpha --test beta( |$)' "$units_calls" || grep -q '^ITEM_DONE item=alpha:src/lib.rs ' "$out" \
     || [[ "$(awk -F'\t' '$2 == "alpha:tests/beta.rs" { print $3, $4 }' "$scratch/logs-units-named/test-functions.tsv" 2>/dev/null)" \
           != "$(awk -F'\t' '$2 == "alpha:tests/beta.rs" { print $3, $4 }' "$scratch/logs-units-whole/test-functions.tsv" 2>/dev/null)" ]]; then
    echo "  ✗ 自检：逐项与复用：点名单跑 alpha:tests/beta.rs 应当只起 --test beta、它的 ITEM_DONE 与整趟里那一项相同，实际退出码 $rc，假 cargo 收到的测试命令 $(grep '^test ' "$units_calls" | tr '\n' '|')，点名「$named_done_line」、整趟「$whole_done_line」"; failures=1; fi   # gate-lint:detail
  # 记红撤绿：FAKE_UNITS_RED=tests::uses_helper 那一趟红（退 1、出路点名红的项 alpha:src/lib.rs），下一趟（假 cargo 不再打 FAILED、源码不动）
  #   uses_helper 照计划现判、原因以「上次判红」起头，结局绿
  units_tree units-red; units_calls="$scratch/calls-units-red.txt"; : > "$units_calls"
  out="$scratch/out-units-red.txt"; checked=$((checked+1))
  FAKE_UNITS_RED=tests::uses_helper units_case "$scratch/units-red" logs-units-red --skip-apply
  local red_round_rc="$rc"
  out="$scratch/out-units-red-next.txt"
  units_case "$scratch/units-red" logs-units-red --skip-apply
  if [[ "$red_round_rc" -ne 1 || $rc -ne 0 ]] || ! grep -qF "rerun-failed-tests.py $scratch/logs-units-red/test.log --item alpha:src/lib.rs" "$scratch/out-units-red.txt" \
     || ! awk -F'\t' 'NF >= 7 && $6 == "tests::uses_helper" && $2 == "judge" && index($7, "上次判红") == 1 { found = 1 } END { exit found ? 0 : 1 }' "$scratch/logs-units-red/test-plan.alpha.out" 2>/dev/null \
     || [[ "$(awk -F'\t' '$1 ~ /uses_helper$/ { print $3, $4 }' "$scratch/logs-units-red/test-functions.tsv" 2>/dev/null)" != "现判 绿" ]]; then
    echo "  ✗ 自检：逐项与复用：记红撤绿：上一趟判红的 uses_helper 下一趟应当照计划现判（原因「上次判红」）、判绿，红的那一趟退 1、出路点名 --item alpha:src/lib.rs，实际两趟退出码 $red_round_rc $rc，下一趟计划里那一条「$(awk -F'\t' 'NF >= 7 && $6 == "tests::uses_helper"' "$scratch/logs-units-red/test-plan.alpha.out" 2>/dev/null | cut -f2,7 | tr '\t' ' ')」；三个条件各自：红的那一趟输出里出路行$(grep -qF "rerun-failed-tests.py $scratch/logs-units-red/test.log --item alpha:src/lib.rs" "$scratch/out-units-red.txt" && echo 在 || echo 不在)、计划里 uses_helper 的原因$(awk -F'\t' 'NF >= 7 && $6 == "tests::uses_helper" && $2 == "judge" && index($7, "上次判红") == 1 { found = 1 } END { exit found ? 0 : 1 }' "$scratch/logs-units-red/test-plan.alpha.out" 2>/dev/null && echo 对 || echo 不对)、test-functions.tsv 里 uses_helper「$(awk -F'\t' '$1 ~ /uses_helper$/ { print $3, $4 }' "$scratch/logs-units-red/test-functions.tsv" 2>/dev/null | tr '\n' '|')」；红的那一趟输出里「$(grep -E 'rerun-failed-tests|FAILED|✗' "$scratch/out-units-red.txt" | head -3 | clip_text 200 | tr '\n' '|')」；日志目录里有 $(ls "$scratch/logs-units-red" 2>/dev/null | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 先验后合（文件头「先验后合」）：不带 --skip-apply 的那一趟测试那一步红（新拷一份样本树，FAKE_UNITS_RED=tests::uses_helper）：退 1，只重跑红的那几项是不带 --skip-apply 的合入后验证
  #   （--item alpha:src/lib.rs 与同一批补丁目录），不打 rerun-failed-tests.py；带 --skip-apply 的那一趟（记红撤绿那一格的红的那一趟）照旧带 --skip-apply
  units_tree units-merge-red; units_calls="$scratch/calls-units-merge-red.txt"; : > "$units_calls"
  out="$scratch/out-units-merge-red.txt"; checked=$((checked+1))
  FAKE_UNITS_RED=tests::uses_helper units_case "$scratch/units-merge-red" logs-units-merge-red
  if [[ $rc -ne 1 ]] || ! grep -qF "scripts/gate/merge-and-verify.sh --item alpha:src/lib.rs $units_patch" "$out" \
     || ! grep -qF "跑什么=合入后验证点名的 alpha:src/lib.rs; 范围=1 项;" "$out" || grep -qF 'merge-and-verify.sh --skip-apply' "$out" || grep -qF 'python3 scripts/gate/rerun-failed-tests.py' "$out" \
     || ! grep -qF "scripts/gate/merge-and-verify.sh --skip-apply --item alpha:src/lib.rs $units_patch" "$scratch/out-units-red.txt"; then
    echo "  ✗ 自检：先验后合：测试那一步红了，只重跑红的那几项应当是不带 --skip-apply 的合入后验证（--item alpha:src/lib.rs、同一批补丁目录，不打 rerun-failed-tests.py；带 --skip-apply 的那一趟照旧带 --skip-apply），实际退出码 $rc：$(grep -E '→ 怎么办' "$out" | head -1 | clip_text 400)"; failures=1; fi   # gate-lint:detail
  # 先验后合：测试那一步红了，只重跑的那一条在 submit -- 之后带 env MERGE_AND_VERIFY_KEEP_COPY=1、点了项的末尾带这一趟的日志目录（文件头「开跑那一刻的指纹」--fingerprint-of 那一条）
  checked=$((checked+1))
  if ! grep -qF "submit -- env MERGE_AND_VERIFY_KEEP_COPY=1 bash $root/scripts/gate/merge-and-verify.sh --item alpha:src/lib.rs $units_patch --fingerprint-of $scratch/logs-units-merge-red" "$scratch/out-units-merge-red.txt"; then
    echo "  ✗ 自检：先验后合：测试那一步红了，只重跑的那一条应当带 env MERGE_AND_VERIFY_KEEP_COPY=1 与 --fingerprint-of <这一趟的日志目录>，实际：$(grep -E '→ 怎么办' "$scratch/out-units-merge-red.txt" | head -1 | clip_text 400)"; failures=1; fi   # gate-lint:detail
  # 每项一行 ITEM_PLAN：整趟一次（留着快照，MERGE_AND_VERIFY_KEEP_COPY=1）记进库，再对判的那棵树 --list-items --with-keys --skip-apply：库与文档测试各项照计划报
  #   （没有快照时就是要验的树本身；带 --skip-apply 在那棵树本身上算键，不另拷临时快照：临时服务按库目录分命名空间，另拷的那一份读不到这一趟记的库）
  units_tree units-keys; units_calls="$scratch/calls-units-keys.txt"; : > "$units_calls"
  out="$scratch/out-units-keys.txt"; checked=$((checked+1))
  MERGE_AND_VERIFY_KEEP_COPY=1 units_case "$scratch/units-keys" logs-units-keys --skip-apply
  local keys_round_rc="$rc"
  keys_tree="$scratch/logs-units-keys/repo"; [[ -d "$keys_tree" ]] || keys_tree="$scratch/units-keys"
  out="$scratch/out-units-keys-list.txt"
  units_case "$keys_tree" logs-units-keys-list --list-items --with-keys --skip-apply
  if [[ "$keys_round_rc" -ne 0 || $rc -ne 0 ]] || ! grep -qxF 'ITEM_PLAN item=alpha:src/lib.rs to_judge=0 reusable=2' "$out" \
     || ! grep -qxF 'ITEM_PLAN item=alpha:tests/beta.rs to_judge=0 reusable=1' "$out" || ! grep -qxF 'ITEM_PLAN item=alpha:doc to_judge=1 reusable=0' "$out"; then
    echo "  ✗ 自检：逐项与复用：--list-items --with-keys 每项一行 ITEM_PLAN（记过库之后 alpha:src/lib.rs 0 与 2、alpha:tests/beta.rs 0 与 1、alpha:doc 1 与 0），实际两趟退出码 $keys_round_rc $rc：$(grep -E '^ITEM_PLAN|✗|算不出' "$out" | tr '\n' '|' | clip_text 400)"; failures=1; fi   # gate-lint:detail
  # 按条复用失灵：样本树的 .unit-verdict-sample/plan-failure 写 exit-1（admission.py test-plan 不算键、退 1）：测试那一步照旧全部现判跑完、判红退 1
  units_tree units-plan-failure; printf 'exit-1\n' > "$scratch/units-plan-failure/.unit-verdict-sample/plan-failure"
  units_calls="$scratch/calls-units-plan-failure.txt"; : > "$units_calls"
  out="$scratch/out-units-plan-failure.txt"; checked=$((checked+1))
  units_case "$scratch/units-plan-failure" logs-units-plan-failure --skip-apply
  if [[ $rc -ne 1 ]] || ! grep -qF '✗ 测试那一步按条复用失灵' "$out" || ! grep -qE '^test --offline --no-fail-fast -p alpha( |$)' "$units_calls" || grep -q -- '--skip' "$units_calls"; then
    echo "  ✗ 自检：逐项与复用：按条复用失灵（test-plan 退 1）应当判红、退 1，测试照旧全部现判跑完（不带 --skip），实际退出码 $rc，假 cargo 收到的测试命令 $(grep '^test ' "$units_calls" | tr '\n' '|')：$(grep -E '✗' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # prune-server 不可用：plan-failure 写 exit-75（test-plan 不连服务、退 75）：测试那一步不起 cargo、报不可用，整条退 75
  units_tree units-unavailable; printf 'exit-75\n' > "$scratch/units-unavailable/.unit-verdict-sample/plan-failure"
  units_calls="$scratch/calls-units-unavailable.txt"; : > "$units_calls"
  out="$scratch/out-units-unavailable.txt"; checked=$((checked+1))
  units_case "$scratch/units-unavailable" logs-units-unavailable --skip-apply
  if [[ $rc -ne 75 ]] || grep -q '^test ' "$units_calls" || ! grep -qF '测试本次未跑：按条复用那一层连不上 prune-server' "$out"; then
    echo "  ✗ 自检：逐项与复用：prune-server 不可用（test-plan 退 75）应当退 75、测试那一步不起 cargo，实际退出码 $rc，假 cargo 收到的测试命令 $(grep -c '^test ' "$units_calls") 条：$(grep -E '✗|测试' "$out" | head -3 | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 重跑命令点名（文件头「只重跑的命令点名」）：上面几格打出的每一条带范围声明的命令（「SINGLEFS_RUN_SCOPE='…' <命令>」，命令到全角括号或行尾为止）逐条拆开，
  #   照服务提交时那一判（task-server/task_server_lib.py 的 unnamed_run_problem，经 .claude/hooks/lib_project_code_runs.py 的 task_server_library 载入，不另写一份）判：
  #   submit 与 -- 之间带 --full-gate、--whole-package-confirmed 的照交给判据，--cwd 照交，假的那一道的路径换成真门禁的路径再判。每一条都要判成点了名，至少判到一条；
  #   另核：原命令点了 --item 的 --skip-apply 核不过那一格打了再验的那一条的，那一条带同一个 --item；打出的命令不经 task-server/task submit 的算拆不开、判红
  checked=$((checked+1))
  local rerun_judged rerun_judge_rc=0 rerun_unnamed rerun_total
  rerun_judged="$(python3 - "$root/.claude/hooks" "$scratch/fake-verification-step.sh" "$root/$VERIFICATION_STAGE_PATH" "$root" \
    "$scratch/out1.txt" "$scratch/out-units-merge-red.txt" "$scratch/out-units-red.txt" "$scratch/out-verification-red.txt" "$scratch/out-verification-red-unknown-item.txt" \
    "$scratch/out-verification-scaled-red.txt" "$scratch/out-verification-merge-red.txt" "$scratch/out-verification-merge-red-gpu.txt" "$scratch/out-verification-product-gpu-red.txt" \
    "$scratch/out-narrow-unapplied.txt" "$scratch/out-narrow-unapplied-named.txt" "$scratch/out-units-left-red.txt" <<'PY_RERUN_NAMED' 2>&1
import os, re, shlex, sys
# 逐条打「<文件名><TAB>named|unnamed|unparsed<TAB><命令>」，末行「total<TAB><条数>」；判据载不进打「library<TAB><原因>」退 1
hooks, fake_stage, real_stage, root = sys.argv[1:5]
sys.path.insert(0, hooks)
import lib_project_code_runs
library, problem = lib_project_code_runs.task_server_library()
if library is None:
    print(f"library\t{problem}")
    sys.exit(1)
scoped = re.compile(r"SINGLEFS_RUN_SCOPE='[^']*' ([^（\n]+)")
total = 0
for path in sys.argv[5:]:
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError:
        continue
    for found in scoped.finditer(text):
        command = found.group(1).strip()
        total += 1
        words = shlex.split(command)
        if len(words) < 3 or os.path.basename(words[0]) != "task" or words[1] != "submit" or "--" not in words:
            print(f"{os.path.basename(path)}\tunparsed\t{command[:300]}")
            continue
        options, argv = words[2:words.index("--")], words[words.index("--") + 1:]
        cwd = options[options.index("--cwd") + 1] if "--cwd" in options else root
        argv = [real_stage if word == fake_stage else word for word in argv]
        verdict = library.unnamed_run_problem(argv, cwd, "--full-gate" in options, "--whole-package-confirmed" in options)
        print(f"{os.path.basename(path)}\t{'named' if verdict is None else 'unnamed'}\t{command[:300]}")
print(f"total\t{total}")
PY_RERUN_NAMED
)" || rerun_judge_rc=$?
  rerun_unnamed="$(grep -cvE $'\tnamed\t|^total\t' <<<"$rerun_judged")"
  rerun_total="$(sed -n 's/^total\t//p' <<<"$rerun_judged")"
  if ((rerun_judge_rc != 0 || rerun_unnamed != 0)) || [[ ! "$rerun_total" =~ ^[1-9][0-9]*$ ]] \
     || { grep -qF "$scratch/logs-narrow-unapplied-named/current-tree-patch" "$scratch/out-narrow-unapplied-named.txt" \
          && ! grep -qF -- "--skip-apply $scratch/logs-narrow-unapplied-named/current-tree-patch --item alpha:src/lib.rs" "$scratch/out-narrow-unapplied-named.txt"; }; then
    echo "  ✗ 自检：重跑命令点名：打出的每条只重跑的命令照 unnamed_run_problem 判都点了名（至少一条；原命令点了 --item 的 --skip-apply 核不过那一格再验的那一条带同一个 --item），实际判法退 $rerun_judge_rc、判了 ${rerun_total:-0} 条、没点名或拆不开的 $rerun_unnamed 条：$(grep -vE $'\tnamed\t|^total\t' <<<"$rerun_judged" | head -n 3 | tr '\n' '|' | clip_text 600)"; failures=1   # gate-lint:detail
  else
    echo "  · 重跑命令点名：判了 $rerun_total 条，都点了名"
  fi
  rm -rf "${scratch:?}"
  if [[ $failures -ne 0 ]]; then
    echo "    → 怎么办：看 verify() 抓 FAILED 行与退出码、拷快照（take_snapshot）、挑槽与拷进槽（take_slot、refresh_slot_snapshot）与收尾（finish）那几处；MERGE_AND_VERIFY_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ 自检通过（查了 $checked 种）：测试红时退 1 并列出红的与重跑命令、补丁核不过整批停退 2、全绿退 0、没设日志目录时日志落在工作根下、"
  echo "    往编过的副本里 rsync -a 铺旧 mtime 的新内容之后照样重编、给了认不出的选项退 2 一样都不做、"
  echo "    clippy 照 check.sh 的 ${#expected_combinations[@]} 个特性组合（${expected_combinations[*]-}）逐个跑、每个带上 check.sh 的 $expected_lint_count 条编码纪律 lint、一个组合红整步红并点名那个组合、"
  echo "    build 只点补丁改到的包与传递依赖它们的包（不点只被依赖的、一个包都没有时报本次未跑）、build 那一条的环境里关掉调试信息（CARGO_PROFILE_DEV_DEBUG=0，test、fmt、clippy 不设）、"
  echo "    cargo test 只起补丁改到的 demo 的库目标、测试那一步经 test-targets-parallel.py 按测试目标并行起、"
  echo "    四条 cargo 命令从 check.sh 现读（只换包装 run_cargo 与 cargo 来回换、各加一个标志都照读到的写法跑）、"
  echo "    check.sh 读不出 lint 表、clippy 那一行没带 lint 表、fmt 那一条换成认不出的包装、test 那一行带收窄目标的标志、缺省特性组合只剩一个、循环改成只取一个组合都退 1 不编、要验的树里有特性组合登记退 1 不编、"
  echo "    成员里没有 验证包退 1 不编、一条测试都没跑退 1、"
  echo "    开跑时拷快照、在快照里打补丁与编、快照根 git init、快照里的修改时刻是拷的那一刻、全绿与测试红收尾都删掉快照与编译目录、MERGE_AND_VERIFY_KEEP_COPY=1 留着并打出路径、留着快照的那一趟在日志目录写归属记录 .owner（不留的不写）、"
  echo "    上一次尝试留下的快照：环境里有任务号、尝试号大于 1 时，开跑删掉同一任务尝试号更小的日志目录里留着的快照（带 MERGE_AND_VERIFY_KEEP_COPY=1 留的也删、日志目录留着）、别的任务号与尝试号不小于这一趟的与没有任务号的不碰、有进程开着里面的文件不删并写明开着的进程、开跑时在日志目录记下任务号与尝试号、"
  echo "    跑着的时候主工作区里那份源码被改、测试与 验证包那一步的键照开跑时那一份算、日志目录在要验的树里面退 2 不拷、"
  echo "    补丁改过的包头一次编与再验同一批补丁都有 Compiling 行、测试那一步不编的包靠 build 的 Compiling 行与 clippy 的 Checking 行核重编、编译目录里的产物比源码新时点名没重编的包退 1、"
  echo "    只跑补丁改到的与变异行点名的测试目标（不带依赖它们的包、不跑包里没点到的目标）、判不出测试目标的四种（没给补丁目录、改 Cargo.lock、改不落在包里的路径、改到落不到目标的文件）测试不跑退 3、改到被 #[path] 引的模块文件时落到引它的每个目标、tests/ 子目录里的文件只落到 mod / #[path] 走得到它的目标（只被两个目标经 #[path] 引的只挑那两个、经 mod.rs 被引的挑 mod 这个子目录的）而谁都走不到才退回整目录并打说明、不变量文档没有代码读就不跑不退 3、判不出测试目标的出路写明点哪几个包（没点名包的几种写明范围交用户定）、有测试目标读就只跑它、只有 build.rs 读退 3、--all-packages 退 2、"
  echo "    包的 Cargo.toml 里 [[bin]] 段有差别（新加、改了）与补丁只改 bin 的根文件时都只挑那个 bin、不另挑同包的集成测试，新加的 [[test]] 段与自动认的测试目标同名把它顶掉时判不出退 3 并点名被顶掉的目标、"
  echo "    段写的 path 先归一（./tests/case_a.rs 就是自动认的 tests/case_a.rs、不算顶掉）、没写 path 的段落到目录形根文件 tests/<名>/main.rs 与 src/bin/<名>/main.rs、只改那份根文件的正文也挑出那个目标、"
  echo "    没写 edition 的包（cargo 按 2015）新加一段 [[test]] 时自动认的测试目标被顶掉、判不出退 3 并写明 edition 2015、"
  echo "    build 与开跑那一刻的指纹算包都认 { workspace = true } 的依赖与树根 [patch.<源>] 里写了 path 的项、"
  echo "    --skip-apply 时补丁反着核得过才往下验、没打在树上的退 1 不编、"
  echo "    先验后合：不带 --skip-apply、留着快照的那一趟验绿或退 3 打第 2 步在快照上起的几条与合进主工作区的整条命令（先核主工作区没变、核要换掉的变异行，再一条 apply-writer-patch.py 合几份补丁合起来的那一份补丁目录），没留快照的与带 --skip-apply 的不打，合起来的那一份补丁目录是几份补丁照原样算出来的、打得上没动过的树，核主工作区没变的那一条主工作区没动退 0、同包里与依赖关系上的包改了或加了退非 0、要换掉的变异行被改了退非 0，改到 verus-pins.tsv 登记的必红世界表的那一趟不落测试目标、不退 3、第 2 步另打一条点那几行的 formal-verification（没登记的 .tsv 照旧判不出退 3），--fingerprint-of 的重跑核到那一趟开跑以来变了退 2，测试与验证包那一步红了只重跑的是不带 --skip-apply、留快照、带 --fingerprint-of 的合入后验证，登记表不落测试目标不退 3，打补丁核不过的出路写段树那一句（建新段树那一条带门禁表与三条 --with；核不过的是只含拒算表的补丁目录时出路写主 agent 从主工作区此刻重做那一份），合进主工作区的补丁目录里大小与修改时刻都相同的改动不被 git 的 stat 缓存漏掉、第 2 步打出的变异行名是照补丁次序合完之后还在的行、"
  echo "    跑完运行目录里不剩它建的编译目录与快照（全绿、测试红、被 TERM 停三种结局都删，调用方设的 CARGO_TARGET_DIR 不删）、编译目录里还有进程开着文件时不删并报出那个进程、"
  echo "    收尾只在起这一趟的那个 shell 里做（fork 出来的子 shell 跑了继承来的 EXIT trap 不删这一趟的快照与编译目录）、"
  echo "    编译目录建在 TMPDIR 下这一趟新建的一份、不在工作根下（日志照旧在工作根下）、收尾时前面一步写满盘编译目录照删（EXIT trap 第一步就删它）、"
  echo "    槽位：登记写坏退 1 不建日志目录、占槽与放槽（快照在槽里、编译目录在这一台的编译根下）、留着的槽不给别的趟、没空槽时照一趟一份、放了槽再接着用、"
  echo "    同一槽下一趟只重编改过的包、并发两趟各占一槽、源树里改了删了的在槽里照改照删且改过的包必重编、槽的编译目录里产物比改过的源码新时判红、"
  echo "    编译目录超上限就清、槽没了的编译目录跟着删、撞上限重来的尝试接着用上一次尝试的那一槽、调低上限之后多出来的槽不占（三个挑槽进程只一个占到）、"
  echo "    slots 是 0 时清多出来的槽与它的编译目录而留着的槽不动、拷进槽之前删掉槽里上一趟留下的编译产物、放槽那一条带 repo/target、"
  echo "    不给自证的登记与编译根时只给另一个工作根的不占也不清（登记三槽与 0 槽，缺省工作根下各槽的编译目录都在）、工作根就是这一台缺省的那一个（经符号链接给的也算）照常占槽、"
  echo "    连同 SINGLEFS_DATA_ROOT 一起换了工作根的不占也不清、只换 SINGLEFS_DATA_ROOT 的照常占槽（缺省的工作根照多机配置的 SHARED_DATA_ROOT 取）、"
  echo "    三槽都留着时槽位汇总写不占槽并逐个列留着的槽、第二台那一层给了编译目录时不占槽并照实说是第二台那一层给的、"
  echo "    占到槽的那一趟每条 cargo 收到 CARGO_INCREMENTAL=0、编之前删掉这一槽编译目录里的 incremental 而别的产物照留、不占槽的那一趟不设、调用方设了的照它不盖也不删缓存、"
  echo "    验证包那一步只把跑到的验证包目标里登记的崩溃枚举用例交按块那一路、挂特性的文件交 feature-gated 点名、产品 GPU 包的真卡对拍交那一格，不起整格、红了退 1 并只重跑红的那一项（经 task-server/task submit 交）、"
  echo "    跑到的目标里没有要另走门禁的就不起那一道、那一道退 3 只在样本根上算过、--list-items 退非 0 判红、跑在第二台上时照起那一道、"
  echo "    给了 --crash-expand-up-to（或环境里已设展开深度变量）把放量与对拍两个展开深度变量交给 验证包那一步与验证包目标的 cargo test、算键的请求、汇总写缩过域、不算门禁通过，缩过域的那一趟红了出路里只重跑的命令带同一个展开深度的两个变量，值不是非负整数退 2 一样都不做、"
  echo "    没给 --crash-expand-up-to 时崩溃域不超上限照整个域跑、算得出 n 用这个 n 接着跑、估不出或算不出 n 缩到 $CRASH_DOMAIN_FALLBACK_DEPTH 接着跑并打这个值的来历，缩了的汇总都写缩过域、给了值照给的不估、估算库起不来判红、"
  echo "    测试那一步的 cargo test 在 -- 之后什么都不带（标 #[ignore] 的硬件与量数用例不起）、成功句不报标 #[ignore] 的条数、"
  echo "    逐项与复用：--list-items 列项不编不跑、在打上这一批补丁的临时快照上列（补丁新建的测试目标列得出）、有判不出测试目标的照列落得到的项并逐条报判不出的、退 3、--item 写错退 2、点名单跑一项与整趟里同一项判定相同、每条测试函数按单元键复用（reuse-three-questions 三问都过）、每项一行 ITEM_DONE 与 ITEM_PLAN、"
  echo "    打出的每条只重跑的命令照 unnamed_run_problem 判都点了名（整批再验点这一趟跑过的项、红的项认不出点这一次的全部项、真卡对拍那一格不点项而带 --whole-package-confirmed）、"
  echo "    判红的照同一把键记红下一趟现判、按条复用失灵判红、prune-server 不可用退 75 不起 cargo、"
  echo "    判不出的留给提交时整轮门禁：带 $UNRESOLVED_OPTION 又点了 --item 的那一趟，判不出测试目标的几处逐条打「留给提交时整轮门禁」、点名的项照跑、汇总写有几处留给提交时整轮门禁、打合进主工作区的命令、退 0，测试红了只重跑的命令带这个选项；不带选项、带了而没点 --item 的照旧退 3、"
  echo "    变异锚点核：这一批补丁弄断别批变异行的锚点判红、点名行名与文件与命中次数、不打合进主工作区的命令，补丁之前就失效的变异行不算这一批弄断的、只报计数、"
  echo "    显卡：这一层不领卡、不挑卡，环境里没有卡时交给验证包那一步起的门禁每一次都不带卡串，环境里交的卡照原样交、"
  echo "    子进程跑超过一个进度间隔时标准输出有进度行（测试每一条与验证包那一步起的门禁每一次，间隔缩到 1 秒演）"
  return 0
}
