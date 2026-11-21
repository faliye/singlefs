#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的 Rust 源码、测试文件名、kb 文本、登记表与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求；refused-tests 格另起 admission.py 的 test-plan 算键、读判定库，连不上或算不出报本次未跑
# gate-stage: 代码类的源码纪律：名字不只由空泛词拼成、测试文件与 #[test] 函数不按里程碑、欠账号起名；kb 登记的格式常量、条文集合与 feature bit 位号与代码逐个对得上；实验二进制有成形的变异表且锚点唯一、有钉绝对值的断言、种子不折叠与读数不恒为字面量 0；crates/ 里放开 unsafe 的地方与登记表逐处对上；不走 GPU 的计算与豁免表逐项对上；崩溃点重放的按段登记成形；代码里我们自己的名字不带序号与位置词；被验 crate 里不自定义、不改名出 Ghost / Tracked；拒算的测试函数与登记表逐条对上
# gate-fixture: yes
# gate-category: 代码类
# gate-similar: doc-text.sh 也是全仓扫写法，但判的是登记过的禁用写法，对象是全部文本文件；这里的 vague-names 与 test-file-names 两格判 .rs 里声明的名字由哪些词拼成、测试文件叫什么；ordinal-names 格判 crates/、scripts/、.claude/gate.d、.claude/hooks 下 .rs、.sh、.py 里的标识符、环境变量名与文件名按词边界含不含词表里的序号与位置词，注释与字符串先剥掉，不判文本（注释与文档里的旧名归 doc-text 的 term-renames 格，只认登记过的）
# gate-similar: model-differential-and-scenarios.sh 也调 scripts/crash/crash-case-check.py，但调的是 one-scenario 那一样、判用例里面的循环，归 harness 类；这里的 test-file-names 格只调 file-names 那一样，判的是文件名
# gate-similar: .claude/singlefs-ai-sop/scripts/naming-lint.sh 是上游的命名纪律，判单字母与常见缩写、词表写死在上游；空泛词表是本项目定的，上游这一批不发版，先放项目本地；序号与位置词表（ordinal-names 格）同样是本项目自己的，而且要判 .sh、.py，上游那份只扫 .rs
# gate-similar: doc-registries.sh 它判欠账表的形状、里程碑收口表收全开着的欠账号、总审核记录每行有去向，读 kb 与 records/；这里的 kb 对账三格拿登记表把决策分项正文里的名字与 Rust 枚举变体配对、把 format-const 标记的值与源码 const 比、把 feature bit 位号与代码常量比，reading-discipline 格按同一份 lib-owed.py 只判两张豁免表挂的欠账号还开着，对象是实验二进制源码；它判 kb 里字段表、字节布局表、段序列登记表与里程碑各步之间的对账，field-table-sums 那一格也经 lib-format-const.py 读标记，判的是字段表加和与结构总宽，不读代码；这里的 format-constants 格拿标记的值与 scripts/experiment-apparatus/、research/、crates/ 源码里的 const 比、旧值字面串不许留，不做算术；它的 declared-counts 那一格也按 lib-owed.py 读欠账表，只数得出欠着多少条；这里只拿开着的编号核豁免表
# gate-similar: checker-independence-and-sync.sh 它也经 lib-format-const.py 读标记、按 lib-owed.py 判布局滞后登记表点名的欠账号还开着，看的是这次改动里布局常量变了 checker 跟没跟；这里看此刻两边的值相等，对象是实验二进制源码与变异表
# gate-similar: crates-mutation-replay.sh 它按 crates/mutations.tsv 逐条改坏源码再跑点名的测试，要编译、跑几个钟头；这里的 mutation-tables 格经同一个模块 scripts/mutation/crates-mutation-rows.py 的 static-check 只做静态判，不跑变异
# gate-similar: doc-experiments.sh 它的 evidence-in-repo 格看这一轮改过的装置与变异表在 research/results/ 里有没有一份不比它旧的产物；这里判变异表本身成不成形、锚点对不对，与实验源码里的断言和读数，不看产物
# gate-similar: crash-point-replay.sh 它按条复用时对拒算的测试函数只报条数与理由、照跑不复用，不核有没有登记；这里的 refused-tests 格拿同一份 admission.py test-plan 的拒算行核登记表 refused-tests.tsv（拒算而没登记、登记了而不再拒算都红），判法放在 scripts/gate/refused-test-registry.py，不并进那一道（那一道的格判的是测试跑得过不过）
#
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就十七格按下面的次序全跑，格名写错退 2）：
# gate-cell: vague-names 函数名与类型名不只由空泛词拼成
# gate-cell: test-file-names 测试文件名不按里程碑、步号起
# gate-cell: test-function-names #[test] 函数名不按里程碑、步号、欠账号起
# gate-cell: format-constants 格式常量在 kb 与实验源码之间同步
# gate-cell: clause-enums 条文列出来的封闭集合，与代码里兑现它的那个枚举逐个成员对上
# gate-cell: feature-bits feature bit 位号：记账表与 D15 登记表逐行一致、不跳号，代码引用的位都在记账表里
# gate-cell: mutation-tables 每个实验二进制都有成形的变异表、锚点唯一，crates/mutations.tsv 的静态几样
# gate-cell: absolute-assertions 每个实验二进制都有钉绝对值的断言
# gate-cell: reading-discipline 实验源码的两条读数纪律：种子不许折叠，读数不许恒为字面量 0
# gate-cell: unsafe-allowances crates/ 里放开 unsafe 的地方与登记表逐处对上（登记的每个函数上恰好一处放开、函数里 unsafe 处数与登记相同，没登记放开的 crate 零处）
# gate-cell: gpu-exemptions 不走 GPU 的计算与豁免表逐项对上：代码里每一处 // gpu-exempt: <截名> 都有表行、每一行都有标记
# gate-cell: crash-point-replay-segments 崩溃点重放的按段登记成形：每行五段、项名在合表里、流名合写法、段号与段写数是非负整数且段写数大于 0、摘要是 64 位小写十六进制、同一条流同一段只一行
# gate-cell: crate-sources-reachable 每一份 .rs 都是某个 crate 的目标，或经 #[path]、mod、include! 引到，没有 cargo 不编的落空源文件
# gate-cell: ordinal-names 代码里我们自己的名字（函数、变量、常量、模块、测试、环境变量、文件与目录）不带序号与位置词
# gate-cell: data-names 字符串字面量（元组首列、=> 右侧、键= 的键名）与 kb 表格里名字形态的串也按命名纪律判：单字母、没登记的缩写判红
# gate-cell: verus-ghost-names 被验 crate（verus-pins.tsv 登记的）里不自己定义名叫 Ghost / Tracked 的类型，也不把 vstd 之外的东西 use … as 改名成这两个名字
# gate-cell: refused-tests 验证包里拒算的测试函数（单元键算不出、按条复用时照跑不复用）与拒算登记表逐条对上：拒算而没登记、登记了而已经不拒算都判红
#   vague-names          词表 .claude/naming-vague-words，还没改完的文件 .claude/naming-vague-exclude
#   test-file-names      跑 scripts/crash/crash-case-check.py --only file-names
#   test-function-names  跑 scripts/crash/crash-case-check.py --only test-names；立账时已在的存量 .claude/gate.d/test-function-names-exempt.tsv
#   format-constants     调 lib-format-const.py；冻结证据登记在 .claude/format-constants-exclude（一行 <路径>  # 理由）
#   clause-enums         按 .claude/gate.d/clause-enum-pairs.tsv 配对
#   feature-bits         .claude/kb/feature-bits.md 与 D15（格式冻结政策） 已定项 4
#   mutation-tables      scripts/mutation/tables/（表名对 scripts/experiment-apparatus 下顶层的 <名>.rs 或带根 main.rs 的装置目录 <名>/）与 crates/mutations.tsv（后者经 scripts/mutation/crates-mutation-rows.py static-check）
#   absolute-assertions  scripts/experiment-apparatus 下除 lib.rs、main.rs 之外的全部（顶层的 .rs 与带根 main.rs 的装置目录），crates/*/src/bin 下以 e<数字>_ 开头的 .rs 与目录
#   reading-discipline   C59 种子折叠、C60 恒为字面量 0 的读数，两张豁免表挂开着的欠账；扫 scripts/experiment-apparatus 下顶层的 .rs 与装置目录、验证包与角色登记为实验装置的包的 src/bin 下 e<号> 起头的 .rs 与目录
#   unsafe-allowances    .claude/gate.d/unsafe-allowances.tsv 与 crates/ 下全部 .rs
#   gpu-exemptions       .claude/gate.d/gpu-exemptions.tsv 与 crates/ 下全部 .rs（经 scripts/crash/gpu-exemptions-check.py check）
#   crash-point-replay-segments  .claude/gate.d/crash-point-replay-segments.tsv（五列：项名、流名、段号、段写数、段写表摘要）与合表 .claude/gate.d/crash-point-replay-items.tsv（经 scripts/crash/lib_crash_point_replay_items.py segments）
#   crate-sources-reachable  scripts/experiment-apparatus 与 crates 下全部 .rs（经 scripts/gate/crate-sources-reachable.py，判法在它文件头）
#   ordinal-names        词表 .claude/naming-ordinal-tokens，还没改完的文件 .claude/naming-ordinal-exclude；扫 crates/、scripts/、.claude/gate.d、.claude/hooks 下 git 列出的 .rs、.sh、.py
#   data-names           scripts/experiment-apparatus/**/*.rs（顶层的装置与装置目录里的子模块）与 crates/*/src/**/*.rs 的字符串字面量、.claude/kb/**/*.md 的表格单元格（经 scripts/gate/data-name-lint.py，判法在它文件头）；登记表 .claude/abbreviations，存量排除表 .claude/data-name-lint-exclude
#   verus-ghost-names    .claude/gate.d/verus-pins.tsv 的 crate 行登记的目录，src/ 下全部 .rs
#   refused-tests        .claude/gate.d/refused-tests.tsv 与 admission.py test-plan 对验证包不带特性那一趟打的 refused 行（判法在 scripts/gate/refused-test-registry.py 文件头）
# 每格的判据、射程、管不到的写在它自己那一段注释里。
# 参数解析、样本根的 .gate-cells、每格一个子 shell、汇总与退出码在共用库 lib/stage-cells.sh。
#
# 格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释，按格组分在 .claude/gate.d/lib/ 下的三份库里；这一份在参数解析与 cd 到项目根之后按下面的次序 source 它们，
# 只留格名表、stage_cell 登记、参数解析与汇总。格的出路里写的「code-source-discipline.sh 里这一格函数上方的注释」，就在下面对应的那份库里：
# 库 lib/code-source-discipline-naming-and-constants-cells.sh：vague-names、test-file-names、test-function-names、format-constants、clause-enums
# 库 lib/code-source-discipline-naming-and-constants-cells.sh：feature-bits
# 库 lib/code-source-discipline-mutation-and-reading-cells.sh：mutation-tables、absolute-assertions、reading-discipline
# 库 lib/code-source-discipline-unsafe-and-registry-cells.sh：unsafe-allowances、gpu-exemptions、crash-point-replay-segments、crate-sources-reachable
# 库 lib/code-source-discipline-unsafe-and-registry-cells.sh：ordinal-names、data-names、verus-ghost-names、refused-tests
# 规则：.claude/rules/verification-reference.md「门禁的结构」「门禁管哪一半」。
#
# 样本：fixtures/code-source-discipline.sh/ 下每格至少一对 <格名>-red / <格名>-green，每份样本根的 .gate-cells 只点名那一格；
# 各格注释里说的 red / green 就是那一格自己的样本。
# 弄坏开关的读法在格函数里，格函数在 lib/ 下的库里；下面每个开关变量（STAGE_CELLS_BREAK 归 lib/stage-cells.sh）在哪份库的代码里被读，按库列在这里：
# 库 lib/code-source-discipline-naming-and-constants-cells.sh：GATE_VAGUE_NAMES_BREAK、CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK
# 库 lib/code-source-discipline-mutation-and-reading-cells.sh：GATE_READING_DISCIPLINE_BREAK、GATE_MUTATION_TABLES_BREAK、GATE_ABSOLUTE_ASSERTIONS_BREAK
# 库 lib/code-source-discipline-unsafe-and-registry-cells.sh：GATE_UNSAFE_ALLOWANCES_BREAK、GATE_ORDINAL_NAMES_BREAK、GATE_VERUS_GHOST_NAMES_BREAK
# 这一道的 shell 代码里不读、由它起的判法脚本读的：KB_NUMBERED_FILE_BREAK、REJECTION_BLOCKS_BREAK、LAG_BASELINE_BREAK、DATA_NAME_LINT_BREAK
# 弄坏开关。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（fixtures/code-source-discipline.sh/ 下的目录名，
# 括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   STAGE_CELLS_BREAK=red-swallowed  共用库吞红：红格按绿记、整道退 0
#       absolute-assertions-red、clause-enums-red、crash-point-replay-segments-red、data-names-red、data-names-folded-arm-red、feature-bits-red、format-constants-exclusion-red、format-constants-red、gpu-exemptions-red、
#       mutation-tables-duplicates-lag-red、mutation-tables-red、mutation-tables-undefined-identifier-red、ordinal-names-red、reading-discipline-red、test-file-names-red、test-function-names-red、crate-sources-reachable-red、
#       mutation-tables-ignore-comments-and-packages-red、mutation-tables-manifest-path-red、mutation-tables-reachability-red、mutation-tables-release-red、mutation-tables-unfiltered-red、
#       mutation-tables-test-target-modules-red、reading-discipline-split-family-red、
#       unsafe-allowances-no-safety-red、unsafe-allowances-other-crate-red、unsafe-allowances-reason-empty-red、unsafe-allowances-root-blank-red、
#       unsafe-allowances-safety-vague-red、unsafe-allowances-source-missing-red、unsafe-allowances-two-allows-red、unsafe-allowances-wrong-function-red、
#       unsafe-allowances-place-count-wrong-red、unsafe-allowances-one-place-no-safety-red、unsafe-allowances-one-place-word-missing-red、unsafe-allowances-allow-count-short-red、
#       unsafe-allowances-allows-on-one-function-red、unsafe-allowances-same-function-two-rows-red、unsafe-allowances-crate-rows-disagree-red、
#       unsafe-allowances-statement-finished-between-red、
#       unsafe-allowances-bin-root-unregistered-red、unsafe-allowances-bin-root-forbid-red、unsafe-allowances-bin-root-not-found-red、
#       unsafe-allowances-unsafe-impl-no-safety-red、unsafe-allowances-unsafe-fn-no-safety-red、
#       unsafe-allowances-line-separator-in-a-comment-red、unsafe-allowances-same-text-without-the-line-separator-red、unsafe-allowances-safety-line-inside-a-string-red、
#       unsafe-allowances-raw-c-string-hides-unsafe-red、unsafe-allowances-include-of-a-non-rs-file-red、unsafe-allowances-path-attribute-to-a-non-rs-file-red、
#       unsafe-allowances-no-mangle-without-the-keyword-red、unsafe-allowances-bin-helper-directory-unregistered-red、
#       unsafe-allowances-same-method-name-registered-without-type-arguments-red、
#       feature-bits-trilingual-red、feature-bits-two-originals-red、verus-ghost-names-red、refused-tests-red、refused-tests-stale-red、
#       vague-names-red（期望退出 1，实测 0）
#   STAGE_CELLS_BREAK=marker-ignored  共用库不认 .gate-cells：每份样本十七格都跑
#       absolute-assertions-green、clause-enums-green、crash-point-replay-segments-green、data-names-green、feature-bits-green、format-constants-green、gpu-exemptions-green、
#       mutation-tables-duplicates-lag-green、mutation-tables-green、mutation-tables-inline-module-literals-green、ordinal-names-green、reading-discipline-green、
#       mutation-tables-ignore-comments-and-packages-green、mutation-tables-manifest-path-green、mutation-tables-reachability-green、mutation-tables-release-green、mutation-tables-unfiltered-green、
#       mutation-tables-test-target-modules-green、reading-discipline-split-family-green、
#       clause-enums-trilingual-green、feature-bits-trilingual-green、format-constants-trilingual-green、
#       test-file-names-green、test-function-names-green、crate-sources-reachable-green、unsafe-allowances-deny-one-green、unsafe-allowances-green、unsafe-allowances-several-rows-green、unsafe-allowances-statement-continued-green、
#       unsafe-allowances-bin-root-green、unsafe-allowances-unsafe-impl-green、unsafe-allowances-same-method-two-types-green、
#       unsafe-allowances-no-mangle-registered-green、unsafe-allowances-bin-helper-directory-registered-green、unsafe-allowances-unsafe-below-the-first-line-of-a-method-chain-green、
#       unsafe-allowances-same-method-name-in-two-inherent-impls-green、unsafe-allowances-macro-expands-three-unsafe-for-one-keyword-green、
#       unsafe-allowances-one-comment-covers-the-second-unsafe-of-a-chain-green、verus-ghost-names-green、refused-tests-green、refused-tests-incomplete、vague-names-green（期望退出 0，实测 1）；absolute-assertions-red、
#       clause-enums-red、crash-point-replay-segments-red、data-names-red、data-names-folded-arm-red、feature-bits-red、format-constants-exclusion-red、format-constants-red、
#       mutation-tables-duplicates-lag-red、mutation-tables-red、mutation-tables-undefined-identifier-red、mutation-tables-ignore-comments-and-packages-red、mutation-tables-manifest-path-red、mutation-tables-reachability-red、mutation-tables-release-red、mutation-tables-unfiltered-red、
#       mutation-tables-test-target-modules-red、reading-discipline-split-family-red、
#       ordinal-names-red、reading-discipline-red、test-file-names-red、test-function-names-red、crate-sources-reachable-red、
#       unsafe-allowances-no-safety-red、unsafe-allowances-other-crate-red、unsafe-allowances-reason-empty-red、unsafe-allowances-root-blank-red、
#       unsafe-allowances-safety-vague-red、unsafe-allowances-source-missing-red、unsafe-allowances-two-allows-red、unsafe-allowances-wrong-function-red、
#       unsafe-allowances-place-count-wrong-red、unsafe-allowances-one-place-no-safety-red、unsafe-allowances-one-place-word-missing-red、unsafe-allowances-allow-count-short-red、
#       unsafe-allowances-allows-on-one-function-red、unsafe-allowances-same-function-two-rows-red、unsafe-allowances-crate-rows-disagree-red、
#       unsafe-allowances-statement-finished-between-red、
#       unsafe-allowances-bin-root-unregistered-red、unsafe-allowances-bin-root-forbid-red、unsafe-allowances-bin-root-not-found-red、
#       unsafe-allowances-unsafe-impl-no-safety-red、unsafe-allowances-unsafe-fn-no-safety-red、
#       unsafe-allowances-line-separator-in-a-comment-red、unsafe-allowances-same-text-without-the-line-separator-red、unsafe-allowances-safety-line-inside-a-string-red、
#       unsafe-allowances-raw-c-string-hides-unsafe-red、unsafe-allowances-include-of-a-non-rs-file-red、unsafe-allowances-path-attribute-to-a-non-rs-file-red、
#       unsafe-allowances-no-mangle-without-the-keyword-red、unsafe-allowances-bin-helper-directory-unregistered-red、
#       unsafe-allowances-same-method-name-registered-without-type-arguments-red、
#       feature-bits-trilingual-red、feature-bits-two-originals-red、verus-ghost-names-red、
#       vague-names-red（输出里找不到「样本只跑 .gate-cells 点名的格：…」）；
#       gpu-exemptions、refused-tests 两格的红样本 expect 不要那一句，十七格都跑照样退 1、点名的红照样报出来，判得对，不在名单里
#   GATE_VAGUE_NAMES_BREAK=trait-methods-judged  vague-names 把 trait 块里的方法也判
#       vague-names-green（期望退出 0，实测 1）
#   GATE_READING_DISCIPLINE_BREAK=reading-writes-unseen  reading-discipline 不认字段简写初始化与经方法调用结果写字段，把这两种写过的读数也当恒为 0
#       reading-discipline-green（期望退出 0，实测 1）
#   GATE_READING_DISCIPLINE_BREAK=asserted-fields-judged  reading-discipline 把进了断言被拿来核的字段（钉值）也当恒为 0 的读数
#       reading-discipline-green（期望退出 0，实测 1）
#   GATE_VAGUE_NAMES_BREAK=string-continuation-judged  vague-names 把以 \n\ 续行的字符串字面量里的源码也判
#       vague-names-green（期望退出 0，实测 1）
#   GATE_VAGUE_NAMES_BREAK=exclusions-ignored  vague-names 不认排除表
#       vague-names-red（输出里找不到「crates/demo/src/renamed.rs 里已经…」）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=format-constants  不比源码 const 的值与 kb 的现行值
#       format-constants-red（输出里找不到「const SAMPLE_HDR = 84」）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=format-constants-exclusion-ignored  排除表照读照判，但登记的路径照样扫
#       format-constants-green、format-constants-trilingual-green（期望退出 0，实测 1）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=format-constants-ignored-files-scanned  按盘上有什么扫，不按 git 列出的扫：被 .gitignore 忽略的编译副本也判
#       format-constants-exclusion-red（输出里找不到「.claude/format-constants-exclude:2 scripts/experiment-apparatus/target/copy/ 下一份被 git 列出的 .rs 都没排到」）；
#       format-constants-green、format-constants-trilingual-green（期望退出 0，实测 1）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=format-constants-research-unscanned  research/ 整片不扫（只扫 scripts/experiment-apparatus/ 与 crates/）：冻结证据里的旧值看不见，排除表登记的冻结证据排不到文件
#       format-constants-exclusion-red（输出里找不到「排除表 .claude/format-constants-exclude 有 3 行不起作用或没写理由」）；
#       format-constants-green、format-constants-trilingual-green（期望退出 0，实测 1）；format-constants-red（输出里找不到「research/prompts/frozen-round/src/main.rs:1  const SAMPLE_HDR = 85」）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=format-constants-exclusion-missing-path-unjudged  排除表里指向不存在的路径不判
#       format-constants-exclusion-red（输出里找不到「.claude/format-constants-exclude:1 指向不存在的路径 research/prompts/gone/」）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=format-constants-exclusion-no-match-unjudged  排除表里一份文件都没排到的路径不判
#       format-constants-exclusion-red（输出里找不到「.claude/format-constants-exclude:2 scripts/experiment-apparatus/target/copy/ 下一份被 git 列出的 .rs 都没排到」）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=format-constants-exclusion-reason-unjudged  排除表里没写理由的行不判
#       format-constants-exclusion-red（输出里找不到「.claude/format-constants-exclude:3 research/prompts/frozen.rs 没写为什么排除」）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=clause-enums  表里的名字按子串认作正文列举的一项
#       clause-enums-red（输出里找不到「表里的「乙」不是 D99 已定项 1 正文那串列举里的一项」）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=feature-bits-decision-any-language  D15 不分语言，取 15-*.md 按文件名排序后最靠前的一份（三语并存时是 .en.md，里面没有「#### 已定项 4」）
#       feature-bits-trilingual-red（输出里找不到「记账表 .claude/kb/feature-bits.md 与位号的唯一登记位（.claude/kb/decisions/15-格式冻结政策.md 已定项 4）对不上」）；
#       feature-bits-trilingual-green（期望退出 0，实测 1）；feature-bits-two-originals-red（输出里找不到「D15（格式冻结政策） 的中文原文决策文件不止一份：」）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=clause-enums-decision-any-language  分项正文不分语言，取 <号>-*.md 按文件名排序后最靠前的一份（三语并存时是 .en.md，里面没有「#### 已定项 1」）
#       clause-enums-trilingual-green（期望退出 0，实测 1）
#   CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK=format-constants-translations-read  译文（.en.md、.ja.md）也当登记位读：同一个标记在三份里各登记一处，译文历史节里与没跟上的正文里的旧值也算残留
#       format-constants-trilingual-green（期望退出 0，实测 1）
#   KB_NUMBERED_FILE_BREAK=first-match  归 lib-item-ref-status.py（设进环境，被 clause-enums、feature-bits 起的 python 继承）：numbered_files 不分语言、交回编号下的全部候选（含译文），两格报「中文原文…不止一份」
#       clause-enums-trilingual-green、feature-bits-trilingual-green（期望退出 0，实测 1）；
#       feature-bits-trilingual-red（输出里找不到「记账表 .claude/kb/feature-bits.md 与位号的唯一登记位（.claude/kb/decisions/15-格式冻结政策.md 已定项 4）对不上」）
#   REJECTION_BLOCKS_BREAK=summaries-only  reading-discipline 逐块打印拒绝只打摘要（共用 lib/rejection-blocks.py）
#       reading-discipline-red（输出里找不到「crates/singlefs-verification/s…」）；
#       reading-discipline-split-family-red（输出里找不到「scripts/experiment-apparatus/e8_seed_and_readings/readings.rs：归基准 HEAD 里的入口文件…」）
#   LAG_BASELINE_BREAK=unreadable-as-absent  归 lib-lag-baseline.py（设进环境，被 mutation-tables、reading-discipline 起的 python 继承）：git 取不到基准那一版的滞后表、豁免表当成「基准里没有这张表」，只缩不涨静默不比
#       reading-discipline-red（输出里找不到「读不出基准 HEAD 里的 .claude/gate.d/experiment-seed-fold-lag.tsv（git show 退 128）」）
#   GATE_UNSAFE_ALLOWANCES_BREAK=count-ignored  unsafe-allowances ② 的总数不数，③ 里根是 deny 时逐行的两条（函数上一处放开、函数体里的处数）一并不判
#       unsafe-allowances-other-crate-red（输出里找不到「singlefs-verification 放开 unsafe_cod…」）；unsafe-allowances-two-allows-red（期望退出 1，实测 0）；
#       unsafe-allowances-place-count-wrong-red、unsafe-allowances-allow-count-short-red、unsafe-allowances-allows-on-one-function-red、
#       unsafe-allowances-bin-root-unregistered-red、unsafe-allowances-bin-helper-directory-unregistered-red（期望退出 1，实测 0）；
#       unsafe-allowances-raw-c-string-hides-unsafe-red、unsafe-allowances-include-of-a-non-rs-file-red、unsafe-allowances-path-attribute-to-a-non-rs-file-red、
#       unsafe-allowances-no-mangle-without-the-keyword-red（输出里找不到「singlefs-verification/src/bin/e900_thread_clock.rs 的 unsafe 关键字 1 处…」）
#   GATE_UNSAFE_ALLOWANCES_BREAK=placement-ignored  unsafe-allowances ③ 的位置不判，逐行的两条一并不判
#       unsafe-allowances-other-crate-red（输出里找不到「singlefs-verification 在登记表里没有放开」）；unsafe-allowances-wrong-function-red（期望退出 1，实测 0）；
#       unsafe-allowances-place-count-wrong-red、unsafe-allowances-allows-on-one-function-red、
#       unsafe-allowances-same-method-name-registered-without-type-arguments-red（期望退出 1，实测 0）；
#       unsafe-allowances-allow-count-short-red（输出里找不到「.claude/gate.d/unsafe-allowances.tsv:3 登记的 crates/singlefs-core/s…」）
#   GATE_UNSAFE_ALLOWANCES_BREAK=safety-comment-ignored  unsafe-allowances ④ 不要 // SAFETY: 注释
#       unsafe-allowances-no-safety-red、unsafe-allowances-one-place-no-safety-red、unsafe-allowances-statement-finished-between-red、
#       unsafe-allowances-unsafe-impl-no-safety-red、unsafe-allowances-unsafe-fn-no-safety-red、
#       unsafe-allowances-line-separator-in-a-comment-red、unsafe-allowances-same-text-without-the-line-separator-red、
#       unsafe-allowances-safety-line-inside-a-string-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=root-level-ignored  unsafe-allowances ① 不看根写了 forbid 还是 deny
#       unsafe-allowances-root-blank-red、unsafe-allowances-bin-root-forbid-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=other-crates-ignored  unsafe-allowances 登记表里一个根都没有放开的 crate 整个不判
#       unsafe-allowances-deny-one-green、unsafe-allowances-green、unsafe-allowances-several-rows-green（输出里找不到「crates/ 里放开 unsafe 的地方与登记表对上：扫…」）；
#       unsafe-allowances-other-crate-red、unsafe-allowances-bin-root-unregistered-red、unsafe-allowances-bin-root-forbid-red、
#       unsafe-allowances-raw-c-string-hides-unsafe-red、unsafe-allowances-include-of-a-non-rs-file-red、unsafe-allowances-path-attribute-to-a-non-rs-file-red、
#       unsafe-allowances-no-mangle-without-the-keyword-red、unsafe-allowances-bin-helper-directory-unregistered-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=reason-ignored  unsafe-allowances 不判登记行的理由栏
#       unsafe-allowances-reason-empty-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=source-ignored  unsafe-allowances 不判登记行的出处栏
#       unsafe-allowances-source-missing-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=safety-premise-ignored  unsafe-allowances ④ 不判 SAFETY 注释提没提到登记的词
#       unsafe-allowances-safety-vague-red、unsafe-allowances-one-place-word-missing-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=allow-count-single  unsafe-allowances ② 放开的总数照一个根一处数（根是 deny 时应当 1 处），不看这个根登了几行
#       unsafe-allowances-several-rows-green、unsafe-allowances-unsafe-impl-green、unsafe-allowances-same-method-two-types-green（期望退出 0，实测 1）；
#       unsafe-allowances-allow-count-short-red（输出里找不到「singlefs-core 放开 unsafe_code 的地方 1 处，根是 deny、应当 2 处…」）
#   GATE_UNSAFE_ALLOWANCES_BREAK=per-row-allows-ignored  unsafe-allowances ③ 不判登记的每个函数上恰好一处放开
#       unsafe-allowances-allows-on-one-function-red（期望退出 1，实测 0）；
#       unsafe-allowances-allow-count-short-red（输出里找不到「.claude/gate.d/unsafe-allowances.tsv:3 登记的 crates/singlefs-core/s…」）
#   GATE_UNSAFE_ALLOWANCES_BREAK=per-function-count-ignored  unsafe-allowances ③ 不判登记的每个函数体里 unsafe 的处数与第 5 段相同
#       unsafe-allowances-place-count-wrong-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=safety-one-place-per-function  unsafe-allowances ④ 一个登记的函数里只判一处 unsafe 有没有 // SAFETY: 注释，同一个函数里另几处不判
#       unsafe-allowances-one-place-no-safety-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=words-one-place-per-function  unsafe-allowances ④ 一个登记的函数里只判一处 SAFETY 注释提没提到登记的词，同一个函数里另几处不判
#       unsafe-allowances-one-place-word-missing-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=registered-words-ignored  unsafe-allowances ④ 每一处都照文件描述符、偏移、长度三个词判，不读登记表第 6 段
#       unsafe-allowances-several-rows-green、unsafe-allowances-bin-root-green、unsafe-allowances-unsafe-impl-green、
#       unsafe-allowances-same-method-two-types-green、unsafe-allowances-no-mangle-registered-green、unsafe-allowances-bin-helper-directory-registered-green、
#       unsafe-allowances-unsafe-below-the-first-line-of-a-method-chain-green、unsafe-allowances-same-method-name-in-two-inherent-impls-green、
#       unsafe-allowances-macro-expands-three-unsafe-for-one-keyword-green、unsafe-allowances-one-comment-covers-the-second-unsafe-of-a-chain-green（期望退出 0，实测 1）；
#       unsafe-allowances-one-place-word-missing-red（输出里找不到「crates/singlefs-core/src/block_device.rs:9 的 // SAFETY: 注释没写出…」）
#   GATE_UNSAFE_ALLOWANCES_BREAK=duplicate-rows-allowed  unsafe-allowances 登记表同一个（crate，函数）登两行不判，表里靠下的那一行顶掉另一行
#       unsafe-allowances-same-function-two-rows-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=crate-levels-disagreement-allowed  unsafe-allowances 登记表同一个 crate 几行第 2 段不一样不判，级别取几行的并集
#       unsafe-allowances-crate-rows-disagree-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=continuation-lines-ignored  unsafe-allowances ④ 不跳同一条语句的续行，只认 unsafe 那一行上方连着的注释
#       unsafe-allowances-statement-continued-green、unsafe-allowances-unsafe-below-the-first-line-of-a-method-chain-green、
#       unsafe-allowances-one-comment-covers-the-second-unsafe-of-a-chain-green（期望退出 0，实测 1）
#   GATE_UNSAFE_ALLOWANCES_BREAK=bin-roots-in-library  unsafe-allowances 不认 bin 根（bin_roots_of 一个都不交），src/bin/ 下的文件全算进库根那一份
#       unsafe-allowances-bin-root-green、unsafe-allowances-unsafe-impl-green、unsafe-allowances-no-mangle-registered-green、
#       unsafe-allowances-bin-helper-directory-registered-green、unsafe-allowances-unsafe-below-the-first-line-of-a-method-chain-green、
#       unsafe-allowances-same-method-name-in-two-inherent-impls-green、unsafe-allowances-macro-expands-three-unsafe-for-one-keyword-green、
#       unsafe-allowances-one-comment-covers-the-second-unsafe-of-a-chain-green（期望退出 0，实测 1）；
#       unsafe-allowances-bin-root-unregistered-red、unsafe-allowances-raw-c-string-hides-unsafe-red、unsafe-allowances-include-of-a-non-rs-file-red、
#       unsafe-allowances-path-attribute-to-a-non-rs-file-red、unsafe-allowances-no-mangle-without-the-keyword-red、
#       unsafe-allowances-bin-helper-directory-unregistered-red（输出里找不到「singlefs-verification/src/bin/e900_thread_clock.rs 的 un…」）；
#       unsafe-allowances-bin-root-forbid-red（输出里找不到「crates/singlefs-verification/src/bin/e900_thread_clock.rs 的级…」）；
#       unsafe-allowances-same-method-name-registered-without-type-arguments-red（输出里找不到「crates/singlefs-verification/src/bin/e900_thread_clock.rs:18 放开…」）
#   GATE_UNSAFE_ALLOWANCES_BREAK=unregistered-bin-roots-ignored  unsafe-allowances 登记表里没有的 bin 根整个不判（不照 forbid 数）
#       unsafe-allowances-bin-root-green（输出里找不到「crates/ 里放开 unsafe 的地方与登记表对上：扫…」）；unsafe-allowances-bin-root-unregistered-red、
#       unsafe-allowances-raw-c-string-hides-unsafe-red、unsafe-allowances-include-of-a-non-rs-file-red、unsafe-allowances-path-attribute-to-a-non-rs-file-red、
#       unsafe-allowances-no-mangle-without-the-keyword-red、unsafe-allowances-bin-helper-directory-unregistered-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=registered-bin-root-unchecked  unsafe-allowances 登记表第 1 段点名的 bin 根在 crate 里不是 bin 根，不判、那几行不管
#       unsafe-allowances-bin-root-not-found-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=signature-outside-body  unsafe-allowances ③ 登记的项从 { 起算，unsafe fn 签名与 unsafe impl 项头里的 unsafe 落在外面
#       unsafe-allowances-unsafe-impl-green、unsafe-allowances-same-method-two-types-green、unsafe-allowances-no-mangle-registered-green（期望退出 0，实测 1）
#   GATE_UNSAFE_ALLOWANCES_BREAK=methods-keyed-by-name  unsafe-allowances impl 块里的方法只按方法名认，登记表写的 <类型>::<方法> 对不上
#       unsafe-allowances-unsafe-impl-green、unsafe-allowances-same-method-two-types-green、
#       unsafe-allowances-same-method-name-in-two-inherent-impls-green（期望退出 0，实测 1）；
#       unsafe-allowances-same-method-name-registered-without-type-arguments-red（输出里找不到「crates/singlefs-verification/src/bin/e900_thread_clock.rs:18 放开…」）
#   GATE_UNSAFE_ALLOWANCES_BREAK=outermost-holder  unsafe-allowances ③ 几行都罩住同一处 unsafe 时取表里靠前的那一行，方法里的 unsafe 记到它所在的 impl 块那一行
#       unsafe-allowances-unsafe-impl-green、unsafe-allowances-same-method-two-types-green（期望退出 0，实测 1）
#   GATE_UNSAFE_ALLOWANCES_BREAK=attributes-not-skipped  unsafe-allowances ④ 往上找 SAFETY 注释碰到属性就停，写在 #[allow(unsafe_code, …)] 上方的不认
#       unsafe-allowances-unsafe-impl-green、unsafe-allowances-no-mangle-registered-green（期望退出 0，实测 1）
#   GATE_UNSAFE_ALLOWANCES_BREAK=lines-split-at-unicode-separators  unsafe-allowances ④ 找 SAFETY 注释的那一份按 splitlines() 切行，U+2028 这类也算换行，与 line_of 的行号错开
#       unsafe-allowances-line-separator-in-a-comment-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=safety-read-from-raw-lines  unsafe-allowances ④ 找 SAFETY 注释读原文的行，字符串字面量里顶格的 // SAFETY: 也当注释
#       unsafe-allowances-safety-line-inside-a-string-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=c-string-prefix-unknown  unsafe-allowances 抹字符串不认 cr 前缀（只认 br），cr"\" 失步、吞掉后面的 unsafe
#       unsafe-allowances-raw-c-string-hides-unsafe-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=included-files-unscanned  unsafe-allowances 不追 include! 与 #[path] 点到的文件（只扫 .rs）
#       unsafe-allowances-include-of-a-non-rs-file-red、unsafe-allowances-path-attribute-to-a-non-rs-file-red（期望退出 1，实测 0）
#   GATE_UNSAFE_ALLOWANCES_BREAK=exported-symbol-attributes-unseen  unsafe-allowances ② 不数不带 unsafe 的 #[no_mangle]、#[export_name]、#[link_section]
#       unsafe-allowances-no-mangle-without-the-keyword-red（期望退出 1，实测 0）；unsafe-allowances-no-mangle-registered-green（期望退出 0，实测 1）
#   GATE_UNSAFE_ALLOWANCES_BREAK=bin-helper-directory-in-library  unsafe-allowances src/bin/<名>.rs 只管它自己，旁边 src/bin/<名>/ 下的文件归了库根
#       unsafe-allowances-bin-helper-directory-registered-green（期望退出 0，实测 1）；
#       unsafe-allowances-bin-helper-directory-unregistered-red（输出里找不到「singlefs-verification/src/bin/e900_thread_clock.rs 的 unsafe 关键字 1 处…」）
#   GATE_UNSAFE_ALLOWANCES_BREAK=chain-lines-not-continued  unsafe-allowances ④ 找语句开头时不认以 . 起头的方法链续行，unsafe 在链的第二行就找不到 let 上方的注释
#       unsafe-allowances-unsafe-below-the-first-line-of-a-method-chain-green（期望退出 0，实测 1）
#   GATE_UNSAFE_ALLOWANCES_BREAK=colliding-methods-keyed-alike  unsafe-allowances 同一份文件里撞了的 <类型>::<方法> 不改用全写，登记行只认得到最靠前的那个
#       unsafe-allowances-same-method-name-in-two-inherent-impls-green（期望退出 0，实测 1）；
#       unsafe-allowances-same-method-name-registered-without-type-arguments-red（输出里找不到「crates/singlefs-verification/src/bin/e900_thread_clock.rs:18 放开…」）
#   test-file-names、test-function-names、crash-point-replay-segments、gpu-exemptions、refused-tests 几格的判法开关（CRASH_CASE_CHECK_BREAK、CRASH_POINT_REPLAY_ITEMS_BREAK、
#   GPU_EXEMPTIONS_CHECK_BREAK、REFUSED_TEST_REGISTRY_BREAK）不在这里列：判法脚本平常跑时设了开关整份退 2（只在 --selftest 里认），在样本上证不了；
#   开关表在 scripts/crash/crash-case-check.py、scripts/crash/lib_crash_point_replay_items.py、scripts/crash/gpu-exemptions-check.py、scripts/gate/refused-test-registry.py 文件头，
#   由判法脚本的 --selftest 证红（lib_crash_point_replay_items.py 那几条归 crash-case-check.py --selftest）
#   GATE_ORDINAL_NAMES_BREAK=compound-names-unseen  ordinal-names 只认整名等于词表里的词，不按词边界认复合名
#       ordinal-names-green（期望退出 0，实测 1）；ordinal-names-red（输出里找不到「crates/demo/src/main.rs:1 名字 enumerate_」）
#   GATE_ORDINAL_NAMES_BREAK=shell-unscanned  ordinal-names 不扫 .sh
#       ordinal-names-red（输出里找不到「scripts/demo/run.sh:2 变量 stream」）
#   GATE_ORDINAL_NAMES_BREAK=comments-and-strings-judged  ordinal-names 不剥注释与字符串，里面的词也当名字判
#       ordinal-names-green（期望退出 0，实测 1）；ordinal-names-red（输出里找不到「2 处名字带序号或位置词」）
#   GATE_ORDINAL_NAMES_BREAK=exclusions-ignored  ordinal-names 不认排除表
#       ordinal-names-green（期望退出 0，实测 1）
#   data-names 格的判法开关在 scripts/gate/data-name-lint.py（文件头列全，--selftest 证红）；这一道里打开时判错的样本：
#   DATA_NAME_LINT_BREAK=string-literals-skipped  data-names 三种字面量形态一个都不扫
#       data-names-green（期望退出 0，实测 1）；data-names-red（输出里找不到「scripts/experiment-apparatus/e1_demo.rs:1  元组首列」）；
#       data-names-folded-arm-red（期望退出 1，实测 0）
#   DATA_NAME_LINT_BREAK=kb-tables-skipped  data-names 不扫 kb 表格
#       data-names-red（输出里找不到「.claude/kb/demo.md:3  kb 表格单元格」）；data-names-green（输出里找不到「数据形态的名字合命名纪律：扫了 4 份文件、9 个串…」）
#   DATA_NAME_LINT_BREAK=single-letters-passed  data-names 单字母放行
#       data-names-red（输出里找不到「「x」是单字母」）；data-names-folded-arm-red（期望退出 1，实测 0）
#   DATA_NAME_LINT_BREAK=folded-arm-skipped  data-names 不认 cargo fmt 折成 => { "名字" } 的 match 分支，只认与 => 紧挨着的串
#       data-names-folded-arm-red（期望退出 1，实测 0）
#   DATA_NAME_LINT_BREAK=registered-ignored  data-names 不认样本自己的 .claude/abbreviations 里登记的缩写
#       data-names-green（期望退出 0，实测 1）
#   DATA_NAME_LINT_BREAK=numbered-ignored  data-names 不认 e<数字> 这类整类登记
#       data-names-green（期望退出 0，实测 1）
#   DATA_NAME_LINT_BREAK=exclusions-ignored  data-names 不认排除表
#       data-names-green（期望退出 0，实测 1）；data-names-red（输出里找不到「…:renamed_long 已经一个违规都没有了」）
#   DATA_NAME_LINT_BREAK=stale-exclusions-passed  data-names 排除表里已经没有违规的行、指向不存在路径的行不判
#       data-names-red（输出里找不到「已经一个违规都没有了」）
#   DATA_NAME_LINT_BREAK=comments-scanned  data-names 注释里的串也扫
#       data-names-green（期望退出 0，实测 1）；data-names-red（输出里找不到「6 处违规、排除表 2 处不合规（扫了 4 份文件、10 个串）」）
#   DATA_NAME_LINT_BREAK=call-arguments-judged  data-names 函数调用的字符串实参（左括号前是标识符）也当元组首列
#       data-names-green（期望退出 0，实测 1）；data-names-red（输出里找不到「6 处违规、排除表 2 处不合规（扫了 4 份文件、10 个串）」）
#   GATE_MUTATION_TABLES_BREAK=named-test-unsearched  mutation-tables 不判点名的测试在不在选的目标里
#       mutation-tables-red（输出里找不到「M_点名的测试不存在」那一条）；mutation-tables-ignore-comments-and-packages-red（输出里找不到「M_点名的用例两个包里都没有」那一条）；
#       mutation-tables-manifest-path-red（输出里找不到「M_独立工程里找不到点名的测试：点名的测试 absent_in_standalone_case 在 crates/standalone 的 --lib（」那一条）
#   GATE_MUTATION_TABLES_BREAK=named-test-target-ignored  mutation-tables 点名的测试在整个包里找、不看选的目标
#       mutation-tables-red（输出里找不到「M_库里点名只在集成测试里的用例」那一条）；
#       mutation-tables-manifest-path-red（输出里找不到「点名的测试 absent_in_standalone_case 在 crates/standalone 的 --lib（」那一条：说明句里的目标说明变成「src/ 与 tests/」）
#   GATE_MUTATION_TABLES_BREAK=directory-target-modules-unread  --test 点名目录式测试目标（tests/<名>/main.rs）时，mod 声明的共用模块只从 tests/<名>.rs 取，目录里 main.rs 与子模块的 mod 声明不跟
#       mutation-tables-green（期望退出 0，实测 1：输出里出现「点名的测试 shared_split_case 在 crates/demo 的 --test split_target（」那一条，它写在 #[path] 引来的 tests/common_split/mod.rs 里）；
#       mutation-tables-test-target-modules-green（期望退出 0，实测 1：输出里出现「点名的测试 shared_case 在 crates/demo 的 --test split_target（」那一条，它写在 #[path] 引来的 tests/common_split_shared/mod.rs 里）
#   GATE_MUTATION_TABLES_BREAK=named-test-definitions-only  mutation-tables 点名的测试只认 fn <名> 的定义（宏展开出来的认不出）
#       mutation-tables-green、mutation-tables-inline-module-literals-green、mutation-tables-test-target-modules-green、mutation-tables-ignore-comments-and-packages-green、
#       mutation-tables-manifest-path-green、mutation-tables-release-green、mutation-tables-unfiltered-green（期望退出 0，实测 1）
#   GATE_MUTATION_TABLES_BREAK=comment-lines-as-attributes  mutation-tables 判 #[ignore] 时把 fn 上面的注释行与行尾注释也当属性
#       mutation-tables-ignore-comments-and-packages-green（期望退出 0，实测 1）
#   GATE_MUTATION_TABLES_BREAK=one-package-per-row  mutation-tables 一行只认整串 cargo 参数里最靠前的那一个 -p
#       mutation-tables-ignore-comments-and-packages-green（期望退出 0，实测 1）；mutation-tables-ignore-comments-and-packages-red（输出里找不到「、crates/retry 的」）
#   GATE_MUTATION_TABLES_BREAK=package-directory-from-name  mutation-tables 包目录不读仓根 Cargo.toml 的 [workspace] members、按 crates/<包名> 硬拼（-p 点到非成员的也照拼）
#       mutation-tables-manifest-path-green（期望退出 0，实测 1：第 9 行 -p renamed-package 的包在 crates/renamed-dir，按 crates/renamed-package 找不到）；
#       mutation-tables-manifest-path-red（输出里找不到「-p standalone-project 不是工作区成员」）
#   GATE_MUTATION_TABLES_BREAK=non-member-package-unjudged  mutation-tables 成员照读，-p 点到非成员的不判红、退回按 crates/<包名> 硬拼
#       mutation-tables-manifest-path-red（输出里找不到「-p standalone-project 不是工作区成员」）
#   GATE_MUTATION_TABLES_BREAK=manifest-path-outside-repo-unjudged  mutation-tables 不判 --manifest-path 解到仓根之外
#       mutation-tables-manifest-path-red（输出里找不到「--manifest-path ../outside-project/Cargo.toml 解到仓根之外」）
#   GATE_MUTATION_TABLES_BREAK=manifest-path-file-name-unjudged  mutation-tables 不判 --manifest-path 指到的文件名是不是 Cargo.toml
#       mutation-tables-manifest-path-red（输出里找不到「--manifest-path crates/standalone/src/lib.rs 的文件名不是 Cargo.toml」）
#   GATE_MUTATION_TABLES_BREAK=manifest-path-missing-file-unjudged  mutation-tables 不判 --manifest-path 指到的文件在不在
#       mutation-tables-manifest-path-red（输出里找不到「--manifest-path crates/standalone-gone/Cargo.toml 指的文件不在」）
#   GATE_MUTATION_TABLES_BREAK=package-and-manifest-path-together-allowed  mutation-tables 不判同一行既带 -p 又带 --manifest-path
#       mutation-tables-manifest-path-red（输出里找不到「同一行既写了 -p demo 又写了 --manifest-path crates/standalone/Cargo.toml」）
#   GATE_MUTATION_TABLES_BREAK=manifest-path-twice-allowed  mutation-tables 不判同一行带两次 --manifest-path
#       mutation-tables-manifest-path-red（输出里找不到「--manifest-path 写了不止一次（crates/standalone/Cargo.toml、crates/renamed-dir/Cargo.toml）」）
#   GATE_MUTATION_TABLES_BREAK=manifest-path-without-value-unjudged  mutation-tables 不判 --manifest-path 后面没写路径
#       mutation-tables-manifest-path-red（输出里找不到「--manifest-path 后面没写路径」）
#   GATE_MUTATION_TABLES_BREAK=manifest-path-not-a-package  mutation-tables 判完 --manifest-path 的路径之后不把那份 Cargo.toml 所在的目录当包目录：点名的测试不在那里找
#       mutation-tables-manifest-path-green（输出里找不到「--manifest-path 指到的都是仓内的 Cargo.toml（2 行）」）；
#       mutation-tables-manifest-path-red（输出里找不到「M_独立工程里找不到点名的测试：点名的测试 absent_in_standalone_case 在 crates/standalone 的 --lib（」）
#   GATE_MUTATION_TABLES_BREAK=inline-module-literals-counted  mutation-tables 配内联 mod 的花括号时不抹注释、字符串、原始串与字符字面量
#       mutation-tables-inline-module-literals-green、mutation-tables-test-target-modules-green（期望退出 0，实测 1）
#   GATE_MUTATION_TABLES_BREAK=undefined-identifiers-ignored  mutation-tables 不读 static-check 的 UNDEFINED 行（替换文引了 crates 里找不到定义的标识符也不判红）
#       mutation-tables-undefined-identifier-red（期望退出 1，实测 0）
#   GATE_MUTATION_TABLES_BREAK=duplicate-key-with-test  mutation-tables 把点名的测试也算进 crates/mutations.tsv 重复行的键
#       mutation-tables-red（输出里找不到「crates/mutations.tsv 第 13、14 行」）
#   GATE_MUTATION_TABLES_BREAK=research-duplicates-unchecked  mutation-tables 不判研究变异表里除变异名外各列全同的行
#       mutation-tables-red（输出里找不到「scripts/mutation/tables/e1_sample.tsv 第 3、4 行」）；mutation-tables-duplicates-lag-green（期望退出 0，实测 1）；
#       mutation-tables-duplicates-lag-red（输出里找不到「重复行滞后表 .claude/gate.d/mutation-duplicates-lag.tsv 有 4 处不对」）
#   GATE_MUTATION_TABLES_BREAK=duplicates-lag-ignored  mutation-tables 不读重复行滞后表 .claude/gate.d/mutation-duplicates-lag.tsv：登记的组照红
#       mutation-tables-duplicates-lag-green（期望退出 0，实测 1）；mutation-tables-duplicates-lag-red（输出里找不到「重复行滞后表 … 有 4 处不对」）
#   GATE_MUTATION_TABLES_BREAK=duplicates-lag-growth-unchecked  mutation-tables 不拿滞后表和基准那一版比：表涨了不红
#       mutation-tables-duplicates-lag-red（输出里找不到「相对基准那一版多出一组」）；mutation-tables-duplicates-lag-green（输出里找不到「1 张研究变异表里没有两行…重复行滞后 2 组…」）
#   GATE_MUTATION_TABLES_BREAK=reachability-lines-unread  mutation-tables 不读 static-check 的可达性行（UNREACHABLE、REACHABILITY_UNJUDGED 不进红句，REACHABILITY_CHECKED、REACHABILITY_NOT_JUDGED 不进成功句）
#       mutation-tables-reachability-red（期望退出 1，实测 0）；mutation-tables-reachability-green（输出里找不到「点名的测试都够得着被改坏的函数（判了 1 行」）
#   GATE_MUTATION_TABLES_BREAK=release-lines-unread  mutation-tables 不读 static-check 的 RELEASE 行（命中重测试目标而第五段没写 --release 的行不判红，成功句里没有 release 那半句）
#       mutation-tables-release-red（期望退出 1，实测 0）；mutation-tables-release-green（输出里找不到「命中重测试目标的行第五段都写了 --release（判了 7 行」）
#   GATE_VERUS_GHOST_NAMES_BREAK=definitions-unseen  verus-ghost-names 不判名叫 Ghost / Tracked 的类型定义
#       verus-ghost-names-red（输出里找不到「crates/self-defined-wrappers/src/lib.rs:3 定义了名叫 Ghost 的 struct」）
#   GATE_VERUS_GHOST_NAMES_BREAK=renames-unseen  verus-ghost-names 不判 use … as Ghost / Tracked
#       verus-ghost-names-red（输出里找不到「crates/renamed-wrappers/src/lib.rs:1 把 crate::holder::Holder 改名成 Tracked」）
#   GATE_VERUS_GHOST_NAMES_BREAK=comments-judged  verus-ghost-names 不去注释、字符串与字符字面量
#       verus-ghost-names-green（期望退出 0，实测 1）
#   GATE_VERUS_GHOST_NAMES_BREAK=vstd-renames-judged  verus-ghost-names 连 use vstd::prelude::Ghost as Ghost 也判红
#       verus-ghost-names-green（期望退出 0，实测 1）
#   GATE_MUTATION_TABLES_BREAK=apparatus-directories-unscanned  mutation-tables 只扫 scripts/experiment-apparatus 顶层的 <名>.rs，带根 main.rs 的装置目录不进单元（缺表与锚点不对看不见）
#       mutation-tables-red（输出里找不到「scripts/mutation/tables/e13_directory_without_table.tsv 不存在（被测的是 scripts/experiment-apparatus/e13_directory_without_table/main.rs）」）；
#       mutation-tables-green（输出里找不到「✓ 6 个实验二进制都有成形的变异表，8 条变异的原文各命中源码一次」）
#   GATE_ABSOLUTE_ASSERTIONS_BREAK=apparatus-directories-unscanned  absolute-assertions 只认顶层的 .rs 文件：装置目录与 .rs 旁边的子模块目录不认
#       absolute-assertions-green（期望退出 0，实测 1）；
#       absolute-assertions-red（输出里找不到「scripts/experiment-apparatus/e12_directory_without_assertion/ 一条绝对值断言都没有」）
#   GATE_READING_DISCIPLINE_BREAK=apparatus-directories-unscanned  reading-discipline 退回只扫 scripts/experiment-apparatus/*.rs 与验证包 src/bin 下的 e<号>*.rs（只扫一层）
#       reading-discipline-green（输出里找不到「扫了 10 个文件（读不动的 0 个）」）；
#       reading-discipline-red（输出里找不到「C59（种子折叠成同一个状态）：4 处违规没有登记在豁免表里」）；
#       reading-discipline-split-family-green（期望退出 0，实测 1：装置只有 e8_seed_and_readings/ 一个目录，退回只扫顶层 .rs 之后一个文件都没扫到）
#   GATE_READING_DISCIPLINE_BREAK=restricted-visibility-unread  reading-discipline 的字段声明只认 pub、不认 pub(super)、pub(crate) 这类带括号的可见性
#       reading-discipline-green（输出里找不到「C60 查了 6 个整型结构体字段」）；
#       reading-discipline-red（输出里找不到「C60（恒定读数没有故障注入自证）：2 处违规没有登记在豁免表里」）
#   GATE_READING_DISCIPLINE_BREAK=lag-split-family-unrecognized  reading-discipline 的只缩不涨按路径自己比、不认拆分族：装置拆成入口加子模块文件、登记行跟着挪过去的，子模块文件都成了族外
#       reading-discipline-split-family-green（期望退出 0，实测 1）；
#       reading-discipline-split-family-red（输出里找不到「scripts/experiment-apparatus/e8_seed_and_readings/readings.rs：归基准 HEAD 里的入口文件…」）
#   GATE_MUTATION_TABLES_BREAK=test-target-module-tree-unwalked  mutation-tables 算点名测试的完整名时不顺目标根的 mod 声明走模块树、只看文件在包目录里的位置猜前缀（tests/ 与 src/bin/ 下子模块文件里的用例全名少了前缀），也不按点名的目标收窄同名用例
#       mutation-tables-test-target-modules-green（期望退出 0，实测 1：按真实全名写 --exact 的第 8 至 10 行筛不到）；
#       mutation-tables-test-target-modules-red（输出里找不到「crates/mutations.tsv:13 M_测试目标子模块里的用例写了裸名：筛选词 deeper_case（--exact） 筛不到点名的 nested_cases::deeper::deeper_case」）
#   DATA_NAME_LINT_BREAK=apparatus-subdirectories-unscanned  data-names 只扫 scripts/experiment-apparatus 顶层的 *.rs，装置目录里的 .rs 不扫（排除表指到子目录文件的行判「不在扫描范围」）
#       data-names-green（期望退出 0，实测 1）；
#       data-names-red（输出里找不到「scripts/experiment-apparatus/e2_split/parts/tests.rs:1  元组首列「slot_mac」里 「mac」是缩写」）
# mutation-tables 别的几样与 reading-discipline 别的几样（只有共用库那两项）判别力靠各自的样本。
#
#   bash .claude/gate.d/code-source-discipline.sh [项目根]                                十七格都跑
#   bash .claude/gate.d/code-source-discipline.sh --list                                  逐行打格名与判什么，不跑格
#   bash .claude/gate.d/code-source-discipline.sh --check <格名>[,<格名>…] [项目根]         只跑点名的格
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
if ! declare -F preflight >/dev/null; then
  echo "  ✗ code-source-discipline.sh：source .claude/scripts/preflight.sh 没成（上面一行报了找不到），preflight 没有定义、参数已被清空，拒绝往下跑（不拦的话 --check 与 --list 会变成全跑）"
  echo "     → 怎么办：这棵树缺 .claude/scripts/preflight.sh：建副本时连 .claude/scripts 与 .claude/singlefs-ai-sop 一起拷（scripts/hosts/minimal-copy.py 逐条加 --with .claude/scripts=… 与 --with .claude/singlefs-ai-sop=…），或回主工作区跑"
  exit 2
