# governance-defs-r2 核查员报告

我交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证

取 Sonnet 报告表 G1#1 引用 `.claude/agents/gate-triage.md:26`（原文与 Sonnet 抄的一致，见下）。把行号加 1 变成 27，`awk 'NR==27' .claude/agents/gate-triage.md` 取到的是完全不同的一句（讲红阶段怎么判「这一轮／不是这一轮」），与被核的引文毫不相干 ⇒ 判 ✗。核查方法本身分辨得出错误行号。

## 输入核实

- 42 个文件的开工快照 `research/prompts/governance-defs-r2-snapshot/sha256sums.txt`：`sha256sum -c` 现跑，42 行全部 `OK`，与主树一致（我自己重跑过，不是照抄主 agent 的说法）。
- 两份报告文件现有 sha256 与主 agent 给的交回值一致：
  - `governance-defs-r2-sonnet-output.md` → `6d720dba38e19b077289134bfd12d7014748290f92ca959b6569ada226fa48e8`（现算相符）
  - `governance-defs-r2-opus-output.md` → `523511e074c5117cc023ce19ae06f30adbae5b02170591fd260a01ef362d33cb`（现算相符）
  → 两份报告都不落「分不清：报告在腿交回之后被改过」。
- Opus 腿的 `run-all.out` 现有 sha256 `659a816fccf1db87973e1d53783b29345c560f564a519b0c8e903c6bccf92451`，与主 agent 给的一致。
- 这一轮是设计/治理轮（判 agent 定义与规则），不是代码轮：`.claude/rules/implementation-workflow.md`「代码轮派腿之前记一份开工快照」限定在代码轮，Sonnet 报告 G4#5 也这样判过，我复核同意。但主 agent 这次仍给了快照（罩被判的定义文件），所以本报告按「给了快照」的路径核，不落「没给快照的设计轮，对不上记分不清」。
- 腿开工时刻 2026-09-26T15:30:00Z 前后：用于第 2 步核对不在快照清单里的文件（`.claude/gate.d/55-qemu-first-transaction.sh`、`.claude/scripts/lkmm.sh`、`.claude/gate.d/57-lkmm.sh`、`.claude/gate.d/87-replay.sh`）——`git log --since=2026-09-26T15:30:00Z` 与 `git status --short` 对这四个文件均无输出，判定它们在腿开工之后未被改动，可以对主树核（不落「被改过」）。

## 复跑

```
bash research/prompts/governance-defs-r2-opus-model/run-all.sh /tmp/claude-0/-home-user-singlefs/77b4b1be-00af-5f85-8659-f6e6d51fb840/scratchpad/defs-r2-verifier/rerun-draft
```
（加 `nice -n 19`，草稿目录在我自己的核查子目录下，模型脚本原样，不改路径变量）。退出码 0，输出 213 行，与原 `run-all.out` 行数相同。把临时路径字段（`/tmp/...`、`tmp.XXXXXXXXXX`）归一化之后 `diff` 两份输出，**逐字节相同（diff 退出码 0）**。
41 个模型文件逐个现算 sha256，与 Opus 报告表中列出的 41 行哈希**全部一致**（`diff` 对比排序后的清单，无差异）。

## Sonnet 腿报告核对表

