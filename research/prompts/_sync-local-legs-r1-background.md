# 同步合并后补回的本地腿两份定义：第一轮正文（2026-09-27）

## 一、这一轮要判什么

2026-09-27 同步远端之后，用户弹窗定「补回远端段、以本地为准、参考远程」。主 agent 据此改了两份定义（diff 在附录二 `research/prompts/_sync-local-legs-r1-diff.md`，逐块去向在 `records/2026-09-27-同步远端冲突块.md`「去向」一节）：

- `.claude/agents/three-way-local-attack.md` 第 3、4、5 步：取号（`ls` 看已用到几号）；保留本地「前台跑、不许 `setsid`、`&`、`disown`」，另加「单次请求最长 `ASK_LOCAL_TIMEOUT`（默认 900 秒）超过 Bash 前台上限时改用 Bash 的 `run_in_background` 起同一条命令、结束本轮等完成通知、退出码取通知里的」；退出码 5 判红后先确认空文件再用 `>|` 重跑同一个号；退出码 6 算「非 0 非 5」停下报缺席；`oov-check.py` 带上提示文件、生词表只打前 300 个字符。
- `.claude/agents/three-way-local-defense.md` 第一段：读攻方定义的小节名从「输入」改成「输入（主 agent 必须给）」。

同一批内容的远端形态（本地腿一律 `run_in_background` 起）在远端治理严查轮里攻过三轮（判决 `research/prompts/governance-defs-r1-main-verification.md`、`-r2-`、`-r3-` 里的 G3 格）。这一轮判的是合并之后的形态，远端判决只当背景，不替这一轮作保。

要判的是：**照改后的两份定义做，有没有一条可达的工作流让本地腿做错、做不下去、被钩子拒、或与规则说法相反。**

## 二、实现今天的样子（主 agent 的观测，2026-09-27 现查）

- 这一轮不动 `crates/`：被判的只有两份 agent 定义，`crates/` 里没有本地腿相关的代码（`grep -rln 'ask-local' crates` 零命中）。
- `research/scripts/ask-local.sh` 第 13 行文件头写退出码；代码里的退出点：第 26、30 行 `exit 2`（取不到 key、提示为空），第 41 行 `exit 3`（请求失败），第 107 行 `exit 5`（损坏闸判红、没设 `ASK_LOCAL_ALLOW_CORRUPT`），第 130 行 `exit 6`（检测器没跑成或找不到）。第 21 行附近 `TIMEOUT="${ASK_LOCAL_TIMEOUT:-900}"`。
- `research/scripts/oov-check.py` 第 249 行 `scan(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None, words)`：第二个参数是提示文件；第 257 行生词表 `[:300]` 截到 300 个字符。
- `.claude/hooks/bash-command-detector.sh`：⑤ 只管整份覆盖 `research/results/` 下未跟踪的文件（第 81、575 行）；⑦ 管在同一个 inode 上改已存在的脚本；④ 管 `run_in_background` 里单独的 `&`（第 198 行 `LONE_AMPERSAND`）。
- `.claude/rules/three-way-inference.md` 第 206 行（「给本地腿的提示一律用英文」一节里，提交 bfc447ea 起是这一句）：⚠️ **本地腿用 Bash 的 `run_in_background` 起、结束本轮等完成通知（单次请求最长 900 秒，超过前台上限），命令里不加 `setsid`、`&`、`disown`。**
- `.claude/agent-common.md`「长活可以等」那一条：长活用 Bash 的 `run_in_background` 起、起完结束本轮。
- Claude Code 的 Bash 工具前台单次上限是 600000 毫秒（工具说明里写的最大 timeout），这是工具的事实，不在仓里。
- 本机本地模型网关 `localhost:8200` 有响应（`curl` 不带 key 得 401）。

## 三、各格要答什么

| 格 | 要答 |
|---|---|
| 甲 前台与后台 | 「前台跑」与「超过前台上限时改用 `run_in_background`」合在一起，腿在开跑之前判得出该走哪一支吗（它事先不知道这次请求要多久）？与 three-way-inference.md 第 206 行「本地腿用 Bash 的 `run_in_background` 起」那句、agent-common「长活可以等」那条是否相反？ |
| 乙 钩子 | 照第 3、4 步写出的命令（`bash research/scripts/ask-local.sh <提示> > <前缀>-output-s<n>.md`、判红后 `>|` 同号重跑、放进 `run_in_background`）会不会被 `bash-command-detector.sh`、`heavy-test-guard.sh`、写范围闸或看门狗拒绝、记检出或误报「结束本轮却不会醒」？ |
| 丙 退出码与计数 | 第 4 步把 2、3、6 都归「停下报缺席」、5 作废重跑，第 6 步「连续五次调用拿不到两份干净的就停」：与 ask-local.sh 的退出点逐个对得上吗？有没有一种退出码走进两条规定、或一条都不走？ |
| 丁 生词表与小节名 | oov-check 带提示文件、300 字截断之后「打满了就逐段通读」这条判得出「打满了」吗？local-defense 改的小节名与攻方定义今天的小节标题逐字一致吗，读那四节之后防方与攻方不同的几处还站得住吗？ |

## 四、分工

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| 云端正推（three-way-forward，sonnet） | 正推：逐句核改后的定义与脚本、规则今天的行为一致 | 丙、丁 | 每一句改动一行：它说了什么、对应的代码或规则在哪一行、一致与否 |
| 云端攻方（three-way-attack，opus） | 反推：找让腿被拒、做错、做不下去的路径 | 乙（钩子、写范围闸、看门狗） | 每条打中给可复现的命令与钩子判定；副本上实跑钩子的 `--selftest` 或喂 JSON 可以，不跑重型测试 |
| 本地攻方（three-way-local-attack） | 反推：规则文字之间的矛盾与时间算术 | 甲 | 按事实表逐格填：900 秒与 600 秒、前台 / 后台两支的判定条件、两条规则原句各要求什么 |

两条攻方腿的攻击面不重叠：云端攻方只管钩子与工具层（乙），本地攻方只管规则文字与时间算术（甲）。

## 五、禁读清单

各条腿不读别的腿这一轮的提示与产出（`research/prompts/sync-local-legs-r1-*-output*.md`、`-local-attack*.md`、`-runlog.md`）。

---

# 附：小节清单（kb-sections.py 全量生成，未经过滤，逐行标抄 / 不抄 / 理由）

### 小节清单：`.claude/agents/three-way-local-attack.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 本地攻方腿（three-way-local-attack） | 抄 | 这一轮判的正是这份改后的定义，全篇要看 |
| （本地攻方腿（three-way-local-attack） 标题之下、第一个下级标题之前的正文：第 12-17 行） | 抄 | 随（与上一行合并成 1-17 行） |
| ## 输入（主 agent 必须给） | 抄 | 丙格要核输入项与钩子、退出码对应 |
| ## 做什么 | 抄 | 正文第 3、4、5 步逐句改动都在这一节，甲丙丁三格都要核 |
| ## 写范围 | 抄 | 全篇一并核，判会不会写到写范围外 |
| ## 产出 | 抄 | 全篇一并核 |
| ## 没做什么（固定会有的） | 抄 | 全篇一并核 |

