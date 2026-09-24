# 逐句核对：`gate-fix-forks-r1-local-attack.md`（轮 gate-fix-forks-r1，本地攻方腿）

按 `.claude/rules/three-way-inference.md`「给本地腿的提示一律用英文」一节：每一句转述写完都要对着原文核一遍；多出来的限定词、括注也单列一行、写明为什么加。下表四列：英文项（英文提示里的原句，按 fact/quote 标签定位）/ 原文文件:行 / 首稿缺的 / 定稿。「首稿缺的」写「无」表示核对时没发现遗漏，定稿与首稿一致。

## T4

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Background rule quote（stage 80 所依的规则） | `.claude/singlefs-ai-sop/rules/test-discipline.md:114-126` | 首稿只译了「臂跟臂比出来的相等，是个很弱的判据」与「后者在互比里跟正确一模一样」两句（7 句里的 2 句），却标成「quoted verbatim」——摘了句还自称逐字引用，属三方规则明令禁止的「摘句」。缺的 5 句：标题句、「⇒ 每写一条互比断言…」「『三条臂的开销相等』不够…」「做法：写完一条互比断言就问自己一句…」「答不上来就补一条…」「这条和『阳性对照必须每条臂都跑』是一件事的两面…」 | 改成整节 7 句全译，标题句一起译进去，见 research/prompts/gate-fix-forks-r1-local-attack.md 第 35-59 行 |
| Fact 4-1（80 号 BINS 与「别处目录只列名不判」） | `.claude/gate.d/80-absolute-assertions.sh:18-19,27` | 无 | 与首稿一致 |
| Fact 4-3 B1 doc comment 首句 | `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:1` | 无 | 与首稿一致 |
| Fact 4-3 B2 doc comment 首句 | `crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs:1` | 无 | 与首稿一致 |
| Fact 4-3 B3 doc comment 首句 | `crates/singlefs-harness/src/bin/first_transaction_on_device.rs:1` | 无 | 与首稿一致 |
| Fact 4-3 B4 doc comment 首句 | `crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs:1` | 无 | 与首稿一致 |

## T7

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Fact 7-1（70 号头部「源码树不在也判红」） | `.claude/gate.d/70-citations.sh:10` | 无 | 与首稿一致 |
| Fact 7-2 R1 quote | `.claude/gate.d/21-decision-items-sync.sh:38` | 无 | 与首稿一致 |
| Fact 7-2 R2 quote | `.claude/gate.d/31-blocking-verdict.sh:68` | 无 | 与首稿一致 |
| Fact 7-2 R3 quote | `.claude/gate.d/57-lkmm.sh:15` | 无 | 与首稿一致 |
| Fact 7-2 R4 quote | `.claude/gate.d/70-citations.sh:26,29` | 无 | 与首稿一致 |
| Fact 7-2 R5 quote | `.claude/gate.d/73-research-gate-lint.sh:19` | 无 | 与首稿一致 |
| Fact 7-2 R6 quote（这一轮新文案） | `.claude/gate.d/77-test-environment.sh:68,70`（工作区未提交改动） | 无 | 与首稿一致 |
| Fact 7-2 R7 quote | `.claude/gate.d/52-segment-registry.sh:27` | 无 | 与首稿一致 |
| Fact 7-2 R8 quote | `.claude/gate.d/21-decision-items-sync.sh:39` | 首稿把「本阶段无对象可判」错译成与「本阶段跳过」相同的「this stage is skipped」，抹掉了脚本自己在两种措辞间保留的区别 | 改成「this stage has nothing to judge」，与 R15、R18 用词一致，见 research/prompts/gate-fix-forks-r1-local-attack.md 第 248-250 行 |
| Fact 7-2 R9 quote | `.claude/gate.d/24-status-redundancy.sh:27` | 无 | 与首稿一致 |
| Fact 7-2 R10 quote | `.claude/gate.d/27-format-constants.sh:39` | 无 | 与首稿一致 |
| Fact 7-2 R11 quote | `.claude/gate.d/34-experiment-index-sync.sh:32` | 无 | 与首稿一致 |
| Fact 7-2 R12 quote | `.claude/gate.d/37-decision-summary-width.sh:18` | 无 | 与首稿一致 |
| Fact 7-2 R13 quote | `.claude/gate.d/40-results-cited.sh:24` | 无 | 与首稿一致 |
| Fact 7-2 R14 quote | `.claude/gate.d/42-first-txn-trio.sh:32` | 无 | 与首稿一致 |
| Fact 7-2 R15 quote | `.claude/gate.d/76-second-txn-hooks.sh:25` | 无 | 与首稿一致 |
| Fact 7-2 R16 quote | `.claude/gate.d/85-repro-command.sh:15` | 无 | 与首稿一致 |
| Fact 7-2 R17 quote | `.claude/gate.d/86-experiment-orphans.sh:14` | 无 | 与首稿一致 |
| Fact 7-2 R18 quote | `.claude/gate.d/88-quoted-result-lines.sh:26` | 无 | 与首稿一致 |
| Fact 7-2 R19 quote | `.claude/gate.d/60-stale-open-items.sh:23` | 无 | 与首稿一致 |
| Fact 7-2 R20 quote | `.claude/gate.d/61-settled-same-file.sh:31` | 无 | 与首稿一致 |
| Fact 7-4（77 号这一轮 diff：新文案 + 新增头部行） | `.claude/gate.d/77-test-environment.sh`（工作区未提交改动，`git diff` 出的两处 hunk） | 无 | 与首稿一致 |

