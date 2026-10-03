#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓、此刻的 Verus 安装与 cargo 缓存，上一次的结论不替这一次作保
# run-condition: none 要的 Verus、cargo 与装置由各格的判法自己判，缺了判红；核还没进仓时 Verus 那四格报本次未跑
# gate-stage: 形式化验证（Verus 证明恢复重放取前缀的核，版本钉住、空真与必红世界；E165 全量的红例钉进快速档）
# gate-category: checker-tier 类
# gate-similar: checker-tier-lkmm.sh 同是起外部验证器的一道，接法照抄它（判法放脚本、按 stage-inputs.tsv 复用、environment= 记工具版本）；它起 herd7 判 litmus，这里起 Verus 判 crates/ 的证明，并进去会让两件不相干的事共用一个复用判定
# gate-similar: checker-tier-crates-mutation-replay.sh 它逐条改坏 crates/ 要求点名的测试红；这里的必红世界要求 Verus 报证明错，写不进它的 crates/mutations.tsv
# gate-similar: code-source-discipline.sh 它的 mutation-tables 格静态判实验二进制的变异表成形、锚点唯一；这里判 E165 红例账与钉住的表、并逐行在今天的实现上重放
# gate-similar: checker-tier-research-build-and-replay.sh 它的 experiment-replay 格整份复跑入库实验比产物；这里只逐行重放钉住的红例（快速档）
# gate-similar: checker-tier-layer0-replay.sh 它判崩溃点重放与登记的崩溃枚举用例；这里判证明与有界穷举的红例，不起崩溃枚举
#
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就六格按下面的次序全跑，格名写错退 2）：
# gate-cell: verus-toolchain-pinned Verus 安装的 version 与 commit、Cargo.lock 里六个包的版本、本机 cargo 缓存里包源码的出处与内容指纹都与登记相同
# gate-cell: verus-proof Verus 带 --no-cheating 验整个 crate，限时内出结局、errors 0、verified 等于登记的数，核里没有作弊标记
# gate-cell: verus-vacuity 核里每一处 requires 的函数体头插 assert(false) 都验不过（前置条件可满足，不空真）
# gate-cell: verus-must-red-worlds 必红世界表每行改坏核之后 Verus 都报错、证明错条数等于第五段的 proof_errors=，原文恰好命中一次
# gate-cell: pinned-red-examples-complete 登记的全部 E165 红例账每一行都钉在 e165-pinned-red-examples.tsv 里，钉着的那一行第 9 段起与红例账原行逐字节相同
# gate-cell: pinned-red-examples-replay 钉住的每一行是 21 段整行抄、抄来的账行与自己的段对得上，点名的变异还在它的表里，装置在今天的实现上逐行打的结果与原实现期望的逐态行逐字节相同
# 参数解析、每格一个子 shell、汇总与退出码经共用库 lib/stage-cells.sh。
#
# 判法都在两份脚本里（判据、默认值、弄坏开关写在各自文件头），这一道只接进门禁、给样本开关：
#   Verus 四格：research/scripts/verus-proof-check.py <toolchain|proof|vacuity|must-red-worlds> <根>，登记表 .claude/gate.d/verus-pins.tsv。
#     核没进仓（登记的核不在、crate 不依赖 vstd、src 里没有 verus!）时四格都退 77；核不在而 crate 已经接了 Verus 判红。
#     读的环境变量：GATE_VERUS_TIMEOUT_SECONDS（限时，默认 600 秒，超时判红：C5（Z3 超时当通过））、GATE_VERUS_MEMORY_MAX（默认 4G）、
#     GATE_VERUS_JOBS（几处同时验）、SINGLEFS_THREAD_CAP（线程）、SINGLEFS_VERUS_ROOT（Verus 安装目录，盖过登记表）。
#   排不上只等一次：这一道给各格一份空文件（VERUS_PROOF_CHECK_QUEUE_REFUSED_FILE）。Verus 那几格里任何一处第一次拿到内存包装的 252（排不上），
#     判法往文件里写一行「哪一处<TAB>出路」，这一格与后面几格还没起的 Verus 都不再排队（已经起了的照常收），pinned-red-examples-replay 格读到它就不编装置、不起重放，
#     各报「没跑成：内存包装排不上，没再排队」，各格的汇总之后另打一行「这一道没跑成：内存包装排不上」；出路照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段。
#     所以整道最坏只等一次等待上限（RUN_WITH_MEMORY_CAP_WAIT_SECONDS，带 SINGLEFS_HEAVY_TESTS 时默认 3600 秒），不是每处、每格各等一次。
#   红例两格：research/scripts/e165-pin-red-examples.py --check / --replay，表的格式写在 .claude/gate.d/e165-pinned-red-examples.tsv 表头；
#     --replay 编装置的内存上限读 GATE_E165_PINNED_MEMORY_MAX（默认 2G，由那份脚本读）。
#     装置 e165_replay_bounded_enumeration 还不认 --pinned-cases 时，replay 格判完「变异还在」那一半就退 77。
#
# 样本（fixtures/checker-tier-formal-verification.sh/<格名>-<情形>/，各放 .gate-cells 点名一格；共享门禁的 stage-selftest.sh 跑）。名字读不出判法的几份：
#   verus-toolchain-pinned-*：假的 Verus 安装与假的 cargo 缓存放在样本的 .formal-sample/ 下
#   verus-proof-timeout-red：rlimit 设成无穷、样本限时 20 秒；verus-proof-killed-red：替身 Verus 起来就 kill -9 自己，判红，✗ 写「Verus 被信号停了」；
#   pinned-red-examples-replay-killed-red：桩装置打一行就 kill -9 自己，判红，✗ 写「装置被信号停了」；这两份另核汇总出路里「被信号停了」那一句（不改证明或实现）
#   verus-proof-kernel-absent（核没进仓）、pinned-red-examples-replay-device-absent（没有桩、装置源码不认 --pinned-cases）：退 77
#   verus-vacuity-closure-red：外层函数的前置条件可满足、里面闭包的不可满足
#   verus-must-red-worlds-：-rlimit-red 改坏之后只剩 rlimit 用尽；-rlimit-with-proof-error-green 另有一条证明错（proof_errors=1）；
#     -legacy-expectation-red 第五段旧写法「verified, 1 errors」；-proof-error-count-red 一条证明错、第五段写 proof_errors=2
#   pinned-red-examples-complete-whole-row-red：⑲ 与账的第 11 段不同，别的段都对；-replay-copied-columns-red：⑳ 与 ⑦ 不同；-replay-old-row-red：8 段的旧行
#   Verus 那几份样本起的是真的 Verus（SINGLEFS_VERUS_ROOT 或登记表 release 行的安装目录），没装就判红，与真仓一样；verus-proof-killed-red 起替身
#   （设了 SINGLEFS_VERUS_ROOT 时它盖过登记表、起的是真 Verus，这一份判错）。
# 只供样本的开关（被判的根不是这道阶段所在的仓、也没设 GATE_STAGED_FROM 时才认；真仓与 --staged 下放了也不认）：
#   .formal-sample/verus-install/         版本一致格改读这里的 version.json（不起它）
#   .formal-sample/cargo-home/            版本一致格改读这里的 registry/src/*/
#   .formal-sample/verus-timeout-seconds  Verus 那几格的限时改成文件里的秒数
#   .formal-sample/e165-stub.sh           replay 格改用这个桩当装置（经 bash 起，不经内存包装）
# 弄坏开关：格的判法那一份的开关 VERUS_PROOF_CHECK_BREAK=<项> 与 E165_PIN_RED_EXAMPLES_BREAK=<项>（逐项让哪份样本判错写在两份脚本的文件头）；
#   汇总与 --check 的归共用库（STAGE_CELLS_BREAK=red-swallowed 让每份红样本整道退 0；marker-ignored 让每份样本六格都跑，汇总那条「跑了 1 格」找不到）。
#   红例两格整行抄（一行 21 段）的三项，开关一行、下一行起写打开之后判错的样本：
#   E165_PIN_RED_EXAMPLES_BREAK=whole-row-unchecked  complete 格不比 ⑨ 起的整行，只认 ①、⑦ 与 ⑤ / ⑥
#       pinned-red-examples-complete-whole-row-red（判绿）
#   E165_PIN_RED_EXAMPLES_BREAK=old-rows-accepted  表里 8 段的旧行照收、不报没整行抄
#       pinned-red-examples-replay-old-row-red（判绿：桩照 ⑦ 打）
#   E165_PIN_RED_EXAMPLES_BREAK=copied-columns-unchecked  不核 ⑪⑫⑬ 与 ②③④、⑳㉑ 与 ⑦⑧
#       pinned-red-examples-replay-copied-columns-red（判绿：桩照 ⑦ 打）
#
#   bash .claude/gate.d/checker-tier-formal-verification.sh [项目根]
#   bash .claude/gate.d/checker-tier-formal-verification.sh --list        逐行打格名与判什么，不起 Verus、不编
#   bash .claude/gate.d/checker-tier-formal-verification.sh --tool-versions   打 Verus 的版本与安装文件的 sha256、安装目录顶层的整体摘要，VERUS_Z3_PATH、VERUS_ROOT、
#     VERUS_USE_RUSTUP 三个变量的取值，以及 Verus 起 rust_verify 用的那条 rustup 工具链：RUSTUP_HOME 与它解析到的路径、工具链 lib/ 与 lib/rustlib/*/lib/ 下全部文件的
#     整体摘要、rustup 程序的 sha256（stage-inputs.tsv 的 environment= 用；工具链里换了一个文件，这一行与阶段指纹都变、旧的全绿标记不再作数；
#     逐样的写法在 research/scripts/verus-proof-check.py 文件头 tool-versions 一段；不起 Verus，不算重型；本机多哈希约 500 MB，页缓存热时多约 0.25 秒）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
VERUS_CHECK="$STAGE_REPOSITORY/research/scripts/verus-proof-check.py"
PIN_CHECK="$STAGE_REPOSITORY/research/scripts/e165-pin-red-examples.py"
ADMISSION_MODULE="$STAGE_REPOSITORY/research/scripts/admission.py"
source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
stage_cell verus-toolchain-pinned cell_verus_toolchain_pinned "Verus 安装的 version 与 commit、Cargo.lock 里六个包的版本、本机 cargo 缓存里包源码的出处与内容指纹都与登记相同" \
  "装登记的 Verus 发布或用 SINGLEFS_VERUS_ROOT 指到它；缓存不对就删掉那个包目录再 cargo fetch --locked；要换版本先照 .claude/gate.d/verus-pins.tsv 的注释复核同源再改登记"
