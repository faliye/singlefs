# gate-shrink-r1 判决（2026-09-28，主 agent singlefs-8f）

正文 `research/prompts/_gate-shrink-r1-body.md`；背景材料 `_gate-shrink-r1-background.md`；附录二 `_gate-shrink-r1-diff.md`；盘点 `gate-shrink-r1-inventory/`。

## 开工快照核对

`sha256sum -c research/prompts/gate-shrink-r1-snapshot/*-sha256.txt`：对不上两份。`.claude/gate.d/20-doc-decision-documents.sh` 在腿工作期间被别的会话改过（云端攻方报告末节「工作期间别的会话改了 20 号与 doc-lint.sh，行号已按现查改过」）；`records/2026-09-28-门禁59号提速与双机分片.md` 是主 agent 自己追加的计划行，三条腿都禁读它。两份云端腿报告的引文用 `python3 research/scripts/cite-check.py research/prompts/gate-shrink-r1-sonnet-output.md research/prompts/gate-shrink-r1-opus-output.md --background research/prompts/_gate-shrink-r1-background.md --background research/prompts/_gate-shrink-r1-diff.md` 按现在的文件核：「✓ 核了 38 处引文，对上 38 处，没判 0 处」。

## 腿与样本

| 腿 | 产物 | 判定 |
|---|---|---|
| 云端正推（Sonnet） | `research/prompts/gate-shrink-r1-sonnet-output.md` | 采信的逐条见下 |
| 云端攻方（Opus） | `research/prompts/gate-shrink-r1-opus-output.md`，模型 `research/prompts/gate-shrink-r1-opus-model/`（`rerun.sh`、`SHA256SUMS`、`outputs/`） | 采信的逐条见下；模型里标「量过」的是副本上的数，判决引的几处主 agent 在主仓现查过原文 |
| 本地攻方 | `research/prompts/gate-shrink-r1-local-attack-output-s1.md`（干净）、`research/prompts/gate-shrink-r1-local-attack-output-s2.md`（干净）、`research/prompts/gate-shrink-r1-local-attack-output-void1.md`（损坏闸判红，参考样本：单字母替换 alter×84）、`research/prompts/gate-shrink-r1-local-attack-output-void2.md`（损坏闸判红，参考样本：同一处替换另加 40 个生词）；提示 `research/prompts/gate-shrink-r1-local-attack.md`，译文核对 `-translation-audit.md`，运行记录 `-runlog.md` | Q-COUNT-1（旧格落不进任何组）两份都 0；Q-COUNT-2（重复）两份都 0；Q-COUNT-3（各组格数与方案）两份不一致：s1 报第 1、2、5 组 39、12、4 格，s2 报全对上。主 agent 现数 `cells.tsv`：doc-kb 37、doc-experiments 10、code-source-discipline 8，与方案相同——s1 那一格现查推翻、不采，s2 采。TB-Q1（删 row27 之后谁判）两份都答「没人判」，与云端攻方 B1 一致；TB-Q2、TB-Q3 两份都指到共享门禁，与云端攻方 B2、B3 一致。两份参考样本在这几格上没有新的反例或数 |

## 逐问判决