| 引用（行号+抄文，摘要指位） | 核的结果 | 命令/依据 |
|---|---|---|
| G1#1 `gate-triage.md:26` 抄「暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/gate-staged.sh`…」 | ✓ | `awk 'NR==26' .claude/agents/gate-triage.md` 逐字对得上 |
| G1#1 `main-agent.md:59`、`implementation-workflow.md:52` 三处同步换成 `gate-staged.sh` | ✓ | 现取三行原文均含该命令 |
| G1#2 `stage-must-run.sh:51-52` | ✓ | `sed -n '51,52p'` 逐字对得上 |
| G1#3 `gate-staged.sh:27-28,43` | ✓ | `sed -n` 三处逐字对得上 |
| G1#4 `54-layer0-replay.sh:230-233` | ✓ | 逐行对得上；`bash -n` 现核语法通过 |
| G1#5 `main-agent.md:59` | ✓ | 与 G1#1 同一行，已核 |
| G1#6 `heavy-test-guard.sh:40` | ✓ | 逐字对得上；`:520` 只作旁证位置未逐字引，未发现不一致 |
| G1#7 `stage-owners.tsv:37`、`agent-common.md:63` | ✓ | 逐字对得上；复跑 `nice -n 19 bash .claude/gate.d/54-layer0-replay.sh`（不带前缀）确实仍被拒（`rc=2`，「gate-triage 不跑「层 0」」，与 Sonnet 附带观测一致） |
| G1「grep 两遍/重复跑」零命中 | ✓ | 现跑同一条 grep，零命中，4 处「两遍」均与本条无关 |
| G2#1 `crash-verifier.md:24` | ✓ | 逐字对得上（与 Opus 引用同一行，两腿一致） |
| G2#1 `stage-inputs.tsv` 第 12 行（55号）、57 号无登记行 | ✓ | `sed -n '12p'`、`grep -c '^57'` = 0 |
| G2#1 `57-lkmm.sh:14`、`lkmm.sh:36` | ✓ | 内容对得上（`lkmm.sh` 不在快照清单，已按「给了快照、不在清单」核对不在腿开工后被改，可对主树核） |
| G2#2 `heavy-test-guard.sh:129` 抄一整行 AGENT_KINDS | **✗（行号）** | 引文实际跨 129-130 两行（`"crates-mutation-stage", "crates-mutation-mutate"}` 在 130 行），只标 `:129` 未标区间；内容本身无误 |
| G2#2 `heavy-test-guard.sh --selftest` 末行「✓ 自检通过（查了 562 种，其中该拒 102 种）」 | ✓ | 现跑复核，末行文字与退出码完全一致 |
| G2#3 `runner-dispatch-guard.sh:139`、`:38` | ✓ | 逐字对得上 |
| G2#3 `runner-dispatch-guard.sh --selftest` 末行「✓ 自检通过（查了 113 种）」 | ✓ | 现跑复核，一致 |
| G2#4 `implementation-writer.md:46`、`:28` | ✓ | 逐字对得上 |
| G2 额外问题（多会话触发频率）「复核不了」 | ✓（如实标注，非事实断言） | 本报告不判 |
| G3#1 `mutate.sh:514,534,536,473,483` | ✓ | 逐字对得上 |
| G3#2 `mutation-triage.md:28` | ✓ | 逐字对得上 |
| G3#3 `mutation-sampling.md:82` | ✓ | 逐字对得上 |
| G3#4 `mutate.sh:647`、`59-crates-mutation-replay.sh:407` | ✓ | 逐字对得上 |
| G3 深挖 `59-crates-mutation-replay.sh:344,450-451`（「failure 在 :451 `if …: sys.exit(1)` 里必判失败」） | **✗（行号）** | 现查：`if` 子句在第 450 行，`sys.exit(1)` 在第 451 行，两句合一引用只标单一行号 `:451`，跨行未标区间 |
| G3 深挖 `59-crates-mutation-replay.sh:341/343/348/349/329/331/333/335/337` 八类判定分支 | ✓ | 逐行核对，caught/failure/invalid/timeout/memory/squeezed/refused/unavailable 与行号一一对应 |
| G4#1 `three-way-verifier.md:21,28` 两处「分不清」文案统一 | ✓ | `grep -n "分不清"` 现查，21、28 两处字面相同 |
| G4#2 `three-way-verifier.md:32` | ✓ | 逐字对得上 |
| G4#3 `three-way-verifier.md:22`（腿开工时刻） | ✓ | 逐字对得上 |
| G4#4 `three-way-verifier.md:28`（给了快照而不在清单里） | ✓ | 逐字对得上 |
| G4#5 `implementation-workflow.md:24` 小节标题 | ✓ | 现查标题与判定逻辑，与「设计轮不给快照」的自洽性判断成立 |
| G5#1 `20-kb-shape.sh:46` | ✓ | 逐字对得上；`path-moves.md` 标题「## 改一个全仓术语：正文之外还有五处会红」现查存在（第 23 行），`path-moves.md:37`「历史类文件同样换名」一字不差 |
| G5#2 `90-term-renames.sh:22` 抄「确实该留旧名的…逐文件登记进 .claude/term-rename-exempt 并写明为什么」 | **✗（行号）** | 引文实际跨 22-23 两行（`echo` 分两行拼接：「…逐文件」在 22 行，「登记进…」在 23 行），只标 `:22` 未标区间；内容本身无误 |
| G5#3（旁证）`.claude/term-rename-exempt:6` 与「不整个目录豁免」相反 | ✓ | 现查该行确为整目录豁免（`.claude/warnings/`），且经 `git show 71f0cbc:...` 确认早于本轮 diff，判定为历史遗留、不计入本轮，Sonnet 的处理合理 |
| G5#4 `75-decision-experiment-links.sh:226`、`20-kb-shape.sh:213` 两条正则 | ✓ | 逐字对得上 |
| G5#5 `agent-common.md:34` | ✓ | 逐字对得上；`kb-scribe.md:4` `tools: Read, Edit, Bash` 现查属实 |
| G5#6 `implementation-writer.md:46,28` | ✓ | 逐字对得上 |
| G5#7 `experiment-runner.md:35` | ✓ | 逐字对得上；`.claude/decision-links-pending` 现查非注释行数为 0 |
| G5#8 `kb-scribe.md:20`、`decisions.md:20` | ✓ | 逐字对得上 |
| G5#9 `experiment-designer.md:22(已删)/28/38` | ✓ | 16-23 行区间现查无「英文名」命中；28、38 行逐字对得上 |
| G5#10 `experiment-runner.md:27` | ✓ | 逐字对得上；`Cargo.toml` 无 `[[bin]]`、`src/bin/` 下蛇形命名现查属实 |

