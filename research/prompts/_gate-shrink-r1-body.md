# 门禁收缩与提炼：gate-shrink-r1（2026-09-28）

**实现今天的样子**（`.claude/rules/implementation-first.md` 第 3 条）：这一轮判的是门禁与协作定义本身（`.claude/gate.d/`、`research/scripts/`、`.claude/hooks/`、`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/rules/`），不改文件系统的设计与实现。`crates/` 只作被判的对象出现：代码类、harness 类、checker 类、checker-tier 类门禁读 `crates/` 的源码与测试，这一轮不改 `crates/` 里任何文件（第一轮的合并也没改，`git diff --stat HEAD -- crates/` 里的改动归别的会话的 KV 与测试工作）。

**用户定案**（2026-09-28，原话）：「门禁部分 可以合成的要合成 而不是互相引用」「你最好按照下面的分类来 文档类门禁 代码类门禁 harness类门禁 checker类门禁 checker-tier类门禁这样组织 每个门类下面几个门禁」「做完后 门禁取消编号 不再以编号命名 以后记录门禁按照全名记录」「继续 门禁需要继续收缩 需要继续分类和精华 同时针对门禁 进行三方和测试」；另把「扫仓的门禁只在提交前由 `gate-staged.sh` 起的整轮跑」定成一条规矩（第一轮 Q9，调度记录 `records/2026-09-28-门禁59号提速与双机分片.md`）。

## 一、现状（事实，出处都能现查）

- 第一轮把本地门禁从 79 道合成 23 道：`ls .claude/gate.d/*.sh`；`wc -l .claude/gate.d/*.sh` 合计 15276 行。
- 逐格盘点（只读调查交回，拷进仓里）：`research/prompts/gate-shrink-r1-inventory/cells.tsv`（84 行，sha256 `74d4bd10ea93787a255cef5ee23b0a41443149f45c7808bcdd0043eba71ea79c`）与同目录 `report.md`（sha256 `69c494d285d9e40f35e95bfbe9e0cff052942542eec6106b1f8ca8a7a4bea898`）。每格十列：判什么、读什么、单格实跑耗时、与共享门禁或别的格是否重复、依附的规矩在不在、记录里抓到过没有、今天对工作区的判定。材料员把整张表原样放进附录。
- 今天对工作区（几个会话未提交的改动都在里面）：绿 54、红 17、退 77 有 4、不跑 7（重型或要编译）、没跑完 2。
- 耗时：84 格里多数不到 1 秒；超过 10 秒的只有 12 号 term-renames（>5 分钟）、47 号 research-script-selftests（>5 分钟）、12 号 clock-times（37.8 秒）、10 号 citations（16.9 秒）、33 号 reading-discipline（13.6 秒）。
- 盘点报的四处同一件事跑两遍：33 号 mutation-tables 与 59 号用同一份 `row_problems`；47 号 agent-write-scope 与共享 `hooks-registered` 各跑一遍同一批钩子自证；43 号 table-shape 与 doc-lint F 都判「同一编号不许两处登记」；54 号快档与 74 号的用例已在共享「构建与单测」的 `cargo test --all`（debug）里跑过一遍。
- 盘点报的次序依赖：`gate.sh` 按文件名排序跑本地阶段（`gate.sh` 第 636 行一带），77 号文件头写「这一阶段排在最后」而 87、92、94 号在它后面；去掉编号改成 `<类>-<内容>.sh` 之后次序按类名字母序（checker、checker-tier、code、doc、harness）。
- 47 号 research-script-selftests 报 4 份「声称有 `--selftest` 却没阶段在跑」：认法是文件里出现 `--selftest` 这串字就算（47 号第 241 行一带）；4 份里只有 `rerun-failed-tests.py` 真有自己的 `--selftest`；原 47 号（HEAD 版）对同一份工作区报同样 4 份。
- 54、59 号与双机分片、按输入复用与全绿标记，用户已定由 singlefs-29 的里程碑三第二轮按「先录入（KV）、后核对」重设计；这一轮对它们只改名、改开关，不加功能。