| 问 | 判决 | 依据 |
|---|---|---|
| 1 合并丢不丢格 | **不丢**。84 格都有去处、没有重复 | 正推「84 格全部找到去处」；本地两份干净样本 Q-COUNT-1、2 都 0；主 agent 现数各组格数与方案相同 |
| 2 大道并成一道 | **打中，采**。按道判的两件——判别力样本的退出码、分诊与派修——并成一道之后不再分辨格 | 云端攻方第二节，M1 量过：红样本 28/28 在别的四道里也退 1、绿样本 8/8 在别的道上判红、want 5/328 由别的道替打；分诊表与 `stage-owners.tsv` 按道一行。正推说「一格红拖累整道」在第一轮合出的 20、32、47 号里就有——对，所以这是今天就有的毛病，不只是 doc-kb 的 |
| 3-B1 删 row27-preconditions | **打中，不删** | 云端攻方 B1，M7 V1 量过：`instance_table.rs` 里出现 `rows.retain(` 时只有这一格判（`.claude/gate.d/43-checks-owed-and-closeout.sh:760` 的探针），C333 仍在欠账表（`.claude/kb/checks-owed.md:279`）；本地两份 TB-Q1「没人判」 |
| 3-B2 | **条件打中**：只删钩子自证的重复调用，钩子注册与 matcher 的判据留着 | 云端攻方 B2，M7 V2 |
| 3-B3 | **没打中，采方案**：删与 doc-lint F 重叠的那一半 | 云端攻方 B3，M7 V3：doc-lint 仍以 G 判红 |
| 3-B4 74 号 | **打中，两臂一起落空**：模型对拍那几段在 `#[ignore]` 用例里，74 号与「构建与单测」都不跑 | 主 agent 现查：`.claude/gate.d/74-model-differential.sh:48` 跑 `cargo test --release -p singlefs-harness --test "$TEST_BINARY" -- --nocapture`（第 32 行 `TEST_BINARY="random_histories"`），不带 `--include-ignored`；`crates/singlefs-harness/tests/random_histories.rs` 26 个 `#[test]` 里 19 个标了 `#[ignore]`；云端攻方实跑 74 号 `outputs/stage74-run1.log` 末行「测试跑过了，这几段却没有「模型对拍 N 步」那一行」，rc=1 |
| 3-B5 | **条件打中，采攻方的认法**：只认 .sh / .py，去掉注释与出路文字后代码里还有 `--selftest` | 云端攻方 B5 与第六节，M2 量过：今天 53 份认 50 份，三份误认去掉 |
| 3-B6 77 号次序 | **部分打中**：依赖次序的是与时刻无关的几类（残留设备、宿主盘剩余空间）；起名 `harness-tests` 会排在 `harness-test-environment` 后面 | 云端攻方 B6、第三节 1，M4 量过 |
| 3-B7 | **采攻方的两处收法**：term-renames 掩码正则加左边界；research-script-selftests 的 runner 分批并行 | 云端攻方第七节：M6 1041 秒到 1.76 秒、逐份输出相同；M9 两次 401 秒到 178、137 秒、45 条都退 0（只两次观测，写明不算稳定） |
| 3-B8 | **打中，不删** experiment-refs、decision-refs | 云端攻方 B8，M7 V8 |
| 3-B9 | batch-scope 判据对、缺成批登记的写法；knowledge-sync 判据对、红是流程欠账（这一轮自己也要写阶段同步记录）；archive-past-rounds 在多会话下判据不合用（攻方推的） | 云端攻方 B9；M8 算过 batch-scope 在暂存树上照样红（188 个触发文件 151 个没登记） |
| 4 次序与改名 | 77 号见 B6；47 号两处认「两位数开头」的 glob（`[0-9][0-9]-*.sh`）去编号时要改，否则部分改名时 fixture-claims 静默跳过；**按全名指门禁从此没人判**（governance-refs 只认「两位数加号」与反引号路径，M5 量过）——打中，采 | 云端攻方第三节 4、5 |
| 5 定义改动 | 这一轮改的那几块（Q8、Q9、Q15 与改门禁号）没有互相矛盾的句子；**两处不是这一轮的**：`.claude/rules/three-way-inference.md` 与 `.claude/main-agent.md` 里撤掉「核查员按轮派」的文字、`.claude/rules/verification.md` 的耗时用例量法改写（连带 `research/scripts/harness-test-timing.py` 删除），不在这一轮判；**一处是这一轮的、要改**：`.claude/rules/implementation-workflow.md`「双机分片」那一条接的 59 号双机写法，与用户同日两条定案不符（59 号整套重设计归 singlefs-29；双机与 GPU 都要从本地配置读 `ENABLE_ACROSS_MACHINES` / `ENABLE_GPU` 才开） | 正推第 5 问；`crash-verifier.md` 把 `stage-owners.tsv` 当「提交时跑哪几道」读，与第二列新意思「判红时派给谁修」要连着读才说得通——采正推的意见：`crash-verifier.md` 自己写明跑哪几道，不再靠这张表 |
| 6 只在提交时跑的代价 | **打中一处硬伤**：34 号 evidence-in-repo 在 `gate-staged.sh` 的链接 worktree 里整格退 77，提交路径上它从来不判 | 主 agent 现查 `.claude/gate.d/34-doc-experiment-pages-and-products.sh:509`、`:526`–`:527`：链接 worktree 退 77，理由是时间戳是检出时写的；它的判据二（kb 与新写的提示拿 `/tmp` 当依据）不看时间戳，照样该判。写入时拦的 W1–W4 与用户「扫仓的门禁只在提交时跑」有交集，交用户 |

