# defs-m3-formal-gate-r1 正推腿（Sonnet）报告

日期 2026-10-01。批次 `records/2026-10-01-里程碑三形式化接入.md#4` 的 4-5、3-8、3-9。格 D4、D5，兼 D1（正文第四节分工表「云端正推（Sonnet）」一行）。
被判对象读主仓现行版本；17 份快照 `sha256sum -c` 的结果写在末尾。引用的行号都是这一轮在被引文件里现查的。
被判的命令一条都没执行；喂 hook 的只喂 JSON、只看退出码（探针与输出在 `research/prompts/defs-m3-formal-gate-r1-sonnet-model/`）。

## 一、结论摘要（细节在后）

- D4：被判的 11 份里凡列「提交时重型门禁」的位置都点了这一道，点法与前四道同形；没有多出的。漏掉的有三处在被判文件里（`agent-common.md` 的内存包装清单、`gate-triage.md` 的内存包装退 25x 分类句、`heavy-test-guard.sh` 的自检成功行与一句「它们底下」），两处在被判文件之外（`implementation-writer.md`、`crash-test/SKILL.md` 的表）。
- D5：这一道新写的句子里没有日期、没有位置指代词（doc-lint 的上下文指代判据对射程内 30 条加行只命中 1 条，那一条是沿袭的旧句）；两处要改：`implementation-workflow.md` 把新句子放进了讲多线程的小节，`verification.md` 与它重复；两处「今天的实现」是相对时间词。
- D1：谁带前缀跑、`verus-stage` 与 `lib_heavy_tests.py` 的判定，25 格探针 0 格与字面不符。照字面做会做错的有四处：整轮 `gate.sh` 的描述（「只核标记」与阶段真跑不符）、`crash-verifier.md` 第 2 步后台模板没带两个上限、第 6 步指纹不含这一道的四类输入、`gate-triage.md` 的 25x 分类句认不出这一道的包装退出；另有一条绕过：python 判法脚本直接起不被闸判重型。

## 二、D4 清单一致

### 2.1 现查命令与原样输出

```
$ for f in <被判的 11 份与 lib_heavy_tests.py>; do echo "$f crates-mutation-replay=$(grep -cF checker-tier-crates-mutation-replay $f) formal-verification=$(grep -cF checker-tier-formal-verification $f)"; done
.claude/rules/verification.md crates-mutation-replay=4 formal-verification=4
.claude/rules/implementation-workflow.md crates-mutation-replay=5 formal-verification=4
.claude/agents/crash-verifier.md crates-mutation-replay=6 formal-verification=5
.claude/main-agent.md crates-mutation-replay=2 formal-verification=2
.claude/agents/gate-triage.md crates-mutation-replay=1 formal-verification=1
.claude/agents/tooling-writer.md crates-mutation-replay=1 formal-verification=1
.claude/agents/investigator.md crates-mutation-replay=1 formal-verification=1
.claude/skills/crash-test/SKILL.md crates-mutation-replay=2 formal-verification=2
.claude/agent-common.md crates-mutation-replay=2 formal-verification=1
.claude/hooks/heavy-test-guard.sh crates-mutation-replay=7 formal-verification=10
.claude/hooks/runner-dispatch-guard.sh crates-mutation-replay=1 formal-verification=1
.claude/hooks/lib_heavy_tests.py crates-mutation-replay=9 formal-verification=4
```

前四道里门禁名字最后一个（crates 变异表）比这一道多出来的行，逐行看过：`implementation-workflow.md` 多的一行是双机分片说明，`crash-verifier.md` 多的两行是写范围与产出（见 2.3 的 D4-e），`agent-common.md` 多的一行是内存包装清单（D4-a），`lib_heavy_tests.py` 多的是双机分片驱动的注释与自检，`heavy-test-guard.sh` 的 7 对 10 是这一道另有自检格与弄坏开关，不是漏。

另按「同一行里并列三个以上门禁名或类名」再扫一遍（扫描输出留在草稿目录 `/tmp/claude-1000/defs-m3-formal-gate-r1-sonnet/matrix.txt`，不入库；结果按行列在 2.2、2.3）。

### 2.2 点了、点法一致的位置（一致）

提交时跑几道的数词都改成「五道」，门禁名字都写 `checker-tier-formal-verification`，类名写「Verus」或「形式化验证」，与同一张清单里「QEMU」「herd7」「crates 变异表」这些类名并用同一种写法：

