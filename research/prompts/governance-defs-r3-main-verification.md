# 严查第二轮之后的七份定义改动：第一轮判决（2026-09-26）

正文 `research/prompts/_governance-defs-r3-body.md`；背景材料 `research/prompts/_governance-defs-r3-background.md`；附录二 `research/prompts/_governance-defs-r3-diff.md`；开工快照 `research/prompts/governance-defs-r3-snapshot/sha256sums.txt`（41 行，腿交齐后 `sha256sum -c` 全部 OK）；派腿时刻 2026-09-26T22:20:01Z。

被判的 7 份：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/agents/crash-verifier.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/kb-scribe.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-verifier.md`。

## 一、这一轮交了什么

| 腿 | 报告 | sha256 |
|---|---|---|
| 云端正推（Sonnet） | `research/prompts/governance-defs-r3-sonnet-output.md`（168 行） | 50c136c14759fc3927d9142253330feae1773f46ea22b16da13da16788b8e50f |
| 云端攻方（Opus） | `research/prompts/governance-defs-r3-opus-output.md`（368 行），模型 `research/prompts/governance-defs-r3-opus-model/`，复跑 `bash run-all.sh <草稿目录>` | ba86f77755a78a60176b55e09422e1b74fc9735217861d2d1fbf016cd4f6f83b |
| 核查员 | `research/prompts/governance-defs-r3-verifier-output.md`（115 行） | e7fb9d07558f12c7f185bee11ac2fa17b7a10405944912fa60abfad83b81608f |
| 本地攻方 / 本地辩方 | **缺席**：这个容器里没有 `~/code/ai-center`；用户选「再走一轮三方（缺本地腿）」 | — |

核查员：两份报告约 46 处可核事实，✓ 43、✗ 0、核不动 2、不适用 1；攻方模型复跑退 0，路径归一化后与 `run-all.out` 逐字相同。核查员点出两条腿共同依赖、都没真跑过的前提：测试二进制被信号杀时 cargo 打什么。主 agent 在草稿目录造一个小 crate 实测（两条测试，第二条 `std::process::abort()`），把 59 号 `judge_one` 的判档逻辑原样抄来喂 stdout + stderr：点名的测试是被杀的那一条 ⇒ 没红；点名的测试跑完之后才被杀 ⇒ 没红；点名的测试没出现 ⇒ 无效（stderr 有 `error: test failed, to rerun pass ...` 与 `process didn't exit successfully: ... (signal: 6, SIGABRT ...)`）。前提坐实。

## 二、跑前条款，各触发没触发

| 条款 | 触发没触发 |
|---|---|
| 一格判「站得住」⇒ 照留 | G4 的连字符与 `driver_e<号>` 登记、G5 的 kb-scribe 两处照留 |
| 一格判「站不住」⇒ 改那一处，判决标「被攻过零轮」 | 触发：D1、D3、D4、D5、D7 与正推 G2、G3 的缺口；D6 轻，一并补 |
| 删改丢了判据 ⇒ 补回 | 没触发 |

## 三、逐条判决（主 agent 逐条现查过）

| 编号 | 格 | 打中了什么 | 现查 | 判决与改法 |
|---|---|---|---|---|
| D1 / 正推 G1 | G1 | 写回的「有 `error:` 行记无效」与 59 号判档次序相反：点名的测试那一行出现了就先记没红 | 实测属实（见第一节） | 站不住。mutation-sampling.md、mutation-triage.md 改成 59 号的判档次序，列全「无效」的三种来源 |
| D2 | G1 | 「无效」的第三个来源：点名的名字认不出；`crates/mutations.tsv` 有 11 行测试名带 `$`，59 号按字面转义去找，永远认不出 | 属实（第 202、297、298、684–689、693、694 行；`git diff 73ba4a4 -- crates/mutations.tsv` 为空，分支之前就这样） | 不分辨，不拿它判这一批；定义里补上这一种来源；表本身交用户 |
| D3 | G2 | 未跟踪文件三道共用 `crates litmus .lkmm-static-only`，别的会话起草 litmus 会挡住不读它的 55、59 | 属实 | 站不住。改成每道各查各的：55、59 查 `crates`，57 查 `crates litmus .lkmm-static-only` |
| 正推 G2 | G2 | 57 号读的 `lib.sh` 在被 git 忽略的规范副本里，定义没交代 | 属实 | 补一句 git 核不到、不在这一步里 |
| D4 | G3 | 被 git 忽略的文件 `git log` / `git status` 看不见改动，核查员会记「没改过」 | 属实（`.claude/singlefs-ai-sop/` 整个被忽略） | 站不住。这类文件对不上记「分不清」 |
| D5 / 正推 G3 | G3 | 只有代码轮一节要求记派腿时刻；设计轮没时刻时判别力自证做不出 ✗；自证挑选少了「`sha256sum -c` 对得上」 | 属实 | 站不住。main-agent「要推论」一行要求给核查员派腿时刻与快照；自证改成只拿副本那一行比，不走「分不清」那几支，挑选条件补上 |
| D6 | G4 | 同一实验第二个 `driver_e<号>` 重名会被 bash 静默盖掉 | 属实（推的后果，照 E158 先例不会撞） | 轻，补：先 grep 现查、加后缀 |
| D7 | G5 | 「2 是包装的用法写错」与包装原样传出命令的 0–249 相反 | 属实（`run-with-memory-cap.sh` 文件头第 18、27 行） | 站不住。改成两种来源看输出分 |
| 正推 G2 | G2 | 变异分诊员输入「59 号输出路径」没指到崩溃验证员的日志路径 | 属实 | 改：写明取崩溃验证员报告里 59 号那一行的日志路径 |

写回的每一处都被攻过零轮，由第三轮确认严查接着攻。

## 四、交用户的

| 项 | 为什么交用户 |
|---|---|
| `crates/mutations.tsv` 11 行测试名带 `$`，下一次真跑 59 号必红 | 改的是 `crates/` 下的变异表或 59 号的判法，分支之前就在，越出这一轮的描述修正 |
| 崩溃验证员的 55、57、59 与 gate-staged.sh 里的同几道各跑一次 | 同 `research/prompts/governance-defs-r2-main-verification.md` 第四节，仍未定 |

## 回看决策

不涉及决策：这一轮判的是 agent 定义与共用约束、两份规则的文字，没有动任何一条决策分项的定案、射程或依据，也没有新增或撤回实验结论。