fi
STAGE_NAME="$(basename "$0")"
STAGE_DIRECTORY="$(cd "$(dirname "$0")" && pwd)"
# 共用模块在这份阶段所在的仓里（样本目录里没有 scripts/）：cd 进被判的仓之前取
STAGE_REPOSITORY="$(cd "$STAGE_DIRECTORY/../.." && pwd)"
CRASH_CASE_CHECK="$STAGE_REPOSITORY/scripts/crash/crash-case-check.py"
FORMAT_CONST_LIBRARY="$STAGE_DIRECTORY/lib-format-const.py"
ITEM_REF_LIBRARY="$STAGE_DIRECTORY/lib-item-ref-status.py"   # 按编号取决策文件、认译文后缀的判法只在这一份
OWED_LIBRARY="$STAGE_DIRECTORY/lib-owed.py"
LAG_BASELINE_LIBRARY="$STAGE_DIRECTORY/lib-lag-baseline.py"   # 滞后表与豁免表和基准那一版比（只缩不涨）：取基准那一版、切表行的读法只在这一份（mutation-tables、reading-discipline 两格调它）
source "$STAGE_DIRECTORY/lib/stage-cells.sh"
bad() { printf '  ✗ %s\n' "$*"; }
howto() { printf '     → %s\n' "$*"; }