stage_cell verus-proof cell_verus_proof "Verus 带 --no-cheating 验整个 crate，限时内出结局、errors 0、verified 等于登记的数，核里没有作弊标记" \
  "照 Verus 报错的位置修证明或实现；verified 数变了而确是加减了证明，就改 .claude/gate.d/verus-pins.tsv 的 crate 行并写明加减了哪几个；超时先查哪条查询慢，不许靠放宽限时了事；✗ 行写「没跑成：内存包装」的照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办，不改证明或实现；✗ 行写「被信号停了」的照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办，这一次的输出不算判定，不改证明或实现"
stage_cell verus-vacuity cell_verus_vacuity "核里每一处 requires 的函数体头插 assert(false) 都验不过（前置条件可满足，不空真）" \
  "验得过的那一处前置条件不可满足：改前置条件让它可满足；插入点认错了就按 research/scripts/verus-proof-check.py 的 requires_insertion_points 改写法；✗ 行写「没跑成：内存包装」的照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办，不改前置条件"
stage_cell verus-must-red-worlds cell_verus_must_red_worlds "必红世界表每行改坏核之后 Verus 都报错、证明错条数等于第五段的 proof_errors=，原文恰好命中一次" \
  "锚点腐化就照今天的核改那一行的原文；改坏了还验得过就补证明或收紧 spec；第五段照约定写 proof_errors=<证明错条数>（不数 rlimit 用尽），条数变了先确认红的还是这条变异要盯的那一处再改；表在 .claude/gate.d/verus-pins.tsv 的 must-red-worlds 行点名的路径；✗ 行写「没跑成：内存包装」的照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办，不改表或证明"
