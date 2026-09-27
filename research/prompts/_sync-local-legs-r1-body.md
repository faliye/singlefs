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