## 采纳的改法（第 4 步实施）

1. **每道门禁的结构**（用户同日定「门禁中的每一条都可以单独出来跑 不要编程一个测试函数 要可以全跑 也可以分项跑」）：格名表、`--list`、`--check <格名>[,<格名>…]`、不带全跑、格名写错退 2；每格一个函数；重型门禁逐项可点名。**另加两样，治第 2 问**：
   - 样本按格跑：被判的根里有标记文件 `.gate-cells`（一行一个格名）、而且被判的根不是阶段所在的仓时，只跑点名的格——stage-selftest 只给根与 `--force`，这样每一格的红绿样本只判它自己；照 47 号已有的样本标记做法。
   - 归属按格登记：`stage-owners.tsv` 改成一行一个「门禁、格名、派给谁修、为什么」；门禁分诊按格报。
2. **收缩后的门禁**（第 2 问改了方案 A 的文档类；用户同日定 15 与 87 合、74 并进 harness 类、55 单留、92 与 94 合）：

| 类 | 门禁（去编号后的名） | 由哪几道并成 |
|---|---|---|
| 文档类 | `doc-decisions` | 20 doc-decision-documents、30 decision-history-entries |
| 文档类 | `doc-registries` | 32 doc-field-and-layout-registry、43 checks-owed-and-closeout（row27 那一格留下、改名为说清它盯收口表前置的名字）、10 references-and-invariants |
| 文档类 | `doc-experiments` | 34 doc-experiment-pages-and-products |
| 文档类 | `doc-process-records` | 11 review-and-sync-records |
| 文档类 | `doc-text` | 12 doc-forbidden-notations-and-old-terms |
| 代码类 | `code-source-discipline` | 13、27、33 |
| 代码类 | `code-tooling` | 47 code-tooling-selftests-and-registries |
| harness 类 | `harness-model-differential-and-scenarios` | 14、74（74 号那几段带 `--include-ignored` 跑点名的用例） |
| harness 类 | `harness-test-environment` | 77（名字排在 harness 类最后；工具层自检加一格判它排在全部跑测试的阶段之后，不靠碰巧的字母序） |
| checker 类 | `checker-independence-and-sync` | 92、94 |
| checker-tier 类 | `checker-tier-research-build-and-replay` | 15、87（两格：单测、复跑，只编一次） |
| checker-tier 类 | `checker-tier-qemu-device-streams`、`checker-tier-lkmm`、`checker-tier-crates-mutation-replay` | 55、57、59 |
| checker-tier 类 | 54 | 名字随 KV 那一批由 singlefs-29 改；这一轮只补最小的 `--list` / `--check` |

   合计 14 道（文档类 5、代码类 2、harness 类 2、checker 类 1、checker-tier 类 4 加 54）。
3. **精华**：B2（只删钩子自证的重复调用）、B3（删与 doc-lint F 重叠的一半）、B5（攻方认法）、B7（正则左边界、runner 分批并行）照采；B1、B8 不删。
4. **修硬伤**：74 号跑点名的 ignore 用例；evidence-in-repo 的判据二在链接 worktree 里照判；governance-refs 认门禁全名与格名；47 号两处两位数 glob；59 号双机只在本地配置 `ENABLE_ACROSS_MACHINES=1` 时开，驱动在判法退 3 时报「双机：关」；`.claude/rules/implementation-workflow.md` 那一条照两条定案改；`crash-verifier.md` 自己写明提交时跑哪几道；`peer-host-lib.sh` 循环里 kill 改成按进程号一次一个（工具层自检报的）。
5. **去编号改全名、文档同步**：调度记录第 4 步那一行的出口与判据。

