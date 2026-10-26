# landing-audit-r6 判决：子 agent 落地审计第九到十一批改动第三轮、最后一轮（主 agent，2026-10-23）

正文：`research/prompts/_landing-audit-r6-body.md`。两条腿：辩方（Sonnet，K1）`research/prompts/landing-audit-r6-sonnet-output.md`（sha256 `0f8c181ef2fb663d32a6ab8a7c2c1fc7ec3bf60f9411abd56baa57352fa2ecef`，模型目录 `research/prompts/landing-audit-r6-sonnet-model/`）、攻方（Opus，K2、K3）`research/prompts/landing-audit-r6-opus-output.md`（sha256 `faf19c876dded53fefddc473becf01eb945a9b13527f0668963155a6d08a9e25`，模型目录 `research/prompts/landing-audit-r6-opus-model/`）。这是第三轮，之后不再开轮。

## 这一轮被判过的文件

- `.claude/hooks/handback-guard.sh`：⑤ 认命名检查自己的退出码、② 认仓里与已登记 worktree 里的报告；攻方 K2-1 打中共用约束第 63 行教的「输出落进文件、再读汇总行」被 ⑤ 拒且拒绝说明写成「没跑过」，K2-3 打中输出太长存成 persisted-output 时 `; echo rc=$?` 那一行读不到（两版都有）；`| tail` 不收站得住（K2-2）。
- `.claude/hooks/lib_three_way_legs.py`：worktree 认法试了 14 种写法没打中；`$SINGLEFS_WORK_ROOT/` 开头的报告路径不认（推的，真交回 0 次）。
- `.claude/hooks/runner-dispatch-guard.sh` 与 `.claude/hooks/fixtures/runner-dispatch-guard/real-prompts/`：没人打中。
- `scripts/agents/agent-watch.py`：取 progress.md 没打中；辩方点出 ⑭ 两轮都没在真进程上量过。
- `.claude/hooks/lib_prompt_scratch_line.py`：文件头「四处各按自己的规矩办」那一句没写进看门狗（措辞）。
- `.claude/hooks/heavy-test-guard.sh`、`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/implementation-workflow.md`：五处例外说法逐字一样，脚本打出的重跑命令主 agent 起得了；攻方 K2-7 与辩方都打中五处都说「跑法照验证段第 1 步」，第 1 步没写这条命令、命令代码段不带前缀，第 4 步写的还是整条重跑。
- `.claude/agents/kb-scribe.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/implementation-writer.md`、三份三方定义、`.claude/hooks/agent-write-scope.tsv`、`.claude/hooks/write-guard.sh`、`.claude/hooks/completion-cleanup-guard.sh`、`.claude/gate.d/code-tooling.sh`、`scripts/gate/merge-and-verify.sh`、`scripts/agents/agent-cost.py`、`scripts/agents/subagent_transcript.py`：前两轮站住的五种形态（写回员目标、三方腿类型表、三个报告名前缀、草稿目录一行收严、坏字节记红）没被第十一批改坏（攻方 K2 没打中那几格）；辩方点出合入后验证新那一步的第一次真跑验收、`selftest-registry.tsv` 里 `merge-and-verify.sh` 那一行的理由、内存上限（推的）三件没去向。
- `.claude/gate.d/replay-comparison-units.tsv`：辩方核出「# 不登记的比对对象」今天是八行（r5 五行、r7 三行），r7 三行没有指向任何欠账号，与 C879（E165 根选择与杀伤界正式跑没有单元）说的五条对不上。
- `records/2026-09-16-subagent拆分提案.md` 第九十节、调度记录：辩方逐条找了去向，见「二、逐格判」K1。

## 一、核查

- 开工快照：`sha256sum -c research/prompts/landing-audit-r6-snapshot/judged-sha256.txt` 40 份全部 OK，`kb-sha256.txt` 全部 OK；腿跑着时没有人改被判文件。
- 引文：`python3 scripts/agents/cite-check.py <两份腿报告> --root . --background research/prompts/_landing-audit-r6-background.md` 末行「✓ 核了 15 处引文、144 处裸行号，对上 15 处，没判 144 处」。
- 攻方模型复跑：模型目录拷到草稿目录，`bash scripts/hosts/run-with-memory-cap.sh 4G bash <副本>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <新草稿目录> --model <副本>` 排队 1137 秒后起跑，退 0，「抽了 3368 份（共 3368 份记录）」；10 份输出里 7 份与 `out/` 逐行相同（含 `r5_report_verdicts`、`r5_naming_real`，第二轮判决的 402 份与 4 份由此复出）；`leg_paths`、`naming_result_shapes`、`watch_progress_all` 三份只差份数（618 → 620、1658 → 1662、172 → 179，腿交回后新落的会话记录），结论行不变（看门狗 2026-10-22 起取到别的目录下的改前 22、改后 0）。
- 辩方模型没复跑：判决不引辩方独有的数，只用它逐条给的出处与去向（每条主 agent 抽核了出处那一行）。
- 攻方没核动、交主 agent 的第十批 507 种、62 种：主工作区今天是第十一批之后的钩子（512 种、78 种），没有第十批那一版可起，核不动。