计数（Sonnet 报告）：核了 38 处（不含判别力自证与输入核实），✓ 35 处，✗ 3 处（均为「引文跨多行只标单一行号」，内容本身未见字面错误），核不动 0 处，分不清 0 处。
## Opus 腿报告核对表

| 引用 | 核的结果 | 命令/依据 |
|---|---|---|
| 快照自证「42 行无一不符」 | ✓ | 现跑 `sha256sum -c`，我自己也得到 42 行 OK |
| 负载自证「ps 里没有 cargo/gate.sh/qemu/fio/vm-bench」 | 核不动（当时状态无法回溯） | 只能确认现在没有这类进程在跑；无法回验开工那一刻的 `ps` 快照 |
| `crash-verifier.md:24` | ✓ | 与 Sonnet 引用同一行，已核为逐字对得上 |
| `stage-inputs.tsv:12(55号)、:13(59号)`、`59-crates-mutation-replay.sh:176` COPIED_INTO_EACH_SHARD | ✓ | `sed -n '12,13p'` 与 `:176` 内容对得上（59 号一行为 `crates/ Cargo.toml Cargo.lock research/scripts/run-with-memory-cap.sh`，与文中一致） |
| `57-lkmm.sh:14`、`lkmm.sh:281` | 部分核（`:14` ✓，`:281` 未逐字核） | `:14` 逐字对得上；`:281` 只核了存在性未逐字比对全文（时间预算原因，标注未做） |
| G2 历史 S1-S4（`g2-diff-check.sh`） | ✓ | 复跑输出与报告表逐字一致（见「复跑」一节，diff 归一化后为 0） |
| G2 放开会话表（324/6/189/192/0/63） | ✓ | 复跑输出数字与报告表逐字一致 |
| S4 cargo 认出结果（`e999_other_session_wip`、`wip_from_other_session`、631/724） | ✓ | 复跑输出逐字一致 |
| `agent-write-scope.tsv:12` | ✓ | 逐字对得上 |
| `agent-common.md:54` | ✓ | 逐字对得上 |
| `main-agent.md:59` 引用（同 Sonnet） | ✓ | 已核 |
| B4 桩测试 green/red/250/251（今天写法与改法两组输出） | ✓ | 复跑输出与报告表逐字一致（含临时路径归一化后） |
| `54-layer0-replay.sh:230,233` | ✓ | 与 Sonnet 引用同段，已核 |
| B5 `stage-owners.tsv:19`（59 号「要几个钟头」） | ✓ | 逐字对得上 |
| `87-replay.sh:26`、`stage-inputs.tsv:15`（87 号一行） | ✓ | 逐字对得上 |
| N1 派发闸 `gate-triage--*` 三个 `rc=2` 输出（含末尾截断的乱码字节） | ✓ | 复跑输出逐字节一致，包括截断处的同一乱码字节 |
| G1 钩子放行/拒绝统计（38 次喂钩子，2 条对照均拒） | ✓ | 复跑 `run-hooks.sh` 段落输出与报告描述一致（现查 g1-triage-gate-staged-noprefix、g1-triage-54-direct 两类确实 `rc=2`，其余 `rc=0`） |
| B6 `experiment-designer.md:28`、`experiment-runner.md:27` | ✓ | 逐字对得上 |
| B6 「16 份登记，0 份含英文名，`e156_allocation_basis_counts.rs` 存在」 | ✓ | 现跑同样的 `ls`/`grep -l`/`ls crates/...`，数字与文件名一致 |
| B1 `mutate.sh:496,512`、`59-crates-mutation-replay.sh:344,450`（`if` 子句，非 Opus 引用的 :451） | ✓ | Opus 在正文里把 `if` 与 `sys.exit(1)` 分别标成 `:450`、`:451` 两个不同行号（未像 Sonnet 那样把两句并成一句只标一个行号），逐查均对得上，无跨行合并的问题 |
| B1 四份合成产物文件（`g3-outputs/*.txt`） | ✓ | `cat` 现读，内容与报告摘录（`compile-error.txt` 的 E0425、`crash-after-named-test-passed.txt` 的栈溢出等）逐字对得上 |
| B7 `three-way-verifier.md:22,28` | ✓ | 与 Sonnet 引用同段，已核 |
| B7 「`.claude/scripts/lkmm.sh` 等四个文件都不在这一轮快照清单里」 | ✓ | 逐个 `grep -qF` 现查（我自己核对快照清单 42 行，确认这四个文件名不在其中） |
| 「没打中的形状」表格的数字（19×2=38、`crates/mutations.tsv` 里 `publish_order_matches_litmus` 0 行） | ✓ | 复跑与现查 `grep` 数字一致 |

