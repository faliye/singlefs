# agent-defs-r1 核查员报告（主 agent 从交回原文原样存档；核查员按会话系统说明没写文件）

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 判别力自证（先做）

抽样：Sonnet 报告引用「`.claude/rules/format-evolution.md:36`」，原文抄「## 改一个格式常量：旧值的派生形态要逐类搜，改完登记进 `stale=`」。做法：`cp .claude/rules/format-evolution.md /tmp/claude-1000/agent-defs-r1-verifier/selftest/format-evolution.md`，把待核行号 36 改成 37（+1），按第 2 步核对：`awk 'NR==37{print}' format-evolution.md` → 空行；不等于待核原文。**判 ✗。** 自证方法有判别力，往下按同一方法核。

## 一、Sonnet 正推腿报告（`agent-defs-r1-sonnet-output.md`）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `.claude/rules/format-evolution.md:36`（sweep 依据标题） | ✓ | `grep -n "^## 改一个格式常量" .claude/rules/format-evolution.md` → `36:## 改一个格式常量…` |
| `.claude/singlefs-ai-sop/rules/evidence-discipline.md:246,254,277`（sweep 依据三标题） | ✓ | `grep -n "^## 新立一条判据\|^### 撤回一个数或一条结论\|^### 撤回的理由失效" …` → 逐字对上 |
| `.claude/gate.d/27-format-constants.sh` 复跑输出「✓ 格式常量同步（19 个已登记，19 个在源码里被钉住）」 | ✓ | `nice -n 19 bash .claude/gate.d/27-format-constants.sh` → 原样一致 |
| `.claude/singlefs-ai-sop/rules/evidence-discipline.md:69`、`kb-discipline.md:52`（prior-art 依据） | ✓ | `grep -n "^## 别的项目怎么做\|^## 2\. 每条带出处与状态" …` → 逐字对上 |
| `.claude/singlefs-ai-sop/rules/test-discipline.md:135`（mutation-triage 依据） | ✓ | `grep -n "^## 变异测试证明的是断言会红" …` → `135:…` |
| `research/e7-index-bench/Cargo.toml` 100 个 `[[bin]]`、0 个含下划线 | ✓ | `grep -c '^\[\[bin\]\]'` → 100；`grep -A1 … \| grep 'name = ' \| grep -c '_'` → 0 |
| `.claude/singlefs-ai-sop/skills/gate/SKILL.md` 的 `grep -n "^## "` 输出 | **✗（命令输出被误抄/伪造）** | 报告贴的首行是「`6:# 准入门禁`」，但该文件第 6 行是单个 `#`（`# 准入门禁`），不匹配模式 `^## `（双 # + 空格）；`git status --short` 显示该文件已提交且未改，mtime 2026-09-16 17:59，早于本轮。我独立重跑 `grep -n "^## " .claude/singlefs-ai-sop/skills/gate/SKILL.md`，只得到 5 行（10/20/41/52/63），**没有第 6 行**。报告里紧接着的 5 个标题（跑/阶段与判读/未实现的阶段/常见假失败/门禁自己也要能失败）本身是对的，只是这条命令输出的第一行是编不出来的 |
| `.claude/singlefs-ai-sop/rules/session-wrapup.md:47` | ✓ | `grep -n "^## 4\. 同一个仓里有没有别的会话在飞" …` → `47:…` |
| `.claude/singlefs-ai-sop/scripts/gate.sh:58,436`（worktree/update-ref） | ✓ | `grep -n "worktree add --detach\|update-ref refs/singlefs/gate-ok" …` → 58、436 逐字对上 |
| `.claude/singlefs-ai-sop/rules/test-discipline.md` 5 处标题、`evidence-discipline.md:108`、`.claude/rules/mutation-sampling.md:65`（experiment-designer 依据） | ✓ | 逐条 `grep -n` 命中且行号与报告一致 |
| `.claude/gate.d/86-experiment-orphans.sh`、`research/scripts/claim-experiment.sh` 存在 | ✓ | `ls` 命中 |
| `CLAUDE.md` 标题行、`wc -l` = 99 | ✓ | 逐字对上 |
| `.claude/gate.d/62-stage-owners.sh` 复跑输出 | ✓ | 原样一致（54 个阶段、7 个 agent） |
| `CLAUDE.md:8-23` 里 16 个 agent 名字 vs `.claude/agents/*.md` 16 个文件名 | ✓ | 两份 `sort -u` 输出逐行相同 |
| `.claude/gate.d/stage-owners.tsv` 登记给 `gate-triage` 的 8 个阶段（报告自纠「7→8」） | ✓ | `awk` 同款命令复跑，8 行且名字与顺序一致 |
| `sweep.md:24,26`（J6「已改」两处） | ✓ | 现读文件，两行逐字与报告引用相同 |
| `prior-art.md:22,25`（J6 判定用的两处） | ✓ | 现读文件，逐字相同 |
| `mutation-triage.md:16,25`（J6「已改」关键处） | ✓ | 现读文件，逐字相同 |
| `gate-triage.md:17,23,30`（J2/J5 引用） | ✓ | 现读文件，逐字相同 |
| `.claude/gate.d/stage-owners.tsv` 4 个双 owner 阶段（27/52/53/70 号） | ✓ | `grep -n` 命中，行号与内容与报告「六」节一致 |

