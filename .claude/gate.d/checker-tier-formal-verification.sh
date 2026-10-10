#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓、此刻的 Verus 安装与 cargo 缓存，上一次的结论不替这一次作保
# run-condition: none 要的 Verus、cargo 与装置由各格的判法自己判，缺了判红；核还没进仓时 Verus 那四格报本次未跑
# gate-stage: 形式化验证（Verus 证明恢复重放取前缀的核，版本钉住、空真与必红世界；E165 全量的红例钉进快档）
# gate-category: checker-tier 类
# gate-similar: checker-tier-lkmm.sh 同是起外部验证器的一道，接法照抄它（判法放脚本、按 stage-inputs.tsv 复用、environment= 记工具版本）；它起 herd7 判 litmus，这里起 Verus 判 crates/ 的证明，并进去会让两件不相干的事共用一个复用判定
# gate-similar: checker-tier-crates-mutation-replay.sh 它逐条改坏 crates/ 要求点名的测试红；这里的必红世界要求 Verus 报证明错，写不进它的 crates/mutations.tsv
# gate-similar: code-source-discipline.sh 它的 mutation-tables 格静态判实验二进制的变异表成形、锚点唯一；这里判 E165 红例账与钉住的表、并逐行在今天的实现上重放
# gate-similar: checker-tier-research-build-and-replay.sh 它的 experiment-replay 格整份复跑入库实验比产物；这里只逐行重放钉住的红例（快档）
# gate-similar: checker-tier-layer0-replay.sh 它判崩溃点重放与登记的崩溃枚举用例；这里判证明与有界穷举的红例，不起崩溃枚举
#
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就六格按下面的次序全跑，格名写错退 2）：
# gate-cell: verus-toolchain-pinned Verus 安装的 version 与 commit、Cargo.lock 里六个包的版本、本机 cargo 缓存里包源码的出处与内容指纹都与登记相同
# gate-cell: verus-proof Verus 带 --no-cheating 验整个 crate，限时内出结局、errors 0、verified 等于登记的数，核里没有作弊标记
# gate-cell: verus-vacuity 核里每一处 requires 的函数体头插 assert(false) 都验不过（前置条件可满足，不空真）
# gate-cell: verus-must-red-worlds 必红世界表每行改坏核之后 Verus 都报错、证明错条数等于第五段的 proof_errors=，原文恰好命中一次
# gate-cell: pinned-red-examples-complete 登记的全部 E165 红例账每一行都钉在 e165-pinned-red-examples.tsv 里，钉着的那一行第 9 到 21 段与红例账原行逐字节相同、第 22 段是那份账的档
# gate-cell: pinned-red-examples-replay 钉住的每一行是 22 段整行抄带档、抄来的账行与自己的段对得上，点名的变异还在它的表里，装置按档在今天的实现上逐行打的结果与原实现期望的逐态行逐字节相同
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
#     每行的 ㉒ 是出那份账的那一趟的档（同一趟 kill-bounds 产物的 device_arguments=）：complete 按档判钉全，replay 按档分组起装置、每组带 ㉒ 的参数（例 --profile r2）。
#     --replay 编装置的内存上限读 GATE_E165_PINNED_MEMORY_MAX（默认 2G，由那份脚本读）。
#     装置 e165_replay_bounded_enumeration 还不认 --pinned-cases 时，replay 格判完「变异还在」那一半就退 77。
#
# 复用按格算（用户 2026-10-08 定，records/2026-09-16-subagent拆分提案.md 第六十三节）：.claude/gate.d/stage-inputs.tsv 里每格一行，键 checker-tier-formal-verification.sh:<格名>，
#   判法在 research/scripts/admission.py 的 gate-reuse（经 research/scripts/stage-must-run.sh）；阶段脚本自己进每格的比对，整张登记表不进，这一格自己的登记行进。
#   Verus 四格的输入只有 crates/singlefs-replay-prefix/（含必红世界表 verus-must-red-worlds.tsv）、.claude/gate.d/verus-pins.tsv、research/scripts/verus-proof-check.py、内存包装、capped.sh、
#     lib/stage-cells.sh 与 --tool-versions 环境（verus-toolchain-pinned 另读仓根 Cargo.lock）；不含别的 crate，core 改动不重跑它们。
#   红例两格的输入是 singlefs-checker-tier 编 e165_replay_bounded_enumeration 的 cargo 依赖闭包（登记行第三列 closure=，admission.py 的 cargo_closure_input_paths 现算：
#     path 依赖的传递闭包，减去 cargo 编这个 bin 读不到的 tests/、别的 bin 的源码与 crate 根下 cargo 不读的文件，例 verus-must-red-worlds.tsv），不手列 crate；加两张变异表、红例账与同一趟 kill-bounds 产物（只取 e165 的）、钉表、判法脚本及其 import；
#     complete 格另读作废红例账登记表 .claude/gate.d/e165-void-red-example-ledgers.tsv（登记的账 --check 跳过，写法在表头）
#     与被取代红例账登记表 .claude/gate.d/e165-superseded-red-example-ledgers.tsv（被同档后一趟取代的完整账 --check 跳过；取代它的账照常要钉全、要同档、
#     要逐行覆盖被取代账的（域, N, 枚举序号, ①），写法在表头）。
#   真仓与 --staged 下整道跑（不给 --check、--item）时每格各问一次：输入没变、上次判绿作数（全绿标记或上次整轮全绿那棵暂存树）就复用，汇总写「复用」，只跑变了的格，
#     六格都复用整道退 77；单跑一格、--check、--item、--list、--list-items 与样本不问复用、不写标记。跑绿的格各写自己的全绿标记与 Verus 版本记录；有一格本次未跑或红，这一趟一份也不写。
# 按条复用（格里再分条；判法与键写在两份判法脚本文件头「逐项点名与按条复用」，存放与键的算法在判定库 research/scripts/lib_verdict_library.py）：
#   要跑的格里，proof、空真每处、必红世界每行、红例钉表每行各按自己的键记判定；键相同、上次判绿就复用，不起 Verus、不编装置；verus-toolchain-pinned 与
#     pinned-red-examples-complete 每趟现判、不复用。键分输入指纹与判法版本两列（判定库文件头「键分两列」）：
#     Verus 那几项共用的输入罩 crates/singlefs-replay-prefix/ 整个目录（跳过 verus-pins.tsv 的 must-red-worlds 行点名的
#     必红世界表：Verus 不读它，表里每行只进它自己那一项的键，改一行只现判那一行，表头与注释不进任何一项的键）、verus-pins.tsv、工具链（--tool-versions 那一行）与限时；
#     红例每行的输入罩这一行、装置的整个 cargo 依赖闭包（admission.py 的 cargo_closure_input_paths）与工具链、⑥ 引的变异行。
#     判法版本是那份判法脚本与判定库两份内容的摘要。共用的输入有一样变了、或判法版本变了，整格每一项都现判。
#   判定库：真仓与 --staged 下在被判那棵树的 git common-dir 的 singlefs-formal-verdicts/（GATE_FORMAL_VERDICT_LIBRARY 盖过它）；一项判完当场原子落盘，
#     被打断之后同一条命令重跑只补没判的。整道跑且没设 SINGLEFS_GATE_FULL=1 才读；--check、--item 与 SINGLEFS_GATE_FULL=1 照样记、不读（--no-reuse）。
#   每项一行「· <项>：绿|红，现判（<原因>）|复用」，每格一行「· 按条：现判 N、复用 M（现判的原因：…）」，整道末了一行「── 按条（…）：现判 N、复用 M」。
# 逐项点名（.claude/rules/verification.md「门禁的结构」里重型门禁的 --list-items / --item）：
#   --list-items 逐行打「项<TAB>格[<TAB>说明]」：toolchain、proof、complete 各一项，空真每处 requires 一项 vacuity:<核>:<requires 行号>，
#     必红世界每行一项 must-red:<变异名>，红例钉表每行一项 pinned:<钉表行号>；只读登记表、核与两张表，不编、不起 Verus 与装置。
#   --item <项>（可给多次，取并集）只跑点了项的那几格，空真、必红世界、红例重放三格只判点名的项；项名写错、缺项名退 2 并列出可点名的项；与 --check、--list-items 不一起用（退 2）。
#   --quick（轻型，.claude/rules/verification.md「两档：轻型与重型」）：核心部分是 QUICK_CORE_ITEMS 那三项（toolchain、proof、complete：工具链、证明、钉全），
#     再连同 --item 点名的项（可变部分，取并集）照 --item 那一路跑：只记判定库、不读（--no-reuse），不问按格复用、不写全绿标记，轻型没有剪枝。
#     与 --check、--list-items 不一起用（退 2）。样本根放 .formal-sample/quick 当成给了 --quick（样本没法带参数）。
#   自证：research/scripts/admission.py --selftest 的 run_formal_cell_reuse_cells（临时仓里跑这份脚本，两份判法脚本换成往跟踪文件记一行的假驱动，不起 Verus、不起 cargo）。弄坏开关 ADMISSION_BREAK=<项>：
#     closure-only-replay-prefix        红例两格的闭包退回只算 crates/singlefs-replay-prefix/，core 改了红例两格不重跑      自证「① 只改一个 core 文件」那一条红
#     proof-key-without-replay-prefix   自证造临时仓的登记表时，verus-proof 那格的键拿掉 crates/singlefs-replay-prefix/      自证「② 只改 replay-prefix」那一条红
# 样本（fixtures/checker-tier-formal-verification.sh/<格名>-<情形>/，各放 .gate-cells 点名一格；共享门禁的 stage-selftest.sh 跑）。名字读不出判法的几份：
#   verus-toolchain-pinned-*：假的 Verus 安装与假的 cargo 缓存放在样本的 .formal-sample/ 下
#   verus-proof-timeout-red：rlimit 设成无穷、样本限时 20 秒；verus-proof-killed-red：替身 Verus 起来就 kill -9 自己，判红，✗ 写「Verus 被信号停了」；
#   pinned-red-examples-replay-killed-red：桩装置打一行就 kill -9 自己，判红，✗ 写「装置被信号停了」；这两份另核汇总出路里「被信号停了」那一句（不改证明或实现）
#   verus-proof-kernel-absent（核没进仓）、pinned-red-examples-replay-device-absent（没有桩、装置源码不认 --pinned-cases）：退 77
#   verus-vacuity-closure-red：外层函数的前置条件可满足、里面闭包的不可满足
#   verus-must-red-worlds-：-rlimit-red 改坏之后只剩 rlimit 用尽；-rlimit-with-proof-error-green 另有一条证明错（proof_errors=1）；
#     -legacy-expectation-red 第五段旧写法「verified, 1 errors」；-proof-error-count-red 一条证明错、第五段写 proof_errors=2
#   pinned-red-examples-complete-void-green：没收尾的一趟登记进作废红例账登记表，跳过、判绿；-complete-void-missing-red：登记的账不在，判红；
#   pinned-red-examples-complete-superseded-green：完整的前一趟登记成被同档后一趟取代，后一趟钉全，跳过前一趟、判绿；
#     -complete-superseded-uncovered-red：前一趟有一个反例状态后一趟的账里没有，判红；
#   pinned-red-examples-complete-whole-row-red：⑲ 与账的第 11 段不同，别的段都对；-replay-copied-columns-red：⑳ 与 ⑦ 不同；-replay-old-row-red：8 段的旧行
#   pinned-red-examples-replay-profiles-green：r1、r2 两档各三行，桩照命令行的 --profile 打（r2 档在 counters 后面多一个 j6qn），判绿；
#     -replay-profile-missing-red：有一行没写档（21 段），判红、出路给 --backfill；-complete-profile-red：r2 档的账（产物 device_arguments=--profile+r2）钉成了 --profile r1，判红
#   Verus 那几份样本起的是真的 Verus（SINGLEFS_VERUS_ROOT 或登记表 release 行的安装目录），没装就判红，与真仓一样；verus-proof-killed-red 起替身
#   （设了 SINGLEFS_VERUS_ROOT 时它盖过登记表、起的是真 Verus，这一份判错）。
# 只供样本的开关（被判的根不是这道阶段所在的仓、也没设 GATE_STAGED_FROM 时才认；真仓与 --staged 下放了也不认）：
#   .formal-sample/verus-install/         版本一致格改读这里的 version.json（不起它）
#   .formal-sample/cargo-home/            版本一致格改读这里的 registry/src/*/
#   .formal-sample/verus-timeout-seconds  Verus 那几格的限时改成文件里的秒数
#   .formal-sample/e165-stub.sh           replay 格改用这个桩当装置（经 bash 起，不经内存包装）
#   .formal-sample/list-items             当成给了 --list-items（样本没法带参数）
#   .formal-sample/items                  每一行当成一个 --item（命令行给了 --item 就不看它）
#   .formal-sample/quick                  当成给了 --quick（样本 quick-green、quick-red：核没进仓，toolchain 与 proof 两项本次未跑，complete 照钉表判绿 / 判红）
#   .formal-sample/run-twice              判定库放这一趟的临时目录、读，整道接连跑两趟：第二趟按条全复用（按条复用的样本）
# 弄坏开关：格的判法那一份的开关 VERUS_PROOF_CHECK_BREAK=<项> 与 E165_PIN_RED_EXAMPLES_BREAK=<项>（逐项让哪份样本判错写在两份脚本的文件头）；
#   汇总与 --check 的归共用库（STAGE_CELLS_BREAK=red-swallowed 让每份红样本整道退 0；marker-ignored 让每份样本六格都跑，汇总那条「跑了 1 格」找不到）。
#   逐项点名与按条复用：判法那一半的开关在两份脚本文件头「逐项点名与按条复用」；这一道自己接的那一半 FORMAL_VERIFICATION_BREAK=<项>[,<项>…]，
#   自证是 research/scripts/admission.py --selftest 的 run_formal_item_cells（临时仓里跑这份脚本、两份判法换成记参数的假驱动）：
#     items-unchecked       --item 写错的项名悄悄丢掉、不退 2        自证「--item 写错退 2」那一条红；样本 pinned-red-examples-replay-item-unknown 判错
#     reuse-on-named-run    --check、--item、SINGLEFS_GATE_FULL=1 照样读判定库（不转 --no-reuse）   自证「--item 只记不读」「SINGLEFS_GATE_FULL=1 只记不读」两条红
#     library-in-tmpdir     判定库放 $TMPDIR（门禁每轮新给、跑完删）而不是 git common-dir   自证「判定库在 git common-dir」那一条红
#     quick-core-unselected  --quick 不把核心部分那三项放进点名，照整道跑                    样本 quick-green 判错（输出里找不到「--quick：核心部分」）
#   红例两格整行抄（一行 21 段）的三项，开关一行、下一行起写打开之后判错的样本：
#   E165_PIN_RED_EXAMPLES_BREAK=whole-row-unchecked  complete 格不比 ⑨ 起的整行，只认 ①、⑦ 与 ⑤ / ⑥
#       pinned-red-examples-complete-whole-row-red（判绿）
#   E165_PIN_RED_EXAMPLES_BREAK=old-rows-accepted  表里 8 段的旧行照收、不报没整行抄
#       pinned-red-examples-replay-old-row-red（判绿：桩照 ⑦ 打）
#   E165_PIN_RED_EXAMPLES_BREAK=copied-columns-unchecked  不核 ⑪⑫⑬ 与 ②③④、⑳㉑ 与 ⑦⑧
#       pinned-red-examples-replay-copied-columns-red（判绿：桩照 ⑦ 打）
#   E165_PIN_RED_EXAMPLES_BREAK=void-ledgers-ignored  complete 格不读作废红例账登记表，登记的账照判
#       pinned-red-examples-complete-void-green（判红：没收尾的那一趟照判）
#   E165_PIN_RED_EXAMPLES_BREAK=void-rows-unchecked  登记表的缺段、理由太短、出处写法、账不在、登记两次都不判
#       pinned-red-examples-complete-void-missing-red（判绿）
#   E165_PIN_RED_EXAMPLES_BREAK=superseded-coverage-unchecked  不核被取代的账每一行在取代它的账里有对应行（被取代登记表另三个开关的自证格写在 e165-pin-red-examples.py 文件头）
#       pinned-red-examples-complete-superseded-uncovered-red（判绿）
#   红例两格认档（㉒）的三项（自证格写在 e165-pin-red-examples.py 文件头「钉表认档」一段）：
#   E165_PIN_RED_EXAMPLES_BREAK=replay-without-profile  replay 格起装置不带 ㉒ 的档参数
#       pinned-red-examples-replay-profiles-green（判红：桩一律按 r1 打，r2 那三行与 ⑦ 不同）
#   E165_PIN_RED_EXAMPLES_BREAK=profile-column-unchecked  21 段的行（没写档）照收、当 --profile r1
#       pinned-red-examples-replay-profile-missing-red（判绿：桩照 ⑦ 打）
#   E165_PIN_RED_EXAMPLES_BREAK=check-ignores-profile  complete 格不比 ㉒，钉成别的档也算钉着
#       pinned-red-examples-complete-profile-red（判绿）
#
#   bash .claude/gate.d/checker-tier-formal-verification.sh [项目根]
#   bash .claude/gate.d/checker-tier-formal-verification.sh --list        逐行打格名与判什么，不起 Verus、不编
#   bash .claude/gate.d/checker-tier-formal-verification.sh --list-items  逐行打可点名的项与它属于哪一格，不起 Verus、不编
#   bash .claude/gate.d/checker-tier-formal-verification.sh --item <项> [--item <项>…] [项目根]   只跑点名的项
#   bash .claude/gate.d/checker-tier-formal-verification.sh --quick [--item <项>…] [项目根]      轻型：核心部分三项，再加点名的项
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
stage_cell pinned-red-examples-complete cell_pinned_red_examples_complete "登记的全部 E165 红例账每一行都钉在 e165-pinned-red-examples.tsv 里，钉着的那一行第 9 到 21 段与红例账原行逐字节相同、第 22 段是那份账的档" \
  "补钉：python3 research/scripts/e165-pin-red-examples.py <没钉全的红例账>（已钉的不重复，⑨ 起不同的另钉一行整行抄，档取同一趟产物的 device_arguments=；这一格的输出里给了整条命令）；表里还有 8 段的旧行或 21 段没写档的行先跑 python3 research/scripts/e165-pin-red-examples.py --backfill；同一档下原实现逐态行不同的，先弄清是实现改了行为还是账跑在别的树上"
