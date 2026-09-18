# 判决：后台任务的通知与交回，三方第一轮（bg-notify-r1，2026-09-18）

<!-- doc-lint:not-numbers A1 A2 A3 A4 L9 L10 L13 L14 X01 X02 X03 X04 X05 X06 X07 X08 X09 X10 X11 X12 X13 F01 F02 F03 F04 F05 F06 F07 C13 V1 V2 V3 -->

正文 `research/prompts/_bg-notify-r1-body.md`；背景材料 `_bg-notify-r1-background.md`；附录二 `_bg-notify-r1-diff.md`；开工快照 `research/prompts/bg-notify-r1-start-snapshot.sha256`（8 个文件；腿与核查员跑着时主 agent 没改。写回前 `sha256sum -c` 只有 `.claude/rules/implementation-workflow.md` 不符，是别的会话加的「测试与崩溃检测优先多线程」一节，这一轮没引它的新内容）。
腿：Opus 攻方 `bg-notify-r1-opus-output.md`（模型 `bg-notify-r1-opus-model/`）、Sonnet 正推 `bg-notify-r1-sonnet-output.md`（模型 `bg-notify-r1-sonnet-model/`）、本地攻方两份样本 `bg-notify-r1-local-attack-output-s1.md`、`-s2.md`；核查员 `bg-notify-r1-verifier-output.md`。

## 一、逐格判定

| 格 | 判定 | 依据 |
|---|---|---|
| A1① 漏禁 | **打中**：共用约束那句只禁三个词与「末尾的 `&`」，`… & echo …`（X01，事故那条删掉 nohup 与 disown）、子 shell 与命令替换里的 `&`（X02、X06、X07）、`wait $!` / `wait -n`（X04、X05）、`coproc` / `tmux -d` / `screen -dm` / `systemd-run`（X08–X11）照字面都放行而都会脱钩 | Opus 报告 A1①，X06 在真 harness 上端到端量过（15:20:02 报完成、真活 15:20:42 才完）；核查员复跑数据一致 |
| A1① 误禁 | **打中一处，形状与腿说的不同**：共用约束那句禁 `setsid`，而裸 `setsid`（不带 `-f`）不 fork，外层照样等 | 主 agent 实测 `bash -c 'setsid sleep 2 …'` 外层 2.0 秒，核查员复现 2.0026 秒；Opus 报告说「误禁的只有 nohup 不带 & 与 setsid -w」隐含裸 setsid 会脱钩，33 条形态里没测过，这句作废（核查员 ✗ 第 2 条） |
| A1② | **打中，弱**：长活与缓存计时器并进同一条 `… & … & wait`（C13）两句字面都满足，而计时器的叫醒丢了 | Opus 报告 A1② |
| A1③ | **打中**：交回那种通知的 note 写「fires each time this agent stops with no live background children」，所以「结束本轮、手里没有在跑的后台任务、也没交回」也发 completed，入口那段没写怎么读；看门狗对它与「等后台」报同一个状态 | Opus 报告 A1③；看门狗 `agent-watch.py` 旧版第 214 行 |
| A1④ | **打中两处**：`.claude/main-agent.md` 第 23 行（主 agent 自己的长命令放后台）与第 21 行（派 general-purpose 的提示只带计时器那一条）没带新规矩；弱的一处（本地腿前台跑与 240 秒前台上限）不在这一轮课题内，不改 | Opus 报告 A1④ |
| A2 表外 | **打中**：仓里 hook 在 33 条表外命令上漏 10、误 7（F02 轮询、F03 `tail --pid`、F04/F05 heredoc 里的 Python 按位与与 Rust 引用都常见）；Opus 提的变体在它的模型上漏 1（X13 前台）、误 1（F06 sed 替换串），**被攻过零轮** | Opus 报告 A2 一至五节；核查员拆两段复跑，数据一致（`run.sh` 一次跑在高负载下 3 次崩溃，是 `probe.py` 第 75 行的竞态，核查员 ✗ 第 4 条） |
| A2 L1–L14 | **本地腿两份样本在 L9、L10、L14 上一致地错**：L9、L14 第②格答「引号里的 & 不算」，与提示里的判据字面及同一份答复对 L13 的判法矛盾；L10 第③格答「setsid 会脱钩」，与实测相反。两份样本的「没打中」与这三格都不采信；它们一致指出的 L13（引号里的 `nohup` 被记）与 L14（引号里内层 shell 的 `&` 真脱钩）与 Opus 同向，按 Opus 的量采纳 | 核查员 ✗ 第 1 条；主 agent 的 hook 真值 `bg-notify-r1-hook-truth.tsv`（在会话暂存目录，核查员独立重跑 14/14 一致） |
| A3 | **一致**；缺一处：入口没写 TaskStop 停掉的子 agent 怎么读（只在别人发起 TaskStop 时才是真缺口） | Sonnet 报告 A3；两句英文 `grep -nF` 精确命中 |
| A4 | **打中一处**：主会话记录不在 `<会话目录>.jsonl` 时看门狗静默退回旧判法；其余四种情形判得对，其中「续做之后又停」与 check-staged「红与 77 混着」仓里自检没有样本 | Sonnet 报告 A4（腿另起临时仓与测试核过）；核查员复跑一致。Sonnet 报告把 `check-staged.sh` 第 75/82/86/89 行抄成带 `#` 注释的样子，源码里没有这些注释（核查员 ✗ 第 3 条）——行号与语句本体对，引文不作依据，结论按腿的复跑产物采信 |