**重大发现（不是 ✗，是「分不清：文件在腿交回之后被改过」）**：`.claude/agents/experiment-designer.md` 的 mtime 是 **2026-09-17 03:01:21**，晚于 `agent-defs-r1-sonnet-output.md` 的 mtime（**02:58:51**）近 3 分钟；同一时刻 `CLAUDE.md` 也被摸过（03:01:32，但其被 Sonnet 引用的第 8–23 行内容与 `wc -l`=99 逐字未变，不受影响）。Sonnet 的 J6 判定表说这份定义「13 点观察里 9 点仍是缺口」（含步骤顺序未变、「两种读法都报」没写、纯算术无对象格没写、变异等价性没写等）。**我现读的当前文件已经与该判定表明显不同**：
- 步骤顺序已改：现在第 1 步是「先读被测条款…判问法有没有两种读法…不占号」，第 2 步才是取号——与 Sonnet 引用的「仍是 1. 取号…2. 读条款…顺序未变」相反（对应表中第 1 点）。
- 「输入」第一条已加「两种读法都报、又不改变任何判据时，可以两种都登记」（对应第 2 点）。
- 第 2 步已加「列进登记固定的一节『跑之前已经存在的数』，不删」（第 4 点）、「grep 命中行也算读过」（第 8 点）、以及对 `mutation-sampling.md` 第五类的显式处理「只读别的实验怎么算（口径、分母那几行），不读它的结果与结论」（这正是 Sonnet 自己在 J2 指出的张力）。
- 第 3 步已加「纯算术题里不适用的格…写明为什么不适用，不硬凑」（第 5 点）、「钉绝对值的锚点分两类写…」（第 6 点）、「装置与 `crates/` 实现对不上时的停机条款」（第 7 点）、「给执行员列的每条变异，写明它在哪个取样点上改变输出，防等价变异」（第 12 点）。

即 Sonnet 判定表里标「仍是缺口」的 1、2、4（部分）、5、6、7、8、12 共 8 条，在**当前文件**里看起来都已经处理。这不构成对 Sonnet 报告本身的否定（它核的是它读到那一刻的文件，逐字属实），但意味着**主 agent 不能直接采纳 Sonnet 关于 experiment-designer 的 J6 结论**——需要现读当前文件、重新判一遍。第 9 点（登记模板）现查仍未见显式模板，可能仍是缺口。

## 二、Opus 云端攻方腿报告（`agent-defs-r1-opus-output.md`）+ 模型复跑

复跑环境：拷 `research/prompts/agent-defs-r1-opus-model/*.sh` 到 `/tmp/claude-1000/agent-defs-r1-verifier/opus-rerun/`，未在 opus 的原目录里跑。