## 二、方案（要腿攻的）

### A 再合：按五类，每类几道

| 类 | 合成后 | 由哪几道并成（第一轮合出的名） | 格数 |
|---|---|---|---|
| 文档类 | doc-kb | 20 doc-decision-documents、30 decision-history-entries、32 doc-field-and-layout-registry、10 references-and-invariants、43 checks-owed-and-closeout：都判 `.claude/kb/` 里决策、变更史、布局表、欠账表与引用 | 37 |
| 文档类 | doc-experiments | 34 doc-experiment-pages-and-products | 10 |
| 文档类 | doc-process-records | 11 review-and-sync-records | 7 |
| 文档类 | doc-text | 12 doc-forbidden-notations-and-old-terms | 3 |
| 代码类 | code-source-discipline | 13 code-vague-names-and-test-file-names、27 code-constants-enums-bits-match-kb、33 code-experiment-and-mutation-source-discipline | 8 |
| 代码类 | code-tooling | 47 code-tooling-selftests-and-registries | 8 |
| 代码类 | code-research-build | 15 research-build（要编译） | 1 |
| harness 类 | harness-tests | 14 harness-one-scenario-per-test、74 model-differential | 2 |
| checker 类 | checker-independence-and-sync | 92 layout-checker-sync、94 checker-implementation-disjoint | 2 |
| checker-tier 类 | 各自一道 | 54、55、57、59、87 不并（重型，各有复用标记与内存上限，54、59 在重设计）；77 test-environment 见 B6 | 6 |

合计 15 道。文件名去编号：`<类>-<内容>.sh`。

### B 砍、改（「精华」）

| # | 对象 | 方案 | 依据 |
|---|---|---|---|
| B1 | 43 号 row27-preconditions | 里程碑二提交之后删这一格 | 只盯里程碑二收口表第 27 行那几笔欠账的前置，今天退 77 |
| B2 | 47 号 agent-write-scope 里对钩子自证的调用 | 删掉与共享 `hooks-registered` 重复的那一段，只留表与定义双向一致 | 盘点第 7 列 |
| B3 | 43 号 table-shape 与 doc-lint F 重叠的那一半 | 删重叠的一半，留 doc-lint 不判的部分 | 盘点第 7 列 |
| B4 | 74 号 model-differential 与「构建与单测」重复跑用例 | 待腿判：留 74 号（它带内存包装、逐段判过）还是靠「构建与单测」 | 盘点第 7 列 |
| B5 | 47 号 research-script-selftests 的认法 | 只认自己实现了 `--selftest` 入口的脚本，不认文字里提到别人的 | 盘点报告 |
| B6 | 77 号 test-environment | 必须排在全部跑测试的阶段之后：去编号之后次序按类名，要么改名让它排最后，要么证明它不依赖次序 | 盘点报告 |
| B7 | 12 号 term-renames、47 号 research-script-selftests 超过 5 分钟 | 查慢在哪，能收就收；不为省时砍格 | 盘点第 6 列 |
| B8 | 10 号 experiment-refs、decision-refs 与 doc-lint H、`number-name-sync` 部分重叠 | 待腿判重叠到什么程度、能不能删 | 盘点第 7 列 |
| B9 | 长期红的格：11 号 batch-scope（149 个触发文件没登记）、knowledge-sync（134 个没点名）、34 号 archive-past-rounds（1816 份没归档） | 待腿判：判据对、是流程欠账，还是判据本身不合用 | 盘点第 10 列 |

### C 第一轮改过的定义与共用约束

第一轮为「扫仓的门禁只在提交时跑」（Q9）、「看门狗放行样本自检」（Q15）、doc-lint 挪到提交时（Q8）与各组合并改了下面这些文件的文字；按合并后 11 号 agent-def-adversarial-review 那一格的要求走这一轮三方：`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/kb-scribe.md`、`experiment-runner.md`、`tooling-writer.md`、`three-way-materials.md`、`prior-art.md`、`crash-verifier.md`、`implementation-writer.md`、`investigator.md`、`kb-spec-drafter.md`，规则 `.claude/rules/implementation-workflow.md`、`implementation-first.md`、`three-way-inference.md`、`format-evolution.md`、`mutation-sampling.md`、`verification.md`、`path-moves.md`，以及 `CLAUDE.md`。材料员照 `git diff HEAD --` 这些路径出 diff。不审：`three-way-verifier.md` 的删除与 `three-way-local-attack.md`（会话开始前已在暂存区）、`.claude/rules/changelog-format.md`（singlefs-0a 那一轮，它另开了 changelog-format-r1）。