stage_cell vague-names cell_vague_names "函数名与类型名不只由空泛词拼成" \
  "按 .claude/singlefs-ai-sop/rules/code-discipline.md「名字：光看名字，就说得出它是什么、做什么」改名；一时改不完的文件登记进 .claude/naming-vague-exclude（一行 <路径>  # 理由），排除表里指向不存在的与已经改完的行删掉，排除只缩不涨"
stage_cell test-file-names cell_test_file_names "测试文件名不按里程碑、步号起" \
  "照 crash-case-check 那一句出路按内容改名（领域在前、场景在后）；判法写在 scripts/crash/crash-case-check.py 文件头，规则在 .claude/rules/verification.md「定义与名字」"
stage_cell test-function-names cell_test_function_names "#[test] 函数名不按里程碑、步号、欠账号起" \
  "照 crash-case-check 那一句出路按测的对象改名，来历进 commit message 或 .claude/kb/；豁免表 .claude/gate.d/test-function-names-exempt.tsv 只缩不涨，改了名的删那一行；判法写在 scripts/crash/crash-case-check.py 文件头 test-names 一段"
stage_cell format-constants cell_format_constants "格式常量在 kb 与实验源码之间同步" \
  "权威是 kb 里的 format-const 标记：改常量三处一起动（kb 标记与正文、实验源码的 const 与钉死它的单测、重跑实验并更新 research/results/ 的产物），旧值只许留在「## 历史版本」之后与 *-history.md 里；原样保存的冻结证据改不了，登记进 .claude/format-constants-exclude（一行 <路径>  # 理由），排除表里指向不存在的与排不到文件的行删掉"