stage_cell pinned-red-examples-complete cell_pinned_red_examples_complete "登记的全部 E165 红例账每一行都钉在 e165-pinned-red-examples.tsv 里，钉着的那一行第 9 段起与红例账原行逐字节相同" \
  "补钉：python3 research/scripts/e165-pin-red-examples.py <没钉全的红例账>（已钉的不重复，⑨ 起不同的另钉一行整行抄；这一格的输出里给了整条命令）；表里还有 8 段的旧行先跑 python3 research/scripts/e165-pin-red-examples.py --backfill"
stage_cell pinned-red-examples-replay cell_pinned_red_examples_replay "钉住的每一行是 21 段整行抄、抄来的账行与自己的段对得上，点名的变异还在它的表里，装置在今天的实现上逐行打的结果与原实现期望的逐态行逐字节相同" \
  "8 段的旧行跑 python3 research/scripts/e165-pin-red-examples.py --backfill 补成 21 段；抄来的账行与自己的段对不上是手改坏了，从 git 找回再钉；变异行不在了就找回那一行；装置逐行对不上是今天的实现在钉住的输入上与原实现不同，按回归修实现；装置报指纹漂了先查它的枚举次序与状态编码；✗ 行写「没跑成：内存包装」的照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办，不改变异行或实现；✗ 行写「被信号停了」的照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办，这一次的输出不算判定，不改变异行或实现"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}
