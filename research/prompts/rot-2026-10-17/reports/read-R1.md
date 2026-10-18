# 回扫 R1：`.claude/rules/*.md` 十二份通读（知识腐烂治理第二轮，批6）

日期：2026-10-17。只读，没改任何文件。没建副本、没起编译、没跑门禁阶段。草稿目录 `/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-R1`（只放了 `paths.tsv`，238 行路径抽取清单，不用清理）。

## 做法

- 十二份从头到尾逐句读（`Read` 分段读到末行；行数见「汇总」）。
- 路径类：`grep -o` 抽出规则里反引号包着的路径与文件名 238 条，逐条 `[ -e ]`，缺的 28 条全是裸文件名，再用 `git ls-files | grep -F "/<名>"` 找到真位置，全部在；`show-me-test.md`、`test-discipline.md` 在 `.claude/singlefs-ai-sop/rules/`。
- 门禁格名：规则里点名的 43 个格名，逐个 `grep -l "^# gate-cell: <格名> " .claude/gate.d/*.sh`，全部命中，且归属的门禁与规则所写相同。
- 环境变量与大写标识符 31 个：`git grep -lF` 除规则、records、research 外都有出处（`INCOMPAT_LAYOUT_*` 是 fs-design 的示意代码，不算）。
- 决策与欠账编号：D5、D9、D12、D13、D14、D15、D17、D19、D21、D22 的标题与已定项号逐个对到 `.claude/kb/decisions/*.md`；C8、C19、C49、C192、C234、C244、C273、C317、C320 简称逐个对到 `.claude/kb/checks-owed.md`；E109、E110 简称对到 `.claude/kb/experiments.md`。
- 脚本事实：读了 `mutate.sh`、`crates-mutation-rows.py`、`checker-tier-formal-verification.sh`、`checker-tier-crash-point-replay.sh`、`crash-point-replay.sh`、`lib_abandoned_rounds.py`、`quote-kb.py`、`lib_heavy_tests.py`、`admission.py`、`code-source-discipline.sh` 的文件头与相关函数，对 `verification.md`、`implementation-workflow.md`、`mutation-sampling.md`、`three-way-inference.md` 里的退出码、计数栏、符号、参数、环境变量、登记表列。
- `python3 scripts/gate/directory-layout-check.py` 现跑：退 0，「判了 research/ 与 scripts/ 下 12498 份文件」。
- 术语改名：把 `.claude/kb/term-renames.md` 的 291 行「整串」「词边界」旧名逐个在十二份里搜，0 命中。

判定取值：要改、要补、不改（不逐行列）；拿不准的写「要人看」。

## 判「要改」「要补」「要人看」的行