### 小节清单：`.claude/agents/three-way-local-defense.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 本地辩方腿（three-way-local-defense） | 抄 | 这一轮判的正是这份改后的定义 |
| （本地辩方腿（three-way-local-defense） 标题之下、第一个下级标题之前的正文：第 12-15 行） | 抄 | 随（与上一行合并成 1-15 行） |
| ## 与本地攻方不同的地方 | 抄 | 正文丁格问的正是小节名与攻方今天标题是否一致，这一节写了要读的小节名 |
| ## 没做什么（固定会有的） | 抄 | 全篇一并核 |

### 小节清单：`.claude/rules/three-way-inference.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 推论要三方独立论证 | 不抄 | 总纲背景，正文没有引到这一句 |
| （推论要三方独立论证 标题之下、第一个下级标题之前的正文：第 2-5 行） | 不抄 | 只说明是项目本地规则，正文没有引到这一句 |
| ## 适用范围 | 不抄 | 正文没有引到这一节 |
| ## 各条腿必须互不重复 | 抄 | 正文『分工』一节『两条攻方腿的攻击面不重叠』一句直接引这一节的判据：攻击面要在正文的分工表里分开写 |
| ## 引 kb 里的条目要整行抄，不许摘句——三处都管 | 不抄 | 本轮材料自己在遵守这一节，正文没有引它 |
| ## 判决由主 agent 做，不由投票做 | 不抄 | 正文没有引到这一节 |
| ## 多轮：一次打穿不算数，三轮里多数打穿才算 | 不抄 | 正文没有说这一轮是第几轮攻，不引用它的判据 |
| ### 交岔路时写岔路单，派实验时带上它 | 不抄 | 这一轮没有交岔路 |
| ## 一条腿只抽一次样不算一次观测——否定结论尤其不算 | 抄 | 主 agent 点名必抄 |
| ## 云端腿的报告要分段落盘 | 不抄 | 正文没有引到这一节 |
| ## 本地腿缺席时必须显式报告 | 抄 | 主 agent 点名必抄 |
| ## 给本地腿的提示一律用英文 | 抄 | 主 agent 点名必抄；正文第二节第 206 行那句引文也在这一节里 |

### 小节清单：`.claude/agent-common.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 项目 subagent 的共用约束 | 不抄 | 总纲背景，正文没有引到这一句 |
| （项目 标题之下、第一个下级标题之前的正文：第 2-5 行） | 不抄 | 总纲背景，正文没有引到这一句 |
| ## 派发 | 不抄 | 正文没有引到这一节 |
| ## 规则怎么读 | 不抄 | 正文没有引到这一节 |
| ## 写 | 不抄 | 正文没有引到这一节 |
| ## 不做 | 抄 | 主 agent 点名必抄的「长活可以等」与「执行前拒绝的写法」都是这一节里的列表项、不是独立标题，整节抄 |
| ## 门禁 | 不抄 | 正文没有引到这一节 |
| ## 报告 | 不抄 | 正文没有引到这一节 |

### 小节清单：`research/prompts/governance-defs-r1-main-verification.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 治理文档核对第 3 轮的十五份定义改动：第一轮判决（2026-09-26） | 不抄 | 背景用，只抄 G3 格判决段，总览不需要 |
| （治理文档核对第 标题之下、第一个下级标题之前的正文：第 2-6 行） | 不抄 | 背景用，只抄 G3 格判决段 |
| ## 一、这一轮交了什么 | 不抄 | 背景用，只抄 G3 格判决段，这节是腿与 sha256 清单，与 G3 无关 |
| ## 二、跑前条款，各触发没触发 | 不抄 | 只抄 G3 格判决段；这一节是整轮跑前条款的触发汇总，不是判决段本身 |
| ## 三、逐条判决（主 agent 逐条现查过） | 不抄 | 用 --extra 按行区间取：只抄这一节里 G3 格的那几行判决与表头 |
| ## 四、交用户的 | 不抄 | 背景用，只抄 G3 格判决段 |
| ## 回看决策 | 不抄 | 背景用，只抄 G3 格判决段 |

### 小节清单：`research/prompts/governance-defs-r2-main-verification.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 严查第一轮之后的十份定义改动：第一轮判决（2026-09-26） | 不抄 | 背景用，只抄 G3 格判决段，总览不需要 |
| （严查第一轮之后的十份定义改动：第一轮判决（2026-09-26） 标题之下、第一个下级标题之前的正文：第 2-6 行） | 不抄 | 背景用，只抄 G3 格判决段 |
| ## 一、这一轮交了什么 | 不抄 | 背景用，只抄 G3 格判决段，这节是腿与 sha256 清单，与 G3 无关 |
| ## 二、跑前条款，各触发没触发 | 不抄 | 只抄 G3 格判决段；这一节是整轮跑前条款的触发汇总，不是判决段本身 |
| ## 三、逐条判决（主 agent 逐条现查过） | 不抄 | 用 --extra 按行区间取：只抄这一节里 G3 格的那几行判决与表头 |
| ## 四、交用户的 | 不抄 | 背景用，只抄 G3 格判决段 |
| ## 回看决策 | 不抄 | 背景用，只抄 G3 格判决段 |

### 小节清单：`research/prompts/governance-defs-r3-main-verification.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 严查第二轮之后的七份定义改动：第一轮判决（2026-09-26） | 不抄 | 背景用，只抄 G3 格判决段，总览不需要 |
| （严查第二轮之后的七份定义改动：第一轮判决（2026-09-26） 标题之下、第一个下级标题之前的正文：第 2-6 行） | 不抄 | 背景用，只抄 G3 格判决段 |
| ## 一、这一轮交了什么 | 不抄 | 背景用，只抄 G3 格判决段，这节是腿与 sha256 清单，与 G3 无关 |
| ## 二、跑前条款，各触发没触发 | 不抄 | 只抄 G3 格判决段；这一节是整轮跑前条款的触发汇总，不是判决段本身 |
| ## 三、逐条判决（主 agent 逐条现查过） | 不抄 | 用 --extra 按行区间取：只抄这一节里 G3 格的那几行判决与表头 |
| ## 四、交用户的 | 不抄 | 背景用，只抄 G3 格判决段 |
| ## 回看决策 | 不抄 | 背景用，只抄 G3 格判决段 |