root_argument=""
tool_versions_requested=0
while (($#)); do
  case "$1" in
    --tool-versions) tool_versions_requested=1; shift ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…]、--tool-versions 与一个项目根。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不了项目根 $ROOT"; echo "     → 怎么办：第一个参数给被判的仓根（不给就判这道阶段所在的仓）。"; exit 2; }
ROOT="$(pwd)"
if ((tool_versions_requested)); then
  exec python3 "$VERUS_CHECK" tool-versions "$ROOT"
fi

sample_mode=0
if [[ "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" && -z "${GATE_STAGED_FROM:-}" ]]; then sample_mode=1; fi
toolchain_options=()
verus_options=()
replay_options=()
if ((sample_mode)); then
  [[ -d .formal-sample/verus-install ]] && toolchain_options+=(--verus-root "$ROOT/.formal-sample/verus-install")
  [[ -d .formal-sample/cargo-home ]] && toolchain_options+=(--cargo-home "$ROOT/.formal-sample/cargo-home")
  [[ -f .formal-sample/verus-timeout-seconds ]] && verus_options+=(--timeout "$(tr -dc '0-9' < .formal-sample/verus-timeout-seconds)")
  [[ -f .formal-sample/e165-stub.sh ]] && replay_options+=(--binary "$ROOT/.formal-sample/e165-stub.sh" --memory-cap none)
fi

cell_verus_toolchain_pinned() { python3 "$VERUS_CHECK" toolchain "$ROOT" ${toolchain_options[@]+"${toolchain_options[@]}"}; }
cell_verus_proof() { python3 "$VERUS_CHECK" proof "$ROOT" ${verus_options[@]+"${verus_options[@]}"}; }
cell_verus_vacuity() { python3 "$VERUS_CHECK" vacuity "$ROOT" ${verus_options[@]+"${verus_options[@]}"}; }
cell_verus_must_red_worlds() { python3 "$VERUS_CHECK" must-red-worlds "$ROOT" ${verus_options[@]+"${verus_options[@]}"}; }
cell_pinned_red_examples_complete() { python3 "$PIN_CHECK" --check --root "$ROOT"; }
cell_pinned_red_examples_replay() {
  # 前面 Verus 那几格已经排不上（判法写了 $queue_refused_record）：这一格不编装置、不起重放，不再排一次队
  if [[ -s "$queue_refused_record" ]]; then
    local refused_where="" refused_howto=""
    IFS=$'\t' read -r refused_where refused_howto < "$queue_refused_record"
    echo "  ✗ 没跑成：内存包装排不上，没再排队（同一道里 $refused_where）：这一格没编装置、没起重放"
    echo "     → 怎么办：${refused_howto:-照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办}"
    return 1
  fi
  python3 "$PIN_CHECK" --replay --root "$ROOT" ${replay_options[@]+"${replay_options[@]}"}
}