| 文件:行 | 原文（截 120 字） | 判定 | 改后句或理由（引原话至少 4 个字，带现查命令） |
|---|---|---|---|
| `.claude/rules/mutation-sampling.md:7` | 它在「变异测试证明的是断言会红，不是覆盖」一节第 1 条还提到减法断言编译期溢出会被记成「无效变异」。 | 要改 | 那一节第 1 条没有这句。现查：`grep -n '无效变异\|编译期溢出\|溢出' .claude/singlefs-ai-sop/rules/test-discipline.md` 零命中；`sed -n 87,99p` 看到第 1 条只写「常量断言写加法，别写减法」。改后：「它在「变异测试证明的是断言会红，不是覆盖」一节第 1 条只要求常量断言写加法、不写减法；减法断言在变异下会在编译期溢出、被记成「无效变异」（见下面第八类）。」 |
| `.claude/rules/mutation-sampling.md:68` | （编译期溢出那一种见 `.claude/singlefs-ai-sop/rules/test-discipline.md`「变异测试证明的是断言会红，不是覆盖」一节第 1 条） | 要改 | 同上，那一条讲的是写加法的做法，没讲溢出被记成无效。改后：「（编译期溢出那一种的避法见 `.claude/singlefs-ai-sop/rules/test-discipline.md`「变异测试证明的是断言会红，不是覆盖」一节第 1 条：常量断言写加法、不写减法）」。`mutate.sh` 文件头确实把编不过记成「⏭ 编不过」（`grep -n '编不过' scripts/mutation/mutate.sh`），所以第八类本身的说法没错，错在出处。 |
| `.claude/rules/fs-design.md:134` | `.claude/singlefs-ai-sop/rules/machine-first.md` 的两条推论——穷举的显式分支、用类型让非法状态无法表示——代码上照用，代码示例见那一篇。 | 要改 | 那一篇没有第二条推论，也没有代码示例。现查：`grep -c '```' .claude/singlefs-ai-sop/rules/machine-first.md` → 0；`grep -n '非法状态' .claude/singlefs-ai-sop/rules/*.md` → `code-discipline.md:116`（「## 类型：让非法状态写不出来」，带 rust 代码块）、`engineering-philosophy.md:14`；`machine-first.md:39` 只有「宁可写穷举的显式分支」。改后：「`.claude/singlefs-ai-sop/rules/machine-first.md` 与 `engineering-philosophy.md` 的两条推论——**穷举的显式分支**、**用类型让非法状态无法表示**——代码上照用，代码示例见 `.claude/singlefs-ai-sop/rules/code-discipline.md` 的「分支：穷举写全，不写通配臂」与「类型：让非法状态写不出来」。」 |
| `.claude/rules/format-evolution.md:68` | format-constants 格从此在 kb 正文与 `research/`、`crates/` 下的 `.rs` 里替你盯着；它只扫这两处，别处一概不扫（门禁脚本、`research/results/` …… | 要改 | 扫描范围少了 `scripts/experiments/`，而实验装置源码现在就在那里。现查：`grep -n 'scanned_roots = \[' .claude/gate.d/code-source-discipline.sh` → 435 行 `["scripts/experiments", "research", "crates"]`；同文件 375 行「扫哪些源码：……scripts/experiments/、research/、crates/ 下的 .rs」；`ls research/` 只有 `prompts results README.md`，`research/prompts` 下 `.rs` 645 份是冻结模型，由 `.claude/format-constants-exclude` 放行。改后：「format-constants 格从此在 kb 正文与 `scripts/experiments/`、`research/`、`crates/` 下的 `.rs` 里替你盯着（`research/prompts/` 下冻结证据里的 `.rs` 由 `.claude/format-constants-exclude` 逐份放行）；它只扫这两类，别处一概不扫（门禁脚本、……」，后面的括号清单里的 `research/prompts/` 改写成 `research/prompts/` 下的 `.md`。 |
| `.claude/rules/implementation-first.md:13` | 4. **`research/` 下的装置与模型只用来独立验证**：它们不与实现共用代码，用来核实现、核推论。 | 要改 | `research/` 下现在没有装置。现查：`ls research/` → `prompts results README.md`；`.claude/rules/directory-layout.md` 第 17 行「反复要用的脚本、装置与输入不放 `research/`，放 `scripts/`」；实验装置在 `ls scripts/experiments/`（`e100_*.rs` 起），各腿自己写的模型在 `research/prompts/*-model/`（`git ls-files research/prompts \| grep -c '\.rs$'` → 645）。改后：「4. **`scripts/experiments/` 下的实验装置与 `research/prompts/` 下各腿自己写的模型只用来独立验证**：它们不与实现共用代码，用来核实现、核推论。」第 14 行「`research/prompts/` 下的冻结证据不回改」不受影响。 |
| `.claude/rules/implementation-workflow.md:11` | code-source-discipline 的 mutation-tables 格（research 的变异表）、`check.sh` | 要改 | 变异表已搬出 `research/`。现查：`ls scripts/mutation/tables \| wc -l` → 185；`.claude/gate.d/code-source-discipline.sh` 第 36 行「mutation-tables      scripts/mutation/tables/ 与 crates/mutations.tsv」。改后：「code-source-discipline 的 mutation-tables 格（实验装置的变异表 `scripts/mutation/tables/`，加 `crates/mutations.tsv` 的静态几样）」。 |
| `.claude/rules/implementation-workflow.md:68` | 核它的崩溃状态块在崩溃判定库里都有单元键相同、不红的判定（两条流的崩溃点重放全量都要 `exhaustive=true`），有红块判红…… | 要改 | 「两条流」的数过时了。现查：`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` → 14；其中第三列带 `exhaustive=<前缀>` 的 7 行（`grep -n 'exhaustive=true' .claude/gate.d/stage-inputs.tsv` → 46、48、49、50、52、53、54 行）；`scripts/gate/admission.py` 文件头「exhaustive=<前缀>  那一行带 exhaustive=true」，2250 行起按 `case.exhaustive_lines` 逐条判。改后：「（登记行第三列写了 `exhaustive=<前缀>` 的崩溃点重放全量，那一行计数行都要带 `exhaustive=true`）」。同一句的「再逐条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）」不变。 |
| `.claude/rules/implementation-workflow.md:10` | 射程是产品代码：`crates/singlefs-core/`、`crates/singlefs-format/`、`crates/singlefs-replay-prefix/`、`crates/singlefs-reference-filesystem/` 的 `src/` …… | 要人看 | 八个 crate 里，射程清单与「测试代码不走这一步」清单（`crates/*/tests/`、harness、checker-tier、checker、`crates/mutations.tsv`）都没点 `crates/singlefs-unit-graph/`。现查：`ls crates/` 八个；`.claude/gate.d/doc-process-records.sh` 327 行 `TEST_CODE_SOURCES='^crates/(singlefs-harness\|singlefs-checker-tier\|singlefs-checker)/src/'` 只去掉三个包，所以门禁的 crates-adversarial-review 格把 `singlefs-unit-graph/src/*.rs` 当产品代码、要三方点名；而它的 `Cargo.toml` 描述写「测试基础设施库，归 checker 档依赖的一侧」。规则与门禁在这个包上说法不一。两种改法：把它写进射程；或把它连同门禁的 `TEST_CODE_SOURCES` 一起并入测试代码一侧。归谁要用户或主 agent 定。 |
| `.claude/rules/implementation-workflow.md:66` | 这个驱动只由这一道门禁调；它与这道门禁的整套重设计归别的会话。 | 要人看 | 「归别的会话」是 2026-10-03 写下的归属（`git log -S'整套重设计归别的会话' -- .claude/rules/implementation-workflow.md` → `0da17aa1 2026-10-03`），仓里查不到那个会话的去向：`git grep -n '整套重设计'` 只剩这一行；`git log --since=2026-10-03 -- scripts/mutation/mutation-shard-run.sh` 只有两次搬目录与改名提交。「只由这一道门禁调」那半句是真的（`.claude/hooks/lib_heavy_tests.py` 里写「只由那一道调」）。后半句今天还是不是真话要主 agent 现查，是就留，不是就删。 |
| `.claude/rules/verification.md:8` | 四个包，四个名字，全仓只用这四个名字称呼它们： | 要人看 | 表里四行（harness 档、checker 档、池级 checker、RefFS）对得上 `crates/` 下的四个包；但 `crates/singlefs-unit-graph/` 是 checker 档的依赖（`grep -n 'unit-graph' crates/singlefs-checker-tier/Cargo.toml` → 17、30 行），本节正文第 36 行（`crates/singlefs-checker-tier/build.rs` 经 `crates/singlefs-unit-graph` 算判器单元键）在用它，这一节没给它归类。与上一行同一个缺口，分类由主 agent 定；定了以后在这里补一行，或在第 8 行写明它不在这张表里。 |
| `.claude/rules/verification.md:56` | 轻型保留到事务功能做全；之后由用户判稳定，再并进重型。 | 要人看 | 来源在：`.claude/kb/verification-build.md:299`「事务功能做全之前算不稳定，什么时候算稳定由用户判」，所以句子照原样成立。拿不准的两点：一是前置「事务功能做全」今天做没做全，我现查不出（`ls .claude/kb/milestone \| wc -l` → 17 份里程碑文件，无一处写这个判定）；二是 kb 那行说稳定之后「删掉」轻型、日常改用重型的剪枝与逐条跑，规则说「再并进重型」，两个去向写法不同（轻型是重型子集，并进与删掉效果相近，但字面不一）。请主 agent 对着当前里程碑判一次，需要的话把这句的去向对齐成 kb 的说法。 |

