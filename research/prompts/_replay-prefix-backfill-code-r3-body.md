# replay-prefix-backfill-code-r3：重放前缀第二轮改法合入之后的复核（2026-11-19）

代码轮第三轮，这一题的最后一轮。第二轮判决 `research/prompts/replay-prefix-backfill-code-r2-main-verification.md` 第三节第 1 到 5 条的改法已经 `scripts/gate/apply-writer-patch.py` 合进主工作区、回读退 0；第 3 条里 `.claude/gate.d/verus-pins.tsv` 第 47–49 行注释由主 agent 用 `scripts/kb/replace-once.py` 直接改（不在补丁里）；第 6 条（欠账登记）还没做。开工快照 `research/prompts/replay-prefix-backfill-code-r3-snapshot/`：`crates-sha256.txt` 七份、`governance-sha256.txt` 一份（`verus-pins.tsv`）、`changes.patch` 是合进来的那一份补丁原样（只改 `src/README.md`、`src/GPU_GC_PROOFS.md`、`verus-must-red-worlds.tsv`，不动 `.rs` 与 `crates/mutations.tsv`）。

方案按 `crates/` 今天的实现来谈。

## 实现今天的样子（主 agent 的观测，2026-11-19 现读）

- `crates/singlefs-replay-prefix/src/README.md:109`、`:111` 的文件区间模块义务数改成 19（实现员照门禁 formal-verification 文件头 verus-proof 格的成项规则数：`file_range_proofs.rs` 10 个 exec fn 加 9 个 proof fn；报告 `~/.local/share/singlefs/work/replay-prefix-backfill-r2-fixes/result.md`「第 1 条」）；文件表补了精确交集长度。
- `src/GPU_GC_PROOFS.md`：第 97 行改成任务不拿卡；四条复跑命令改成点名全名加 `--exact`（范围 4、5、82、2 个测试）；真卡对拍改成交 GPU 服务代跑、接口待定，两组 12 个全名列在文里；并进了 `NOT_RUN_NO_GPU_ADAPTER` 怎么读的现状句；G11 标成「模型，定义自检（三条 ensures 是 `record_contribution` 定义的展开）」。
- 必红表 `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv`：新加 13 行（含 `write-intersection-empty-has-zero-length`、`indexed-intersection-empty-has-zero-length` 两个 `None` 支），7 行第五段条数照第二轮判决第二节第 9 条点名的那一趟实测改，第 365 行注释改成 G30 没有产品调用点，覆盖写组合与精确交集长度两段补了注释行。
- `.claude/gate.d/verus-pins.tsv:47`「十个核（G30 没有产品调用点）与一个边界见证」、`:48`「整 crate 的义务数见 crate 行」、`:49` 去掉了日期；crate 行 `:50` 是 214。

## 这一轮攻什么

1. 第二轮改法第 1 条：19 这个数按成项规则数得对不对；`src/README.md` 里别处的义务数与必红数（实现员报的 `:117`、`:119`、`:126`、`:148`：`:119`「共 5 条义务」把 nonlinear_arith 子证明算成一项、几行按批次记的必红数、`:119`「同日继续补齐」是历史式说法）。
2. 第 2 条：复跑命令点名的全名是不是都在源码里、范围数对不对；真卡那一节有没有给出能照抄却会被拒的命令；`NOT_RUN_NO_GPU_ADAPTER` 那几句与 `.claude/rules/mutation-sampling.md`、`.claude/rules/verification.md` 讲同一件事的原文对不对得上。复跑一节点名的测试有一部分在 `singlefs-background-gpu` 与 `singlefs-background-gpu-tests`，那两个包别的会话正在拆，这一轮只判今天的树。
3. 第 4 条：新加 13 行必红每一行改坏的是不是名字说的那条 ensures；7 行条数改的值与实测对不对。第二轮只抽查了新加 75 行里的 30 行，剩下 45 行这一轮补看。
4. 主 agent 直接改的 `verus-pins.tsv` 三行注释说的是不是现状。
5. 第 5 条：G11 的「定义自检」与文件图例对不对得上。

已攻过而没打中的角度不重复：u32 界断言、窗口常数推导、覆盖写按段 ensures、G17 量词、两份测试的参照与边界值、交集长度那一路的定义与两条分割引理。

## 验证读数（主 agent 经任务服务跑出的）

- 合入后验证（`scripts/gate/merge-and-verify.sh --item singlefs-replay-prefix:src/lib.rs`）：照预期退 3（必红表判不出测试目标），「fmt 过」「clippy 过」，验证红 0 项。
- formal-verification 在合入前的快照上点 21 项（`proof` 加 20 行必红）：「必红世界：…的 20 行（--item 点名的 20 行）每行改坏之后 Verus 都报错，证明错落点与第六段 landing= 相同」；「verified 之和 214、0 errors（登记 214）」；「formal-verification.sh：2 格里 2 格绿」。

### 检查单：走不到的理由二次查实

本轮改动没有这一类：补丁只改两份说明文件与必红表，没有 `.rs`（实现员报告「走不到的理由」一节：「无（没改产品代码）」）。

### 检查单：类型与名字

本轮改动没有新加的类型与函数；新加 13 行必红的变异名（第一段）判说不说得出它改坏的是哪一条。

### 检查单：不兼容旧代码与旧格式

本轮改动没有这一类：没有认旧版本的分支或旧布局号。

### 检查单：不新建判定库

本轮改动没有这一类：必红表是门禁读的登记表，判定进 prune-server 那一份库。

### 检查单：所有任务多机可用

本轮改动没有新测试；说明文件里的复跑命令有没有写死本机路径、主机名，腿核。

### 检查单：测试优先多线程

本轮改动没有这一类：没有新测试与崩溃检测。

### 检查单：带 --item 的命令与项

实现员报告「要主 agent 跑的命令」点 21 项（`proof` 加 20 行必红），主 agent 照原样交了，结局见「验证读数」。腿核新加 13 行能不能被 formal-verification 的 `--list-items` 列出、说明文件里的复跑命令能不能原样交。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没有新加崩溃点重放流或崩溃枚举用例。