### 小节清单：`records/2026-09-27-同步远端冲突块.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 同步远端时的冲突块（2026-09-27） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节 |
| （同步远端时的冲突块（2026-09-27 标题之下、第一个下级标题之前的正文：第 2-4 行） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节 |
| ## `.claude/agent-common.md`（4 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 35 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 2 块（冲突时在第 52 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 3 块（冲突时在第 67 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 4 块（冲突时在第 100 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/agent-def-review-exempt`（1 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 6 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/agents/crash-verifier.md`（3 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 3 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 2 块（冲突时在第 31 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 3 块（冲突时在第 57 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/agents/experiment-designer.md`（1 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 39 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/agents/experiment-runner.md`（2 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 29 行起） | 抄 | 随（标题文字与 three-way-local-attack.md 那一块重复，quote-kb 的反向核对按文件不分父节，不改归属就会误判；靠 --extra records/2026-09-27-同步远端冲突块.md:385-385 免生成，不新增内容，仍是 experiment-runner.md 那一节，正文没有引它） |
| ### 第 2 块（冲突时在第 61 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/agents/gate-triage.md`（1 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 27 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/agents/implementation-writer.md`（1 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 32 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/agents/kb-scribe.md`（1 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 32 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/agents/mutation-triage.md`（1 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 19 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/agents/sweep.md`（1 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 24 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/agents/three-way-attack.md`（1 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 51 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/agents/three-way-local-attack.md`（2 块） | 抄 | 主 agent 点名必抄的两节之一 |
| ### 第 1 块（冲突时在第 29 行起） | 抄 | 随（父节用 @标题 带出） |
| ### 第 2 块（冲突时在第 43 行起） | 抄 | 随（父节用 @标题 带出） |
| ## `.claude/agents/three-way-local-defense.md`（1 块） | 抄 | 主 agent 点名必抄的两节之二 |
| ### 第 1 块（冲突时在第 13 行起） | 抄 | 随（父节用 @标题 带出） |
| ## `.claude/agents/three-way-verifier.md`（1 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 29 行起） | 抄 | 随（标题文字与 three-way-local-attack.md 那一块重复，quote-kb 的反向核对按文件不分父节，不改归属就会误判；靠 --extra records/2026-09-27-同步远端冲突块.md:385-385 免生成，不新增内容，仍是 three-way-verifier.md 那一节，正文没有引它） |
| ## `.claude/gate.d/10-kb-rot.sh`（2 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 2 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 2 块（冲突时在第 34 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/gate.d/fixtures/10-kb-rot.sh/red/expect`（1 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 5 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/gate.d/stage-owners.tsv`（1 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 37 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/hooks/bash-command-detector.sh`（2 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 2487 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 2 块（冲突时在第 2504 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/hooks/heavy-test-guard.sh`（3 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 147 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 2 块（冲突时在第 555 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 3 块（冲突时在第 760 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/hooks/runner-dispatch-guard.sh`（4 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 7 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 2 块（冲突时在第 123 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 3 块（冲突时在第 691 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 4 块（冲突时在第 867 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/main-agent.md`（4 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 14 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 2 块（冲突时在第 24 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 3 块（冲突时在第 38 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 4 块（冲突时在第 73 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `.claude/rules/implementation-workflow.md`（2 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 48 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 2 块（冲突时在第 93 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `CLAUDE.md`（1 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 109 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## `research/scripts/archive-past-rounds.py`（2 块） | 不抄 | 主 agent 只点名两份 agent 定义两节与「去向」一节，这一节不在射程内 |
| ### 第 1 块（冲突时在第 23 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ### 第 2 块（冲突时在第 288 行起） | 不抄 | 父节「不抄」，这一子节没有被 @标题 带出，独立判：主 agent 没点名这一节 |
| ## 去向（2026-09-27，用户弹窗定） | 抄 | 主 agent 点名必抄的文末一节 |


---

# 附：抽出的条款全文（quote-kb.py 整段抄，回读逐字节比对过）

**出处 `.claude/agents/three-way-local-attack.md:1-17`（整段抄，未转述）**

```markdown
---
name: three-way-local-attack
description: 三方论证的本地攻方腿：把攻方问题译成英文、驱动本机本地模型作答并过损坏闸。只在主 agent 点名派发、并给出分给本地攻方的攻击面时用；不要自动派发。
tools: Read, Bash
model: sonnet  # 只定驱动 ask-local.sh 的外壳；推论出自本机本地模型（经 ~/code/ai-center 网关 :8200，模型 id local），不是 sonnet
effort: high
omitClaudeMd: true
required-inputs: 禁读清单, 草稿目录, 提示文件
---

# 本地攻方腿（three-way-local-attack）

开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。

推论出自本机本地模型，你负责把问题忠实地交给它、把它的答复原样收回来。不替它补推理，也不判它答得对不对。
开工先读：`.claude/rules/three-way-inference.md`「各条腿必须互不重复」「一条腿只抽一次样不算一次观测——否定结论尤其不算」「本地腿缺席时必须显式报告」「给本地腿的提示一律用英文」。

```

**出处 `.claude/agents/three-way-local-attack.md:18-24`（整段抄，未转述）**

```markdown
## 输入（主 agent 必须给）

- 轮名、正文与附录路径、分给本地攻方的攻击面（与云端攻方腿不重叠）。
- 提示文件路径（形态 `research/prompts/<轮>-local-attack.md`）与样本文件名前缀。
- 草稿目录。
- 这一轮的禁读清单。

```

**出处 `.claude/agents/three-way-local-attack.md:25-34`（整段抄，未转述）**

```markdown
## 做什么

1. 把攻击面写成英文提示：自足、不用任何 markdown 强调、答案按编号、每条答复写「什么现象会推翻它」。能落成数的问题给写死的事实表（数整行抄，核对表里写来源文件:行），要模型按表格逐格填，不许只答 yes / no。提示里明令答复不写代码行号与文件行号（按函数名、表格行号指），核对表与运行记录里把样本自带的行号一律标「模型自给、未核」。
2. 逐句核转述：每一句英文与原文并排，缺一个限定词就补上；英文比原文多出来的限定词、括注也单列一行，写明为什么加。核对表写进 `research/prompts/<轮>-local-attack-translation-audit.md`（英文项 / 原文文件:行 / 首稿缺的 / 定稿）。
3. 取号：`ls <前缀>-output-s*.md` 看已用到几号，取下一个没用过的 `<n>`（重定向到一个已有样本的号会把它整份盖掉）。前台跑 `bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-s<n>.md`，不许 `setsid`、`&`、`disown`；单次请求最长 `ASK_LOCAL_TIMEOUT`（默认 900 秒）超过 Bash 前台上限时，改用 Bash 的 `run_in_background` 起同一条命令，起完结束本轮等完成通知，退出码取通知里的。
4. 退出码 5：作废副本由脚本留成 `-output-void<n>.md`，这一份不算；判红那次重定向建出的 `s<n>` 是空文件，先确认它大小为 0，再用 `>|` 重定向到同一个号重跑（`>|` 在开没开 `noclobber` 时都写得进）；退出码 0 的每次调用占一个号，带损坏的也占号、文件留着，运行记录里标带损坏。退出码非 0 非 5（包括 6：损坏检测器没跑成或找不到，这一份没验过）、或网关不通：停下，报「本地腿缺席」。
5. 闸判绿也要跑 `python3 research/scripts/oov-check.py <样本> <提示文件>`（带上提示文件，提示里的专名才不算生词）看它列出的生词，生词表只打前 300 个字符，打满了就不以它为全、逐段通读样本：见到缺头的粘连词（例：`achievesceives`）就把那份记成「带损坏」，不当干净样本；拿不准是新造词还是粘连的，一律记带损坏。另外通读一遍样本：缺词、断句这类损坏（例：一个列表里整个词没了，只剩孤零零的标点），同样记带损坏。
   5b. 提示里有事实表题的：主 agent 另给一份算好的答案表的路径（放在草稿目录之外、提示里不引，本地模型读不到）；收回样本后用一段 python 逐格比数，只打印对不上的格号、不打印答案，不一致的格在运行记录里标「算术对不上」；主 agent 没给答案表的，运行记录写「没有答案表，算术没比」。
6. 攒到至少两份干净样本为止；连续五次调用（判红作废的也算）拿不到两份干净的，停下照实报。

```

