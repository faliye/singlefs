# 工具与治理批的零轮形态：第二轮正文（2026-10-13）

<!-- doc-lint:not-numbers V1 V2 V3 V4 V5 V6 V7 V8 L1 L2 L3 L4 L5 L6 L7 X1 X2 X3 X4 X5 X6 X7 -->

## 一、这一轮要判什么

同一题（零轮形态）的第二轮。第一轮判决 `research/prompts/m3-defs-b6-r1-main-verification.md`：V2、V4 站得住，V1、V3、V6、V7 站不住，V5、V8 部分站不住，改法 L1–L7。批 7 三件与批 7 收尾照改法改完（调度记录 `records/2026-10-12-里程碑三工具与治理批.md`）。

第二轮分工（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」）：第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方。这一轮派辩方腿与攻方腿：辩方复核第一轮判决够不够得着、L1–L7 是不是对每一条打中都修到了、有没有改过头；攻方攻 L1–L7 本身与它们带进来的新机制。

| 格 | 第一轮改法 | 改后的形态 | 在哪 |
|---|---|---|---|
| X1 | L1 | ⑮「线程上限：」一行只许一个十进制整数，任何括注都拒，说明另起一行「线程上限说明：…」不判；「线程上限已判」顶缺行；九种类型 | `.claude/hooks/runner-dispatch-guard.sh` ⑮、`scripts/agents/dispatch-prompt.py`、`.claude/main-agent.md`、`.claude/agent-common.md`、九份定义「输入」 |
| X2 | L2 | heavy-thread-defaults 退回「引号里的全大写线程变量名出现就算读」，`#[cfg(test)]` 后接 `mod`（不含外置 `mod x;`）才跳过，`|| X=N` / `&& X=N` 认成钉数，排除表加 `dispatch-prompt.py` 读表那一行与 `replay.sh` 夹下限那一行、删两行陈旧锚串 | `.claude/gate.d/code-tooling.sh`、`.claude/heavy-thread-exempt` |
| X3 | L3 | 重型测试闸仓副本那一支拒绝说明写全起法；`env` 合写选项、缩写长选项、单独的 `-`、`-S` / `--split-string` 照拆再判 | `.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/lib_shell_words.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/agent-common.md` |
| X4 | L4 | ⑥ 与门禁 abandoned-rounds 格：别的轮名按仓里认得的轮名全集判；免出处只给库里写死的四行旧登记，不按日期 | `.claude/hooks/lib_abandoned_rounds.py`、`runner-dispatch-guard.sh` ⑥、`.claude/gate.d/doc-process-records.sh`、`.claude/rules/three-way-inference.md` |
| X5 | L5 | private-values 路径比对按私有值整串在整条路径里找，命中段整段打码、重叠并段 | `.claude/gate.d/doc-text.sh` |
| X6 | L6 | ⑭ 多机一行与脚本文件头 `# long-run-hosts:` 只认 `lib_long_run.py` 的 `CANONICAL_HOSTS_SENTENCE`（逐字，后面可接「；」与不含数字的说明） | `.claude/hooks/lib_long_run.py`、`runner-dispatch-guard.sh` ⑭、`scripts/hosts/detached-run.sh`、`scripts/hosts/host-dispatch.py`、`scripts/agents/dispatch-prompt.py`、定义与 `main-agent.md` |
| X7 | L7 | 豁免表 `.claude/long-run-live-hosts-exempt` 每行带被豁免脚本的 sha256，`exempt-check` 与门禁格 long-run-header-exempt 比对 | `.claude/hooks/lib_long_run.py`、`.claude/gate.d/code-tooling.sh`、`.claude/long-run-live-hosts-exempt` |