## 汇总

| 文件 | 行数 | 含可核事实的行数（*） | 要改 | 要补 | 要人看 |
|---|---|---|---|---|---|
| fs-design.md | 189 | 17 | 1 | 0 | 0 |
| format-evolution.md | 74 | 31 | 1 | 0 | 0 |
| three-way-inference.md | 166 | 38 | 0 | 0 | 0 |
| mutation-sampling.md | 93 | 16 | 2（同一件事两处） | 0 | 0 |
| implementation-workflow.md | 68 | 28 | 2 | 0 | 2 |
| verification.md | 137 | 63 | 0 | 0 | 2 |
| test-script-discipline.md | 106 | 32 | 0 | 0 | 0 |
| implementation-first.md | 19 | 8 | 1 | 0 | 0 |
| path-moves.md | 58 | 22 | 0 | 0 | 0 |
| directory-layout.md | 28 | 11 | 0 | 0 | 0 |
| changelog-format.md | 46 | 19 | 0 | 0 | 0 |
| timer-only-remind.md | 29 | 11 | 0 | 0 | 0 |
| 合计 | 1013 | 296 | 7 | 0 | 4 |

（*）列的口径：非标题行里含反引号词、D / E / C 编号、「门禁」、`.claude/`、`scripts/`、`crates/` 之一的行数，命令 `grep -E '...' 文件 \| grep -vc '^#'`。它是「要逐句现查的行」的上界，不是现状分句数：一行里常有三五个可核分句（verification.md 第 13、36、135 行各十几个），分句逐个核过，没有数过个数，所以不报分句数。`wc -l .claude/rules/*.md` 合计 1013 行，与派发给的十二份对得上。

