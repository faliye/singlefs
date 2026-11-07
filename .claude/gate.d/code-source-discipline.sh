#!/usr/bin/env bash
# task-server-cargo-family: -p singlefs-unit-graph --release
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的 Rust 源码、测试文件名、kb 文本、登记表与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求；refused-tests 格另起 admission.py 的 test-plan 算键、读判定库，连不上或算不出报本次未跑
# gate-stage: 代码类的源码纪律：名字不只由空泛词拼成、测试文件与 #[test] 函数不按里程碑、欠账号起名；kb 登记的格式常量、条文集合与 feature bit 位号与代码逐个对得上；实验二进制有成形的变异表且锚点唯一、有钉绝对值的断言、种子不折叠与读数不恒为字面量 0；crates/ 里放开 unsafe 的地方与登记表逐处对上；不走 GPU 的计算与豁免表逐项对上；崩溃点重放的按段登记成形；代码里我们自己的名字不带序号与位置词；被验 crate 里不自定义、不改名出 Ghost / Tracked；拒算的测试函数与登记表逐条对上
# gate-fixture: yes
# gate-category: 代码类
# gate-similar: doc-text.sh 也是全仓扫写法，但判的是登记过的禁用写法与改过名的术语的旧名，对象是全部文本文件；这里的 vague-names 与 test-file-names 两格判 .rs 里声明的名字由哪些词拼成、测试文件叫什么；ordinal-names 格判 crates/、scripts/、.claude/gate.d、.claude/hooks 下 .rs、.sh、.py 里的标识符、环境变量名与文件名按词边界含不含词表里的序号与位置词，注释与字符串先剥掉，不判文本（注释与文档里的旧名归 doc-text 的 term-renames 格，只认登记过的）
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
#   mutation-tables      scripts/mutation/tables/ 与 crates/mutations.tsv（后者经 scripts/mutation/crates-mutation-rows.py static-check）
#   absolute-assertions  scripts/experiment-apparatus 下除 lib.rs、main.rs 之外的全部，crates/*/src/bin 下以 e<数字>_ 开头的
#   reading-discipline   C59 种子折叠、C60 恒为字面量 0 的读数，两张豁免表挂开着的欠账
#   unsafe-allowances    .claude/gate.d/unsafe-allowances.tsv 与 crates/ 下全部 .rs
#   gpu-exemptions       .claude/gate.d/gpu-exemptions.tsv 与 crates/ 下全部 .rs（经 scripts/crash/gpu-exemptions-check.py check）
#   crash-point-replay-segments  .claude/gate.d/crash-point-replay-segments.tsv（五列：项名、流名、段号、段写数、段写表摘要）与合表 .claude/gate.d/crash-point-replay-items.tsv（经 scripts/crash/lib_crash_point_replay_items.py segments）
#   crate-sources-reachable  scripts/experiment-apparatus 与 crates 下全部 .rs（经 scripts/gate/crate-sources-reachable.py，判法在它文件头）
#   ordinal-names        词表 .claude/naming-ordinal-tokens，还没改完的文件 .claude/naming-ordinal-exclude；扫 crates/、scripts/、.claude/gate.d、.claude/hooks 下 git 列出的 .rs、.sh、.py
#   data-names           scripts/experiment-apparatus/*.rs 与 crates/*/src/**/*.rs 的字符串字面量、.claude/kb/**/*.md 的表格单元格（经 scripts/gate/data-name-lint.py，判法在它文件头）；登记表 .claude/abbreviations，存量排除表 .claude/data-name-lint-exclude
#   verus-ghost-names    .claude/gate.d/verus-pins.tsv 的 crate 行登记的目录，src/ 下全部 .rs
#   refused-tests        .claude/gate.d/refused-tests.tsv 与 admission.py test-plan 对验证包不带特性那一趟打的 refused 行（判法在 scripts/gate/refused-test-registry.py 文件头）
# 每格的判据、射程、管不到的写在它自己那一段注释里。
# 参数解析、样本根的 .gate-cells、每格一个子 shell、汇总与退出码在共用库 lib/stage-cells.sh。
# 规则：.claude/rules/verification.md「门禁的结构」「门禁管哪一半」。
#
# 样本：fixtures/code-source-discipline.sh/ 下每格至少一对 <格名>-red / <格名>-green，每份样本根的 .gate-cells 只点名那一格；
# 各格注释里说的 red / green 就是那一格自己的样本。
# 弄坏开关。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（fixtures/code-source-discipline.sh/ 下的目录名，
# 括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   STAGE_CELLS_BREAK=red-swallowed  共用库吞红：红格按绿记、整道退 0
#       absolute-assertions-red、clause-enums-red、crash-point-replay-segments-red、data-names-red、feature-bits-red、format-constants-exclusion-red、format-constants-red、gpu-exemptions-red、
#       mutation-tables-duplicates-lag-red、mutation-tables-red、mutation-tables-undefined-identifier-red、ordinal-names-red、reading-discipline-red、test-file-names-red、test-function-names-red、crate-sources-reachable-red、
#       mutation-tables-ignore-comments-and-packages-red、mutation-tables-manifest-path-red、mutation-tables-reachability-red、mutation-tables-release-red、
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
#       feature-bits-trilingual-red、feature-bits-two-originals-red、verus-ghost-names-red、refused-tests-red、
#       vague-names-red（期望退出 1，实测 0）
#   STAGE_CELLS_BREAK=marker-ignored  共用库不认 .gate-cells：每份样本十七格都跑
#       absolute-assertions-green、clause-enums-green、crash-point-replay-segments-green、data-names-green、feature-bits-green、format-constants-green、gpu-exemptions-green、
#       mutation-tables-duplicates-lag-green、mutation-tables-green、mutation-tables-inline-module-literals-green、ordinal-names-green、reading-discipline-green、
#       mutation-tables-ignore-comments-and-packages-green、mutation-tables-manifest-path-green、mutation-tables-reachability-green、mutation-tables-release-green、
#       clause-enums-trilingual-green、feature-bits-trilingual-green、format-constants-trilingual-green、
#       test-file-names-green、test-function-names-green、crate-sources-reachable-green、unsafe-allowances-deny-one-green、unsafe-allowances-green、unsafe-allowances-several-rows-green、unsafe-allowances-statement-continued-green、
#       unsafe-allowances-bin-root-green、unsafe-allowances-unsafe-impl-green、unsafe-allowances-same-method-two-types-green、
#       unsafe-allowances-no-mangle-registered-green、unsafe-allowances-bin-helper-directory-registered-green、unsafe-allowances-unsafe-below-the-first-line-of-a-method-chain-green、
#       unsafe-allowances-same-method-name-in-two-inherent-impls-green、unsafe-allowances-macro-expands-three-unsafe-for-one-keyword-green、
#       unsafe-allowances-one-comment-covers-the-second-unsafe-of-a-chain-green、verus-ghost-names-green、refused-tests-green、vague-names-green（期望退出 0，实测 1）；absolute-assertions-red、
#       clause-enums-red、crash-point-replay-segments-red、data-names-red、feature-bits-red、format-constants-exclusion-red、format-constants-red、
#       mutation-tables-duplicates-lag-red、mutation-tables-red、mutation-tables-undefined-identifier-red、mutation-tables-ignore-comments-and-packages-red、mutation-tables-manifest-path-red、mutation-tables-reachability-red、mutation-tables-release-red、
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
#       reading-discipline-red（输出里找不到「crates/singlefs-verification/s…」）
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
#       data-names-green（期望退出 0，实测 1）；data-names-red（输出里找不到「scripts/experiment-apparatus/e1_demo.rs:1  元组首列」）
#   DATA_NAME_LINT_BREAK=kb-tables-skipped  data-names 不扫 kb 表格
#       data-names-red（输出里找不到「.claude/kb/demo.md:3  kb 表格单元格」）；data-names-green（输出里找不到「数据形态的名字合命名纪律：扫了 2 份文件、8 个串…」）
#   DATA_NAME_LINT_BREAK=single-letters-passed  data-names 单字母放行
#       data-names-red（输出里找不到「「x」是单字母」）
#   DATA_NAME_LINT_BREAK=registered-ignored  data-names 不认样本自己的 .claude/abbreviations 里登记的缩写
#       data-names-green（期望退出 0，实测 1）
#   DATA_NAME_LINT_BREAK=numbered-ignored  data-names 不认 e<数字> 这类整类登记
#       data-names-green（期望退出 0，实测 1）
#   DATA_NAME_LINT_BREAK=exclusions-ignored  data-names 不认排除表
#       data-names-green（期望退出 0，实测 1）；data-names-red（输出里找不到「…:renamed_long 已经一个违规都没有了」）
#   DATA_NAME_LINT_BREAK=stale-exclusions-passed  data-names 排除表里已经没有违规的行、指向不存在路径的行不判
#       data-names-red（输出里找不到「已经一个违规都没有了」）
#   DATA_NAME_LINT_BREAK=comments-scanned  data-names 注释里的串也扫
#       data-names-green（期望退出 0，实测 1）；data-names-red（输出里找不到「5 处违规、排除表 2 处不合规（扫了 2 份文件、9 个串）」）
#   DATA_NAME_LINT_BREAK=call-arguments-judged  data-names 函数调用的字符串实参（左括号前是标识符）也当元组首列
#       data-names-green（期望退出 0，实测 1）；data-names-red（输出里找不到「5 处违规、排除表 2 处不合规（扫了 2 份文件、9 个串）」）
#   GATE_MUTATION_TABLES_BREAK=named-test-unsearched  mutation-tables 不判点名的测试在不在选的目标里
#       mutation-tables-red（输出里找不到「M_点名的测试不存在」那一条）；mutation-tables-ignore-comments-and-packages-red（输出里找不到「M_点名的用例两个包里都没有」那一条）
#   GATE_MUTATION_TABLES_BREAK=named-test-target-ignored  mutation-tables 点名的测试在整个包里找、不看选的目标
#       mutation-tables-red（输出里找不到「M_库里点名只在集成测试里的用例」那一条）
#   GATE_MUTATION_TABLES_BREAK=named-test-definitions-only  mutation-tables 点名的测试只认 fn <名> 的定义（宏展开出来的认不出）
#       mutation-tables-green、mutation-tables-inline-module-literals-green、mutation-tables-ignore-comments-and-packages-green（期望退出 0，实测 1）
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
#       mutation-tables-inline-module-literals-green（期望退出 0，实测 1）
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
# absolute-assertions 没有弄坏开关，mutation-tables 别的几样与 reading-discipline（只有共用库那两项）判别力靠各自的样本。
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

# ── vague-names ──
# 判据：`git ls-files` 列出的 crates/**/*.rs 与 scripts/experiment-apparatus/**/*.rs 里，每一处 `fn <名>` 与 `struct|enum|type|trait <名>`：
# 函数名按 `_` 切词、类型名按大写字母切词，切出来的每一个词（小写）都在 .claude/naming-vague-words 里，判违规，逐处列「文件:行 名字」。
# 不判的：`impl <trait> for <类型>` 块里的方法与关联类型（名字是 trait 定的）、`main`、以 `\n\` 续行的字符串字面量里的源码（测试夹具）、宏展开出来的名字（认不出）。
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

listed = subprocess.run(["git", "-c", "core.quotepath=false", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", "crates", "scripts/experiment-apparatus"],
                        capture_output=True)
paths = sorted(path for path in listed.stdout.decode("utf-8", "replace").split("\0") if path.endswith(".rs") and os.path.isfile(path))
if not paths:
    print("  ! 本次无对象可判：crates/ 与 scripts/experiment-apparatus/ 下一份 .rs 都没列出来")
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
        inside_string_literal = line.rstrip("\n").endswith("\\n\\") and BROKEN != "string-continuation-judged"
        if match and not (match.group(1) in ("fn", "type") and inside_trait_impl) and match.group(2) != "main" and not inside_string_literal:
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
# 判据：scripts/crash/crash-case-check.py --only file-names <项目根>：crates/singlefs-verification/tests/*.rs
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
  echo "     → 怎么办：照上面那一句出路改名；判法写在 scripts/crash/crash-case-check.py 文件头，规则在 .claude/rules/verification.md「定义与名字」。"
  return 1
}

# ── test-function-names ──（还 checks-owed.md C638：用例函数名带欠账号没有判）
# 判据：scripts/crash/crash-case-check.py --only test-names <项目根>：crates/*/tests/ 下（含子目录）与 crates/*/src/ 下 .rs 里带 #[test] 的函数名，
# 拿与 test-file-names 同一条 MILESTONE_FILE_NAME 判（里程碑序数、步号、增补号、并行线号、欠账号 c<三位数>），判法只在那一份里。
# 立账时已在的存量登记在 .claude/gate.d/test-function-names-exempt.tsv（「<文件><TAB><函数名><TAB># 理由>」），登记了的不判，对不上的豁免行判红；表只缩不涨（不判涨，靠 review）。
# 一条 #[test] 都没有：这一格本次无对象可判。
# 管不到的：带实验号（e<数>）与 m<数>_ 的名字（算不算来历没定，不在那条判据里）；宏展开出来的测试。
# 样本：test-function-names-red（src/ 里一条 c123_、tests/ 子目录里一条带 _c789 的、豁免表里一行对不上，三处点名判红；登记了的 c456_ 不报）；
#   test-function-names-green（一条按内容起名的、一条登记了豁免的 c456_，报「查了 2 条……豁免……1 条不判」）。
cell_test_function_names() {
  local test_names_exit
  if python3 "$CRASH_CASE_CHECK" --only test-names "$ROOT"; then test_names_exit=0; else test_names_exit=$?; fi
  case "$test_names_exit" in
    0|77) return "$test_names_exit" ;;
  esac
  echo "  ✗ #[test] 函数名：crash-case-check --only test-names 退 $test_names_exit（逐处列在上面）"
  echo "     → 怎么办：照上面那一句出路按测的对象改名，来历进 commit message 或 .claude/kb/；判法写在 scripts/crash/crash-case-check.py 文件头 test-names 一段。"
  return 1
}