stage_cell clause-enums cell_clause_enums "条文列出来的封闭集合，与代码里兑现它的那个枚举逐个成员对上" \
  "先判哪一边错了：代码加了成员就写回那条分项正文、再往 .claude/gate.d/clause-enum-pairs.tsv 补一行；条文才是对的就改代码；别只改表让两边看着一致，改分项正文照 .claude/rules/format-evolution.md 记进变更史"
stage_cell feature-bits cell_feature_bits "feature bit 位号：记账表与 D15 登记表逐行一致、不跳号，代码引用的位都在记账表里" \
  "位号的唯一登记位是 D15（格式冻结政策） 已定项 4 那张表：两处不一致改 .claude/kb/feature-bits.md 的记账表；每张位图从 0 严格递增不跳号；代码新引用的位先在两处登记"
stage_cell mutation-tables cell_mutation_tables "每个实验二进制都有成形的变异表、锚点唯一，crates/mutations.tsv 的静态几样" \
  "缺表的写 scripts/mutation/tables/<bin名>.tsv；锚点改到今天源码里逐字存在、只出现一次的那一段；crates/mutations.tsv 照上面各条的出路改（六段、只许 \\n 转义、点名的测试跑得到、改的文件在门禁 crates-mutation-replay 的拷贝范围里），改完单跑那几行证明点名的测试红"