**出处 `.claude/agents/three-way-local-attack.md:35-38`（整段抄，未转述）**

```markdown
## 写范围

- 提示文件、核对表、样本文件（`<前缀>-output-s<n>.md`，作废副本由脚本自己写）、「产出」一节的运行记录（`research/prompts/<轮>-local-attack-runlog.md`）、草稿目录。除此之外不写。

```

**出处 `.claude/agents/three-way-local-attack.md:39-42`（整段抄，未转述）**

```markdown
## 产出

- 回复与一份短报告（写进 `research/prompts/<轮>-local-attack-runlog.md`）：每份样本的退出码、词数、生词清单、干净 / 带损坏 / 作废；核对表路径。不写样本答了什么、几份样本方向一不一致。

```

**出处 `.claude/agents/three-way-local-attack.md:43-46`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 不解读、不总结、不采纳本地模型的答复；判它打没打中是主 agent 的事。

```

**出处 `.claude/agents/three-way-local-defense.md:1-15`（整段抄，未转述）**

```markdown
---
name: three-way-local-defense
description: 三方论证的本地辩方腿：把辩方问题译成英文、驱动本机本地模型作答并过损坏闸。只在主 agent 点名派发、并给出要辩护的一方时用；不要自动派发。
tools: Read, Bash
model: sonnet  # 只定驱动 ask-local.sh 的外壳；推论出自本机本地模型（经 ~/code/ai-center 网关 :8200，模型 id local），不是 sonnet
effort: high
omitClaudeMd: true
required-inputs: 禁读清单, 草稿目录, 提示文件
---

# 本地辩方腿（three-way-local-defense）

开工先读 `.claude/agent-common.md`（这份定义开了 `omitClaudeMd`，要用的规则照共用约束「规则怎么读」一节读），再读 `.claude/agents/three-way-local-attack.md` 的「输入（主 agent 必须给）」「做什么」「写范围」「产出」四节：做法与它逐条相同，只有下面几处不同。
开工先读：`.claude/agents/three-way-local-attack.md` 里「开工先读：」那一行点名的小节。

```

**出处 `.claude/agents/three-way-local-defense.md:16-22`（整段抄，未转述）**

```markdown
## 与本地攻方不同的地方

- 立场是辩方：替被判出局、被攻、或被计划排除的一方，找它站得住的理由，写出具体机制（哪个文件、哪一步）；站不住就说站不住、为什么。辩护落得成数的（这一方在某段历史上是几、判红没有），同样按攻方第 1 步给事实表、要模型逐格填。提示里先替每一条写一个攻方问题（它被质疑的那一句，整行抄出处），再让本地模型逐条辩护；攻方问题同样要逐句核转述。
- 输入里的「攻击面」换成「要辩护的一方」（主 agent 给：被复核的判决或被排除的候选，原文路径）。
- 行号同攻方第 1 步：提示里明令答复不写行号，样本自带的行号在运行记录里标「模型自给、未核」。
- 文件名形态（攻方定义里出现的 `-local-attack` 文件名，不论在哪一节，一律照这里换）：提示 `research/prompts/<轮>-local-defense.md`，核对表 `research/prompts/<轮>-local-defense-translation-audit.md`，运行记录 `research/prompts/<轮>-local-defense-runlog.md`。样本照攻方写成 `<前缀>-output-s<n>.md`，前缀取派发提示给的（与攻方「输入」那一项同），不写死。

```

**出处 `.claude/agents/three-way-local-defense.md:23-27`（整段抄，未转述）**

```markdown
## 没做什么（固定会有的）

- 不解读、不总结、不采纳本地模型的答复。
- 样本不够两份就照实报。

```

**出处 `.claude/rules/three-way-inference.md:19-37`（整段抄，未转述）**

```markdown
## 各条腿必须互不重复

「三方」是这套流程沿用的名字；一轮派三条推论腿：云端攻方（Opus）、云端正推或云端辩方（Sonnet，一轮一条）、本地攻方或本地辩方（一轮一条，派哪一条由主 agent 按这一轮的攻击面与要复核的判决定，在正文分工表里写明理由）。

| 腿 | 用什么 | 怎么调 |
|---|---|---|
| 本地攻方 | Qwen3-Next-80B-A3B-Thinking-AWQ-4bit，经 `~/code/ai-center` 网关（`:8200`，模型 id `local`） | 派 `three-way-local-attack`，由它写提示、调 `bash research/scripts/ask-local.sh <提示文件>`，**提示用英文写** |
| 本地辩方 | 同一个本地模型，另一份提示 | 派 `three-way-local-defense`，同上 |
| 云端正推或云端辩方 | Sonnet | 派 `three-way-forward` 或 `three-way-defense`（定义里 `model: sonnet`） |
| 云端攻方 | Opus | 派 `three-way-attack`（定义里 `model: opus`） |

**「不能重复」指的是各条腿要拿到不同的切入角，不是只换个模型名。**
每条腿要被指定一个不同的立场（例如：正推 / 反推 / 找反例），
正推腿与攻方腿分担 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 三步里的正推与反推，第三步校验由主 agent 判决时逐条现查（「判决由主 agent 做，不由投票做」一节）。
这一轮派本地攻方时，它与云端攻方同为攻方，**两条攻方腿的攻击面要在正文的分工表里分开写**；写不出两组不重叠的攻击面，这一轮的正文就还没写完。
本地辩方替被判出局或被攻的一方辩护。本地腿按「一条腿只抽一次样不算一次观测」那一节抽样。

**本地腿只问能落成数、能逐格判的题。** 提示里给写死的事实表（每行在核对表里写来源文件与行号），要模型按表格逐格填，不许只答 yes / no。

```

**出处 `.claude/rules/three-way-inference.md:149-164`（整段抄，未转述）**

```markdown
## 一条腿只抽一次样不算一次观测——否定结论尤其不算

模型的答复是**有变化的观测**，因此 `.claude/singlefs-ai-sop/rules/test-discipline.md`
「单次观测不算数」那条对它成立：同一份提示、同一个模型，两次可以给出方向相反的答案。

⚠️ **两个方向的门槛不一样**，与那条规则同形：