被判的定义与规则文件（判决按路径点名）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/agents/clerk.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/tooling-writer.md`。

冻结副本：`~/.local/share/singlefs/work/m3-defs-b6-r2/repo/`（集成副本标签 `after-b7-fixup`，没有 `.git`）；零轮形态起动过的 33 份文件的 sha256 在 `research/prompts/m3-defs-b6-r2-snapshot/frozen-copy-sha256.txt`。第一轮冻结副本（`after-b6-fixup`）在 `~/.local/share/singlefs/work/m3-defs-b6-r1/repo/`，零轮之前那一版在 `~/.local/share/singlefs/work/m3-defs-b3-r2/repo/`。腿只读这三份冻结副本，不读主工作区。冻结副本根下有本机私有配置 `multi-host.env`：报告、模型目录与回复里一个值都不许写出来。

## 二、实现今天的样子（主 agent 的观测，2026-10-13 现查）

- `crates/` 这一题不改。
- 集成副本复核：批 7（`~/.local/share/singlefs/work/m3-tooling-b2-integration/verify8.summary`，184 项判错 4 项：1 项真红是两行陈旧排除锚串，3 项是复核脚本直接设库开关被门禁格覆盖）；批 7 收尾之后（`verify9-out/`）code-tooling 样本档绿、heavy-thread-defaults 与 long-run-header-exempt 与 gate-structure 三格在集成真树上绿、long-run-exempt 四个开关各自红。
- 上一题与这一题第一轮的另立账不在射程里，不再交，除非 L1–L7 让它变得更糟。

## 三、要答的

- X1：只认整数之后，派发提示里正当的说明有没有地方写；「线程上限说明：」那一行会不会被当成第二个「线程上限」。
- X2：退回宽判法之后，第一轮之前那批误判（字符串比较、名字元组、字典实参）回来了几处，排除表能不能兜住、是不是只缩不涨。
- X3：`env` 拆法的边界（`env --` 之后、`env -u`、带等号的长选项、`-S` 里再嵌 `env -C`）。
- X4：「仓里认得的轮名全集」怎么取、漏取一类会怎样；旧行名单与登记表现有各行。
- X5：整串比对的误报（私有值是路径的前缀、常见目录名）；输出里还有没有别的通道漏值。
- X6：规范句逐字之后，现有脚本与定义里的写法能不能照抄过闸；说明里「不含数字」挡掉了哪些正当写法。
- X7：sha256 带进豁免表之后，E171 下一次修订、文件换行符、执行位变化各会怎样。

## 四、分工

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| **云端辩方（Sonnet）** | 复核第一轮判决：L1–L7 落到冻结副本里，是不是对第一轮两腿每一条打中都修到了（拿第一轮两腿报告 `research/prompts/m3-defs-b6-r1-sonnet-output.md`、`-opus-output.md` 与模型目录里的喂法重喂），有没有改过头（第一轮判对的行为被改坏）；第一轮哪一格判重、判轻 | X1–X7 | 逐条「修到 / 没修到 / 改过头」与喂法、改前改后退出码；门禁格起不了的写成小树与命令标「要主 agent 跑」 |
| **云端攻方（Opus）** | 造工作流历史攻 L1–L7 本身与它们带进来的新机制 | X1、X2、X3、X5、X6、X7 | 每条历史写成可复现的步骤，在冻结副本拷贝里真跑钩子与脚本的判定；每条打中在第一轮冻结副本上也喂一遍，写明是这一批带进来的还是之前就中；第一轮已攻过的角度不再交，除非改后又中 |

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格判「站得住」⇒ 照留 | 两条腿在那一格都给不出没修到、改过头或新的判错历史 |
| 一格判「站不住」⇒ 改法进下一批，改完开第三轮（最终轮） | 给出一条可达历史：照定义做却违反用户定案（默认 31、固定不看忙闲、只核自己的且豁免表不登记别人没审过的、私有信息不进仓、在跑的不动、仓副本自证收回放宽），或定义说拒而钩子不拒、钩子拒而定义没写 |
| 打中也打中第一轮冻结副本 ⇒ 不拿它否这批改动，另立一笔账 | 同一段历史在 `~/.local/share/singlefs/work/m3-defs-b6-r1/repo/` 上给出同样的错 |
| 辩方报「改过头」⇒ 照「站不住」办 | 第一轮判对的一段历史，改后判错 |

⚠️ 打中之后先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 六、各条腿交什么

- 方案按冻结副本今天的样子谈（`crates/` 照「实现今天的样子」那一节）。
- 报告先写进 `research/prompts/m3-defs-b6-r2-<腿名>-output.md`（辩方 `-sonnet-output.md`、攻方 `-opus-output.md`），每次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建，回复只写短句。
- 草稿放腿自己的目录 `~/.local/share/singlefs/work/m3-defs-b6-r2-<腿名>/`；要改钩子或脚本做实验的，在那里另拷一份冻结副本改，不改冻结副本本身。驱动脚本与喂法随报告拷进 `research/prompts/m3-defs-b6-r2-<腿名>-model/`（含 `tools/rerun.sh`；复跑经 `scripts/hosts/run-with-memory-cap.sh` 时上限给 2G）。
- 引规则、定义与 kb 条款写那份文件自己的行号，去冻结副本里现查，不从背景材料里数。
- 不跑登记的 `--selftest`，核钩子用喂 JSON 给钩子入口、核脚本用它的判法函数；门禁格（`--check`）子 agent 起不了，读格的代码造出小树与命令写进模型目录，标「要主 agent 跑」。