- `crash-verifier.md`：「Verus 形式化验证这几道重阶段」（`.claude/agents/crash-verifier.md:3`）、「里提交时跑的这五道门禁」（`.claude/agents/crash-verifier.md:14`）、「只在提交时（或主 agent 转达用户要求时）跑这五道」（`.claude/agents/crash-verifier.md:31`）。
- `main-agent.md`：内存上限句（`.claude/main-agent.md:52`）、两条 `crates/` 改动的派发行（61、63）、「暂存之后、提交之前跑门禁」行（66）都点了。
- `implementation-workflow.md`：复用判定（32）、「每次提交代码」表格行（53）、重型测试定义（57）。`verification.md`：checker 档表格行（13）、复用判定句（37）、「谁在什么时候跑得了 checker 档」行（106）。
- 其余：`gate-triage.md:28`、`investigator.md:32`、`tooling-writer.md:49`、`crash-test/SKILL.md:15` 与 40、`agent-common.md:52`、`heavy-test-guard.sh:60`、62、600，`runner-dispatch-guard.sh:578` 与 579、`lib_heavy_tests.py:60`。
- 钩子判定三处对得上：`lib_heavy_tests.py` 的类别表有「"verus-stage": "Verus",」（`.claude/hooks/lib_heavy_tests.py:132`），阶段名表有「"checker-tier-formal-verification.sh": "verus-stage"}」（`.claude/hooks/lib_heavy_tests.py:152`），`heavy-test-guard.sh` 的 crash-verifier 集合有 `verus-stage`（`.claude/hooks/heavy-test-guard.sh:175`），gate-triage 集合没有它（`.claude/hooks/heavy-test-guard.sh:176`）；与 `heavy-test-guard.sh:48` 的文件头说明一致：「提交时由它带前缀单独跑，主 agent 带前缀也能单独跑，gate-triage 起的整轮 gate.sh 里也有它」。
- 不是漏的两处：`verification.md:89` 列「可点的项」只有四道，这一道没有 `--item`（阶段只认 `--list`、`--check`、`--tool-versions`，见 `.claude/gate.d/checker-tier-formal-verification.sh` 的参数解析）；`stage-owners.tsv` 里这一道六格归实现员或工具实现员（「验不过、verified 数变了都由写证明的实现员修」，`.claude/gate.d/stage-owners.tsv:10`），那张表判的是「红了先派给谁修」，不是「谁跑」，与「crash-verifier 只跑、只报，不修」不冲突。
- 多出的：没有。

### 2.3 漏掉或不一致的位置

**D4-a 冲突（低）`agent-common.md:54`**。这一句列哪几道「在里面逐条套了」内存包装，写的是：
「与门禁 checker-tier-crates-mutation-replay 在里面逐条套了」（`.claude/agent-common.md:54`），收尾「都直接跑，外面不再包一层」（`.claude/agent-common.md:54`）。
对应 `crash-verifier.md` 第 1b 步已写「checker-tier-formal-verification 也在里面套了（每次 Verus、E165 装置各经包装）」（`.claude/agents/crash-verifier.md:29`）。这一道事实上在阶段里面经包装：`research/scripts/verus-proof-check.py` 默认上限「"memory_cap": os.environ.get("GATE_VERUS_MEMORY_MAX") or "4G"」（`research/scripts/verus-proof-check.py:737`），装置一侧「"memory_cap": os.environ.get("GATE_E165_PINNED_MEMORY_MAX") or "2G"」（`research/scripts/e165-pin-red-examples.py:411`）。共用约束这一句的清单里没有它，也没有这两个变量名；读共用约束又不读 crash-verifier 定义的角色（目前只有 crash-verifier 会起这一道）不受影响，所以评低。

**D4-b 冲突（中）`gate-triage.md:29` 的 25x 分类句**。原句：
「✗ 行写着内存包装退 25x 的（research-unit-tests 格与 harness-model-differential-and-scenarios 的「内存包装 research/scripts/run-with-memory-cap.sh 退 <码>」）」（`.claude/agents/gate-triage.md:29`），1b 的括注同样只点这两道：「同一道的 research-unit-tests 格与 harness-model-differential-and-scenarios 在阶段里面经包装」（`.claude/agents/gate-triage.md:27`）。
这一道在整轮 `gate.sh` 里真会跑（见 4.2），它的内存包装结局不是这个措辞：Verus 一侧打「内存包装自己的结局（退 {outcome['exit']}」（`research/scripts/verus-proof-check.py:356`），装置一侧只打「装置退 {completed.returncode}」（`research/scripts/e165-pin-red-examples.py:398`）。gate-triage 照字面读，这两种红认不出是包装退出，会落到「看它点名的文件与行在不在暂存区 diff 的块里」那条路上去判「这一轮」或「不是这一轮」，而不是判「环境（上限）」「并发」。推翻它的现象：把这一道的包装退出措辞补进那句、或那句改成不按措辞认，之后再造一次 250 退出看分类。