| 结论 | 采信条件 |
|---|---|
| **打中了**（给出反例、指出矛盾） | 一次就值得去核。它是个线索，真伪由主 agent 现查坐实，抽样次数不改变这一步 |
| **没打中**（「构造不出反例」「没发现问题」） | **一次不算**。它与「这一轮它没想到」分不开，而两者在答复里长得一模一样 |

⇒ **做法**：拿一条腿的「没打中」去支撑任何结论之前，至少再抽一次；两次都没打中才记「没打中」，
两次不一致就照 `.claude/singlefs-ai-sop/rules/test-discipline.md` 记「不稳定」，不下结论。
云端腿同样适用——这条管的是「模型答复」这类观测，不是本机那条腿特有的：云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿，报告写成 `<轮>-<模型>-s2-output.md`（例 `-opus-s2-output.md`），模型目录写成 `<轮>-<模型>-s2-model/`，不盖掉第一条腿的。

```

**出处 `.claude/rules/three-way-inference.md:177-181`（整段抄，未转述）**

```markdown
## 本地腿缺席时必须显式报告

`ask-local.sh` 取不到 key、网关不通、或正文为空时，**一律报错退出，不许静默跳过**。
少了腿就说少了哪几条腿（网关不通时本地攻方、本地辩方一起缺），然后由人决定要不要在剩下的腿上继续。

```

**出处 `.claude/rules/three-way-inference.md:182-217`（整段抄，未转述）**

```markdown
## 给本地腿的提示一律用英文

本地那条腿是 4-bit 量化模型，**中文输出会退化性复读**（「有效的有效性」「恢复恢复」），
**英文不会**。数据与口径见 `.claude/kb/tooling.md`。

⇒ **提示用英文写，答案也让它用英文回。**

⚠️ **英文提示里的每一句转述，写完都要对着原文核一遍。** 本地腿读不到中文附录，条款只能译成英文转述，而转述正是「引 kb 里的条目要整行抄」那条纪律够不着的缺口：译的人觉得忠实，丢的往往是一个限定词。⇒ 写完英文提示，把每一句转述与附录里的原文并排放一遍，缺一个限定词就补上。**多出来的也要列**：英文比原文多一个限定词、多一个括注，同样在核对表里单列一行、写明为什么加。

**这条不靠自觉**：`ask-local.sh` 里有一道会拒绝的闸——输出判定为字词损坏时
**退出码 5**，并打印下一步。

⚠️ **损坏有四类，签名互不相同，分在两个检测器里**（`corruption-check.py` 查复读、成对标记落单、实词自复读，`oov-check.py` 查拼接）：复读（多吐了）、
成对标记落单（整段掉了，`**Attack P1**:it impossible.**`）、
拼接（两词粘死，`configurationing` `batchinggroup`；**后一个词还可以缺头**——
`inaccessibleisabled` 缺一个字母、`cryptographicord`（cryptographic + [Rec]ord）缺三个，
缺到尾巴不成词时前几条规则全切不开，靠「长前缀 + 既不成词也不是后缀的短尾巴」这条才抓得到；**粘上的还可以是这个词自己尾部的一截**（`reachableachable`，短到 6 个字母的 `anewew`），或者前缀是词表外的派生词（`observationalomputational`）；闸判绿之后照样通读，闸只认得登记过的形态）。第四类是**实词自复读**
（`resetting resetting`，同一个 ≥5 字母的词连着出现两次、中间隔一个空格）——
闸只查长实词（短虚词的自复读在正常英文里合法），不收窄就误报。
`ask-local.sh` 串跑 `corruption-check.py` 与 `oov-check.py`，任一判红即拒绝。
**只跑一个就有一到三类没人查。**

⚠️ **提示里不许用 markdown 强调。**

⚠️ **本地腿用 Bash 的 `run_in_background` 起、结束本轮等完成通知（单次请求最长 900 秒，超过前台上限），命令里不加 `setsid`、`&`、`disown`。**

**闸判红之后**：那一轮**作废重跑**，不许记成「三方不一致」——
否则每次都会不一致，这条规则就退化成了摆设。
⚠️ **这不是洁癖**：**损坏的不只是词。**
确有必要采用带损坏的输出时，设 `ASK_LOCAL_ALLOW_CORRUPT=1`，
并在结论里写明这一票带瑕疵。

⚠️ **闸报「没跑成」或「没做」时不是通过。** 检测器自己出错（退出码 2）或找不到，与判红（退出码 1）
是两件事，`ask-local.sh` 分开报：前者退出 6、不打正文、正文留成作废副本，判红退出 5。退出 6 就是**这一项没验**，
不许当成验过了。

```

**出处 `.claude/agent-common.md:41-79`（整段抄，未转述）**

```markdown
## 不做