# ── 格 format-constants：格式常量在 kb 与实验源码之间同步
#
# 权威在 kb 的 format-const 标记，紧挨着定这个值的那句话（文法与解析在 lib-format-const.py，doc-registries、checker-independence-and-sync 共用）：
#     <!-- format-const: JOURNAL_HEADER_BYTES = 86 stale=hdr=84|room=428 -->
# 判红：标记按文法读不出来（多写了键、值不是整数）；同一个名字登记两处（同一份文件里两次也算）；
# scripts/experiment-apparatus/、research/、crates/ 源码里同名 const 的值与标记不等（只认类型是单个标识符、值是整数字面量的 const，同名的数组扫描表不算）；
# stale= 列的旧值字面串还留在 kb 正文或源码里。旧值只许留在「## 历史版本」之后与 *-history.md 里。
# 扫哪些源码：`git ls-files --cached --others --exclude-standard` 列出的 scripts/experiment-apparatus/、research/、crates/ 下的 .rs（已跟踪的与还没跟踪、没被忽略的；research/ 下只剩 prompts/ 与 results/，扫它是为了冻结证据里的旧值照判、由排除表逐份放行）；
# 被 .gitignore 忽略的编译副本不扫（例如 scripts/experiment-apparatus/target/ 下的 crates 副本：本机有、干净的克隆里没有，扫它等于两台机器判出两个结果）。
# 冻结证据（research/prompts/ 下原样保存的模型产出）改了就不是原样（.claude/singlefs-ai-sop/rules/evidence-discipline.md「原样保存的证据不许事后改」），
# 登记在 .claude/format-constants-exclude（一行 <路径>  # 理由，目录写成以 / 结尾）：登记路径下的 .rs 既不比 const 的值、也不查旧值字面串。没有这个文件按空表算。
# 排除表判红：路径不存在；路径在、但没排到一份被 git 列出的 .rs；# 后面的理由不足 4 个字。排除只缩不涨：路径改名了、证据删了，那一行就删。
# 成功句报：扫了几份 .rs、排除表排除了几份。不在 git 仓里（git ls-files 起不来）：本次未跑，退 77。
# 为什么要这一格：research-build-and-replay 逐字节比产物，源码与产物一起停在旧值时它照样绿；doc-registries 只看 kb 内部。
# 射程：只管登记了标记的常量，没登记的新格式常量照样能漂，靠人。
# 译文：登记位和旧值字面串只认中文原文（kb 下不带 .en / .ja 后缀的 .md）。译文（<名>.en.md、<名>.ja.md）里同一个标记抄了一遍，算进来就是同一个名字登记三处；
# 译文的历史节标题是译名，切不开，里面留着的旧值也会算成残留。译文跟没跟上中文原文归 doc-text 的 trilingual-files 一类格，不归这一格。
# 样本里 format-constants-trilingual-green 放三语三份，译文抄了标记、历史节留着旧值，日语译文的正文没跟上、还写着旧值，判绿。
# 样本：red 有源码落后于 kb 的常量、正文残留的旧值、多写了键的标记、同一份文件里登记两次的名字，以及值跨行写、带 pub(crate) 的两处落后声明；
# green 另放一张与格式常量同名的扫描表（数组类型），判绿。
# 排除表的样本：red 里放一份落后的冻结证据、不登记，判红；green 里同样一份登记进排除表，判绿，另有一份被 .gitignore 忽略的编译副本带着旧值，不扫；
# format-constants-exclusion-red 的排除表三行各错一样（路径不存在、目录被忽略而排不到文件、没写理由），三条都要点名。
cell_format_constants() {
  [[ -d .claude/kb ]] || { echo "  ! 找不到 kb，这一格跳过"; exit 77; }

  python3 - "$FORMAT_CONST_LIBRARY" "$ITEM_REF_LIBRARY" <<'PY'
import glob, importlib.util, os, subprocess, sys

library_spec = importlib.util.spec_from_file_location("format_const", sys.argv[1])
format_const = importlib.util.module_from_spec(library_spec)
library_spec.loader.exec_module(format_const)
item_ref_spec = importlib.util.spec_from_file_location("item_ref_status", sys.argv[2])
item_ref_status = importlib.util.module_from_spec(item_ref_spec)
item_ref_spec.loader.exec_module(item_ref_status)
format_constants_break = os.environ.get("CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK", "")
break_value_comparison = format_constants_break == "format-constants"
EXCLUDE_FILE = ".claude/format-constants-exclude"

def body_of(text):
    """正文 = 「## 历史版本」之前那一段。历史里留旧值是文档纪律要求的，不判红。"""
    i = text.find('\n## 历史版本')
    return text if i < 0 else text[:i]

# *-history.md 不是登记位：变更史原样引用标记是历史陈述，不算第二处权威记录。
# 译文（<名>.en.md、<名>.ja.md）也不是登记位：同一个标记在三份里各抄一遍，算成第二、第三处权威记录就是误判重复；
# 译文的「历史版本」节标题是译名（如 `## Historical Versions`），body_of 切不开，里面按历史陈述留着的旧值也会被当成残留。认译文后缀的判法只在 lib-item-ref-status.py 的 language_of。
kb_all = sorted(glob.glob('.claude/kb/**/*.md', recursive=True))
kb_files = [f for f in kb_all if not f.endswith('-history.md') and '/decisions-history/' not in f
            and (format_constants_break == "format-constants-translations-read" or not item_ref_status.language_of(f))]   # 弄坏开关：译文也进来

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
# 实验源码在 scripts/experiment-apparatus/，冻结证据在 research/prompts/（排除表逐份放行），crates/ 照判
scanned_roots = ["scripts/experiment-apparatus", "research", "crates"]
if format_constants_break == "format-constants-research-unscanned":
    scanned_roots = ["scripts/experiment-apparatus", "crates"]   # 弄坏开关：research/ 整片不扫，冻结证据里的旧值看不见、排除表那一行排不到文件
if format_constants_break == "format-constants-ignored-files-scanned":
    listed_sources = [path for root in scanned_roots for path in glob.glob(f'{root}/**/*.rs', recursive=True)]   # 弄坏开关：按盘上有什么扫，被忽略的编译副本也进来
else:
    listing = subprocess.run(["git", "-c", "core.quotepath=false", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", *scanned_roots], capture_output=True)
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
# 源码改了而产物没重跑，由 research-build-and-replay 逐字节比对抓。
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
        print(f'  ✗ {problem}')   # gate-lint:detail
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
# 分项正文取哪一份：只认中文原文 <号>-<名>.md（判法在 lib-item-ref-status.py 的 numbered_files），译文 .en.md、.ja.md 的标题是译名、找不到「#### 已定项 k」；
# 中文原文不是恰好一份判红。clause-enums-trilingual-green 放三语三份、译文没有中文小节标题，判绿。
cell_clause_enums() {
  local TABLE=".claude/gate.d/clause-enum-pairs.tsv"
  if [[ ! -f "$TABLE" ]]; then
    echo "  ⊘ 本次未跑：$TABLE 不在，今天无对象可判"
    exit 77
  fi

  local report
  if ! report="$(TABLE="$TABLE" python3 - "$ITEM_REF_LIBRARY" 2>&1 <<'PY'
import os, re, glob, sys, importlib.util

item_ref_spec = importlib.util.spec_from_file_location("item_ref_status", sys.argv[1])
item_ref_status = importlib.util.module_from_spec(item_ref_spec)
item_ref_spec.loader.exec_module(item_ref_status)

break_substring_match = os.environ.get("CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK") == "clause-enums"
break_any_language_decision = os.environ.get("CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK") == "clause-enums-decision-any-language"

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
    # 决策正文有中文原文与 .en.md、.ja.md 两份译文，分项正文只认中文原文（按编号取哪一份的判法只在 lib-item-ref-status.py 的 numbered_files）
    if break_any_language_decision:
        files = sorted(glob.glob(f".claude/kb/decisions/{int(decision[1:]):02d}-*.md"))[:1]   # 弄坏开关：不分语言，取按文件名排序最靠前的一份（三语并存时是 .en.md）
    else:
        files = item_ref_status.numbered_files(".claude/kb/decisions", int(decision[1:]))
    if not files:
        return None, f"找不到 {decision} 的中文原文正文文件（.claude/kb/decisions/ 下不带 .en / .ja 译文后缀的那一份）"
    if len(files) > 1:
        return None, f"{decision} 的中文原文正文文件不止一份：{'、'.join(files)}"
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
# 三条判据，任一条不成立判红：
#   ① 记账表的（类别，位号）集合与已定项 4 登记表逐行对得上，且记账表的「语义一句话」逐字包含登记表同一位的「含义」；
#   ② 每张位图内位号严格递增、不跳号（已分配的位必须是 0..n-1）；
#   ③ `crates/*/src/**/*.rs` 里名字带 INCOMPAT / COMPAT_RO / COMPAT 段的常量，值解得出位号的，那个位号必须在记账表里；
# 格式锁定之前位号可回收、可重排，不判「位用过不许回收」；同一个（类别，位号）出现两行时，位号不再严格递增，由 ② 报。
# 射程：③ 只认写在一行里、值是位掩码字面量的常量声明。值由别的常量合成的（`SUPPORTED_INCOMPAT_BITS`）、声明跨行的，
# 解不出位号，逐个列进成功那句的「没判位号的」名单。不经具名常量的裸字面量（直接写 `0x01`）不在射程里，靠 review。
# 样本：red 的记账表跳号、同一位登记两行不同语义、多出一位登记表里没有，代码样本又引用了表里没有的一位，①②③ 都必须报出来（同一位两行由 ② 报次序不对）；
# green 只有位 0 一行、语义与登记表对得上，判绿。
# D15 取哪一份：只认中文原文 15-<名>.md（判法在 lib-item-ref-status.py 的 numbered_files，孤立译文一份都不算），译文 .en.md、.ja.md 里没有「#### 已定项 4」；
# 中文原文不是恰好一份判红（零份与两份各一句出路）。feature-bits-trilingual-red 放三语三份、记账表比中文登记表多一位，译文登记表把那一位分出去了，
# 必须按中文原文判出「登记表把它列在未分配区间 1..255 里」；feature-bits-trilingual-green 三语三份、对得上，判绿；feature-bits-two-originals-red 编号 15 下有两份中文文件，判红。
cell_feature_bits() {
  [[ -f .claude/kb/feature-bits.md ]] || { echo "  ! 没有 .claude/kb/feature-bits.md，这一格无对象可判"; exit 77; }

  python3 - "$ITEM_REF_LIBRARY" <<'PY'
import glob, importlib.util, os, re, sys

item_ref_spec = importlib.util.spec_from_file_location("item_ref_status", sys.argv[1])
item_ref_status = importlib.util.module_from_spec(item_ref_spec)
item_ref_spec.loader.exec_module(item_ref_status)

break_any_language_decision = os.environ.get("CODE_CONSTANTS_ENUMS_BITS_MATCH_KB_BREAK") == "feature-bits-decision-any-language"
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
# 决策正文有三份：中文原文 15-<名>.md 与译文 15-<名>.en.md、15-<名>.ja.md；登记位只认中文原文（译文的标题与表头是译名，找不到「#### 已定项 4」）。
# 按编号取哪一份文件的判法只在 lib-item-ref-status.py 的 numbered_files（不给引用方就取中文原文，孤立译文一份都不交回）。
if break_any_language_decision:
    decision_files = sorted(glob.glob(".claude/kb/decisions/15-*.md"))[:1]   # 弄坏开关：不分语言，取按文件名排序最靠前的一份（三语并存时是 .en.md）
else:
    decision_files = item_ref_status.numbered_files(".claude/kb/decisions", 15)
d15_allocated = {}      # (类别, 位号) -> 含义
d15_unallocated = {}    # 类别 -> [(起, 止)]
d15_path = ""
if not decision_files:
    reject(
        "找不到 D15（格式冻结政策） 的中文原文决策文件（.claude/kb/decisions/15-*.md，不带 .en / .ja 译文后缀的那一份）——位号的唯一登记位不在，没法比对",
        [],
        "确认决策正文的中文原文还在 .claude/kb/decisions/ 下，只有译文（15-*.en.md、15-*.ja.md）不算；改了文件名就把 15- 这个编号前缀留着，这一格按它找。",
    )
elif len(decision_files) > 1:
    reject(
        f"D15（格式冻结政策） 的中文原文决策文件不止一份：{'、'.join(decision_files)}——位号的唯一登记位不知道是哪一份，没法比对",
        [],
        "编号 15 只留一份中文原文（<号>-<名>.md）；译文放在同名的 .en.md、.ja.md 里，不另起编号相同的第二份中文文件。",
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
# research 那侧：scripts/experiment-apparatus/ 下每个 *.rs 在 scripts/mutation/tables/ 下有同名 .tsv，至少一条成形的变异
# （三段制表符分隔）、没有坏行；每条原文（\n 还原成换行）在源码里恰好命中一次（mutate.sh 把命中 0 次或多次的那一条记「⚓ 锚点失效」、别的照跑，整轮退 3）；
# 原文与替换文里只许 \n 这一种反斜杠转义。没有同名二进制的表（shell 探针的）不判锚点，成功行逐个列名。
# 重复行：每张表（有没有同名二进制都判）里除第一列变异名外各列逐字全同的几行判红，逐组列表名、行号与变异名（一处改法写了几行，条数看着多、真改法没多）；
#   弄坏开关 GATE_MUTATION_TABLES_BREAK=research-duplicates-unchecked 不判这一样：red 样本 e1_sample.tsv 第 3、4 行那一组找不到。
# 为什么：kb 里每一句「N 条变异全部被抓」都靠这些表复现。
#
# crates/mutations.tsv（门禁 crates-mutation-replay 的表，六段）：那一道要跑几个钟头，改 crates/ 的实现员不跑它，
# 它派活之前判的几样在这里先静态判。判法只有一份，在 scripts/mutation/crates-mutation-rows.py 的 row_problems，这里经 static-check 调：
#   成形：六段；替换文可以空（删掉原文），其余五段不许空；
#   锚点：原文的 \n 还原成换行后，在「文件」那一段指的源码里恰好命中一次，文件不在、读不了也算（那一道把这样的行记「锚点失效」、别的行照判）；
#   测试名：必须红的测试名是一个用例名（不带正则符号与空白）；
#   转义：原文与替换文里只许 \n 这一种反斜杠转义（别的按字面写进源码，那条变异编不过、等于没跑）；
#   拷贝范围 ①：某一行改的文件不在 .claude/gate.d/stage-inputs.tsv 里那一道登记的路径之内（它只把这些路径拷进每片的源码副本）；
#   拷贝范围 ②：crates/ 下 .rs 里以 "../ 起头、按所在 crate 的根解析之后逃出 crates/、又不在拷贝范围里的字符串字面量
#     （#[path = …] 与注释里的不算）：运行时读它的测试在副本里没改坏就红，那一道记「基线不绿」。
#     管不到：路径拼在变量里、环境变量给的、format! 拼出来的、运行期再 join("..") 的；指到仓根本身的（"../../"）按不在范围里判。
#   替换文引的标识符：static-check 的 UNDEFINED 行（替换文施加之后新出现、被改文件与它所在的包的 .rs 里都一处没有定义的标识符，判法在
#     scripts/mutation/crates-mutation-rows.py 的 replacement_undefined_identifiers）判红：那条变异套上去编不过，crates-mutation-replay 记无效、整道红。
#     成功句报 IDENTIFIERS_CHECKED 那一行的两个数（判了几行、没判几行：文件不是 .rs 或锚点失效）。
#     弄坏开关 GATE_MUTATION_TABLES_BREAK=undefined-identifiers-ignored 不读 UNDEFINED 行：样本 mutation-tables-undefined-identifier-red（替换文引了哪里都没有的 halved_before）判绿、样本判错。
#   点名的测试够得着被改坏的函数：static-check 一律判（判法在 scripts/mutation/crates-mutation-rows.py 文件头「点名测试够不着被改坏的函数」，没有开关参数）。
#     UNREACHABLE（点名测试的闭包里根本没有被改坏的单元，或开 GPU 时被改坏的单元只在 GPU 路上不走的 CPU 判器入口后面：改了它测试照样过）与
#     REACHABILITY_UNJUDGED（判不了够不够得着：锚点失效、拒算、闭包算不成）进红句；REACHABILITY_CHECKED 进成功句（判了几行、没判几行、豁免几行、截过闭包几行、开没开 GPU）。
#     选路表、截断表、豁免表点名的函数在这棵树里一个都没有（门禁样本那种小根）时 static-check 打 REACHABILITY_NOT_JUDGED：成功句写「本次未判，不算判过」，不红；
#     有的在、有的不在是配置错（ERROR，static-check 退 2，整格没判完）。真仓上这一半的结论随多机配置（ENABLE_GPU）变，GPU 没开的机器只判 closure 那一路。
#     弄坏开关 GATE_MUTATION_TABLES_BREAK=reachability-lines-unread 不读这几行：样本 mutation-tables-reachability-red（第 3 行改的函数没有测试调到）判绿、
#     mutation-tables-reachability-green 成功句里没有可达性那半句，两份样本都判错。
#     样本 mutation-tables-reachability-green：照 mutation-tables-green 拷来，crates/mutations.tsv 换成一行（改 counted，点名的测试调得到它），
#     crates 下加 singlefs-verification 一个小包，放着选路表、截断表、豁免表点名的那几个函数（空函数体），所以这一格不是「本次未判」；
#     mutation-tables-reachability-red 是它的整份拷贝，变异表多第 3 行（改 never_called_by_the_test，点名的测试调不到它：closure）。
#   重测试目标只许 release：static-check 的 RELEASE 行（命中重测试目标而第五段没写 --release / --profile release 的行，判法在 scripts/mutation/lib_release_only_tests.py，
#     命中的两样是 gpu 特性与 .claude/gate.d/stage-inputs.tsv 的 crash-case: 行登记的测试目标；登记表读本模块所在那一份仓的，样本根里没有）逐行进红句，点名行号、变异名与依据；
#     成功句报「命中重测试目标的行第五段都写了 --release（判了 N 行」，N 是表的行数（static-check 每行都判）。登记表读不了 static-check 打 ERROR、退 2，照「没跑完」判红。
#     弄坏开关 GATE_MUTATION_TABLES_BREAK=release-lines-unread 不读 RELEASE 行：样本 mutation-tables-release-red 判绿、mutation-tables-release-green 成功句里没有 release 那半句，两份样本都判错。
#     样本 mutation-tables-release-green：照 mutation-tables-green 拷来，crates/mutations.tsv 多第 8 行（-p demo --features gpu --release --lib，点名 lib_case）；
#     mutation-tables-release-red 是同一份拷贝，第 8 行第五段不写 --release（-p demo --features gpu --lib），只这一样判红。
# 另判四样那一道不判的：重复行——「文件、原文、替换文」三样逐字全同的几行判红（点名的测试、cargo 参数不同也算：一处改法留一行），
#   逐组列行号与变异名，组里 cargo 参数也全同的另注一句；变异名相同的两行另判（按名字分格的报告把两条并成一格）。
#   弄坏开关 GATE_MUTATION_TABLES_BREAK=duplicate-key-with-test 把点名的测试也算进重复的键：red 样本第 13、14 行那一组（点名的测试不同）找不到；
# 滞后表：两侧的重复行里还不能删的组（有装置或名单按名字引着、臂表与键表按臂或键登记）登记在 .claude/gate.d/mutation-duplicates-lag.tsv（写法在它的表头），
#   登记的组不判红，成功句前逐组列名、成功句报「重复行滞后 N 组」。判红的：一行不是四列或有空格子；表那一列不是 crates/mutations.tsv、也不是 scripts/mutation/tables/<表名>.tsv，
#   或那张表不在；名字在那张表第一列里找不到；名字此刻不是同一组（三样不全同 / 除变异名外各列不全同）；那一组里还有没登记的名字；名字在表里不唯一、对上不止一组；
#   两行罩同一组；只缩不涨——此刻的一行在基准那一版（scripts/gate/changed-paths.sh 的 gate_diff_base gate）同一张表里找不到名字是它超集的一行。
#   基准里还没有这张表时不比、成功句写明；git 取不到基准那一版判红。名字改了也算涨（按名字认组）。
#   弄坏开关 GATE_MUTATION_TABLES_BREAK=duplicates-lag-ignored 不读滞后表：mutation-tables-duplicates-lag-green 登记的两组照红，mutation-tables-duplicates-lag-red 里滞后表那四处不对也不报，两份样本都判错；
#   duplicates-lag-growth-unchecked 不和基准比：mutation-tables-duplicates-lag-red 里比基准多登记的那一组不红，mutation-tables-duplicates-lag-green 成功句里「没涨」那半句也不打，两份样本都判错。
#   样本 mutation-tables-duplicates-lag-green：crates 表与 e1_sample.tsv 各一组重复、滞后表两行罩住、基准那一版与此刻相同；
#   mutation-tables-duplicates-lag-red：基准那一版只登记 crates 那一组，此刻多登记 e1_sample.tsv 那一组（涨了），另有一行名字找不到、一行名字不是同一组、一行漏登记组里一个名字。
# 包的认法（#[ignore]、筛选词、点名的测试在不在选的目标里这三样都先认包）：cargo 参数在 `--` 之前点名的全部包，每个认成一个包目录，下面三样在包目录的 src/ 与 tests/ 里判：
#   -p <包>、--package <包>（空格隔开或写 =）只认工作区成员：成员从仓根 Cargo.toml 的 [workspace] members 现读（通配照 glob 展开，exclude 的不算），
#     包名取各成员目录的 Cargo.toml 的 [package] name，目录名与包名不必相同；点到不是成员的（例：被 exclude 掉的独立工程 prune-server）判红，出路写 --manifest-path。
#   --manifest-path <路径>（空格隔开或写 =）取那份 Cargo.toml 所在的目录当包目录，路径从仓根算起；解到仓根之外、文件名不是 Cargo.toml、文件不在的判红。
#   同一行既带 -p 又带 --manifest-path、带两次 --manifest-path、--manifest-path 后面没写路径的判红，那一行不再往下判。
#   成功句报两个数：-p 点名的包解出目录的行数、--manifest-path 解出目录的行数。
#   弄坏开关与各自的样本见文件头开关表（package-directory-from-name 等九条）：样本 mutation-tables-manifest-path-green 加了 Cargo.toml 工作区（成员 crates/demo、目录名与包名不同的 crates/renamed-dir），
#   与被 exclude 掉的独立工程 crates/standalone（自己的 Cargo.toml 带空的 [workspace]；测试在它的 src/ 与 tests/ 里），第 7 至 9 行判绿：--manifest-path 空格写法与等号写法各一行、-p renamed-package 一行；
#   mutation-tables-manifest-path-red 是它的整份拷贝，多第 11 至 18 行各判红一样：-p 点非成员、路径在仓根之外、文件名不是 Cargo.toml、文件不在、-p 与 --manifest-path 并写、--manifest-path 写两次、后面没写路径、
#   独立工程里找不到点名的测试。别的 mutation-tables 样本各加了一份只列自己用到的包的工作区 Cargo.toml 与成员的 Cargo.toml。
# #[ignore]、筛选词、点名的测试在不在选的目标里这三样的包取 cargo 参数在 `--` 之前点名的全部包（-p、--package，空格隔开或写 =；--manifest-path 同理）：一行写了几个 -p，点名的测试落在其中任一个包里就算在。
#   弄坏开关 GATE_MUTATION_TABLES_BREAK=one-package-per-row 只认最靠前的那一个 -p：样本 mutation-tables-ignore-comments-and-packages-green
#   第 9 行（-p demo -p retry --lib，点名的测试只在 retry 里）判红、整格退 1，mutation-tables-ignore-comments-and-packages-red 第 11 行的报告里没有 retry 那一个包。
# 点名的测试标了 #[ignore] 而参数里没有 --include-ignored：只认 fn 上面的属性行（每行去掉行尾 // 起的注释），夹在属性之间的注释行照样认得出下面的 fn，
#   但注释行与行尾注释里写的 #[ignore] 不算标了；管不到：属性的字符串值里写的 #[ignore]、跨行的属性。
#   弄坏开关 GATE_MUTATION_TABLES_BREAK=comment-lines-as-attributes 注释行也算属性：样本 mutation-tables-ignore-comments-and-packages-green
#   第 8 行（点名的 commented_ignore_case 只在注释里写着 #[ignore]）判红、整格退 1；
# `--` 之后的筛选词一个都筛不到点名的测试（libtest 比的是完整测试名：模块路径由点名的函数在哪个文件、套在哪几层内联 mod 里算出；
#   配 `mod 名 {` 与花括号之前先把注释、字符串、原始串与字符字面量抹成空白，串里的 `mod tests {` 与落单的花括号不算。
#   弄坏开关 GATE_MUTATION_TABLES_BREAK=inline-module-literals-counted 退回直接扫原文：样本 mutation-tables-inline-module-literals-green 的 banner.rs 里
#   串里的「mod tests { {」与字符字面量 '{' 把顶层的测试算成 banner::tests::…，按 --exact 点名的 banner::banner_length_counts_the_braces 筛不到、整格退 1）；
# 点名的测试在 cargo 参数选的目标里找不到（--lib：src/ 除 src/bin/ 与 src/main.rs；--bin <名>：src/bin/<名>.rs 与 src/bin/<名>/；
#   --test <名>：tests/<名>.rs、tests/<名>/ 与它 mod 进来的 tests/ 下文件；都没写或 --tests、--all-targets 这一类：src/ 与 tests/ 全部），
#   按 scripts/gate/lib_function_units.py 的 rust_lex 切词，名字是其中一个标识符就算找得到（宏展开出来的测试名在调用处是标识符，认得出；
#   只在注释、字符串里出现的不算）。测试改了名、删了，那一道照这一行跑时点名的测试不在输出里，变异永远报没红。
#   弄坏开关 GATE_MUTATION_TABLES_BREAK=named-test-unsearched 不判这一样：red 样本「点名的测试不存在」那几条找不到；
#   named-test-target-ignored 不看选的目标、整个包里找：red 样本里 --lib 点名只在 tests/ 里的 slow_heavy_case 那一条找不到；
#   named-test-definitions-only 只认 `fn <名>` 的定义：green 样本里宏展开出来的 generated_by_a_macro_case 那一行判红、整格退 1。
#
# 这一格不跑测试、不判抓不抓得到（逐条施加替换文、照跑测试，归 research-build-and-replay 的 mutation-tables 格），只保证装置在、形状对。
# research 那侧「表今天还会不会红」靠改实验代码时手跑 scripts/mutation/mutate.sh，门禁不判：C40（变异表没存）。
#
# 样本：red 里 research 那侧有一份没有同名表的实验二进制（e2_naked）、一条锚点命中 0 次、一条替换文带 \& 的、一张不是 UTF-8 的表（setup.sh 现造）、
# 除变异名外各列全同的两行（e1_sample.tsv 第 3、4 行）；
# crates/mutations.tsv 那侧有锚点腐化、转义写进源码、文件列是目录、点名标了 #[ignore] 的用例没带开关、两种筛选词对不上、文件在拷贝范围外、
# 测试名带符号、测试名空着、点名的测试不存在、--lib 点名只在 tests/ 里的测试、同一处改法点名两条不同测试的两行（后五样 setup.sh 现写）各一行，另有一份逃出 crates/ 的相对路径字面量；
# green 是同形的一套都对得上的（含替换文空着的删行变异、读 litmus/ 的测试、点名宏展开出来的测试），每个实验二进制各有一张锚点唯一的表。
# mutation-tables-undefined-identifier-red 是 green 的整份拷贝，crates/mutations.tsv 多一行替换文引了哪里都没有定义的 halved_before（第 8 行），只这一样判红。
# mutation-tables-ignore-comments-and-packages-green 是 green 的整份拷贝，多一个包 crates/retry 与 crates/demo/tests/commented.rs，crates/mutations.tsv 多两行判绿的：
# 第 8 行点名注释里写着 #[ignore]、本身没标的 commented_ignore_case，不带 --include-ignored；第 9 行 -p demo -p retry --lib 点名只在 retry 里的测试。
# mutation-tables-ignore-comments-and-packages-red 是它的整份拷贝，多 crates/demo/tests/interleaved.rs 与两行判红的：第 10 行点名真标了 #[ignore]、
# 注释行夹在属性之间的 interleaved_slow_case，不带 --include-ignored；第 11 行 -p demo -p retry --lib 点名两个包里都没有的测试。
cell_mutation_tables() {
BIN_DIR=scripts/experiment-apparatus
MUT_DIR=scripts/mutation/tables
[[ -d "$BIN_DIR" && -d "$MUT_DIR" ]] || { echo "  ! 找不到 $BIN_DIR 或 $MUT_DIR，这一格跳过"; exit 77; }

missing=(); malformed=()
for src in "$BIN_DIR"/*.rs; do
  case "$(basename "$src")" in lib.rs|main.rs) continue ;; esac   # crate 根的库与主二进制不是一份实验
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
bad = []; stray = []; unreadable = []; duplicates = []; checked = 0; tables_read = 0
check_duplicates = "research-duplicates-unchecked" not in os.environ.get("GATE_MUTATION_TABLES_BREAK", "")
def stray_escapes(segment_name, segment):
    """mutate.sh 与 crates-mutation-replay 只把 \\n 还原成换行，别的反斜杠按字面写进源码（C427）。"""
    return [(segment_name, match.group(0)) for match in re.finditer(r"\\(.)", segment) if match.group(1) != "n"]
def duplicate_groups(stem, table_lines):
    """一张表里除第一列变异名外各列逐字全同的几行：每组一项 (表名, 「第 a、b 行（名；名）」, 组内各行的变异名用 \\x1f 连起来)。注释行与空行不算。"""
    groups = {}
    for lineno, line in enumerate(table_lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        parts = line.split("\t")
        groups.setdefault(tuple(parts[1:]), []).append((lineno, parts[0]))
    return [(stem, "第 " + "、".join(str(lineno) for lineno, _name in rows) + " 行（" + "；".join(name for _lineno, name in rows) + "）",
             "\x1f".join(name for _lineno, name in rows))
            for rows in groups.values() if len(rows) > 1]
for tsv in sorted(glob.glob(os.path.join(mut_dir, "*.tsv"))):
    stem = os.path.basename(tsv)[:-4]
    src_path = os.path.join(bin_dir, stem + ".rs")
    # 重复行每张表都判（有没有同名二进制都算）；读不了的表在下面锚点那一段判红，没有同名二进制的读不了也判红
    try:
        table_lines = open(tsv, encoding="utf-8").read().split("\n")
    except (OSError, UnicodeDecodeError) as error:
        table_lines = None
        if not os.path.exists(src_path):
            unreadable.append((stem, f"{type(error).__name__}: {error}"))
    if table_lines is not None:
        tables_read += 1
        if check_duplicates:
            duplicates += duplicate_groups(stem, table_lines)
    # 没有同名二进制的表（shell 探针的变异表）不在锚点的射程：它们的被测对象不是 .rs；成功行逐个列名
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
print("TABLES_READ", tables_read)
for d in duplicates:
    print("DUP", *d, sep="\t")
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
MUTATION_ROWS_MODULE="$STAGE_REPOSITORY/scripts/mutation/crates-mutation-rows.py"
crates_report=""
if [[ -f crates/mutations.tsv ]]; then
  crates_report="$(MUTATION_ROWS_MODULE="$MUTATION_ROWS_MODULE" python3 - <<'PY'
import glob
import importlib.util
import os
import re
import sys
import tomllib
sys.dont_write_bytecode = True
specification = importlib.util.spec_from_file_location("crates_mutation_rows", os.environ["MUTATION_ROWS_MODULE"])
mutation_rows = importlib.util.module_from_spec(specification)
specification.loader.exec_module(mutation_rows)
first_line_of_change = {}
first_line_of_name = {}
attributes_by_package = {}
def module_path_of_file(package_directory, path):
    """包目录的 src 下一个 .rs 在 libtest 测试名里的模块前缀：src/lib.rs 与 src/bin/<名>.rs 是空，src/a.rs 与 src/a/mod.rs 是 a，src/a/b.rs 是 a::b；tests/ 下的是空。"""
    relative = os.path.relpath(path, package_directory)
    parts = relative.split(os.sep)
    if parts[0] != "src" or parts[1:2] == ["bin"] or parts[1:] == ["lib.rs"]:
        return []
    parts = parts[1:]
    parts[-1] = parts[-1][:-len(".rs")]
    if parts[-1] == "mod":
        parts = parts[:-1]
    return parts
# 配花括号之前先抹掉的几样：行注释、块注释（可嵌套，找到开头再往后数）、原始串（找到开头再找收尾）、普通串与字符字面量
MASKED_START = re.compile(r"""//[^\n]*|/\*|(?<![A-Za-z0-9_])b?r(?P<hashes>#*)"|b?"(?:\\.|[^"\\])*(?:"|\Z)|b?'(?:\\(?:x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f]{1,6}\}|[^\n])|[^\\'\n])'""", re.S)
def masked_comments_and_literals(text):
    """注释、字符串、原始串与字符字面量抹成同样长度的空白（换行留着），偏移不变：配花括号与找 `mod 名 {` 只看代码。"""
    pieces, position = [], 0
    while True:
        found = MASKED_START.search(text, position)
        if not found:
            return "".join(pieces) + text[position:]
        end = found.end()
        if found.group(0) == "/*":
            depth = 1
            while end < len(text) and depth:
                depth += 1 if text.startswith("/*", end) else -1 if text.startswith("*/", end) else 0
                end += 2 if text.startswith(("/*", "*/"), end) else 1
        elif found.group("hashes") is not None:
            closing = '"' + found.group("hashes")
            close_at = text.find(closing, end)
            end = len(text) if close_at < 0 else close_at + len(closing)
        pieces += [text[position:found.start()], re.sub(r"[^\n]", " ", text[found.start():end])]
        position = end
def enclosing_inline_modules(text):
    """(偏移, 那一刻套着的内联 mod 名单) 的有序表：按 `mod 名 {` 与花括号配平；注释、字符串、原始串与字符字面量先抹掉再配
    （弄坏开关 GATE_MUTATION_TABLES_BREAK=inline-module-literals-counted 下直接扫原文，串里的 `mod tests {` 与落单花括号照配）。"""
    events, stack, depth = [], [], 0
    if "inline-module-literals-counted" not in os.environ.get("GATE_MUTATION_TABLES_BREAK", ""):
        text = masked_comments_and_literals(text)
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
comment_lines_as_attributes = "comment-lines-as-attributes" in os.environ.get("GATE_MUTATION_TABLES_BREAK", "")
def attribute_code(lines_above):
    """fn 上面那一串（属性行与夹在中间的注释行）里只留属性行，每行去掉行尾 `//` 起的注释：注释行（`//`、`///`）与行尾注释里写的 `#[ignore]` 不算标了 ignore
    （弄坏开关 GATE_MUTATION_TABLES_BREAK=comment-lines-as-attributes 下整串原样交回，注释行里的 `#[ignore]` 也算）。"""
    if comment_lines_as_attributes:
        return lines_above
    return "\n".join(line.split("//", 1)[0] for line in lines_above.split("\n") if line.lstrip().startswith("#["))
def test_attributes_of_package(package_directory):
    """包目录的 tests/ 与 src/ 下每个 fn 名 → [(它上面那串属性行（注释去掉，见 attribute_code）, libtest 里的完整测试名)]；每个包目录只扫一次。"""
    if package_directory not in attributes_by_package:
        table = {}
        for directory_name in ("tests", "src"):
            for root, _directories, files in os.walk(os.path.join(package_directory, directory_name)):
                for file_name in sorted(files):
                    if not file_name.endswith(".rs"):
                        continue
                    path = os.path.join(root, file_name)
                    try:
                        text = open(path, encoding="utf-8").read()
                    except (OSError, UnicodeDecodeError):
                        continue
                    prefix = module_path_of_file(package_directory, path)
                    events = enclosing_inline_modules(text)
                    for found in re.finditer(r"((?:[ \t]*#\[[^\n]*\n|[ \t]*//[^\n]*\n)*)[ \t]*(?:pub\s+)?fn\s+(\w+)\s*\(", text):
                        modules = []
                        for offset, stack in events:
                            if offset > found.start(2):
                                break
                            modules = stack
                        table.setdefault(found.group(2), []).append((attribute_code(found.group(1)), "::".join(prefix + modules + [found.group(2)])))
        attributes_by_package[package_directory] = table
    return attributes_by_package[package_directory]
# 点名的测试在选的目标里找不找得到：切词用 lib_function_units.py 的 rust_lex（与单元切法同一份词法，不另写一份去注释、去字符串）
named_test_break = os.environ.get("GATE_MUTATION_TABLES_BREAK", "")
function_units_specification = importlib.util.spec_from_file_location(
    "lib_function_units", os.path.join(os.path.dirname(os.environ["MUTATION_ROWS_MODULE"]), "..", "gate", "lib_function_units.py"))
function_units = importlib.util.module_from_spec(function_units_specification)
function_units_specification.loader.exec_module(function_units)
words_by_path = {}
def words_of(path):
    """一份 .rs 里的标识符集合与紧跟在 fn 后面的标识符集合（注释、字符串、字符字面量里的不算）；读不了的两样都空。"""
    if path not in words_by_path:
        try:
            kinds, texts = function_units.rust_lex(open(path, encoding="utf-8").read())[:2]
        except (OSError, UnicodeDecodeError):
            kinds, texts = [], []
        identifiers = {text for kind, text in zip(kinds, texts) if kind == function_units.IDENT}
        defined = {texts[index + 1] for index in range(len(texts) - 1)
                   if kinds[index] == function_units.IDENT and texts[index] == "fn" and kinds[index + 1] == function_units.IDENT}
        modules = [texts[index + 1] for index in range(len(texts) - 2)
                   if texts[index] == "mod" and kinds[index + 1] == function_units.IDENT and texts[index + 2] == ";"]
        words_by_path[path] = (identifiers, defined, modules)
    return words_by_path[path]
def rust_files_under(directory):
    found = []
    for root, _directories, files in os.walk(directory):
        found += [os.path.join(root, name) for name in files if name.endswith(".rs")]
    return sorted(found)
def selected_target_files(package_directory, arguments):
    """cargo test 参数在 `--` 之前选的编译目标的源文件与一句说明。--lib、--bin、--test 认；都没写或别的选法按 src/ 与 tests/ 全部。"""
    base = package_directory
    before_separator = arguments[:arguments.index("--")] if "--" in arguments else arguments
    selections = []
    for position, word in enumerate(before_separator):
        if word == "--lib":
            selections.append(("lib", None))
        elif word in ("--bin", "--test") and position + 1 < len(before_separator):
            selections.append((word[2:], before_separator[position + 1]))
        elif word.startswith(("--bin=", "--test=")):
            selections.append(tuple(word[2:].split("=", 1)))
        elif word in ("--tests", "--bins", "--all-targets", "--examples", "--benches"):
            selections.append(("all", None))
    if not selections or ("all", None) in selections or "named-test-target-ignored" in named_test_break:
        return rust_files_under(os.path.join(base, "src")) + rust_files_under(os.path.join(base, "tests")), f"{package_directory} 的 src/ 与 tests/"
    files, shown = [], []
    for kind, value in selections:
        if kind == "lib":
            files += [path for path in rust_files_under(os.path.join(base, "src"))
                      if not path.startswith(os.path.join(base, "src", "bin") + os.sep) and path != os.path.join(base, "src", "main.rs")]
            shown.append("--lib")
        elif kind == "bin":
            files += [os.path.join(base, "src", "bin", value + ".rs")] + rust_files_under(os.path.join(base, "src", "bin", value))
            shown.append(f"--bin {value}")
        else:
            root_file = os.path.join(base, "tests", value + ".rs")
            files += [root_file] + rust_files_under(os.path.join(base, "tests", value))
            for module in words_of(root_file)[2]:
                files += [os.path.join(base, "tests", module + ".rs")] + rust_files_under(os.path.join(base, "tests", module))
            shown.append(f"--test {value}")
    return [path for path in dict.fromkeys(files) if os.path.isfile(path)], f"{package_directory} 的 {'、'.join(shown)}"
workspace_members_cache = []
def workspace_members():
    """仓根 Cargo.toml 的 [workspace] members 现读：{包名: 目录（相对仓根）}。members 里的通配照 glob 展开，exclude 里的目录不算，
    包名取各成员目录的 Cargo.toml 的 [package] name（目录名与包名不必相同）。仓根或成员的 Cargo.toml 读不了就抛，这一段 python 崩了，整格按「没跑完」判红。"""
    if not workspace_members_cache:
        with open("Cargo.toml", "rb") as handle:
            workspace = tomllib.load(handle).get("workspace", {})
        excluded = {os.path.normpath(path) for path in workspace.get("exclude", [])}
        members = {}
        for pattern in workspace.get("members", []):
            for directory in sorted(glob.glob(pattern)):
                directory = os.path.normpath(directory)
                if directory in excluded or not os.path.isdir(directory):
                    continue
                with open(os.path.join(directory, "Cargo.toml"), "rb") as handle:
                    package_name = tomllib.load(handle).get("package", {}).get("name")
                if package_name is not None:
                    members.setdefault(package_name, directory)
        workspace_members_cache.append(members)
    return workspace_members_cache[0]
repository_root = os.path.realpath(".")
def manifest_package_directory(path):
    """`--manifest-path <路径>` 指的那份 Cargo.toml 所在的目录（相对仓根）与问题句，两样里恰有一样是 None：路径从仓根算起（cargo 在仓根起），
    解到仓根之外、文件名不是 Cargo.toml、文件不在的回问题句。
    弄坏开关 GATE_MUTATION_TABLES_BREAK 的 manifest-path-outside-repo-unjudged、manifest-path-file-name-unjudged、manifest-path-missing-file-unjudged 各不判其中一样；
    manifest-path-not-a-package 三样都判，但判完不把那个目录当包目录用。"""
    target = os.path.realpath(os.path.join(repository_root, path))
    if not (target == repository_root or target.startswith(repository_root + os.sep)) and "manifest-path-outside-repo-unjudged" not in named_test_break:
        return None, f"--manifest-path {path} 解到仓根之外（{target}）；写仓内独立工程的 Cargo.toml（相对仓根的路径）"
    if os.path.basename(target) != "Cargo.toml" and "manifest-path-file-name-unjudged" not in named_test_break:
        return None, f"--manifest-path {path} 的文件名不是 Cargo.toml；写成指到那个独立工程的 Cargo.toml"
    if not os.path.isfile(target) and "manifest-path-missing-file-unjudged" not in named_test_break:
        return None, f"--manifest-path {path} 指的文件不在（仓根下 {os.path.relpath(target, repository_root)}）；写成那个独立工程的 Cargo.toml 的真实路径"
    if "manifest-path-not-a-package" in named_test_break:
        return None, None
    return os.path.relpath(os.path.dirname(target), repository_root), None
def packages_of(arguments):
    """cargo test 参数在 `--` 之前点名的包 → (包目录列表, 问题句列表, 经 -p 解出的目录, 经 --manifest-path 解出的目录)，目录照写的次序、去重、相对仓根：
      -p <包>、--package <包>、-p=<包>、--package=<包>   只认仓根 Cargo.toml 的 [workspace] members 里的包（workspace_members 现读，目录名与包名不必相同）；
                                                       不是成员的（被 exclude 掉的独立工程、哪里都没有的名字）判问题，出路是 --manifest-path
      --manifest-path <路径>、--manifest-path=<路径>    取那份 Cargo.toml 所在的目录当包目录（manifest_package_directory），独立工程用它点名
    同一行既带 -p 又带 --manifest-path、带两次 --manifest-path、--manifest-path 后面没写路径的判问题；有问题的写法不再往下解目录。
    弄坏开关 GATE_MUTATION_TABLES_BREAK：
      one-package-per-row  只认整串参数里最靠前的那一个 `-p <包>`
      package-directory-from-name  包目录不读仓根 Cargo.toml、按 crates/<包名> 硬拼（-p 点到非成员也照拼）
      non-member-package-unjudged  成员照读，-p 点到非成员的不判问题，退回按 crates/<包名> 硬拼
      package-and-manifest-path-together-allowed、manifest-path-twice-allowed、manifest-path-without-value-unjudged  各不判其中一样"""
    before_separator = arguments[:arguments.index("--")] if "--" in arguments else arguments
    names, manifests = [], []
    for position, word in enumerate(before_separator):
        following = before_separator[position + 1] if position + 1 < len(before_separator) else None
        if word in ("-p", "--package"):
            if following is not None:
                names.append(following)
        elif word.startswith(("-p=", "--package=")):
            names.append(word.split("=", 1)[1])
        elif word == "--manifest-path":
            manifests.append(following)
        elif word.startswith("--manifest-path="):
            manifests.append(word.split("=", 1)[1])
    names = list(dict.fromkeys(names))
    if "one-package-per-row" in named_test_break:
        names = [arguments[arguments.index("-p") + 1]] if "-p" in arguments[:-1] else []
    manifest_words = [f"--manifest-path {path}" if path is not None else "--manifest-path" for path in manifests]
    if names and manifests and "package-and-manifest-path-together-allowed" not in named_test_break:
        return [], [f"同一行既写了 {' '.join('-p ' + name for name in names)} 又写了 {' '.join(manifest_words)}；"
                    "二选一：工作区成员写 -p <包名>，不在工作区的独立工程写 --manifest-path <它的 Cargo.toml>"], [], []
    if len(manifests) > 1 and "manifest-path-twice-allowed" not in named_test_break:
        return [], [f"--manifest-path 写了不止一次（{'、'.join(path or '（没写路径）' for path in manifests)}）；一行只许一个，要测两个独立工程就拆成两行"], [], []
    problems, from_names, from_manifests = [], [], []
    for path in manifests:
        if path is None:
            if "manifest-path-without-value-unjudged" not in named_test_break:
                problems.append("--manifest-path 后面没写路径；写成 --manifest-path <那个独立工程的 Cargo.toml>")
            continue
        directory, problem = manifest_package_directory(path)
        if problem is not None:
            problems.append(problem)
        elif directory is not None:
            from_manifests.append(directory)
    for name in names:
        if "package-directory-from-name" in named_test_break:
            from_names.append(os.path.join("crates", name))
            continue
        members = workspace_members()
        if name in members:
            from_names.append(members[name])
        elif "non-member-package-unjudged" in named_test_break:
            from_names.append(os.path.join("crates", name))
        else:
            problems.append(f"-p {name} 不是工作区成员（仓根 Cargo.toml 的 [workspace] members 里没有叫这个名字的包；成员有：{'、'.join(sorted(members))}）；"
                            "不在工作区的独立工程用 --manifest-path <它的 Cargo.toml>")
    return list(dict.fromkeys(from_names + from_manifests)), problems, from_names, from_manifests
rows, _malformed = mutation_rows.read_table("crates/mutations.tsv")
# 弄坏开关 duplicate-key-with-test：把点名的测试也算进重复的键（同一处改法点名两条不同测试的几行就放过去了）
duplicate_key_with_test = "duplicate-key-with-test" in os.environ.get("GATE_MUTATION_TABLES_BREAK", "")
rows_with_member_packages = rows_with_manifest_paths = 0
for row in rows:
    line_number, fields = row.line_number, row.fields
    name, path = row.name, row.path
    # 几行的（文件, 原文, 替换文）逐字相同：同一处改法写了几行，表里的条数看着多、真改法没多；一处改法留一行
    change = (path, fields[2], fields[3]) + ((fields[5],) if duplicate_key_with_test else ())
    first_line_of_change.setdefault(change, []).append((line_number, name, fields[4]))
    # 变异名相同 ⇒ 按名字分格的报告把两条并成一格，其中一条的结论看不见。
    if name in first_line_of_name:
        print("CRATES_DUP", line_number, name, f"变异名与第 {first_line_of_name[name]} 行相同", sep="\t")
    else:
        first_line_of_name[name] = line_number
    # 点名的测试标了 #[ignore]（只留下的不是分档的那几条：要显卡、量显存与速度、要补丁表副本），cargo test 参数里却没有 --include-ignored / --ignored：
    # crates-mutation-replay 与 prove-red.sh 照这一行跑，点名的测试被 libtest 跳过，这条变异永远报「没红」，而它守的那一格其实没人看。
    arguments = fields[4].split()
    # 一行写了几个 -p：点名的测试落在其中任一个包里就算在（筛选词、点名的测试在不在目标里、#[ignore] 三样都按这几个包的并集判）；
    # 包目录：-p 点名的取仓根 Cargo.toml 的工作区成员，--manifest-path 点名的取那份 Cargo.toml 所在的目录；认不出的整行报 CRATES_PACKAGE，那一行不再往下判
    package_directories, package_problems, package_directories_from_names, package_directories_from_manifests = packages_of(arguments)
    for problem in package_problems:
        print("CRATES_PACKAGE", line_number, name, problem, sep="\t")
    rows_with_member_packages += bool(package_directories_from_names)
    rows_with_manifest_paths += bool(package_directories_from_manifests)
    test_name = fields[5].split("::")[-1].strip()
    # cargo test 参数里 `--` 之后的位置参数是 libtest 的筛选词：点名的测试一个都筛不到，那条测试根本不跑，变异永远报没红。
    libtest_arguments = arguments[arguments.index("--") + 1:] if "--" in arguments else []
    filters = [word for word in libtest_arguments if not word.startswith("-")]
    exact = "--exact" in libtest_arguments
    # libtest 拿筛选词去比完整测试名（模块路径 + 函数名）：完整名从点名的函数在哪个文件、套在哪几层内联 mod 里算出来；找不到那个函数的不判。
    found_tests = [found for package_directory in package_directories for found in test_attributes_of_package(package_directory).get(test_name, [])] if test_name else []
    candidates = [full_name for _attributes, full_name in found_tests]
    if candidates and filters and not any((full_name == word) if exact else (word in full_name) for word in filters for full_name in candidates):
        print("CRATES_FILTER", line_number, name, f"筛选词 {' '.join(filters)}{'（--exact）' if exact else ''} 筛不到点名的 {' 或 '.join(candidates)}", sep="\t")
    # 点名的测试在选的目标里找不到（改了名、删了）：那一道照这一行跑，点名的测试不在输出里，变异永远报没红
    if package_directories and test_name and "named-test-unsearched" not in named_test_break:
        target_files, target_shown = [], []
        for package_directory in package_directories:
            package_files, package_shown = selected_target_files(package_directory, arguments)
            target_files += package_files
            target_shown.append(package_shown)
        look_in = 1 if "named-test-definitions-only" in named_test_break else 0
        if not any(test_name in words_of(path)[look_in] for path in target_files):
            print("CRATES_MISSING", line_number, name, f"点名的测试 {test_name} 在 {'、'.join(target_shown)}（{len(target_files)} 份 .rs）里找不到", sep="\t")
    if package_directories and test_name and not ({"--include-ignored", "--ignored"} & set(arguments)):
        if found_tests and all("#[ignore" in attributes for attributes, _full_name in found_tests):
            print("CRATES_IGNORED", line_number, name, f"点名的 {test_name} 标了 #[ignore]，cargo test 参数「{fields[4]}」里没有 --include-ignored", sep="\t")
# 重复的改法逐组一行：「第 a、b 行（a 名；b 名）」，组里 cargo 参数也全同的另注一句
for (path, *_rest), group in first_line_of_change.items():
    if len(group) > 1:
        same_arguments = "；这几行的 cargo test 参数也全同" if len({arguments for _line, _name, arguments in group}) == 1 else ""
        print("CRATES_DUP_CHANGE", "第 " + "、".join(str(line) for line, _name, _arguments in group) + " 行（"
              + "；".join(f"{line} {name}" for line, name, _arguments in group) + f"），改的是 {path}{same_arguments}",
              "\x1f".join(name for _line, name, _arguments in group), sep="\t")
print("CRATES_PACKAGES_CHECKED", rows_with_member_packages, rows_with_manifest_paths, sep="\t")
print("CRATES_CHECKED", len(rows))
PY
)"
  crates_exit_code=$?
fi
# ── 门禁 crates-mutation-replay 派活之前判的那几样与拷贝范围：共用模块的 static-check 逐处打
# MALFORMED / ANCHOR / NAME / ROW / STRAY / LITERAL / UNDEFINED / RELEASE，末尾 COPIED、IDENTIFIERS_CHECKED 与 CHECKED；退 0 一处都没有、退 1 有、别的退出码是它自己没判完（ERROR 行是原因）
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
# 替换文引了 crates 里找不到定义的标识符（文件头这一格「替换文引的标识符」）；弄坏开关 undefined-identifiers-ignored：不读这种行
crates_undefined=()
if [[ "${GATE_MUTATION_TABLES_BREAK:-}" != *undefined-identifiers-ignored* ]]; then
  mapfile -t crates_undefined < <(grep $'^UNDEFINED\t' <<<"$scope_report")
fi
# 命中重测试目标而第五段没写 --release（文件头这一格「重测试目标只许 release」）；弄坏开关 release-lines-unread：不读这种行，成功句里也不打 release 那半句
crates_release=(); release_lines_read=""
if [[ "${GATE_MUTATION_TABLES_BREAK:-}" != *release-lines-unread* ]]; then
  mapfile -t crates_release < <(grep $'^RELEASE\t' <<<"$scope_report")
  release_lines_read=1
fi
scope_identifiers="$(sed -n 's/^IDENTIFIERS_CHECKED\t//p' <<<"$scope_report")"
# 点名的测试够不着被改坏的函数（文件头这一格「点名测试够不着被改坏的函数」）：static-check 在 scope_report 里一律判，UNREACHABLE 与 REACHABILITY_UNJUDGED 进红句，
# REACHABILITY_CHECKED 与 REACHABILITY_NOT_JUDGED 进成功句。弄坏开关 reachability-lines-unread 退回不读这几行：UNREACHABLE 与 REACHABILITY_UNJUDGED 不进红句、成功句里的可达性那半句也不打
scope_unreachable=(); scope_reachability_unjudged=(); scope_reachability_checked=""; scope_reachability_not_judged=""
if [[ "${GATE_MUTATION_TABLES_BREAK:-}" != *reachability-lines-unread* ]]; then
  mapfile -t scope_unreachable < <(grep $'^UNREACHABLE\t' <<<"$scope_report")
  mapfile -t scope_reachability_unjudged < <(grep $'^REACHABILITY_UNJUDGED\t' <<<"$scope_report")
  scope_reachability_checked="$(sed -n 's/^REACHABILITY_CHECKED\t//p' <<<"$scope_report")"
  scope_reachability_not_judged="$(sed -n 's/^REACHABILITY_NOT_JUDGED\t//p' <<<"$scope_report")"
fi
mapfile -t scope_errors < <(sed -n 's/^ERROR\t//p' <<<"$scope_report")
IFS=$'\t' read -r scope_row_count scope_source_count scope_literal_count <<<"$scope_checked"
crates_checked="$(sed -n 's/^CRATES_CHECKED //p' <<<"$crates_report")"
mapfile -t crates_dup < <(grep $'^CRATES_DUP\t' <<<"$crates_report")
mapfile -t crates_duplicate_groups < <(grep $'^CRATES_DUP_CHANGE\t' <<<"$crates_report")
mapfile -t crates_ignored < <(grep '^CRATES_IGNORED' <<<"$crates_report")
mapfile -t crates_filter < <(grep '^CRATES_FILTER' <<<"$crates_report")
mapfile -t crates_missing_tests < <(grep '^CRATES_MISSING' <<<"$crates_report")
mapfile -t crates_packages < <(grep $'^CRATES_PACKAGE\t' <<<"$crates_report")
IFS=$'\t' read -r rows_with_member_packages rows_with_manifest_paths < <(sed -n 's/^CRATES_PACKAGES_CHECKED\t//p' <<<"$crates_report")
anchor_checked="$(sed -n 's/^CHECKED //p' <<<"$anchor_report")"
tables_read="$(sed -n 's/^TABLES_READ //p' <<<"$anchor_report")"
mapfile -t research_duplicate_groups < <(grep $'^DUP\t' <<<"$anchor_report")
mapfile -t anchor_bad < <(grep '^BAD' <<<"$anchor_report")
mapfile -t stray_rows < <(grep '^STRAY' <<<"$anchor_report")
mapfile -t anchor_unreadable < <(grep '^UNREADABLE' <<<"$anchor_report")
mapfile -t tables_without_binary < <(sed -n 's/^NO_BINARY\t//p' <<<"$anchor_report")
# ── 重复行的滞后表：登记的组不判红，表只缩不涨（判法写在这一格的文件头注释「滞后表」那几行）──
DUPLICATES_LAG=.claude/gate.d/mutation-duplicates-lag.tsv
lag_base=""
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  LIB_CHANGED_PATHS="$STAGE_REPOSITORY/scripts/gate/changed-paths.sh"
  # shellcheck source=../../scripts/gate/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
  lag_base="$(gate_diff_base gate)"
fi
duplicates_lag_judge="$(cat <<'PY'
import importlib.util, os, sys
lag_path, mutation_directory, base = sys.argv[1], sys.argv[2], sys.argv[3]
lag_baseline_spec = importlib.util.spec_from_file_location("lag_baseline", sys.argv[4])
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)
broken = os.environ.get("GATE_MUTATION_TABLES_BREAK", "")
CRATES_TABLE = "crates/mutations.tsv"
groups = []   # (表, 原样输出的那一行去掉名字那一段, 组内名字的列表)
for line in sys.stdin.read().split("\n"):
    fields = line.split("\t")
    if fields[0] == "CRATES_DUP_CHANGE" and len(fields) == 3:
        groups.append((CRATES_TABLE, "\t".join(fields[:2]), fields[2].split("\x1f")))
    elif fields[0] == "DUP" and len(fields) == 4:
        groups.append((f"{mutation_directory}/{fields[1]}.tsv", "\t".join(fields[:3]), fields[3].split("\x1f")))
def names_of_table(path):
    """那张表第一列的全部名字（注释行与空行不算）；读不了返回 None。"""
    try:
        text = open(path, encoding="utf-8").read()
    except (OSError, UnicodeDecodeError):
        return None
    return {line.split("\t")[0] for line in text.split("\n") if line.strip() and not line.startswith("#")}
def parse_rows(text, label):
    """滞后表的行：[(行号, 表, 名字列表)]，形状不对的进 problems。切行经 lib-lag-baseline.py 的 content_rows（注释与空行不算）。"""
    rows, problems = [], []
    for line_number, fields in lag_baseline.content_rows(text):
        if len(fields) != 4 or not all(field.strip() for field in fields):
            problems.append(f"{label}:{line_number} 不是四列、或有空格子（表、组内名字、为什么滞后、归哪一次处理）")
            continue
        rows.append((line_number, fields[0].strip(), fields[1].split("|")))
    return rows, problems
problems = []
rows = []
malformed_count = 0
if "duplicates-lag-ignored" not in broken and os.path.exists(lag_path):
    try:
        rows, problems = parse_rows(open(lag_path, encoding="utf-8").read(), lag_path)
        malformed_count = len(problems)
    except (OSError, UnicodeDecodeError) as error:
        problems.append(f"{lag_path} 读不了（{type(error).__name__}: {error}）")
covered = {}   # 组的下标 → 罩住它的那一行的行号
lag_lines = []
table_names = {}
for line_number, table, names in rows:
    where = f"{lag_path}:{line_number}"
    if table != CRATES_TABLE and not (table.startswith(mutation_directory + "/") and table.endswith(".tsv")):
        problems.append(f"{where} 的表「{table}」不是 {CRATES_TABLE}，也不是 {mutation_directory}/<表名>.tsv")
        continue
    if table not in table_names:
        table_names[table] = names_of_table(table)
    if table_names[table] is None:
        problems.append(f"{where} 指的表 {table} 不在或读不了")
        continue
    if len(set(names)) != len(names):
        problems.append(f"{where} 里同一个名字写了不止一次")
        continue
    missing = [name for name in names if name not in table_names[table]]
    if missing:
        problems.append(f"{where} 的这几个名字在 {table} 里找不到：" + "；".join(missing))
        continue
    exact = [index for index, (group_table, _shown, group_names) in enumerate(groups) if group_table == table and set(group_names) == set(names)]
    if len(exact) > 1:
        problems.append(f"{where} 的名字在 {table} 里对上 {len(exact)} 组（名字在表里不唯一），一行只罩一组")
        continue
    if not exact:
        holding = [group_names for group_table, _shown, group_names in groups if group_table == table and set(names) <= set(group_names)]
        if holding:
            problems.append(f"{where} 那一组里还有没登记的名字：" + "；".join(sorted(set(holding[0]) - set(names))))
        else:
            problems.append(f"{where} 的名字此刻不是同一组重复行（" + ("三样不全同" if table == CRATES_TABLE else "除变异名外各列不全同") + "）：" + "；".join(names))
        continue
    if exact[0] in covered:
        problems.append(f"{where} 与第 {covered[exact[0]]} 行罩的是同一组")
        continue
    covered[exact[0]] = line_number
    lag_lines.append(f"{table}\t" + "；".join(names))
# 只缩不涨：此刻的每一行，基准那一版同一张表里要有一行的名字是它的超集；基准里还没有这张表不比，git 取不到基准那一版判红
baseline_note = "不在 git 仓里，没和基准比"
if base and rows and "duplicates-lag-growth-unchecked" not in broken:
    # 取基准那一版：lib-lag-baseline.py 的 baseline_text，「基准里没有这张表」与「git 取不到」分开，后者判红
    shown, baseline_problem = lag_baseline.baseline_text(base, lag_path)
    if baseline_problem is not None:
        problems.append(f"{baseline_problem.sentence(200)}，只缩不涨没比")
    elif shown is None:
        baseline_note = f"基准 {base[:12]} 里还没有这张表，没比只缩不涨"
    else:
        baseline_rows, _baseline_problems = parse_rows(shown, f"基准里的 {lag_path}")
        grown = [(line_number, table, names) for line_number, table, names in rows
                 if not any(base_table == table and set(names) <= set(base_names) for _n, base_table, base_names in baseline_rows)]
        for line_number, table, names in grown:
            problems.append(f"{lag_path}:{line_number} 相对基准那一版多出一组（{table}：" + "；".join(names) + "）：这张表只缩不涨")
        baseline_note = f"与基准 {base[:12]} 那一版按组比，" + (f"多出 {len(grown)} 组" if grown else "没涨")
elif base and "duplicates-lag-growth-unchecked" in broken:
    baseline_note = "没和基准比"
elif base:
    baseline_note = "表里没有行，不用和基准比"
for index, (_table, shown, _names) in enumerate(groups):
    if index not in covered:
        print(shown)
for line in lag_lines:
    print("LAG", line, sep="\t")
for problem in problems:
    print("LAG_PROBLEM", problem, sep="\t")
print("LAG_BASELINE", baseline_note, sep="\t")
print("LAG_ROWS", len(rows) + malformed_count, sep="\t")
PY
)"
lag_report="$( { printf '%s\n' ${crates_duplicate_groups[@]+"${crates_duplicate_groups[@]}"} ${research_duplicate_groups[@]+"${research_duplicate_groups[@]}"}; } \
  | python3 -c "$duplicates_lag_judge" "$DUPLICATES_LAG" "$MUT_DIR" "$lag_base" "$LAG_BASELINE_LIBRARY")"
lag_exit_code=$?
mapfile -t crates_dup_changes < <(sed -n 's/^CRATES_DUP_CHANGE\t//p' <<<"$lag_report")
mapfile -t research_duplicates < <(grep $'^DUP\t' <<<"$lag_report")
mapfile -t lag_groups < <(sed -n 's/^LAG\t//p' <<<"$lag_report")
mapfile -t lag_problems < <(sed -n 's/^LAG_PROBLEM\t//p' <<<"$lag_report")
lag_baseline="$(sed -n 's/^LAG_BASELINE\t//p' <<<"$lag_report")"
lag_rows="$(sed -n 's/^LAG_ROWS\t//p' <<<"$lag_report")"
# 两段 python 自己崩了（退出码非 0、或没报出 CHECKED 那一行）：它们没判完，不许读成「没有 BAD 行」
script_failures=()
if [[ "${anchor_exit_code:-1}" -ne 0 || -z "$anchor_checked" ]]; then
  script_failures+=("research 变异表的锚点检查没跑完（python 退出码 ${anchor_exit_code:-?}，CHECKED 行「${anchor_checked}」）")
fi
if [[ -f crates/mutations.tsv ]] && [[ "${crates_exit_code:-1}" -ne 0 || -z "$crates_checked" ]]; then
  script_failures+=("crates/mutations.tsv 的重复、#[ignore]、筛选词、点名的测试在不在目标里这几样检查没跑完（python 退出码 ${crates_exit_code:-?}，CRATES_CHECKED 行「${crates_checked}」）")
fi
if [[ "${lag_exit_code:-1}" -ne 0 || -z "$lag_rows" ]]; then
  script_failures+=("重复行滞后表 $DUPLICATES_LAG 的判法没跑完（python 退出码 ${lag_exit_code:-?}，LAG_ROWS 行「${lag_rows}」），重复行哪几组算滞后没判")
fi
if [[ "$scope_exit_code" -ne 0 && "$scope_exit_code" -ne 1 ]] || [[ -z "$scope_checked" ]]; then
  script_failures+=("门禁 crates-mutation-replay 派活之前那几样与拷贝范围的检查没跑完（scripts/mutation/crates-mutation-rows.py static-check 退出码 ${scope_exit_code}，CHECKED 行「${scope_checked}」；${scope_errors[*]:-没报原因}）")
fi

if ((${#missing[@]} + ${#malformed[@]} + ${#anchor_bad[@]} + ${#crates_bad[@]} + ${#crates_dup[@]} + ${#crates_dup_changes[@]} + ${#research_duplicates[@]} + ${#lag_problems[@]} + ${#crates_ignored[@]} + ${#crates_filter[@]} + ${#crates_missing_tests[@]} + ${#stray_rows[@]} + ${#crates_stray[@]} + ${#crates_malformed[@]} + ${#crates_names[@]} + ${#anchor_unreadable[@]} + ${#script_failures[@]} + ${#scope_rows[@]} + ${#scope_literals[@]} + ${#crates_undefined[@]} + ${#crates_release[@]} + ${#crates_packages[@]} + ${#scope_unreachable[@]} + ${#scope_reachability_unjudged[@]})); then
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
    echo "  ✗ 这些变异条目的「原文」在源码里不是恰好命中一次（mutate.sh 把这几条记「⚓ 锚点失效」、不跑，别的照跑，整轮退 3）："
    while IFS=$'\t' read -r _ stem name lineno hits; do
      printf '      %s:%s %s：原文在 %s 里命中 %s 次\n' "$MUT_DIR/$stem.tsv" "$lineno" "$name" "$BIN_DIR/$stem.rs" "$hits"   # gate-lint:detail
    done < <(printf '%s\n' "${anchor_bad[@]}")
    echo "    命中 0 次：源码改过而表没跟；命中多次：原文要多带一行上下文才唯一。"
  fi
  if ((${#missing[@]} + ${#malformed[@]} + ${#anchor_bad[@]})); then
    echo "  → 怎么办：缺表的写 scripts/mutation/tables/<bin名>.tsv（每行：变异名<TAB>原文<TAB>替换文）；"
    echo "    锚点对不上的把「原文」改成今天源码里逐字存在、且只出现一次的那一段，"
    echo "    改完跑 bash scripts/mutation/mutate.sh <bin> <源文件> <表> 证明每条都被抓，再来。"
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
    echo "  ✗ crates/mutations.tsv 这些行的原文或替换文里有 \\n 以外的反斜杠转义（门禁 crates-mutation-replay 只还原 \\n）："
    while IFS=$'\t' read -r _ lineno name segment escape; do
      printf '      crates/mutations.tsv:%s %s：%s里的 %s\n' "$lineno" "$name" "$segment" "$escape"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_stray[@]}")
    echo "    → 怎么办：去掉那个反斜杠，改完单跑那一行证明点名的测试红（2026-09-20 第 180 行踩过：替换文写 \\&report.outcome，"
    echo "      写进源码是字面反斜杠、编不过，crates-mutation-replay 把它报成「点名的测试没跑到」，出路指向「先造一条会红的用例」，方向是反的）。"
  fi
  if ((${#crates_dup_changes[@]})); then
    echo "  ✗ crates/mutations.tsv 里有 ${#crates_dup_changes[@]} 组行的「文件、原文、替换文」三样逐字全同（同一处改法写了不止一行）："
    printf '      crates/mutations.tsv %s\n' "${crates_dup_changes[@]}"   # gate-lint:detail
    echo "    → 怎么办：一处改法留一行，多的删。"
  fi
  if ((${#crates_dup[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行的变异名与前面某一行相同（按名字分格的报告会把两条并成一格，其中一条的结论看不见）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_dup[@]}")
    echo "    → 怎么办：给后面那一行起一个说它改了什么的新名字（名字写它是什么、做什么，不加序号）。"
  fi
  if ((${#research_duplicates[@]})); then
    echo "  ✗ 这些变异表里有 ${#research_duplicates[@]} 组行除变异名外各列逐字全同（同一处改法写了不止一行）："
    while IFS=$'\t' read -r _ stem where; do
      printf '      %s %s\n' "$MUT_DIR/$stem.tsv" "$where"   # gate-lint:detail
    done < <(printf '%s\n' "${research_duplicates[@]}")
    echo "    → 怎么办：一处改法留一行，多的删。"
  fi
  if ((${#lag_problems[@]})); then
    echo "  ✗ 重复行滞后表 $DUPLICATES_LAG 有 ${#lag_problems[@]} 处不对（指到不存在的名字、名字此刻不是同一组、或相对基准那一版涨了）："
    printf '      %s\n' "${lag_problems[@]}"   # gate-lint:detail
    echo "    → 怎么办：名字照那张表第一列逐字写、一行写全一组的名字（用 | 隔）；那一组已经不重复了（删了多的、或改了写法）就删掉这一行；"
    echo "      这张表只缩不涨，新出的重复行不往里加，照上面重复行那一条一处改法留一行。"
  fi
  if ((${#crates_packages[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行 cargo test 参数里点名的包认不出来（crates-mutation-replay 照这一行跑 cargo test，包点错了它起不来，或点名的测试不在包里）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_packages[@]}")
    echo "    → 怎么办：工作区成员（仓根 Cargo.toml 的 [workspace] members）写 -p <包名>；被 exclude 掉的独立工程写 --manifest-path <它的 Cargo.toml>（相对仓根的路径，只许仓内）；"
    echo "      一行只用其中一种、--manifest-path 只写一次；改完单跑那一行证明点名的测试红（scripts/mutation/prove-red.sh）。"
  fi
  if ((${#crates_filter[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行 cargo test 参数里的筛选词筛不到点名的测试（crates-mutation-replay 跑它时那条测试不跑，变异永远报没红）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_filter[@]}")
    echo "    → 怎么办：把 \`--\` 后面的筛选词改成点名测试名的一段（测试改过名、拆过的多半是这个），或者删掉筛选词跑整个测试目标。"
  fi
  if ((${#crates_missing_tests[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行点名的测试在 cargo test 参数选的目标里找不到（crates-mutation-replay 跑它时输出里没有这条测试，变异永远报没红）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_missing_tests[@]}")
    echo "    → 怎么办：测试改了名就把最后一列改成新名字；测试挪了目标就改 cargo 参数里的 --lib / --test <名>；测试删了就换一条今天还在、改坏这一处会红的测试，"
    echo "      改完单跑那一行证明点名的测试红（scripts/mutation/prove-red.sh）。"
  fi
  if ((${#crates_ignored[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行点名的测试标了 #[ignore]，cargo test 参数里却没有 --include-ignored（crates-mutation-replay 跑它时那条测试被跳过，变异永远报没红）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_ignored[@]}")
    echo "    → 怎么办：在那一行 cargo test 参数的 \`--\` 后面加 --include-ignored（没有 \`--\` 就补 \`-- --include-ignored\`），改完单跑那一行证明点名的测试红。"
  fi
  if ((${#crates_malformed[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行不成形（六段制表符分隔：变异名、文件、原文、替换文、cargo test 参数、必须红的测试名；替换文可以空，那是删掉原文，其余五段不许空；门禁 crates-mutation-replay 派活之前同样判红，一条都不跑）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_malformed[@]}")
    echo "    → 怎么办：补上空着的那一段，制表符删补到恰好六段；原文 / 替换文里的换行写成 \\n。要删掉原文就让替换文空着（两个制表符挨着）。"
  fi
  if ((${#crates_names[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行的「必须红的测试名」不是一个用例名（带了正则符号或空白，门禁 crates-mutation-replay 按字面在测试输出里永远找不到，派活之前同样判红）："
    while IFS=$'\t' read -r _ lineno name expected; do
      printf '      crates/mutations.tsv:%s %s：「%s」\n' "$lineno" "$name" "$expected"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_names[@]}")
    echo "    → 怎么办：那一列写成用例名本身（或带模块路径 tests::<名字>），去掉 \$、^ 这类符号；crates-mutation-replay 已经按「在输出里只许对上一个用例」判，不用再靠 \$ 锚定。"
  fi
  if ((${#crates_bad[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行的「原文」在源码里不是恰好命中一次（门禁 crates-mutation-replay 把这几行记「锚点失效」、不跑，别的行照判，整道判红）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_bad[@]}")
    echo "    → crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \\n），改完单跑那几行证明点名的测试红。"
  fi
  if ((${#scope_rows[@]})); then
    echo "  ✗ crates/mutations.tsv 这些行改的文件不在门禁 crates-mutation-replay 每片拷的范围里（crates-mutation-replay 派活之前就整道判红，一条都不跑）："
    while IFS=$'\t' read -r _ lineno name path; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$path"   # gate-lint:detail
    done < <(printf '%s\n' "${scope_rows[@]}")
    echo "    → 怎么办：把那个路径（或它所在的目录）登记进 .claude/gate.d/stage-inputs.tsv 里 crates-mutation-replay.sh 那一行（拷贝范围就是那一行登记的路径，现在是：${scope_copied}），"
    echo "      或者把这条变异改到拷贝范围之内的文件上。"
  fi
  if ((${#scope_literals[@]})); then
    echo "  ✗ crates/ 下这些 .rs 里有以 \"../ 起头、按所在 crate 的根解析之后逃出 crates/、又不在门禁 crates-mutation-replay 拷贝范围里的字符串字面量（crates-mutation-replay 的副本里没有那个路径，运行时读它的测试没改坏就红，crates-mutation-replay 记「基线不绿」）："
    while IFS=$'\t' read -r _ file lineno literal resolved; do
      printf '      %s:%s %s → %s\n' "$file" "$lineno" "$literal" "$resolved"   # gate-lint:detail
    done < <(printf '%s\n' "${scope_literals[@]}")
    echo "    → 怎么办：把那个路径所在的目录登记进 .claude/gate.d/stage-inputs.tsv 里 crates-mutation-replay 那一行（它同时是 crates-mutation-replay 的复用判定输入，登记之后 crates-mutation-replay 整张重跑一次），"
    echo "      或者让测试读 crates/ 之内的文件；#[path = …] 指的源文件不算，不用管。"
  fi
  if ((${#crates_undefined[@]})); then
    echo "  ✗ crates/mutations.tsv 这 ${#crates_undefined[@]} 行的替换文引了 crates 里找不到定义的标识符（被改文件与它所在的包的 .rs 里一处都没有；套上去编不过，门禁 crates-mutation-replay 记无效变异、整道判红）："
    while IFS=$'\t' read -r _ lineno name path identifiers; do
      printf '      crates/mutations.tsv:%s %s（改 %s）：%s\n' "$lineno" "$name" "$path" "$identifiers"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_undefined[@]}")
    echo "    → 怎么办：替换文引了已删或改名的标识符：改那一行的替换文（换成今天源码里还在的名字，或照今天的代码重写这一处改法），"
    echo "      改完单跑 python3 scripts/mutation/crates-mutation-rows.py static-check . 看那一行不再报 UNDEFINED，再单跑那一行证明点名的测试红（scripts/mutation/prove-red.sh）。"
  fi
  if ((${#crates_release[@]})); then
    echo "  ✗ crates/mutations.tsv 这 ${#crates_release[@]} 行命中重测试目标（gpu 特性、登记的崩溃枚举用例目标）而第五段没写 --release（门禁 crates-mutation-replay 与证红照补 --release 起，表上写的参数与真跑的不一致）："
    while IFS=$'\t' read -r _ lineno name basis; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$basis"   # gate-lint:detail
    done < <(printf '%s\n' "${crates_release[@]}")
    echo "    → 怎么办：第五段补上 --release（插在第一个 -- 之前；--profile release 也算写了），判法在 scripts/mutation/lib_release_only_tests.py；"
    echo "      改完单跑 python3 scripts/mutation/crates-mutation-rows.py static-check . 看那一行不再报 RELEASE，再单跑那一行证明点名的测试红（scripts/mutation/prove-red.sh）。"
  fi
  if ((${#scope_unreachable[@]})); then
    echo "  ✗ crates/mutations.tsv 这 ${#scope_unreachable[@]} 行点名的测试够不着被改坏的函数（closure：点名测试的单元闭包里根本没有被改坏的单元；gpu-route：开 GPU 时点名测试走 GPU 判段，被改坏的单元只在 GPU 路上不走的 CPU 判器入口后面；改了它测试照样过，这一行等于没判）："
    while IFS=$'\t' read -r _ lineno name route mutated test cargo_arguments; do
      printf '      crates/mutations.tsv:%s %s（%s）：被改坏的 %s，点名的测试 %s（门禁给这一行用的 cargo 参数 %s）\n' "$lineno" "$name" "$route" "$mutated" "$test" "$cargo_arguments"   # gate-lint:detail
    done < <(printf '%s\n' "${scope_unreachable[@]}")
    echo "    → 怎么办：换成调得到被改坏函数的用例，或删掉这一行；换完单跑 python3 scripts/mutation/crates-mutation-rows.py static-check . 看那一行不再报 UNREACHABLE，再单跑那一行证明点名的测试红（scripts/mutation/prove-red.sh）；"
    echo "      判法与每一种的出路见 scripts/mutation/crates-mutation-rows.py 文件头「点名测试够不着被改坏的函数」。"
  fi
  if ((${#scope_reachability_unjudged[@]})); then
    echo "  ✗ crates/mutations.tsv 这 ${#scope_reachability_unjudged[@]} 行判不了点名的测试够不够得着被改坏的函数（锚点失效、拒算、闭包算不成；判不了不算够得着）："
    while IFS=$'\t' read -r _ lineno name why; do
      printf '      crates/mutations.tsv:%s %s：%s\n' "$lineno" "$name" "$why"   # gate-lint:detail
    done < <(printf '%s\n' "${scope_reachability_unjudged[@]}")
    echo "    → 怎么办：照每行写的原因改（锚点失效的先改原文；闭包拒算的照 scripts/gate/lib_function_units.py 文件头「拒算」改成字面写法）；改完单跑 python3 scripts/mutation/crates-mutation-rows.py static-check . 看那一行不再报 REACHABILITY_UNJUDGED。"
  fi
  exit 1
fi
n=$(ls "$BIN_DIR"/*.rs | grep -cvE '/(lib|main)\.rs$')   # 与上面的循环同一个射程：crate 根的库与主二进制不算
crates_summary="；没有 crates/mutations.tsv"
[[ -f crates/mutations.tsv ]] && crates_summary="；crates/mutations.tsv ${scope_row_count} 条的原文各命中源码一次（成形：六段，替换文可以空、其余五段不空；点名的测试名都是用例名）"
crates_summary+="；门禁 crates-mutation-replay 的拷贝范围（${scope_copied}）：变异表 ${scope_row_count} 行改的文件都在里面，crates/ 下 ${scope_source_count} 份 .rs 里逃出 crates/ 的相对路径字面量 ${scope_literal_count} 处都在里面"
if [[ -n "$scope_identifiers" ]]; then
  IFS=$'\t' read -r identifiers_judged identifiers_unjudged <<<"$scope_identifiers"
  crates_summary+="；替换文引的标识符判了 ${identifiers_judged} 行、都找得到定义（没判 ${identifiers_unjudged} 行：文件不是 .rs 或锚点失效）"
fi
if [[ -n "$release_lines_read" && -f crates/mutations.tsv ]]; then
  crates_summary+="；命中重测试目标的行第五段都写了 --release（判了 ${scope_row_count} 行：gpu 特性、登记的崩溃枚举用例目标）"
fi
if [[ -n "$scope_reachability_checked" ]]; then
  IFS=$'\t' read -r reachability_judged reachability_unjudged reachability_exempt reachability_cut _reachability_rest <<<"$scope_reachability_checked"
  reachability_gpu="${scope_reachability_checked##*$'\t'}"   # 最后一段：ENABLE_GPU=1 或 GPU 没开（前一段多机配置路径可以是空的，不按位置读）
  crates_summary+="；点名的测试都够得着被改坏的函数（判了 ${reachability_judged} 行、没判 ${reachability_unjudged} 行，豁免 ${reachability_exempt} 行，截过闭包 ${reachability_cut} 行；${reachability_gpu}）"
elif [[ -n "$scope_reachability_not_judged" ]]; then
  crates_summary+="；点名的测试够不够得着被改坏的函数本次未判（${scope_reachability_not_judged}），不算判过"
fi
if ((${#tables_without_binary[@]})); then
  crates_summary+="；没有同名实验二进制、锚点不在这一格射程的表 ${#tables_without_binary[@]} 张：$(printf '%s ' "${tables_without_binary[@]}")"
fi
if ((${#lag_groups[@]})); then
  echo "  · 重复行滞后 ${#lag_groups[@]} 组（$DUPLICATES_LAG 登记、这一格不判红；逐组：表、组内各行的变异名）："
  printf '      %s\n' "${lag_groups[@]//$'\t'/ }"
fi
lag_summary="；重复行滞后 ${#lag_groups[@]} 组（$DUPLICATES_LAG 登记 ${lag_rows} 行，${lag_baseline}）"
[[ -f "$DUPLICATES_LAG" ]] || lag_summary="；重复行滞后 0 组（没有 $DUPLICATES_LAG）"
echo "  ✓ $n 个实验二进制都有成形的变异表，${anchor_checked} 条变异的原文各命中源码一次，${tables_read} 张研究变异表里没有两行除变异名外各列全同${lag_summary}${crates_summary}（这一格不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行改法相同、点名标了 #[ignore] 的测试都带 --include-ignored、筛选词筛得到点名的测试、点名的测试在选的目标里找得到、-p 点名的包都是仓根 Cargo.toml 的工作区成员（${rows_with_member_packages:-0} 行）、--manifest-path 指到的都是仓内的 Cargo.toml（${rows_with_manifest_paths:-0} 行）——crates 那张表的重复按「文件 + 原文 + 替换文」三项认，变异名另判）"
}

# ── 格 absolute-assertions：每个实验二进制都要有钉绝对值的断言
#
# 依据：.claude/singlefs-ai-sop/rules/test-discipline.md「只让多条臂互相比，测不出「所有臂一起错」」。
# 只判机器判得了的一半：这个实验有没有任何一条把某个量与数字字面量比死的断言；钉的是不是对的量、绝对值是不是独立算出来的，留给人。
# 射程：scripts/experiment-apparatus 下除 lib.rs、main.rs 之外的每一份，加上 crates/*/src/bin 下文件名以 e<数字>_ 开头的
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
BINS=scripts/experiment-apparatus
# $BINS 不在不等于无对象：住在 crates/ 下、以 e<数字>_ 开头的实验照判；两处都一份没有，才在末尾退 77
judged=() uncovered=()
for f in "$BINS"/*.rs; do
  case "$(basename "$f")" in lib.rs|main.rs) continue ;; esac   # crate 根的库与主二进制不是一份实验
  [[ -f "$f" ]] && judged+=("$f")   # 目录是空的时候 glob 原样留着，不是一份实验
done
in_bins=${#judged[@]}
for directory in crates/*/src/bin; do
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
  echo "               不许从代码里读回来。加完用 scripts/mutation/mutate.sh 证明它会红——"
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
# 扫 scripts/experiment-apparatus/*.rs 与 crates/singlefs-verification/src/bin/e<号>*.rs，只看注释与字符串之外的源码；两条都判完再退出：
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
# 一份住在 crates/singlefs-verification/src/bin/ 下同样折叠了种子的装置，豁免表基准那一版的 blob 被 setup.sh 从对象库里删掉，必须判红；
# green 是同一份源码加上两张对得上的豁免表，必须判绿（别的格的实验二进制一处都不犯，扫到的文件数随之是 4）。
cell_reading_discipline() {
base=""
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  LIB_CHANGED_PATHS="$STAGE_REPOSITORY/scripts/gate/changed-paths.sh"
  # shellcheck source=../../scripts/gate/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
  base="$(gate_diff_base gate)"
fi
python3 - "$base" "$OWED_LIBRARY" "$STAGE_DIRECTORY/lib/rejection-blocks.py" "$LAG_BASELINE_LIBRARY" <<'PY'
import glob, importlib.util, os, re, subprocess, sys

base = sys.argv[1]
lag_baseline_spec = importlib.util.spec_from_file_location("lag_baseline", sys.argv[4])
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)
owed_library_spec = importlib.util.spec_from_file_location("owed", sys.argv[2])
owed_library = importlib.util.module_from_spec(owed_library_spec)
owed_library_spec.loader.exec_module(owed_library)
rejection_blocks_spec = importlib.util.spec_from_file_location("rejection_blocks", sys.argv[3])
rejection_blocks = importlib.util.module_from_spec(rejection_blocks_spec)
rejection_blocks_spec.loader.exec_module(rejection_blocks)
BIN_GLOBS = ("scripts/experiment-apparatus/*.rs", "crates/singlefs-verification/src/bin/e[0-9]*.rs")
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
paths = sorted(path for pattern in BIN_GLOBS for path in glob.glob(pattern)
               if os.path.basename(path) not in ("lib.rs", "main.rs"))   # crate 根的库与主二进制不是一份实验
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
        # 经方法调用的结果写字段（`history_mut(…).名 = 值`）与结构体字面量里的字段简写（一行只有 `名,`，值是同名的局部变量）也算写过
        if "reading-writes-unseen" not in os.environ.get("GATE_READING_DISCIPLINE_BREAK", ""):
            assigned += [match.group(1).strip() for match in re.finditer(r"\)\s*\.\s*" + name + r"\s*=(?!=)\s*([^;,}\n]+)", text)]
            if re.search(r"^\s*" + name + r"\s*,\s*$", text, re.M):
                continue
        # 进了断言被拿来核的字段（钉值表的期望一侧，`assert_eq!(现算, 钉的.名)`）是钉值，不是没人算的读数
        if "asserted-fields-judged" not in os.environ.get("GATE_READING_DISCIPLINE_BREAK", "") and \
                re.search(r"assert(?:_eq|_ne)?!\s*\([^;]*?\.\s*" + name + r"\b", text):
            continue
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
    for line_number, fields in lag_baseline.content_rows(open(path, encoding="utf-8").read()):
        if len(fields) != 4 or not all(field.strip() for field in fields):
            malformed.append(f"第 {line_number} 行：{chr(9).join(fields)}")
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
    # 「基准里没有这张表」与「git 取不到基准那一版」分开判（lib-lag-baseline.py 的 baseline_rows）：后者判红，不当成初次登记静默不比
    baseline, problem = lag_baseline.baseline_rows(base, path)
    if problem is not None:
        reject(f"{problem.sentence(None)}，「只缩不涨」这一条没比：",
               [problem.stderr.strip()[:200]],
               ["怎么办：按上面 git 的报错修好仓库状态（基准要存在、对象库没坏，多半是对象库缺了那一份）再跑；取不到、读不出基线都不是「初次登记」。"])
        return None
    if baseline is None:
        return None
    counts = {}
    for _line_number, fields in baseline:
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
                "               改了就要重跑这个实验并按新产物逐个回对正文引的数（门禁 research-build-and-replay 会逐字节判红直到重跑）。",
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
# 登记表 .claude/gate.d/unsafe-allowances.tsv 一处放开一行（八段写法在表头）：每个根一组行。根是 crate 的库根（第 1 段只写 crate 目录名）或一个 bin 根
# （第 1 段写 <crate 目录名>/<bin 根相对 crate 目录的路径>）；没有放开的根一行、第 3 到第 8 段写 -；放开了的根每个放开的项一行。
# 项（第 4 段）三种写法：函数写函数名；直接写在 impl 块里的方法写 <类型>::<方法>（trait impl 与固有 impl 都按实现它的类型认，trait 名不进键：
#   两个类型各实现一次同一个 trait，方法同名也是两个键）；impl 块本身写 impl <Trait> for <类型>（固有 impl 写 impl <类型>）。类型与 trait 都取路径最后一节、去掉 <…>。
# 先判表本身，逐行：
#   - 八段都有内容；第 2 段只由 forbid、deny 组成；写了 deny 才有放开，第 3 到第 6 段（文件、项、项里 unsafe 几处、SAFETY 注释必须提到的词）
#     要么都写、要么都是 -；几处是正整数；必须提到的词按 | 切开，每个词都不空；
#   - 同一个（根，项）登两行判红（没有放开的那一行项写 -，同一个根写两行这种也红）；
#   - 同一个根的几行第 2 段不一样判红（各段写 - 的那一行与放开的行并存就是这一种）；
#   - 放开了的行，第 7 段理由不是 - 且不少于 20 个字，第 8 段出处是仓里真有的文件路径、或「用户 YYYY-MM-DD 弹窗定」加那个路径；history-kept: 出处栏认的写法本身，按格式占位写
#     没放开的行两段都写 -。只判理由在、够长，出处判文件存在。
# 再扫 crates/<crate>/ 下所有 .rs（跳过 target/），先抹掉注释与字符串（行号不变），按根分开（bin_roots_of）：
#   bin 根是 cargo 自动认的二进制目标根 src/main.rs、src/bin/<名>.rs、src/bin/<名>/main.rs；src/main.rs 只管它自己，
#   src/bin/<名>.rs 管它自己与 src/bin/<名>/ 下的文件（它 mod 引的子模块 cargo 就在那里找；那个目录里另有 main.rs 时整个目录归 src/bin/<名>/main.rs），
#   src/bin/<名>/main.rs 管 src/bin/<名>/ 下的整个目录；别的 .rs 都归库根（src/lib.rs）那一份，tests/、benches/ 也在里头；
#   include!("…") 与 #[path = "…"] 点到的文件（按写着它的那份文件所在目录解析）归写着它的那份文件的根，一并扫，不看扩展名；
#   点到的是哪个 crate 都当 .rs 扫的文件不扫第二遍，点到而不在、在仓根之外的跳过。每个根各自判 ①–④：
#   ① 根文件写了 `#![forbid(unsafe_code)]` 或 `#![deny(unsafe_code)]`，登记了的根什么都不写判红；登记表只许 forbid 的根写 deny 判红；
#      登记表里没有的 bin 根照 forbid 判：不写级别也行，写了 deny 判红；登记了的 bin 根在 crate 里不是 bin 根（文件不在、写法不对）判红；
#   ② 总数：`allow` / `expect` / `warn` 里带 `unsafe_code` 的属性（含 cfg_attr 里的）数一遍，根是 forbid 时必须 0 处，
#      根是 deny 时必须等于这个根登记的行数；`unsafe` 关键字（unsafe 块、unsafe fn 的签名、unsafe impl 的项头）同样数，根是 forbid 时 0 处，
#      根是 deny 时等于这个根各行第 5 段之和；不带 unsafe 写的外层属性 #[no_mangle]、#[export_name = …]、#[link_section = …]（含 cfg_attr 里的）
#      unsafe_code 这条 lint 照样报，各算一处 unsafe，位置记在它后面紧挨着的项头上（写成 #[unsafe(no_mangle)] 的由里面的 unsafe 关键字数，不重数）；
#   ③ 位置：每一处放开紧挨着的项是某一行登记的（第 3 段那个文件里的第 4 段那个项；属性与项头之间隔着别的属性与注释照认），不是整文件的内层属性 `#![allow(...)]`；
#      登记了放开的根里，每个 `unsafe` 关键字落在某一行登记的项里：项的范围从项头起（unsafe、pub 这类限定词算进去，所以 unsafe fn 签名与 unsafe impl 项头
#      里的 unsafe 都在里头）到项尾；几行都罩住同一处的（方法在它所在的 impl 块里）归范围最小的那一行；
#      根是 deny 时逐行再判：登记的项上恰好一处放开，项里 `unsafe` 关键字恰好第 5 段那么多处（这两条另随 ② 或 ③ 的弄坏开关一起不判）；
#   ④ 每个 `unsafe` 所在那条语句（或项）的开头一行上方，连着的 `//` 注释里有一行以 `// SAFETY:` 起头（中间隔着独占几行的外层属性照往上找，
#      所以 SAFETY 注释写在 #[allow(unsafe_code, …)] 上方或它与项头之间都认）；往上找读的是只留 // 注释、抹掉字符串与块注释的那一份，
#      按 \n 切行（与 line_of 数行号同一个切法；U+2028 这类不算换行），字符串里顶格写的 // SAFETY: 不算；落在登记项里的每一处，从那一行到语句开头上一行的注释文字
#      要提到那一行第 6 段登记的每个词（不分大小写按字面认；文件描述符、偏移、长度三个词另认同义写法，见 SAFETY_WORD_SYNONYMS）；
#   ⑤ crates/ 下出现登记表里没有库根那一行的 crate 目录判红；登记表里的 crate 目录不在仓里，跳过那几行并在成功那句里点名。
# 成功那句报扫了几个 crate、几个 .rs、登记表几行、放开几处、unsafe 几处、bin 根几个（登记了的几个），再逐个根列级别、放开、unsafe、.rs 与登记的行数
#   （没登记的 bin 根按 crate 合成一项）。
# 射程：tests/、benches/ 各是自己的 crate 根，lib.rs 的 forbid 管不到它们（编译器也不拦），这一格把它们算进库根那一份数；src/main.rs 用 mod 引的 src/ 下的文件也算库根那一份。
# 抹字符串认 `"…"`（连同 b、c 前缀）、原始字符串（r、br、cr 前缀）与字符字面量；research/ 不在射程。
# 按设计不展开宏，只认写出来的 unsafe 关键字（理由写在登记表表头）：宏定义里一个 unsafe 块展开几次都只算定义处一处，SAFETY 注释也只在定义处找。
# 项怎么认（items_of）：impl 块认 `unsafe impl` / `impl` 起头、前一个非空白字符是 { } ; ] 或文件开头的（`-> impl Trait` 这类类型位置上的不算），
#   项头到 { 为止；函数认 `fn <名>`，签名到圆括号、方括号之外的第一个 { 或 ; 为止。trait 定义里的方法照函数名认。
#   同一份文件里几个方法的 <类型>::<方法> 一样（同一个类型的几个固有 impl、或几个 trait impl 里同名的方法）时，这几个都改用全写（写法在 items_of 上方的注释与登记表表头）。
# 语句开头怎么找（statement_start_line）：从 `unsafe` 那一行往上，上一行抹掉注释与字符串之后以 STATEMENT_CONTINUES 列的符号结尾（没收尾），
#   或这一行以 CHAIN_CONTINUES（单个 .）起头（方法链的续行，中间只隔着注释与空行的照跳过），就算同一条语句的续行，一并跳过接着往上；
#   rustfmt 把 `let x =` 与 `unsafe { … }` 折成两行、把 `let x = a()` 与 `.map(|_| unsafe { … })` 折成几行时，写在 `let` 上方的注释照样认。
#   SAFETY 只按语句开头认，不按 unsafe 所在那一行认：写在链中间、续行上方的注释不认。哪几种结尾算没收尾、逗号与 { 为什么不算，写在 STATEMENT_CONTINUES 上方。
#   同一条语句里几处 unsafe（续行接着的第二个 unsafe 块）共用语句开头上方那一段注释，按设计不要求各写一段（理由写在登记表表头）。
#   注释与语句开头之间隔着一条已经收尾的别的语句（以 ; 结尾），那段注释不算这一处的。
# 不判的：理由与 SAFETY 前提写得对不对、`unsafe` 块里的代码对不对（靠 review）。
# 样本（fixtures/code-source-discipline.sh/ 下，各自带登记表与它的出处文件 records/sample-decision.md）：
#   unsafe-allowances-green               core 根 forbid 零处放开、另一个 crate 也 forbid
#   unsafe-allowances-deny-one-green      deny 恰一处
#   unsafe-allowances-several-rows-green  core 登两行：打洞那个函数一处 unsafe、读内存堆那个函数两处，各提到自己那一行登记的词；harness 根 forbid
#   unsafe-allowances-two-allows-red      两处 allow 叠在同一个函数上
#   unsafe-allowances-wrong-function-red  allow 与 unsafe 在别的函数上
#   unsafe-allowances-no-safety-red       没有 SAFETY 注释
#   unsafe-allowances-root-blank-red      core 根什么都不写
#   unsafe-allowances-other-crate-red     harness 的 tests/ 里 allow 加 unsafe，根仍是 forbid
#   unsafe-allowances-reason-empty-red    放开了而理由栏写 -
#   unsafe-allowances-source-missing-red  出处指向仓里没有的文件
#   unsafe-allowances-safety-vague-red    有 SAFETY 注释但没提描述符、偏移、长度
#   unsafe-allowances-place-count-wrong-red        两行的处数登反了（登 2 处的函数里 1 处、登 1 处的函数里 2 处），总数照样对得上
#   unsafe-allowances-one-place-no-safety-red      登 2 处的读内存堆那个函数里，读堆（read_the_memory_heap）那一处 unsafe 上方的注释不以 // SAFETY: 起头
#   unsafe-allowances-one-place-word-missing-red   登 2 处的读内存堆那个函数里，读堆那一处的 SAFETY 注释没提到登记的词 hal
#   unsafe-allowances-allow-count-short-red        登了两行，只有一个函数上有 allow
#   unsafe-allowances-allows-on-one-function-red   登了两行，两处 allow 叠在同一个函数上、另一个没有，总数照样对得上
#   unsafe-allowances-same-function-two-rows-red   同一个函数登了两行
#   unsafe-allowances-crate-rows-disagree-red      core 一行各段写 -（只许 forbid）、一行放开（forbid,deny）
#   unsafe-allowances-statement-continued-green    rustfmt 折行：// SAFETY: 写在 `let return_code =` 上方，`unsafe { … }` 在下一行；只有 core 一个 crate
#   unsafe-allowances-statement-finished-between-red  同一段 // SAFETY: 与 `let return_code =` 之间隔着一条以 ; 收尾的 `let checked_length = length;`
#   下面七份的库根 crates/singlefs-verification/src/lib.rs 写 forbid；前三份的 bin 根是 src/bin/e900_thread_clock.rs（读线程钟），unsafe-impl 那三份是 src/bin/e900_counting_allocator.rs：
#   unsafe-allowances-bin-root-green               bin 根写 deny、单独登一行（一个函数一处 unsafe）；另一个 bin 根 e901_plain_report.rs 没登记、不写级别、零处
#   unsafe-allowances-bin-root-unregistered-red    bin 根没登记、不写级别，写了一处带 SAFETY 注释的 unsafe
#   unsafe-allowances-bin-root-forbid-red          表里给 bin 根只登 forbid 一行，bin 根却写 deny、放开一个函数
#   unsafe-allowances-bin-root-not-found-red       表里登记的 bin 根 e999_removed_allocator.rs 不在（crate 里只有没登记的 e901_plain_report.rs）
#   unsafe-allowances-unsafe-impl-green            bin 根里计数分配器：unsafe impl 一行（1 处）、两个 unsafe fn 方法各一行（签名加函数体各 1 处）；
#                                                  SAFETY 注释写在放开属性上方，有一处放开属性折成四行
#   unsafe-allowances-unsafe-impl-no-safety-red    同上，unsafe impl 上方的注释不以 // SAFETY: 起头
#   unsafe-allowances-unsafe-fn-no-safety-red      同上，alloc 签名上方的注释不以 // SAFETY: 起头（函数体里那一处照写着）
#   unsafe-allowances-same-method-two-types-green  core 库根 deny：两个类型各实现一次 GlobalAlloc、各有 alloc 与 dealloc，按 <类型>::<方法> 各登一行；
#                                                  SAFETY 注释写在放开属性与项头之间
#   下面十五份是 m4-gpugc-unsafe-gate-r1 判决 Q1 那八处判法与两样按设计的钉样，库根 forbid、bin 根都是 src/bin/e900_thread_clock.rs：
#   unsafe-allowances-line-separator-in-a-comment-red          注释里夹一个 U+2028，SAFETY 与 unsafe 之间隔着一条收尾的语句（按 \n 切行才判红）
#   unsafe-allowances-same-text-without-the-line-separator-red 同一份源码，U+2028 换成空格，照样红
#   unsafe-allowances-safety-line-inside-a-string-red          // SAFETY: 那一行在字符串字面量里，不是注释
#   unsafe-allowances-raw-c-string-hides-unsafe-red            两个 cr"\" 中间夹一处 unsafe；bin 根没登记，照 forbid 红
#   unsafe-allowances-include-of-a-non-rs-file-red             bin 根 include! 一份 .inc，unsafe 在 .inc 里；bin 根没登记
#   unsafe-allowances-path-attribute-to-a-non-rs-file-red      bin 根 #[path] 引一份 .inc 当模块，unsafe 在 .inc 里；bin 根没登记
#   unsafe-allowances-no-mangle-without-the-keyword-red        不带 unsafe 的 #[no_mangle]；bin 根没登记
#   unsafe-allowances-no-mangle-registered-green               bin 根 deny，登记放开导出符号的函数（1 处），SAFETY 注释写在放开属性上方
#   unsafe-allowances-bin-helper-directory-registered-green    放开写在 src/bin/e900_thread_clock/ 下的子模块里，登记在 bin 根名下
#   unsafe-allowances-bin-helper-directory-unregistered-red    同样的子模块里写 unsafe，bin 根没登记，红在 bin 根名下
#   unsafe-allowances-unsafe-below-the-first-line-of-a-method-chain-green  unsafe 在 .map(…) 那一行，SAFETY 写在 let 上方
#   unsafe-allowances-same-method-name-in-two-inherent-impls-green         impl Reader<u8> 与 impl Reader<u16> 各有 read，登 Reader<u16>::read
#   unsafe-allowances-same-method-name-registered-without-type-arguments-red  同样两个 read，登记仍写 Reader::read，红在放开那一处、点名全写
#   unsafe-allowances-macro-expands-three-unsafe-for-one-keyword-green     按设计：宏定义里一处 unsafe 调三次，登 1 处
#   unsafe-allowances-one-comment-covers-the-second-unsafe-of-a-chain-green  按设计：一条语句两处 unsafe 共用语句开头上方的一段 SAFETY，登 2 处
# 弄坏开关 GATE_UNSAFE_ALLOWANCES_BREAK 各项在文件头。
cell_unsafe_allowances() {
  [[ -d crates ]] || { echo "  ! 没有 crates/ 目录，这一格无对象可判"; exit 77; }

  python3 - <<'PY'
import os, re, sys

BREAK = os.environ.get("GATE_UNSAFE_ALLOWANCES_BREAK", "")
REGISTRY_PATH = ".claude/gate.d/unsafe-allowances.tsv"
REGISTRY_FIELD_COUNT = 8
REASON_MINIMUM = 20
PLACE_COUNT = re.compile(r"[1-9][0-9]*")
# 登记表第 6 段里这三个词另认同义写法：打洞那一行（singlefs-core）登的是这三个词，它的 SAFETY 注释写的是「描述符」「offset」这类说法；别的词按字面认。
SAFETY_WORD_SYNONYMS = {"文件描述符": re.compile(r"描述符|file descriptor|\bfd\b", re.I),
                        "偏移": re.compile(r"偏移|offset", re.I),
                        "长度": re.compile(r"长度|length|\blen\b", re.I)}
# 弄坏开关 registered-words-ignored 下每一处都照这三个词判，不读登记表第 6 段。
HOLE_PUNCHING_WORDS = ("文件描述符", "偏移", "长度")
# 一行抹掉注释与字符串之后以这几种符号结尾，就是语句没收尾、下一行是它的续行：=（连同 +=、== 这类以 = 结尾的）、( 与 [、
# 运算符 + - * / % & | ^ ! < >（=> 与 -> 的末字符也在里头）。逗号与 { 不算：match 的一个分支、结构体字面量的一个字段、
# 函数头或块的头一行以它们结尾时，下一行是另一处的代码，跳过去会把写给别处的 SAFETY 注释认成这一处的。
STATEMENT_CONTINUES = re.compile(r"[=(\[+\-*/%&|^!<>]\s*$")
# 一行抹掉注释与字符串之后以单个 . 起头（不是 .. 区间），是方法链接着上面那一段：rustfmt 把 `let x = a()` 与 `.map(|_| unsafe { … })` 折成几行时，
# unsafe 所在那一行不是语句开头，写在 `let` 上方的注释照样认。SAFETY 只按语句开头认，写在链中间那几行上方的注释不认。
CHAIN_CONTINUES = re.compile(r"\s*\.(?!\.)")
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
# 原始字符串 r"…" 前面可以紧挨着的一个前缀字母：b（字节串 br"…"）与 c（C 字符串 cr"…"）。弄坏开关 c-string-prefix-unknown 下只认 b：
# cr"\" 里的 \ 被当成转义，抹字符串失步、吞掉后面的代码。
RAW_STRING_PREFIXES = "b" if BREAK == "c-string-prefix-unknown" else "bc"


def strip_comments_and_strings(text, keep_line_comments=False):
    """注释整段抹成空格，字符串只抹里面（引号留着），换行都留着，行号与每个字符的位置都不变。
    keep_line_comments 为真时 // 注释原样留着（块注释照抹）：找 SAFETY 注释读这一份，字符串里顶格写的 // SAFETY: 不当注释。"""
    out, index, length = [], 0, len(text)
    while index < length:
        char = text[index]
        if text.startswith("//", index):
            end = text.find("\n", index)
            end = length if end < 0 else end
            out.append(text[index:end] if keep_line_comments else blank(text[index:end])); index = end
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
            or (text[index - 1] in RAW_STRING_PREFIXES and (index == 1 or not (text[index - 2].isalnum() or text[index - 2] == "_")))
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
# 项头：函数的限定词连 fn 与函数名；impl 块的 unsafe impl / impl。属性后面跟着的项从 ITEM_AFTER_ATTRIBUTES 收尾处起。
FUNCTION_ITEM = re.compile(
    r"(?:\bpub(?:\s*\([^)]*\))?\s+)?(?:\b(?:const|async|unsafe|safe|default)\s+|\bextern\s+\"[^\"]*\"\s+)*\bfn\s+([A-Za-z_][A-Za-z0-9_]*)"
)
IMPL_ITEM = re.compile(r"\b(?:unsafe\s+)?impl\b")
ITEM_AFTER_ATTRIBUTES = re.compile(r"\s*(?:#\s*\[[^\]]*\]\s*)*")
UNSAFE_WORD = re.compile(r"\bunsafe\b")
# 2021 版不带 unsafe 写的这三种外层属性，unsafe_code 这条 lint 照样报（2024 版要写成 #[unsafe(no_mangle)]，那里的 unsafe 由 UNSAFE_WORD 数）：
# 各算项里的一处 unsafe，位置记在它后面紧挨着的项头上。
EXPORTED_SYMBOL_ATTRIBUTE = re.compile(r"\b(?:no_mangle|export_name|link_section)\b")
# include!("…") 与 #[path = "…"]：在抹过注释与字符串的那一份里认，路径从原文同一位置读（字符串里只抹内容、引号留着）。
FILE_REFERENCE = re.compile(r'(?:\binclude!\s*\(\s*|#\s*\[\s*path\s*=\s*)"')
# cargo 自动认的二进制目标根，相对 crate 目录。
BIN_ROOT_FILE = re.compile(r"src/main\.rs|src/bin/[^/]+\.rs")
BIN_ROOT_DIRECTORY_MAIN = re.compile(r"src/bin/[^/]+/main\.rs")


def bin_roots_of(relative_paths):
    """crate 目录下（相对路径）的 bin 根：{根文件: 这个根管的文件}。src/main.rs 只管它自己；src/bin/<名>.rs 管它自己与 src/bin/<名>/ 下的全部
    （它的 mod 照 cargo 的规矩从那个目录找；同一个目录里另有 src/bin/<名>/main.rs 时那个目录归后者）；src/bin/<名>/main.rs 管 src/bin/<名>/ 下的全部。
    弄坏开关 bin-roots-in-library 下一个都不认（全算进库根）；bin-helper-directory-in-library 下 src/bin/<名>.rs 只管它自己。"""
    if BREAK == "bin-roots-in-library":
        return {}
    roots = {}
    for relative in relative_paths:
        if BIN_ROOT_FILE.fullmatch(relative):
            members = [relative]
            helper_directory = relative[:-len(".rs")] + "/"
            if (relative.startswith("src/bin/") and BREAK != "bin-helper-directory-in-library"
                    and helper_directory + "main.rs" not in relative_paths):
                members += [other for other in relative_paths if other.startswith(helper_directory)]
            roots[relative] = members
        elif BIN_ROOT_DIRECTORY_MAIN.fullmatch(relative):
            directory = relative[:-len("main.rs")]
            roots[relative] = [other for other in relative_paths if other.startswith(directory)]
    return roots


def closing_brace(cleaned, opening):
    """从 opening 处的 { 起，配对的 } 之后那个位置。"""
    depth, index = 0, opening
    while index < len(cleaned):
        depth += {"{": 1, "}": -1}.get(cleaned[index], 0)
        index += 1
        if depth == 0:
            break
    return index


def signature_end(cleaned, index):
    """从 index 往后，圆括号、方括号之外的第一个 { 或 ;：交回（位置, 那个字符）；到文末都没有交回（文末, ""）。"""
    depth = 0
    while index < len(cleaned):
        char = cleaned[index]
        if char in "([":
            depth += 1
        elif char in ")]":
            depth -= 1
        elif char in "{;" and depth <= 0:
            return index, char
        index += 1
    return index, ""


def skip_generic_parameters(text):
    """impl 后面紧跟的 <…>（-> 里的 > 不算）去掉，交回余下的。"""
    stripped = text.lstrip()
    if not stripped.startswith("<"):
        return stripped
    depth = 0
    for index, char in enumerate(stripped):
        if char == "<":
            depth += 1
        elif char == ">" and stripped[index - 1] != "-":
            depth -= 1
            if depth == 0:
                return stripped[index + 1:]
    return ""


TYPE_LEADING_QUALIFIERS = re.compile(r"^(?:\s+|[&!]|'[A-Za-z_][A-Za-z0-9_]*|\bmut\b|\bdyn\b)*")
INLINE_MODULE = re.compile(r"\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{")


def path_name(text):
    """类型或 trait 的写法里认名字：去掉打头的 &、!、生命周期、mut、dyn，取 < 之前那段路径的最后一节。"""
    text = TYPE_LEADING_QUALIFIERS.sub("", text)
    return "".join(text.split("<", 1)[0].split("::")[-1].split())


def written_type(text):
    """类型或 trait 连同 <…> 里的实参：去掉打头的 &、!、生命周期、mut、dyn 与全部空白，< 之前那段路径只留最后一节（实参原样留着）。"""
    compact = "".join(TYPE_LEADING_QUALIFIERS.sub("", text).split())
    head, angle, arguments = compact.partition("<")
    return head.split("::")[-1] + angle + arguments


def items_of(cleaned):
    """一份抹过注释与字符串的文件里的 impl 块与函数，每项 {"key", "start", "body", "end"}：key 是登记表第 4 段的写法，start 是项头起点，
    body 是 { 的位置（没有函数体的声明是 None），end 是项尾之后。弄坏开关 methods-keyed-by-name 下方法也只按方法名认。
    同一份文件里几个方法的 <类型>::<方法> 一样（同一个类型的几个 impl 块里同名的方法）时，这几个都改用全写：
    <内联模块路径::><类型连同实参>::<方法>，trait impl 里的写成 <<内联模块路径::><类型连同实参> as <Trait 连同实参>>::<方法>；
    弄坏开关 colliding-methods-keyed-alike 下不改（撞了的几个同一个键，登记表那一行只认得到最靠前的那个）。"""
    modules = []
    for match in INLINE_MODULE.finditer(cleaned):
        modules.append((match.group(1), match.end() - 1, closing_brace(cleaned, match.end() - 1)))

    def module_path_at(position):
        return "".join(f"{name}::" for name, opening, closing in modules if opening < position < closing)

    impls = []
    for match in IMPL_ITEM.finditer(cleaned):
        before = match.start() - 1
        while before >= 0 and cleaned[before].isspace():
            before -= 1
        if before >= 0 and cleaned[before] not in "{};]":
            continue
        body, char = signature_end(cleaned, match.end())
        if char != "{":
            continue
        header = re.split(r"\bwhere\b", skip_generic_parameters(cleaned[match.end():body]), maxsplit=1)[0]
        parts = re.split(r"\bfor\b(?!\s*<)", header, maxsplit=1)
        type_name = path_name(parts[-1])
        key = f"impl {path_name(parts[0])} for {type_name}" if len(parts) == 2 else f"impl {type_name}"
        full_type = module_path_at(match.start()) + written_type(parts[-1])
        full_owner = f"<{full_type} as {written_type(parts[0])}>" if len(parts) == 2 else full_type
        impls.append({"key": key, "type": type_name, "full_owner": full_owner,
                      "start": match.start(), "body": body, "end": closing_brace(cleaned, body)})
    functions = []
    for match in FUNCTION_ITEM.finditer(cleaned):
        boundary, char = signature_end(cleaned, match.end())
        body = boundary if char == "{" else None
        end = closing_brace(cleaned, boundary) if char == "{" else boundary + 1
        functions.append({"name": match.group(1), "start": match.start(), "body": body, "end": end})
    for function in functions:
        enclosing = [item for item in impls + functions
                     if item["body"] is not None and item["body"] < function["start"] < item["end"]]
        innermost = min(enclosing, key=lambda item: item["end"] - item["body"], default=None)
        is_method = innermost is not None and "type" in innermost and BREAK != "methods-keyed-by-name"
        function["key"] = f"{innermost['type']}::{function['name']}" if is_method else function["name"]
        if is_method:
            function["full_key"] = f"{innermost['full_owner']}::{function['name']}"
    if BREAK != "colliding-methods-keyed-alike":
        method_key_counts = {}
        for function in functions:
            if "full_key" in function:
                method_key_counts[function["key"]] = method_key_counts.get(function["key"], 0) + 1
        for function in functions:
            if "full_key" in function and method_key_counts[function["key"]] > 1:
                function["key"] = function["full_key"]
    return impls + functions


def item_span(item):
    """登记的一项管的范围 [起, 止)：从项头起到项尾。弄坏开关 signature-outside-body 下从 { 起（签名与 impl 项头落在外面）。"""
    if BREAK == "signature-outside-body":
        return None if item["body"] is None else (item["body"], item["end"])
    return item["start"], item["end"]


def attribute_lines(cleaned):
    """独占几行的外层属性（同一行上属性前后没有别的代码）占的行号（从 1 数）：找 SAFETY 注释时隔着它们往上找。
    弄坏开关 attributes-not-skipped 下一行都不算（碰到属性就停）。"""
    lines = set()
    if BREAK == "attributes-not-skipped":
        return lines
    for start, end, inner, body in attributes(cleaned):
        line_start = cleaned.rfind("\n", 0, start) + 1
        line_end = cleaned.find("\n", end)
        line_end = len(cleaned) if line_end < 0 else line_end
        if not inner and not cleaned[line_start:start].strip() and not cleaned[end:line_end].strip():
            lines.update(range(line_of(cleaned, start), line_of(cleaned, end - 1) + 1))
    return lines


def statement_start_line(cleaned_lines, unsafe_line):
    """unsafe 所在那条语句的开头一行（从 0 数）：从 unsafe 那一行往上，上一行抹掉注释与字符串之后以 STATEMENT_CONTINUES 结尾的，
    或这一行以 CHAIN_CONTINUES 起头的（方法链接着上面那一段；中间只有注释与空行的照跳过），算同一条语句的续行，一并跳过接着往上。
    弄坏开关 continuation-lines-ignored 下两种都不跳；chain-lines-not-continued 下不认以 . 起头的那一种。"""
    start = unsafe_line - 1
    while BREAK != "continuation-lines-ignored" and start > 0:
        if STATEMENT_CONTINUES.search(cleaned_lines[start - 1]):
            start -= 1
        elif BREAK != "chain-lines-not-continued" and CHAIN_CONTINUES.match(cleaned_lines[start]):
            above = start - 1
            while above > 0 and not cleaned_lines[above].strip():
                above -= 1
            if not cleaned_lines[above].strip():
                break
            start = above
        else:
            break
    return start


def safety_comment(comment_lines_of_the_file, cleaned_lines, unsafe_line, skipped_lines):
    """unsafe 所在那条语句（或项）的开头一行上方，连着的 // 注释里有一行以 // SAFETY: 起头：返回从那一行往下到语句开头之间的注释文字；没有返回 None。
    comment_lines_of_the_file 是只留 // 注释、抹掉字符串与块注释的那一份按 \\n 切的行（strip_comments_and_strings 带 keep_line_comments），
    与 cleaned_lines 行号一一对上；字符串里顶格写的 // SAFETY: 在这一份里已经抹掉，不当注释。
    往上找注释时 skipped_lines（attribute_lines 给的属性行）照跳过，不算注释也不打断。"""
    start = statement_start_line(cleaned_lines, unsafe_line)
    comment_lines = []
    number = start - 1
    while number >= 0:
        stripped = comment_lines_of_the_file[number].lstrip()
        if stripped.startswith("//"):
            comment_lines.append(comment_lines_of_the_file[number])
            if stripped.startswith("// SAFETY:"):
                return "\n".join(reversed(comment_lines))
        elif number + 1 not in skipped_lines:
            break
        number -= 1
    return None


def safety_word_pattern(word):
    """登记表第 6 段的一个词怎么认：SAFETY_WORD_SYNONYMS 里的三个词认同义写法，别的词不分大小写按字面认。"""
    return SAFETY_WORD_SYNONYMS.get(word) or re.compile(re.escape(word), re.I)


def registered_row_holding(allowance_rows, spans, path, position):
    """unsafe 关键字（path 文件里的 position 处）落在哪一行登记的项里：几行都罩住它的取范围最小的那一行（方法先于它所在的 impl 块）；
    不在任何一行里返回 None。弄坏开关 outermost-holder 下取表里靠前的那一行。"""
    covering = []
    for row in allowance_rows:
        span = spans.get((row["file"], row["function"]))
        if path == row["file"] and span is not None and span[0] <= position < span[1]:
            covering.append((span[1] - span[0], row))
    if not covering:
        return None
    if BREAK == "outermost-holder":
        return covering[0][1]
    return min(covering, key=lambda pair: pair[0])[1]


# ── 登记表 ──
registry = {}           # 根（第 1 段：库根写 crate 目录名，bin 根写 <crate 目录名>/<bin 根>）→ {项（没有放开的那一行是 -）: 登记的那一行}
row_of_function = {}    # （根，项）→ 表里的行号
if not os.path.isfile(REGISTRY_PATH):
    reject(f"没有登记表 {REGISTRY_PATH}", [], f"照 {REGISTRY_PATH} 的表头写法建这张表：crates/ 下每个 crate 至少一行；表是这一格判的依据，不能没有。")
    finish()
for number, raw in enumerate(open(REGISTRY_PATH, encoding="utf-8").read().splitlines(), 1):
    if not raw.strip() or raw.lstrip().startswith("#"):
        continue
    fields = raw.split("\t")
    if len(fields) != REGISTRY_FIELD_COUNT or not all(field.strip() for field in fields):
        reject(f"{REGISTRY_PATH}:{number} 不是八段（根、根允许的级别、放开的文件、放开的项、项里 unsafe 几处、SAFETY 注释必须提到的词、理由、出处），段间用制表符，每段都有内容",
               [raw], "照表头写法改这一行；没有放开的行，第 3 到第 8 段都写 -。")
        continue
    root_name, levels, allowed_file, allowed_function, place_count, words, reason, source = (field.strip() for field in fields)
    level_list = levels.split(",")
    word_list = [word.strip() for word in words.split("|")]
    row_allows = allowed_file != "-"
    if (any(level not in ("forbid", "deny") for level in level_list)
            or ("deny" in level_list) != row_allows
            or any((field == "-") == row_allows for field in (allowed_function, place_count, words))
            or (row_allows and (not PLACE_COUNT.fullmatch(place_count) or not all(word_list)))):
        reject(f"{REGISTRY_PATH}:{number} 这一行自相矛盾", [raw],
               "级别只写 forbid 或 forbid,deny；写了 deny 才有放开，第 3 到第 6 段（文件、项、项里 unsafe 几处、SAFETY 注释必须提到的词）都写上，"
               "没有 deny 这四段都写 -；几处写正整数，必须提到的词用 | 分隔、每个词都不空。")
        continue
    function_key = (root_name, allowed_function)
    if function_key in row_of_function and BREAK != "duplicate-rows-allowed":
        subject = "没有放开的那一行" if allowed_function == "-" else f"函数 {allowed_function} "
        reject(f"{REGISTRY_PATH}:{number} {root_name} 的{subject}登了两行（另一行在第 {row_of_function[function_key]} 行）", [raw],
               "一个函数只登一行：函数里的几处 unsafe 合写在那一行的第 5 段（几处）与第 6 段（每一处都要提到的词）；没有放开的 crate 只写一行。"
               "方法写 <类型>::<方法>、impl 块写 impl <Trait> for <类型>，两个类型的同名方法是两个键。")
        continue
    if not row_allows:
        if reason != "-" or source != "-":
            reject(f"{REGISTRY_PATH}:{number} {root_name} 没有放开，理由与出处两段却写了内容", [raw],
                   "没有放开的行，理由与出处都写 -。")
            continue
    else:
        if BREAK != "reason-ignored" and (reason == "-" or len(reason) < REASON_MINIMUM):
            reject(f"{REGISTRY_PATH}:{number} {root_name} 放开了 unsafe，理由栏{'是 -' if reason == '-' else f'只有 {len(reason)} 个字'}（不少于 {REASON_MINIMUM} 个字）", [raw],
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
                reject(f"{REGISTRY_PATH}:{number} {root_name} 的出处不成立：{source_problem}", [raw],
                       "出处写记录这次定案的仓内文件路径，或「用户 YYYY-MM-DD 弹窗定」加那份记录的路径；记录文件要真在仓里。")
                continue
    row_of_function[function_key] = number
    registry.setdefault(root_name, {})[allowed_function] = {
        "number": number, "levels": level_list, "file": allowed_file, "function": allowed_function,
        "count": int(place_count) if row_allows else 0, "words": word_list if row_allows else []}
if BREAK != "crate-levels-disagreement-allowed":
    for root_name, rows in registry.items():
        if len({",".join(row["levels"]) for row in rows.values()}) > 1:
            numbers = "、".join(str(row["number"]) for row in rows.values())
            spelled = "、".join(f"第 {row['number']} 行 {','.join(row['levels'])}" for row in rows.values())
            reject(f"{REGISTRY_PATH}:{numbers} {root_name} 的几行第 2 段（根允许的级别）不一样：{spelled}", [],
                   "同一个 crate 的几行第 2 段写成一样：有放开的 crate 每一行都写 forbid,deny，不再留各段写 - 的那一行；没有放开的 crate 只写各段写 - 的那一行。"
                   "bin 根与库根各是一个根，各写各的一组。")
finish()


def crate_of(root_name):
    """登记表第 1 段属于哪个 crate 目录：库根就是它自己，bin 根取第一个 / 之前。"""
    return root_name.split("/", 1)[0]

crate_names = sorted(entry for entry in os.listdir("crates") if os.path.isdir(os.path.join("crates", entry)))
# scanned 一个根一项：（第 1 段的写法, 是不是没登记的 bin 根, 所属 crate, .rs 几个, 根的级别, 放开几处, unsafe 几处, 登记几行）
file_count, crate_count, scanned = 0, 0, []


def judge_root(label, crate_name, root_path, files, rows, registered):
    """判一个根（库根或 bin 根）的 ①–④：label 是登记表第 1 段的写法，files 是这个根管的 {路径: (原文, 抹过注释与字符串的)}，
    rows 是这个根登记的行；没登记的 bin 根（registered 为假）rows 是空的，级别照 forbid、不写也行。"""
    allowance_rows = [row for row in rows.values() if row["file"] != "-"]
    levels = [level for level in ("forbid", "deny") if any(level in row["levels"] for row in rows.values())] if registered else ["forbid"]
    has_allowance = bool(allowance_rows)
    relaxes, unsafes, level, items_by_path, skipped_by_path, comment_lines_by_path = [], [], None, {}, {}, {}
    for path, (original, cleaned) in files.items():
        places = []
        for start, end, inner, body in attributes(cleaned):
            if (not inner and BREAK != "exported-symbol-attributes-unseen"
                    and EXPORTED_SYMBOL_ATTRIBUTE.search(body) and not UNSAFE_WORD.search(body)):
                places.append((path, line_of(cleaned, start), ITEM_AFTER_ATTRIBUTES.match(cleaned, end).end()))
            if RELAX.search(body):
                if re.match(r"\s*(?:pub(?:\s*\([^)]*\))?\s+)?assume_specification\b", cleaned[end:]):
                    continue
                if path not in items_by_path:
                    items_by_path[path] = items_of(cleaned)
                following_start = ITEM_AFTER_ATTRIBUTES.match(cleaned, end).end()
                following = next((item for item in items_by_path[path] if item["start"] == following_start), None)
                relaxes.append((path, line_of(cleaned, start), None if inner else (following["key"] if following else None)))
            if path == root_path and inner and LEVEL.search(body):
                found = LEVEL.search(body).group(1)
                level = "forbid" if (found == "forbid" or level == "forbid") else "deny"
        for match in UNSAFE_WORD.finditer(cleaned):
            places.append((path, line_of(cleaned, match.start()), match.start()))
        unsafes.extend(sorted(places, key=lambda place: (place[1], place[2])))
    scanned.append((label, not registered, crate_name, len(files), level, len(relaxes), len(unsafes), len(rows)))

    if root_path not in files:
        reject(f"{label} 没有 crate 根 {root_path}", [], "登记表与判法都按 src/lib.rs 当库根；crate 改了布局就先改这一格。")
        return
    if BREAK != "root-level-ignored":
        if level is None and registered:
            reject(f"{root_path} 没写 #![forbid(unsafe_code)] 也没写 #![deny(unsafe_code)]", [],
                   "根上写 #![forbid(unsafe_code)]；登记了放开的 crate 写 #![deny(unsafe_code)]。")
        elif level is not None and level not in levels:
            unregistered_note = "" if registered else f"这个 bin 根在登记表里没有行，照 forbid 判；要在这里放开，先给它登记（第 1 段写 {label}）。"
            reject(f"{root_path} 的级别是 {level}，登记表只许 {'、'.join(levels)}", [],
                   f"根改回 forbid；要放开 unsafe 先改 {REGISTRY_PATH} 的行并走三方。{unregistered_note}")
    if level is None and not registered:
        level = "forbid"
    if level is None or level not in levels:
        return
    level_text = level if registered else "forbid（没登记的 bin 根照 forbid 判）"
    allowance_in_force = level == "deny"
    if allowance_in_force:
        expected_relaxes = 1 if BREAK == "allow-count-single" else len(allowance_rows)
        expected_unsafes = sum(row["count"] for row in allowance_rows)
        relax_basis = f"（登记表 {len(allowance_rows)} 行，一行一处）"
        unsafe_basis = f"（登记表 {len(allowance_rows)} 行第 5 段之和）"
    else:
        expected_relaxes, expected_unsafes, relax_basis, unsafe_basis = 0, 0, "", ""
    spans = {}
    for row in allowance_rows:
        if row["file"] in files:
            if row["file"] not in items_by_path:
                items_by_path[row["file"]] = items_of(files[row["file"]][1])
            item = next((item for item in items_by_path[row["file"]] if item["key"] == row["function"]), None)
            spans[(row["file"], row["function"])] = None if item is None else item_span(item)
    holders = [registered_row_holding(allowance_rows, spans, path, position) for path, line, position in unsafes]
    registered_items = "、".join(f"{row['file']} 的 {row['function']}" for row in allowance_rows)
    if BREAK != "count-ignored":
        if len(relaxes) != expected_relaxes:
            reject(f"{label} 放开 unsafe_code 的地方 {len(relaxes)} 处，根是 {level_text}、应当 {expected_relaxes} 处{relax_basis}",
                   [f"{path}:{line}" for path, line, function in relaxes] or ["（没有）"],
                   f"根写 forbid 就一处都不许放开；根写 deny 就登记的每一行恰好放开一处。新增一处放开要改 {REGISTRY_PATH} 的登记并走三方，不是加一个 allow。")
        if len(unsafes) != expected_unsafes:
            reject(f"{label} 的 unsafe 关键字 {len(unsafes)} 处，根是 {level_text}、应当 {expected_unsafes} 处{unsafe_basis}",
                   [f"{path}:{line}" for path, line, position in unsafes] or ["（没有）"],
                   f"tests/、benches/ 的文件算进库根那一份数，src/bin/ 下的 bin 根各自判、没登记的照 forbid；不要在那里写 unsafe"
                   f"（不带 unsafe 的 #[no_mangle]、#[export_name]、#[link_section] 各算一处，include! 与 #[path] 点到的文件算进写着它的那个根）。"
                   f"要新增走 {REGISTRY_PATH} 的登记（bin 根第 1 段写 <crate 目录名>/<bin 根>）与三方。")
    if BREAK != "placement-ignored":
        registered_pairs = {(row["file"], row["function"]) for row in allowance_rows}
        for path, line, function in relaxes:
            if function is None:
                reject(f"{path}:{line} 是整文件的内层属性或后面没有函数，不是登记的那几处",
                       [], f"放开只许是 #[allow(unsafe_code)] 紧挨着登记的函数、方法或 impl 块（{registered_items or '这个根没有登记放开'}）；要换地方改 {REGISTRY_PATH} 的登记并走三方。")
            elif (path, function) not in registered_pairs:
                registered_text = f"登记的是 {registered_items}" if has_allowance else f"{label} 在登记表里没有放开"
                reject(f"{path}:{line} 放开 unsafe_code 的函数是 {function}，{registered_text}",
                       [], f"放开只许在登记的那几处；要新增或换地方改 {REGISTRY_PATH} 的登记并走三方（方法写 <类型>::<方法>，impl 块写 impl <Trait> for <类型>）。")
        if has_allowance:
            for (path, line, position), holder in zip(unsafes, holders):
                if holder is None:
                    reject(f"{path}:{line} 的 unsafe 不在登记的 {registered_items} 函数体里", [],
                           f"unsafe 只许写在登记的那几项里（项从项头算起，unsafe fn 签名与 unsafe impl 项头都在里头）；要换地方改 {REGISTRY_PATH} 的登记并走三方。")
    if allowance_in_force and BREAK not in ("count-ignored", "placement-ignored"):
        for row in allowance_rows:
            if BREAK != "per-row-allows-ignored":
                beside = [f"{path}:{line}" for path, line, function in relaxes if (path, function) == (row["file"], row["function"])]
                if len(beside) != 1:
                    reject(f"{REGISTRY_PATH}:{row['number']} 登记的 {row['file']} 的 {row['function']} 上放开 unsafe_code 的属性 {len(beside)} 处，应当恰好 1 处",
                           beside or ["（没有）"],
                           "登记的每个函数上恰好写一处 #[allow(unsafe_code, reason = \"…\")]：多出的删掉，缺的补上（reason 照登记表这一行第 7 段）。")
            if BREAK != "per-function-count-ignored":
                inside = [f"{path}:{line}" for (path, line, position), holder in zip(unsafes, holders) if holder is row]
                if len(inside) != row["count"]:
                    reject(f"{REGISTRY_PATH}:{row['number']} 登记的 {row['file']} 的 {row['function']} 里 unsafe 关键字 {len(inside)} 处，登记表这一行第 5 段写的是 {row['count']} 处",
                           inside or ["（没有）"],
                           f"函数里多出的 unsafe 删掉；确要加减，改 {REGISTRY_PATH} 这一行的第 5 段并走三方。")
    judged_rows = set()
    for (path, line, position), holder in zip(unsafes, holders):
        function_already_judged = holder is not None and holder["number"] in judged_rows
        if holder is not None:
            judged_rows.add(holder["number"])
        if path not in skipped_by_path:
            skipped_by_path[path] = attribute_lines(files[path][1])
        if path not in comment_lines_by_path:
            # 弄坏开关 safety-read-from-raw-lines 下读原文（字符串里顶格的 // SAFETY: 也当注释）；
            # lines-split-at-unicode-separators 下按 splitlines() 切（U+2028 这类也算换行，与 line_of 按 \n 数的行号错开）。
            text = files[path][0] if BREAK == "safety-read-from-raw-lines" else strip_comments_and_strings(files[path][0], keep_line_comments=True)
            comment_lines_by_path[path] = text.splitlines() if BREAK == "lines-split-at-unicode-separators" else text.split("\n")
        comment = safety_comment(comment_lines_by_path[path], files[path][1].split("\n"), line, skipped_by_path[path])
        if comment is None:
            if BREAK != "safety-comment-ignored" and not (BREAK == "safety-one-place-per-function" and function_already_judged):
                reject(f"{path}:{line} 的 unsafe 上方没有 // SAFETY: 注释", [],
                       "在 unsafe 上方连着写 // SAFETY: 起头的注释，说清调用的前提为什么成立；语句折成几行的，写在语句开头那一行上方，中间不隔别的语句；"
                       "unsafe fn 与 unsafe impl 写在项头上方，中间隔着 #[allow(unsafe_code, …)] 这类独占几行的属性照认。")
        elif holder is not None and BREAK != "safety-premise-ignored" and not (BREAK == "words-one-place-per-function" and function_already_judged):
            words = HOLE_PUNCHING_WORDS if BREAK == "registered-words-ignored" else holder["words"]
            absent = [word for word in words if not safety_word_pattern(word).search(comment)]
            if absent:
                reject(f"{path}:{line} 的 // SAFETY: 注释没写出它依赖的前提：没提到{'、'.join(absent)}", comment.splitlines(),
                       f"SAFETY 注释要写出调用依赖的前提，{REGISTRY_PATH}:{holder['number']} 第 6 段登记的词（{'、'.join(holder['words'])}）这个函数里每一处都要提到；"
                       "这一格只查这几个词有没有出现，写得对不对靠 review。")


def read_source(path):
    original = open(path, encoding="utf-8", errors="replace").read()
    return original, strip_comments_and_strings(original)


def referenced_files(path, original, cleaned):
    """path 这份文件里 include!("…") 与 #[path = "…"] 点到的文件（相对仓根、规范化过），都按 path 所在的目录解析；
    内联 mod 块里的 #[path] 照同一个目录解析（cargo 在那里还要拼上模块路径，这一格不拼）。"""
    targets = []
    for match in FILE_REFERENCE.finditer(cleaned):
        closing = cleaned.find('"', match.end())
        if closing >= 0:
            targets.append(os.path.normpath(os.path.join(os.path.dirname(path), original[match.end():closing])))
    return targets


rust_files_by_crate = {}
for name in crate_names:
    found = []
    for directory, subdirectories, names in os.walk(os.path.join("crates", name)):
        subdirectories[:] = sorted(entry for entry in subdirectories if entry != "target" and not entry.startswith("."))
        found += [os.path.join(directory, file_name) for file_name in sorted(names) if file_name.endswith(".rs")]
    rust_files_by_crate[name] = found
scanned_by_some_crate = {path for found in rust_files_by_crate.values() for path in found}
included_count = 0

for name in crate_names:
    if name not in registry:
        reject(f"crates/{name} 没在登记表里", [f"{REGISTRY_PATH} 没有 {name} 这一行"],
               f"新 crate 先在 {REGISTRY_PATH} 登记一行（根要写 forbid；要放开 unsafe 的走三方、级别写 forbid,deny，放开的每个函数一行，写明文件、函数、unsafe 几处与 SAFETY 注释必须提到的词）。")
        continue
    crate_roots = {root_name: rows for root_name, rows in registry.items() if crate_of(root_name) == name}
    if BREAK == "other-crates-ignored" and not any(row["file"] != "-" for rows in crate_roots.values() for row in rows.values()):
        continue
    crate_directory = os.path.join("crates", name)
    files = {path: read_source(path) for path in rust_files_by_crate[name]}
    file_count += len(files)
    crate_count += 1
    bin_roots = bin_roots_of(sorted(os.path.relpath(path, crate_directory) for path in files))
    # 每份文件归哪个根（库根是 None）；include! 与 #[path] 点到的文件归写着它的那份文件的根，一并扫：
    # 点到的是哪个 crate 都扫的 .rs 不再扫第二遍，点到而不在的跳过（编译过不了的写法归编译器）。弄坏开关 included-files-unscanned 下一份都不追（只扫 .rs）。
    owner = {os.path.join(crate_directory, member): root for root, members in bin_roots.items() for member in members}
    pending = [] if BREAK == "included-files-unscanned" else sorted(files)
    while pending:
        including = pending.pop(0)
        for target in referenced_files(including, *files[including]):
            if (target in files or target in scanned_by_some_crate or os.path.isabs(target)
                    or target.split(os.sep)[0] == ".." or not os.path.isfile(target)):
                continue
            files[target] = read_source(target)
            owner[target] = owner.get(including)
            included_count += 1
            pending.append(target)
    for root_name in sorted(crate_roots):
        if root_name != name and root_name[len(name) + 1:] not in bin_roots and BREAK != "registered-bin-root-unchecked":
            numbers = "、".join(str(row["number"]) for row in crate_roots[root_name].values())
            reject(f"{REGISTRY_PATH}:{numbers} 登记的根 {root_name} 不是 crates/{name}/ 下的 bin 根",
                   [f"crates/{name}/ 下的 bin 根：{'、'.join(sorted(bin_roots)) or '（没有）'}"],
                   "第 1 段写 bin 根时写 <crate 目录名>/src/main.rs、<crate 目录名>/src/bin/<名>.rs 或 <crate 目录名>/src/bin/<名>/main.rs，那个文件要在；"
                   "bin 根改了名或删了，改这几行或删掉。")
    judge_root(name, name, os.path.join(crate_directory, "src", "lib.rs"),
               {path: texts for path, texts in files.items() if owner.get(path) is None}, crate_roots[name], True)
    for root in sorted(bin_roots):
        root_name = f"{name}/{root}"
        if root_name not in crate_roots and BREAK == "unregistered-bin-roots-ignored":
            continue
        root_files = {path: texts for path, texts in files.items() if owner.get(path) == root}
        judge_root(root_name, name, os.path.join(crate_directory, root), root_files, crate_roots.get(root_name, {}), root_name in crate_roots)

finish()
missing = sorted({crate_of(root_name) for root_name in registry} - set(crate_names))
row_total = sum(len(rows) for rows in registry.values())
relax_total = sum(entry[5] for entry in scanned)
unsafe_total = sum(entry[6] for entry in scanned)
bin_entries = [entry for entry in scanned if entry[0] != entry[2]]
pieces = []
for crate_name in dict.fromkeys(entry[2] for entry in scanned):
    entries = [entry for entry in scanned if entry[2] == crate_name]
    pieces += [f"{label} 根 {level}、放开 {relaxes} 处、unsafe {unsafes} 处、{count} 个 .rs、登记 {rows} 行"
               for label, unregistered, _crate, count, level, relaxes, unsafes, rows in entries if not unregistered]
    unregistered_entries = [entry for entry in entries if entry[1]]
    if unregistered_entries:
        pieces.append(f"{crate_name} 没登记的 bin 根 {len(unregistered_entries)} 个（照 forbid 判）、放开 {sum(entry[5] for entry in unregistered_entries)} 处、"
                      f"unsafe {sum(entry[6] for entry in unregistered_entries)} 处、{sum(entry[3] for entry in unregistered_entries)} 个 .rs")
summary = "；".join(pieces)
print(f"  ✓ crates/ 里放开 unsafe 的地方与登记表对上：扫了 {crate_count} 个 crate、{file_count} 个 .rs、登记表 {row_total} 行、放开 {relax_total} 处、unsafe {unsafe_total} 处、"
      f"bin 根 {len(bin_entries)} 个（登记了的 {sum(1 for entry in bin_entries if not entry[1])} 个）（{summary}）；登记了但仓里没有的 crate {len(missing)} 个：{'、'.join(missing) or '无'}；"
      f"include! 与 #[path] 点到、没有哪个 crate 当 .rs 扫的文件另扫了 {included_count} 份（算进写着它的那份文件的根，上面各根的个数里含着）")
PY
}

# ── 格 gpu-exemptions：不走 GPU 的计算与豁免表逐项对上（欠账 C626（走 GPU 的计算类没有登记））
# 判法只在 scripts/crash/gpu-exemptions-check.py check：豁免表 .claude/gate.d/gpu-exemptions.tsv 每一行四段（截名、代码标记名、为什么上不了 GPU、
# 用户点头日期），crates/ 下 .rs 里每一处 // gpu-exempt: <截名> 都有表行、每一行都有标记。被判的根里没有 crates/：这一格本次无对象可判。
# 管不到的：一截计算真在不在 GPU 上跑、标记写没写在真不走 GPU 的那一截上、为什么写得对不对，靠 review 与用户点头。
cell_gpu_exemptions() {
  if [[ ! -d "$ROOT/crates" ]]; then
    echo "  ! 没有 crates/：豁免表这一格本次无对象可判"
    return 77
  fi
  python3 "$STAGE_REPOSITORY/scripts/crash/gpu-exemptions-check.py" check "$ROOT"
}

# ── 格 crash-point-replay-segments：崩溃点重放的按段登记成形
# 判法只在 scripts/crash/lib_crash_point_replay_items.py segments（两个入口 scripts/crash/crash-point-replay.sh、scripts/crash/crash-amplification.sh 开跑前调同一份）：
# .claude/gate.d/crash-point-replay-segments.tsv 每一行五段（项名、流名、段号、段写数、段写表摘要），项名在合表 .claude/gate.d/crash-point-replay-items.tsv 里，
# 流名匹配 ^[A-Za-z][A-Za-z0-9_]*$（引擎按它取这条流的行），段号与段写数是非负整数且段写数大于 0，摘要是 64 位小写十六进制，同一（流名，段号）只一行。
# 被判的根里合表与按段登记文件都没有：这一格本次无对象可判；合表在而按段登记文件不在判红（入口开跑前要读它）。
# 管不到的：摘要是不是那一段写表真的 sha256、段号与段写数对不对得上那条流（引擎跑起来才知道），靠引擎那一侧与 review。
cell_crash_point_replay_segments() {
  if [[ ! -f "$ROOT/.claude/gate.d/crash-point-replay-segments.tsv" && ! -f "$ROOT/.claude/gate.d/crash-point-replay-items.tsv" ]]; then
    echo "  ! 没有崩溃点重放的合表与按段登记文件：这一格本次无对象可判"
    return 77
  fi
  python3 "$STAGE_REPOSITORY/scripts/crash/lib_crash_point_replay_items.py" segments "$ROOT"
}


# ── 格 crate-sources-reachable：没有 cargo 不编的落空源文件
# 判据与射程在 scripts/gate/crate-sources-reachable.py 文件头：scripts/experiment-apparatus 与 crates 下每一份 .rs，要么是某个 crate 的目标
# （[lib]、[[bin]]、[[test]]、自动发现的那几类），要么从目标经 #[path]、mod、include! 引得到，include_str!、include_bytes! 当文本读的也算。
# 管不到的：引到了却是死代码（cfg 永远不开的模块）、引它的那个目标自己从来不跑。
# red：一份平铺 crate 里多一份谁都没引的 .rs；green：同一份 crate 里每份都登记或被引。
cell_crate_sources_reachable() {
  local directories=() directory
  for directory in "$ROOT/scripts/experiment-apparatus" "$ROOT/crates"; do
    [[ -d "$directory" ]] && directories+=("$directory")
  done
  if ((${#directories[@]} == 0)); then
    echo "  ! 本次无对象可判：$ROOT 下没有 scripts/experiment-apparatus/ 与 crates/"
    return 77
  fi
  python3 "$STAGE_REPOSITORY/scripts/gate/crate-sources-reachable.py" "${directories[@]}"
}

# ── 格 ordinal-names：代码里我们自己的名字不带序号与位置词（records/2026-10-13-复用键与全仓改名.md）
# 判据：git 列出的 crates/、scripts/、.claude/gate.d、.claude/hooks 下每一份 .rs、.sh、.py（这几处里的样本也算），下面三样名字按词边界含了
#   词表 .claude/naming-ordinal-tokens 里的一个词，判违规，逐处列「文件:行 种类 名字（含 词）」：
#   ① 文件名与它上面每一级目录名（目录名的违规记在那一级目录的路径上，同一个目录只报一次）；
#   ② 源码里的名字：.rs 把注释与字符串字面量（含原始串、字节串、字符字面量）抹成空白之后的每一个标识符（函数、变量、常量、模块、类型、字段、
#      测试、生命周期都在内；同一份文件同一个名字只报第一处）；.sh 的函数名、赋值（含 local / export / declare / readonly 与命令前的 X=…）、
#      for / read 的变量与 $ 展开里的变量名（# 注释与单引号串去掉，双引号串里只留 $ 展开，引号可以跨行）；.py 经 tokenize 出来的每一个标识符
#      （关键字除外）；.sh 里起 python 的 heredoc 正文按 .py 判，别的 heredoc 正文当数据，只认界符不带引号时里面的 $ 展开；
#   ③ 环境变量名：.rs 里 env::var / var_os / env! / option_env! / set_var / remove_var / .env(…) 的字符串参数，.py 里 os.environ.get / [] / setdefault / pop、
#      getenv / putenv / unsetenv 的字符串参数（.sh 里的环境变量就是变量，归 ②）。
# 词边界：名字与词都先摊平——驼峰拆开（FooBar 拆成 foo、bar 两段）、大小写不分、字母数字之外的字符（_、-、.、/）一律当边界——
#   词前后紧挨的不是字母或数字就算命中，复合名里夹着的也算；词后面紧跟一位数字的不算（那是另一个号）。
# 判法是纯函数（ordinal_tokens_in 输入一个名字、输出命中的词；names_in 输入一份源文件的路径与文本、输出它的名字表），不起 cargo、不编译。
# 词表一行一个小写词（字母起头，只含字母、数字与 _），# 后写为什么它是序号或位置词；写坏的行、同一个词登两行、没写理由的行判红，词表不在判红。
# 还没改完的逐处登记在 .claude/naming-ordinal-exclude（一行 <路径>  # 理由；路径是文件，或目录名本身带词的那一级目录），登记的那一处的违规不判；
#   排除项指向不存在的路径、那一处已经一个违规都没有了、或没写理由，判红（排除只缩不涨）。排除表不在等于一条都没排。
# tokenize 认不出的 .py（与 .sh 里认不出的 python heredoc）逐个列在「扫了…」那一行，不算判过；一份 .rs / .sh / .py 都没列出来：这一格本次无对象可判。
# 管不到的：注释、文档与输出文字里的序数与位置代称（归 doc-text 的 term-renames 格，只认 .claude/kb/term-renames.md 登记过的旧名，没登记的靠 review）；
#   词表外的写法（别的序数词、位置词，要判就往词表里加）；词在名字里其实是别的意思（说「流里最早那一个下标」的名字也照判，改名或登记排除）；
#   .sh 里命令的参数与数组下标、Rust 宏展开出来的名字。
# red：一份 .rs 带 enumerate_ 接层序号的函数、一份 .sh 给带流序号的变量赋值，排除表一行指向已经改好名的文件、一行指向不存在的文件；
# green：同样的词只出现在 .rs / .sh / .py 的注释与字符串里、词后面紧跟一位数字的名字、登记在排除表里还没改完的那一份 .rs。
cell_ordinal_names() {
python3 - <<'PY'
import bisect, io, keyword, os, re, subprocess, sys, tokenize

TOKENS_FILE = ".claude/naming-ordinal-tokens"
EXCLUDE_FILE = ".claude/naming-ordinal-exclude"
SCANNED_DIRECTORIES = ("crates", "scripts", ".claude/gate.d", ".claude/hooks")
SUFFIXES = (".rs", ".sh", ".py")
BROKEN = os.environ.get("GATE_ORDINAL_NAMES_BREAK", "")


def registry(path):
    """一行一条 `<条目>  # 理由` 的登记表：[(行号, 条目, 理由)]；文件不在返回 None。"""
    if not os.path.isfile(path):
        return None
    entries = []
    for number, line in enumerate(open(path, encoding="utf-8"), 1):
        body = line.split("#", 1)[0].strip()
        if body:
            entries.append((number, body, line.split("#", 1)[1].strip() if "#" in line else ""))
    return entries


def normalized_words(name):
    """名字按词边界摊平：驼峰拆开、全部小写、字母数字之外的一律当成 `_`。"""
    name = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", name)
    name = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", name)
    return re.sub(r"[^a-z0-9]+", "_", name.lower()).strip("_")


def token_patterns(tokens):
    patterns = []
    for token in tokens:
        flat = re.escape(normalized_words(token))
        if BROKEN == "compound-names-unseen":
            patterns.append((token, re.compile("^" + flat + "$")))
        else:
            patterns.append((token, re.compile(r"(?<![a-z0-9])" + flat + r"(?![a-z0-9])")))
    return patterns


def ordinal_tokens_in(name, patterns):
    """纯函数：名字里按词边界含了词表里的哪几个词（次序照词表）。"""
    flat = normalized_words(name)
    return [token for token, pattern in patterns if pattern.search(flat)]


# ── 各语言里我们自己的名字：(行号, 种类, 名字) ──

RUST_ENVIRONMENT = re.compile(r"(?:\benv::var(?:_os)?|\bvar(?:_os)?|\benv!|\boption_env!|\bset_var|\bremove_var|\.env|\.env_remove)\s*\(\s*$")
RUST_LEXEME_START = re.compile(r"//|/\*|(?<![A-Za-z0-9_])b?r#*\"|(?<![A-Za-z0-9_])b\"|\"|'")
RUST_BLOCK_COMMENT_EDGE = re.compile(r"/\*|\*/")
RUST_STRING_BODY = re.compile(r"(?:\\.|[^\"\\])*\"", re.DOTALL)
RUST_CHARACTER_LITERAL = re.compile(r"'(?:\\(?:u\{[0-9A-Fa-f]+\}|x[0-9A-Fa-f]{2}|.)|[^\\'\n])'")
IDENTIFIER = re.compile(r"(?<![A-Za-z0-9_])(?:r#)?([A-Za-z_][A-Za-z0-9_]*)")


def line_of(line_starts, offset):
    return bisect.bisect_right(line_starts, offset)


def rust_names(text):
    """.rs：注释与字符串字面量抹成空白后的每一个标识符（含生命周期与标签），加读写环境变量处字符串里的变量名。"""
    names, code, index, length = [], [], 0, len(text)
    line_starts = [0] + [match.end() for match in re.finditer("\n", text)]
    blank = lambda piece: re.sub(r"[^\n]", " ", piece)
    while index < length:
        start = RUST_LEXEME_START.search(text, index)
        if not start:
            code.append(text[index:]); break
        code.append(text[index:start.start()])
        lexeme, index = start.group(0), start.start()
        if lexeme == "//":
            end = text.find("\n", index)
            end = length if end < 0 else end
        elif lexeme == "/*":
            depth, end = 1, index + 2
            while depth:
                edge = RUST_BLOCK_COMMENT_EDGE.search(text, end)
                if not edge:
                    end = length; break
                depth += 1 if edge.group(0) == "/*" else -1
                end = edge.end()
        elif lexeme == "'":
            literal = RUST_CHARACTER_LITERAL.match(text, index)
            if not literal:
                code.append("'"); index += 1; continue
            end = literal.end()
        else:
            opening = start.end()
            hashes = lexeme.count("#") if "r" in lexeme else None
            if hashes is None:
                body = RUST_STRING_BODY.match(text, opening)
                end = body.end() if body else length
            else:
                closing = text.find('"' + "#" * hashes, opening)
                end = length if closing < 0 else closing + 1 + hashes
            content = text[opening:end - 1 - (hashes or 0)]
            if RUST_ENVIRONMENT.search("".join(code[-8:])[-40:]) and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", content):
                names.append((line_of(line_starts, index), "环境变量", content))
        if BROKEN == "comments-and-strings-judged":
            code.append(text[index:end])
        else:
            code.append(blank(text[index:end]))
        index = end
    stripped = "".join(code)
    seen = set()
    for match in IDENTIFIER.finditer(stripped):
        name = match.group(1)
        if name in seen:
            continue
        seen.add(name)
        names.append((line_of(line_starts, match.start()), "名字", name))
    return names


SHELL_HEREDOC = re.compile(r"<<(-?)\s*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\2")
SHELL_ASSIGNMENT = re.compile(r"(?:^|[\s;&|(`{!])([A-Za-z_][A-Za-z0-9_]*)(?:\[[^\]]*\])?\+?=")
SHELL_FUNCTION = re.compile(r"^\s*(?:function\s+([A-Za-z_][A-Za-z0-9_:.-]*)|([A-Za-z_][A-Za-z0-9_:.-]*)\s*\(\s*\))")
SHELL_EXPANSION = re.compile(r"\$\{?#?!?([A-Za-z_][A-Za-z0-9_]*)")
SHELL_FOR = re.compile(r"(?:^|[\s;&|(])(?:for|select)\s+([A-Za-z_][A-Za-z0-9_]*)\s+in\b")
SHELL_READ = re.compile(r"(?:^|[\s;&|(])read((?:\s+[^\s;&|<>)]+)+)")


def read_variables(arguments):
    """read 的参数里哪几个是变量名：-a 后面那一个是数组名，-d -i -n -N -p -t -u 后面那一个是选项值。"""
    words, names, index = arguments.split(), [], 0
    while index < len(words):
        word = words[index]
        if word.startswith("-"):
            if word[-1] in "dinNptu":
                index += 1
            elif word[-1] == "a" and index + 1 < len(words):
                names.append(words[index + 1]); index += 1
        elif re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", word):
            names.append(word)
        index += 1
    return names


def shell_names(text, unjudged):
    """.sh：函数名、赋值（含 local / export / declare / readonly 与命令前的 X=…）的变量名、for / read 的变量、$ 展开里的变量名。
    逐字扫一遍：# 注释去掉，单引号串（与 $'…'）抹成空白，双引号串里只留 $ 展开，引号可以跨行；
    heredoc 起头那一行起 python 的，heredoc 正文按 python 判；别的 heredoc 正文是数据，只认界符不带引号时里面的 $ 展开。"""
    names, lines, number, quote = [], text.split("\n"), 0, None
    while number < len(lines):
        line = lines[number]
        number += 1
        out, index, heredocs = [], 0, []
        while index < len(line):
            character = line[index]
            if BROKEN == "comments-and-strings-judged" and quote is None:
                heredoc = SHELL_HEREDOC.match(line, index) if line.startswith("<<", index) and not line.startswith("<<<", index) else None
                if heredoc:
                    heredocs.append((heredoc.group(3), bool(heredoc.group(2)), heredoc.group(1) == "-"))
                    index = heredoc.end(); continue
                out.append(character); index += 1; continue
            if quote is None:
                if character == "#" and (index == 0 or line[index - 1] in " \t;|&("):
                    break
                if character == "\\":
                    out.append(line[index:index + 2]); index += 2; continue
                if line.startswith("<<", index) and not line.startswith("<<<", index):
                    heredoc = SHELL_HEREDOC.match(line, index)
                    if heredoc:
                        heredocs.append((heredoc.group(3), bool(heredoc.group(2)), heredoc.group(1) == "-"))
                        out.append(" " * (heredoc.end() - index)); index = heredoc.end(); continue
                if line.startswith("$'", index):
                    quote = "$'"; out.append("@ "); index += 2; continue
                if character in "'\"":
                    quote = character; out.append("@"); index += 1; continue
                out.append(character)
            elif quote in ("'", "$'"):
                if quote == "$'" and character == "\\":
                    out.append("  "); index += 2; continue
                if character == "'":
                    quote = None
                out.append(" ")
            else:
                if character == "\\":
                    out.append("  "); index += 2; continue
                if character == '"':
                    quote = None; out.append(" ")
                elif character == "$":
                    expansion = SHELL_EXPANSION.match(line, index)
                    if expansion:
                        out.append(" " + expansion.group(0)); index = expansion.end(); continue
                    out.append(" ")
                else:
                    out.append(" ")
            index += 1
        code = "".join(out)
        for match in SHELL_FUNCTION.finditer(code):
            names.append((number, "函数", match.group(1) or match.group(2)))
        for match in SHELL_ASSIGNMENT.finditer(code):
            names.append((number, "变量", match.group(1)))
        for match in SHELL_FOR.finditer(code):
            names.append((number, "变量", match.group(1)))
        for match in SHELL_READ.finditer(code):
            names.extend((number, "变量", name) for name in read_variables(match.group(1)))
        for match in SHELL_EXPANSION.finditer(code):
            names.append((number, "变量", match.group(1)))
        for delimiter, quoted, strip_tabs in heredocs:
            body_start = number
            while number < len(lines) and (lines[number].lstrip("\t") if strip_tabs else lines[number]).rstrip() != delimiter:
                number += 1
            body = lines[body_start:number]
            number += 1
            if re.search(r"(?<![A-Za-z0-9_./-])python3?(?![A-Za-z0-9_.-])", line):
                inner_names = python_names("\n".join(body))
                if inner_names is None:
                    unjudged.append(f"第 {body_start} 行起的 python heredoc")
                    continue
                names.extend((body_start + inner, kind, name) for inner, kind, name in inner_names)
            elif not quoted:
                for offset, body_line in enumerate(body, 1):
                    for match in SHELL_EXPANSION.finditer(body_line):
                        names.append((body_start + offset, "变量", match.group(1)))
    return names


PYTHON_ENVIRONMENT_CALLS = ("environ.get", "environ.setdefault", "environ.pop", "getenv", "putenv", "unsetenv", "environ[")


def python_names(text):
    """.py：tokenize 出来的每一个标识符（关键字除外，f 串花括号里的表达式也算），加读写环境变量处字符串里的变量名；认不出返回 None。"""
    names, previous, seen = [], [], set()
    try:
        tokens = list(tokenize.generate_tokens(io.StringIO(text).readline))
    except (tokenize.TokenError, IndentationError, SyntaxError):
        return None
    for token in tokens:
        if token.type == tokenize.NAME and not keyword.iskeyword(token.string) and token.string not in seen:
            seen.add(token.string)
            names.append((token.start[0], "名字", token.string))
        elif token.type == tokenize.STRING:
            tail = "".join(previous[-4:])
            literal = re.fullmatch(r"[rRbBuU]?(['\"])([A-Za-z_][A-Za-z0-9_]*)\1", token.string)
            if literal and any(tail.endswith(call) or tail.endswith(call + "(") for call in PYTHON_ENVIRONMENT_CALLS):
                names.append((token.start[0], "环境变量", literal.group(2)))
        if BROKEN == "comments-and-strings-judged" and token.type in (tokenize.STRING, tokenize.COMMENT):
            for word in re.findall(r"[A-Za-z_][A-Za-z0-9_]*", token.string):
                names.append((token.start[0], "名字", word))
        if token.type not in (tokenize.NL, tokenize.NEWLINE, tokenize.COMMENT, tokenize.INDENT, tokenize.DEDENT):
            previous.append(token.string)
    return names


def names_in(path, text, unjudged):
    """纯函数：一份源文件里我们自己的名字 [(行号, 种类, 名字)]；认不出的部分记进 unjudged（逐个列出，不算判过）。"""
    if path.endswith(".rs"):
        return rust_names(text)
    if path.endswith(".sh"):
        if BROKEN == "shell-unscanned":
            return []
        inner_unjudged = []
        names = shell_names(text, inner_unjudged)
        unjudged.extend(f"{path} {part}" for part in inner_unjudged)
        return names
    names = python_names(text)
    if names is None:
        unjudged.append(path)
        return []
    return names


def main():
    tokens = registry(TOKENS_FILE)
    if tokens is None:
        print(f"  ✗ 读不到序号与位置词表 {TOKENS_FILE}")
        print("     → 怎么办：从 git 恢复它；这一格没有词表就什么都判不了，不是通过。")
        return 1
    table_problems = []
    seen_tokens = {}
    for number, token, reason in tokens:
        if not re.fullmatch(r"[a-z][a-z0-9_]*", token):
            table_problems.append(f"{TOKENS_FILE}:{number}「{token}」不是小写字母起头、只含小写字母、数字与 _ 的词")
        elif normalized_words(token) in seen_tokens:
            table_problems.append(f"{TOKENS_FILE}:{number}「{token}」与第 {seen_tokens[normalized_words(token)]} 行登记的是同一个词")
        else:
            seen_tokens[normalized_words(token)] = number
        if len(reason) < 4:
            table_problems.append(f"{TOKENS_FILE}:{number}「{token}」没写为什么算序号或位置词（# 后面至少 4 个字）")
    patterns = token_patterns([token for _number, token, _reason in tokens])
    excluded = registry(EXCLUDE_FILE) or []

    listed = subprocess.run(["git", "-c", "core.quotepath=false", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", *SCANNED_DIRECTORIES],
                            capture_output=True)
    if listed.returncode != 0:
        print(f"  ✗ git ls-files 退 {listed.returncode}：{listed.stderr.decode('utf-8', 'replace').strip()}")
        print("     → 怎么办：在 git 仓的根上跑这一道；列不出文件就判不了，不是通过。")
        return 1
    paths = sorted(path for path in listed.stdout.decode("utf-8", "replace").split("\0") if path.endswith(SUFFIXES) and os.path.isfile(path))
    if not paths:
        print("  ! 本次无对象可判：" + "、".join(SCANNED_DIRECTORIES) + " 下一份 .rs、.sh、.py 都没列出来")
        return 77

    violations = {}  # 违规记在哪一处（文件或目录的路径） -> [行]
    unparsed = []
    directories_seen = set()
    for path in paths:
        parts = path.split("/")
        for depth in range(1, len(parts)):
            directory = "/".join(parts[:depth])
            if directory in directories_seen:
                continue
            directories_seen.add(directory)
            hit = ordinal_tokens_in(parts[depth - 1], patterns)
            if hit:
                violations.setdefault(directory, []).append(f"{directory}/ 目录名「{parts[depth - 1]}」（含 {'、'.join(hit)}）")
        hit = ordinal_tokens_in(parts[-1], patterns)
        if hit:
            violations.setdefault(path, []).append(f"{path} 文件名（含 {'、'.join(hit)}）")
        text = open(path, encoding="utf-8", errors="replace").read()
        reported = set()
        for number, kind, name in names_in(path, text, unparsed):
            if (kind, name) in reported:
                continue
            hit = ordinal_tokens_in(name, patterns)
            if hit:
                reported.add((kind, name))
                violations.setdefault(path, []).append(f"{path}:{number} {kind} {name}（含 {'、'.join(hit)}）")

    excluded_paths = {entry for _number, entry, _reason in excluded}
    problems, exclusion_problems = [], []
    for owner in sorted(violations):
        if owner in excluded_paths and BROKEN != "exclusions-ignored":
            continue
        problems += violations[owner]
    for number, entry, reason in excluded:
        if not os.path.exists(entry):
            exclusion_problems.append(f"{EXCLUDE_FILE}:{number} 指向不存在的路径 {entry}")
        elif entry not in violations:
            exclusion_problems.append(f"{EXCLUDE_FILE}:{number} {entry} 里已经一个带序号或位置词的名字都没有了，这一行删掉")
        elif len(reason) < 4:
            exclusion_problems.append(f"{EXCLUDE_FILE}:{number} {entry} 没写为什么先不改（# 后面至少 4 个字）")

    unparsed_note = "无" if not unparsed else "、".join(unparsed)
    print(f"  · 扫了 {len(paths)} 份 .rs / .sh / .py 与它们所在的 {len(directories_seen)} 个目录，词表 {len(patterns)} 个词，排除 {len(excluded)} 条；"
          f"tokenize 认不出、没判的 {len(unparsed)} 处：{unparsed_note}")
    for problem in table_problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    if table_problems:
        print(f"  ✗ {TOKENS_FILE} 有 {len(table_problems)} 行写坏了")  # gate-lint:summary
        print("     → 怎么办：一行一个小写词（字母起头，只含字母、数字与 _），同一个词只登一行，# 后面写为什么它是序号或位置词。")
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    if problems:
        print(f"  ✗ {len(problems)} 处名字带序号或位置词（词表 {TOKENS_FILE}，按词边界认，复合名也算）")  # gate-lint:summary
        print("     → 怎么办：按 .claude/agent-common.md「写」一节「名字、注释、文档、输出文字都不用序数或位置代称」改成说内容的名字（例 enumerate_crash_states、new_pool_file_creation_stream），")
        print("                改过的名字登记进 .claude/kb/term-renames.md；一时改不完的文件登记进 " + EXCLUDE_FILE + "（一行 <路径>  # 理由），改完删那一行。")
    for problem in exclusion_problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    if exclusion_problems:
        print(f"  ✗ {EXCLUDE_FILE} 有 {len(exclusion_problems)} 行不起作用或没写理由")  # gate-lint:summary
        print("     → 怎么办：指向不存在的与已经改完的那几行删掉；留着的每一行写明为什么先不改。排除只缩不涨。")
    if table_problems or problems or exclusion_problems:
        return 1
    print(f"  ✓ 名字里没有序号与位置词（扫了 {len(paths)} 份 .rs / .sh / .py、{len(directories_seen)} 个目录，词表 {len(patterns)} 个词，排除 {len(excluded)} 条）")
    return 0


sys.exit(main())
PY
}

# ── 格 data-names：以数据形式存在的名字也按命名纪律判（欠账 C430）
# 判法只在 scripts/gate/data-name-lint.py：naming-lint 只查我们声明的名字，而字符串字面量与 kb 表格里还活着一批人真正读到的名字——
#   scripts/experiment-apparatus/*.rs 与 crates/*/src/**/*.rs 里的元组首列 ("名字", …)、=> "名字" 右侧、格式串里 键= 的键名，
#   .claude/kb/**/*.md 表格单元格里反引号包着的名字（只认整串是 [a-z][a-z0-9_]*）。
# 按同一套判：_ 切词，单字母判红，词在 naming-lint.sh 的缩写词表里（脚本从那份读，不另抄一份）或在脚本的补充缩写里（mac）而没登记进 .claude/abbreviations 判红；
#   e<数字>、c<数字> 这类整类登记的编号放行。已经落进留存产物、改名要连产物一起重跑的存量逐条登记在 .claude/data-name-lint-exclude
#   （一行 <路径>:<串>  # 理由；指向不存在的路径、已经一个违规都没有了的行、没写理由的行判红，排除只缩不涨）。
# 成功句报：扫了几份文件、几个串，排除表放行了几条。扫不到一份 .rs 与 kb：这一格本次无对象可判（退 77）。
# 管不到的：不是这三种形态的名字（结构体字段赋的字符串、函数调用的实参、拼出来的名字），词表之外的缩写，名字起得好不好，靠 review。
# red：一份 .rs 里元组首列、=> 右侧、键= 各一个带缩写或单字母的名字，一份 kb 表格里一个带缩写的单元格，排除表一行指向已经改好名的串、一行指向不存在的文件；
# green：样本自己的 .claude/abbreviations 登记了缩写与整类编号，同样的词只出现在注释、函数调用实参、围栏代码块与表格外的正文里，排除表登记的那个串还在。
cell_data_names() {
  python3 "$STAGE_REPOSITORY/scripts/gate/data-name-lint.py" "$ROOT"
}

# ── 格 verus-ghost-names：被验 crate 里不自己定义名叫 Ghost / Tracked 的类型，也不把别的东西改名成这两个名字（records/2026-10-29-subagent拆分提案.md 第一百三十三节，取甲）
# 为什么：函数单元图的两份实现（crates/singlefs-unit-graph/src/verus_ghost_code.rs 与 scripts/gate/lib_function_units.py 的 VERUS_GHOST_WRAPPERS）按类型名认 Ghost / Tracked，
#   不分 vstd 的那两个与同名的自定义类型；被验 crate 里出现同名的自定义类型，单元图就把真代码当幽灵代码剔出键。两份实现不动，在源码这一侧拒同名。
# 对象：被判的根里 VERUS_PINS_TABLE（.claude/gate.d/verus-pins.tsv）每一行「crate<TAB><目录>…」登记的 crate，<目录>/src/ 下全部 .rs（去掉注释、字符串与字符字面量之后判，行号照原文）。
# 判据，判红的两样：
#   ① 类型定义：struct、enum、union、type、trait 后面紧跟的名字是 Ghost 或 Tracked（被验 crate 自己定义的，都不来自 vstd）；
#   ② 改名：use 语句里「<路径> as Ghost」「<路径> as Tracked」（花括号分组里的按分组前缀补全路径），路径不是 vstd:: 起头、以同名结尾的那一种（use vstd::prelude::Ghost as Ghost 放过）。
#   登记表不在、或一行 crate 都没有：退 77；登记的目录不在：红，点名那一行。
# 管不到的：宏展开出来的定义与改名；extern crate vstd as … 这类把整个 vstd 改名之后再取 Ghost 的写法；不叫这两个名字、却在别处被单元图当幽灵包装认的类型（单元图只认这两个名字）。
# 样本：verus-ghost-names-red（登记两个 crate：一个 src 里有 pub struct Ghost<T>、type Tracked = …，另一个有 use crate::holder::Holder as Tracked 与分组 use crate::{wrap::Wrapper as Ghost}，四处逐条点名判红；
#   另一行登记的目录不在，判红点名）；verus-ghost-names-green（use vstd::prelude::*、use vstd::prelude::Ghost as Ghost、注释与字符串里写的「struct Ghost」、
#   没登记的 crate 里的 struct Ghost 都不判，判绿报查了几份 .rs）。
# 弄坏开关 GATE_VERUS_GHOST_NAMES_BREAK=<项>，见文件头开关表：definitions-unseen 不判 ①；renames-unseen 不判 ②；comments-judged 不去注释与字符串；vstd-renames-judged 连 vstd 的同名改名也判红。
# 退役判据：两份单元图实现改成按解析出的路径认幽灵包装（拆分提案第一百三十三节的乙）时删掉这一格。
VERUS_PINS_TABLE=".claude/gate.d/verus-pins.tsv"
cell_verus_ghost_names() {
python3 - "$VERUS_PINS_TABLE" "${GATE_VERUS_GHOST_NAMES_BREAK:-}" <<'PY'
import glob, os, re, sys

table_path, broken = sys.argv[1], sys.argv[2]
NAMES = ("Ghost", "Tracked")
DEFINITION = re.compile(r"\b(struct|enum|union|type|trait)\s+(Ghost|Tracked)\b")
USE_STATEMENT = re.compile(r"\buse\s+([^;]*);")
RENAME = re.compile(r"((?:[A-Za-z_][A-Za-z0-9_]*::)*[A-Za-z_][A-Za-z0-9_]*)\s+as\s+(Ghost|Tracked)\b")


def code_only(text):
    """注释、字符串与字符字面量换成空格（换行照留，行号不变）。"""
    out, index, length = [], 0, len(text)
    while index < length:
        if text.startswith("//", index):
            end = text.find("\n", index)
            end = length if end < 0 else end
            out.append(" " * (end - index))
            index = end
        elif text.startswith("/*", index):
            depth, cursor = 1, index + 2
            while cursor < length and depth:
                if text.startswith("/*", cursor):
                    depth, cursor = depth + 1, cursor + 2
                elif text.startswith("*/", cursor):
                    depth, cursor = depth - 1, cursor + 2
                else:
                    cursor += 1
            out.append(re.sub(r"[^\n]", " ", text[index:cursor]))
            index = cursor
        elif (re.match(r'(?:b?r(#*)")', text[index:index + 8])
              and not (index and (text[index - 1].isalnum() or text[index - 1] == "_"))):
            hashes = re.match(r'b?r(#*)"', text[index:index + 8]).group(1)
            end = text.find('"' + hashes, text.find('"', index) + 1)
            end = length if end < 0 else end + 1 + len(hashes)
            out.append(re.sub(r"[^\n]", " ", text[index:end]))
            index = end
        elif text[index] == '"':
            cursor = index + 1
            while cursor < length and text[cursor] != '"':
                cursor += 2 if text[cursor] == "\\" else 1
            out.append(re.sub(r"[^\n]", " ", text[index:cursor + 1]))
            index = cursor + 1
        elif text[index] == "'" and re.match(r"'(?:\\.|[^\\'\n])'", text[index:index + 4]):
            literal = re.match(r"'(?:\\.|[^\\'\n])'", text[index:index + 4]).group(0)
            out.append(" " * len(literal))
            index += len(literal)
        else:
            out.append(text[index])
            index += 1
    return "".join(out)


def renames(statement_body):
    """use 语句正文里改名成 Ghost / Tracked 的 → [(补全的路径, 新名)]；花括号分组按分组前缀补全。"""
    found = []
    for match in RENAME.finditer(statement_body):
        prefix_parts, depth = [], 0
        for position in range(match.start() - 1, -1, -1):
            character = statement_body[position]
            if character == "}":
                depth += 1
            elif character == "{":
                if depth:
                    depth -= 1
                else:
                    head = re.search(r"((?:[A-Za-z_][A-Za-z0-9_]*::)+)\s*$", statement_body[:position])
                    if head:
                        prefix_parts.insert(0, head.group(1))
        found.append(("".join(prefix_parts) + match.group(1), match.group(2)))
    return found


if not os.path.isfile(table_path):
    print(f"  ! 没有 {table_path}，没有登记被验的 crate，这一格无对象可判")
    sys.exit(77)
rows = []
with open(table_path, encoding="utf-8", errors="replace") as handle:
    for number, line in enumerate(handle, 1):
        columns = line.rstrip("\n").split("\t")
        if columns[0] == "crate" and len(columns) >= 2:
            rows.append((number, columns[1].strip()))
if not rows:
    print(f"  ! {table_path} 里一行 crate 都没有，这一格无对象可判")
    sys.exit(77)
problems, scanned = [], 0
for number, directory in rows:
    if not os.path.isdir(directory):
        problems.append(f"{table_path}:{number} 登记的被验 crate 目录 {directory} 不在")
        continue
    for path in sorted(glob.glob(os.path.join(directory, "src", "**", "*.rs"), recursive=True)):
        scanned += 1
        text = open(path, encoding="utf-8", errors="replace").read()
        code = text if "comments-judged" in broken else code_only(text)
        if "definitions-unseen" not in broken:
            for match in DEFINITION.finditer(code):
                line_number = code.count("\n", 0, match.start()) + 1
                problems.append(f"{path}:{line_number} 定义了名叫 {match.group(2)} 的 {match.group(1)}（被验 crate 里的同名类型，单元图会当 vstd 的幽灵包装剔出键）")
        if "renames-unseen" not in broken:
            for statement in USE_STATEMENT.finditer(code):
                for full_path, new_name in renames(statement.group(1)):
                    from_vstd = full_path.startswith("vstd::") and full_path.endswith("::" + new_name)
                    if from_vstd and "vstd-renames-judged" not in broken:
                        continue
                    line_number = code.count("\n", 0, statement.start()) + 1
                    problems.append(f"{path}:{line_number} 把 {full_path} 改名成 {new_name}（不是 vstd 的同名类型，单元图会当幽灵包装剔出键）")
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")   # gate-lint:detail
    print(f"  ✗ 被验 crate 里有 {len(problems)} 处自己定义或改名出 Ghost / Tracked（查了 {len(rows)} 个登记的 crate、{scanned} 份 .rs）")
    print("     → 怎么办：给自定义的类型、改名的东西换一个说出它是什么的名字（Ghost / Tracked 只留给 vstd 的那两个）；登记的目录搬了家就照 .claude/rules/path-moves.md 改 verus-pins.tsv 那一行；"
          "判据在 code-source-discipline.sh 里这一格函数上方的注释")
    sys.exit(1)
print(f"  ✓ 被验 crate 里没有自己定义或改名出的 Ghost / Tracked：查了 {len(rows)} 个登记的 crate、{scanned} 份 .rs")
PY
}

# ── 格 refused-tests：验证包里拒算的测试函数与拒算登记表逐条对上
# 判法只在 scripts/gate/refused-test-registry.py check（它文件头写取法、完整性闸与弄坏开关）：照门禁 crash-point-replay 的 crash-case-replay 格同一套起法跑
# admission.py test-functions-request 与 test-plan（验证包不带特性那一趟、--exclude-registered），逐条拒算行（单元键算不出：闭包读活树、读运行时拼出的仓根路径这类，按条复用时
# 照跑、不复用、不入库）与登记表 .claude/gate.d/refused-tests.tsv 比：拒算的测试函数不在表里判红；表里登了而已经不拒算的判红（只缩不涨）；理由至少 8 个字，同一个测试函数不登两行。
# 登记由主 agent 在合入后验证之后二次确认再写，实现员不登记（.claude/main-agent.md「合入后验证之后判新出现的拒算」）。
# 样本根里放 .refused-tests-listing.tsv（照 test-plan 标准输出的写法造的文件）的，样本根（不是这道阶段所在的仓、也不是 --staged 的临时树）拿它当计划输出，不起 admission.py。
# 取不到清单（判定库服务连不上、test-plan 没打完、条数对不上）：这一格本次未跑（退 77），不当通过。
# 管不到的：该不该拒算（要不要二次确认通过）靠主 agent 的二次确认；挂特性的测试文件、产品 GPU 包与别的包里的 #[test] 的拒算不取。
cell_refused_tests() {
  local -a sample_plan=()
  if [[ -f "$ROOT/.refused-tests-listing.tsv" && "$(pwd -P)" != "$STAGE_CELLS_STAGE_REPOSITORY" && -z "${GATE_STAGED_FROM:-}" ]]; then
    sample_plan=(--plan "$ROOT/.refused-tests-listing.tsv")
  fi
  python3 "$STAGE_REPOSITORY/scripts/gate/refused-test-registry.py" check "$ROOT" ${sample_plan[@]+"${sample_plan[@]}"}
}

stage_cells_run "$ROOT"
