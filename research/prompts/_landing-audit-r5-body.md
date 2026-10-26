# landing-audit-r5：子 agent 落地审计第九、十批改动第二轮正文（2026-10-23）

<!-- doc-lint:not-numbers K1 K2 K3 -->

## 一、题面

第一轮判决 `research/prompts/landing-audit-r4-main-verification.md` 采了八条修补（拆分提案 `records/2026-09-16-subagent拆分提案.md` 第九十节 ㉘），第十批四件（任务 32 到 34，逐件结果在调度记录 `records/2026-10-22-子agent结果落地审计.md` 节「第十批」）照它改了定义、规则、钩子、门禁与脚本，交回报告归档在 `research/prompts/subagent-landing-audit-2026-10-22/b10-*.md`；主 agent 另改了几处（「实现今天的样子」列了）。这是第二轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第二轮分工：第一轮的修补本身、产生结论的方法、从未被看过的地方），不重复第一轮已攻过的角度（第一轮：每处改法做没做到九十节那一行、新句子与已有的矛盾、哪一条最脆）：

1. 第一轮判决够不够得着：采与不采的理由站不站得住，收严有没有丢掉原来罩住的对象。
2. 第十批的修补本身：改法做没做到判决那一条要的，带没带出新的误拦、误放、矛盾。
3. 产生结论的方法：第一轮的打中全是在拷出来的钩子上喂造出来的输入量的，真派发、真交回里这些形状出现过几次；两轮都没看过的地方。

被判的文件：

| 类 | 文件 |
|---|---|
| 定义、共用约束与规则 | `.claude/main-agent.md`（第 3 条、第 7 与 29 行、调度表合入后验证那一行与写回员那一行）、`.claude/agent-common.md`（第 58 行一带）、`.claude/rules/implementation-workflow.md`（「checker 档只在提交时跑，harness 随时跑」那张表两行） |
| 钩子与库 | `.claude/hooks/heavy-test-guard.sh`（`MERGE_VERIFY_QUICK_STEP` 与两处拒绝说明）、`.claude/hooks/runner-dispatch-guard.sh`、`.claude/hooks/handback-guard.sh`、`.claude/hooks/completion-cleanup-guard.sh`（只判认草稿目录那一行的读法）、`.claude/hooks/lib_prompt_scratch_line.py`（新建）、`.claude/hooks/lib_three_way_legs.py`、`.claude/hooks/selftest-registry.tsv` |
| 门禁与登记 | `.claude/gate.d/code-tooling.sh`（break-switch-replay 格按字节收输出、`SELFTEST_RUNNERS` 加新库）、`.claude/gate.d/break-switch-replay-files.tsv`、`.claude/gate.d/replay-comparison-units.tsv`（文件头指到 C879 那一行）；新样本 `.claude/gate.d/fixtures/code-tooling.sh/break-switch-replay-undecodable-red/` |
| 脚本 | `scripts/agents/subagent_transcript.py`（平常跑设开关就拒）、`scripts/agents/agent-watch.py`（只判自证里造的 ✗ 改成八进制转义那四行） |
| kb 与记录 | `.claude/kb/checks-owed.md` 的 C879（E165 根选择与杀伤界正式跑没有单元）那一句（三语各一份）；`records/2026-09-16-subagent拆分提案.md` 第九十节 ㉘ 到 ㉛；调度记录节「第九批收尾」「第十批」 |

## 二、实现今天的样子（主 agent 的观测，2026-10-23）

方案按 `crates/` 与 `scripts/` 今天的实现来谈（主工作区现状）。这一批没有改 `crates/` 下的任何文件；`scripts/gate/merge-and-verify.sh` 的 checker 档快档那一步判的是 `crates/singlefs-checker-tier` 的快档测试，这一批只改了说它的那几句定义与拒绝说明。

