# archive-rename-r1 判决：术语全仓改名 + 实验记录归档 + 配套门禁改动（2026-09-21）

背景材料 `_archive-rename-r1-background.md`（正文 `_archive-rename-r1-body.md` + 小节清单 + 附录一），diff 附录 `_archive-rename-r1-diff.md`。判据 J1–J8 在正文第三节，跑前写死。

**开工快照是倒推件，不是派腿时记的。** 主 agent 漏了 `.claude/rules/implementation-workflow.md`「代码轮派腿之前记一份开工快照」那一步，核查员据此拒绝开工（对的）。补的快照在 `/tmp/claude-1000/archive-rename-r1-snapshot/`，按每份文件此刻 `git status --porcelain` 的两位状态码倒推：`[ M]` 取 HEAD 版、`[MM]` 取索引版、`[M ]` 与 `[  ]` 取工作区原件；主 agent 派腿之后自己改过、且与腿引用的载体有交集的只有三份，逐处列在快照目录的 `倒推说明.md` 里。欠账 C434（代码轮的开工快照没有会红的检查）。

## 〇、核查员的核对表与它纠正的一处说法

核对表 `research/prompts/archive-rename-r1-verifier-checks.md`（382 行）。判别力自证：把攻方腿对 `crates/singlefs-harness/src/first_transaction_regions.rs:78` 的引文行号改成 79 去核，取到的是 derive 属性、与引文不符，判 ✗ ——方法分得出差别。

70 处核对：✓64、✗5、核不动 1。5 处 ✗ 全是行号或计数偏 1–3（`lib-link-targets.py` 的正则在 82 行不是 80、`mask_code()` 的范围是 52-62 不是 54-61、fixtures 跳过行号里的 54、`git diff --cached --numstat` 是 155 行插入不是 157、`.claude/kb/term-renames.md` 那句引文在第 12 行不是第 11 行），没有一处是「引的内容根本不存在」或「产物数字被编造」。

**两条要记下来的**：

1. **主 agent 给核查员的那句「其余九份自派腿以来一处没动」不准确。** `.claude/gate.d/90-term-renames.sh`、`.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md`、`.claude/term-rename-exempt` 三份在快照之后被主 agent 改过，改动正是在回应 J8② 与 J8③ 两处打中。核查员全程按快照核、不对主树，所以判定不受影响；但这句话本身是错的，记在这里。⇒ 这也是 C434（代码轮的开工快照没有会红的检查）要拦的另一半：快照记下之后主 agent 还会接着改，「哪几份、几点、改了几处」同样要有东西盯着，不能靠说。
2. **攻方腿 J1 报的那个读数今天不复现。** 核查员按 `replay.sh` 的 `driver_e142` 现编现跑，得到 `equal=21`（21 个区域全部逐字节相等），不是腿报的 `equal=19` 加两行 `equal=unknown`。腿自己在报告里写明了它用的是过期构建产物，所以这不是夸大。⇒ **J1 的那个具体实例不成立**，站住的是它指出的结构问题：配对键失配时报 `equal=unknown` 而不判红，F1 停机条款够不着——那一条记成 C435（装置与 crates 逐字节比对的配对键失配时静默少比）。

## 一、按路径点名这次改动的文件

`research/scripts/archive-past-rounds.py`、`research/scripts/sweep-term.py`、`research/scripts/insert-row.py`、`research/scripts/replay.sh`、`.claude/gate.d/40-results-cited.sh`、`.claude/gate.d/88-quoted-result-lines.sh`、`.claude/gate.d/90-term-renames.sh`、`.claude/gate.d/91-archive-past-rounds.sh`、`.claude/kb/term-renames.md`、`.claude/term-rename-exempt`、`.claude/rules/path-moves.md`、`.claude/agent-common.md`、`research/perf-by-milestone.md`；`crates/` 这几份（改名波及的断言、文件名与格式化）：`crates/singlefs-core/src/system_configuration.rs`（由 `crates/singlefs-core/src/superblock.rs` 改名而来）、`crates/singlefs-harness/src/device_log.rs`、`crates/singlefs-harness/src/first_transaction_regions.rs`、`crates/singlefs-harness/tests/first_transaction_step_five_publish.rs`、`crates/singlefs-harness/tests/first_transaction_step_six_recovery.rs`、`crates/singlefs-format/src/lib.rs`、`crates/singlefs-core/src/write_accounting.rs`、`crates/singlefs-harness/src/bin/first_transaction_on_device.rs`、`crates/singlefs-harness/src/lib.rs`、`crates/singlefs-harness/src/segments.rs`。