## 二、逐格判

### K1（辩方：未定项清单的完整性）

| 条 | 腿报 | 主 agent 判 |
|---|---|---|
| 89 行延后项的去向 | 有 41、半 20、无 26（归成 17 件），另 2 行总述 | 采：没去向的 17 件全数登记进九十节 ㉟，一件一格 |
| ⑭ 两轮都没在真进程上量过 | 第二轮判决写「第三轮点名量」，第三轮正文没点名 | 采：主 agent 写正文漏了；登记 ㉟ |
| 验证段第 1 步命令不带前缀 | 第 7、96 行带，第 52 行代码段不带；第十、十一批都留给第二轮判，第二轮判决一次没提 | 采，与攻方 K2-7 合并：登记 ㉞ |
| 第十、十一批没有收拢表与结局账 | `.claude/main-agent.md` 第 27 行要求每批做 | 采：主 agent 收尾当场补（结局账 `records/landing-audit-b11-outcome-2026-10-23.txt`，查了 280 个子 agent；收拢表写进调度记录节「第十一批收尾」） |
| 合入后验证新那一步三件 | 第一次真跑的验收、登记表那一行理由、内存上限（推的）没去向 | 采：登记 ㉟ |
| ㉑ 只测了全大写与全小写 | 九十节写「不分大小写」 | 采：登记 ㉟（混合大小写实测） |
| 账表三方腿桶改读登记后变多少没量 | — | 采：登记 ㉟ |
| 「弄坏开关点名不全」散在五处，九十节只登两处 | — | 采：并成 ㉟ 一格 |
| 杂项八件 | 派发闸跨行「」引文、`dispatch-prompt.py` 文件头第 38、39 行过期、`claim-experiment.sh` 不在复跑表、`CLAIM_SKIP_*` 不是 `_BREAK` 名字、`stage-owners.tsv` 不含设计员、`PIPESTATUS`、遗留的看门狗自证目录、worktree 里只写相对路径的报告 | 采：登记 ㉟ |
| 九十节去向过期 | ㉛、㉜、㉘ 已做仍写待办；⑯、㉒、㉓、㉖(b)(c)、㉚ 看门狗那一半同样过期 | 采：主 agent 收尾当场改去向列 |
| ㉖(d) 没有对象 | `lib-forbidden-notations.py` 没有 `--selftest` 入口 | 采：㉖ 里删那一件 |
| ⑩ 前提一半不成立 | 上游 `gate-overlap.py` 的 `bash_write_targets` 已认 `cp`、`mv` 的目的路径，只剩 `hook-swap.py` 一种 | 采：⑩ 改成只剩那一种 |
| ㉝(c) 重复、几处该并 | ㉝(c) 与 `.claude/gate.d/break-switch-outside-selftest-pending.tsv` 第 6 行重复；㉚ 剩余与 ㉝(a)(b)、㉗(i) 与 ㉙(a) 同一处 | 采：去向列写「并入」 |
| C879 与比对对象表 | 表里「不登记」一栏今天八行，r7 三行没有指向任何欠账号；暂存区那一版只有五行，㉖(e)「只缩不涨」要先让 r7 三行进基准再立格 | 采为事实：那三行是别的会话加的，登记 ㉟ 并转告；C879 本身不改 |

### K2（攻方：「已定」项撑不撑得住）

| 条 | 腿报 | 主 agent 判 |
|---|---|---|
| K2-1（轻到中） | 共用约束第 63 行教的「输出落进文件、再读汇总行」第十批放行、第十一批被 ⑤ 拒，拒绝说明又写成「没跑过」；真交回新拒 0 份 | 采，零轮：登记 ㉞ |
| K2-2 | `| tail` 不收站得住，该改的是拒绝说明与共用约束第 99 行 | 采：`| tail` 维持不收；措辞登记 ㉞ |
| K2-3（轻） | 命令退 0、输出太长存成 persisted-output 时 `; echo rc=$?` 那一行读不到；两版都有 | 采，零轮：登记 ㉞ |
| K2-7（轻） | 五处例外都说「跑法照验证段第 1 步」，第 1 步没写这条命令，第 4 步写的是整条重跑 | 采，与 K1 那一条合并：登记 ㉞ |
| `$SINGLEFS_WORK_ROOT/` 报告路径（推的） | 与 `lib_work_root.py` 不一致，真交回 0 次 | 登记 ㉞ |
| 库文件头措辞 | 「四处各按自己的规矩办」没写进看门狗 | 主 agent 收尾当场改 |
| 没打中 | worktree 认法（14 种写法）、看门狗取 progress.md、前两轮站住的五种形态 | 记没打中 |