**D4-c 冲突（低）`heavy-test-guard.sh` 两处文字**。自检成功行的清单「子 agent 跑 checker 档（按包判）、全量测试、check.sh、整轮门禁、QEMU、herd7、crates 变异整表拒绝」（`.claude/hooks/heavy-test-guard.sh:1286`）没有 Verus，而自检里已有这一道的格（`.claude/hooks/heavy-test-guard.sh:804` 起的几格）；POLICY 的第二段「与它们底下的 checker 档包 singlefs-checker-tier 的测试（快档与登记的全量用例）、qemu-system、lkmm.sh / herd7、crates 变异整表」（`.claude/hooks/heavy-test-guard.sh:606`）列的是五道「底下」起的东西，没有 verus 与 E165 装置。两处都是给人看的提示文字，不影响判定。

**D4-d 漏（中，被判文件之外）`implementation-writer.md:58`**。「checker 档（`checker-tier-layer0-replay` 快档与全量、QEMU、herd7、crates 变异表）归 `crash-verifier`」（`.claude/agents/implementation-writer.md:58`）。实现员是写证明与改核的人，它的「没做什么」把 checker 档归谁写成四样，没有 Verus 那一道；闸会拒它跑，所以不会误跑，但它交回时会少写这一条。

**D4-e 提示（不评冲突）`crash-verifier.md` 写范围**。写范围一句点了这一类阶段自己用的临时目录与编译产物：「checker-tier-crates-mutation-replay 的 `GATE_MUTATION_TARGET_DIR` 等」（`.claude/agents/crash-verifier.md:39`），这一道的装置编译目录默认落在 `${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/singlefs-e165-pinned-target`（`research/scripts/e165-pin-red-examples.py:28`）、Verus 每处一份副本，句尾有「等」，也没有写成不许；只作提示，不评冲突。交回闸 `handback-scratch-check.sh` 认不认这个目录，我没验（见「没做什么」）。

**D4-f 漏（低到中，被判文件之外，供主 agent 判归不归这一轮）**

- `crash-test/SKILL.md` 的表「## 四样各落一道门禁」（`.claude/skills/crash-test/SKILL.md:17`）列崩溃点重放、模型对拍、QEMU、LKMM 四样，QEMU 与 LKMM 两行标「重型」「不声明覆盖」；frontmatter 描述句也只点这四样（`.claude/skills/crash-test/SKILL.md:3`）。这一道是第五道重型门禁，阶段文件头没有 `# gate-covers:`（`grep -c gate-covers` 对它是 0，对 QEMU 那一道是 1），表里没有它的行，标题的「四样」也与之不符。同一份 skill 的第 15 行与第 40 行已点了它。
- 全绿标记的清单在三处注释里只点前三道：`research/scripts/admission.py:70` 的「stage-marker-write <根> <阶段文件名>   判绿之后写这一格标记（checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 自己调）」、`research/scripts/admission.py:1124` 的「先看这一道在被判那棵树上的全绿标记（checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 判绿时自己写」、`research/scripts/stage-must-run.sh:17` 的「在这之前先看一眼全绿标记：checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 判绿时」。这一道的阶段也写标记：`python3 "$ADMISSION_MODULE" stage-marker-write`（`.claude/gate.d/checker-tier-formal-verification.sh:147` 一带，文件头也写了）。
- `.claude/gate.d/code-tooling.sh:669` 的「重型测试——层 0、QEMU、herd7、crates 变异整表」、`README.md:126` 的「QEMU 真设备、herd7、crates 变异表复跑」：不含这一道。后者是 gate.sh 注释行，说「含 checker 档快档与逐条全绿标记核对」，整轮里实际有这一道（见 4.2）。

## 三、D5 规则写法

