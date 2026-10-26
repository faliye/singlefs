# shell-words-r1：共用切词补认法、detached-run 写法认法与 heredoc 进替换（2026-10-26）

## 一、题面

欠账 `records/2026-10-23-subagent拆分提案.md` 第四十三节第 1、2 行与第四十节第 18 行，用户先前定「另开一批单独修，单独走三方」。工具实现员已经改完并换上（报告 `~/.local/share/singlefs/work/debt-tw-shell-words/result.md`，只含这一批改动的 diff 在同目录 `own.diff`，757 行）。这一轮攻的是改完的钩子代码：新认法有没有判错（该拒的放了、该放的拒了），改动有没有让原来判对的写法判错。

被判的文件：
- `.claude/hooks/lib_shell_words.py`（`skip_prefixes`、`shell_invocation`、`shell_tokens`、`brace_expand`、`brace_expand_once`、`brace_sequence`、`resolve_path`、`single_leading_slash`、`heredoc_feeds_shell`、`open_substitution_tail`、`commands_at_command_position`、`words_break_is_set`）
- `.claude/hooks/bash-command-detector.sh`（`detached_run_copy_destinations`、`detached_run_cat_destinations`、`detached_run_commands`、`detached_run_calls`、`shell_layers`、自证）
- `.claude/hooks/heavy-test-guard.sh`（`copy_pairs`、`cat_copy_pairs`、`COPYING_COMMANDS`、`INSTALL_OPTIONS_WITH_VALUE`、自证、文件头与 `POLICY` 的广谱变异一句）

## 二、实现今天的样子（主 agent 的观测，2026-10-26）

- `crates/` 里没有相关实现：`grep -rln 'shell_tokens\|heredoc_feeds_shell' crates/` 零命中。被判的是钩子，不是文件系统代码；方案按钩子今天的代码来谈。
- 实现员的语料对比（报告「语料对比」一节，主 agent 没复跑）：从 1000 份会话记录取去重后 146675 条命令，切词结果变了 773 条，两版钩子判定变了 20 次：19 次 implementation-writer 从拒变放（花括号没展开、目标被当算不出而误拒），1 次收紧（`env "$var=$val" bash stage-selftest.sh` 照 env 真实行为当赋值）；主 agent 的命令判定一次都没变。
- 实现员自己点的最脆弱两处（报告「给三方的材料」）：花括号展开放在 `shell_tokens` 里、只分引号里外，分不出 `[[ … ]]`、`case` 模式、`declare` 参数这些 bash 不展开的位置，`env a={x,y}` 不展开，`${a:-{b,c}}` 嵌套会切错；`open_substitution_tail` 自己维护一套引号与括号状态，与 `command_substitution_end` / `shell_tokens` 不是同一份，双引号里开的 `$(` 里再有双引号时会判错。
- 第四十三节第 2 行里 6 类写法没加认法，靠「不换写法绕过去」的规矩（删签名行或改签名的拷贝、签名挪出前 4 KiB、python subprocess、xargs、find -exec、PATH 上的拷贝）。
- 两份钩子的自证在主工作区退 0；13 个新弄坏开关各红在文件头点名的格（`double-slash-kept` 在重型闸不红，文件头写了原因）。`heavy-test-guard.sh` 自证此刻另红两格，是别的会话新加的 GPU 格，实现员把自己的改动反向撤掉再跑照样红。

## 三、格

### W1　花括号展开的射程
展开放在所有钩子共用的切词里。格里要问：哪些位置 bash 不展开而这里展开（偏严的误拒），哪些位置 bash 展开而这里不展开（偏松的漏拒）；有没有一种写法借花括号让一条该拒的命令（重型测试、写出写范围、按模式杀进程）切成放行的样子。

### W2　heredoc 进没收尾的替换
`open_substitution_tail` 与另两份切词各维护一套状态。格里要问：两套状态对不上的输入里，判错的方向是偏严还是回到改前的漏拒；有没有一种写法让 `$(bash <<EOF` 这类喂给 shell 的 heredoc 照旧被当数据剥掉。

### W3　detached-run 写法的新认法
`cat >` 拷出、命令词代入变量、eval 字面代码、`/usr/bin/time` / `flock` / `systemd-run --wait` 包着起、重型闸认 `ln -s` / `install` / `mv` 再起。格里要问：新认法会不会误拒正当的写法（主 agent 按 `.claude/main-agent.md` 照抄的那几种起法）；没加认法的 6 类之外，还有没有一样改动之后变得认不出的。

### W4　env 宽松赋值的收紧
`env` 后面名字不合变量名的也当赋值。格里要问：收紧之后有没有正当命令被拒（语料里只有 1 次）；`env -S`、`env -u`、`env -i` 与带 `--` 的组合判得对不对。

### W5　time、builtin、function、bash 选项与 `//` 路径
格里要问：这几样补认之后，原来判对的写法有没有判错；`bash -` 读标准输入、`bash --rcfile <文件>` 这类有没有被当成「起了一个 shell 跑脚本」而误判。

### W6　语料证据够不够
146675 条命令、判定变 20 次。格里要问：这份语料罩不罩得住 W1–W5 的射程（哪类写法语料里没有）；判定没变是不是因为这类写法在语料里根本不出现。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | W1–W6 | 从 bash 手册的展开次序与各钩子的判据推：每格列出新认法该拒、该放的写法族，逐族核今天的代码判得对不对，给出判对的依据（代码行） |
| 云端攻方（Opus） | W1–W6 | 在钩子的拷贝上造反例：喂一条命令，让新认法误拒正当命令，或让该拒的命令借这次的改动放行；每个反例给原样命令与钩子的原样输出 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号；冻结副本路径在派发提示里给。
- 腿喂钩子在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里拷一份钩子喂，不在冻结副本或主工作区里改；喂法与原样输出写进报告。
- 不跑重型测试、不起 cargo、不碰 GPU；自证与探针经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 4G 的上限。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