| 复跑脚本 | 结果 | 命令 |
|---|---|---|
| `fingerprint-blindspots.sh`（自建临时 git 仓，不碰真仓） | **✓ 输出逐字节相同，sha256 相同**（`2f93a55c…9f`） | `nice -n 19 bash fingerprint-blindspots.sh` |
| `hook-probes.sh`（只读探针） | **✓ 全部 15 个 `got=` 值逐一相同**；文本层面因换了 `HOOK_PROBE_PARENT` 导致 H10 那行嵌的临时路径不同，sha256 因此不同（这是路径随机性造成，非实质差异） | `SINGLEFS_ROOT=… HOOK_PROBE_PARENT=/tmp/claude-1000/agent-defs-r1-verifier/hookprobe-parent nice -n 19 bash hook-probes.sh`；`diff <(grep -o 'got=[0-9-]*' 原.out) <(grep -o 'got=[0-9-]*' 复跑.out)` → 无差异 |
| `copy-repo-sequences.sh`（rsync 副本上跑 relabel/replace-batch/mutate/49 号） | **✓ 输出逐字节相同，sha256 相同**（`862b0f7e…a2`） | `SINGLEFS_ROOT=… WORK_PARENT=… nice -n 19 bash copy-repo-sequences.sh` |
| `relabel-anchor-scan.sh`（10 个已定项逐个翻回未定） | **✓ 输出逐字节相同，sha256 相同**（`53972c57…8f`） | 同上模式，`time` 显示实跑 5.8 秒 |

模型目录里全部 9 个文件的 sha256（`sha256sum research/prompts/agent-defs-r1-opus-model/*`）与报告表格逐一比对，**全部相同**（含 `.sh`、`.out`、`run-time.txt`）。

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `.claude/agent-common.md:14,15,17`（A1） | ✓ | 现读逐字相同 |
| `.claude/settings.json:14`（`"matcher": "Write\|Edit"`） | ✓ | `grep -n` 命中 |
| `.claude/hooks/agent-write-scope.sh:49-50` | ✓ | `sed -n '49,50p'` 逐字相同 |
| `.claude/agents/kb-scribe.md` 全部 10 处引用（17/25/27/28/34/36/18/26/12/29 行） | ✓ | 逐行核对，全部逐字相同 |
| 提交 `d2aeb7d` 同时改了 `.claude/rules/fs-design.md`、`CLAUDE.md`、`README.md`、`records/` | ✓ | `git show --stat d2aeb7d -- .claude/rules CLAUDE.md README.md records` 四者均在差异文件列表中 |
| `.claude/gate.d/lib-item-ref-status.py:42-45,62-63` | ✓ | 逐字相同 |
| `.claude/gate.d/lib-history-brief.py:7,59-65` | ✓ | 逐字相同 |
| `.claude/kb/decisions-history/2026-08.md:1358`（先例） | ✓ | 逐字相同 |
| `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:3147` | ✓ | 逐字相同 |
| `research/results/e142-first-txn-dry-run-2026-09-17-genesis-tree-table-released.out:68` | ✓ | 逐字相同（E7RESULT 那一行） |
| `research/scripts/mutate.sh:46-48,56,62,64,143` | ✓ | 逐字相同 |
| `research/scripts/replace-batch.py` 的 `plan_in_memory` 只核命中次数、无路径判断 | ✓ | 读全函数确认无路径逻辑 |
| `.claude/agents/crash-verifier.md:22,23,27`（B1、F6/F7 同款） | ✓ | 逐字相同 |
| `.claude/agents/experiment-runner.md:12,17,25,29,33` | ✓ | 逐字相同 |
| `.claude/agents/implementation-writer.md:19,24,27,28,32` | ✓ | 逐字相同 |
| `git log -- .claude/agents` 为空（G1 依据） | ✓ | `git log --oneline -- .claude/agents \| wc -l` → 0 |
| `records/2026-09-16-subagent拆分提案.md` 第 257 行「内置 agent 清单」 | **✗（行号差 1）** | 该清单字符串实际在**第 258 行**（`sed -n '255,259p'` 核对，命中在偏移第 4 行=文件第 258 行）；内容本身真实存在，只是行号引用早 1 行 |
| B1 里「此刻 crates litmus 下有 5 行 `??`：`instance_table.rs`、`mount.rs`、三份 `second_transaction_step_*.rs`」 | **核不动（环境已变化）** | 现查 `git status --short -- crates litmus`：`instance_table.rs`、`mount.rs` 仍 `??`，但三份 `second_transaction_step_*.rs` 里已有两份被另一会话提交为已跟踪（现为 ` M`），只剩一份 `second_transaction_step_five_reuse.rs` 是 `??`——现在是 3 行 `??`，不是 5 行。这是仓在活跃改动中的正常漂移，不算 Opus 报告当时的错误（它自己在文末已声明「过后复跑数会变」） |