## 这一轮审过的定义与规则（逐份按路径）

判的是附录二里标为这一轮（gate-shrink，即 Q8、Q9、Q15 与改门禁号）的那几块；标为别的会话的那几块不在这一轮判。

| 文件 | 判定 |
|---|---|
| `.claude/agent-common.md` | 认：「门禁」一节改成交回前不跑门禁阶段、归属表第二列改成判红时派给谁修；改门禁号那几处认 |
| `.claude/main-agent.md` | 认这一轮那几块（第 10 条、改门禁号）；撤核查员的文字不是这一轮的，不判 |
| `.claude/agents/kb-scribe.md` | 认这一轮那几块（交回前不跑阶段、撤写入后钩子、改门禁号）；变更史写法那几块归 singlefs-0a 的 changelog-format-r1，不判 |
| `.claude/agents/experiment-runner.md` | 认 |
| `.claude/agents/tooling-writer.md` | 认（第 7 步只跑自己动过的阶段的样本） |
| `.claude/agents/three-way-materials.md` | 认（第 0 步改成跑合并后那一格） |
| `.claude/agents/prior-art.md` | 认（第 0 步改成 `--check citations`） |
| `.claude/agents/crash-verifier.md` | 认改门禁号；要补一处：自己写明提交时跑哪几道（第 5 问） |
| `.claude/agents/implementation-writer.md` | 认改门禁号；交回前跑登记给它的阶段那一句照 Q9 删（己件报告列的遗留） |
| `.claude/agents/investigator.md` | 认改门禁号 |
| `.claude/agents/kb-spec-drafter.md` | 认改门禁号；变更史写法那几块归 singlefs-0a，不判 |
| `.claude/rules/implementation-workflow.md` | 认改门禁号；「双机分片」那一条接的 59 号写法要照用户两条定案改（第 5 问） |
| `.claude/rules/implementation-first.md` | 认 |
| `.claude/rules/three-way-inference.md` | 认改门禁号那几处；撤核查员的文字不是这一轮的，不判 |
| `.claude/rules/format-evolution.md` | 认改门禁号那几处；变更史写法那几块归 singlefs-0a，不判 |
| `.claude/rules/mutation-sampling.md` | 认 |
| `.claude/rules/verification.md` | 认 12、13、14、54 号那几处；耗时用例量法的改写不是这一轮的，不判；加一节「门禁的结构」（第 4 步） |
| `.claude/rules/path-moves.md` | 认 |
| `CLAUDE.md` | 认这一轮那几块（第 22 行门禁归提交时、改门禁号）；`@.claude/rules/changelog-format.md` 那一行归 singlefs-0a，不判 |

## 交用户（被攻过零轮的，照实标）

| # | 事 | 为什么交用户 |
|---|---|---|
| U1 | 文档类拆成 5 道（上表）而不是 1 道 doc-kb | 主 agent 在判决里推出的组合，被攻过零轮；拆法按攻方给的判据「修它们的是不是同一个 agent、同一个写范围」大致分，没逐格量 |
| U2 | 写入时拦的 W1（kb 与新写的提示拿 `/tmp` 当依据）、W2（几个会话共用的编号撞号）：加进 write-guard 的禁用写法 | 与「扫仓的门禁只在提交时跑」不冲突（write-guard 是写入那一刻的单点拒绝，不是扫仓），但是新行为、被攻过零轮 |
| U3 | archive-past-rounds 保护还在跑的轮、batch-scope 认成批登记 | 攻方推的，被攻过零轮；这一轮不做，记欠 |
| U4 | 74 号修好之后真跑模型对拍那几段：单条 debug 下 60 秒以上的耗时用例，跑出来红的话是 `crates/` 或测试的问题，归别的会话 | 这一轮只让它被跑到 |

## 回看决策

不涉及决策：这一轮只动门禁、钩子、研究脚本、协作定义与规则，`.claude/kb/decisions/` 下的决策正文与分项一条不改。
