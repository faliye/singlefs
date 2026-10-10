#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的 Rust 源码、测试文件名、kb 文本、登记表与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 代码类的源码纪律：名字不只由空泛词拼成、测试文件不按里程碑起名；kb 登记的格式常量、条文集合与 feature bit 位号与代码逐个对得上；实验二进制有成形的变异表且锚点唯一、有钉绝对值的断言、种子不折叠与读数不恒为字面量 0；crates/ 里放开 unsafe 的地方与登记表逐处对上；crates 变异表的轻型登记在改坏对象在 crates/singlefs-core/src/ 下的行里每个测试目标一行；不走 GPU 的计算与豁免表逐项对上；崩溃点重放的按段登记成形
# gate-category: 代码类
# gate-similar: doc-text.sh 也是全仓扫写法，但判的是登记过的禁用写法与改过名的术语的旧名，对象是全部文本文件；这里的 vague-names 与 test-file-names 两格判 .rs 里声明的名字由哪些词拼成、测试文件叫什么
# gate-similar: harness-model-differential-and-scenarios.sh 也调 research/scripts/crash-case-check.py，但调的是 one-scenario 那一样、判用例里面的循环，归 harness 类；这里的 test-file-names 格只调 file-names 那一样，判的是文件名
# gate-similar: .claude/singlefs-ai-sop/scripts/naming-lint.sh 是上游的命名纪律，判单字母与常见缩写、词表写死在上游；空泛词表是本项目定的，上游这一批不发版，先放项目本地
# gate-similar: doc-registries.sh 它判欠账表的形状、里程碑收口表收全开着的欠账号、总审核记录每行有去向，读 kb 与 records/；这里的 kb 对账三格拿登记表把决策分项正文里的名字与 Rust 枚举变体配对、把 format-const 标记的值与源码 const 比、把 feature bit 位号与代码常量比，reading-discipline 格按同一份 lib-owed.py 只判两张豁免表挂的欠账号还开着，对象是实验二进制源码；它判 kb 里字段表、字节布局表、段序列登记表与里程碑各步之间的对账，field-table-sums 那一格也经 lib-format-const.py 读标记，判的是字段表加和与结构总宽，不读代码；这里的 format-constants 格拿标记的值与 research/、crates/ 源码里的 const 比、旧值字面串不许留，不做算术；它的 declared-counts 那一格也按 lib-owed.py 读欠账表，只数得出欠着多少条；这里只拿开着的编号核豁免表
# gate-similar: checker-independence-and-sync.sh 它也经 lib-format-const.py 读标记、按 lib-owed.py 判布局滞后登记表点名的欠账号还开着，看的是这次改动里布局常量变了 checker 跟没跟；这里看此刻两边的值相等，对象是实验二进制源码与变异表
# gate-similar: checker-tier-crates-mutation-replay.sh 它按 crates/mutations.tsv 逐条改坏源码再跑点名的测试，要编译、跑几个钟头；这里的 mutation-tables 格经同一个模块 research/scripts/crates-mutation-rows.py 的 static-check 只做静态判，不跑变异
# gate-similar: doc-experiments.sh 它的 evidence-in-repo 格看这一轮改过的装置与变异表在 research/results/ 里有没有一份不比它旧的产物；这里判变异表本身成不成形、锚点对不对，与实验源码里的断言和读数，不看产物
#
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就十二格按下面的次序全跑，格名写错退 2）：
# gate-cell: vague-names 函数名与类型名不只由空泛词拼成
# gate-cell: test-file-names 测试文件名不按里程碑、步号起
# gate-cell: format-constants 格式常量在 kb 与实验源码之间同步
# gate-cell: clause-enums 条文列出来的封闭集合，与代码里兑现它的那个枚举逐个成员对上
# gate-cell: feature-bits feature bit 位号：记账表与 D15 登记表逐行一致、不跳号，代码引用的位都在记账表里
# gate-cell: mutation-tables 每个实验二进制都有成形的变异表、锚点唯一，crates/mutations.tsv 的静态几样
# gate-cell: absolute-assertions 每个实验二进制都有钉绝对值的断言
# gate-cell: reading-discipline 实验源码的两条读数纪律：种子不许折叠，读数不许恒为字面量 0
# gate-cell: unsafe-allowances crates/ 里放开 unsafe 的地方与登记表逐处对上（core 恰好打洞那一处、别的 crate 零处）
# gate-cell: mutation-light-rows crates/mutations.tsv 里改坏对象在 crates/singlefs-core/src/ 下的行里每个测试目标在轻型变异登记表里恰好一行、点名的行都在变异表里
# gate-cell: gpu-exemptions 不走 GPU 的计算与豁免表逐项对上：代码里每一处 // gpu-exempt: <截名> 都有表行、每一行都有标记
# gate-cell: crash-point-replay-segments 崩溃点重放的按段登记成形：项名在合表里、段号与段写数是非负整数且段写数大于 0、摘要是 64 位小写十六进制、同一项同一段只一行
#   vague-names          词表 .claude/naming-vague-words，还没改完的文件 .claude/naming-vague-exclude
#   test-file-names      跑 research/scripts/crash-case-check.py --only file-names
#   format-constants     调 lib-format-const.py；冻结证据登记在 .claude/format-constants-exclude（一行 <路径>  # 理由）
#   clause-enums         按 .claude/gate.d/clause-enum-pairs.tsv 配对
#   feature-bits         .claude/kb/feature-bits.md 与 D15（格式冻结政策） 已定项 4
#   mutation-tables      research/mutations/ 与 crates/mutations.tsv（后者经 research/scripts/crates-mutation-rows.py static-check）
#   absolute-assertions  research/e7-index-bench/src/bin 下全部，别处 src/bin 下以 e<数字>_ 开头的
#   reading-discipline   C59 种子折叠、C60 恒为字面量 0 的读数，两张豁免表挂开着的欠账
#   unsafe-allowances    .claude/gate.d/unsafe-allowances.tsv 与 crates/ 下全部 .rs
#   mutation-light-rows  .claude/gate.d/crates-mutation-light-rows.tsv 与 crates/mutations.tsv（经 research/scripts/crates-mutation-light-rows.py check）
#   gpu-exemptions       .claude/gate.d/gpu-exemptions.tsv 与 crates/ 下全部 .rs（经 research/scripts/gpu-exemptions-check.py check）
#   crash-point-replay-segments  .claude/gate.d/crash-point-replay-segments.tsv 与合表 .claude/gate.d/crash-point-replay-items.tsv（经 research/scripts/lib_crash_point_replay_items.py segments）
# 每格的判据、射程、管不到的写在它自己那一段注释里。
# 参数解析、样本根的 .gate-cells、每格一个子 shell、汇总与退出码在共用库 lib/stage-cells.sh。
# 规则：.claude/rules/verification.md「门禁的结构」「门禁管哪一半」。
#
# 样本：fixtures/code-source-discipline.sh/ 下每格至少一对 <格名>-red / <格名>-green，每份样本根的 .gate-cells 只点名那一格；
# 各格注释里说的 red / green 就是那一格自己的样本。
# 弄坏开关。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（fixtures/code-source-discipline.sh/ 下的目录名，
# 括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   STAGE_CELLS_BREAK=red-swallowed  共用库吞红：红格按绿记、整道退 0
#       absolute-assertions-red、clause-enums-red、crash-point-replay-segments-red、feature-bits-red、format-constants-exclusion-red、format-constants-red、gpu-exemptions-red、mutation-light-rows-red、
#       mutation-tables-red、reading-discipline-red、test-file-names-red、
#       unsafe-allowances-no-safety-red、unsafe-allowances-other-crate-red、unsafe-allowances-reason-empty-red、unsafe-allowances-root-blank-red、
#       unsafe-allowances-safety-vague-red、unsafe-allowances-source-missing-red、unsafe-allowances-two-allows-red、unsafe-allowances-wrong-function-red、
#       vague-names-red（期望退出 1，实测 0）
#   STAGE_CELLS_BREAK=marker-ignored  共用库不认 .gate-cells：每份样本十二格都跑
#       absolute-assertions-green、clause-enums-green、crash-point-replay-segments-green、feature-bits-green、format-constants-green、gpu-exemptions-green、mutation-light-rows-green、
#       mutation-tables-green、reading-discipline-green、
#       test-file-names-green、unsafe-allowances-deny-one-green、unsafe-allowances-green、vague-names-green（期望退出 0，实测 1）；absolute-assertions-red、
#       clause-enums-red、crash-point-replay-segments-red、feature-bits-red、format-constants-exclusion-red、format-constants-red、gpu-exemptions-red、mutation-light-rows-red、mutation-tables-red、
#       reading-discipline-red、test-file-names-red、
#       unsafe-allowances-no-safety-red、unsafe-allowances-other-crate-red、unsafe-allowances-reason-empty-red、unsafe-allowances-root-blank-red、
#       unsafe-allowances-safety-vague-red、unsafe-allowances-source-missing-red、unsafe-allowances-two-allows-red、unsafe-allowances-wrong-function-red、
#       vague-names-red（输出里找不到「样本只跑 .gate-cells 点名的格：…」）
#   GATE_VAGUE_NAMES_BREAK=trait-methods-judged  vague-names 把 trait 块里的方法也判
#       vague-names-green（期望退出 0，实测 1）
#   GATE_VAGUE_NAMES_BREAK=exclusions-ignored  vague-names 不认排除表
#       vague-names-red（输出里找不到「crates/demo/src/renamed.rs 里已经…」）
#   CRASH_CASE_CHECK_BREAK=milestone-names-unseen  test-file-names 认不出按里程碑起的名
#       test-file-names-red（期望退出 1，实测 0）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=format-constants  不比源码 const 的值与 kb 的现行值
#       format-constants-red（输出里找不到「const SAMPLE_HDR = 84」）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=format-constants-exclusion-ignored  排除表照读照判，但登记的路径照样扫
#       format-constants-green（期望退出 0，实测 1）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=format-constants-ignored-files-scanned  按盘上有什么扫，不按 git 列出的扫：被 .gitignore 忽略的编译副本也判
#       format-constants-green（期望退出 0，实测 1）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=format-constants-exclusion-missing-path-unjudged  排除表里指向不存在的路径不判
#       format-constants-exclusion-red（输出里找不到「.claude/format-constants-exclude:1 指向不存在的路径 research/prompts/gone/」）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=format-constants-exclusion-no-match-unjudged  排除表里一份文件都没排到的路径不判
#       format-constants-exclusion-red（输出里找不到「.claude/format-constants-exclude:2 research/target/copy/ 下一份被 git 列出的 .rs 都没排到」）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=format-constants-exclusion-reason-unjudged  排除表里没写理由的行不判
#       format-constants-exclusion-red（输出里找不到「.claude/format-constants-exclude:3 research/frozen.rs 没写为什么排除」）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=clause-enums  表里的名字按子串认作正文列举的一项
#       clause-enums-red（输出里找不到「表里的「乙」不是 D99 已定项 1 正文那串列举里的一项」）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=feature-bits  不判同一个（类别，位号）登记两行
#       feature-bits-red（输出里找不到「记账表里同一个位号登记了不止一行」）
#   REJECTION_BLOCKS_BREAK=summaries-only  reading-discipline 逐块打印拒绝只打摘要（共用 lib/rejection-blocks.py）
#       reading-discipline-red（输出里找不到「crates/singlefs-checker-tier/s…」）
#   GATE_UNSAFE_ALLOWANCES_BREAK=count-ignored  unsafe-allowances ② 的个数不数
#       unsafe-allowances-other-crate-red（输出里找不到「singlefs-harness 放开 unsafe_cod…」）；unsafe-allowances-two-allows-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=placement-ignored  unsafe-allowances ③ 的位置不判
#       unsafe-allowances-other-crate-red（输出里找不到「singlefs-harness 在登记表里没有放开」）；unsafe-allowances-wrong-function-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=safety-comment-ignored  unsafe-allowances ④ 不要 // SAFETY: 注释
#       unsafe-allowances-no-safety-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=root-level-ignored  unsafe-allowances ① 不看根写了 forbid 还是 deny
#       unsafe-allowances-root-blank-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=other-crates-ignored  unsafe-allowances 登记表里没有放开的 crate 整个不判
#       unsafe-allowances-deny-one-green、unsafe-allowances-green（输出里找不到「crates/ 里放开 unsafe 的地方与登记表对上：扫…」）；
#       unsafe-allowances-other-crate-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=reason-ignored  unsafe-allowances 不判登记行的理由栏
#       unsafe-allowances-reason-empty-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=source-ignored  unsafe-allowances 不判登记行的出处栏
#       unsafe-allowances-source-missing-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=safety-premise-ignored  unsafe-allowances ④ 不判 SAFETY 注释写没写前提
#       unsafe-allowances-safety-vague-red（期望退出 1，实测 0）
#   mutation-light-rows 与 gpu-exemptions 两格的判法开关在各自的判法脚本里（文件头列全、--selftest 证红）；这一道里打开时判错的样本：
#   CRATES_MUTATION_LIGHT_ROWS_BREAK=missing-targets-accepted  轻型变异登记表缺测试目标不红
#       mutation-light-rows-red（输出里找不到「在轻型登记表里没有一行」）
#   CRATES_MUTATION_LIGHT_ROWS_BREAK=unknown-rows-accepted  点名的变异名不在变异表里不红
#       mutation-light-rows-red（输出里找不到「不是 crates/mutations.tsv 里成形的一行的变异名」）
#   CRATES_MUTATION_LIGHT_ROWS_BREAK=non-core-rows-accepted  登记了改坏对象不在 crates/singlefs-core/src/ 下的行不红
#       mutation-light-rows-red（输出里找不到「不在 crates/singlefs-core/src/ 下」）
#   CRATES_MUTATION_LIGHT_ROWS_BREAK=non-core-targets-required  核心行之外的测试目标也要登记一行（旧判法）
#       mutation-light-rows-green（期望退出 0，实测 1）
#   GPU_EXEMPTIONS_CHECK_BREAK=unregistered-markers-accepted  代码里的标记没有表行不红
#       gpu-exemptions-red（输出里找不到「豁免表里没有 continuation 这一行」）
#   GPU_EXEMPTIONS_CHECK_BREAK=unmarked-rows-accepted  豁免表的行在代码里没有标记不红
#       gpu-exemptions-red（输出里找不到「second-crash-during-mount 在 crates/ 里一处标记都没有」）
#   crash-point-replay-segments 格的判法开关在 research/scripts/lib_crash_point_replay_items.py（文件头列全，research/scripts/crash-case-check.py --selftest 证红）；这一道里打开时判错的样本：
#   CRASH_POINT_REPLAY_ITEMS_BREAK=segment-unknown-item-accepted  按段登记的项名不在合表里不红
#       crash-point-replay-segments-red（输出里找不到「项名 ghost-item 不是合表」）
#   CRASH_POINT_REPLAY_ITEMS_BREAK=segment-numbers-unchecked  按段登记的段号与段写数不核
#       crash-point-replay-segments-red（输出里找不到「段号「01」不是非负整数」）
#   CRASH_POINT_REPLAY_ITEMS_BREAK=segment-digest-unchecked  按段登记的摘要不核
#       crash-point-replay-segments-red（输出里找不到「不是 64 位小写十六进制」）
#   CRASH_POINT_REPLAY_ITEMS_BREAK=segment-duplicate-accepted  按段登记同一项同一段两行不红
#       crash-point-replay-segments-red（输出里找不到「已在第 」）
# mutation-tables、absolute-assertions 没有弄坏开关，reading-discipline 只有共用库那一项，判别力靠各自的样本。
#
#   bash .claude/gate.d/code-source-discipline.sh [项目根]                                十二格都跑
#   bash .claude/gate.d/code-source-discipline.sh --list                                  逐行打格名与判什么，不跑格
#   bash .claude/gate.d/code-source-discipline.sh --check <格名>[,<格名>…] [项目根]         只跑点名的格
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_NAME="$(basename "$0")"
STAGE_DIRECTORY="$(cd "$(dirname "$0")" && pwd)"
# 共用模块在这份阶段所在的仓里（样本目录里没有 research/scripts/）：cd 进被判的仓之前取
STAGE_REPOSITORY="$(cd "$STAGE_DIRECTORY/../.." && pwd)"
CRASH_CASE_CHECK="$STAGE_REPOSITORY/research/scripts/crash-case-check.py"
FORMAT_CONST_LIBRARY="$STAGE_DIRECTORY/lib-format-const.py"
OWED_LIBRARY="$STAGE_DIRECTORY/lib-owed.py"
source "$STAGE_DIRECTORY/lib/stage-cells.sh"
bad() { printf '  ✗ %s\n' "$*"; }
howto() { printf '     → %s\n' "$*"; }

stage_cell vague-names cell_vague_names "函数名与类型名不只由空泛词拼成" \
  "按 .claude/singlefs-ai-sop/rules/code-discipline.md「名字：光看名字，就说得出它是什么、做什么」改名；一时改不完的文件登记进 .claude/naming-vague-exclude（一行 <路径>  # 理由），排除表里指向不存在的与已经改完的行删掉，排除只缩不涨"
stage_cell test-file-names cell_test_file_names "测试文件名不按里程碑、步号起" \
  "照 crash-case-check 那一句出路按内容改名（领域在前、场景在后）；判法写在 research/scripts/crash-case-check.py 文件头，规则在 .claude/rules/verification.md「定义与名字」"
stage_cell format-constants cell_format_constants "格式常量在 kb 与实验源码之间同步" \
  "权威是 kb 里的 format-const 标记：改常量三处一起动（kb 标记与正文、实验源码的 const 与钉死它的单测、重跑实验并更新 research/results/ 的产物），旧值只许留在「## 历史版本」之后与 *-history.md 里；原样保存的冻结证据改不了，登记进 .claude/format-constants-exclude（一行 <路径>  # 理由），排除表里指向不存在的与排不到文件的行删掉"