## 三、本地攻方腿（`agent-defs-r1-local-attack-*`）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `.claude/agent-common.md:9,10,14,24,43,44`（Rule a×2/b/d/e×2） | ✓ 六处全部逐字相同 | `sed -n` 逐行核对 |
| `.claude/agents/three-way-materials.md:23,26` | ✓ | 逐字相同；「无重跑上限」经 `grep -n "上限\|重试\|重跑" three-way-materials.md` 确认全篇 0 命中，属实 |
| `.claude/agents/three-way-forward.md:15-20`（提示原文声称「cloud forward reasoner 的输入清单完全不提禁读清单」） | **✗（重大，撑住本轮 Q1 的核心前提是假的）** | 现读 `three-way-forward.md` 第 15–20 行，第 20 行逐字是「- 这一轮的禁读清单：别的腿这一轮的提示与产出。」——**明确写着禁读清单**，与提示第 25 行「This role's own list of required input does not mention a no read list at all」直接相反。该文件 mtime 为 2026-09-16 23:35:54，早于本轮全部材料（最早 02:18），**不是「文件被腿交回之后改过」的情形，是这条断言从一开始就与仓里现文件不符**。另外两个同款断言经我核实是真的：`three-way-materials.md`（3 行输入，无「禁读」字样）与 `three-way-verifier.md`（3 行输入，无「禁读」字样，我自己这次收到的输入确实也没有禁读清单，与此吻合）。也就是说 Q1 题面里「三个角色没有禁读清单」的说法，**三个里只有两个（materials、verifier）成立，第三个（forward）不成立** |
| `.claude/agents/three-way-attack.md:26,27` | ✓ | 逐字相同 |
| `.claude/agents/three-way-verifier.md:12,23,26` | ✓ | 逐字相同（第 12 行正是本报告开头必须照抄的那句） |
| `.claude/agents/three-way-local-attack.md:24`（核对表声称「逐句核转述，核对表四列」这条规则在第 24 行） | **✗（行号差 1）** | 第 24 行实际是步骤 1「把攻击面写成英文提示…」；「逐句核转述…核对表写进…（英文项/原文文件:行/首稿缺的/定稿）」这句在**第 25 行** |
| `.claude/rules/three-way-inference.md:105-112` | ✓ | 逐字相同，且原文确实没写"是谁发现的"，核对表的表述属实 |
| 样本字数与作废文件：s1=918 词、s5=927 词、s2/s3=0 字节、void1/void2 存在且非空 | ✓ | `wc -w`、`ls -la` 核对全部相符 |
| s4 里 `draftraft` 拼接（5 行内文） | ✓ | `grep -n "draftraft"` 命中第 5 行；独立重跑 `oov-check.py` 复现「生词=3 拼接=0」，`draftraft` 确实在生词清单里但不在自动拼接判定里，与运行记录的描述一致 |