# 复用（接法同 checker-tier-lkmm.sh）：真仓与 --staged 下整道跑时，stage-inputs.tsv 本阶段那一行的路径与 environment=（--tool-versions 那一行）都没变就整道退 77；判定只认 1 为可跳过，别的非 0 判红
if ((sample_mode == 0 && STAGE_CELLS_CHECK_GIVEN == 0)); then
  source "$STAGE_REPOSITORY/research/scripts/stage-run-or-skip.sh" || { echo "  ✗ source 不进 research/scripts/stage-run-or-skip.sh，判不出这一道要不要跑"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一道按红记。"; exit 1; }
  if ( stage_run_or_skip "$STAGE_REPOSITORY/research/scripts/stage-must-run.sh" "复用上一次整轮全绿的判定" \
         "要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；这一道读哪几条路径、Verus 的版本怎么进判定见 .claude/gate.d/stage-inputs.tsv。" -- "$ROOT" "$(basename "$0")" ); then
    reuse_exit=0
  else
    reuse_exit=$?
  fi
  if ((reuse_exit == 77)); then
    [[ -n "${GATE_NOT_RUN_FILE:-}" ]] && printf '%s 整道复用上一次整轮全绿的判定（输入与 Verus 版本都没变），本次未跑\n' "$(basename "$0")" >> "$GATE_NOT_RUN_FILE"
    exit 77
  fi
  ((reuse_exit == 0)) || exit "$reuse_exit"
fi

# 本次未跑的格先收进自己的文件：有一格本次未跑就不记环境、不写全绿标记（没判全的一趟不算上次成功），再原样转给 $GATE_NOT_RUN_FILE
not_run_record="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
# 一道里只排一次：Verus 那几格第一次拿到内存包装的 252 就往这份文件写一行（verus-proof-check.py 的 QueueRefusal），之后的格读到就不再排队
queue_refused_record="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; rm -f "${not_run_record:?}"; exit 1; }
if ( export GATE_NOT_RUN_FILE="$not_run_record" VERUS_PROOF_CHECK_QUEUE_REFUSED_FILE="$queue_refused_record"; stage_cells_run "$ROOT" ); then stage_exit=0; else stage_exit=$?; fi
if [[ -n "${GATE_NOT_RUN_FILE:-}" && -s "$not_run_record" ]]; then cat "$not_run_record" >> "$GATE_NOT_RUN_FILE"; fi
if [[ -s "$queue_refused_record" ]]; then
  IFS=$'\t' read -r refused_where refused_howto < "$queue_refused_record"
  echo "  ✗ 这一道没跑成：内存包装排不上（$refused_where），之后的格没再排队，整道只等了一次"
  echo "     → 怎么办：${refused_howto:-照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办}；空出来之后整道重跑，不改证明、核、必红世界表或实现"
fi
rm -f "${queue_refused_record:?}"
if ((stage_exit == 0 && sample_mode == 0 && STAGE_CELLS_CHECK_GIVEN == 0)) && [[ ! -s "$not_run_record" ]]; then
  # 写不成只报不红：后果只是下一趟多跑
  if ! python3 "$ADMISSION_MODULE" gate-record-environment "$ROOT" "$(basename "$0")"; then
    echo "  ! 没记下这一次的 Verus 版本（admission.py gate-record-environment 没成）：下一趟整轮这一道照跑，不会复用"
  fi
  if ! python3 "$ADMISSION_MODULE" stage-marker-write "$ROOT" "$(basename "$0")" >/dev/null; then
    echo "  ! 没写成这一道的全绿标记（admission.py stage-marker-write 没成）：下一趟这一道照跑，不会复用"
  fi
fi
rm -f "${not_run_record:?}"
exit "$stage_exit"