## 要主 agent 现查

1. `.claude/rules/directory-layout.md:21-22`（「不变量清单、登记表、钉表这类只被某一组测试读的定义与输入，放在那组测试所在的 crate 目录里，不放 `.claude/gate.d/`」）与仓里现状不一致，但它是做法不是事实句，我没判「要改」：`ls .claude/gate.d/*.tsv` 里有 `verus-pins.tsv`、`e165-pinned-red-examples.tsv`、`crash-point-replay-items.tsv`、`crash-point-replay-segments.tsv`、`crates-mutation-light-rows.tsv`、`cpu-judging-sites.tsv`、`gpu-exemptions.tsv`，都是某一组测试的登记表或钉表。要么这条规则有例外没写，要么这些表该搬。该由主 agent 或用户定，定了再改这一节。
2. `.claude/rules/format-evolution.md:26`「不超过 100 字」：门禁里没有这个数的检查。`grep -n '\b100\b' .claude/gate.d/doc-decisions.sh .claude/scripts/gen-decision-items.py` 零命中；门禁只判 `decisions.md` 索引表「结论」列的 200 字上限（`decision-summary-width` 格，上限从 `.claude/kb/decisions.md:13` 读）。规则里没写「门禁管这一半」，所以不算说错；如果要写进「门禁管不到的」一并补，由主 agent 定。
3. 我读了 `.claude/hooks/lib_heavy_tests.py` 的文件头、没有逐个分支对照 `implementation-workflow.md:49` 与 `verification.md:135` 两段长清单（重型测试的逐类写法）。清单里每个脚本、门禁、参数名我现查都在，但「某个用法算不算重型」这类行为判定，要靠 `python3 .claude/hooks/lib_heavy_tests.py --selftest`（登记为 light，见 `.claude/hooks/selftest-registry.tsv`）才能全核，我没跑。
4. `.claude/rules/path-moves.md:36`「`scripts/kb/sweep-term.py` 把文件名形态、历史里 `research/results/` 下出现过而树里没有的串当已归档产物的名字，不换也不报」：我只核了 `sweep-term.py` 文件头的登记表写法与 `--check/--apply` 用法，没读到实现这一支，没核。
5. `.claude/rules/implementation-workflow.md:17`「（rules-lint 扫这三处）」：上游 `.claude/singlefs-ai-sop/scripts/gate.sh` 第 492 行那一阶段扫 `CLAUDE.md`、`.claude/agents/*.md`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/skills/*/SKILL.md` 五处，那一句说的三处都在其中，所以判不改；如果想写全，加上 `CLAUDE.md` 与 skill 正文。
6. 本轮没有任何一份判「要补」。我反向想过的几件新东西：运行范围声明（`verification.md:58`）、判定库哨兵（`verification.md:59`）、定时只用 `remind.sh`（`timer-only-remind.md`）、叫停轮出处（`three-way-inference.md:121`）、每台默认重活线程数（`implementation-workflow.md:64`），规则里都有一处记着，且指到的文件都在。

## 没做什么

- 没改任何文件；没建副本；没起编译；没跑 `gate.sh`、任何门禁阶段、`--selftest`（`directory-layout-check.py` 是只读判，跑了一次）。
- 没读会话记录与 `~/.claude/`。
- 现状句数没有精确到分句，见汇总表下的口径说明。
- `three-way-inference.md`、`test-script-discipline.md`、`path-moves.md`、`changelog-format.md`、`timer-only-remind.md`、`directory-layout.md` 里没找到过时的事实句，不代表没有：我核的是引到的文件、格名、环境变量、编号简称、参数、退出码、状态；纯做法句（该怎么写、怎么判）不判对错。
- 「要改」的 7 行里，改后句只是建议，主 agent 或写回员落笔前请再现查一遍上面各行给出的命令。