计数（Opus 报告）：核了 24 处，✓ 22 处，✗ 0 处，核不动 1 处（开工时刻的 `ps` 负载状态无法回溯），部分核 1 处（`lkmm.sh:281` 存在性核过、全文未逐字比对，如实标注未做）。

## 判定一览

- 两份腿报告的引用/产物/复跑命令**绝大多数逐字对得上**主树或它们自己的模型产物；复跑命令（`run-all.sh`）产出与留存的 `run-all.out` 逐字节相同（归一化临时路径后），41 个模型文件哈希与报告表全部一致。
- 发现的 3 处 ✗ 全部同一种形态：**引文实际跨两行，报告只标了其中一个行号**（Sonnet 的 `heavy-test-guard.sh:129`、`59-crates-mutation-replay.sh:451`、`90-term-renames.sh:22`）。三处的**内容本身未见字面错误**，只是行号定位不精确（少了区间标注），不构成事实性误引。

## 没做什么

- 不判 G1-G5 各条推论打中成不成立、该不该采纳；不判 B1-B7、S1-S4 等攻击构造是否有效，那是主 agent 的职责。
- 未逐字比对 `.claude/scripts/lkmm.sh:281` 全文与 Opus 报告摘要的对应关系（只核了存在性），标注为部分核。
- 未回验 Opus 报告「负载自证」（`ps -o pid,args` 在腿开工那一刻的输出）——那一刻的进程状态无法在事后重建，标注核不动。
- 未对本轮之外、两份报告都未点名引用的文件做额外抽查（例如背景材料、附录里其余未被引用的段落），只核两份报告实际写出的引用与产物。
- 未跑任何重型测试（54/55/57/59/87、整轮门禁），与两条腿一致。
- 本地腿本轮缺席（两份腿报告均已注明理由），第 5 步「本地腿转述核对表」不适用，未做。
- 没有对 `.claude/hooks/heavy-test-guard.sh:519-520`（Sonnet G1#6 提及但未逐字引用的旁证位置）做逐字核对，只确认了该处内容与判定描述不矛盾。