## 三、要腿答的

1. **A 的合并丢不丢覆盖**：每一道被并的门禁、每一格，在合成后的哪一道、哪一格里；有没有一格在合并里消失、或两格合成一格后判据变宽变窄。拿 `cells.tsv` 逐行对。
2. **合并后单道的格数与失败信息**：doc-kb 37 格，一格红让整道红；按 `sop-first.md`「每一条拒绝都必须给出下一步」，出路还能不能指到那一格、那条规则？`--check <格名>` 够不够用？有没有该拆回来的。
3. **B1–B9 每一条**：删了或改了之后，什么违规会从此没人判？举一个具体的输入。B4、B8、B9 给出判断与依据。
4. **B6 次序**：去编号之后按类名排序，哪些阶段的判法依赖前面的阶段跑过（盘点报了 77 号、20 号 `--write` 先于 30 号 `--write`、55/57/59/74/87 看 staged-green）；给出不靠字母序偏门的办法，或证明不依赖。
5. **C 定义改动**：逐份核 diff 是不是只做了 Q8、Q9、Q15 与改门禁号要的改动；有没有改出互相矛盾的两句（例：一处说「交回前不跑门禁」、另一处还要求跑某一道）；阶段归属表第二列改成「判红时派给谁修」之后，`crash-verifier.md` 靠这张表挑提交时跑哪几道，还说得通吗。
6. **「只在提交时跑」这条规矩的代价**：书记员、执行员写错的 kb 格式，从此要到提交时才红、再派人修。有没有该留在写入那一刻就拦的（像 write-guard 那样的钩子），列出来并说理由。

## 四、分工

本地腿：攻（A 的逐格映射：给一张写死的事实表——被并门禁的每一格与方案里的合成后门禁——要模型逐格填「落在哪一道」「判据是否原样」，数出消失的格与重复的格；攻击面与云端攻方不重叠）。没有前一轮判决要复核，辩方这一轮没有对象。

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | 第 1、2、5、6 问 | 从 `cells.tsv` 与合成后的脚本出发，给出一套完整的合并与改名方案：每道合成后门禁的格表、出路、`gate-category`、跑的次序；定义改动逐份判 |
| 云端攻方（Opus） | 第 3、4、6 问与 A 里 doc-kb | 专找「合了、砍了之后从此没人判」的违规：对 B1–B9 每条造一个具体的违规输入；对 doc-kb 37 格造「一格红把别的格的出路淹掉」的场景；对次序依赖造「按类名排序后判错」的场景 |
| 本地攻方 | 第 1 问 | 按事实表逐格填：旧门禁、旧格名、方案里的去处、判据原样与否；报消失的格数与重复的格数。事实表每行写来源文件与行号，一张表不超过 6 行，分几张 |

## 五、交付

- 腿的报告写 `research/prompts/gate-shrink-r1-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建），回复只写短句。
- 模型放 `research/prompts/gate-shrink-r1-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`；草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb、规则、门禁脚本写那份文件自己的行号，行号去文件里现查。
- 方案按 `crates/` 与 `.claude/gate.d/` 今天的样子来谈。
- **资源**：线程上限 4；跑门禁只跑非重型的格（`--check <格名>`），一律经 `bash research/scripts/run-with-memory-cap.sh 8G <命令…>`；不跑 54、55、57、59、87 号，不跑 `gate.sh` 整轮与 `gate-staged.sh`；只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`；交回之前后台不许留着跑的东西。
- 不写时刻，时区名也不写；变体起新名字，不用撇号角标。