**下面这些不是这一轮改的**，是门禁 56 号的 diff 基准（`refs/singlefs/gate-ok`，停在 `b1c8cefb`、四个提交之前）把前几次提交的改动一并带进了范围。现查 `git diff --cached HEAD -- crates` 这一次提交对它们一个字节都没动，各自的判决在当时那几次提交里：`crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker/src/lib.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-core/src/block_device.rs`、`crates/singlefs-core/src/lib.rs`、`crates/singlefs-core/src/make_filesystem.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-harness/src/crash.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/model.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-harness/src/scenario.rs`。基准为什么会停在四个提交之前，记在 C439（gate-ok 前移让同轮早先写的行不再被判）。

**改过的 agent 定义**（门禁 72 号按路径点名）：`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md` 两份去掉自指称呼（「这个实验」换成「登记里」「实验页」等不自指的说法，见第六节 J8 那一行的处置）；`.claude/agents/three-way-verifier.md` 去掉一处位置指代（「按下面第 2 步核它」改成「按第 2 步核它」）。三份改的都是措辞，判据、输入与交付一个字没动；这一轮的三方本身就用了核查员那份定义，它按定义拒绝开工那一次正是它在起作用（第〇节）。

## 二、各格判定

| 格 | 判定 | 依据 | 处置 |
|---|---|---|---|
| J1 一处字面量改动有没有改变行为 | **打中，但不触发反向接受条款** | 攻方腿：区域名那个串是 E142 装置与 `crates/` 逐字节比对的配对键，两端对不上时不判红、静默少比两格（`equal=unknown`），而 F1 停机条款只认 `equal=false`。腿自己写明用的是过期构建产物，真正可达的同型历史没实测 | 条款说「J1 打中 ⇒ 四处字面量改动整个撤回」，不采纳：腿在报告里自己写了撤回之后那两格**照样中**——撤回只是把两端一起换回旧名，键还是键、还是只有一句注释钉着。真正的病是**配对失败不判红**，与改不改名无关 ⇒ 记 C435（装置与 `crates/` 逐字节比对的配对键失配时静默少比，不判红） |
| J2 段序列那个串横跨四处载体，有没有一处没跟上 | **打中，已改** | 主 agent 现查坐实：`research/perf-by-milestone.md:350` 说「产物 `e142-…-2026-09-14-round2-slot4096.out` 里逐字是」，而那份产物（版本库历史里）第 34 行写 `superblock_slot×2`，正文 `:354` 写的是新名 | 改指向：换成树里真正含那一行的 `e142-first-txn-dry-run-2026-09-18-change-count-three.out` 第 34 行，两行逐字节相同 |
| J3 归档还有没有别的检查失去输入而没人报 | **打中两处** | ① 主 agent 自己先打中：四份被装置源码 `include_str!` **编译期**读的产物被当成记录删掉，HEAD 上 `research` 整个编不过 ② 正推腿打中第二处：`.claude/agents/three-way-materials.md:20` 与 `:28` 用反引号写着已被删的 `research/prompts/_m2-code-r1-diff.md` 全路径，`still_an_input()` 的 `INPUT_TREES`、`rewrite_links()` 的 `SCAN`、门禁 23 号的 `mask_code()` 三层各自够不着 | ① 已改：`still_an_input()` 跑在删之前、点到名的不进待删集合；`missing_code_inputs()` 全仓扫 `include!` 一族指空的判红；自检补四格。**这一轮真正要证的那一格已证**：同一批四份产物现在被认出来是输入而不是记录 ② 记 C436（反引号里的路径，归档与门禁 23 号都够不着） |
| J4 「删上一轮、留本轮」在边界上对不对 | **打中一处（构造边界）** | 正推腿现造现测：`git mv`、部分暂存、软链接都判得对；`research/results/` 下一个未改动的 gitlink（`160000`）会让 `--apply` 在 `os.remove()` 上抛未捕获的 `IsADirectoryError` 直接中止。仓里现在没有子模块 | 条款说 J4 打中逐条记账、不阻塞 ⇒ 记 C437（归档遇到 gitlink 会未捕获异常中止） |
| J5 `replay.sh`「产物已归档」一档会不会掩盖真的对不上 | **没打中** | 本地攻方两次抽样都判「分得出」，理由指到 `replay.sh` 的分支：那一档只在产物**不在树里**时给，产物在树里而内容退化仍走 `exact` 比对判红。主 agent 先前三向自证过（归档→0、一致→0、篡改→1） | 不改 |
| J6 门禁 88 号收成「按本次改动判」之后漏判了什么 | **打中两处** | 正推腿现造现测：① 当天新写进「## 历史版本」一节的错行完全不判（脚本按整节排除，不分新旧）② `refs/singlefs/gate-ok` 在任意一次全绿时前移，同一轮内该点之前写下的错行永久脱离判定范围 | 条款说 J6 打中「那个射程改回去，并写明改回之后失去什么」。**射程不改回去**：历史节整节不判是用户 2026-09-21 定的（以前的是历史、仅作参考），改回去等于判一批产物已归档的旧引文。两处各记一笔：C438（历史节里当天新写的错行不判）、C439（`gate-ok` 前移让同轮早先写的行脱离判定范围） |
| J7 豁免 6 条有没有一条把本该改的挡住 | **没打中** | 本地攻方两次抽样逐条分类，6 条都归「别家术语或工具自身」；主 agent 另用一条比工具更宽的独立正则（含各种驼峰蛇形变体与 `sb` 系缩写）全仓复扫，10 处命中全是 jbd2 / btrfscue / fscrypt / F2FS 的引文、警告记录里必须留旧名的原话、以及一句记变更的话 | 不改。豁免这一轮从 6 条涨到 13 条，新增七条都是这一轮打中之后补的（冻结证据目录、历史类文件、引文块） |
| J8 这次改动让哪些先前成立的断言不再成立 | **打中三处** | ① `research/perf-by-milestone.md` 整个在所有门禁射程之外（88 号的对象集写死 `.claude/kb/**/*.md`）② 门禁 90 号的出路逐字要求「源码与留存产物同改」，与 `evidence-discipline`「原样保存的证据不许事后改」正面打架 ③ `.claude/kb/experiments/152-*.md:243`「整行抄自产物」变成假话（归档产物第 21 行写 `superblock_slot×4`） | ① 已改：88 号射程扩到 `research/**/*.md`（排除 `prompts/`、`results/` 两个冻结目录）② 已改：90 号的出路改成**产物分两类**（见第三节）③ 已改：那一行引文块回到产物的逐字原样（留旧名），并登记进豁免表 |

