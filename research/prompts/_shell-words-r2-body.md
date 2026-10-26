# shell-words-r2：共用切词修补的第二轮（2026-10-26）

## 一、题面

第一轮判决 `research/prompts/shell-words-r1-main-verification.md` 采了正推腿打中的五处（W1 花括号空份、W2 双引号里的 `$(` 收不了尾、W3 `2>` 与 `ln -s` 相对源、W4 `env =1`、W5 `bash -c --` 与三处偏严），修补已换上（报告 `~/.local/share/singlefs/work/debt-tw-shell-words-fix/result.md`；只含修补的 diff 拷在冻结副本 `fix.diff`）。这一轮攻修补本身：修得对不对，修补有没有带进新的错判。用户 2026-10-26 定这一轮派两条正推式腿、不派 Opus 攻方（第一轮两条攻方腿都被安全分类器拦下，判决第二节）。

被判的文件（快照 `research/prompts/shell-words-r2-snapshot/files-sha256.txt`）：`.claude/hooks/lib_shell_words.py`、`.claude/hooks/bash-command-detector.sh`、`.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/lib_heavy_tests.py`。

## 二、实现今天的样子（主 agent 的观测，2026-10-26）

- `crates/` 里没有相关实现：`grep -rln 'shell_tokens\|heredoc_feeds_shell' crates/` 零命中。被判的是项目的协作钩子；方案按钩子今天的代码来谈。
- 主 agent 现喂此刻的重型闸（主 agent 不带前缀）：`{cargo,} test --all`、`cd "$(git rev-parse --show-toplevel)" && bash <<'EOF'`（heredoc 里 `cargo test --all`）、`env =1 cargo test --all` 三条都 rc=2，`ls -la` rc=0。
- 修补那一件的读数（主 agent 没复跑，是线索）：正推腿第一轮的 575 条在改前、改后两棵树上重喂，迁移 `0->0 166 / 0->2 168 / 2->0 12 / 2->2 229`，「改前拒、改后放」的 12 条是第一轮认定的改前误拒。
- 修补的要点：W1 展开出的空串照 bash 去掉、引号写出的空串留着，补零序列照 bash 算并与 `lib_heavy_tests.py` 共用；W2 `open_substitution_tail` 改走 `command_substitution_end` / `double_quote_end` / `backquote_end` 同一套扫描；W3 重定向的 fd 号挂到运算符上、`ln -s` 相对源从链接所在目录算；W4 `env` 后开头是 `=` 的词当赋值；W5 `bash -c` 之后接着认选项、`builtin` 后不是内建的放行、只定义不调用的 `function` 放行（只在命令文本这一层）、`bash -s --` 后的词当位置参数。
- 第一轮正推腿的复跑装置还在：`research/prompts/shell-words-r1-sonnet-model/`（`tools/feed.sh`、`tools/rerun.sh`）。

## 三、格

### V1　修补修得对不对
第一轮五格的反例在修补后是否都判对；修补的每一处改法与 bash 的真实行为（展开次序、重定向、`env` 与 `bash` 的参数解析）对得上没有。

### V2　修补带进的新错判
修补改了共用的扫描与展开：格里要问，原来判对、第一轮没列到的写法里，有没有修补之后判错的（误拒或漏拒）；W5 新放行的三类会不会把该拒的命令一起放了。

### V3　从文件头判据出发的正当命令
不看 diff，只读三份钩子文件头写明的判据（什么拒、什么放）与 `.claude/main-agent.md`、`.claude/agent-common.md` 里照抄的起法，逐条造正当命令喂此刻的钩子，核有没有误拒；文件头写明要拒的写法类别里，各造一条最常见的写法核它照拒。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 正推腿甲（Sonnet） | V1、V2 | 照第一轮正推的做法：真 bash 与 GNU env 逐词对照，修补前后两棵树逐族喂，列出每一族判对判错与依据 |
| 正推腿乙（Sonnet） | V3 | 只从钩子文件头写明的判据与两份定义里照抄的起法出发，造正当命令与各类该拒写法的常见形态，核此刻的钩子判得对不对；不读 diff |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号；冻结副本路径在派发提示里给。
- 腿喂钩子在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里拷一份钩子喂，不在冻结副本或主工作区里改；喂法与原样输出写进报告。
- 不跑重型测试、不起 cargo、不碰 GPU；自证与探针经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 4G 的上限，排不上可以带 `ulimit -v` 与 `nice` 直接跑并写明。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