### K3（攻方：报出的数能不能核）

| 条 | 腿报 | 主 agent 判 |
|---|---|---|
| 复出来的 | 402 份（逐字节相同）、4 份、3、46、37、9、22、41；693 今天 695（多的两份是这一轮两条腿），七份自证的种数 | 采：主 agent 复跑核了 402 与 4、22（「一、核查」） |
| 第九批收尾结局账 249 个，照原命令今天 280 个 | 时间窗含当天，记录没留截止点 | 采：主 agent 收尾补跑落盘 280 个，调度记录写明这一份取的是 2026-10-23 写判决时的截面；第九批那一份 249 个是同一窗口更早的截面 |
| 两份判决的 cite-check 读数复不出 | 用的是 `--root .`，今天分别 22 处、20 处对不上；快照目录里没有能当根的树 | 采：方法上登记 ㉞（判决里的 cite-check 改对开工快照倒推出的树跑，或在判决里写明是写判决那一刻的主工作区） |
| 第二轮攻方模型 `orig/` 被改名会话改过，两份 sha256 对不上 | 别的会话的改动 | 采为事实：只差路径，照 `.claude/rules/path-moves.md` 第 6 步不算证据被改，调度记录写明 |

## 三、判决

1. 前两轮站住、第三轮仍站住的形态：第九到十一批的改法——交回闸 ⑤ 认命名检查自己的退出码（`| tail` 不收）、交回闸 ② 与派发闸 ⑰ 认仓里与已登记 worktree 里的报告、看门狗按行首「草稿目录：」取 progress.md、派发闸 ⑳ 读钉住的真提示、草稿目录一行收严、写回员目标、三方腿类型表、三个报告名前缀、坏字节记红、合入后验证那一步与五处例外说法。三轮里两轮以上没被打穿，算站住。
2. 这一轮新打中的都是零轮：K2-1、K2-2 的措辞、K2-3、K2-7 并第 1 步不带前缀、`$SINGLEFS_WORK_ROOT/` 路径、判决里 cite-check 的根。照「第三轮之后停」，不再为它们开轮，记成九十节 ㉞，交用户表列出。
3. 辩方找出的 17 件没去向的延后项记成九十节 ㉟；九十节过期的去向列、㉖(d)、⑩ 的前提、该并的几行，主 agent 收尾当场改。
4. 第十、十一批的收拢表与结局账，主 agent 收尾当场补。

## 四、交用户

| 项 | 被攻过 | 去向 |
|---|---|---|
| 交回闸 ⑤ 与共用约束第 63、99 行的写法对齐（输出落进文件、再读汇总行那一种怎么认；`| tail` 不收的拒绝说明） | 零轮 | 九十节 ㉞，下一个题目 |
| 输出存成 persisted-output 时 `rc=` 那一行读不到 | 零轮 | 九十节 ㉞ |
| 验证段第 1 步补上脚本给的重跑命令与前缀，第 4 步与它对齐 | 零轮 | 九十节 ㉞ |
| `$SINGLEFS_WORK_ROOT/` 开头的报告路径 | 零轮（推的） | 九十节 ㉞ |
| 判决里 cite-check 对哪棵树跑 | 零轮 | 九十节 ㉞ |
| 辩方找出的 17 件没去向的延后项 | 没攻过 | 九十节 ㉟ |

无设计岔路；要不要为 ㉞、㉟ 开下一个题目，收尾时问。

## 五、没核动的

- 辩方模型没复跑，辩方独有的数不引。
- 第十批那一版钩子的 507 种、62 种：那一版已不在主工作区，核不动。
- ⑭ 三轮都没在真进程上量过。

## 回看决策

不涉及决策：被判的都是 agent 定义、共用约束、规则、钩子、门禁与协作脚本的工作流写法，加上两份调度记录；欠账表 C879 只核了两处说法，不改它管的 E165（恢复重放有界穷举与变异杀伤界） 结论与状态。