判据读的是 `.claude/singlefs-ai-sop/rules/rules-discipline.md` 的「1. 正文只写四样」与「7. 正文不许用位置指代，也不许自称」。射程：「射程是 `rules/*.md`、`CLAUDE.md`、`agents/*.md`、`skills/*/SKILL.md`。」（`.claude/singlefs-ai-sop/rules/rules-discipline.md:61`）；agent 定义「同规矩，检查各自另立」（`.claude/singlefs-ai-sop/rules/rules-discipline.md:7`）；共用约束自己的开头写「这份与各份定义只写怎么做：步骤、约束、判据、交付。」（`.claude/agent-common.md:4`）。`main-agent.md`、`agent-common.md` 不在上游 doc-lint 那一条的文件名射程里（该判据的条件是「"$f" == */agents/*.md || "$f" == */skills/*/SKILL.md」，`.claude/singlefs-ai-sop/scripts/doc-lint.sh:523`），我仍按同一把尺子读。

### 3.1 机械那一半（原样输出）

做法：把 diff 文件里射程内（规则、定义、skill、`main-agent.md`、`agent-common.md`）的加行，拿上游 doc-lint 的上下文指代判据（`ctx` 那几行抄成 python 正则）与自称词「本节、本章节、本表、本文档、本条」各跑一遍。脚本 `research/prompts/defs-m3-formal-gate-r1-sonnet-model/d5-scan.py`，输出：

```
命中 .claude/main-agent.md diff 文件第 229 行 '下面那'
扫了 30 条射程内的加行，命中 1 条
```

那一条是 `main-agent.md` 那一行表格里沿袭的「合入后验证」「（下面那一行）」（`.claude/main-agent.md:61`），不是这一道新写的字；HEAD 里同一句也在（`git show HEAD:.claude/main-agent.md | grep -c '下面那一行'` 输出 1）。这一道新增的字里：日期 0 处，位置指代词 0 处，自称词 0 处。逐词比对 diff 抽出这一道的新增片段，存在草稿目录 `ins.txt`（不入库）。

### 3.2 逐句判（四样 vs 论证）

| 位置 | 新增的句子 | 判 |
|---|---|---|
| `crash-verifier.md:22` | 「每个 Verus 进程一个（空真与必红世界几处同时验，每处各占一份）」（`.claude/agents/crash-verifier.md:22`） | 括注是判据的一部分：它说明上限按进程给、一道里会有几个同时在跑，主 agent 才知道总量；去掉括注，给的人不知道「每个」指什么。不是论证。 |
| `crash-verifier.md:29` | 「checker-tier-formal-verification 也在里面套了（每次 Verus、E165 装置各经包装）」（`.claude/agents/crash-verifier.md:29`） | 「做什么、不做什么」的射程句，留；无日期。 |
| `crash-verifier.md:28` | 「checker-tier-formal-verification 查 `crates` 与 `research/results/e165-*-red-examples.tsv`」（`.claude/agents/crash-verifier.md:28`） | 「什么在哪里」，留。 |
| `main-agent.md:52` | 「checker-tier-formal-verification 给两个（每个 Verus 进程一个、E165 装置一个）」（`.claude/main-agent.md:52`） | 判据，留。 |
| `implementation-workflow.md:74` | 「形式化验证由门禁 checker-tier-formal-verification 判」（`.claude/rules/implementation-workflow.md:74`）一句话 | 内容是「什么规则在哪里」，不违四样；两个问题见 3.3、3.4。 |
| `verification.md:98` | 新增表格行「形式化验证：恢复重放取前缀的核经 Verus 验过」（`.claude/rules/verification.md:98`） | 「判什么 / 谁判」表的一行，与同表其余行同形；问题见 3.4。 |
| 其余（清单里补一个门禁名、数词四改五） | — | 不含判断，无论证。 |

### 3.3 落点（中）：`implementation-workflow.md:74`

那一句接在「门禁管哪一半」段的末尾，而那一段住在标题「## 测试与崩溃检测优先多线程」（`.claude/rules/implementation-workflow.md:69`）之下，前文是崩溃点重放与「别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。」（`.claude/rules/implementation-workflow.md:74`）。形式化验证与多线程无关，找「第五道门禁在规则哪里定义」的人不会去这一节。同样的内容已经在三处：`implementation-workflow.md:32`（复用）、`:53`（每次提交代码一行）与 `verification.md:98`（判什么、谁判），所以这一句是第四份。我不判留不留；若留，落点与重复要主 agent 定。

### 3.4 相对时间词（低）：「今天的实现」

`implementation-workflow.md:74` 与 `verification.md:98` 各有一处「在今天的实现上逐行重放」（`.claude/rules/implementation-workflow.md:74`、`.claude/rules/verification.md:98`）。「今天」随读的日子变，规则正文读起来是「当前」。这个词抄自阶段文件头格名表同一句；同样的词在 14 份规则、定义与手册里都有（`grep -c '今天'` 命中 14 份文件），所以只评低。推翻：主 agent 判这类词在规则里本来就算「当前」，就不改。

### 3.5 射程外的两句（只列）

`heavy-test-guard.sh` 文件头里这一道的说明带了理由：「（写证明的实现员要用，判法在 lib_heavy_tests.py）」（`.claude/hooks/heavy-test-guard.sh:48`）；`lib_heavy_tests.py` 的注释带了实测：「验一个 crate 是秒级（接回可行性那一趟量到 8–21 秒）」（`.claude/hooks/lib_heavy_tests.py:61`）。钩子文件不在 `rules-discipline.md` 第 61 行的射程里，只列出，不判。

## 四、D1 谁跑、什么时候跑（照字面逐个角色推）

探针：`research/prompts/defs-m3-formal-gate-r1-sonnet-model/probe.py`（25 格，只喂 JSON 给 `.claude/hooks/heavy-test-guard.sh`、只看退出码，被判的命令不执行；`rerun.sh` 一键复跑，`SHA256SUMS` 同目录）。输出末行：「共 25 格，期望对不上的 0 格」。

### 4.1 角色表（探针退出码与字面对照）

| 角色 | 字面要它怎么跑 | 探针（退出码） | 判 |
|---|---|---|---|
| crash-verifier | 提交时（或转达用户要求时）命令带 `SINGLEFS_HEAVY_TESTS=commit` 跑五道，这一道与 54 号并行起（`.claude/agents/crash-verifier.md:31`） | D1-01 前缀加 nice 退 0；D1-02 前缀、两个上限、nice 退 0；D1-03 次序反过来退 0；D1-04 后台花括号写法退 0；D1-05 外面再包内存包装退 0；D1-07 `=user-request` 退 0；D1-08 单跑 `--check verus-proof` 退 0；D1-25 第 1 步的 `stage-marker-check` 退 0；D1-06 不带前缀退 2 | 一致 |
| 主 agent | 带前缀也能单独跑（`.claude/hooks/heavy-test-guard.sh:48`）；派 crash-verifier 跑这五道（`.claude/main-agent.md:66`） | D1-14 带前缀退 0；D1-15 不带退 2 | 一致 |
| gate-triage | 起整轮 `gate-staged.sh` / `gate.sh`；这一道「你不直接调它们」 | D1-12、D1-13 退 0；D1-10 带前缀直接调这一道退 2；D1-11 带前缀 `--check` 单跑一格退 2 | 一致（歧义见 4.5） |
| implementation-writer 及其余子 agent | 不跑这一道；直接起 `verus` 验自己的证明不算重型（`.claude/hooks/lib_heavy_tests.py:61`） | D1-16 带前缀退 2；D1-17 `--check verus-proof` 退 2；D1-24 `--tool-versions` 退 2；D1-18 直接起 `verus` 退 0 | 一致 |

`--list` 不带前缀退 0（D1-09），与文件头「带 --list、--list-items…哪一道都不算」一致。三处判定对得上：`verus-stage` 在类别表（`.claude/hooks/lib_heavy_tests.py:132`）、阶段名表（`.claude/hooks/lib_heavy_tests.py:152`）、crash-verifier 的放行集合（`.claude/hooks/heavy-test-guard.sh:175`），gate-triage 的集合没有它（`.claude/hooks/heavy-test-guard.sh:176`）。闸的拒绝文字与结果同向：D1-10 拒因是「gate-triage 不跑「Verus」」。

### 4.2 整轮 `gate.sh` 里它怎么走：冲突（中，沿袭前四道的写法）

字面：「这五道在 `gate.sh` 里只核标记与复用判定，它不直接调」（`.claude/main-agent.md:66`）。
实现：阶段开头先问复用「if ( stage_run_or_skip」（`.claude/gate.d/checker-tier-formal-verification.sh:125`），退 77 才整道不跑；不作数就往下「stage_cells_run "$ROOT"」（`.claude/gate.d/checker-tier-formal-verification.sh:140`）逐格真跑六格。复用的两条路都不作数时它真跑：先看全绿标记「if not break_is_set("stage-marker-ignored"):」（`research/scripts/admission.py:1126`），再看暂存树与 `refs/sop/staged-green`、24 小时上限与 `environment=`（`research/scripts/admission.py:1178` 的 `judge_gate_environment`）。
对照：`gate-triage.md:28` 的「这几道重阶段在 `gate.sh` 里照各自的复用判定走」（`.claude/agents/gate-triage.md:28`）是对的，`main-agent.md:66` 的「只核标记」不对。前四道的阶段同样先 `stage_run_or_skip` 再真跑（`.claude/gate.d/checker-tier-lkmm.sh:65`、`.claude/gate.d/checker-tier-crates-mutation-replay.sh:200`），所以这句话本来就这么写，不是这一道独有。
照字面做会做错的情形：提交时若 crash-verifier 没跑过这一道、或跑过之后树变了，gate-triage 起的整轮会在自己手里真跑 Verus 六格并编装置；而 gate-triage 的输入没有这一道的两个内存上限（`.claude/agents/gate-triage.md:20` 起的输入一节没有内存上限这一项），阶段用默认值：「"memory_cap": os.environ.get("GATE_VERUS_MEMORY_MAX") or "4G"」（`research/scripts/verus-proof-check.py:737`）与「"memory_cap": os.environ.get("GATE_E165_PINNED_MEMORY_MAX") or "2G"」（`research/scripts/e165-pin-red-examples.py:411`）。
推翻它的现象：在 `--staged` 的整轮里造「没有标记、没有 staged-green」，看这一道有没有出六格的 ✓ 行。我没跑（重型，见「没做什么」）。

### 4.3 `crash-verifier.md` 第 2 步的后台模板没带两个上限：冲突（中）

第 1b 步写：「输入给的两个上限设进阶段读的变量，写在 `nice` 前面」（`.claude/agents/crash-verifier.md:29`）。第 2 步给的后台起法模板是：「{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1」（`.claude/agents/crash-verifier.md:31`），里面没有那两个变量，且同一句话写「五道的命令都带输入给的那个前缀」。变量名与阶段读的一致（`research/scripts/verus-proof-check.py:737`、`research/scripts/e165-pin-red-examples.py:411`），闸对变量与前缀的次序都放行（D1-02、D1-03）。执行员照抄第 2 步的模板而漏看第 1b 步，这一道会按默认 4G / 2G 跑，输入里「量过的」上限被丢，报告里写的上限与实际不符。推翻：看 crash-verifier 实跑时的命令行里有没有这两个变量。

### 4.4 第 6 步指纹不含这一道的四类输入：冲突（低，沿袭）

第 6 步的指纹命令是「`git diff HEAD -- crates litmus Cargo.toml Cargo.lock research/scripts | sha256sum`」（`.claude/agents/crash-verifier.md:35`）；这一道登记的输入另有 `.claude/gate.d/verus-pins.tsv`、`.claude/gate.d/e165-pinned-red-examples.tsv`、`research/results/`、`research/mutations/e165_replay_bounded_enumeration_custom.tsv`（`.claude/gate.d/stage-inputs.tsv:31`，同一行读得到）。这四处在两个阶段之间变了，第 6 步的「前后指纹相同」仍成立，「这几个绿对应的不是同一份源码」就报不出来。开跑前第 1 步的 `git diff --quiet` 用这一行的全部路径，覆盖了开跑那一刻，没覆盖跑的过程中。QEMU 那一道读 `research/results/` 同样不在第 6 步的指纹里，所以是沿袭。

### 4.5 `gate-triage.md:28` 的「单跑某一格」措辞：歧义（低，沿袭）

「单跑某一道或某一格（`--check <格名>`）也带同一个前缀」（`.claude/agents/gate-triage.md:28`）。对这一道与另外四道，带前缀也被拒（D1-10、D1-11）；能单跑的只有整轮之外的 experiment-replay 格与提交时才跑的检查。上一句已写「你不直接调它们」，读的人多半不会误起，所以评低。

### 4.6 绕过闸的一条路：判法脚本直接起（中，事实，是否算「绕」由主 agent 判）

闸按阶段文件名认这一道：「只有门禁那一道（逐格验、空真与必红世界逐处起 Verus、编装置）算」（`.claude/hooks/lib_heavy_tests.py:62`）；python 脚本「不读正文，按名字与位置…认」只给了广谱变异（`.claude/hooks/heavy-test-guard.sh:42`），已知脚本表里没有这两份判法（`.claude/hooks/lib_heavy_tests.py:159` 起的 `KNOWN_SCRIPT_LOCATIONS`）。探针：

```
   退 0 期望 — | D1-19 实现员：python3 起判法脚本验证明（门禁那一道的判法） | agent=implementation-writer
   退 0 期望 — | D1-20 实现员：python3 起判法脚本跑必红世界（94 处 Verus） | agent=implementation-writer
   退 0 期望 — | D1-21 实现员：python3 起红例重放（编装置、跑装置） | agent=implementation-writer
   退 0 期望 — | D1-22 门禁分诊员：python3 起红例重放 | agent=gate-triage
```

这四条命令做的是门禁那六格里同样的活（`research/scripts/verus-proof-check.py` 起 Verus、`research/scripts/e165-pin-red-examples.py --replay` 编并起装置，都经内存包装，`research/scripts/verus-proof-check.py:737` 与 `research/scripts/e165-pin-red-examples.py:411`），闸全部放行。正文说子 agent 不跑的是「形式化验证（门禁 checker-tier-formal-verification）」（`.claude/agent-common.md:52`），字面上只点门禁，所以没有哪句话被违反；闸与正文一致，缺口是「做同样的活」的另一条路。推翻：把这两份脚本名加进已知脚本表后，D1-19 到 D1-22 退 2。必红世界那一条每次起 94 处 Verus，我没跑，只看了闸。

### 4.7 派发情形

- 派 crash-verifier 少给两个内存上限：派发闸读定义 frontmatter 的 `required-inputs:`（`.claude/hooks/runner-dispatch-guard.sh:123`），`crash-verifier.md` 没有这一项（`grep -c required-inputs .claude/agents/crash-verifier.md` 输出 0，`investigator.md` 输出 1），所以闸不拦；只有共用约束一句「输入缺一样就不开工，回复只写缺什么」（`.claude/agent-common.md:12`）靠执行员自觉。评低。
- 派 crash-verifier 不写「重型测试：不跑」：放行，因为它在持有人集合里（`.claude/hooks/runner-dispatch-guard.sh:69`）。
- 派别的类型去跑这一道：提示里没有「重型测试：不跑」一行时派发被拒；写了那一行又要它跑，派发放行、执行时被拒（D1-16），与派发闸文字「执行时 .claude/hooks/heavy-test-guard.sh 也拒」（`.claude/hooks/runner-dispatch-guard.sh:579`）一致。

### 4.8 复用路径与 D2 的交界（只记，不判）

crash-verifier 第 1 步用 `stage-marker-check`，「退 0 就抄它打的 ok 行、这一道记「复用」不跑」（`.claude/agents/crash-verifier.md:28`）。该子命令只看标记在不在与有没有过 24 小时：「return True, [f"ok {marker_path} {finished_text}"]」（`research/scripts/admission.py:2252`），标记的键是输入指纹，指纹里有工具链与构建环境，没有 Verus 版本（`stage_fingerprint` 的说明句列了登记路径、登记表、阶段脚本、登记行、工具链、构建环境）。这一条路上不比 `environment=`；比环境的在 `judge_gate_environment`，只在整轮的暂存树那条路上走。归 D2，我没造「换 Verus、输入没变」的情形。

### 4.9 `crash-verifier.md` 没有给这一道写「不由你外推」（低，建议性）

「全绿只说明这几道的证据要求满足；层 0 覆盖到哪几条流…看 `.claude/gate.d/checker-tier-layer0-replay.sh` 文件头与 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行，不由你外推。」只给层 0 写了范围（`.claude/agents/crash-verifier.md:48`）。这一道的四个 Verus 格验的是单独 crate `crates/singlefs-replay-prefix` 的核，真实恢复代码一侧只由红例钉表的两格按有限的钉住行覆盖（`.claude/rules/verification.md:98` 的行写了这两样）。执行员报「形式化验证绿」可能被读成「恢复代码整体被证明」。我不判要不要加，也没核真实恢复代码与核之间的依赖关系。

## 五、判定一览

| 格 | 判定 | 一句话 | 什么现象会推翻 |
|---|---|---|---|
| D4 被判 11 份里的清单 | 一致 | 凡列提交时重型门禁处都点了，点法与前四道同形，没有多出 | 再扫出一处并列三个以上门禁名而没有这一道的行 |
| D4-a `agent-common.md:54` | 冲突（低） | 「在里面逐条套了」清单里没有这一道，也没有两个变量名 | 补上后 `grep` 该行含 `checker-tier-formal-verification` |
| D4-b `gate-triage.md:29`、`:27` | 冲突（中） | 25x 分类只认两道的措辞，这一道两种包装退出（「内存包装自己的结局」「装置退」）认不出 | 造一次 250 退出，gate-triage 仍判「环境（上限）」 |
| D4-c `heavy-test-guard.sh:1286`、`:606` | 冲突（低） | 自检成功行与 POLICY 第二段的清单没有 Verus | 补上 |
| D4-d `implementation-writer.md:58`（文件外） | 漏（中） | 「checker 档（…）归 crash-verifier」少这一道 | 补上 |
| D4-e `crash-verifier.md:39` | 提示 | 写范围点了变异那一道的编译目录，没点这一道的 | — |
| D4-f 文件外同类清单 | 漏（低到中） | `crash-test/SKILL.md:17` 的「四样」表、`admission.py:70`、`admission.py:1124`、`stage-must-run.sh:17`、`code-tooling.sh:669`、`README.md:126` | 主 agent 判这些不归这一轮 |
| D5 机械（日期、位置指代、自称） | 一致 | 这一道新增的字里 0 处；射程内 30 条加行只命中沿袭的「（下面那一行）」 | 换一份更严的判据扫出新命中 |
| D5 论证与经过 | 一致 | `crash-verifier.md:22` 的括注是判据，其余是「什么在哪里」 | 主 agent 判那条括注是论证 |
| D5 落点 | 冲突（中） | `implementation-workflow.md:74` 放进多线程小节，且是第四份重复 | 主 agent 判这一句应留在那里 |
| D5 相对时间词 | 低 | 两处「在今天的实现上」 | 主 agent 判这类词在规则里算「当前」 |
| D1 谁跑、带什么前缀 | 一致 | 25 格探针（四个角色）0 格与字面不符 | 再造一条字面写了而闸判反的命令 |
| D1 `verus-stage` 与 `lib_heavy_tests.py:132`、`:152` | 一致 | 三处判定对得上，gate-triage 集合不含它 | — |
| D1 整轮 `gate.sh` | 冲突（中，沿袭） | `main-agent.md:66` 写「只核标记与复用判定」，阶段在复用不作数时真跑六格 | 在 `--staged` 下造无标记场景，这一道不出六格 ✓ 行 |
| D1 `crash-verifier.md` 第 2 步模板 | 冲突（中） | 后台模板没带两个上限，照抄会用默认 4G / 2G | crash-verifier 实跑命令行里有这两个变量 |
| D1 第 6 步指纹 | 冲突（低，沿袭） | 指纹不含这一道的 `verus-pins.tsv`、红例钉表、`research/results/`、红例变异表 | 把这四处并进指纹后，改它们会报「不是同一份源码」 |
| D1 `gate-triage.md:28` 单跑措辞 | 歧义（低，沿袭） | 对这一道带前缀 `--check` 也被拒 | — |
| D1 判法脚本直接起 | 事实（中） | 四条 python 命令做同样的活，闸全部放行（D1-19 到 D1-22） | 加进已知脚本表后这四条退 2 |
| D1 派发 | 一致，另有一缺（低） | `crash-verifier.md` 没有 `required-inputs:`，两个上限靠自觉 | 加上后派发闸拒缺上限的提示 |
| D1 复用路径与 D2 的交界 | 只记 | `stage-marker-check` 不比 `environment=` | 归 D2 |
| D1 `crash-verifier.md:48` 的范围句 | 建议（低） | 只给层 0 写了「不由你外推」 | — |

## 六、没做什么

- 没跑整道门禁、没跑 Verus、没起 E165 装置、没跑整轮 `gate.sh`、没跑任何 `--selftest`、没执行被判的命令。4.2「整轮里真跑」与 4.3 的后果是读阶段脚本与 `admission.py` 推出来的，没有在 `--staged` 的临时树里造「无标记」场景核。
- 没造 D2 的「换 Verus、输入没变」复用情形，没验 D3 的并行争编译目录与 600 秒限时，没验交回闸 `handback-scratch-check.sh` 认不认 `${TMPDIR:-/tmp}/singlefs-e165-pinned-target`，没核 `research/scripts/memory-peaks.tsv` 里有没有这一道整条命令的峰值（`crash-verifier.md:22` 要求写明「量过的」还是「推的」）。以上归 Opus 腿的格，不替它判。
- 被判 diff 里与这一道不相干的改动夹在同一块里（广谱变异、`detached-run.sh` 剥壳、`test-script-discipline.md` 迁移与测试文件起名、崩溃放量行的复用与续跑句、双机分片旁的「别的会话」改字、`HEREDOC_DETACHED_RUN` 弄坏开关），我没判，也没把它们算进上面任何一格。
- 没替主 agent 采纳或出判决；没读禁读清单里的文件与目录（Opus 腿的报告与模型）。
- 草稿在 `/tmp/claude-1000/defs-m3-formal-gate-r1-sonnet/`（`matrix.txt`、`ins.txt`、`added.tsv`、`files-*.txt`、`probe-out.txt`、`hook-detections.jsonl`），只有扫描与探针输出，没有编译目录、工作树与仓副本，所以没有要删的；不入库，因为结论里引的数与行都已在报告或模型目录里复现（探针与 D5 扫描脚本在 `research/prompts/defs-m3-formal-gate-r1-sonnet-model/`）。

## 七、交回前核对

```
$ sha256sum -c research/prompts/defs-m3-formal-gate-r1-snapshot/sha256sums.txt | grep -c ': OK$'
17
$ wc -l < research/prompts/defs-m3-formal-gate-r1-snapshot/sha256sums.txt
17
```

17 份都对上，没有对不上的。

```
$ python3 research/scripts/cite-check.py research/prompts/defs-m3-formal-gate-r1-sonnet-output.md --root . | tail -1
  ✓ 核了 49 处引文，对上 49 处，没判 0 处（逐处列在上面）
```
