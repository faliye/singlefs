# shell-words-r2：共用切词修补的第二轮——判决（2026-10-26）

<!-- doc-lint:not-numbers V1 V2 V3 W1 W2 W3 W4 W5 -->

## 一、这一轮的材料与证据

| 项 | 路径 | sha256 前 8 位 |
|---|---|---|
| 正文 | `research/prompts/_shell-words-r2-body.md` | a46abf61 |
| 正推腿甲（Sonnet，V1、V2） | `research/prompts/shell-words-r2-sonnet-output.md`（731 行），模型目录 `shell-words-r2-sonnet-model/` | 8f8b88c9 |
| 正推腿乙（Sonnet，V3） | `research/prompts/shell-words-r2-sonnet-s2-output.md`，模型目录 `shell-words-r2-sonnet-s2-model/` | cc8710b0 |
| 被判文件快照 | `research/prompts/shell-words-r2-snapshot/files-sha256.txt` | — |

- 这一轮照用户 2026-10-26 定案派两条正推式腿、不派 Opus 攻方（第一轮判决第二节、第六节）。
- 写判决前，快照清单五份 `sha256sum -c` 全部 OK。
- `python3 scripts/agents/snapshot-tree.py shell-words-r2 --out ~/.local/share/singlefs/work/shell-words-r1-verify/shell-words-r2-tree`：五份倒推出、sha256 对上。
- `python3 scripts/agents/cite-check.py research/prompts/shell-words-r2-sonnet-output.md research/prompts/shell-words-r2-sonnet-s2-output.md --root ~/.local/share/singlefs/work/shell-words-r1-verify/shell-words-r2-tree --background research/prompts/_shell-words-r2-body.md`：「核了 172 处引文、19 处裸行号，对上 152 处，没判 39 处」。没判的里 19 处是 `.claude/agent-common.md` 与 `.claude/main-agent.md` 的行（快照树里没有这两份），抽核 `.claude/agent-common.md:10`、`:69` 两处与此刻原文对上；其余 20 处是腿原样输出里的沙盒路径，不是引文。

## 二、逐条判决

| # | 格 | 腿报的 | 主 agent 现查 | 判 |
|---|---|---|---|---|
| 1 | V1 第一轮五格修得对不对 | 21 条关键反例修补后 21/21 判对；575 条迁移数与修补那一件报的逐数相同 | 第一轮那三条反例在此刻钩子上都 rc=2（第一轮判决之后主 agent 现喂） | 修得对，采 |
| 2 | V2 函数名拼写拆开的调用 | W5 新放行「只定义不调用的 function」把 `function foo { cargo test --all; }; "fo"o` 这类拆开拼写的调用一起放了，110 条 | 草稿树现喂主 agent 不带前缀：第一轮被判那一版 rc=2、此刻 rc=0 | 打中（修补带进的回归），采 |
| 3 | V2 数字紧贴 `>` 而不是 fd | `cat s.sh 01> d.sh`、`"2">`、`3>d.sh >&3` 这类漏拒，15 条 | `cat s.sh 01> d.sh; bash d.sh` 第一轮被判那一版与此刻都 rc=0 | 采；不是这一轮修补带进的，一起补 |
| 4 | V2 `bash -c` 后长选项与 `-n` | bash 在 `-c` 之后只认短选项，`bash -c --posix '…'` 会报错不起，修补后误拒 162 条，另 18 条 `-n` | 没现喂 | 采（偏严，交修补） |
| 5 | V2 `env =1` 后再接选项 | 6 条误拒 | 没现喂 | 采（偏严，交修补） |
| 6 | V1 残留 | 带步长、两端宽度不一的补零序列（`{01..100..3}`）10 条；ANSI-C 引号 `$'..\'..'` 不认；heredoc 写文件的消费方（`heavy-test-guard.sh` 第 603 行附近）不看 fd 号，8 条误拒——三样修补前后一样 | 没现喂 | 采，一起补 |
| 7 | V3 case 模式词 | case 里 `;;` 之后的模式词（`herd7)`、`killall)` 等）被当命令词，重型闸与检出钩子都误拒，27 行 | `case "$x" in a) echo a;; herd7) echo h;; esac` 改前、第一轮被判那一版、此刻三版都 rc=2 | 采；早就有的偏严误拒，一起补 |
| 8 | V3 只定义不调用的 function 在主工作区被编译目录那一道拒 | 重型一类跳过了函数体，编译目录那一道没跳，2 行 | 没现喂 | 采（与第 2 条同一处，交修补时一起改：只跳没有被任何写法调到的函数体） |
| 9 | V3 ⑫ 花括号目标 | 文件头写「花括号展开算不出，一律拒」，实现是展开后逐个落点判 | 没现喂 | 采：改文件头照实现写 |
| 10 | V3 ⑨ 相对路径起点规则没说 | 实测不看 hook 输入的 cwd | 没现喂 | 采：照 ⑧ ⑫ 改成从 hook 输入的 cwd 起算，文件头同写 |
| 11 | 两条腿列的「修补前后一样错」的其余几类 | `>>`、`&>`、`>&`、`tee` 拷出不认、`bash -s < 文件`、`xargs env`、名单外启动器 | 没现喂 | 不在这一批射程，进下一轮清单 |

## 三、改法

派工具实现员一次改全第 2–10 条；第 2 条是这一轮修补带进的漏拒，先改先换上。验收：两条腿各自的复跑装置（`shell-words-r2-sonnet-model/tools/rerun.sh`、`shell-words-r2-sonnet-s2-model/tools/rerun.sh`）在改后的钩子上重跑，点名的反例都判对，其余条目与这一轮读数相同。

## 四、下一轮

改完开第三轮（最终轮），只攻这一轮改法站住的形态；派法照第一轮用户定案的两条正推式腿。

## 五、交用户

| 项 | 状态 |
|---|---|
| 第 11 条那几类 | 进下一轮清单，零轮 |

## 这一轮被判过的文件

- `.claude/hooks/lib_shell_words.py`：V2 打中（函数调用、fd 数字、`bash -c` 选项、`env =1`）、V3 打中（case 模式词）
- `.claude/hooks/heavy-test-guard.sh`：V1 残留（heredoc 写文件不看 fd）、V3 打中（case 模式词）
- `.claude/hooks/bash-command-detector.sh`：V3 打中（case 模式词、⑨ 起点、⑫ 文件头）
- `.claude/hooks/lib_heavy_tests.py`：V3 打中（编译目录那一道跳函数体）

## 回看决策

不涉及决策：被判的是项目的协作钩子，不碰 `.claude/kb/decisions/` 下任何一条。