stage_cell absolute-assertions cell_absolute_assertions "每个实验二进制都有钉绝对值的断言" \
  "给它加一条把被量的那个数钉死的断言，绝对值由独立算术给出、不许从代码里读回来；加完用 scripts/mutation/mutate.sh 证明它会红"
stage_cell reading-discipline cell_reading_discipline "实验源码的两条读数纪律：种子不许折叠，读数不许恒为字面量 0" \
  "种子先过一次乘法混淆再置位（seed.wrapping_mul(0xD1B5_4A32_D192_ED03) | 1），恒为 0 的字段接上真读数；动不了的存量登记进 .claude/gate.d/experiment-seed-fold-lag.tsv / experiment-constant-reading-lag.tsv，挂 .claude/kb/checks-owed.md 里开着的 C59 / C60，只缩不涨"
stage_cell unsafe-allowances cell_unsafe_allowances "crates/ 里放开 unsafe 的地方与登记表逐处对上（登记的每个函数上恰好一处放开、函数里 unsafe 处数与登记相同，没登记放开的 crate 零处）" \
  "先看红的是哪一条：登记行不成形、同一个函数登了两行、同一个根几行的级别不一样、理由栏或出处栏缺、根没写 forbid / deny、登记的 bin 根不在、放开的总数与登记的行数不等、多出的 allow(unsafe_code) 或 unsafe 关键字、放开的位置不是登记的函数（方法、impl 块）、登记的函数上不是恰好一处放开、函数里 unsafe 的处数与登记的不等、unsafe 上方缺 // SAFETY: 或没提到那一行登记的词、tests/ 里写了 unsafe、没登记的 bin 根写了 unsafe。没理由的放开删掉；确要新增一处，改 .claude/gate.d/unsafe-allowances.tsv 的登记并走三方（.claude/rules/three-way-inference.md），不改这一格的判法去迁就"
