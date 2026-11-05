# shell-words-r3：共用切词修补的第三轮（最终轮）——判决（2026-10-28）

<!-- doc-lint:not-numbers X1 X2 X3 W1 W2 W3 W4 W5 Z1 Z2 Z3 Z4 Z5 Z6 Z7 Z8 Z9 Z10 Z11 Z12 Z13 Z14 Z15 Z16 -->

## 一、这一轮的材料与证据

| 项 | 路径 | sha256 前 8 位 |
|---|---|---|
| 正文 | `research/prompts/_shell-words-r3-body.md` | c5aeda2d |
| 正推腿甲（Sonnet，X1、X3） | `research/prompts/shell-words-r3-sonnet-output.md`（317 行），模型目录 `shell-words-r3-sonnet-model/` | 33277e81 |
| 正推腿乙（Sonnet，X2） | `research/prompts/shell-words-r3-sonnet-s2-output.md`（425 行），模型目录 `shell-words-r3-sonnet-s2-model/` | 91ba6036 |
| 被判文件快照 | `research/prompts/shell-words-r3-snapshot/files-sha256.txt` | — |
| 冻结副本 | `~/.local/share/singlefs/work/shell-words-r3-frozen/`（五份钩子与 r1.diff、fix.diff、fix2.diff） | — |

开工快照倒推与引文核对（主 agent 跑的原样命令）：

```
python3 scripts/agents/snapshot-tree.py shell-words-r3 --out <草稿目录>/shell-words-r3-tree
python3 scripts/agents/cite-check.py research/prompts/shell-words-r3-sonnet-output.md research/prompts/shell-words-r3-sonnet-s2-output.md --root <草稿目录>/shell-words-r3-tree --background research/prompts/_shell-words-r3-body.md
```

快照点名 6 份，倒推出 6 份，sha256 都对上（五份钩子取自 HEAD，正文取工作区此刻那一份）。引文核了 18 处，都对上；裸行号 5 处没判，主 agent 逐处人工核：`bash-command-detector.sh:29` 原文「或之后有 `tail --pid` / `while|until … kill -0` 轮询」与腿乙引的意思一致，`heavy-test-guard.sh:176`、`:195` 是腿引的文件头说法所在行。

## 二、逐条判决

主 agent 现喂：把命令写成 PreToolUse 输入，在草稿目录 cwd 下直接喂此刻的 `.claude/hooks/heavy-test-guard.sh`（主 agent 身份、不带前缀）。

### X1　前两轮站住的判定撑不撑得住（腿甲）

| # | 条 | 腿报的 | 主 agent 现查 | 判 |
|---|---|---|---|---|
| 1 | W1、W4、二-2、二-4–8、二-10 | 9 条撑得住，各自的漏拒都是 0（样本 4838 条加族清单） | 没逐条复跑；X3 的逐数对上是旁证 | 撑得住，采 |
| 2 | W2 命令替换的括号状态 | 命令替换里不加括号的 case 模式 `a)`、注释里的 `)` 把替换提前收尾，四棵树都漏拒，不留检出记录 | `echo "$(case a in a) cargo test --all;; esac)"` 退 0；对照 `cargo test --all` 退 2 | 打中（静默漏拒，前两轮样本之外），采 |
| 3 | W5 关键字 | `coproc cargo test --all` 四棵树都漏拒；`bash --version s.sh` 偏严 | `coproc cargo test --all` 退 0 | 漏拒那一半打中，采；偏严那一半没现喂，采为线索 |
| 4 | W3 / 二-3 拷出再起 | `ln -s $PWD/源`、`cat < 源 > 目的`、组重定向、`x=1>` 漏拒；检出钩子 `{v}>` 偏严（第一轮修补后才有） | 没现喂 | 采；`>`、组重定向这几种与第二轮第 11 条的拷出类同一族 |
| 5 | 二-9 花括号落点 | `dd of=records/{a,b}.md` 与文件头不符，偏严 | 没现喂 | 采（偏严） |

### X2　留到下一轮的清单全不全（腿乙）