## T8

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Fact 8-1（clip() 的两步逻辑与两个正则） | `.claude/scripts/gen-decision-items.py:25-47` | 两处：① 第二个正则 `[\s/、,，·的与和]+$` 里其实有三种逗号字符（半角逗号 `,`、全角逗号 `，`、顿号 `、`），首稿写成「a comma in either of two scripts」，把三种记成两种、且「script」这个说法本身也不准（三个都是标点，不是「文字系统」）；② 第一个正则 `[A-Z]-?\d+(?:\.\d+)?\s*$` 结尾还有一个 `\s*`（允许尾随空白），首稿的描述里没提这一段 | ① 改成「any one of three different comma characters (an ASCII half width comma, a full width Chinese comma, and the separate ideographic enumeration comma)」；② 加上「and optional trailing whitespace」；两处见 research/prompts/gate-fix-forks-r1-local-attack.md 第 355-365 行 |
| Fact 8-2（decisions.md 已有的真实剪坏例子） | `.claude/kb/decisions/05-快照-空间记账机制.md:28`（源串）；`.claude/kb/decisions.md`（已提交、未重新生成的产物，剪坏结果目前在第 167 行，行号会随下次生成变，不当作定位用） | 无——这一条不是对某句原文的转述，是拿新旧两版 `clip()` 逻辑对同一个真实输入字符串重跑一遍、把结果写成事实，原文只提供输入串本身（已在 fact 8-1 的转述里核过） | 与首稿一致 |
| Fact 8-4（`doc-lint:not-numbers` 标记原文） | `.claude/kb/INDEX.md:29` | 无——逐字符核对，与原文完全一致（见运行记录） | 与首稿一致 |

## T9

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Fact 9-1（`decision_names()` 的正则与「首行」限定） | `.claude/gate.d/lib-history-brief.py:91-97` | 无 | 与首稿一致 |
| Fact 9-2（`mentions()` 扫的文本来源 + 正则） | `.claude/gate.d/lib-history-brief.py:75-81` | 首稿写「its title plus two specific quick-note fields」，把 `what_of(entry)` 这个「有标题用标题、没标题退回用『改了什么』快查」的取一法说成单纯「标题」，既漏了退回条件这个限定词，也把来源数目从三个（`what_of` 二选一 + 改前 + 改后）数错成两个 | 改成「its own title, or, when it has no title, its own quick note for what changed, plus its own quick notes for what came before and what came after」，见 research/prompts/gate-fix-forks-r1-local-attack.md 第 486-489 行 |
| Fact 9-3（`EntriesLost` 与 `report_lost()` 的逻辑与提示语） | `.claude/gate.d/lib-history-brief.py:143-146,178-186` | 无 | 与首稿一致 |
| Fact 9-4（生成逻辑不记得「今天以外的任何一天」） | 由 `.claude/gate.d/lib-history-brief.py` 全文现查得出的「没有」——不是某一句原文的转述，是核过全文之后确认它不读 git log 或任何历史记录 | 无 | 与首稿一致 |

## T10

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Background（`lib-format-const.py` 模块 docstring 的两种读法） | `.claude/gate.d/lib-format-const.py:17-18` | 无 | 与首稿一致 |
| Fact 10-1（27 号调用签名：默认读法、`only_scalar_types`、不带 `only_top_level_public`、射程 research/+crates/、只看标记里已登记的名字） | `.claude/gate.d/27-format-constants.sh:87`（调用行）；`.claude/gate.d/lib-format-const.py:119`（默认参数定义） | 无 | 与首稿一致 |
| Fact 10-2（39 号不调 `read_rust_consts`，改调 `parse_marks`） | `.claude/gate.d/39-field-table-sum.sh:59` | 无 | 与首稿一致 |
| Fact 10-3（92 号头部注释 + 行内注释两处逐字引用） | `.claude/gate.d/92-layout-checker-sync.sh:25-26,66-67` | 无 | 与首稿一致 |
| Fact 10-4（72 个顶格 pub const、15 个在 integer_literal 下读不出、14 个标量表达式 + 1 个数组字面量） | 现查数据，来自实跑 `.claude/gate.d/lib-format-const.py` 的 `read_rust_consts` 对 `crates/singlefs-format/src/lib.rs` 现场取值——不是某一句原文的转述 | 无 | 与首稿一致 |