stage_cell gpu-exemptions cell_gpu_exemptions "不走 GPU 的计算与豁免表逐项对上：代码里每一处 // gpu-exempt: <截名> 都有表行、每一行都有标记" \
  "代码里不走 GPU 的那一截在函数上一行写 // gpu-exempt: <截名>，豁免表 .claude/gate.d/gpu-exemptions.tsv 登记同名一行（每加一行先问用户点头）；表里一处标记都没有的行，那一截上了 GPU 就删掉、没上就补标记；改完单跑 python3 scripts/crash/gpu-exemptions-check.py check 到它退 0"
stage_cell crash-point-replay-segments cell_crash_point_replay_segments "崩溃点重放的按段登记成形：每行五段、项名在合表里、流名合写法、段号与段写数是非负整数且段写数大于 0、摘要是 64 位小写十六进制、同一条流同一段只一行" \
  "照上面逐处列的改 .claude/gate.d/crash-point-replay-segments.tsv（写法在它的表头）：每行五段（项名、流名、段号、段写数、段写表摘要），项名照合表 .claude/gate.d/crash-point-replay-items.tsv 写，流名照引擎驱动行的 flow= 写（字母起头，只许字母、数字与 _），段号与段写数写十进制整数、段写数大于 0，摘要写那一段写表的 sha256（64 位小写十六进制），同一条流同一段只留一行；文件不在就照表头建回来；改完单跑 python3 scripts/crash/lib_crash_point_replay_items.py segments 到它退 0"