## 三、这一轮定下来的一条判据：留存产物分两类

攻方 J8② 把两条规矩的冲突摆上台面。定法：

- **跑得出来的**（装置源码在树里、`replay.sh` 判 `exact`）跟着源码一起换名，换完复跑必须仍然逐字节相同——同一份源码重新生成，不是改证据。
- **跑不出来的**（已归档、要虚机或真设备、别人的产物）一个字节都不许动，连同引它的正文**整段留旧名**，登记进 `.claude/term-rename-exempt`。

分不清时问一句：**这份产物今天跑得出来吗**。跑不出来就不许改它，也不许改引它的那一行。写进 `.claude/rules/path-moves.md`「改一个全仓术语」一节与门禁 90 号的出路。

## 四、门禁侧的净变化

| 门禁 | 改了什么 | 为什么 |
|---|---|---|
| 40 号 | 认裸文件名当「点名」；新加第三道：点名的产物要么在树里、要么在版本库历史里取得回来 | 归档把引用改成只留文件名，原判据认不出来、13 个已跑实验一起红；只放宽第二道会让它退化成「写个像文件名的串」，所以第三道必须同时加 |
| 88 号 | 射程扩到 `research/**/*.md`；树里找不到时去版本库历史里找归档产物 | 前者补 J8① 那个洞；后者让判据由「抓归档过的产物」回到「抓抄错的数」，全量扫 596 行引文，树里找不到 416 行、去历史里找只剩 1 行——那 1 行正是 J8③ |
| 90 号 | 出路改成产物分两类 | J8② |
| 91 号 | 新立：还留着上一轮的实验记录就判红 | 用户 2026-09-21 定的归档规矩 |

## 五、改名是不是纯替换：三次独立验证

用户要求这次改名「多次验证、注意各种驼峰蛇形变体」。三次各走不同的路子：

| # | 路子 | 结果 |
|---|---|---|
| 1 | `sweep-term.py --check`（工具自己的检测正则比替换表宽，见 C428） | 全仓 1212 份文本文件、13 条改名登记、13 条各带理由的豁免，搜不出旧名 |
| 2 | 主 agent 另写一条**比工具更宽**的独立正则（`super[_ -]?[Bb]lock`、`SUPER_BLOCK`、`SuperBlock`、`superBlock`、中文旧名，以及 `sb` / `_sb` / `sb_` 三种缩写形态）全仓复扫，不用工具的表 | 10 处命中，逐处现查全是正当豁免：jbd2 / btrfscue / fscrypt / F2FS 的引文、警告记录里必须留旧名的原话、一句记变更的话。`sb` 系缩写 0 处；另核了没有新造 `sc` / `SysCfg` 一类缩写 |
| 3 | 把**改名前**的每一份留存产物按登记表正向替换，与树里这一版逐字节比 | `research/results/` 下 10 份在 `3cff909^` 有对应版本的产物，**10 / 10 逐字节相同**（含文件名本身也改过的 5 份，按旧路径取）。⇒ 改名在产物上是纯替换，没有一个数变过。这与攻方腿 J8② 独立做的那次一致：它用改名后编的二进制现跑 `e115`，输出与树里那份逐字节相同 |

## 回看决策

不涉及决策：理由是这一轮改的是流程、门禁与文档，`.claude/kb/decisions/` 下的条款一条都没动，盘上格式与字节表也没动（2026-09-21 现查 `git diff --cached --name-only -- .claude/kb/decisions .claude/kb/layout`，零命中）。

## 历史版本

### 2026-09-21

- 首版。