| # | 腿报的 | 主 agent 现查 | 判 |
|---|---|---|---|
| 6 | 清单里的 A1 拷出、A2 `bash -s < 文件`、A3 xargs、A4 名单外启动器、B2 重定向目标花括号、C 函数当启动器都复现；B1、B3 没复现（规则没说） | 没现喂 | 采 |
| 7 | 清单外 63 条，16 个子类（Z1–Z16）：文件头写明看不见或接受的 38 条，文件头没写的 23 条，文件头写了认、实际不认的 2 条 | Z15 `F=--workspace; cargo test $F`、`cargo $(echo test --workspace)` 退 0；Z16 `bash -c 'cargo "$@"' _ test --workspace`、`sh -c 'exec "$@"' sh cargo test --workspace`、`bash -c '$0 test --workspace' cargo` 退 0 | 打中，清单不全，采 |
| 8 | Z13：bg 里 `tail --pid`、`kill -0` 轮询等它，文件头 `bash-command-detector.sh:29` 写了算等，实际按没等拒 | 文件头第 29 行原文核过；钩子退出码没现喂 | 采（文件头与实现不符） |

主 agent 现查时另看到：cwd 在主工作区时，上面五条都被合入锁那一道拒，拒的理由不是重型。所以在主工作区里喂样本，看不出这一类漏拒。

### X3　修补报的数能不能核（腿甲）

| # | 腿报的 | 判 |
|---|---|---|
| 9 | 逐条变了的表、keyfeeds 11 处、small_checks、腿乙 37 条、575 条矩阵与「改前拒改后放」13 条全部对上；逐族迁移表 17 族里 13 族逐格相同，brace、env、r1-575 共 13 条对不上，来自腿甲的裁判把 sudo、systemd-run 换成替身；fuzz2 一条 ±1 没追到 | 数能核，采；对不上的那几格出自两边裁判的口径不同，不算修补报错 |
| 10 | 14 份自证退出码与格数相同；「heavy-test-guard 两格红在改前就有」复核不了 | 那两格照修补报告记，不是这一批的；复核不了照写 |

## 三、最终轮之后怎么办

这一轮是最终轮，不再开第四轮。前两轮采的第 2–10 条改法，在这一轮站住了（X1 里撑得住的 9 条，X3 数对上）。这一轮打中的几处（第 2、3、4、5、7、8 条）不是前两轮修补带进来的：四棵树（第一轮之前、第一轮被判、第二轮被判、此刻）的退出码相同，只有检出钩子 `{v}>` 那一条是第一轮修补带出的偏严。它们并进一百三十七节，下一批钩子修补一次改：

- 静默漏拒，先改：命令替换里的 case 模式与注释（第 2 条）、`coproc`（第 3 条）、位置参数拼命令与变量拼旗标（第 7 条 Z15、Z16）。
- 其余漏拒与偏严：第 4、5、8 条与 Z 类余下的几类，照腿乙报告「五」逐类定放不放。

这一批改法都是零轮，下一次改完照治理文件按批走三方的规矩办（要用户点名）。

## 回看决策

不涉及决策：被判的是项目的协作钩子，不碰 `.claude/kb/decisions/` 下任何一条。

## 这一轮被判过的文件

- `.claude/hooks/lib_shell_words.py`：X1 打中（命令替换里的 case 模式与注释收尾、`coproc` 不在关键字表、`{v}>`）；X2 打中（位置参数拼命令、变量拼旗标）
- `.claude/hooks/heavy-test-guard.sh`：X1 打中（拷出再起的几种写法）；X2 打中（Z15、Z16）
- `.claude/hooks/bash-command-detector.sh`：X1 打中（二-9 花括号落点偏严）；X2 打中（Z13 文件头与实现不符、Z6 here-string 与 `env -S` 里的 claude）
- `.claude/hooks/lib_heavy_tests.py`：X1 撑得住（二-2 编译目录那一道跳函数体，偏严的 POSIX `foo() {…}` 归第 1 条之外的余项）
- `.claude/hooks/lib_workspace_cargo_lock.py`：X1 撑得住（第二轮第 8 条）