stage_cell crate-sources-reachable cell_crate_sources_reachable "每一份 .rs 都是某个 crate 的目标，或经 #[path]、mod、include! 引到，没有 cargo 不编的落空源文件" \
  "照上面逐份列的办：要编的在它那个 crate 的 Cargo.toml 里登记成 [[bin]]（或 [lib]、[[test]]），或从某个目标用 #[path]、mod 引它；不要了就删掉。判法在 scripts/gate/crate-sources-reachable.py 文件头"
stage_cell ordinal-names cell_ordinal_names "代码里我们自己的名字（函数、变量、常量、模块、测试、环境变量、文件与目录）不带序号与位置词" \
  "照 .claude/agent-common.md「写」一节改成说内容的名字（写它是什么、做什么，例 enumerate_crash_states、new_pool_file_creation_stream），改过的名字登记进 .claude/kb/term-renames.md；一时改不完的文件登记进 .claude/naming-ordinal-exclude（一行 <路径>  # 理由），排除表里指向不存在的与已经改完的行删掉，排除只缩不涨；词表是 .claude/naming-ordinal-tokens"
stage_cell data-names cell_data_names "字符串字面量（元组首列、=> 右侧、键= 的键名）与 kb 表格里名字形态的串也按命名纪律判：单字母、没登记的缩写判红" \
  "名字写全，或把缩写登记进 .claude/abbreviations（一行 <缩写>  # 全称与含义）；已经落进留存产物、改名要连产物一起重跑的存量登记进 .claude/data-name-lint-exclude（一行 <路径>:<串>  # 理由），排除表里已经没有违规的行与指向不存在路径的行删掉，排除只缩不涨；判法与开关在 scripts/gate/data-name-lint.py 文件头"