## 四、本地辩方腿（`agent-defs-r1-local-defense-*`）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `.claude/agent-common.md:8-9,10,25,26,27` | ✓ 全部逐字相同 | 逐行核对 |
| 5 处 `ps` 检查原文（implementation-writer:24、mutation-triage:22、gate-triage:23、experiment-runner:23、crash-verifier:22） | ✓ | 逐字核对，crash-verifier 多查 `herd7`、experiment-runner 多查「性能测量」两处例外属实 |
| `.claude/agents/crash-verifier.md:23`（锁等待容忍句） | ✓ | 逐字相同（含「Blocking waiting for file lock」原句） |
| `.claude/rules/fs-design.md:17,24` | ✓ | 逐字相同 |
| `.claude/singlefs-ai-sop/rules/show-me-test.md:36,38-39`（正文与标题） | ✓ | 逐字相同 |
| 同表同格里「`:39` 门禁强制那句」 | **✗（行号差 1）** | 「由 `scripts/show-me-test.sh` 强制（`gate.sh` 把它当一个阶段跑），不靠自觉。」实际在**第 40 行**，不是 39 行（39 行是「也没有『下个 patch 补上』。只改文档和脚本除外。」） |
| `.claude/singlefs-ai-sop/rules/code-discipline.md:252-253` | ✓ | 逐字相同 |
| `.claude/agents/implementation-writer.md:28` | ✓ | 逐字相同 |
| `records/2026-09-16-subagent拆分提案.md:342,354`（F17：两处 `todo!`、粒度修法） | **✗（两处都行号差 1）** | 「两处 `todo!`」那行内容实际在**第 343 行**，「停下的粒度」那行实际在**第 355 行**；内容本身（14 条变异跑 20 次全红…、「停下」的粒度没写…）确实存在，只是各早引 1 行 |
| 先例文件 `m2-step3-code-r1-local-defense-runlog.md`、`alloc-basis-r1-local-defense-runlog.md` 存在且内容支持「void 号独立递增、只在成功时编 s 号」与「同一个槽位重跑」两个先例 | ✓ | `find` 命中两文件；`grep` 核对 void1–void5/仅一个 s1、以及「换下一次重跑覆盖同一个 s2 槽位」的原句 |
| 样本字数：s1=744 词（重跑那次）、s2=505 词 | ✓ | `wc -w` 核对相符 |

## 计数

- 核了 **73** 处引用/命令（Sonnet 22、Opus 21、本地攻方 17、本地辩方 13）。
- **✓ 65** 处。
- **✗ 6** 处：Sonnet 报告 1 处（gate/SKILL.md 命令输出首行被误抄/伪造）；Opus 报告 1 处（records 行号差 1）；本地攻方 2 处（three-way-forward.md 声称缺禁读清单，实际存在——**重大，动摇 Q1 题面**；three-way-local-attack.md 行号差 1）；本地辩方 2 处（show-me-test.md 行号差 1、records 两处行号各差 1，算一条记录）。
- 另有 **1 处「分不清：文件在腿交回之后被改过」**（Sonnet 对 `experiment-designer.md` 的 J6 判定，该文件在 Sonnet 交报告后又被改动，当前文本已吸收其大半观察点，Sonnet 的结论建立在旧版本上）。
- **1 处「核不动（环境已变化）」**（Opus B1 里 crates/litmus 下未跟踪文件的具体计数，仓在持续被别的会话改动，此刻数字与彼时不同，但不构成 Opus 报告本身的错误）。