- 主 agent 在主工作区重跑过的读数（调度记录节「第十批」逐件写了）：派发闸 507 种、交回闸 62 种、`lib_prompt_scratch_line.py` 18 格、`heavy-test-guard.sh` 1402 种（其中该拒 266 种）、完工清理闸、`lib_three_way_legs.py`、`dispatch-prompt.py` 92 项、`agent-cost.py` 23 项、`subagent_transcript.py` 13 项、`agent-watch.py`，都退 0；门禁 code-tooling 的 break-switch-replay、break-switch-outside-selftest、break-switch-headers 三格绿，样本档 `stage-selftest.sh --item code-tooling.sh` 97 个用例判对；research-gate-lint 格剩两类红归别的会话（暂存区两份 `.sh` 没有执行位、两处循环里逐个 kill），这一批的那三处已清。
- 主 agent 自己改、没被工具员做过的：`heavy-test-guard.sh` 给主 agent 的「照抄」那一行加 `merge_verify_hint`（没有单独的自证格）；`code-tooling.sh` 的 `SELFTEST_RUNNERS` 与 `break-switch-replay-files.tsv` 各加新库一条；`agent-watch.py` 自证里四行 `printf` 的 ✗ 换成 `\342\234\227`；`replay-comparison-units.tsv` 文件头与 C879 三语互指；交回闸 ⑤ 也认退 1 的命名检查（整条最后一条就是那条命名检查时）是主 agent 判下的。
- 别的会话这几天在同几份文件上也有改动，不在被判之列：`bash-command-detector.sh` 的 ⑫ 收严、`completion-cleanup-guard.sh` 的运行目录归属判法、`heavy-test-guard.sh` 拒在主工作区直接起会编的 cargo、`scripts/hosts/` 下的编译目录写法。腿判到它们时写明是别人的改动。
- 暂存区里有一截不是本会话暂存的改动，快照对 HEAD 取 diff。

开工快照：`research/prompts/landing-audit-r5-snapshot/judged-sha256.txt`、`research/prompts/landing-audit-r5-snapshot/judged-vs-head.diff`。

## 三、格

### K1　第一轮判决够不够得着（辩方）

- 第一轮判决节「二、逐格判」里不采的两处（⑯ 前一半：派得出就算坐实；⑯ 后一半：线程上限与预计挂钟两张表不算三方腿类型表）与「没打中」记的几处：理由站不站得住。
- K2-4 并 K3-2 收严成只认行首「草稿目录：」：判决说「收严不丢罩住的对象」，第十批交回 `b10-hooks.md` 第八节写了 ⑯ 收严后行首带列表记号、引用记号或粗体的不再建写者锁，`a9717cf07fb6e7c21` 那一类 ⑰ 不再拦。这两样是不是判决当时就该算进代价的，判决那句话还成立吗。
- 主 agent 判下的交回闸 ⑤ 认退 1、K2-6 只补互指不改 C879 结论：是不是同样打中替代方案（例：只认退 0 会不会误拒真跑完的；K2-6 换成删那五行会怎样）。

### K2　第十批的修补本身（攻方）

- 新库 `lib_prompt_scratch_line.py` 与四处调用：认法与骨架 `python3 scripts/agents/dispatch-prompt.py <类型>` 打的那一行是不是逐字一致；围栏代码块里、引用里的「草稿目录：」认不认。
- 交回闸 ⑤ 新判法（顺序执行位置、退出码）：任务 33 交回自己列的认不出的写法之外，还有没有误拒真跑完的、误放没跑完的。
- `lib_three_way_legs.py` 的 `is_leg_report_path`、`leg_report_mentions`：派发闸与交回闸两侧认法逐字一致吗；派发闸自证读的 14 份真提示按 sha256 钉住，那个目录被删或改时这一格怎么报。
- `MERGE_VERIFY_QUICK_STEP` 与两处拒绝说明、四处定义里的例外说法：几处写的是不是同一个例外，有没有一处写宽了（例：把「合入后验证那一步」写成「合入后验证」整段都不弹窗）。
- `code-tooling.sh` break-switch-replay 格按字节收之后：坏字节以外的判法与改前逐条相同吗；超时、退出码的收法有没有跟着变。

### K3　产生结论的方法与没看过的地方（攻方）

- 第一轮 K3-1、K3-2 打中的形状与第十批收严之后新拒的形状，在真会话记录（`~/.claude/projects/-home-fy5090-code-singlefs/*/subagents/`）的派发提示与交回里出现过几次；照「一条腿只抽一次样不算一次观测」，给出数法与数。
- 两轮都没看过的：拆分提案九十节 ㉚ 记的 `scripts/` 下另三种草稿目录认法与新库并存时，同一份提示在派发闸、骨架、账表、看门狗那里认出的草稿目录会不会不同；`subagent_transcript.py` 守卫对 import 它的三处（`agent-cost.py`、`continuation-guard.sh`、`agent-watch.py`）平常跑时有没有影响。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | K1 | 复核第一轮判决：采与不采的理由够不够得着，收严的代价算全没有 |
| 云端攻方（Opus） | K2、K3 | 攻第十批的修补本身，拿真会话记录量第一轮打中的形状 |