stage_cell verus-ghost-names cell_verus_ghost_names "被验 crate（verus-pins.tsv 登记的）里不自己定义名叫 Ghost / Tracked 的类型，也不把 vstd 之外的东西 use … as 改名成这两个名字" \
  "给自定义的类型、改名的东西换一个说出它是什么的名字（Ghost / Tracked 只留给 vstd 的那两个，函数单元图按名字认它们）；判据在 code-source-discipline.sh 里这一格函数上方的注释"
stage_cell refused-tests cell_refused_tests "验证包里拒算的测试函数（单元键算不出、按条复用时照跑不复用）与拒算登记表逐条对上：拒算而没登记、登记了而已经不拒算都判红" \
  "新出现的拒算先改测试让单元键算得出（不读活树、不读运行时拼出的仓根路径）；断言就是关于真仓、非读活树不可的，交主 agent 二次确认，确认了由主 agent 在 .claude/gate.d/refused-tests.tsv 加一行「测试函数全名<TAB>为什么非读活树不可」（实现员不登记）；表里已经不拒算的行删掉；现读每条的状态跑 python3 scripts/gate/refused-test-registry.py list"
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

# 按格组 source 三份库；它们读的 ROOT、STAGE_NAME、STAGE_DIRECTORY、STAGE_REPOSITORY 都在上面设好
source "$STAGE_DIRECTORY/lib/code-source-discipline-naming-and-constants-cells.sh"
source "$STAGE_DIRECTORY/lib/code-source-discipline-mutation-and-reading-cells.sh"
source "$STAGE_DIRECTORY/lib/code-source-discipline-unsafe-and-registry-cells.sh"


# 这一道的格互不依赖、都只读被判的树：并行跑（.claude/gate.d/lib/stage-cells.sh 文件头「格并行」）
stage_cells_run "$ROOT"
