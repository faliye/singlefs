# agent-defs-r2 本地辩方运行记录（2026-09-17）

提示：`research/prompts/agent-defs-r2-local-defense.md`（63 行、5064 词；三条候选各配一问攻击，26 条事实 F1–F26）；
核对表：`research/prompts/agent-defs-r2-local-defense-translation-audit.md`。
前台跑 `nice -n 19 bash research/scripts/ask-local.sh <提示文件>`，未用 `setsid` / `&` / `disown`。

开工前 `ps` 看到本机有一个 `cargo test --release -p singlefs-harness --test second_transaction_step_zero_layer0` 在跑（别的会话的）；
`three-way-local-defense`（依 `three-way-local-attack` 的「做什么」）没有「开跑前 ps 见重负载就停」这条要求（那是给会编译 / 跑门禁的定义准备的），
这一轮不编译、不跑门禁，`ps` 检查不适用，照常跑。

## 逐次调用

| 次序 | 目标文件 | 退出码 | 词数 | 判定依据 | 结果 |
|---|---|---|---|---|---|
| 1 | `-output-s1.md`（首次尝试） | 5 | 0（该目标文件本次为空） | `ask-local.sh` 内部字词损坏闸判红：`生词=4 拼接=1`，拼接命中 `streamlines(=stream+lines)`；其余生词 `exactness incident's assertion's` | 作废，原样输出留存为 `research/prompts/agent-defs-r2-local-defense-output-void1.md`；`-output-s1.md` 本次为空（0 字节），换下一次重跑覆盖同一个 s1 槽位 |
| 2 | `-output-s1.md`（重跑） | 0 | 289 | `ask-local.sh` 内部闸判绿；另跑 `python3 research/scripts/oov-check.py`（三方规则「闸判绿也要跑 oov-check.py」）：生词=4（`archiving`、`assertion's`、`registration's` 等），拼接=0，均为合法英文词或所有格形式；通读一遍未见缺词、断句 | 记为 **s1，干净** |
| 3 | `-output-s2.md` | 0 | 354 | `ask-local.sh` 内部闸判绿；`oov-check.py` 复核：生词=4（`subagents`、`retelling`、`subagent's`、`archiving`），拼接=0，均为合法英文词；通读一遍未见缺词、断句 | 记为 **s2，干净** |

两份干净样本已够（`three-way-local-attack.md`「做什么」第 6 条「攒到至少两份干净样本为止」，`three-way-local-defense.md` 依它）。
三次调用里跑了 1 次判红、2 次判绿，坏率（1/3）低于 `three-way-local-defense.md` 记录的「实测（2026-09-16 第一轮）：辩方提示在本地模型上五次坏三次」，未到「连续五次拿不到两份干净」的停止线。

本地网关（`~/code/ai-center` :8200）全程可用，没有缺席；两次成功调用与一次判红调用均在前台完成，未出现「正文为空」（退出码 4）或网关不通（退出码 2、3）的情形。

## 样本内容形状（不解读、不采纳，只记录形式是否合规）

s1、s2 均按提示要求编号 1–3 作答、每条以 `Refuted if:` 收尾，未见 markdown 粗体/斜体/标题。
两份样本对同一问题给出的判定方向一致：Q1、Q2 均判 `DOES NOT HOLD`（旧做法站不住），Q3 均判旧做法（F19）站得住、新规则（F18）过宽。
s1、s2 两份的具体论证路径不完全相同（例如 s2 对 Q1 多绕了一层「replace-once.py / replace-batch.py 本身算不算经过 Write/Edit 工具」的论证，与提示给的事实 F4「旧规则下这些脚本是经 Bash 调用的」在字面上不完全贴合）；这一差异是否影响判定的可靠性，不在本轮职责内判断，交主 agent。

## 没做什么（固定会有的）

- 不解读、不总结、不采纳本地模型的答复；打没打中是主 agent 的事。
- 未读别的腿这一轮的产出（`research/prompts/agent-defs-r2-*-output*.md` 中云端腿的产出、`agent-defs-r2-*-model/`、`agent-defs-r2-local-attack*`）与别的腿的草稿目录，按这一轮禁读清单执行。