stage_cell clause-enums cell_clause_enums "条文列出来的封闭集合，与代码里兑现它的那个枚举逐个成员对上" \
  "先判哪一边错了：代码加了成员就写回那条分项正文、再往 .claude/gate.d/clause-enum-pairs.tsv 补一行；条文才是对的就改代码；别只改表让两边看着一致，改分项正文照 .claude/rules/format-evolution.md 记进变更史"
stage_cell feature-bits cell_feature_bits "feature bit 位号：记账表与 D15 登记表逐行一致、不跳号，代码引用的位都在记账表里" \
  "位号的唯一登记位是 D15（格式冻结政策） 已定项 4 那张表：两处不一致改 .claude/kb/feature-bits.md 的记账表；每张位图从 0 严格递增不跳号；代码新引用的位先在两处登记"
stage_cell mutation-tables cell_mutation_tables "每个实验二进制都有成形的变异表、锚点唯一，crates/mutations.tsv 的静态几样" \
  "缺表的写 research/mutations/<bin名>.tsv；锚点改到今天源码里逐字存在、只出现一次的那一段；crates/mutations.tsv 照上面各条的出路改（六段、只许 \\n 转义、点名的测试跑得到、改的文件在门禁 checker-tier-crates-mutation-replay 的拷贝范围里），改完单跑那几行证明点名的测试红"
stage_cell absolute-assertions cell_absolute_assertions "每个实验二进制都有钉绝对值的断言" \
  "给它加一条把被量的那个数钉死的断言，绝对值由独立算术给出、不许从代码里读回来；加完用 research/scripts/mutate.sh 证明它会红"
stage_cell reading-discipline cell_reading_discipline "实验源码的两条读数纪律：种子不许折叠，读数不许恒为字面量 0" \
  "种子先过一次乘法混淆再置位（seed.wrapping_mul(0xD1B5_4A32_D192_ED03) | 1），恒为 0 的字段接上真读数；动不了的存量登记进 .claude/gate.d/experiment-seed-fold-lag.tsv / experiment-constant-reading-lag.tsv，挂 .claude/kb/checks-owed.md 里开着的 C59 / C60，只缩不涨"
stage_cell unsafe-allowances cell_unsafe_allowances "crates/ 里放开 unsafe 的地方与登记表逐处对上（core 恰好打洞那一处、别的 crate 零处）" \
  "先看红的是哪一条：登记行的理由栏或出处栏缺、根没写 forbid / deny、多出的 allow(unsafe_code) 或 unsafe 关键字、放开的位置不是登记的函数、unsafe 上方缺 // SAFETY: 或没写出前提、tests/ 里写了 unsafe。没理由的放开删掉；确要新增一处，改 .claude/gate.d/unsafe-allowances.tsv 的登记并走三方（.claude/rules/three-way-inference.md），不改这一格的判法去迁就"
stage_cell mutation-light-rows cell_mutation_light_rows "crates/mutations.tsv 里改坏对象在 crates/singlefs-core/src/ 下的行里每个测试目标在轻型变异登记表里恰好一行、点名的行都在变异表里" \
  "照上面逐处列的改 .claude/gate.d/crates-mutation-light-rows.tsv（写法与挑法在它的表头）：缺测试目标的照那一句补行，点名的行被删或改名的换成同一个测试目标剩下行号最小的那一条；改完单跑 python3 research/scripts/crates-mutation-light-rows.py check 到它退 0"
stage_cell gpu-exemptions cell_gpu_exemptions "不走 GPU 的计算与豁免表逐项对上：代码里每一处 // gpu-exempt: <截名> 都有表行、每一行都有标记" \
  "代码里不走 GPU 的那一截在函数上一行写 // gpu-exempt: <截名>，豁免表 .claude/gate.d/gpu-exemptions.tsv 登记同名一行（每加一行先问用户点头）；表里一处标记都没有的行，那一截上了 GPU 就删掉、没上就补标记；改完单跑 python3 research/scripts/gpu-exemptions-check.py check 到它退 0"
stage_cell crash-point-replay-segments cell_crash_point_replay_segments "崩溃点重放的按段登记成形：项名在合表里、段号与段写数是非负整数且段写数大于 0、摘要是 64 位小写十六进制、同一项同一段只一行" \
  "照上面逐处列的改 .claude/gate.d/crash-point-replay-segments.tsv（写法在它的表头）：项名照合表 .claude/gate.d/crash-point-replay-items.tsv 写，段号与段写数写十进制整数、段写数大于 0，摘要写那一段写表的 sha256（64 位小写十六进制），同一项同一段只留一行；文件不在就照表头建回来；改完单跑 python3 research/scripts/lib_crash_point_replay_items.py segments 到它退 0"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

root_argument=""
while (($#)); do
  case "$1" in
    -*)
      bad "认不出的选项 $1"
      howto "只认 --list、--check <格名>[,<格名>…] 与一个项目根；格名用 bash .claude/gate.d/code-source-discipline.sh --list 看。"
      exit 2 ;;
    *)
      if [[ -n "$root_argument" ]]; then
        bad "给了两个项目根：$root_argument 与 $1"
        howto "参数只认一个项目根；不给项目根就取这个脚本往上两级。"
        exit 2
      fi
      root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || { bad "进不去项目根 $ROOT"; howto "第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }
ROOT="$(pwd)"

# ── vague-names ──
# 判据：`git ls-files` 列出的 crates/**/*.rs 与 research/e7-index-bench/src/**/*.rs 里，每一处 `fn <名>` 与 `struct|enum|type|trait <名>`：
# 函数名按 `_` 切词、类型名按大写字母切词，切出来的每一个词（小写）都在 .claude/naming-vague-words 里，判违规，逐处列「文件:行 名字」。
# 不判的：`impl <trait> for <类型>` 块里的方法（名字是 trait 定的）、`main`、宏展开出来的名字（认不出）。
# 还没改完名的文件逐个登记在 .claude/naming-vague-exclude（一行 <路径>  # 理由），那份文件里的违规不判；排除项指向不存在的文件、
# 或那份文件里已经一处违规都没有了，判红（排除只缩不涨）。一份 .rs 都没扫到：这一格本次无对象可判。
# 管不到的：名字里夹了一个非空泛词、而那个词本身也什么都没说（run_the_cell_again）——词表只认整名全空泛，这一类靠 review。
cell_vague_names() {
python3 - <<'PY'
import os, re, subprocess, sys

WORDS_FILE = ".claude/naming-vague-words"
EXCLUDE_FILE = ".claude/naming-vague-exclude"
BROKEN = os.environ.get("GATE_VAGUE_NAMES_BREAK", "")

def registry(path):
    entries = []
    if not os.path.isfile(path):
        return None
    for number, line in enumerate(open(path, encoding="utf-8"), 1):
        body = line.split("#", 1)[0].strip()
        if body:
            entries.append((number, body, line.split("#", 1)[1].strip() if "#" in line else ""))
    return entries

words = registry(WORDS_FILE)
if words is None:
    print(f"  ✗ 读不到空泛词表 {WORDS_FILE}")
    print("     → 怎么办：从 git 恢复它；这一格没有词表就什么都判不了，不是通过。")
    sys.exit(1)
vague = {word.lower() for _number, word, _reason in words}
excluded = registry(EXCLUDE_FILE) or []

listed = subprocess.run(["git", "-c", "core.quotepath=false", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", "crates", "research/e7-index-bench/src"],
                        capture_output=True)
paths = sorted(path for path in listed.stdout.decode("utf-8", "replace").split("\0") if path.endswith(".rs") and os.path.isfile(path))
if not paths:
    print("  ! 本次无对象可判：crates/ 与 research/e7-index-bench/src/ 下一份 .rs 都没列出来")
    sys.exit(77)

DECLARATION = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+|async\s+|unsafe\s+|extern\s+\"[^\"]*\"\s+)*(fn|struct|enum|type|trait)\s+([A-Za-z_][A-Za-z0-9_]*)")
TRAIT_IMPL = re.compile(r"^\s*impl\b.*\bfor\b")

def words_of(kind, name):
    if kind == "fn":
        return [part.lower() for part in name.split("_") if part]
    return [part.lower() for part in re.findall(r"[A-Z][a-z0-9]*|[a-z0-9]+", name)]

def violations_in(path):
    found = []
    depth = 0
    trait_impl_depths = []
    for number, line in enumerate(open(path, encoding="utf-8", errors="replace"), 1):
        code = line.split("//", 1)[0]
        if TRAIT_IMPL.match(code) and BROKEN != "trait-methods-judged":
            trait_impl_depths.append(depth)
        match = DECLARATION.match(code)
        inside_trait_impl = bool(trait_impl_depths) and depth > trait_impl_depths[-1]
        if match and not (match.group(1) == "fn" and inside_trait_impl) and match.group(2) != "main":
            parts = words_of(match.group(1), match.group(2))
            if parts and all(part in vague for part in parts):
                found.append(f"{path}:{number} {match.group(1)} {match.group(2)}")
        depth += code.count("{") - code.count("}")
        while trait_impl_depths and depth <= trait_impl_depths[-1] and "}" in code:
            trait_impl_depths.pop()
    return found

excluded_paths = {entry for _number, entry, _reason in excluded}
problems, exclusion_problems = [], []
scanned = 0
for path in paths:
    scanned += 1
    found = violations_in(path)
    if path in excluded_paths and BROKEN != "exclusions-ignored":
        if not found:
            exclusion_problems.append(f"{EXCLUDE_FILE}：{path} 里已经一个空泛名都没有了，这一行删掉")
        continue
    problems += found
for number, entry, reason in excluded:
    if not os.path.isfile(entry):
        exclusion_problems.append(f"{EXCLUDE_FILE}:{number} 指向不存在的文件 {entry}")
    elif len(reason) < 4:
        exclusion_problems.append(f"{EXCLUDE_FILE}:{number} {entry} 没写为什么先不改（# 后面至少 4 个字）")

for problem in problems:
    print(f"  ✗ {problem}")  # gate-lint:detail
if problems:
    print(f"  ✗ {len(problems)} 个函数名或类型名只由空泛词拼成（每个词都在 {WORDS_FILE} 里）")  # gate-lint:summary
    print("     → 怎么办：按 .claude/singlefs-ai-sop/rules/code-discipline.md「名字：光看名字，就说得出它是什么、做什么」改名：说出跑的是什么、判的是哪条、取的是哪个量；")
    print(f"                一时改不完的文件登记进 {EXCLUDE_FILE}（一行 <路径>  # 理由），改完删那一行。")
for problem in exclusion_problems:
    print(f"  ✗ {problem}")  # gate-lint:detail
if exclusion_problems:
    print(f"  ✗ {EXCLUDE_FILE} 有 {len(exclusion_problems)} 行不起作用或没写理由")  # gate-lint:summary
    print("     → 怎么办：指向不存在的与已经改完的那几行删掉；留着的每一行写明为什么先不改。排除只缩不涨。")
if problems or exclusion_problems:
    sys.exit(1)
print(f"  ✓ 没有只由空泛词拼成的函数名与类型名（扫了 {scanned} 份 .rs，词表 {len(vague)} 个词，排除 {len(excluded)} 份还没改完的文件）")
PY
}

# ── test-file-names ──
# 判据：research/scripts/crash-case-check.py --only file-names <项目根>：crates/singlefs-harness/tests/*.rs 与 crates/singlefs-checker-tier/tests/*.rs
# 的文件名不带里程碑序数、步号、增补号、并行线号、欠账号（那份脚本的 MILESTONE_FILE_NAME），判法只在那一份里。
# 两个 tests/ 下一份 .rs 都没有：这一格本次无对象可判。
# 管不到的：按里程碑起、却不是那几种写法的名字（`milestone_three_*` 这类），靠 review；别的包的测试文件不扫。
cell_test_file_names() {
  local file_names_exit
  if python3 "$CRASH_CASE_CHECK" --only file-names "$ROOT"; then file_names_exit=0; else file_names_exit=$?; fi
  case "$file_names_exit" in
    0|77) return "$file_names_exit" ;;
  esac
  echo "  ✗ 测试文件名：crash-case-check --only file-names 退 $file_names_exit（逐处列在上面）"
  echo "     → 怎么办：照上面那一句出路改名；判法写在 research/scripts/crash-case-check.py 文件头，规则在 .claude/rules/verification.md「定义与名字」。"
  return 1
}