- 不做 git 写操作（`add`、`commit`、`stash`、`checkout`、`restore`、`reset`、`worktree`……），不提交，不推送。门禁自带的 git 写只有 `gate-triage` 例外，见它的定义。
- 不建、不改 `.claude/agents/`、`.claude/settings*.json`、`.claude/hooks/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`，也不碰 `~/.claude/`。`tooling-writer` 例外：按它定义「写范围」一节改 `.claude/agents/`、`.claude/hooks/`、`.claude/rules/` 与 `.claude/settings.json` 的 `hooks` 一节；`.claude/singlefs-ai-sop/` 与 `~/.claude/` 它同样不碰。
- 不读派发提示给的禁读清单里的文件。
- 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
- 重型测试（名字含 `layer0` 的测试二进制与 54 号、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、87 号全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段（15、74 号虽然也跑 `cargo test`，按轻阶段对待；逐条的判法在 `.claude/hooks/heavy-test-guard.sh` 文件头）。`crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
- 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，名字含 `layer0` 的测试二进制也不许。
- 跑编译出来的代码经内存包装：`cargo test` / `run` / `bench`（带 `--no-run` 只编不跑的不算）与直接执行 cargo 编出来的二进制（实验装置、测试二进制），一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>` 跑；写进自己的脚本再跑的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取派发提示给的（定义另写了来源的照定义），都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`research/scripts/mutate.sh`、`research/scripts/replay.sh` 与门禁 59 号在里面逐条套了，门禁 15、74 号在阶段里面经包装（派发提示给了上限的，设进阶段文件头写的变量：15 号 `GATE_RESEARCH_BUILD_MEMORY_MAX`，74 号 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`），都直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。`cargo build`、`cargo clippy`、`cargo fmt` 不跑编出来的代码，不要求经它。
- 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
- 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
- 读大文件（超过 500 行）先 grep 定位、再按行段读（Read 的 offset / limit），不整份读；同一份文件读过一次、之后没改过，就不再整份重读。长命令的输出落进草稿目录的文件，只读汇总行（`tail -n 20`、`grep -c`、`grep ✗`），不把整份日志、整份 diff 读进上下文。上下文到 600k 就停在一个能交接的点交回（定义另写了线的照定义）：报告写做完的、做到一半的（文件与差哪一步）、没开的。
- 派发提示里有「线程上限：N」的，编译、测试、变异、产物、原型扫描的每条命令都用 `bash research/scripts/capped.sh N <命令>` 起（它把 cargo 与 `crates/singlefs-harness` 各装置读的线程变量一次设成 N）；实验装置自己的线程变量（`KS_THREADS` 这类）同样设成不超过 N。主 agent 发消息改了 N，从下一条命令起照新的。
- **崩溃点测试不衡量时间成本，也不为省时间缩范围**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，也不用先算它值不值。
- 禁止自行扩大任务范围，只改自己的部分。一个任务一个出口，达成出口即终止任务，扩大任务必须弹窗。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
- 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用，每件的退出码照「执行前拒绝的写法」那一条 ④ 的写法收。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。等自己起的后台任务一律结束本轮等完成通知，不写轮询它输出文件的循环（run_in_background 里也拒，见「执行前拒绝的写法」②）。
  等长活时不起缓存计时器，结束本轮直接等完成通知。
  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；前台没有超时的等待循环、run_in_background 里轮询本会话后台任务输出文件的循环、把活放出追踪、run_in_background 里后面没有 `wait` 的单独 `&` 在执行前被拒（「执行前拒绝的写法」那一条的 ②③④），run_in_background 里别的等待循环只记检出，交给主 agent 判断。跑超过 30 分钟的量，每完成一格往草稿目录的 `progress.md` 追加一行（格名、耗时、下一格预计多久），看门狗的整点询问会附上它的末几行。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
- 执行前拒绝的写法：项目 settings 里注册的这几道 hook 在执行前（收工闸在收工前）拒绝这几类写法（退出 2，stderr 写原因与出路），主 agent 与子 agent 一样拒，只挂在一方才用的工具上的在条目里写明；拒的只是这一次调用，不停已经在跑的东西。被拒了照出路改写法，不换写法绕过去；改不成的写进报告交主 agent。
  - Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）：② 只在前台拒（run_in_background 里的等待循环只记检出、不拒），④ 只在 run_in_background 里拒，其余前台与 run_in_background 一样拒：
    ① 起看门狗的错误写法：`research/scripts/watch.sh`、`research/scripts/agent-watch.py watch` 不用 run_in_background 起，或命令里带单独的 `&`、`nohup`、`disown`、`setsid`、把输出丢进 `/dev/null`。看门狗只有主 agent 起。
    ② 前台没有超时的等待循环：`until` / `while` 里有 `sleep`，外面没套 `timeout`；run_in_background 里条件或循环体点名了本会话后台任务输出文件（`tasks/<id>.output`）的等待循环，有没有 timeout 都拒。要等就用 run_in_background 起、结束本轮等完成通知，或 `python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。
    ③ 把活放出追踪：`disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux` / `screen` 的分离模式、不等到结束的 `systemd-run`。
    ④ run_in_background 里有作业以单独的 `&` 收尾，之后同一条命令里没有 `wait`。要并行就每件把退出码写进自己的文件（`{ <命令>; echo "$?" > <文件>; } &`，文件名带批号与件号，每批开跑前先删掉这一批的文件），最后单独一个不带参数的 `wait` 等齐，再按派活的次序逐个读那几个文件；读之前数一遍，文件数与派出去的件数对不上，整批作废；不用 `wait "$pid"` 收。
    ⑤ 整份覆盖 `research/results/` 下已存在又没进 git 的产物：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 的目标、`dd of=`、`truncate`。追加（`>>`、`tee -a`）与新文件名不拦；要换就按日期另存新文件名，旧的留着。
    ⑥ 终止进程不是点名一个自己起的进程号或任务号：`kill` 的目标带负号、是 0 或 `$PPID`、一次给几个、是命令替换或通配，在循环里逐个发（`proc.py stop` 同样），按名字、按 cgroup 或 `/proc` 挑进程，写死的进程号指到自己会话的祖先、别的会话起的进程或 SSH / VSCode / 本地模型服务，`systemctl` 停 SSH、登录会话、本地模型服务这类单元，关机重启。放行的是点名一个：`kill "$!"`、`kill %1`、单独一条 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`。
    ⑧ 有 Edit 工具的项目子 agent 在 Bash 里就地改仓内文件：命令位置上的 `sed -i`（目标不在 `/tmp/` 下），python 代码里 `open(<仓内路径>, 'w')` 这一类、`Path(<仓内路径>).write_text(`。仓内文件用 Edit 改，草稿目录照写。
    ⑦ 在同一个 inode 上改已经存在的脚本（`.sh`、`.py` 或带执行位的文件，在仓里或 `/tmp/claude-1000/` 下）：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` 的目标、`dd of=`、`truncate`、python 的 `open(…, 'w')` 这一类。改脚本写到同目录临时文件再 `mv` 换上，或用 `research/scripts/replace-once.py` / `insert-row.py` 定点改；追加与新建不拦。
  - 上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh`：命令位置上的 `pgrep -f`、`pkill -f`、`killall`。
  - 重型测试闸（`.claude/hooks/heavy-test-guard.sh`）：越出重型测试那一条的命令（主 agent 不带 `SINGLEFS_HEAVY_TESTS` 前缀也拒）；子 agent 不经内存包装跑编译出来的代码（「跑编译出来的代码经内存包装」那一条）。
  - 写闸（`.claude/hooks/write-guard.sh`，只看 Write / Edit 工具）：Write 整份覆盖仓里已存在又没进 git 的文件；项目子 agent 写到写范围表（`.claude/hooks/agent-write-scope.tsv`）里自己那几行之外；写进的内容里有撇号类角标（字符与起名的写法见 `.claude/rules/path-moves.md`「变体起新名字，不用角标」）。
  - 交回闸（上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/handback-scratch-check.sh`，挂交回工具 SubagentHandback 与 SubagentStop，只判子 agent）：临时目录里自己建的编译目录、工作树与仓副本还在，交回报告里又没逐个写全路径与为什么不删（`.claude/singlefs-ai-sop/rules/session-wrapup.md`「5. 子 agent 交回之前，删掉自己建的编译目录与仓副本」）。
  - 收工闸（上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/gate-reuse-check.sh`，挂 Stop 与 SubagentStop）：这个会话新建或改过的门禁与钩子没写 `# gate-similar:` / `# hook-events:`、该点名的已有门禁与钩子没点全、或整段抄了已有的一份（`.claude/singlefs-ai-sop/rules/sop-first.md`「加门禁或钩子之前，先找已有的」）。
  - 弹窗闸（`.claude/hooks/ask-user-claim-guard.sh`，挂 AskUserQuestion，主 agent 用）：问句或选项说明里一句话带断言词（不可能、造不出、从来不、从来没有、一定、必然、永远不、绝不会、恒为），同一句里没有出处（反引号里的路径或命令、文件:行号、`research/results/` 下的文件、`name=` 开头的产物行、「实测」「量过」「产物」「输出」旁边带数或路径），也没写「推的」「没量过」「推测」「估计」「粗估」之一。
  - 派发闸（`.claude/hooks/runner-dispatch-guard.sh`，挂 Agent / Task，主 agent 用）：派 crash-verifier、gate-triage 之外的类型，提示里没有一行「重型测试：不跑」；不读共用约束的类型没写「开工先读 `.claude/agent-common.md`」；缺定义 frontmatter `required-inputs:` 要的输入；提示要子 agent 写进它写范围之外的路径；派 `implementation-writer` 没写「要动的 crates 文件：…」、超过 8 个、或与在跑的实现员撞文件；在跑的 opus 子 agent 满 8 个、或同一族还在限额窗口里；派 `kb-scribe` 的规格过不了 `research/scripts/kb-spec-check.py`；派 `experiment-designer` 写准入判输入没变的重跑登记；派 `mutation-triage` 没给变异表；派 `experiment-runner` 没写「这一段回答的岔路：…」（只修锚点、只补落产物除外），续做没写「上一段岔路表里还差：…」或写了一行都不差；派 `kb-scribe`、`implementation-writer` 要改的文件落在还没写判决的三方轮的开工快照里。
  - 续做闸（`.claude/hooks/continuation-guard.sh`，挂 SendMessage，主 agent 用）：给已经交回过（它自己的会话记录，或主会话记录里的交回消息）、或最近一次任务通知是 failed / killed 的子 agent 发消息。
  - 交回正文闸（`.claude/hooks/handback-guard.sh`，挂交回工具 SubagentHandback，只判项目子 agent）：交回正文超过 2000 字；三方腿与核查员交回里点名的报告过不了 `research/scripts/cite-check.py`；书记员与执行员的交回或报告里有没报数、或报 0 项的绿行。

```

**出处 `records/2026-09-27-同步远端冲突块.md:385-419`（整段抄，未转述）**

````markdown
## `.claude/agents/three-way-local-attack.md`（2 块）

### 第 1 块（冲突时在第 29 行起）

远端：

```text
3. 取号：`ls <前缀>-output-s*.md` 看已用到几号，取下一个没用过的 `<n>`（重定向到一个已有样本的号会把它整份盖掉）。用 Bash 的 `run_in_background` 起 `bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-s<n>.md`（单次请求最长 `ASK_LOCAL_TIMEOUT` 默认 900 秒，超过前台上限），起完结束本轮等完成通知，退出码取通知里的；命令里不加 `setsid`、`&`、`disown`。
4. 退出码 5：作废副本由脚本留成 `-output-void<n>.md`，这一份不算；判红那次重定向建出的 `s<n>` 是空文件，先确认它大小为 0，再用 `>|` 重定向到同一个号重跑（`>|` 在开没开 `noclobber` 时都写得进）；退出码 0 的每次调用占一个号，带损坏的也占号、文件留着，运行记录里标带损坏。退出码非 0 非 5（包括 6：损坏检测器没跑成或找不到，这一份没验过）、或网关不通：停下，报「本地腿缺席」。
5. 闸判绿也要跑 `python3 research/scripts/oov-check.py <样本> <提示文件>`（带上提示文件，提示里的专名才不算生词）看它列出的生词，生词表只打前 300 个字符，打满了就不以它为全、逐段通读样本：见到缺头的粘连词（例：`achievesceives`）就把那份记成「带损坏」，不当干净样本；拿不准是新造词还是粘连的，一律记带损坏。另外通读一遍样本：缺词、断句这类损坏（例：一个列表里整个词没了，只剩孤零零的标点），同样记带损坏。
```

本地（留下的）：

```text
3. 前台跑 `bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-s<n>.md`，不许 `setsid`、`&`、`disown`。
4. 退出码 5：作废副本由脚本留成 `-output-void<n>.md`，这一份不算；判红那次重定向建出的 `s<n>` 是空文件，沿用同一个号重跑；退出码 0 的每次调用占一个号，带损坏的也占号、文件留着，运行记录里标带损坏。退出码非 0 非 5、或网关不通：停下，报「本地腿缺席」。
5. 闸判绿也要跑 `python3 research/scripts/oov-check.py <样本>` 看它列出的生词：见到缺头的粘连词（例：`achievesceives`）就把那份记成「带损坏」，不当干净样本；拿不准是新造词还是粘连的，一律记带损坏。另外通读一遍样本：缺词、断句这类损坏（例：一个列表里整个词没了，只剩孤零零的标点），同样记带损坏。
   5b. 提示里有事实表题的：主 agent 另给一份算好的答案表的路径（放在草稿目录之外、提示里不引，本地模型读不到）；收回样本后用一段 python 逐格比数，只打印对不上的格号、不打印答案，不一致的格在运行记录里标「算术对不上」；主 agent 没给答案表的，运行记录写「没有答案表，算术没比」。
```

### 第 2 块（冲突时在第 43 行起）

远端：

```text
- 提示文件、核对表、样本文件（`<前缀>-output-s<n>.md`，作废副本由脚本自己写）、运行记录（`research/prompts/<轮>-local-attack-runlog.md`）。除此之外不写。
```

本地（留下的）：

```text
- 提示文件、核对表、样本文件（`<前缀>-output-s<n>.md`，作废副本由脚本自己写）、「产出」一节的运行记录（`research/prompts/<轮>-local-attack-runlog.md`）、草稿目录。除此之外不写。
```

````

**出处 `records/2026-09-27-同步远端冲突块.md:420-435`（整段抄，未转述）**

````markdown
## `.claude/agents/three-way-local-defense.md`（1 块）

### 第 1 块（冲突时在第 13 行起）

远端：

```text
开工先读 `.claude/agent-common.md`（这份定义开了 `omitClaudeMd`，要用的规则照共用约束「规则怎么读」一节读），再读 `.claude/agents/three-way-local-attack.md` 的「输入（主 agent 必须给）」「做什么」「写范围」三节：做法与它逐条相同，只有下面几处不同。
```

本地（留下的）：

```text
开工先读 `.claude/agent-common.md`（这份定义开了 `omitClaudeMd`，要用的规则照共用约束「规则怎么读」一节读），再读 `.claude/agents/three-way-local-attack.md` 的「输入」「做什么」「写范围」「产出」四节：做法与它逐条相同，只有下面几处不同。
```

````

**出处 `records/2026-09-27-同步远端冲突块.md:1277-1314`（整段抄，未转述）**

```markdown
## 去向（2026-09-27，用户弹窗定）

用户四问的原选：甲「保留本地」；乙-1「补回并按现值重算」；乙-2 全选四组并注「以本地为准 参考远程」；丙「保留本地内容」。逐块照办如下，远端内容只在不与本地冲突、本地又没有的地方补进来。

| 文件 | 块 | 去向 |
|---|---|---|
| `.claude/agent-common.md` | 1 | 补远端：只有 Edit 没有 Write 的定义照排他写新建；「没有 Write 的」一条罩到另带 WebFetch、WebSearch 的。换 inode 那半句本地「执行前拒绝的写法」⑦ 已罩住，不补 |
| `.claude/agent-common.md` | 2 | 本地（崩溃验证员跑 54）；补远端「15、74 号按轻阶段对待」与指到 `heavy-test-guard.sh` 文件头。远端 254 重跑一次那句与本地「不绕开包装重跑」冲突，不补 |
| `.claude/agent-common.md` | 3 | 本地 |
| `.claude/agent-common.md` | 4 | 补远端：重型那几道只在提交时或用户要求时、带前缀 |
| `.claude/agent-def-review-exempt` | 1 | 远端 18 行补回，理由原样加一句来历，哈希按补回之后的定义重算 |
| `.claude/agents/crash-verifier.md` | 1、3 | 本地 |
| `.claude/agents/crash-verifier.md` | 2 | 本地；补远端 55、57、59 开跑前核工作区与暂存区在这一道输入上相同；E142 带简称 |
| `.claude/agents/experiment-designer.md` | 1 | 本地表格；补「第一行英文名」 |
| `.claude/agents/experiment-runner.md` | 1 | 本地；补英文名命名、入库装置不跑 `mutate.sh`、入库装置编在仓根 `target/`、`driver_e<号>` 登记、7b ② 的括注 |
| `.claude/agents/experiment-runner.md` | 2 | 本地；补变异三个数的数法与 `mutate.sh` 的撞顶、超时 |
| `.claude/agents/gate-triage.md` | 1 | 本地；命令换成远端的 `research/scripts/gate-staged.sh`（整轮全绿才前移 `refs/sop/staged-green`）；补「碰了 54 号脚本本身也算这一轮」 |
| `.claude/agents/implementation-writer.md` | 1 | 本地（54 归崩溃验证员）；补证红经内存包装、clippy 那几条 `-D` 照 `check.sh` 现抄 |
| `.claude/agents/kb-scribe.md` | 1 | 补远端：`crates/**/*.rs` 逐个列给主 agent；写入后钩子那一段 |
| `.claude/agents/mutation-triage.md` | 1 | 本地；补没登记 `[[bin]]` 的 bin 名取法 |
| `.claude/agents/sweep.md` | 1 | 本地；补 ASCII 连字符 |
| `.claude/agents/three-way-attack.md` | 1 | 本地；补副本与自己的模型可以编译跑 |
| `.claude/agents/three-way-local-attack.md` | 1 | 本地（前台跑）；补取号、超过前台上限时改用 `run_in_background`、`>|` 重跑、退出码 6、oov-check 带提示文件与 300 字 |
| `.claude/agents/three-way-local-attack.md` | 2 | 本地 |
| `.claude/agents/three-way-local-defense.md` | 1 | 本地；小节名照攻方定义改成「输入（主 agent 必须给）」 |
| `.claude/agents/three-way-verifier.md` | 1 | 本地（`cite-check.py` 与对快照核），远端逐文件 sha256 那一套不补 |
| `.claude/gate.d/10-kb-rot.sh` | 1 | 本地加补远端 gate-stage 第 4 段与 gate-similar 行 |
| `.claude/gate.d/10-kb-rot.sh` | 2 | 已在前一步修（GATE_DIRECTORY） |
| `.claude/gate.d/fixtures/10-kb-rot.sh/red/expect` | 1 | 两边合起来，远端 5 条 want 补回；样本判对 |
| `.claude/gate.d/stage-owners.tsv` | 1 | 本地 |
| `.claude/hooks/bash-command-detector.sh` | 1、2 | 本地 |
| `.claude/hooks/heavy-test-guard.sh` | 1–3 | 本地；自检已过 |
| `.claude/hooks/runner-dispatch-guard.sh` | 1–4 | 本地。远端逐句判重型的解析器与它的八十来格自检不补：本地换成「重型测试：不跑」一行的判法，远端的岔路、快照冲突两条本地 ⑥ ⑪ 已有 |
| `.claude/main-agent.md` | 1–4 | 本地；另把第 57、58 行残留的「层 0 全量主 agent 跑」改成崩溃验证员跑，第 61 行分诊员命令换 `gate-staged.sh` |
| `.claude/rules/implementation-workflow.md` | 1、2 | 本地；另把表里没冲突带进来的两句远端口径（层 0 全量由主 agent 跑）改成崩溃验证员跑 |
| `CLAUDE.md` | 1 | 本地 |
| `research/scripts/archive-past-rounds.py` | 1、2 | 补远端：点名 `link-targets.py` |

```

**出处 `research/prompts/governance-defs-r1-main-verification.md:28-29`（整段抄，未转述）**

```markdown
| 编号 | 格 | 打中了什么 | 现查 | 判决与改法 |
|---|---|---|---|---|
```

**出处 `research/prompts/governance-defs-r1-main-verification.md:34-35`（整段抄，未转述）**

```markdown
| A5 | G3 | 样本号已被占时 `>` 会整份盖掉干净样本 | 属实（改前也中） | 改：第 3 步先 `ls` 取下一个没用过的号 |
| A6 | G3 | 「共用约束开着 noclobber」与实测不符 | 属实 | 改：删掉这句理由，写「`>|` 在开没开 noclobber 时都写得进」 |
```

**出处 `research/prompts/governance-defs-r2-main-verification.md:28-29`（整段抄，未转述）**

```markdown
| 编号 | 格 | 打中了什么 | 现查 | 判决与改法 |
|---|---|---|---|---|
```

**出处 `research/prompts/governance-defs-r2-main-verification.md:30-30`（整段抄，未转述）**

```markdown
| B1 / 正推 G3 | G3 | 59 号「计数：」行没有 💥、⚠️ 两栏；进程没跑完的落「无效」或「没红」，都判红；改后文字说「那一行各栏都有」「💥 不判失败」对 59 号不成立 | `59-crates-mutation-replay.sh` 的 `judge_one` 属实：编译错误记 invalid，否则记 failure（「没跑到」） | 站不住。mutation-sampling.md 与 mutation-triage.md 写明 59 号没有这两栏、两种落法都判红；「💥 不判失败」限定为 `mutate.sh` |
```

**出处 `research/prompts/governance-defs-r3-main-verification.md:28-29`（整段抄，未转述）**

```markdown
| 编号 | 格 | 打中了什么 | 现查 | 判决与改法 |
|---|---|---|---|---|
```

**出处 `research/prompts/governance-defs-r3-main-verification.md:34-35`（整段抄，未转述）**

```markdown
| D4 | G3 | 被 git 忽略的文件 `git log` / `git status` 看不见改动，核查员会记「没改过」 | 属实（`.claude/singlefs-ai-sop/` 整个被忽略） | 站不住。这类文件对不上记「分不清」 |
| D5 / 正推 G3 | G3 | 只有代码轮一节要求记派腿时刻；设计轮没时刻时判别力自证做不出 ✗；自证挑选少了「`sha256sum -c` 对得上」 | 属实 | 站不住。main-agent「要推论」一行要求给核查员派腿时刻与快照；自证改成只拿副本那一行比，不走「分不清」那几支，挑选条件补上 |
```

**出处 `records/2026-09-27-同步远端冲突块.md:385-385`（整段抄，未转述）**

```markdown
## `.claude/agents/three-way-local-attack.md`（2 块）
```
