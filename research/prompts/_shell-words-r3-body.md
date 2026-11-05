# shell-words-r3：共用切词修补的第三轮（最终轮，2026-10-28）

## 一、题面

第二轮判决 `research/prompts/shell-words-r2-main-verification.md` 采了第 2–10 条，修补已换上（报告 `~/.local/share/singlefs/work/debt-tw-shell-words-fix2/result.md`；只含这一批的 diff 拷在冻结副本 `fix2.diff`，前两批的 diff 是 `r1.diff`、`fix.diff`）。这一轮是最终轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」的第三轮分工攻三件事：前两轮判定站住的形态撑不撑得住、留到下一轮的清单全不全、修补那一件报出的数能不能核。只攻前两轮站住的形态；这一轮里新冒出来的形态不再开轮，写进报告的「新形态」一节。派法同第二轮：两条正推式腿，不派 Opus 攻方（用户 2026-10-26 弹窗定）。

被判的文件（快照 `research/prompts/shell-words-r3-snapshot/files-sha256.txt`）：`.claude/hooks/lib_shell_words.py`、`.claude/hooks/bash-command-detector.sh`、`.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/lib_workspace_cargo_lock.py`。

## 二、实现今天的样子（主 agent 的观测，2026-10-28）

- `crates/` 里没有相关实现：`grep -rln 'shell_tokens\|heredoc_feeds_shell' crates/` 零命中。被判的是项目的协作钩子；方案按钩子今天的代码来谈。
- 五份钩子此刻与 HEAD（提交 f4605214d 起）逐字节相同，与冻结副本逐字节相同（主 agent `cmp` 过）。
- 主 agent 拿修补那一件的喂法在主工作区现喂：重型闸 `swap/feeds-i-guard.json` 144 条、检出钩子 `swap/feeds-i-det.json` 40 条，「0 条不对」（`hook-swap.py --feed-only`）。
- 修补那一件报的读数（主 agent 没复跑，是线索）：在报告「三、三份复跑的读数」一节：正推腿甲各族判定迁移表与逐条变了的表；正推腿乙 1943 条里不同 37 条；第一轮 575 条里「改前拒、改后放」13 条（第一轮核过的 12 条加 `env FOO=1 -- cargo test --all`）。
- 前两轮留到下一轮的清单（这一轮不攻其中任何一类会被拒或被放，只核清单全不全）：第二轮判决第 11 条（`>>`、`&>`、`>&`、`tee` 拷出不认、`bash -s < 文件`、`xargs env`、名单外启动器）；修补报告列的新偏严三类（cat 不看源文件在不在 32 条、检出钩子重定向目标花括号展开成几个词 4 条、函数体之外引号里的花括号被当认不准的调用 1 条）；正推腿乙 C12c 五格（仓副本里经脚本自己的函数 `run_logged` 起 checker 档，属名单外启动器）。
- 第二轮两条腿的复跑装置在 `research/prompts/shell-words-r2-sonnet-model/`、`research/prompts/shell-words-r2-sonnet-s2-model/`，第一轮的在 `research/prompts/shell-words-r1-sonnet-model/`。已知缺陷：判法 `oracle.py` 对样本起真 `bash -c` 时只杀那一层 bash，样本里 `cat >(…)` 一类起的孙进程留下（修补那一件一趟留了 69 个，报告「五、复跑留下的进程」）。这一轮的腿不复用那份判法起真 bash，自己写的判法把每条样本放进它自己的进程组（`setsid`）并在超时与结束时杀整组，跑完核没有留下进程。

## 三、格

### X1　前两轮站住的判定撑不撑得住
第一轮五格（W1–W5）与第二轮第 2–10 条，每一条的判法在此刻五份钩子上是不是照 bash 的真实行为（展开次序、重定向、`env` 与 `bash` 的参数解析、case 与 `[[ ]]` 的词法、函数定义与调用）判；造前两轮反例之外、同一判法覆盖到的写法（换引号、换分隔符、换嵌套层、换 cwd、换起法）核它们照判。

### X2　留到下一轮的清单全不全
不看 diff 与前两轮反例，只读五份钩子文件头写明的判据（什么拒、什么放）与 `.claude/main-agent.md`、`.claude/agent-common.md` 里照抄的起法，造正当命令与各类该拒写法，在此刻钩子上喂；判错的逐条归类：落进「二、实现今天的样子」列的哪一类，还是不在任何一类里。不在任何一类里的才是这一格要交的。

### X3　修补那一件报的数能不能核
修补报告「三、三份复跑的读数」与「四、用到共用切词的其余钩子与脚本的自证」里的每个数，用独立的判法重算一遍（不复用报告里的 `famcmp.py`、`judge.py` 与腿的 `oracle.py`），逐数对；对不上的写清是哪个数、两边各怎么算。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 正推腿甲（Sonnet） | X1、X3 | 照第二轮正推腿甲的做法：真 bash 与 GNU env 逐词对照；X3 自己写判法、自己起真 bash，逐数核修补报告 |
| 正推腿乙（Sonnet） | X2 | 只从钩子文件头写明的判据与两份定义里照抄的起法出发，造正当命令与各类该拒写法的常见形态；不读 diff、不读前两轮的腿报告与反例 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号；冻结副本路径在派发提示里给。
- 腿喂钩子在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里拷一份钩子喂，不在冻结副本或主工作区里改；喂法与原样输出写进报告。
- 起真 bash 的判法把每条样本放进自己的进程组，超时与结束都杀整组；交回前 `ps` 核自己起的进程一个不剩，留下了写清是哪条样本。
- 不跑重型测试、不起 cargo、不碰 GPU；自证与探针经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 4G 的上限，排不上可以带 `ulimit -v` 与 `nice` 直接跑并写明。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