stage_cell pinned-red-examples-replay cell_pinned_red_examples_replay "钉住的每一行是 22 段整行抄带档、抄来的账行与自己的段对得上，点名的变异还在它的表里，装置按档在今天的实现上逐行打的结果与原实现期望的逐态行逐字节相同" \
  "8 段的旧行与 21 段没写档的行跑 python3 research/scripts/e165-pin-red-examples.py --backfill 补成 22 段；抄来的账行与自己的段对不上是手改坏了，从 git 找回再钉；变异行不在了就找回那一行；装置逐行对不上是今天的实现在钉住的输入上与原实现不同，按回归修实现；装置报指纹漂了先查它的枚举次序与状态编码；✗ 行写「没跑成：内存包装」的照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办，不改变异行或实现；✗ 行写「被信号停了」的照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办，这一次的输出不算判定，不改变异行或实现"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}
root_argument=""
tool_versions_requested=0
list_items_requested=0
quick_requested=0
requested_items=()
QUICK_CORE_ITEMS=(toolchain proof complete)   # 轻型的核心部分：工具链、证明、钉全（.claude/rules/verification.md「两档：轻型与重型」）
while (($#)); do
  case "$1" in
    --tool-versions) tool_versions_requested=1; shift ;;
    --list-items) list_items_requested=1; shift ;;
    --quick) quick_requested=1; shift ;;
    --item|--item=*)
      if [[ "$1" == --item ]]; then
        if (($# < 2)) || [[ -z "$2" ]]; then
          echo "  ✗ --item 后面缺项名"
          echo "     → 怎么办：写成 --item <项>（可给多次）；可点名的项用 bash .claude/gate.d/$(basename "$0") --list-items 列。"
          exit 2
        fi
        requested_items+=("$2"); shift 2
      else
        requested_items+=("${1#--item=}"); shift
      fi ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…]、--list-items、--item <项>、--quick、--tool-versions 与一个项目根。"
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
# 只供样本的点名开关：.formal-sample/list-items 在就当成给了 --list-items，.formal-sample/items 的每一行当成一个 --item（命令行给了的不看它）
if ((sample_mode)) && [[ -f .formal-sample/list-items ]]; then list_items_requested=1; fi
if ((sample_mode)) && ((${#requested_items[@]} == 0)) && [[ -f .formal-sample/items ]]; then
  while IFS= read -r sample_item || [[ -n "$sample_item" ]]; do
    [[ -n "$sample_item" && "$sample_item" != \#* ]] && requested_items+=("$sample_item")
  done < .formal-sample/items
fi
if ((sample_mode)) && [[ -f .formal-sample/quick ]]; then quick_requested=1; fi
# --quick（文件头「逐项点名」一段的 --quick）：核心部分三项并进点名，照 --item 那一路跑（只记不读判定库、不问按格复用、不写全绿标记）
if ((quick_requested)); then
  if ((list_items_requested)) || ((STAGE_CELLS_CHECK_GIVEN)); then
    echo "  ✗ --quick 不与 --list-items、--check 一起用：--quick 已经定了跑哪几项"
    echo "     → 怎么办：只写 --quick（可再带 --item <项>）；要列项单独跑 --list-items，要整格跑只写 --check <格名>。"
    exit 2
  fi
  quick_named_count=${#requested_items[@]}
  if [[ ",${FORMAL_VERIFICATION_BREAK:-}," != *,quick-core-unselected,* ]]; then
    requested_items=("${QUICK_CORE_ITEMS[@]}" ${requested_items[@]+"${requested_items[@]}"})
    echo "  · --quick：核心部分 ${QUICK_CORE_ITEMS[*]} 三项$( ((quick_named_count)) && echo "，另加点名的 $quick_named_count 项" )；轻型没有剪枝：判定库只记不读"
  fi
fi

# 可点名的项（--list-items 打它、--item 照它核）：两份判法各列自己那几格的，「项<TAB>格[<TAB>说明]」；只读登记表、核、必红世界表与钉表，不编、不起 Verus 与装置
formal_items_file=""
list_formal_items() {
  formal_items_file="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
  local listing_exit
  if python3 "$VERUS_CHECK" list-items "$ROOT" > "$formal_items_file"; then listing_exit=0; else listing_exit=$?; fi
  if ((listing_exit != 0)); then
    echo "  ✗ 列不出 Verus 那几格的项：research/scripts/verus-proof-check.py list-items 退 $listing_exit"
    echo "     → 怎么办：照它上面打的 ✗ 修登记表 .claude/gate.d/verus-pins.tsv 或必红世界表；列项只读文件，不起 Verus。"
    rm -f "${formal_items_file:?}"; exit 1
  fi
  if python3 "$PIN_CHECK" --list-items --root "$ROOT" >> "$formal_items_file"; then listing_exit=0; else listing_exit=$?; fi
  if ((listing_exit != 0)); then
    echo "  ✗ 列不出红例两格的项：research/scripts/e165-pin-red-examples.py --list-items 退 $listing_exit"
    echo "     → 怎么办：照它上面打的 ✗ 修钉表 .claude/gate.d/e165-pinned-red-examples.tsv；列项只读文件，不编装置。"
    rm -f "${formal_items_file:?}"; exit 1
  fi
}
if ((list_items_requested)); then
  if ((${#requested_items[@]})) || ((STAGE_CELLS_CHECK_GIVEN)); then
    echo "  ✗ --list-items 只列项，不与 --item、--check 一起用"
    echo "     → 怎么办：先单独跑 --list-items 看项名，再用 --item <项> 点名跑。"
    exit 2
  fi
  list_formal_items
  cat "$formal_items_file"
  rm -f "${formal_items_file:?}"
  exit 0
fi
# --item：点名的项各属哪一格照列出的表找；格只跑点了项的那几格（次序照登记），vacuity、must-red、pinned 那几格只判点名的项（转给判法的 --item）
declare -A item_options_of_cell=()
if ((${#requested_items[@]})); then
  if ((STAGE_CELLS_CHECK_GIVEN)); then
    echo "  ✗ --item 与 --check 不一起用：--item 已经定了跑哪几格"
    echo "     → 怎么办：只写 --item <项>（可给多次）；要整格跑就只写 --check <格名>。"
    exit 2
  fi
  list_formal_items
  declare -A cell_of_item=()
  while IFS=$'\t' read -r listed_item listed_cell _listed_note; do
    [[ -n "$listed_item" ]] && cell_of_item[$listed_item]="$listed_cell"
  done < "$formal_items_file"
  unknown_items=()
  declare -A cells_wanted=()
  for wanted_item in "${requested_items[@]}"; do
    if [[ -z "${cell_of_item[$wanted_item]+set}" ]]; then
      [[ ",${FORMAL_VERIFICATION_BREAK:-}," == *,items-unchecked,* ]] || unknown_items+=("$wanted_item")
      continue
    fi
    cells_wanted[${cell_of_item[$wanted_item]}]=1
    case "${cell_of_item[$wanted_item]}" in
      verus-vacuity|verus-must-red-worlds|pinned-red-examples-replay)
        item_options_of_cell[${cell_of_item[$wanted_item]}]+="$wanted_item"$'\n' ;;
    esac
  done
  if ((${#unknown_items[@]})); then
    echo "  ✗ 没有这些项：${unknown_items[*]}"
    echo "     → 怎么办：项名照下面这张表写（第一列是项、第二列是格），给几次 --item 取并集："
    cut -f1,2 "$formal_items_file" | sed 's/^/       /'   # gate-lint:detail
    rm -f "${formal_items_file:?}"
    exit 2
  fi
  rm -f "${formal_items_file:?}"
  STAGE_CELLS_SELECTED=()
  for cell_name in "${STAGE_CELLS_NAMES[@]}"; do
    [[ -n "${cells_wanted[$cell_name]+set}" ]] && STAGE_CELLS_SELECTED+=("$cell_name")
  done
  STAGE_CELLS_CHECK_GIVEN=1
  STAGE_CELLS_SELECTION_SOURCE="--item 点名 ${#requested_items[@]} 项"
fi
# 把这一格点名的项交成判法的 --item 参数（一项一对），交到 item_arguments 数组里
item_arguments_of() {
  item_arguments=()
  local item_line
  while IFS= read -r item_line; do
    [[ -n "$item_line" ]] && item_arguments+=(--item "$item_line")
  done <<<"${item_options_of_cell[$1]:-}"
}
toolchain_options=()
verus_options=()
replay_options=()
if ((sample_mode)); then
  [[ -d .formal-sample/verus-install ]] && toolchain_options+=(--verus-root "$ROOT/.formal-sample/verus-install")
  [[ -d .formal-sample/cargo-home ]] && toolchain_options+=(--cargo-home "$ROOT/.formal-sample/cargo-home")
  [[ -f .formal-sample/verus-timeout-seconds ]] && verus_options+=(--timeout "$(tr -dc '0-9' < .formal-sample/verus-timeout-seconds)")
  [[ -f .formal-sample/e165-stub.sh ]] && replay_options+=(--binary "$ROOT/.formal-sample/e165-stub.sh" --memory-cap none)
fi
# 判定库（research/scripts/lib_verdict_library.py）：真仓与 --staged 下放被判那棵树的 git common-dir 里（跨轮留着，几棵 worktree 共用），
#   GATE_FORMAL_VERDICT_LIBRARY 盖过它；整道跑（不给 --check、--item）且没设 SINGLEFS_GATE_FULL=1 时读（键相同就复用），别的只记不读（--no-reuse）。
#   样本不读不写，只有放了 .formal-sample/run-twice 的那份：判定库放这一趟的临时目录、读，整道接连跑两趟（第二趟应当全复用）。
verdict_library=""
verdict_rounds=1
verdict_library_is_temporary=0
if ((sample_mode == 0)); then
  if [[ -n "${GATE_FORMAL_VERDICT_LIBRARY:-}" ]]; then
    verdict_library="$GATE_FORMAL_VERDICT_LIBRARY"
  elif common_directory="$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$common_directory" ]]; then
    verdict_library="$common_directory/singlefs-formal-verdicts"
    [[ ",${FORMAL_VERIFICATION_BREAK:-}," == *,library-in-tmpdir,* ]] && verdict_library="${TMPDIR:-/tmp}/singlefs-formal-verdicts"
  else
    echo "  ! 取不到 $ROOT 的 git common-dir，判定库没处放：这一趟每一项都现判、不记"
  fi
elif [[ -f .formal-sample/run-twice ]]; then
  verdict_library="$(mktemp -d)" || { echo "  ✗ 建不了临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
  verdict_library_is_temporary=1
  verdict_rounds=2
fi
if [[ -n "$verdict_library" ]]; then
  library_options=(--verdict-library "$verdict_library")
  if { ((STAGE_CELLS_CHECK_GIVEN)) || [[ "${SINGLEFS_GATE_FULL:-}" == 1 ]]; } && [[ ",${FORMAL_VERIFICATION_BREAK:-}," != *,reuse-on-named-run,* ]]; then library_options+=(--no-reuse); fi
  verus_options+=("${library_options[@]}")
  replay_options+=("${library_options[@]}")
fi

cell_verus_toolchain_pinned() { python3 "$VERUS_CHECK" toolchain "$ROOT" ${toolchain_options[@]+"${toolchain_options[@]}"}; }
cell_verus_proof() { python3 "$VERUS_CHECK" proof "$ROOT" ${verus_options[@]+"${verus_options[@]}"}; }
cell_verus_vacuity() { item_arguments_of verus-vacuity; python3 "$VERUS_CHECK" vacuity "$ROOT" ${verus_options[@]+"${verus_options[@]}"} ${item_arguments[@]+"${item_arguments[@]}"}; }
cell_verus_must_red_worlds() { item_arguments_of verus-must-red-worlds; python3 "$VERUS_CHECK" must-red-worlds "$ROOT" ${verus_options[@]+"${verus_options[@]}"} ${item_arguments[@]+"${item_arguments[@]}"}; }
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
  item_arguments_of pinned-red-examples-replay
  python3 "$PIN_CHECK" --replay --root "$ROOT" ${replay_options[@]+"${replay_options[@]}"} ${item_arguments[@]+"${item_arguments[@]}"}
}

# 按格复用（复用的判法同 checker-tier-lkmm.sh，粒度是格）：真仓与 --staged 下整道跑时（不是样本、没给 --check、--item），每格各问一次 research/scripts/stage-must-run.sh，
# 键是 <本阶段文件名>:<格名>（.claude/gate.d/stage-inputs.tsv 里每格一行）：那一格登记的输入（Verus 四格另加 environment= 的 --tool-versions 那一行）没变、上次判绿作数
# （全绿标记，或上次整轮全绿那棵暂存树）就复用，只跑输入变了的格；六格都复用就整道退 77。判定的退出码只认 1 为可跳过、0 为要跑，别的非 0 判红（同 research/scripts/stage-run-or-skip.sh）。
# 要跑的格里再按条复用（文件头「按条复用」）。
reused_cells=()
if ((sample_mode == 0 && STAGE_CELLS_CHECK_GIVEN == 0)); then
  must_run_cells=()
  reuse_lines=()
  for cell_name in "${STAGE_CELLS_SELECTED[@]}"; do
    cell_reason="$(bash "$STAGE_REPOSITORY/research/scripts/stage-must-run.sh" "$ROOT" "$(basename "$0"):$cell_name")"; cell_reuse_exit=$?
    case "$cell_reuse_exit" in
      0) must_run_cells+=("$cell_name"); reuse_lines+=("  $cell_name：重跑（$cell_reason）") ;;
      1) reused_cells+=("$cell_name"); reuse_lines+=("  $cell_name：复用（$cell_reason）") ;;
      *) echo "  ✗ 判不出格 $cell_name 要不要跑：research/scripts/stage-must-run.sh 退 $cell_reuse_exit（它只许退 0 要跑、1 可跳过），这一道按红记"
         echo "     → 怎么办：单独执行 bash research/scripts/stage-must-run.sh $ROOT $(basename "$0"):$cell_name 看它停在哪；文件不在或起不来就先修它，修好之前这一道一直红，不退 77。"
         exit 1 ;;
    esac
  done
  echo "── 按格复用判定：${#STAGE_CELLS_SELECTED[@]} 格里 ${#reused_cells[@]} 格复用、${#must_run_cells[@]} 格重跑（每格的输入登记在 .claude/gate.d/stage-inputs.tsv，键 $(basename "$0"):<格名>）"
  printf '%s\n' "${reuse_lines[@]}"
  if ((${#must_run_cells[@]} == 0)); then
    [[ -n "${GATE_NOT_RUN_FILE:-}" ]] && printf '%s 每一格都复用上一次判绿的判定（各格的输入与 Verus 版本都没变），本次未跑\n' "$(basename "$0")" >> "$GATE_NOT_RUN_FILE"
    echo "  ! 本阶段跳过（每一格都复用上一次判绿的判定）"
    echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；每格读哪几条路径、Verus 的版本怎么进判定见 .claude/gate.d/stage-inputs.tsv。"
    exit 77
  fi
  STAGE_CELLS_SELECTED=("${must_run_cells[@]}")
  STAGE_CELLS_SELECTION_SOURCE="按格复用：${#reused_cells[@]} 格复用，只跑输入变了的 ${#must_run_cells[@]} 格"
fi

# 本次未跑的格先收进自己的文件：有一格本次未跑就不记环境、不写全绿标记（没判全的一趟不算上次成功），再原样转给 $GATE_NOT_RUN_FILE
not_run_record="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
# 一道里只排一次：Verus 那几格第一次拿到内存包装的 252 就往这份文件写一行（verus-proof-check.py 的 QueueRefusal），之后的格读到就不再排队
queue_refused_record="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; rm -f "${not_run_record:?}"; exit 1; }
# 按条的现判、复用数：判法每格往这份文件追加一行「格<TAB>现判数<TAB>复用数<TAB>原因分栏」（FORMAL_ITEM_TALLY_FILE），跑完整道相加
item_tally_record="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; rm -f "${not_run_record:?}" "${queue_refused_record:?}"; exit 1; }
for ((verdict_round = 1; verdict_round <= verdict_rounds; verdict_round++)); do
  ((verdict_rounds > 1)) && echo "── 第 $verdict_round 趟（样本 .formal-sample/run-twice：判定库 $verdict_library，第二趟应当全复用）"
  : > "$item_tally_record"
  if ( export GATE_NOT_RUN_FILE="$not_run_record" VERUS_PROOF_CHECK_QUEUE_REFUSED_FILE="$queue_refused_record" FORMAL_ITEM_TALLY_FILE="$item_tally_record"; stage_cells_run "$ROOT" ); then stage_exit=0; else stage_exit=$?; fi
done
if [[ -s "$item_tally_record" ]]; then
  judged_total=0; reused_total=0; tally_cells=0
  while IFS=$'\t' read -r tally_cell tally_judged tally_reused tally_reasons; do
    tally_cells=$((tally_cells + 1))
    judged_total=$((judged_total + tally_judged)); reused_total=$((reused_total + tally_reused))
    echo "  $tally_cell：现判 $tally_judged、复用 $tally_reused${tally_reasons:+（现判的原因：$tally_reasons）}"
  done < "$item_tally_record"
  echo "── 按条（$tally_cells 格有逐项判定；verus-toolchain-pinned 与 pinned-red-examples-complete 每趟现判、不复用）：现判 $judged_total、复用 $reused_total"
fi
rm -f "${item_tally_record:?}"
((verdict_library_is_temporary)) && rm -rf "${verdict_library:?}"
if [[ -n "${GATE_NOT_RUN_FILE:-}" && -s "$not_run_record" ]]; then cat "$not_run_record" >> "$GATE_NOT_RUN_FILE"; fi
if [[ -s "$queue_refused_record" ]]; then
  IFS=$'\t' read -r refused_where refused_howto < "$queue_refused_record"
  echo "  ✗ 这一道没跑成：内存包装排不上（$refused_where），之后的格没再排队，整道只等了一次"
  echo "     → 怎么办：${refused_howto:-照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办}；空出来之后整道重跑，不改证明、核、必红世界表或实现"
fi
rm -f "${queue_refused_record:?}"
if ((${#reused_cells[@]})); then
  echo "── 汇总（按格复用）：${#reused_cells[@]} 格复用上一次判绿的判定，本次没跑"
  for cell_name in "${reused_cells[@]}"; do echo "  $cell_name：复用"; done
fi
if ((stage_exit == 0 && sample_mode == 0 && STAGE_CELLS_CHECK_GIVEN == 0)) && [[ ! -s "$not_run_record" ]]; then
  # 跑了的每一格各记一份 Verus 版本（登记了 environment= 的那四格）与全绿标记，键是 <阶段>:<格名>；复用的格不动（它们的标记还在）。写不成只报不红：后果只是下一趟这一格多跑
  for cell_name in "${STAGE_CELLS_SELECTED[@]}"; do
    if ! python3 "$ADMISSION_MODULE" gate-record-environment "$ROOT" "$(basename "$0"):$cell_name"; then
      echo "  ! 没记下格 $cell_name 这一次的 Verus 版本（admission.py gate-record-environment 没成）：下一趟整轮这一格照跑，不会复用"
    fi
    if ! python3 "$ADMISSION_MODULE" stage-marker-write "$ROOT" "$(basename "$0"):$cell_name" >/dev/null; then
      echo "  ! 没写成格 $cell_name 的全绿标记（admission.py stage-marker-write 没成）：下一趟这一格照跑，不会复用"
    fi
  done
fi
rm -f "${not_run_record:?}"
exit "$stage_exit"