# ── 格 format-constants：格式常量在 kb 与实验源码之间同步
#
# 权威在 kb 的 format-const 标记，紧挨着定这个值的那句话（文法与解析在 lib-format-const.py，doc-registries、checker-independence-and-sync 共用）：
#     <!-- format-const: JOURNAL_HEADER_BYTES = 86 stale=hdr=84|room=428 -->
# 判红：标记按文法读不出来（多写了键、值不是整数）；同一个名字登记两处（同一份文件里两次也算）；
# research/、crates/ 源码里同名 const 的值与标记不等（只认类型是单个标识符、值是整数字面量的 const，同名的数组扫描表不算）；
# stale= 列的旧值字面串还留在 kb 正文或源码里。旧值只许留在「## 历史版本」之后与 *-history.md 里。
# 扫哪些源码：`git ls-files --cached --others --exclude-standard` 列出的 research/、crates/ 下的 .rs（已跟踪的与还没跟踪、没被忽略的）；
# 被 .gitignore 忽略的编译副本不扫（例如 research/target/ 下的 crates 副本：本机有、干净的克隆里没有，扫它等于两台机器判出两个结果）。
# 冻结证据（research/prompts/ 下原样保存的模型产出）改了就不是原样（.claude/singlefs-ai-sop/rules/evidence-discipline.md「原样保存的证据不许事后改」），
# 登记在 .claude/format-constants-exclude（一行 <路径>  # 理由，目录写成以 / 结尾）：登记路径下的 .rs 既不比 const 的值、也不查旧值字面串。没有这个文件按空表算。
# 排除表判红：路径不存在；路径在、但没排到一份被 git 列出的 .rs；# 后面的理由不足 4 个字。排除只缩不涨：路径改名了、证据删了，那一行就删。
# 成功句报：扫了几份 .rs、排除表排除了几份。不在 git 仓里（git ls-files 起不来）：本次未跑，退 77。
# 为什么要这一格：checker-tier-research-build-and-replay 逐字节比产物，源码与产物一起停在旧值时它照样绿；doc-registries 只看 kb 内部。
# 射程：只管登记了标记的常量，没登记的新格式常量照样能漂，靠人。
# 样本：red 有源码落后于 kb 的常量、正文残留的旧值、多写了键的标记、同一份文件里登记两次的名字，以及值跨行写、带 pub(crate) 的两处落后声明；
# green 另放一张与格式常量同名的扫描表（数组类型），判绿。
# 排除表的样本：red 里放一份落后的冻结证据、不登记，判红；green 里同样一份登记进排除表，判绿，另有一份被 .gitignore 忽略的编译副本带着旧值，不扫；
# format-constants-exclusion-red 的排除表三行各错一样（路径不存在、目录被忽略而排不到文件、没写理由），三条都要点名。
cell_format_constants() {
  [[ -d .claude/kb ]] || { echo "  ! 找不到 kb，这一格跳过"; exit 77; }

  python3 - "$FORMAT_CONST_LIBRARY" <<'PY'
import glob, importlib.util, os, subprocess, sys

library_spec = importlib.util.spec_from_file_location("format_const", sys.argv[1])
format_const = importlib.util.module_from_spec(library_spec)
library_spec.loader.exec_module(format_const)
format_constants_break = os.environ.get("CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK", "")
break_value_comparison = format_constants_break == "format-constants"
EXCLUDE_FILE = ".claude/format-constants-exclude"

def body_of(text):
    """正文 = 「## 历史版本」之前那一段。历史里留旧值是文档纪律要求的，不判红。"""
    i = text.find('\n## 历史版本')
    return text if i < 0 else text[:i]

# *-history.md 不是登记位：变更史原样引用标记是历史陈述，不算第二处权威记录。
kb_all = sorted(glob.glob('.claude/kb/**/*.md', recursive=True))
kb_files = [f for f in kb_all if not f.endswith('-history.md') and '/decisions-history/' not in f]

# ---- 1. 收标记：读不出来的、同一个常量登记了不止一处的，判红 ----
marks, registrations, bad = {}, {}, []
unparsable_count = 0
for f in kb_files:
    parsed = format_const.parse_marks(open(f, encoding='utf-8').read())
    for unparsable in parsed.unparsable:
        unparsable_count += 1
        bad.append(f'{f}:{unparsable.line_number}  format-const 标记按文法读不出来：「{unparsable.excerpt}」')
    for mark in parsed.marks:
        registrations.setdefault(mark.name, []).append(f'{f}:{mark.line_number}')
        marks.setdefault(mark.name, (f, mark.value, list(mark.stale_literals)))

duplicate_count = 0
for name, places in registrations.items():
    if len(places) > 1:
        duplicate_count += 1
        bad.append(f'{name}：登记了 {len(places)} 处（{" 与 ".join(places)}）—— 同一个事实只许一处权威记录')

if not marks and not bad:
    print('  ! kb 里一个 format-const 标记都没有，这一格**什么也没验**')
    print('     → 在定住格式常量的那句话旁边加 <!-- format-const: 名字 = 值 stale=旧字面串 -->')
    sys.exit(1)

# ---- 2. 源码里的 const 定义必须等于 kb 的现行值 ----
# 格式常量模块住 crates/singlefs-format，同一套标记绑住它
if format_constants_break == "format-constants-ignored-files-scanned":
    listed_sources = glob.glob('research/**/*.rs', recursive=True) + glob.glob('crates/**/*.rs', recursive=True)   # 弄坏开关：按盘上有什么扫，被忽略的编译副本也进来
else:
    listing = subprocess.run(["git", "-c", "core.quotepath=false", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", "research", "crates"], capture_output=True)
    if listing.returncode != 0:
        print(f"  ⊘ 本次未跑：git ls-files 退 {listing.returncode}（{listing.stderr.decode('utf-8', 'replace').strip()[:200]}），分不出哪些 .rs 被 git 忽略")
        print("     → 在 git 仓里跑这一格；不是仓的目录（拷出来的树）判不了。")
        sys.exit(77)
    listed_sources = [path for path in listing.stdout.decode("utf-8", "replace").split("\0") if path.endswith(".rs") and os.path.isfile(path)]
listed_sources = sorted(set(listed_sources))

# 排除表：一行 <路径>  # 理由；目录写成以 / 结尾，路径等于它、或落在它下面的 .rs 都排除
exclusion_entries = []
if os.path.isfile(EXCLUDE_FILE):
    for number, line in enumerate(open(EXCLUDE_FILE, encoding='utf-8'), 1):
        entry_text, _, reason_text = line.partition('#')
        if entry_text.strip():
            exclusion_entries.append((number, entry_text.strip(), reason_text.strip()))
exclusion_problems, excluded_files = [], set()
for number, entry, reason in exclusion_entries:
    entry_base = entry.rstrip('/')
    matched = [path for path in listed_sources if path == entry_base or path.startswith(entry_base + '/')]
    if not os.path.exists(entry_base):
        if format_constants_break != "format-constants-exclusion-missing-path-unjudged":
            exclusion_problems.append(f'{EXCLUDE_FILE}:{number} 指向不存在的路径 {entry}')
    elif not matched:
        if format_constants_break != "format-constants-exclusion-no-match-unjudged":
            exclusion_problems.append(f'{EXCLUDE_FILE}:{number} {entry} 下一份被 git 列出的 .rs 都没排到（被 .gitignore 忽略、下面没有 .rs，或路径写错了）')
    if len(reason) < 4 and format_constants_break != "format-constants-exclusion-reason-unjudged":
        exclusion_problems.append(f'{EXCLUDE_FILE}:{number} {entry} 没写为什么排除（# 后面至少 4 个字）')
    excluded_files.update(matched)
if format_constants_break == "format-constants-exclusion-ignored":
    excluded_files = set()   # 弄坏开关：排除表照读照判，但登记的路径照样扫
srcs = [path for path in listed_sources if path not in excluded_files]
seen_in_src = set()
for f in srcs:
    source_text = open(f, encoding='utf-8', errors='ignore').read()
    for declaration in format_const.read_rust_consts(source_text, only_scalar_types=True):
        if declaration.name not in marks:
            continue
        name = declaration.name
        seen_in_src.add(name)
        # 值要读到分号为止：只取开头那段数字时，`16384 * 2` 读成 16384 判通过，`16 * 1024` 与 `16_384` 读成 16 判红。
        if declaration.value is None:
            shown = format_const.normalized_value_text(declaration.value_text)
            bad.append(f'{f}:{declaration.line_number}  const {name} 的值写成了「{shown}」，门禁读不出它等于几 → 写成整数字面量（可带 _ 分隔）')
            continue
        kbf, want, _ = marks[name]
        if break_value_comparison:
            continue   # 弄坏开关：不比源码 const 的值与 kb 的现行值
        if declaration.value != want:
            bad.append(f'{f}:{declaration.line_number}  const {name} = {declaration.value}，而 {kbf} 定的现行值是 {want}')

# ---- 3. 旧值的字面串不许留在正文与源码里 ----
# research/results/ 不扫：产物是那一轮的原始输出，不许事后改（.claude/singlefs-ai-sop/rules/evidence-discipline.md「原样保存的证据不许事后改」）；
# 源码改了而产物没重跑，由 checker-tier-research-build-and-replay 逐字节比对抓。
scan = [(f, format_const.strip_marks(body_of(open(f, encoding='utf-8').read()))) for f in kb_files]
scan += [(f, open(f, encoding='utf-8', errors='ignore').read()) for f in srcs]

for name, (kbf, want, stale) in sorted(marks.items()):
    for s in stale:
        for f, text in scan:
            if s in text:
                ln = text[:text.index(s)].count('\n') + 1
                bad.append(f'{f}:{ln}  还留着 {name} 的旧值字面串「{s}」（现行值 {want}）')

if exclusion_problems:
    for problem in exclusion_problems:
        print(f'  ✗ {problem}')
    print(f'  ✗ 排除表 {EXCLUDE_FILE} 有 {len(exclusion_problems)} 行不起作用或没写理由')
    print('     → 指向不存在的与排不到文件的那几行删掉（排除只缩不涨）；留着的每一行在 # 后面写明为什么不判（原样保存的证据、改了就不是原样）。')
if bad:
    print('  ✗ 格式常量在 kb 与实验源码之间对不上：')
    for b in bad[:20]:
        print(f'     {b}')
    if len(bad) > 20:
        print(f'     …… 另有 {len(bad)-20} 处')
    print('     → 权威是 kb 里的 format-const 标记。改常量要三处一起动：')
    print('       ① kb 标记与正文 ② 实验源码的 const 与钉死它的单测 ③ 重跑实验并更新 research/results/ 的产物')
    print('     → 旧值只许留在「## 历史版本」之后与 *-history.md 里。')
    if unparsable_count:
        print('     → 读不出来的标记照这一条文法改写：<!-- format-const: 名字 = 整数 stale=旧串|旧串 -->，')
        print('       stale= 之外不许有别的键；它读不出来时，这个常量在这一格眼里就没登记过。')
        print('       它只是正文里举的例子、不是登记，就去掉 <!--，写成「`format-const: 名字 = 值`」。')
    if duplicate_count:
        print('     → 一个名字只留一处登记（定这个值的那一处）；别处要提它，写成不带 <!-- 的文字，例如「`format-const: 名字`」。')
    sys.exit(1)
if exclusion_problems:
    sys.exit(1)

only_kb = sorted(set(marks) - seen_in_src)
print(f'  ✓ 格式常量同步（{len(marks)} 个已登记，{len(seen_in_src)} 个在源码里被钉住；扫了 {len(srcs)} 份 .rs，排除表排除了 {len(excluded_files)} 份）')
if only_kb:
    print(f'     ! 这些还没有任何实验源码用到，这一格对它们只验了唯一性：{", ".join(only_kb)}')
PY
}

# ── 格 clause-enums：条文列出来的封闭集合，与代码里兑现它的那个枚举逐个成员对上
#
# 拿 .claude/gate.d/clause-enum-pairs.tsv 逐对判，任一条不成立判红：
#   ① 枚举里每个变体都在表里有一行（代码加了成员而没登记）；
#   ② 表里每一行的变体在枚举里真的存在（代码删了成员而表没跟）；
#   ③ 表里每一行「条文里对应的名字」是那条分项正文里用「 / 」列举的一项，逐字、不按子串认；
#   ④ 正文那串「 / 」列举的每一项都在表里有一行（条文加了成员、枚举与表都没动）。
#      那串列举取正文里与表中名字重合最多的一串「甲 / 乙 / 丙」（项里不含括号、逗号、句号、分号与 `*`），
#      末项后面紧跟的「六种」这类计数词不算进项名。
# 表不在、或一行登记都没有（只剩注释），退 77。
# 射程：只判名字对得上，不判这个成员的语义是不是条文说的那个（靠 review）。
# 样本：red 的枚举多一个没登记的变体 `Bing`、表里一行只写了列举项「乙种」的一截「乙」、条文多列了一项「丙种」，三处都要点名；
# green 的枚举、表与条文列举逐个对上。
cell_clause_enums() {
  local TABLE=".claude/gate.d/clause-enum-pairs.tsv"
  if [[ ! -f "$TABLE" ]]; then
    echo "  ⊘ 本次未跑：$TABLE 不在，今天无对象可判"
    exit 77
  fi

  local report
  if ! report="$(TABLE="$TABLE" python3 - 2>&1 <<'PY'
import os, re, glob

break_substring_match = os.environ.get("CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK") == "clause-enums"

rows = []
for number, line in enumerate(open(os.environ["TABLE"], encoding="utf-8"), 1):
    stripped = line.rstrip("\n")
    if not stripped.strip() or stripped.lstrip().startswith("#"):
        continue
    parts = stripped.split("\t")
    if len(parts) != 5 or not all(part.strip() for part in parts):
        print("BAD", f"{os.environ['TABLE']}:{number}", "这一行不是五段非空（决策分项、枚举落点、变体名、条文里的名字、为什么）", sep="\t")
        continue
    rows.append((number, *[part.strip() for part in parts]))

def clause_text(item):
    match = re.match(r"^(D\d+)\s+已定项\s+(\d+)$", item)
    if not match:
        return None, f"决策分项写成「{item}」，认不出「D<n> 已定项 <k>」这个形态"
    decision, number = match.group(1), match.group(2)
    files = glob.glob(f".claude/kb/decisions/{int(decision[1:]):02d}-*.md")
    if not files:
        return None, f"找不到 {decision} 的正文文件"
    text = open(files[0], encoding="utf-8").read()
    section = re.search(rf"^#### 已定项 {number}[:：].*?(?=^#### |\Z)", text, re.S | re.M)
    if not section:
        return None, f"{files[0]} 里找不到「#### 已定项 {number}」这一节"
    return section.group(0), None

def enum_variants(where):
    path, _, name = where.rpartition(":")
    if not path or not name:
        return None, f"枚举落点写成「{where}」，认不出「<文件>:<枚举名>」这个形态"
    if not os.path.exists(path):
        return None, f"{path} 不在了"
    text = open(path, encoding="utf-8").read()
    block = re.search(rf"enum\s+{re.escape(name)}\s*\{{(.*?)^\}}", text, re.S | re.M)
    if not block:
        return None, f"{path} 里找不到 `enum {name}`"
    body = re.sub(r"//.*", "", block.group(1))
    body = re.sub(r"#\[[^\]]*\]", "", body)
    return re.findall(r"^\s*([A-Z][A-Za-z0-9]*)\s*(?:\{|\(|,|$)", body, re.M), None

LIST_RUN = re.compile(r"[^/（）()，,。；;\n*]+(?:\s*/\s*[^/（）()，,。；;\n*]+)+")
COUNT_WORD = re.compile(r"[一二三四五六七八九十两0-9]+\s*种$")

def enumerated_items(section, names):
    """分项正文里与 names 重合最多的那一串「甲 / 乙 / 丙」列举：返回项的列表；一串都没有返回 None。"""
    best, best_overlap = None, 0
    for run in LIST_RUN.finditer(section):
        items = [COUNT_WORD.sub("", item.strip()).strip() for item in run.group(0).split("/")]
        overlap = len(set(items) & set(names))
        if overlap > best_overlap:
            best, best_overlap = items, overlap
    return best

def is_enumerated_item(chinese, items):
    if break_substring_match:
        return any(chinese in item for item in items)   # 弄坏开关：按子串认
    return chinese in items

if not rows:
    print("EMPTY", sep="\t")

clauses, enums = {}, {}
checked = 0
for number, item, where, variant, chinese, _why in rows:
    if item not in clauses:
        clauses[item] = clause_text(item)
    section, why = clauses[item]
    if why:
        print("BAD", f"{os.environ['TABLE']}:{number}", why, sep="\t")
        continue
    if where not in enums:
        enums[where] = enum_variants(where)
    variants, why = enums[where]
    if why:
        print("BAD", f"{os.environ['TABLE']}:{number}", why, sep="\t")
        continue
    checked += 1
    if variant not in variants:
        print("BAD", f"{os.environ['TABLE']}:{number}",
              f"表里写着变体 `{variant}`，而 {where} 的枚举里没有它（代码删了成员而表没跟）", sep="\t")
    if chinese not in section:
        print("BAD", f"{os.environ['TABLE']}:{number}",
              f"{item} 的正文里找不到「{chinese}」（条文没跟上代码，或者名字改了而表没跟）", sep="\t")

# ③ 的逐字那一半与 ④：按（分项，枚举）成组，取正文里那串列举，两边集合逐项比。
groups = {}
for number, item, where, variant, chinese, _why in rows:
    if clauses.get(item, (None, "x"))[1] or enums.get(where, (None, "x"))[1]:
        continue
    groups.setdefault((item, where), []).append((number, chinese))
for (item, where), members in sorted(groups.items()):
    names = [chinese for _number, chinese in members]
    items = enumerated_items(clauses[item][0], names)
    if items is None:
        print("BAD", f"{item}", f"正文里找不到一串用「 / 」分隔、含表中名字的列举，{where} 那张表逐项对不了", sep="\t")
        continue
    for number, chinese in members:
        if chinese in clauses[item][0] and not is_enumerated_item(chinese, items):
            print("BAD", f"{os.environ['TABLE']}:{number}",
                  f"表里的「{chinese}」不是 {item} 正文那串列举里的一项（列举是：{' / '.join(items)}；按子串对上的不算）", sep="\t")
    for extra in items:
        if extra not in names:
            print("BAD", f"{item}",
                  f"条文列了「{extra}」，而 {os.environ['TABLE']} 里 {where} 没有它那一行（条文加了成员而枚举与表都没跟）", sep="\t")

# 反过来：枚举里有、表里没有的变体。
for where, (variants, why) in enums.items():
    if why:
        continue
    registered = {variant for _n, _i, w, variant, _c, _y in rows if w == where}
    for variant in variants:
        if variant not in registered:
            print("BAD", where,
                  f"枚举里有变体 `{variant}`，而 {os.environ['TABLE']} 里一行都没有（代码加了成员而没登记）", sep="\t")

print("COUNT", checked, len(enums), len(clauses), sep="\t")
PY
)"; then
    echo "  ✗ 扫描没跑完：内嵌 python 自己出错了，一对都没比"
    printf '%s\n' "$report" | tail -5 | sed 's/^/      /'   # gate-lint:detail
    echo "    → 怎么办：按上面的报错修 .claude/gate.d/${STAGE_NAME} 里 clause-enums 那一格的 python；"
    echo "      这一步没跑完就是什么都没查，不是通过。"
    exit 1
  fi

  if ! grep -q '^COUNT' <<<"$report"; then
    echo "  ✗ 扫描没跑完：内嵌 python 没报出 COUNT 那一行（多半是它自己崩了）"
    echo "    → 怎么办：单独跑一遍这一格（--check clause-enums）看 python 的报错；没有 COUNT 就是一对都没比，"
    echo "      而汇总行看着与比过了一模一样。"
    exit 1
  fi

  local -a bad
  mapfile -t bad < <(grep '^BAD' <<<"$report")

  if ((${#bad[@]})); then
    echo "  ✗ 条文列的集合与代码里的枚举对不上："
    while IFS=$'\t' read -r _ where why; do
      printf '      %s：%s\n' "$where" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${bad[@]}")
    echo "    → 怎么办：先判是哪一边错了——代码加了成员就把它写回那条分项的正文、再往 $TABLE 补一行；"
    echo "      条文才是对的就改代码。⚠️ 别只改表让两边看着一致：表是登记位，不是第三种说法。"
    echo "      改分项正文是一次决策变更，照 .claude/rules/format-evolution.md 记进当月变更史。"
    exit 1
  fi

  if grep -q '^EMPTY' <<<"$report"; then
    echo "  ⊘ 本次无对象可判：$TABLE 里一行登记都没有（只剩注释与空行）"
    exit 77
  fi
  local checked enum_count clause_count
  read -r _ checked enum_count clause_count < <(grep '^COUNT' <<<"$report")
  echo "  ✓ 条文列的集合与代码里的枚举逐个对得上（${checked} 对成员、${enum_count} 个枚举、${clause_count} 条分项；两个方向都逐项比过：表里的名字都是正文列举的一项，正文列举的每一项都在表里）"
  exit 0
}

# ── 格 feature-bits：feature bit 位号：记账表与 D15 登记表逐行一致、不跳号，代码引用的位都在记账表里
#
# 依据：D15（格式冻结政策） 已定项 10（记账落在 .claude/kb/feature-bits.md）与已定项 4（位号的唯一登记位那张表）。
# 四条判据，任一条不成立判红：
#   ① 记账表的（类别，位号）集合与已定项 4 登记表逐行对得上，且记账表的「语义一句话」逐字包含登记表同一位的「含义」；
#   ② 每张位图内位号严格递增、不跳号（已分配的位必须是 0..n-1）；
#   ③ `crates/*/src/**/*.rs` 里名字带 INCOMPAT / COMPAT_RO / COMPAT 段的常量，值解得出位号的，那个位号必须在记账表里；
#   ④ 同一个（类别，位号）出现两行（退役位被赋予新语义就是这个形状）。
# 为什么：回收一个位，旧镜像那一位会被新代码读成「新特性已启用」，文件系统会真的做错事。
# 射程：③ 只认写在一行里、值是位掩码字面量的常量声明。值由别的常量合成的（`SUPPORTED_INCOMPAT_BITS`）、声明跨行的，
# 解不出位号，逐个列进成功那句的「没判位号的」名单。不经具名常量的裸字面量（直接写 `0x01`）不在射程里，靠 review。
# 样本：red 的记账表跳号、同一位登记两行不同语义、多出一位登记表里没有，代码样本又引用了表里没有的一位，①②③④ 都必须报出来；
# green 只有位 0 一行、语义与登记表对得上，判绿。
cell_feature_bits() {
  [[ -f .claude/kb/feature-bits.md ]] || { echo "  ! 没有 .claude/kb/feature-bits.md，这一格无对象可判"; exit 77; }

  python3 - <<'PY'
import glob, os, re, sys

break_duplicate_check = os.environ.get("CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK") == "feature-bits"
FEATURE_BITS_PATH = ".claude/kb/feature-bits.md"
BITMAPS = ("incompat", "compat_ro", "compat")
BITS_PER_BITMAP = 256
EXPECTED_HEADER = ["位号", "类别", "名称", "引入版本", "引入 commit", "状态", "语义一句话"]

problems = []


def reject(summary, details, howto):
    problems.append((summary, details, howto))


def normalize(text):
    return re.sub(r"\s+", "", text.replace("*", "").replace("`", ""))


def is_separator(cells):
    return all(re.fullmatch(r":?-{2,}:?", cell) for cell in cells)


def read_table(lines, start_index, column_count):
    """从 lines[start_index] 起读一张 markdown 表，返回去掉表头与分隔行之后的每行单元格。"""
    rows = []
    for line in lines[start_index:]:
        stripped = line.strip()
        if not stripped.startswith("|"):
            break
        cells = [cell.strip() for cell in stripped.strip("|").split("|")]
        if len(cells) != column_count:
            break
        if is_separator(cells):
            continue
        rows.append(cells)
    return rows


# ── D15 已定项 4 登记表：位号的唯一登记位 ────────────────────────
decision_files = sorted(glob.glob(".claude/kb/decisions/15-*.md"))
d15_allocated = {}      # (类别, 位号) -> 含义
d15_unallocated = {}    # 类别 -> [(起, 止)]
d15_path = ""
if not decision_files:
    reject(
        "找不到 D15（格式冻结政策） 的决策文件（.claude/kb/decisions/15-*.md）——位号的唯一登记位不在，没法比对",
        [],
        "确认决策正文还在 .claude/kb/decisions/ 下；改了文件名就把 15- 这个编号前缀留着，这一格按它找。",
    )
else:
    d15_path = decision_files[0]
    d15_lines = open(d15_path, encoding="utf-8").read().split("\n")
    section_start = section_end = -1
    for index, line in enumerate(d15_lines):
        if line.startswith("#### 已定项 4"):
            section_start = index
        elif section_start >= 0 and line.startswith("#### "):
            section_end = index
            break
    if section_end < 0:
        section_end = len(d15_lines)
    header_index = -1
    for index in range(section_start, section_end):
        if section_start < 0:
            break
        cells = [cell.strip() for cell in d15_lines[index].strip().strip("|").split("|")]
        if d15_lines[index].strip().startswith("|") and len(cells) == 4 and "bitmap" in cells[0] and "含义" in cells[2]:
            header_index = index
            break
    if header_index < 0:
        reject(
            f"{d15_path} 的「已定项 4」里找不到位分配登记表（表头要是 bitmap / 位 / 含义 / 出处 四列）",
            [f"已定项 4 这一节：第 {section_start + 1} 行起" if section_start >= 0 else "连「#### 已定项 4」这个标题都没找到"],
            "位号的唯一登记位就是那张表，表没了这一道就没有比对的对象。把表写回已定项 4，四列照旧：bitmap、位、含义、出处。",
        )
    else:
        malformed = []
        for cells in read_table(d15_lines, header_index + 1, 4):
            bitmap, bit_field, meaning, _source = cells
            if bitmap == "bitmap":
                continue
            if bitmap not in BITMAPS:
                malformed.append(f"类别「{bitmap}」不是 incompat / compat_ro / compat 之一：{' | '.join(cells)}")
                continue
            if re.fullmatch(r"\d+", bit_field):
                d15_allocated[(bitmap, int(bit_field))] = meaning
            elif re.fullmatch(r"\d+\.\.\d+", bit_field):
                if "未分配" not in meaning:
                    malformed.append(f"{bitmap} {bit_field} 是一段区间，含义里却没写「未分配」：{meaning}")
                    continue
                low, high = bit_field.split("..")
                d15_unallocated.setdefault(bitmap, []).append((int(low), int(high)))
            else:
                malformed.append(f"「位」这一格既不是位号也不是 a..b 区间：{' | '.join(cells)}")
        if malformed:
            reject(
                f"{d15_path} 的位分配登记表有行读不出来，位号集合拼不完整",
                malformed,
                "每行的「位」写成一个位号（已分配）或 a..b 区间（未分配，含义里要写「未分配」），类别只许 incompat / compat_ro / compat。",
            )
        elif not d15_allocated and not d15_unallocated:
            reject(
                f"{d15_path} 的位分配登记表一行都没读出来",
                [],
                "扫到 0 行不是通过。检查表是不是紧挨着表头、有没有被围栏包住；表的四列是 bitmap、位、含义、出处。",
            )
        else:
            holes = []
            for bitmap in BITMAPS:
                covered = sorted(bit for (bitmap_name, bit) in d15_allocated if bitmap_name == bitmap)
                for low, high in d15_unallocated.get(bitmap, []):
                    covered.extend(range(low, high + 1))
                covered_sorted = sorted(covered)
                if covered_sorted != list(range(BITS_PER_BITMAP)):
                    missing = sorted(set(range(BITS_PER_BITMAP)) - set(covered_sorted))
                    repeated = sorted({bit for bit in covered_sorted if covered_sorted.count(bit) > 1})
                    holes.append(f"{bitmap}：漏了 {len(missing)} 位（最小 {missing[0] if missing else '—'}）、重了 {len(repeated)} 位")
            if holes:
                reject(
                    f"{d15_path} 的位分配登记表自己没罩满 0..{BITS_PER_BITMAP - 1}，有洞或有重叠",
                    holes,
                    "每张位图的已分配位加上未分配区间要正好覆盖 0..255 一次。罩不满时「这一位归谁」就没有答案，比对也无从谈起。",
                )

# ── 记账表 ──────────────────────────────────────────────
feature_lines = open(FEATURE_BITS_PATH, encoding="utf-8").read().split("\n")
marker_index = -1
for index, line in enumerate(feature_lines):
    if line.strip() == "<!-- feature-bits:table -->":
        marker_index = index
        break
header_index = -1
if marker_index >= 0:
    for index in range(marker_index + 1, len(feature_lines)):
        stripped = feature_lines[index].strip()
        if stripped.startswith("|"):
            header_index = index
            break
        if stripped.startswith("#"):
            break

rows = []
if marker_index < 0 or header_index < 0:
    reject(
        f"{FEATURE_BITS_PATH} 里找不到记账表（要有一行 <!-- feature-bits:table -->，它后面第一张 markdown 表就是记账表）",
        [],
        "在记账表上面单独写一行 <!-- feature-bits:table -->，表头七列照 D15（格式冻结政策） 已定项 10：" + " | ".join(EXPECTED_HEADER),
    )
else:
    header_cells = [cell.strip() for cell in feature_lines[header_index].strip().strip("|").split("|")]
    if header_cells != EXPECTED_HEADER:
        reject(
            f"{FEATURE_BITS_PATH} 记账表的表头与 D15（格式冻结政策） 已定项 10 定的列序对不上",
            [f"读到：{' | '.join(header_cells)}", f"要的是：{' | '.join(EXPECTED_HEADER)}"],
            "七列照 D15（格式冻结政策） 已定项 10 写，顺序不许换——别处按列位取值，换了顺序取出来的就是别的东西。",
        )
    else:
        malformed = []
        for cells in read_table(feature_lines, header_index + 1, len(EXPECTED_HEADER)):
            bit_field, bitmap = cells[0], cells[1]
            if not re.fullmatch(r"\d+", bit_field):
                malformed.append(f"「位号」不是一个整数：{' | '.join(cells)}")
                continue
            if bitmap not in BITMAPS:
                malformed.append(f"「类别」不是 incompat / compat_ro / compat 之一：{' | '.join(cells)}")
                continue
            empty = [EXPECTED_HEADER[i] for i, cell in enumerate(cells) if not cell]
            if empty:
                malformed.append(f"{bitmap} 位 {bit_field}：这几列是空的：{'、'.join(empty)}")
                continue
            rows.append((int(bit_field), bitmap, cells[2], cells[3], cells[4], cells[5], cells[6]))
        if malformed:
            reject(
                f"{FEATURE_BITS_PATH} 记账表有行读不出来",
                malformed,
                "每行七格都要填：位号写整数，类别写 incompat / compat_ro / compat，代码侧没有常量的「名称」写破折号。",
            )

# ── ④ 同一个（类别，位号）出现两行 ───────────────────────────
seen = {}
duplicates = []
for bit, bitmap, name, version, commit, status, semantics in rows:
    key = (bitmap, bit)
    if key in seen:
        previous = seen[key]
        shape = "两行语义不同——退役位被赋予新语义就是这个形状" if normalize(previous) != normalize(semantics) else "两行语义相同，是重复登记"
        duplicates.append(f"{bitmap} 位 {bit}：{shape}；先写的是「{previous}」，后写的是「{semantics}」")
    else:
        seen[key] = semantics
if duplicates and not break_duplicate_check:   # 弄坏开关：不判同一个（类别，位号）登记两行
    reject(
        f"{FEATURE_BITS_PATH} 记账表里同一个位号登记了不止一行",
        duplicates,
        "一位只许一行。位一旦用过不许回收（D15（格式冻结政策） 已定项 10）：退役的位把状态改成「退役」、语义永久锁定为退役前最后一次使用的那一句，"
        "新特性另取一个没用过的位，不许改写旧行的语义。",
    )

# ── ② 每张位图内严格递增、不跳号 ─────────────────────────────
order_problems = []
for bitmap in BITMAPS:
    in_file_order = [bit for bit, row_bitmap, *_rest in rows if row_bitmap == bitmap]
    if not in_file_order:
        continue
    if any(later <= earlier for earlier, later in zip(in_file_order, in_file_order[1:])):
        order_problems.append(f"{bitmap}：表里的位号不是严格递增的，读到的次序是 {in_file_order}")
    distinct = sorted(set(in_file_order))
    if distinct != list(range(len(distinct))):
        gaps = sorted(set(range(distinct[-1] + 1)) - set(distinct))
        order_problems.append(f"{bitmap}：已分配的位跳号了，缺 {gaps}（已分配的位必须是 0..{len(distinct) - 1}）")
if order_problems:
    reject(
        f"{FEATURE_BITS_PATH} 记账表的位号次序不对",
        order_problems,
        "每张位图的位号从 0 开始严格递增、一位不跳（D15（格式冻结政策） 已定项 10「位号严格递增、不许跳号占位」）。"
        "跳号占位等于替一个还没人写的特性把位占住，而占位的那一位在盘上与「已启用」长得一模一样。",
    )

# ── ① 与 D15 登记表逐行对得上 ───────────────────────────────
if rows and (d15_allocated or d15_unallocated):
    registry_keys = set(d15_allocated)
    ledger_keys = {(bitmap, bit) for bit, bitmap, *_rest in rows}
    mismatches = []
    for bitmap, bit in sorted(ledger_keys - registry_keys):
        where = ""
        for low, high in d15_unallocated.get(bitmap, []):
            if low <= bit <= high:
                where = f"，登记表把它列在未分配区间 {low}..{high} 里"
        mismatches.append(f"记账表登记了 {bitmap} 位 {bit}，{d15_path} 已定项 4 的登记表里没有这一位{where}")
    for bitmap, bit in sorted(registry_keys - ledger_keys):
        mismatches.append(f"{d15_path} 已定项 4 把 {bitmap} 位 {bit} 分出去了（{d15_allocated[(bitmap, bit)]}），记账表里没有它")
    for bit, bitmap, name, version, commit, status, semantics in rows:
        meaning = d15_allocated.get((bitmap, bit))
        if meaning is None:
            continue
        if normalize(meaning) not in normalize(semantics):
            mismatches.append(
                f"{bitmap} 位 {bit} 的语义与登记表对不上：登记表的「含义」是「{meaning}」，记账表的「语义一句话」是「{semantics}」"
            )
    if mismatches:
        reject(
            f"记账表 {FEATURE_BITS_PATH} 与位号的唯一登记位（{d15_path} 已定项 4）对不上",
            mismatches,
            "位号的唯一登记位是 D15（格式冻结政策） 已定项 4 那张表，记账表只记账：两处不一致时改记账表，不改登记表。"
            "「语义一句话」要逐字包含登记表同一位的「含义」——整行抄过来再往后接细节，别做一个更短的版本。",
        )

# ── ③ 代码里引用的位必须在记账表里 ────────────────────────────
CONST_NAME = re.compile(r"\bconst\s+([A-Z][A-Z0-9_]*)")
CONST_FULL = re.compile(r"\bconst\s+([A-Z][A-Z0-9_]*)\s*:\s*[^=;]+=\s*([^;]+);")
NUMERIC = re.compile(r"^(?:0x([0-9a-fA-F_]+)|([0-9_]+))(?:[ui](?:8|16|32|64|128|size))?$")
SHIFT = re.compile(r"^1(?:[ui](?:8|16|32|64|128|size))?\s*<<\s*([0-9_]+)(?:[ui](?:8|16|32|64|128|size))?$")


def bitmap_of(constant_name):
    segments = constant_name.split("_")
    if "INCOMPAT" in segments:
        return "incompat"
    for index in range(len(segments) - 1):
        if segments[index] == "COMPAT" and segments[index + 1] == "RO":
            return "compat_ro"
    if "COMPAT" in segments:
        return "compat"
    return None


def decode_bit(expression):
    stripped = re.sub(r"//.*$", "", expression).strip()
    match = NUMERIC.fullmatch(stripped)
    if match:
        value = int(match.group(1).replace("_", ""), 16) if match.group(1) else int(match.group(2).replace("_", ""))
        if value != 0 and value & (value - 1) == 0:
            return value.bit_length() - 1, ""
        return None, f"值 `{stripped}` 不是单个位掩码"
    match = SHIFT.fullmatch(stripped)
    if match:
        return int(match.group(1).replace("_", "")), ""
    return None, f"值 `{stripped}` 不是位掩码字面量"


rust_files = sorted(glob.glob("crates/*/src/**/*.rs", recursive=True))
decoded = []
undecided = []
for path in rust_files:
    for line_number, line in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        expressions = {name: expression for name, expression in CONST_FULL.findall(line)}
        for name in CONST_NAME.findall(line):
            bitmap = bitmap_of(name)
            if bitmap is None:
                continue
            if name not in expressions:
                undecided.append((name, path, line_number, "常量声明没写在一行里，解不出值"))
                continue
            bit, reason = decode_bit(expressions[name])
            if bit is None:
                undecided.append((name, path, line_number, reason))
            else:
                decoded.append((bitmap, bit, name, path, line_number))

ledger_keys = {(bitmap, bit) for bit, bitmap, *_rest in rows}
unregistered = [
    f"{path}:{line_number} 的 {name} 指 {bitmap} 位 {bit}，{FEATURE_BITS_PATH} 的记账表里没有这一位"
    for bitmap, bit, name, path, line_number in decoded
    if (bitmap, bit) not in ledger_keys
]
if unregistered:
    reject(
        f"代码引用了记账表里不存在的 feature bit 位",
        unregistered,
        f"先在 {FEATURE_BITS_PATH} 的记账表里给这一位写一行（七列齐全，引入 commit 用 git log 查，别填猜的），"
        "并在 D15（格式冻结政策） 已定项 4 的登记表里把它从未分配区间里切出来——两处都到位这一道才绿。",
    )

for summary, details, howto in problems:
    print(f"  ✗ {summary}")                      # gate-lint:summary
    for detail in details:
        print(f"     {detail}")                  # gate-lint:detail
    print(f"     → 怎么办：{howto}")
if problems:
    sys.exit(1)

skipped_text = "；".join(
    f"{name}（{path}:{line_number}，{reason}）" for name, path, line_number, reason in undecided
) or "无"
print(
    f"  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 {len(rows)} 位、"
    f"登记表分出去 {len(d15_allocated)} 位；"
    f"扫了 {len(rust_files)} 个 .rs，认出 {len(decoded) + len(undecided)} 处 feature bit 常量、解出位号 {len(decoded)} 处；"
    f"没判位号的 {len(undecided)} 处：{skipped_text}）"
)
PY
}

# ── 格 mutation-tables：每个实验二进制都要有变异表
#
# research 那侧：research/e7-index-bench/src/bin/ 下每个 *.rs 在 research/mutations/ 下有同名 .tsv，至少一条成形的变异
# （三段制表符分隔）、没有坏行；每条原文（\n 还原成换行）在源码里恰好命中一次（mutate.sh 命中 0 次或多次就退 3，后面一条都不跑）；
# 原文与替换文里只许 \n 这一种反斜杠转义。没有同名二进制的表（shell 探针的）不判锚点，成功行逐个列名。
# 为什么：kb 里每一句「N 条变异全部被抓」都靠这些表复现。
#
# crates/mutations.tsv（门禁 checker-tier-crates-mutation-replay 的表，六段）：那一道要跑几个钟头，改 crates/ 的实现员不跑它，
# 它派活之前判的几样在这里先静态判。判法只有一份，在 research/scripts/crates-mutation-rows.py 的 row_problems，这里经 static-check 调：
#   成形：六段；替换文可以空（删掉原文），其余五段不许空；
#   锚点：原文的 \n 还原成换行后，在「文件」那一段指的源码里恰好命中一次，文件不在、读不了也算；
#   测试名：必须红的测试名是一个用例名（不带正则符号与空白）；
#   转义：原文与替换文里只许 \n 这一种反斜杠转义（别的按字面写进源码，那条变异编不过、等于没跑）；
#   拷贝范围 ①：某一行改的文件不在 .claude/gate.d/stage-inputs.tsv 里那一道登记的路径之内（它只把这些路径拷进每片的源码副本）；
#   拷贝范围 ②：crates/ 下 .rs 里以 "../ 起头、按所在 crate 的根解析之后逃出 crates/、又不在拷贝范围里的字符串字面量
#     （#[path = …] 与注释里的不算）：运行时读它的测试在副本里没改坏就红，那一道记「基线不绿」。
#     管不到：路径拼在变量里、环境变量给的、format! 拼出来的、运行期再 join("..") 的；指到仓根本身的（"../../"）按不在范围里判。
# 另判三样那一道不判的：两行重复；点名的测试标了 #[ignore] 而参数里没有 --include-ignored；
# `--` 之后的筛选词一个都筛不到点名的测试（libtest 比的是完整测试名：模块路径由点名的函数在哪个文件、套在哪几层内联 mod 里算出）。
#
# 这一格不跑变异（全表要逐条重编译，量级是小时），只保证装置在、形状对。research 那侧「表今天还会不会红」靠改实验代码时手跑
# research/scripts/mutate.sh，门禁不跑：C40（变异表没存）。
#
# 样本：red 里 research 那侧有一份没有同名表的实验二进制（e2_naked）、一条锚点命中 0 次、一条替换文带 \& 的、一张不是 UTF-8 的表（setup.sh 现造）；
# crates/mutations.tsv 那侧有锚点腐化、转义写进源码、文件列是目录、点名慢档用例没带开关、两种筛选词对不上、文件在拷贝范围外、
# 测试名带符号、测试名空着（setup.sh 现写）各一行，另有一份逃出 crates/ 的相对路径字面量；
# green 是同形的一套都对得上的（含替换文空着的删行变异、读 litmus/ 的测试），每个实验二进制各有一张锚点唯一的表。
cell_mutation_tables() {
BIN_DIR=research/e7-index-bench/src/bin
MUT_DIR=research/mutations
[[ -d "$BIN_DIR" && -d "$MUT_DIR" ]] || { echo "  ! 找不到 $BIN_DIR 或 $MUT_DIR，这一格跳过"; exit 77; }

missing=(); malformed=()
for src in "$BIN_DIR"/*.rs; do
  stem="$(basename "$src" .rs)"
  tsv="$MUT_DIR/$stem.tsv"
  if [[ ! -f "$tsv" ]]; then missing+=("$tsv 不存在（被测的是 $src）"); continue; fi
  # 至少一条成形的变异行：非注释、非空、恰好三段
  ok_rows=$(awk -F'\t' '!/^#/ && NF==3 && $1!="" && $2!="" {n++} END{print n+0}' "$tsv")
  bad_rows=$(awk -F'\t' '!/^#/ && NF!=3 && $0!="" {n++} END{print n+0}' "$tsv")
  if [[ "$ok_rows" -eq 0 || "$bad_rows" -gt 0 ]]; then malformed+=("$tsv：成形 $ok_rows 条、坏 $bad_rows 行"); fi
done

# ── research 变异表的锚点与转义：只做子串计数，不跑变异 ──
anchor_report="$(BIN_DIR="$BIN_DIR" MUT_DIR="$MUT_DIR" python3 - <<'PY'
import os, glob, re
bin_dir = os.environ["BIN_DIR"]; mut_dir = os.environ["MUT_DIR"]
bad = []; stray = []; unreadable = []; checked = 0
def stray_escapes(segment_name, segment):
    """mutate.sh 与 checker-tier-crates-mutation-replay 只把 \\n 还原成换行，别的反斜杠按字面写进源码（C427）。"""
    return [(segment_name, match.group(0)) for match in re.finditer(r"\\(.)", segment) if match.group(1) != "n"]
for tsv in sorted(glob.glob(os.path.join(mut_dir, "*.tsv"))):
    stem = os.path.basename(tsv)[:-4]
    src_path = os.path.join(bin_dir, stem + ".rs")
    # 没有同名二进制的表（shell 探针的变异表）不在这一格的射程：它们的被测对象不是 .rs；成功行逐个列名
    if not os.path.exists(src_path):
        print("NO_BINARY", tsv, sep="\t")
        continue
    # 读不了（不是 UTF-8、是个目录、没权限）的一份判红、接着判下一份：崩在这里会让整段报告一行不剩
    try:
        src = open(src_path, encoding="utf-8").read()
        table_lines = open(tsv, encoding="utf-8").read().split("\n")
    except (OSError, UnicodeDecodeError) as error:
        unreadable.append((stem, f"{type(error).__name__}: {error}"))
        continue
    for lineno, line in enumerate(table_lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        parts = line.rstrip("\n").split("\t")
        if len(parts) != 3:
            continue
        name, frm, to = parts
        checked += 1
        # 口径与 mutate.sh 一致：表里的 \n 先还原成换行，再数子串
        hits = src.count(frm.replace("\\n", "\n"))
        if hits != 1:
            bad.append((stem, name, lineno, hits))
        for segment_name, segment in (("原文", frm), ("替换文", to)):
            for found in stray_escapes(segment_name, segment):
                stray.append((stem, name, lineno) + found)
print("CHECKED", checked)
for b in bad:
    print("BAD", *b, sep="\t")
for s in stray:
    print("STRAY", *s, sep="\t")
for u in unreadable:
    print("UNREADABLE", *u, sep="\t")
PY
)"
anchor_exit_code=$?
# ── crates/mutations.tsv 只在这里判的三样：两行重复、#[ignore]、筛选词；行经共用模块的 read_table 读（成形的口径只有它一份）──
MUTATION_ROWS_MODULE="$STAGE_REPOSITORY/research/scripts/crates-mutation-rows.py"
crates_report=""
if [[ -f crates/mutations.tsv ]]; then
  crates_report="$(MUTATION_ROWS_MODULE="$MUTATION_ROWS_MODULE" python3 - <<'PY'
import importlib.util
import os
import re
import sys
sys.dont_write_bytecode = True
specification = importlib.util.spec_from_file_location("crates_mutation_rows", os.environ["MUTATION_ROWS_MODULE"])
mutation_rows = importlib.util.module_from_spec(specification)
specification.loader.exec_module(mutation_rows)
first_line_of_change = {}
first_line_of_name = {}
attributes_by_package = {}
def module_path_of_file(package, path):
    """crates/<包>/src 下一个 .rs 在 libtest 测试名里的模块前缀：src/lib.rs 与 src/bin/<名>.rs 是空，src/a.rs 与 src/a/mod.rs 是 a，src/a/b.rs 是 a::b；tests/ 下的是空。"""
    relative = os.path.relpath(path, os.path.join("crates", package))
    parts = relative.split(os.sep)
    if parts[0] != "src" or parts[1:2] == ["bin"] or parts[1:] == ["lib.rs"]:
        return []
    parts = parts[1:]
    parts[-1] = parts[-1][:-len(".rs")]
    if parts[-1] == "mod":
        parts = parts[:-1]
    return parts
def enclosing_inline_modules(text):
    """(偏移, 那一刻套着的内联 mod 名单) 的有序表：按 `mod 名 {` 与花括号配平；字符串与注释里的花括号不另判。"""
    events, stack, depth = [], [], 0
    for token in re.finditer(r"\bmod\s+(\w+)\s*\{|[{}]", text):
        if token.group(1):
            depth += 1
            stack.append((token.group(1), depth))
        elif token.group(0) == "{":
            depth += 1
        else:
            if stack and stack[-1][1] == depth:
                stack.pop()
            depth -= 1
        events.append((token.end(), [name for name, _ in stack]))
    return events
def test_attributes_of_package(package):
    """包 crates/<包> 的 tests/ 与 src/ 下每个 fn 名 → [(它上面那串属性与注释行, libtest 里的完整测试名)]；每个包只扫一次。"""
    if package not in attributes_by_package:
        table = {}
        for directory_name in ("tests", "src"):
            for root, _directories, files in os.walk(os.path.join("crates", package, directory_name)):
                for file_name in sorted(files):
                    if not file_name.endswith(".rs"):
                        continue
                    path = os.path.join(root, file_name)
                    try:
                        text = open(path, encoding="utf-8").read()
                    except (OSError, UnicodeDecodeError):
                        continue
                    prefix = module_path_of_file(package, path)
                    events = enclosing_inline_modules(text)
                    for found in re.finditer(r"((?:[ \t]*#\[[^\n]*\n|[ \t]*//[^\n]*\n)*)[ \t]*(?:pub\s+)?fn\s+(\w+)\s*\(", text):
                        modules = []
                        for offset, stack in events:
                            if offset > found.start(2):
                                break
                            modules = stack
                        table.setdefault(found.group(2), []).append((found.group(1), "::".join(prefix + modules + [found.group(2)])))
        attributes_by_package[package] = table
    return attributes_by_package[package]
rows, _malformed, _memory, _timeout = mutation_rows.read_table("crates/mutations.tsv")
for row in rows:
    line_number, fields = row.line_number, row.fields
    name, path = row.name, row.path
    # 两行的（文件, 原文, 替换文, 点名的测试）逐字相同：表里少了一条真改法，而条数看着没少。
    # 只按改法认会误拒：同一个改法让两条不同的测试变红是合法的。
    change = (path, fields[2], fields[3], fields[5])
    if change in first_line_of_change:
        print("CRATES_DUP", line_number, name, f"（文件, 原文, 替换文, 点名的测试）与第 {first_line_of_change[change]} 行逐字相同", sep="\t")
    else:
        first_line_of_change[change] = line_number
    # 变异名相同 ⇒ 按名字分格的报告把两条并成一格，其中一条的结论看不见。
    if name in first_line_of_name:
        print("CRATES_DUP", line_number, name, f"变异名与第 {first_line_of_name[name]} 行相同", sep="\t")
    else:
        first_line_of_name[name] = line_number
    # 点名的测试标了 #[ignore]（harness 慢档用例、checker 档的全量），cargo test 参数里却没有 --include-ignored / --ignored：
    # checker-tier-crates-mutation-replay 与 prove-red.sh 照这一行跑，点名的测试被 libtest 跳过，这条变异永远报「没红」，而它守的那一格其实没人看。
    arguments = fields[4].split()
    package = arguments[arguments.index("-p") + 1] if "-p" in arguments[:-1] else None
    test_name = fields[5].split("::")[-1].strip()
    # cargo test 参数里 `--` 之后的位置参数是 libtest 的筛选词：点名的测试一个都筛不到，那条测试根本不跑，变异永远报没红。
    libtest_arguments = arguments[arguments.index("--") + 1:] if "--" in arguments else []
    filters = [word for word in libtest_arguments if not word.startswith("-")]
    exact = "--exact" in libtest_arguments
    # libtest 拿筛选词去比完整测试名（模块路径 + 函数名）：完整名从点名的函数在哪个文件、套在哪几层内联 mod 里算出来；找不到那个函数的不判。
    candidates = [full_name for _attributes, full_name in test_attributes_of_package(package).get(test_name, [])] if package and test_name else []
    if candidates and filters and not any((full_name == word) if exact else (word in full_name) for word in filters for full_name in candidates):
        print("CRATES_FILTER", line_number, name, f"筛选词 {' '.join(filters)}{'（--exact）' if exact else ''} 筛不到点名的 {' 或 '.join(candidates)}", sep="\t")
    if package and test_name and not ({"--include-ignored", "--ignored"} & set(arguments)):
        found_tests = test_attributes_of_package(package).get(test_name, [])
        if found_tests and all("#[ignore" in attributes for attributes, _full_name in found_tests):
            print("CRATES_IGNORED", line_number, name, f"点名的 {test_name} 标了 #[ignore]，cargo test 参数「{fields[4]}」里没有 --include-ignored", sep="\t")
print("CRATES_CHECKED", len(rows))
PY
)"
  crates_exit_code=$?
fi
# ── 门禁 checker-tier-crates-mutation-replay 派活之前判的那几样与拷贝范围：共用模块的 static-check 逐处打
# MALFORMED / ANCHOR / NAME / ROW / STRAY / LITERAL，末尾 COPIED 与 CHECKED；退 0 一处都没有、退 1 有、别的退出码是它自己没判完（ERROR 行是原因）
scope_report=""; scope_exit_code=0
if [[ -f "$MUTATION_ROWS_MODULE" ]]; then
  scope_report="$(python3 "$MUTATION_ROWS_MODULE" static-check "$ROOT")" || scope_exit_code=$?
else
  scope_exit_code=127; scope_report=$'ERROR\t'"找不到 $MUTATION_ROWS_MODULE"
fi
scope_copied="$(sed -n 's/^COPIED\t//p' <<<"$scope_report")"
scope_checked="$(sed -n 's/^CHECKED\t//p' <<<"$scope_report")"
mapfile -t crates_malformed < <(grep $'^MALFORMED\t' <<<"$scope_report")
mapfile -t crates_bad < <(grep $'^ANCHOR\t' <<<"$scope_report")
mapfile -t crates_names < <(grep $'^NAME\t' <<<"$scope_report")
mapfile -t scope_rows < <(grep $'^ROW\t' <<<"$scope_report")
mapfile -t crates_stray < <(grep $'^STRAY\t' <<<"$scope_report")
mapfile -t scope_literals < <(grep $'^LITERAL\t' <<<"$scope_report")
mapfile -t scope_errors < <(sed -n 's/^ERROR\t//p' <<<"$scope_report")
IFS=$'\t' read -r scope_row_count scope_source_count scope_literal_count <<<"$scope_checked"
crates_checked="$(sed -n 's/^CRATES_CHECKED //p' <<<"$crates_report")"
mapfile -t crates_dup < <(grep '^CRATES_DUP' <<<"$crates_report")
mapfile -t crates_ignored < <(grep '^CRATES_IGNORED' <<<"$crates_report")
mapfile -t crates_filter < <(grep '^CRATES_FILTER' <<<"$crates_report")
anchor_checked="$(sed -n 's/^CHECKED //p' <<<"$anchor_report")"
mapfile -t anchor_bad < <(grep '^BAD' <<<"$anchor_report")
mapfile -t stray_rows < <(grep '^STRAY' <<<"$anchor_report")
mapfile -t anchor_unreadable < <(grep '^UNREADABLE' <<<"$anchor_report")
mapfile -t tables_without_binary < <(sed -n 's/^NO_BINARY\t//p' <<<"$anchor_report")
# 两段 python 自己崩了（退出码非 0、或没报出 CHECKED 那一行）：它们没判完，不许读成「没有 BAD 行」
script_failures=()
if [[ "${anchor_exit_code:-1}" -ne 0 || -z "$anchor_checked" ]]; then
  script_failures+=("research 变异表的锚点检查没跑完（python 退出码 ${anchor_exit_code:-?}，CHECKED 行「${anchor_checked}」）")
fi
if [[ -f crates/mutations.tsv ]] && [[ "${crates_exit_code:-1}" -ne 0 || -z "$crates_checked" ]]; then
  script_failures+=("crates/mutations.tsv 的重复、#[ignore]、筛选词检查没跑完（python 退出码 ${crates_exit_code:-?}，CRATES_CHECKED 行「${crates_checked}」）")
fi
if [[ "$scope_exit_code" -ne 0 && "$scope_exit_code" -ne 1 ]] || [[ -z "$scope_checked" ]]; then
  script_failures+=("门禁 checker-tier-crates-mutation-replay 派活之前那几样与拷贝范围的检查没跑完（research/scripts/crates-mutation-rows.py static-check 退出码 ${scope_exit_code}，CHECKED 行「${scope_checked}」；${scope_errors[*]:-没报原因}）")
fi

if ((${#missing[@]} + ${#malformed[@]} + ${#anchor_bad[@]} + ${#crates_bad[@]} + ${#crates_dup[@]} + ${#crates_ignored[@]} + ${#crates_filter[@]} + ${#stray_rows[@]} + ${#crates_stray[@]} + ${#crates_malformed[@]} + ${#crates_names[@]} + ${#anchor_unreadable[@]} + ${#script_failures[@]} + ${#scope_rows[@]} + ${#scope_literals[@]})); then
  if ((${#script_failures[@]})); then
    echo "  ✗ 锚点或拷贝范围检查的 python 没跑完，这一段等于没判："
    printf '      %s\n' "${script_failures[@]}"
    echo "    → 怎么办：看上面 python 打出的报错（多半是某份文件读不了），修好再跑；没跑完不许当成锚点都对得上。"
  fi
  if ((${#anchor_unreadable[@]})); then
    echo "  ✗ 这些变异表或它的被测源码读不了（不是 UTF-8、是目录、没权限），表里的条目一条都没判："
    while IFS=$'\t' read -r _ stem why; do
      printf '      %s 或 %s：%s\n' "$MUT_DIR/$stem.tsv" "$BIN_DIR/$stem.rs" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${anchor_unreadable[@]}")
    echo "    → 怎么办：转成 UTF-8 文本（或把放错的目录挪走），再跑；mutate.sh 读它时同样会失败。"
  fi
  if ((${#missing[@]})); then
    echo "  ✗ 这些实验二进制没有同名变异表："   # gate-lint:detail
    printf '      %s\n' "${missing[@]}"
  fi
  if ((${#malformed[@]})); then
    echo "  ✗ 这些变异表不成形（要求每行三段制表符分隔，至少一条）："
    printf '      %s\n' "${malformed[@]}"
    echo "    → 每行写成「变异名<TAB>原文<TAB>替换文」三段，坏行补齐或删掉，至少留一条成形的。"
  fi
  if ((${#anchor_bad[@]})); then
    echo "  ✗ 这些变异条目的「原文」在源码里不是恰好命中一次（mutate.sh 会退出码 3，后面的条目一条都不跑）："
    while IFS=$'\t' read -r _ stem name lineno hits; do
      printf '      %s:%s %s：原文在 %s 里命中 %s 次\n' "$MUT_DIR/$stem.tsv" "$lineno" "$name" "$BIN_DIR/$stem.rs" "$hits"   # gate-lint:detail
    done < <(printf '%s\n' "${anchor_bad[@]}")
    echo "    命中 0 次：源码改过而表没跟；命中多次：原文要多带一行上下文才唯一。"
  fi
  if ((${#missing[@]} + ${#malformed[@]} + ${#anchor_bad[@]})); then
    echo "  → 怎么办：缺表的写 research/mutations/<bin名>.tsv（每行：变异名<TAB>原文<TAB>替换文）；"
    echo "    锚点对不上的把「原文」改成今天源码里逐字存在、且只出现一次的那一段，"
    echo "    改完跑 bash research/scripts/mutate.sh <bin> <源文件> <表> 证明每条都被抓，再来。"
  fi
  if ((${#stray_rows[@]})); then
    echo "  ✗ 这些变异条目的原文或替换文里有 \\n 以外的反斜杠转义（mutate.sh 只还原 \\n，别的按字面写进源码）："
    while IFS=$'\t' read -r _ stem name lineno segment escape; do
      printf '      %s:%s %s：%s里的 %s\n' "$MUT_DIR/$stem.tsv" "$lineno" "$name" "$segment" "$escape"   # gate-lint:detail
    done < <(printf '%s\n' "${stray_rows[@]}")
    echo "    → 怎么办：去掉那个反斜杠。替换文按字面写进源码，\\& \\\" 这类会原样落进去、那份源码编不过，"
    echo "      变异既不算被抓也不算没红（.claude/rules/mutation-sampling.md 第八类：无效变异），而跑变异的脚本报的是「点名的测试没跑到」。"
  fi
  if ((${#crates_stray[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行的原文或替换文里有 \\n 以外的反斜杠转义（门禁 checker-tier-crates-mutation-replay 只还原 \\n）："
    while IFS=$'\t' read -r _ lineno name segment escape; do
      printf '      crates/mutations.tsv:%s %s：%s里的 %s\n' "$lineno" "$name" "$segment" "$escape"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_stray[@]}")
    echo "    → 怎么办：去掉那个反斜杠，改完单跑那一行证明点名的测试红（2026-09-20 第 180 行踩过：替换文写 \\&report.outcome，"
    echo "      写进源码是字面反斜杠、编不过，checker-tier-crates-mutation-replay 把它报成「点名的测试没跑到」，出路指向「先造一条会红的用例」，方向是反的）。"
  fi
  if ((${#crates_dup[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行与前面某一行重复（表里的条数没少，真改法少了一条）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_dup[@]}")
    echo "    → 怎么办：这一行本来要守的那一格今天没有守卫。把它换成真正打在那一格上的改法（改完单跑它、证明点名的测试红），"
    echo "      或者确认它多余就整行删掉。⚠️ 同一个改法让**两条不同的测试**变红是合法的，这道检查按「改法 + 点名的测试」四项认，不会拦那一种。"
  fi
  if ((${#crates_filter[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行 cargo test 参数里的筛选词筛不到点名的测试（checker-tier-crates-mutation-replay 跑它时那条测试不跑，变异永远报没红）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_filter[@]}")
    echo "    → 怎么办：把 \`--\` 后面的筛选词改成点名测试名的一段（测试改过名、拆过的多半是这个），或者删掉筛选词跑整个测试目标。"
  fi
  if ((${#crates_ignored[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行点名的测试标了 #[ignore]，cargo test 参数里却没有 --include-ignored（checker-tier-crates-mutation-replay 跑它时那条测试被跳过，变异永远报没红）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_ignored[@]}")
    echo "    → 怎么办：在那一行 cargo test 参数的 \`--\` 后面加 --include-ignored（没有 \`--\` 就补 \`-- --include-ignored\`），改完单跑那一行证明点名的测试红。"
  fi
  if ((${#crates_malformed[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行不成形（六段制表符分隔：变异名、文件、原文、替换文、cargo test 参数、必须红的测试名；替换文可以空，那是删掉原文，其余五段不许空；门禁 checker-tier-crates-mutation-replay 派活之前同样判红，一条都不跑）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_malformed[@]}")
    echo "    → 怎么办：补上空着的那一段，制表符删补到恰好六段；原文 / 替换文里的换行写成 \\n。要删掉原文就让替换文空着（两个制表符挨着）。"
  fi
  if ((${#crates_names[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行的「必须红的测试名」不是一个用例名（带了正则符号或空白，门禁 checker-tier-crates-mutation-replay 按字面在测试输出里永远找不到，派活之前同样判红）："
    while IFS=$'\t' read -r _ lineno name expected; do
      printf '      crates/mutations.tsv:%s %s：「%s」\n' "$lineno" "$name" "$expected"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_names[@]}")
    echo "    → 怎么办：那一列写成用例名本身（或带模块路径 tests::<名字>），去掉 \$、^ 这类符号；checker-tier-crates-mutation-replay 已经按「在输出里只许对上一个用例」判，不用再靠 \$ 锚定。"
  fi
  if ((${#crates_bad[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行的「原文」在源码里不是恰好命中一次（门禁 checker-tier-crates-mutation-replay 预扫会整张表退出，一条都不跑）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_bad[@]}")
    echo "    → crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \\n），改完单跑那几行证明点名的测试红。"
  fi
  if ((${#scope_rows[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行改的文件不在门禁 checker-tier-crates-mutation-replay 每片拷的范围里（checker-tier-crates-mutation-replay 派活之前就整道判红，一条都不跑）："
    while IFS=$'\t' read -r _ lineno name path; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$path"   # gate-lint:detail
    done < <(printf '%s\n' "${scope_rows[@]}")
    echo "    → 怎么办：把那个路径（或它所在的目录）登记进 .claude/gate.d/stage-inputs.tsv 里 checker-tier-crates-mutation-replay.sh 那一行（拷贝范围就是那一行登记的路径，现在是：${scope_copied}），"
    echo "      或者把这条变异改到拷贝范围之内的文件上。"
  fi
  if ((${#scope_literals[@]})); then
    echo "  ✗ crates/ 下这些 .rs 里有以 \"../ 起头、按所在 crate 的根解析之后逃出 crates/、又不在门禁 checker-tier-crates-mutation-replay 拷贝范围里的字符串字面量（checker-tier-crates-mutation-replay 的副本里没有那个路径，运行时读它的测试没改坏就红，checker-tier-crates-mutation-replay 记「基线不绿」）："
    while IFS=$'\t' read -r _ file lineno literal resolved; do
      printf '      %s:%s %s → %s\n' "$file" "$lineno" "$literal" "$resolved"   # gate-lint:detail
    done < <(printf '%s\n' "${scope_literals[@]}")
    echo "    → 怎么办：把那个路径所在的目录登记进 .claude/gate.d/stage-inputs.tsv 里 checker-tier-crates-mutation-replay 那一行（它同时是 checker-tier-crates-mutation-replay 的复用判定输入，登记之后 checker-tier-crates-mutation-replay 整张重跑一次），"
    echo "      或者让测试读 crates/ 之内的文件；#[path = …] 指的源文件不算，不用管。"
  fi
  exit 1
fi
n=$(ls "$BIN_DIR"/*.rs | wc -l)
crates_summary="；没有 crates/mutations.tsv"
[[ -f crates/mutations.tsv ]] && crates_summary="；crates/mutations.tsv ${scope_row_count} 条的原文各命中源码一次（成形：六段，替换文可以空、其余五段不空；点名的测试名都是用例名）"
crates_summary+="；门禁 checker-tier-crates-mutation-replay 的拷贝范围（${scope_copied}）：变异表 ${scope_row_count} 行改的文件都在里面，crates/ 下 ${scope_source_count} 份 .rs 里逃出 crates/ 的相对路径字面量 ${scope_literal_count} 处都在里面"
if ((${#tables_without_binary[@]})); then
  crates_summary+="；没有同名实验二进制、锚点不在这一格射程的表 ${#tables_without_binary[@]} 张：$(printf '%s ' "${tables_without_binary[@]}")"
fi
echo "  ✓ $n 个实验二进制都有成形的变异表，${anchor_checked} 条变异的原文各命中源码一次${crates_summary}（这一格不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复、点名标了 #[ignore] 的测试都带 --include-ignored、筛选词筛得到点名的测试——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）"
}

# ── 格 absolute-assertions：每个实验二进制都要有钉绝对值的断言
#
# 依据：.claude/singlefs-ai-sop/rules/test-discipline.md「只让多条臂互相比，测不出「所有臂一起错」」。
# 只判机器判得了的一半：这个实验有没有任何一条把某个量与数字字面量比死的断言；钉的是不是对的量、绝对值是不是独立算出来的，留给人。
# 射程：research/e7-index-bench/src/bin 下每一份，加上别处 crates/*/src/bin、research/*/src/bin 下文件名以 e<数字>_ 开头的
# （按是不是实验认，不按住在哪个目录认）；别处不以 e<数字>_ 开头的是装置工具，不判，成功行逐个列名。
# 断言按语句认、不按行认：rustfmt 会把长断言拆成多行，逐行 grep 会把拆开的断言误判成没有。
# 样本：red 有只比相对值的（e7 目录单行 e1_relative、crates/ 下单行 e2_crates_relative）、e7 目录下拆成多行但比字符串的（e2_multiline_relative），
# 与注释掉的、只判非零的、消息文本里带「, 5)」的（e6_commented_and_structural），四份都要点名判红；
# green 里 e7 目录下每一份都钉了绝对值（e1_pinned 单行、e5_multiline_pinned 拆成多行用 `assert_eq!`），crates/ 下一份单行（e3_crates_pinned）、
# 一份拆成多行用 `assert!` 带消息（e4_crates_multiline_pinned），判绿；另放一份不以 e<数字>_ 开头的 crates/demo/src/bin/probe.rs，成功行要把它列成没判的。

# 绝对值断言：与数字字面量比死，assert_eq! 与 assert! 两种形态都认。
# assert! 那一支只认 ==、<=、>=、<、>，而且运算符前一个字不能是 !、=、<、>：`!= 0` 是结构性质（非空、非零），不是把量钉死在一个数上。
ABSOLUTE_ASSERTION_PATTERN='assert_eq!\([^;]*, *-?[0-9][0-9_]*(\.[0-9]+)?\)|assert!\([^;]*[^!<>=](==|<=|>=|<|>) *-?[0-9][0-9_]*(\.[0-9]+)?[,)]'

# 把拆成多行的 assert!()/assert_eq!() 按圆括号配平拼回一条条完整语句，逐条打到 stdout；配平之前挖掉字符串字面量与行注释。
# 每行先掐掉两端空白再首尾相接、不插分隔符：rustfmt 常把收尾的 `);` 单独放一行，原样拼会在数字和 `)` 之间垫出空白，判据就认不出了。
join_assertion_statements() {
  awk '
    function trim(s) {
      gsub(/^[ \t]+|[ \t]+$/, "", s)
      return s
    }
    function strip(line,    s) {
      s = line
      gsub(/"([^"\\]|\\.)*"/, "\"\"", s)
      sub(/\/\/.*/, "", s)
      return s
    }
    function paren_delta(s,    i, c, d) {
      d = 0
      for (i = 1; i <= length(s); i++) {
        c = substr(s, i, 1)
        if (c == "(") d++
        else if (c == ")") d--
      }
      return d
    }
    BEGIN { in_statement = 0; depth = 0; buffer = "" }
    {
      raw_line = $0
      if (!in_statement) {
        # 起点在挖掉字符串与行注释之后的那一行上找：注释掉的 `// assert_eq!(x, 5);`、字符串里的「assert!(」都不是一条断言
        code_line = strip(raw_line)
        if (!match(code_line, /assert(_eq)?!\(/)) next
        piece = trim(substr(code_line, RSTART))
        buffer = piece
        depth = paren_delta(piece)
        in_statement = 1
      } else {
        # 判据套在挖掉字符串与行注释之后的语句上：消息文本里的「, 5)」不算把量钉死
        piece = trim(strip(raw_line))
        buffer = buffer piece
        depth += paren_delta(piece)
      }
      if (depth <= 0) {
        print buffer
        in_statement = 0
        buffer = ""
        depth = 0
      }
    }
  ' "$1"
}

cell_absolute_assertions() {
BINS=research/e7-index-bench/src/bin
# $BINS 不在不等于无对象：住在 crates/ 下、以 e<数字>_ 开头的实验照判；两处都一份没有，才在末尾退 77
judged=() uncovered=()
for f in "$BINS"/*.rs; do
  [[ -f "$f" ]] && judged+=("$f")   # 目录是空的时候 glob 原样留着，不是一份实验
done
in_bins=${#judged[@]}
for directory in crates/*/src/bin research/*/src/bin; do
  [[ -d "$directory" && "$directory" != "$BINS" ]] || continue
  for f in "$directory"/*.rs; do
    [[ -f "$f" ]] || continue
    if [[ "$(basename "$f")" =~ ^e[0-9]+_ ]]; then judged+=("$f"); else uncovered+=("$f"); fi
  done
done
bad=0; n=0
for f in "${judged[@]}"; do
  n=$((n+1))
  c=$(join_assertion_statements "$f" | grep -cE "$ABSOLUTE_ASSERTION_PATTERN")
  if (( c == 0 )); then
    echo "  ✗ $f 一条绝对值断言都没有"
    bad=$((bad+1))
  fi
done

if ((bad)); then
  echo "     → 怎么办：给它加一条把**被量的那个数**钉死的断言，绝对值要由**独立算术**给出，"
  echo "               不许从代码里读回来。加完用 research/scripts/mutate.sh 证明它会红——"
  echo "               实测教训：先加的断言可能一条变异都拦不住（E9 踩过），只有变异测试分得开。"
  exit 1
fi
((n)) || { echo "  ! $BINS 下一个 .rs 都没有、别处也没有 e<数字>_ 开头的，这一格无对象可判"; exit 77; }
echo "  ✓ $n 个实验二进制各自至少有一条绝对值断言（$BINS 下 $in_bins 份，别处 src/bin 下以 e<数字>_ 开头的 $(( n - in_bins )) 份）"
echo "    没判的 ${#uncovered[@]} 份（别处 src/bin 下不以 e<数字>_ 开头，按装置工具算）："
for f in ${uncovered[@]+"${uncovered[@]}"}; do echo "      $f"; done
}

# ── 格 reading-discipline：实验源码的两条读数纪律——种子不许折叠，读数不许恒为字面量 0
#
# 扫 research/e7-index-bench/src/bin/*.rs 与 crates/singlefs-checker-tier/src/bin/e<号>*.rs，只看注释与字符串之外的源码；两条都判完再退出：
#   ① C59（种子折叠成同一个状态）：名字里带 seed 的标识符后面直接跟 `| 1` 或 `& !1`，判红。`seed | 1` 把 2 与 3、4 与 5 折成同一个状态，
#      轮间变异系统性偏小。改法：先乘法混淆再置位，`seed.wrapping_mul(0xD1B5_4A32_D192_ED03) | 1`（奇数乘子是双射）。
#   ② C60（恒定读数没有故障注入自证）：本文件里声明的整型结构体字段，从没出现在写入位置（`= 非零表达式`、`+=` 这类复合赋值、`&mut`），
#      而每一处结构体字面量初始化都是字面量 0，判红：这样的读数与「被测对象真的是 0」在产物里长得一模一样。
# 存量登记进两张豁免表（改一处就要重跑那个实验、逐个回对正文引的数），四列制表符分隔：文件路径、定位（①写行号，②写 `结构体.字段`）、欠账编号、为什么还没改：
#   .claude/gate.d/experiment-seed-fold-lag.tsv（挂 C59）
#   .claude/gate.d/experiment-constant-reading-lag.tsv（挂 C60）
# 豁免表的闸，任一条不成立判红：编号在 checks-owed.md 欠着那张表里（按 lib-owed.py 读，认不出「### 已还清」标题判红）；
# 每一行对得上这一轮扫出来的一处违规；只缩不涨：与基准提交比，同一个文件的登记行数不许变多，基准里没有的文件不许出现。
# 基准里还没有这张表算初次登记、成功行照实报；git 取不到基准那一版判红，不当初次登记。
#
# 射程：
#   ① 只认名字带 seed 的标识符。先把种子搬进别的名字再折叠（`let base = seed; let mut state = base | 1;`）认不出；
#      折叠之后再混淆（`(seed | 1) ^ K`、`(seed | 1).wrapping_mul(K)`）认得。只判表达式形态，不追这个值拿去做什么。
#   ② 只认本文件里 `struct X { 字段: 整型 }` 声明的字段；局部变量、非整型字段、别的 crate 的结构体、宏展开的字段罩不到。
#      用非字面量初始化（`field: total`）算写过。C60 的另一半（每个进结论的读数要有一次让它变号的故障注入自证）要人做。
#   只缩不涨按每个文件的登记行数比（行号会随无关改动漂），同一个文件里删一行再加一行看不出来。
#
# 样本：red 里 e8_seed_and_readings.rs 同时犯两条，另带一张挂在认不出的欠账表上的豁免表、
# 一份住在 crates/singlefs-checker-tier/src/bin/ 下同样折叠了种子的装置，豁免表基准那一版的 blob 被 setup.sh 从对象库里删掉，必须判红；
# green 是同一份源码加上两张对得上的豁免表，必须判绿（别的格的实验二进制一处都不犯，扫到的文件数随之是 4）。
cell_reading_discipline() {
base=""
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  LIB_CHANGED_PATHS="$STAGE_REPOSITORY/research/scripts/changed-paths.sh"
  # shellcheck source=../../research/scripts/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
  base="$(gate_diff_base gate)"
fi
python3 - "$base" "$OWED_LIBRARY" "$STAGE_DIRECTORY/lib/rejection-blocks.py" <<'PY'
import glob, importlib.util, os, re, subprocess, sys

base = sys.argv[1]
owed_library_spec = importlib.util.spec_from_file_location("owed", sys.argv[2])
owed_library = importlib.util.module_from_spec(owed_library_spec)
owed_library_spec.loader.exec_module(owed_library)
rejection_blocks_spec = importlib.util.spec_from_file_location("rejection_blocks", sys.argv[3])
rejection_blocks = importlib.util.module_from_spec(rejection_blocks_spec)
rejection_blocks_spec.loader.exec_module(rejection_blocks)
BIN_GLOBS = ("research/e7-index-bench/src/bin/*.rs", "crates/singlefs-checker-tier/src/bin/e[0-9]*.rs")
BIN_GLOB = " 与 ".join(BIN_GLOBS)
OWED_PATH = ".claude/kb/checks-owed.md"
SEED_LAG = ".claude/gate.d/experiment-seed-fold-lag.tsv"
READING_LAG = ".claude/gate.d/experiment-constant-reading-lag.tsv"

INTEGER = r"(?:u8|u16|u32|u64|u128|usize|i8|i16|i32|i64|i128|isize)"
# 名字带 seed 的标识符紧跟 `| 1` 或 `& !1`。`1` 后面允许一个整型后缀，
# 不允许再跟数字或下划线——`seed | 16` 与 `seed | 1_000` 不是这个形态。
SEED_FOLD = re.compile(r"(?<![\w.])(\w*[sS]eed\w*)\s*(?:\|\s*1|&\s*!\s*1)(?:" + INTEGER + r")?(?![\w.])")
SEED_IDENT = re.compile(r"(?<![\w.])\w*[sS]eed\w*(?![\w])")
STRUCT_HEAD = re.compile(r"\bstruct\s+(\w+)\s*(?:<[^{};]*>)?\s*\{")
STRUCT_FIELD = re.compile(r"(?:^|,)\s*(?:pub\s+)?(\w+)\s*:\s*" + INTEGER + r"\s*(?=,|$)", re.M)
LITERAL_ZERO = re.compile(r"^0(?:" + INTEGER + r")?$")

failures = []          # 每项是一块要打印的拒绝：(摘要, 明细列表, 出路列表)
def reject(summary, details, steps):
    failures.append((summary, details, steps))

def blank_out(text):
    """把注释与字符串字面量换成同样长度的空白，行号与列号都不变。"""
    out, index, size = [], 0, len(text)
    while index < size:
        char = text[index]
        if char == "/" and index + 1 < size and text[index + 1] == "/":
            end = text.find("\n", index)
            end = size if end < 0 else end
            out.append(" " * (end - index)); index = end
        elif char == "/" and index + 1 < size and text[index + 1] == "*":
            depth, end = 1, index + 2
            while end < size and depth:
                if text[end] == "/" and end + 1 < size and text[end + 1] == "*": depth += 1; end += 2
                elif text[end] == "*" and end + 1 < size and text[end + 1] == "/": depth -= 1; end += 2
                else: end += 1
            out.append("".join(c if c == "\n" else " " for c in text[index:end])); index = end
        elif char == '"':
            end = index + 1
            while end < size:
                if text[end] == "\\": end += 2; continue
                if text[end] == '"': end += 1; break
                end += 1
            out.append("".join(c if c == "\n" else " " for c in text[index:end])); index = end
        else:
            out.append(char); index += 1
    return "".join(out)

# ── 被扫集合：一次扫描，两条判据与「没查的是哪些」都从它现算 ───────────────
paths = sorted(path for pattern in BIN_GLOBS for path in glob.glob(pattern))
if not paths:
    print(f"  ✗ {BIN_GLOB} 一个文件都没扫到")
    print("     → 怎么办：实验 bin 目录搬了家就同步改这一格里的 BIN_GLOBS（.claude/rules/path-moves.md）；")
    print("               扫到 0 个对象而报绿，与判过了在门禁输出里一模一样。")
    sys.exit(1)

sources, unreadable = {}, []
for path in paths:
    try:
        sources[path] = blank_out(open(path, encoding="utf-8").read())
    except OSError as error:
        unreadable.append(f"{path}：{error}")

# ① 折叠写法：文件 → [(行号, 原样片段)]
seed_folds, seed_identifier_count, files_without_seed = {}, 0, []
for path, text in sources.items():
    seed_identifier_count += len(SEED_IDENT.findall(text))
    if not SEED_IDENT.search(text):
        files_without_seed.append(os.path.basename(path))
    hits = []
    for line_number, line in enumerate(text.split("\n"), 1):
        hits.extend((line_number, match.group(0).strip()) for match in SEED_FOLD.finditer(line))
    if hits:
        seed_folds[path] = hits

# ② 恒为字面量 0 的整型字段：文件 → [(结构体.字段, 初始化处数)]
constant_readings, field_count, files_without_field = {}, 0, []
for path, text in sources.items():
    declared = {}
    for head in STRUCT_HEAD.finditer(text):
        # 花括号要数着配对：拿「下一个 }」凑合，一行写完的结构体会一路吃到别的结构体的收尾，把别人的字段记到自己名下。
        depth, cursor = 1, head.end()
        while cursor < len(text) and depth:
            if text[cursor] == "{": depth += 1
            elif text[cursor] == "}": depth -= 1
            cursor += 1
        if depth:
            continue
        for field in STRUCT_FIELD.finditer(text[head.end():cursor - 1]):
            declared.setdefault(field.group(1), head.group(1))
    field_count += len(declared)
    if not declared:
        files_without_field.append(os.path.basename(path))
    frozen = []
    for field, owner in sorted(declared.items()):
        name = re.escape(field)
        prefix = r"(?<![\w.])(?:\w+\s*\.\s*)*" + name
        compound = re.search(prefix + r"\s*(?:[-+*/|&^%]=|<<=|>>=)", text)
        assigned = [match.group(1).strip() for match in re.finditer(prefix + r"\s*=(?!=)\s*([^;,}\n]+)", text)]
        borrowed = re.search(r"&\s*mut\s+(?:\w+\s*\.\s*)*" + name + r"\b", text)
        if compound or borrowed or any(not LITERAL_ZERO.match(value) for value in assigned):
            continue
        initialised = [match.group(1).strip() for match in re.finditer(r"(?<![\w.])" + name + r"\s*:\s*([^,}\n]+)", text)]
        initialised = [value for value in initialised if not re.fullmatch(INTEGER, value)]
        if initialised and all(LITERAL_ZERO.match(value) for value in initialised):
            frozen.append((f"{owner}.{field}", len(initialised)))
    if frozen:
        constant_readings[path] = frozen

if unreadable:
    reject(f"{len(unreadable)} 个实验源码读不动，这一轮对它们两条判据都没跑：",
           unreadable,
           ["怎么办：把读不动的原因修掉（权限、编码）再跑；读不动的文件在这一格里既不算判过也不算判红，",
            "          而一个整批读不动的目录会让两条判据都扫到 0 项，末尾照样报绿。"])

# ── 豁免表 ─────────────────────────────────────────────────────────────
owed_table = owed_library.read_owed_table(OWED_PATH)
owed_open = set(owed_table.open_names)
owed_unrecognised = owed_table.file_found and not owed_table.paid_heading_found

def read_lag(path, label):
    """读一张豁免表，返回 {(文件, 定位): (欠账编号, 行号)}；形状坏了的行当场登记成拒绝。"""
    rows, malformed = {}, []
    if not os.path.isfile(path):
        return rows
    for line_number, line in enumerate(open(path, encoding="utf-8"), 1):
        line = line.rstrip("\n")
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 4 or not all(field.strip() for field in fields):
            malformed.append(f"第 {line_number} 行：{line}")
            continue
        source, locator, number, _reason = (field.strip() for field in fields)
        rows[(source, locator)] = (number, line_number)
    if malformed:
        reject(f"{label} 里 {len(malformed)} 行不是四列、或有空格子：", malformed,
               [f"怎么办：{path} 每行四列用制表符分隔：文件路径、定位、欠账编号、为什么还没改；四列都要有内容，理由不许省。"])
    return rows

def baseline_counts(path):
    """基准提交里这张表每个文件登记了几行；表在基准里不存在时返回 None（这次是初次登记）。"""
    if not base:
        return None
    # 「基准里没有这张表」与「git 取不到基准那一版」分开判：后者当成初次登记，只缩不涨就静默不比了
    listed = subprocess.run(["git", "-c", "core.quotepath=false", "ls-tree", "--name-only", base, "--", path],
                            capture_output=True, text=True)
    if listed.returncode != 0:
        reject(f"取不到基准 {base} 里的 {path}（git ls-tree 退 {listed.returncode}），「只缩不涨」这一条没比：",
               [listed.stderr.strip()[:200]],
               ["怎么办：按上面 git 的报错修好仓库状态（基准要存在、对象库没坏）再跑；取不到基线不是「初次登记」。"])
        return None
    if not listed.stdout.strip():
        return None
    result = subprocess.run(["git", "show", f"{base}:{path}"], capture_output=True, text=True)
    if result.returncode != 0:
        reject(f"读不出基准 {base} 里的 {path}（git show 退 {result.returncode}），「只缩不涨」这一条没比：",
               [result.stderr.strip()[:200]],
               ["怎么办：按上面 git 的报错修好仓库状态（多半是对象库缺了那一份）再跑；读不出基线不是「初次登记」。"])
        return None
    counts = {}
    for line in result.stdout.split("\n"):
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 4:
            continue
        counts[fields[0].strip()] = counts.get(fields[0].strip(), 0) + 1
    return counts

def judge(label, owed_number, lag_path, violations, locator_of, what_to_fix, how_to_fix):
    """一条判据一段：没豁免的违规、指向空处的编号、对不上的登记行、涨了的登记行，四样都判完。"""
    lag = read_lag(lag_path, label)
    excused, unexcused = [], []
    for path, hits in sorted(violations.items()):
        for hit in hits:
            locator = locator_of(hit)
            if (path, locator) in lag:
                excused.append((path, locator))
            else:
                unexcused.append(f"{path}:{locator}    {hit[1] if isinstance(hit[1], str) else ''}".rstrip())
    if unexcused:
        reject(f"{label}：{len(unexcused)} 处违规没有登记在豁免表里：", unexcused,
               [f"怎么办：两条出路，别两边都不做。① 就地改掉——{how_to_fix}；",
                "               改了就要重跑这个实验并按新产物逐个回对正文引的数（门禁 checker-tier-research-build-and-replay 会逐字节判红直到重跑）。",
                f"               ② 今天改不动的，登记进 {lag_path}：四列写文件路径、定位、{owed_number}、为什么还没改。",
                f"               ⚠️ 这张表只缩不涨——新写的实验源码一律走出路 ①，{what_to_fix}。"])
    if lag and owed_unrecognised:
        reject(f"{label}：豁免表有 {len(lag)} 行要核欠账编号，而 {OWED_PATH} 里认不出「### 已还清」那一行标题，分不出哪些账还开着", [],
               ["怎么办：欠账表按「### 已还清」整行标题切成开着与还清两段（.claude/gate.d/lib-owed.py）；标题改了名或丢了就改回来，",
                "               别让这一格对着一张认不出的表判——认不出时连历史版本节里的表格行都会被算成开着的账。"])
    dangling = [f"{source}:{locator} 挂的是 {number}（第 {line_number} 行）"
                for (source, locator), (number, line_number) in sorted(lag.items())
                if number not in owed_open and not owed_unrecognised]
    if dangling:
        reject(f"{label}：豁免表里 {len(dangling)} 行挂的欠账编号不在 {OWED_PATH} 欠着那张表里：", dangling,
               ["怎么办：登记一条豁免，等于承认这一处今天还没改，那笔账要有人排期。",
                f"               编号写成欠着那张表里开着的一行（这一条是 {owed_number}）；",
                "               那笔账已经还清了，就把这些豁免行一起删掉——账还清了就不该再有豁免。"])
    live = {(path, locator_of(hit)) for path, hits in violations.items() for hit in hits}
    stale = [f"{source}:{locator}（第 {line_number} 行）"
             for (source, locator), (_number, line_number) in sorted(lag.items())
             if (source, locator) not in live]
    if stale:
        reject(f"{label}：豁免表里 {len(stale)} 行对不上这一轮扫出来的任何一处违规：", stale,
               ["怎么办：这一处已经改好了就删掉这一行——留着的豁免行会让人以为那个文件还欠着，",
                "               而它其实已经干净了（这张表只缩不涨，改好一处就删一行）；",
                "               只是定位挪了（行号随无关改动漂、字段改了名），就把定位那一列改成现在的值。"])
    baseline = baseline_counts(lag_path)
    grown = []
    if baseline is not None:
        current = {}
        for source, _locator in lag:
            current[source] = current.get(source, 0) + 1
        for source, count in sorted(current.items()):
            was = baseline.get(source, 0)
            if count > was:
                grown.append(f"{source}：基准 {base} 里 {was} 行，现在 {count} 行")
    if grown:
        reject(f"{label}：豁免表涨了——{len(grown)} 个文件的登记行数比基准多：", grown,
               ["怎么办：这张表只缩不涨。新冒出来的违规一律就地改，不许加一行豁免把它按下去——",
                "               加得了一行，这一格对新写的实验源码就什么都拦不住了。",
                f"               真有非加不可的理由，那是一次规则变更：先改 {lag_path} 的表头注释与这一格的判据，说清为什么。"])
    return len(excused), len(lag), baseline is not None

seed_excused, seed_rows, seed_compared = judge(
    "C59（种子折叠成同一个状态）", "C59", SEED_LAG, seed_folds,
    lambda hit: str(hit[0]),
    "它们一处都不许出现在这张表里",
    "把 `seed | 1` 改成先乘法混淆再置位，例 `seed.wrapping_mul(0xD1B5_4A32_D192_ED03) | 1`")
reading_excused, reading_rows, reading_compared = judge(
    "C60（恒定读数没有故障注入自证）", "C60", READING_LAG, constant_readings,
    lambda hit: hit[0],
    "它们一处都不许出现在这张表里",
    "要么让这个字段真的被写（把它算出来），要么删掉它——一个恒为 0 的读数不该进产物")

if failures:
    rejection_blocks.print_blocks(failures, continuation_indent="       ")
    sys.exit(1)

def listing(names):
    """一行塞几个名字，按显示宽度折行——名字本身不许被折断，折断的文件名 grep 不出来。"""
    lines, current = [], ""
    for index, name in enumerate(names):
        piece = name + ("、" if index + 1 < len(names) else "")
        if current and len(current) + len(piece) > 100:
            lines.append("        " + current); current = ""
        current += piece
    if current:
        lines.append("        " + current)
    return "\n".join(lines)

seed_hits = sum(len(hits) for hits in seed_folds.values())
reading_hits = sum(len(hits) for hits in constant_readings.values())
compared = "、".join(name for name, done in (("C59", seed_compared), ("C60", reading_compared)) if done)
compared = compared or "两张豁免表在基准里都还不存在，这一次是初次登记，没有基线可比"
print(f"  ✓ 实验源码纪律：扫了 {len(paths)} 个文件（读不动的 0 个）；"
      f"C59 查了 {seed_identifier_count} 处名字带 seed 的标识符，折叠写法 {seed_hits} 处（豁免表 {seed_rows} 行，放行 {seed_excused} 处）；"
      f"C60 查了 {field_count} 个整型结构体字段，恒为字面量 0 的 {reading_hits} 个（豁免表 {reading_rows} 行，放行 {reading_excused} 处）；"
      f"只缩不涨（基准 {base or '不是 git 仓，这一条没跑'}）：{compared}")
print(f"    C59 对这 {len(files_without_seed)} 个文件没有对象可判（一处名字带 seed 的标识符都没有）：")
print(listing(files_without_seed) if files_without_seed else "        （没有）")
print(f"    C60 对这 {len(files_without_field)} 个文件没有对象可判（一个整型结构体字段都没有）：")
print(listing(files_without_field) if files_without_field else "        （没有）")
PY
}

# ── 格 unsafe-allowances：crates/ 里放开 unsafe 的地方，与登记表逐处对上
#
# 登记表 .claude/gate.d/unsafe-allowances.tsv 一个 crate 一行（六段写法在表头）。先判表本身：放开了的行，第 5 段理由不是 - 且不少于 20 个字，
# 第 6 段出处是仓里真有的文件路径、或「用户 YYYY-MM-DD 弹窗定」加那个路径；没放开的行两段都写 -。只判理由在、够长，出处判文件存在。
# 再扫 crates/<crate>/ 下所有 .rs（跳过 target/），先抹掉注释与字符串（行号不变），判：
#   ① 根 crates/<crate>/src/lib.rs 写了 `#![forbid(unsafe_code)]` 或 `#![deny(unsafe_code)]`，什么都不写判红；登记表只许 forbid 的 crate 写 deny 判红；
#   ② 放开的个数：`allow` / `expect` / `warn` 里带 `unsafe_code` 的属性（含 cfg_attr 里的）数一遍，根是 forbid 时必须 0 处，
#      根是 deny 时必须恰好 1 处；`unsafe` 关键字（unsafe 块、unsafe fn、unsafe impl）同样数，数目与放开的处数相同；
#   ③ 那一处放开的位置：属性紧挨着的函数是登记表第 4 段、在登记表第 3 段那个文件里，不是整文件的内层属性 `#![allow(...)]`；
#      `unsafe` 关键字落在那个函数体里；
#   ④ 每个 `unsafe` 上方连着的 `//` 注释里有一行以 `// SAFETY:` 起头；放开了的 crate 里，从那一行到 `unsafe` 上一行的注释文字
#      要提到文件描述符（描述符 / file descriptor / fd）、偏移（偏移 / offset）、长度（长度 / length / len）；
#   ⑤ crates/ 下出现登记表里没有的 crate 目录判红；登记表里的 crate 目录不在仓里，跳过那一行并在成功那句里点名。
# 射程：tests/、benches/、src/bin/ 各是自己的 crate 根，lib.rs 的 forbid 管不到它们（编译器也不拦），所以 ②③④ 数 crate 目录下全部 .rs。
# 抹字符串认 `"…"`、原始字符串与字符字面量；宏展开出来的 unsafe 认不出；research/ 不在射程。
# 不判的：理由与 SAFETY 前提写得对不对、`unsafe` 块里的代码对不对（靠 review）。
# 样本（fixtures/code-source-discipline.sh/ 下，各自带登记表与它的出处文件 records/sample-decision.md）：
#   unsafe-allowances-green               core 根 forbid 零处放开、另一个 crate 也 forbid
#   unsafe-allowances-deny-one-green      deny 恰一处
#   unsafe-allowances-two-allows-red      两处 allow 叠在同一个函数上
#   unsafe-allowances-wrong-function-red  allow 与 unsafe 在别的函数上
#   unsafe-allowances-no-safety-red       没有 SAFETY 注释
#   unsafe-allowances-root-blank-red      core 根什么都不写
#   unsafe-allowances-other-crate-red     harness 的 tests/ 里 allow 加 unsafe，根仍是 forbid
#   unsafe-allowances-reason-empty-red    放开了而理由栏写 -
#   unsafe-allowances-source-missing-red  出处指向仓里没有的文件
#   unsafe-allowances-safety-vague-red    有 SAFETY 注释但没提描述符、偏移、长度
# 弄坏开关 GATE_UNSAFE_ALLOWANCES_BREAK 各项在文件头。
cell_unsafe_allowances() {
  [[ -d crates ]] || { echo "  ! 没有 crates/ 目录，这一格无对象可判"; exit 77; }

  python3 - <<'PY'
import os, re, sys

BREAK = os.environ.get("GATE_UNSAFE_ALLOWANCES_BREAK", "")
REGISTRY_PATH = ".claude/gate.d/unsafe-allowances.tsv"
REASON_MINIMUM = 20
SAFETY_PREMISES = (("文件描述符", re.compile(r"描述符|file descriptor|\bfd\b", re.I)),
                   ("偏移", re.compile(r"偏移|offset", re.I)),
                   ("长度", re.compile(r"长度|length|\blen\b", re.I)))
problems = []


def reject(summary, details, howto):
    problems.append((summary, details, howto))


def finish():
    for summary, details, howto in problems:
        print(f"  ✗ {summary}")                      # gate-lint:summary
        for detail in details:
            print(f"     {detail}")                  # gate-lint:detail
        print(f"     → 怎么办：{howto}")
    if problems:
        sys.exit(1)


def blank(piece):
    return re.sub(r"[^\n]", " ", piece)


RAW_STRING_OPEN = re.compile(r'r(#*)"')


def strip_comments_and_strings(text):
    """注释整段抹成空格，字符串只抹里面（引号留着），换行都留着，行号不变。"""
    out, index, length = [], 0, len(text)
    while index < length:
        char = text[index]
        if text.startswith("//", index):
            end = text.find("\n", index)
            end = length if end < 0 else end
            out.append(blank(text[index:end])); index = end
        elif text.startswith("/*", index):
            depth, end = 1, index + 2
            while end < length and depth:
                if text.startswith("/*", end):
                    depth += 1; end += 2
                elif text.startswith("*/", end):
                    depth -= 1; end += 2
                else:
                    end += 1
            out.append(blank(text[index:end])); index = end
        elif char == "r" and RAW_STRING_OPEN.match(text, index) and (
            index == 0 or not (text[index - 1].isalnum() or text[index - 1] == "_")
            or (text[index - 1] == "b" and (index == 1 or not (text[index - 2].isalnum() or text[index - 2] == "_")))
        ):
            opening = RAW_STRING_OPEN.match(text, index)
            closing = '"' + opening.group(1)
            close_at = text.find(closing, opening.end())
            if close_at < 0:
                out.append(blank(text[index:])); index = length
            else:
                out.append(text[index:opening.end()] + blank(text[opening.end():close_at]) + closing)
                index = close_at + len(closing)
        elif char == '"':
            end = index + 1
            while end < length and text[end] != '"':
                end += 2 if text[end] == "\\" else 1
            end = min(end, length)
            out.append('"' + blank(text[index + 1:end]) + ('"' if end < length else ""))
            index = min(end + 1, length)
        elif char == "'":
            if index + 1 < length and text[index + 1] == "\\":
                end = text.find("'", index + 3)
                end = length if end < 0 else end + 1
                out.append(blank(text[index:end])); index = end
            elif index + 2 < length and text[index + 2] == "'" and text[index + 1] != "'":
                out.append("'" + blank(text[index + 1]) + "'"); index += 3
            else:
                out.append(char); index += 1
        else:
            out.append(char); index += 1
    return "".join(out)


def line_of(text, position):
    return text.count("\n", 0, position) + 1


ATTRIBUTE_OPEN = re.compile(r"#\s*(!?)\s*\[")


def attributes(cleaned):
    """逐个 #[...] / #![...]：(起点, 终点, 是不是内层属性, 里面的文本)。"""
    found, index = [], 0
    while True:
        opening = ATTRIBUTE_OPEN.search(cleaned, index)
        if not opening:
            return found
        depth, end = 1, opening.end()
        while end < len(cleaned) and depth:
            depth += {"[": 1, "]": -1}.get(cleaned[end], 0)
            end += 1
        found.append((opening.start(), end, opening.group(1) == "!", cleaned[opening.end():end - 1]))
        index = end


RELAX = re.compile(r"\b(allow|expect|warn)\s*\([^)]*\bunsafe_code\b")
LEVEL = re.compile(r"\b(forbid|deny)\s*\([^)]*\bunsafe_code\b")
FUNCTION_AFTER_ATTRIBUTES = re.compile(
    r"\s*(?:#\s*\[[^\]]*\]\s*)*(?:pub(?:\s*\([^)]*\))?\s+)?(?:(?:const|async|unsafe|safe)\s+|extern\s+\"[^\"]*\"\s+)*fn\s+([A-Za-z_][A-Za-z0-9_]*)"
)
UNSAFE_WORD = re.compile(r"\bunsafe\b")


def function_body_span(cleaned, name):
    """名为 name 的函数体的 [起, 止)；找不到返回 None。"""
    declaration = re.search(r"\bfn\s+" + re.escape(name) + r"\b", cleaned)
    if not declaration:
        return None
    start = cleaned.find("{", declaration.end())
    if start < 0:
        return None
    depth, end = 0, start
    while end < len(cleaned):
        depth += {"{": 1, "}": -1}.get(cleaned[end], 0)
        end += 1
        if depth == 0:
            break
    return start, end


def safety_comment(original_lines, unsafe_line):
    """unsafe 所在行的上方，连着的 // 注释里有一行以 // SAFETY: 起头：返回从那一行到 unsafe 上一行的注释文字；没有返回 None。"""
    number = unsafe_line - 2
    while number >= 0 and original_lines[number].lstrip().startswith("//"):
        if original_lines[number].lstrip().startswith("// SAFETY:"):
            return "\n".join(original_lines[number:unsafe_line - 1])
        number -= 1
    return None


# ── 登记表 ──
registry = {}
if not os.path.isfile(REGISTRY_PATH):
    reject(f"没有登记表 {REGISTRY_PATH}", [], f"照 {REGISTRY_PATH} 的表头写法建这张表：crates/ 下每个 crate 一行；表是这一格判的依据，不能没有。")
    finish()
for number, raw in enumerate(open(REGISTRY_PATH, encoding="utf-8").read().splitlines(), 1):
    if not raw.strip() or raw.lstrip().startswith("#"):
        continue
    fields = raw.split("\t")
    if len(fields) != 6 or not all(field.strip() for field in fields):
        reject(f"{REGISTRY_PATH}:{number} 不是六段（crate、根允许的级别、放开的文件、放开的函数、理由、出处），段间用制表符，每段都有内容",
               [raw], "照表头写法改这一行；没有放开的文件、函数、理由、出处四段写 -。")
        continue
    crate, levels, allowed_file, allowed_function, reason, source = (field.strip() for field in fields)
    bad_levels = [level for level in levels.split(",") if level not in ("forbid", "deny")]
    if bad_levels or crate in registry or (("deny" in levels.split(",")) != (allowed_file != "-")) or ((allowed_file == "-") != (allowed_function == "-")):
        reject(f"{REGISTRY_PATH}:{number} 这一行自相矛盾或重复登记", [raw],
               "级别只写 forbid 或 forbid,deny；写了 deny 才有放开的文件与函数，没有 deny 两段都写 -；同一个 crate 只登记一行。")
        continue
    if allowed_file == "-":
        if reason != "-" or source != "-":
            reject(f"{REGISTRY_PATH}:{number} {crate} 没有放开，理由与出处两段却写了内容", [raw],
                   "没有放开的行，理由与出处都写 -。")
            continue
    else:
        if BREAK != "reason-ignored" and (reason == "-" or len(reason) < REASON_MINIMUM):
            reject(f"{REGISTRY_PATH}:{number} {crate} 放开了 unsafe，理由栏{'是 -' if reason == '-' else f'只有 {len(reason)} 个字'}（不少于 {REASON_MINIMUM} 个字）", [raw],
                   "理由栏写为什么非放开不可、不放开的替代为什么不行；理由写不出来的放开不登记。")
            continue
        if BREAK != "source-ignored":
            source_words = source.split()
            source_path = source_words[-1]
            source_prefix = " ".join(source_words[:-1])
            if source == "-":
                source_problem = "出处栏是 -，放开了的行要写谁哪天定的"
            elif source_prefix and not re.fullmatch(r"用户 \d{4}-\d{2}-\d{2} 弹窗定", source_prefix):
                source_problem = f"出处栏「{source}」的写法不对：只写仓里的路径，或「用户 YYYY-MM-DD 弹窗定 <路径>」"
            elif os.path.isabs(source_path) or ".." in source_path.split("/") or not os.path.isfile(source_path):
                source_problem = f"出处栏指向的 {source_path} 不是仓里的文件（要写仓内的相对路径、文件存在）"
            else:
                source_problem = None
            if source_problem:
                reject(f"{REGISTRY_PATH}:{number} {crate} 的出处不成立：{source_problem}", [raw],
                       "出处写记录这次定案的仓内文件路径，或「用户 YYYY-MM-DD 弹窗定」加那份记录的路径；记录文件要真在仓里。")
                continue
    registry[crate] = (levels.split(","), allowed_file, allowed_function)
finish()

crate_names = sorted(entry for entry in os.listdir("crates") if os.path.isdir(os.path.join("crates", entry)))
file_count, scanned = 0, []
for name in crate_names:
    if name not in registry:
        reject(f"crates/{name} 没在登记表里", [f"{REGISTRY_PATH} 没有 {name} 这一行"],
               f"新 crate 先在 {REGISTRY_PATH} 登记一行（根要写 forbid，要放开 unsafe 的走三方、级别写 forbid,deny 并写明文件与函数）。")
        continue
    levels, allowed_file, allowed_function = registry[name]
    has_allowance = allowed_file != "-"
    if BREAK == "other-crates-ignored" and not has_allowance:
        continue
    files = {}
    for directory, subdirectories, names in os.walk(os.path.join("crates", name)):
        subdirectories[:] = sorted(entry for entry in subdirectories if entry != "target" and not entry.startswith("."))
        for file_name in sorted(names):
            if file_name.endswith(".rs"):
                path = os.path.join(directory, file_name)
                original = open(path, encoding="utf-8", errors="replace").read()
                files[path] = (original, strip_comments_and_strings(original))
    file_count += len(files)
    root_path = f"crates/{name}/src/lib.rs"
    relaxes, unsafes, level = [], [], None
    for path, (original, cleaned) in files.items():
        for start, end, inner, body in attributes(cleaned):
            if RELAX.search(body):
                following = FUNCTION_AFTER_ATTRIBUTES.match(cleaned, end)
                relaxes.append((path, line_of(cleaned, start), None if inner else (following.group(1) if following else None)))
            if path == root_path and inner and LEVEL.search(body):
                found = LEVEL.search(body).group(1)
                level = "forbid" if (found == "forbid" or level == "forbid") else "deny"
        for match in UNSAFE_WORD.finditer(cleaned):
            unsafes.append((path, line_of(cleaned, match.start()), match.start()))
    scanned.append((name, len(files), level, len(relaxes), len(unsafes)))

    if root_path not in files:
        reject(f"{name} 没有 crate 根 {root_path}", [], "登记表与判法都按 src/lib.rs 当根；crate 改了布局就先改这一格。")
        continue
    if BREAK != "root-level-ignored":
        if level is None:
            reject(f"{root_path} 没写 #![forbid(unsafe_code)] 也没写 #![deny(unsafe_code)]", [],
                   "根上写 #![forbid(unsafe_code)]；登记了唯一一处放开的 crate 写 #![deny(unsafe_code)]。")
        elif level not in levels:
            reject(f"{root_path} 的级别是 {level}，登记表只许 {'、'.join(levels)}", [],
                   f"根改回 forbid；要放开 unsafe 先改 {REGISTRY_PATH} 的行并走三方。")
    if level is None or level not in levels:
        continue
    expected = 1 if level == "deny" else 0
    if BREAK != "count-ignored":
        if len(relaxes) != expected:
            reject(f"{name} 放开 unsafe_code 的地方 {len(relaxes)} 处，根是 {level}、应当 {expected} 处",
                   [f"{path}:{line}" for path, line, function in relaxes] or ["（没有）"],
                   f"根写 forbid 就一处都不许放开；根写 deny 就恰好放开登记的那一处。新增一处放开要改 {REGISTRY_PATH} 的登记并走三方，不是加一个 allow。")
        if len(unsafes) != expected:
            reject(f"{name} 的 unsafe 关键字 {len(unsafes)} 处，根是 {level}、应当 {expected} 处",
                   [f"{path}:{line}" for path, line, position in unsafes] or ["（没有）"],
                   f"tests/、benches/、src/bin/ 的文件各是自己的 crate 根，lib.rs 的 forbid 管不到；不要在那里写 unsafe。要新增走 {REGISTRY_PATH} 的登记与三方。")
    if BREAK != "placement-ignored":
        for path, line, function in relaxes:
            if function is None:
                reject(f"{path}:{line} 是整文件的内层属性或后面没有函数，不是登记的那一处",
                       [], f"放开只许是 #[allow(unsafe_code)] 紧挨着 {allowed_function}；要换地方改 {REGISTRY_PATH} 的登记并走三方。")
            elif (path, function) != (allowed_file, allowed_function):
                registered = f"登记的是 {allowed_file} 的 {allowed_function}" if has_allowance else f"{name} 在登记表里没有放开"
                reject(f"{path}:{line} 放开 unsafe_code 的函数是 {function}，{registered}",
                       [], f"放开只许在登记的那一处；要新增或换地方改 {REGISTRY_PATH} 的登记并走三方。")
        if has_allowance and allowed_file in files:
            span = function_body_span(files[allowed_file][1], allowed_function)
            for path, line, position in unsafes:
                if path != allowed_file or span is None or not (span[0] <= position < span[1]):
                    reject(f"{path}:{line} 的 unsafe 不在登记的 {allowed_file} 的 {allowed_function} 函数体里", [],
                           f"unsafe 只许写在登记的那个函数里；要换地方改 {REGISTRY_PATH} 的登记并走三方。")
    for path, line, position in unsafes:
        comment = safety_comment(files[path][0].splitlines(), line)
        if comment is None:
            if BREAK != "safety-comment-ignored":
                reject(f"{path}:{line} 的 unsafe 上方没有 // SAFETY: 注释", [],
                       "在 unsafe 上方连着写 // SAFETY: 起头的注释，说清调用的前提为什么成立。")
        elif has_allowance and BREAK != "safety-premise-ignored":
            absent = [label for label, pattern in SAFETY_PREMISES if not pattern.search(comment)]
            if absent:
                reject(f"{path}:{line} 的 // SAFETY: 注释没写出它依赖的前提：没提到{'、'.join(absent)}", comment.splitlines(),
                       "SAFETY 注释要写出调用依赖的前提：文件描述符有效（取自哪里、活到什么时候）、偏移与长度在什么范围内（为什么不越界、不溢出）；"
                       "这一格只查这几个词有没有出现，写得对不对靠 review。")

finish()
missing = [name for name in registry if name not in crate_names]
summary = "；".join(f"{name} 根 {level}、放开 {relaxes} 处、unsafe {unsafes} 处、{count} 个 .rs" for name, count, level, relaxes, unsafes in scanned)
print(f"  ✓ crates/ 里放开 unsafe 的地方与登记表对上：扫了 {len(scanned)} 个 crate、{file_count} 个 .rs（{summary}）；登记了但仓里没有的 crate {len(missing)} 个：{'、'.join(missing) or '无'}")
PY
}

# ── 格 mutation-light-rows：crates/mutations.tsv 里改坏对象在 crates/singlefs-core/src/ 下的行里每个测试目标在轻型变异登记表里恰好一行、点名的行都在变异表里
# 判法只在 research/scripts/crates-mutation-light-rows.py check（门禁 checker-tier-crates-mutation-replay 的 --quick 读同一张表、先经同一份核法）；
# 登记表写法与挑法在 .claude/gate.d/crates-mutation-light-rows.tsv 表头，变异表的成形照 crates-mutation-rows.py 的 read_table。
# 被判的根里没有 crates/mutations.tsv：这一格本次无对象可判。
# 管不到的：挑的那一行改坏之后点名的测试真红不红（归门禁 checker-tier-crates-mutation-replay --quick 真跑）。
cell_mutation_light_rows() {
  if [[ ! -f "$ROOT/crates/mutations.tsv" ]]; then
    echo "  ! 没有 crates/mutations.tsv：轻型变异登记表这一格本次无对象可判"
    return 77
  fi
  python3 "$STAGE_REPOSITORY/research/scripts/crates-mutation-light-rows.py" check "$ROOT"
}

# ── 格 gpu-exemptions：不走 GPU 的计算与豁免表逐项对上（欠账 C626（走 GPU 的计算类没有登记），判决 m3-crash-enumeration-merge-r3 第三节 J5）
# 判法只在 research/scripts/gpu-exemptions-check.py check：豁免表 .claude/gate.d/gpu-exemptions.tsv 每一行四段（截名、代码标记名、为什么上不了 GPU、
# 用户点头日期），crates/ 下 .rs 里每一处 // gpu-exempt: <截名> 都有表行、每一行都有标记。被判的根里没有 crates/：这一格本次无对象可判。
# 管不到的：一截计算真在不在 GPU 上跑、标记写没写在真不走 GPU 的那一截上、为什么写得对不对，靠 review 与用户点头。
cell_gpu_exemptions() {
  if [[ ! -d "$ROOT/crates" ]]; then
    echo "  ! 没有 crates/：豁免表这一格本次无对象可判"
    return 77
  fi
  python3 "$STAGE_REPOSITORY/research/scripts/gpu-exemptions-check.py" check "$ROOT"
}

# ── 格 crash-point-replay-segments：崩溃点重放的按段登记成形（判决 m3-crash-enumeration-merge-r3 第三节 J10、M4 第 15 条）
# 判法只在 research/scripts/lib_crash_point_replay_items.py segments（入口 research/scripts/crash-point-replay.sh 开跑前调同一份）：
# .claude/gate.d/crash-point-replay-segments.tsv 每一行四段（项名、段号、段写数、段写表摘要），项名在合表 .claude/gate.d/crash-point-replay-items.tsv 里，
# 段号与段写数是非负整数且段写数大于 0，摘要是 64 位小写十六进制，同一项同一段只一行。
# 被判的根里合表与按段登记文件都没有：这一格本次无对象可判；合表在而按段登记文件不在判红（入口开跑前要读它）。
# 管不到的：摘要是不是那一段写表真的 sha256、段号与段写数对不对得上那条流（引擎跑起来才知道），靠引擎那一侧与 review。
cell_crash_point_replay_segments() {
  if [[ ! -f "$ROOT/.claude/gate.d/crash-point-replay-segments.tsv" && ! -f "$ROOT/.claude/gate.d/crash-point-replay-items.tsv" ]]; then
    echo "  ! 没有崩溃点重放的合表与按段登记文件：这一格本次无对象可判"
    return 77
  fi
  python3 "$STAGE_REPOSITORY/research/scripts/lib_crash_point_replay_items.py" segments "$ROOT"
}

stage_cells_run "$ROOT"
