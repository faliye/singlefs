# shell-words-r1：共用切词补认法、detached-run 写法认法与 heredoc 进替换——第一轮判决（2026-10-26）

<!-- doc-lint:not-numbers W1 W2 W3 W4 W5 W6 -->

## 一、这一轮的材料与证据

| 项 | 路径 | sha256 前 8 位 |
|---|---|---|
| 正文 | `research/prompts/_shell-words-r1-body.md` | 1d0c06ec |
| 正推腿（Sonnet，W1–W6） | `research/prompts/shell-words-r1-sonnet-output.md`（465 行），模型目录 `shell-words-r1-sonnet-model/` | 7dd8ca32 |
| 攻方腿（Opus，W1–W6） | `research/prompts/shell-words-r1-opus-output.md` | e7228ede |
| 攻方腿重派（Opus，W1–W6，找错判样本） | `research/prompts/shell-words-r1-opus-s2-output.md` | c7d00261 |
| 被判文件快照 | `research/prompts/shell-words-r1-snapshot/files-sha256.txt` | — |

- 写判决前，快照清单五份 `sha256sum -c` 全部 OK。
- `python3 scripts/agents/snapshot-tree.py shell-words-r1 --out ~/.local/share/singlefs/work/shell-words-r1-verify/shell-words-r1-tree`：「清单点名 5 份，倒推出 5 份，sha256 对上 5 份」。
- `python3 scripts/agents/cite-check.py research/prompts/shell-words-r1-sonnet-output.md research/prompts/shell-words-r1-opus-output.md research/prompts/shell-words-r1-opus-s2-output.md --root ~/.local/share/singlefs/work/shell-words-r1-verify/shell-words-r1-tree --background research/prompts/_shell-words-r1-body.md`：「核了 31 处引文、3 处裸行号，对上 30 处，没判 4 处」。没判的 4 处逐处人工核：`.claude/main-agent.md:72`（快照树里没有这份）现查第 72 行含 `detached-run.sh start --wall-clock <within-2h|over-2h> --label <名> -- <命令>`，对上；另 3 处（`/scratch/d2.sh:4`、`/scratch/sub/l2.sh:4`、`/scratch/s.sh:4`）是腿的原样输出里的临时文件，不是引文。

## 二、攻方缺席

两条攻方腿都在搭好新旧两版装置、冒烟三条之后被安全分类器拦下，六格都是「没核成」，不当「没打中」。两条腿停之前的冒烟三条一致：主 agent 不带前缀起 `time -p cargo test --all`，新版拒、旧版放（这次改动要补的写法）；`ls` 两版都放；clerk 起 `detached-run.sh` 两版都拒。这一轮只有正推腿一条腿判了格。

## 三、逐条判决

| # | 格 | 腿报的 | 主 agent 现查 | 判 |
|---|---|---|---|---|
| 1 | W1 花括号 | 整词展开出空份时不去掉那一份：`{cargo,} test --all` 等 9 条改前拒、改后放；补零序列 `{01..02}` 漏拒 | 在草稿两棵树（改后 = 快照那一刻的钩子，改前 = `own.diff` 反向打回）上用正推腿的 `tools/feed.sh` 喂重型闸，主 agent 不带前缀：`{cargo,} test --all` 改后 rc=0、改前 rc=2 | 打中，采 |
| 2 | W2 heredoc 进替换 | `open_substitution_tail` 在双引号里收不了 `)`，`cd "$(git rev-parse --show-toplevel)" && bash <<'EOF'` 这类顶层 heredoc 被当数据剥掉，重型闸与检出器各 8 条改前拒、改后放 | 同一喂法：`cd "$(git rev-parse --show-toplevel)" && bash <<'EOF'` 里 `cargo test --all`，改后 rc=0、改前 rc=2 | 打中，采；这一写法在会话里常见，口子在线上，先修 |
| 3 | W3 detached-run 新认法 | `.claude/main-agent.md` 的起法一条没误拒；`cat … > 拷贝 2>/dev/null` 两份钩子都漏；`ln -s` 相对源跨目录一漏一误拒；名单外至少 25 个启动器不认 | 引文核过（见第一节）；漏认与误拒没现喂 | 采前两处（交修补）；名单外启动器不在这一批的射程，进下一轮清单 |
| 4 | W4 env 宽松赋值 | `-S`、`-u`、`-i`、`--`、`-C` 判对；`env =1 cargo test --all` 改前改后都放（代码特意排除开头 `=`） | 同一喂法：`env =1 cargo test --all` 改后 rc=0、改前 rc=0 | 采（不是这次改出来的回归，顺手补） |
| 5 | W5 time / builtin / function / bash 选项 / `//` | 原来判对的没判错；`bash -c -- '代码'` 漏拒（改前也漏）；`builtin` 后不是内建、只定义不调用的 `function`、`bash -s -- 词` 三处偏严 | 没现喂 | 采（交修补，修补那一件先复现再改） |
| 6 | W6 语料证据 | 没核成：读不到实现员那份语料；自己的 4545 条仓内文档语料四组 0 条不同，但上面各反例的形状在里面 0 条——「判定没变」是因为没出现 | 读过报告这一节 | 采结论：实现员的语料对比不支撑「没回归」 |

## 四、改法

修补派给工具实现员（草稿目录 `~/.local/share/singlefs/work/debt-tw-shell-words-fix`）：W1 照 bash 去掉空份并认补零序列；W2 让 `open_substitution_tail` 与 `command_substitution_end` / `shell_tokens` 共用一份引号与括号状态；W3 两处照重定向与 `ln` 相对路径语义改；W4 开头 `=` 也当赋值；W5 四处照 bash 行为。验收：正推腿的 575 条在改前、改后两棵树上重喂，除认定的 12 条改前误拒外「改前拒、改后放」为 0。W1、W2 先修先换上。

## 五、下一轮

修补交回后开第二轮，攻第一轮的修补本身。攻方腿这一轮两次被拦，第二轮的腿怎么派交用户定。

## 六、交用户

| 项 | 状态 |
|---|---|
| 第二轮攻方腿的做法 | 用户 2026-10-26 弹窗定「两条正推式腿」：一条照第一轮正推的做法核修补，另一条 Sonnet 换切入角（只从钩子文件头写明的判据出发逐条造正当命令、核误拒），不派 Opus 攻方 |
| 名单外启动器（至少 25 个）要不要认 | 进下一轮清单，零轮 |

## 这一轮被判过的文件

- `.claude/hooks/lib_shell_words.py`：W1、W2、W4、W5 打中
- `.claude/hooks/bash-command-detector.sh`：W2、W3 打中
- `.claude/hooks/heavy-test-guard.sh`：W1、W2、W3 打中

## 回看决策

不涉及决策：被判的是项目的协作钩子，不碰 `.claude/kb/decisions/` 下任何一条。