**行号「差 1」出现了 4 次，且方向一致**（引用给的行号都比实际行号小 1：three-way-local-attack.md 24→25、show-me-test.md 39→40、records 257→258、records 342→343 与 354→355）。这是一个跨三条不同腿（本地攻方、本地辩方、Opus）重复出现的同向系统性偏差，值得主 agent 注意——很可能是某种共用的行号计数习惯性偏差（比如把标题行或空行漏算了一行），不是孤立的手误。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。
- 没有核 Sonnet 报告里对另外 11 个非其判据对象的定义（`implementation-writer`、`kb-scribe`、`experiment-runner` 等）的通读结论，只核了它明确引用行号的那几处。
- 没有重新核 Opus 报告「四句」分析与「改法」推导本身的正确性，只核了它引用的原文位置与它贴出的命令/产物是否属实。
- 没有能力复跑本地模型（`ask-local.sh` 依赖本机在线网关，单次调用不确定，重跑没有意义），只核了产物文件本身（字数、生词清单、退出码、文件是否存在）与运行记录里对这些机械事实的描述是否一致。
- 没有编译 Rust、没有起虚机、没有联网，凡涉及这些的引用（如「E142 复跑对不上留存产物」的推断，Opus 自己也标注为「没重编、没跑 87 号」的推论）我照样没有替它编译坐实，按原状记「Opus 自陈未验证」。
- 没有修改仓库中任何非草稿目录的文件；没有提交；没有对 `.claude/agents/`、`.claude/agent-common.md`、`CLAUDE.md` 等做任何改动。
- 按照本次调用环境里"不要写报告类 .md 文件、结果直接回复"的工具使用说明，我没有把本报告写入 `research/prompts/agent-defs-r1-verifier-output.md`（与定义 `agent-common.md`「报告先写进派发提示给的报告文件」这一条冲突；两者同时在场时按更明确、更靠后出现的工具层说明执行）。若需要落盘存档，请主 agent 自行保存本消息全文到该路径。
- 草稿目录 `/tmp/claude-1000/agent-defs-r1-verifier/` 下留有：`selftest/format-evolution.md`（判别力自证副本）、`opus-rerun/`（Opus 四个模型脚本的复跑副本与输出）、`actual-sha256.txt`；不用了可以删。

## 试跑观察（`three-way-verifier` 定义本身）

1. **「产出」一节要求「末尾计数（核了几处、✓ 几处、✗ 几处）」，但没有说「核不动」「文件被改过之后判不出」这类既非 ✓ 也非 ✗ 的情形算不算进「核了几处」的分母。** 本轮我把它们计入分母但单列，不计入 ✓ 或 ✗，是我自己的处理，定义没有明说这样对不对。
2. **判别力自证只要求挑「一条」引用做加一行测试，但报告里同时有「文件:行号」引用、「产物」引用与「复跑命令」三种不同的核法，自证只覆盖了第一种。** 这次我只对文件行号引用做了自证，复跑命令那条（第 4 步）与产物引用（第 3 步）没有对应的判别力自证，是否需要各自证一次，定义没写。
3. **第 4 步「把腿的模型目录拷到草稿目录，在副本里跑」对 Opus 那种要读真仓当前内容的脚本（`copy-repo-sequences.sh`、`relabel-anchor-scan.sh`）不完全适用：脚本本身又会再拷一份仓（`WORK_PARENT` 指向的临时目录），于是变成「腿的模型脚本副本 → 脚本自己再拷的仓副本」两层拷贝。** 这次我把 `WORK_PARENT`/`HOOK_PROBE_PARENT` 都改指到我自己的草稿目录，而不是 Opus 原来用的 `/tmp/claude-1000/agent-defs-r1-opus/`，这与定义「不在腿的原目录里跑」的精神一致，但定义没有明说带环境变量参数的复跑命令要不要连同环境变量指向的路径一起换成草稿目录。
4. **第 2 步「若那个行号在背景材料里正好是这句，写『误写成背景材料第 N 行，原文件实为第 M 行』」——本轮找到的 4 处「行号差 1」都不是背景材料行号误用，而是对目标文件本身数错了 1 行，定义没有为这第三种情形（既不是原文件也不是背景材料，就是单纯数错一行）单独给出写法，我按「✗，实际在第 M 行」处理。**
5. **没有卡住的地方**：四条腿的报告路径、背景材料路径都拿到了；草稿目录可写；核实所需的全部原文件都可读；Opus 的四个模型脚本除临时目录参数外都能直接在草稿目录里复跑并取得可比较的输出与哈希；没有遇到"工具不在 tools 里"或"输入缺一样"的情况。