## 二、写回（这一轮做完的）

| # | 打中 | 改在哪 | 被攻过几轮 |
|---|---|---|---|
| 1 | A1①漏禁与误禁、A1② | `.claude/agent-common.md` 第 40 行：交给 `run_in_background` 的命令要一直跑到真正的活结束才退出，列出不许用的放后台形态（`disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 分离、`systemd-run`、子 shell 或 `$( … )` 里的 `&`），`&` 只在随后用不带参数的 `wait` 等齐时用；缓存计时器单独起一次 | 零轮（第一轮攻的是改前那句） |
| 2 | A1③、A3 缺的一处 | `.claude/main-agent.md`「交回怎么读」：第三种 completed（没有交回、没有那两句）读成「不会自己醒」，看它最后在等什么再发消息或停掉；TaskStop 停掉的不再交回、看门狗按被停退出 | 零轮 |
| 3 | A1④ | `.claude/main-agent.md` 第 21、23 行：派不读共用约束的 agent 时把「长活可以等」那一条的后台写法写进提示；主 agent 自己的长命令同样照那一条写 | 零轮 |
| 4 | A2 表外 | `.claude/hooks/bash-command-detector.sh` 换成 Opus 的变体，另把 `setsid` 收成只认 `-f` / `--fork`（核查员核出的误检）；文件头写明认的形态与判不到的（前台 `… &`、引号里当数据的 `&`、`… & (wait)`、`bash -c '… &'; wait`）；自检加 X01–X13、F01–F07 与三条 setsid，共 40 种，`BASH_COMMAND_DETECTOR_DISABLE_SELF_BACKGROUND=1` 判红 17 种 | 零轮（Opus 的变体只在它自己的模型上量过） |
| 5 | A1③ 在机器那一侧 | `research/scripts/agent-watch.py` 新告警「结束本轮却不会醒」：结束本轮、没交回、起过的后台任务都已收到完成通知（`origin` 为 task-notification 的消息或 `queued_command` 附件）、之后没有新通知，60 秒后报；自检加「当场完了又去等」「通知刚到正要醒」两个样本，`AGENT_WATCH_BREAK=nowake` 判红 | 零轮（主 agent 自己加的） |
| 6 | A4 | `agent-watch.py`：主会话记录不在时报告里写明「被停只按打断判」；自检加「停了、续做、又停」样本与「主会话记录不在」一格。`research/scripts/check-staged.sh` 自检加「红与 77 混着」一格 | 零轮 |

对 `a339b505557ce905e` 那一段历史（13:53:44 结束本轮时起过六个后台任务，四个当场完了、两个还活着）新告警不报，是对的：那一刻它确实还有东西会叫醒它；脱钩的 `gate.sh` 由 hook 的第三种检出管。

## 三、交用户（零轮形态）

上表六项都没被攻过。按 `.claude/rules/implementation-workflow.md`「打中的写回代码，再攻一轮」应当再开第二轮攻这六项；用户 2026-09-18 要求这一批做完就提交收尾，第二轮没开，登记在 `records/2026-09-16-subagent拆分提案.md` 第三十三节。第二轮要攻的面：hook 变体在真实命令上的误检率（Opus 没有频率数）、「结束本轮却不会醒」的 60 秒宽限在主 agent 忙时通知排队（拆分提案第三十节，最长 10 分 18 秒）下会不会误报、共用约束那句的清单是不是又漏了一种放后台的形态。

## 四、没派的与没做的

没派辩方腿（第一轮）。本地腿抽了两次，两次一致地在三格上错，按「没打中一次不算」与核查结果不采信它的「没打中」。

## 五、同一批里用户直接定的定义改动（没走三方）

用户 2026-09-18 定：三方的云端腿 effort 改为 high，原话「将三轮中的opus 和 sonnet 改为high 模式 现在太高了。 无需验证这一点，改了就行了」。
改的是 `.claude/agents/three-way-attack.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/three-way-defense.md` 三份 frontmatter，各加一行 `effort: high`；这三份没有被这一轮或别的轮攻过，这里点名只为登记它们的来由。
同一天用户又定：去掉子 agent 等长活时约 4 分钟一次的缓存续期（原话「每 4 分钟复检一次子 agent 内部的ping 也可以去掉了」）。`.claude/agent-common.md` 第 41 行改成「等长活时不起缓存计时器，结束本轮直接等完成通知」，写回 1 里「缓存计时器单独起一次」那句随之删掉；`.claude/main-agent.md` 第 21 行改成子 agent 不续提示缓存、主 agent 也不 ping；同样没走三方。`research/scripts/cache-keepalive.sh` 留在仓里，没有定义再叫它。
