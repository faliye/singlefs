# 广谱变异接进提交门禁的治理改动：第二轮判决（2026-10-03）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 H1 H2 H3 H4 H5 C1 C2 C3 A4 E4 -->

## 一、这一轮的材料与证据

| 项 | 路径 | sha256 前 8 位 |
|---|---|---|
| 正文 | `research/prompts/_m3-broad-gate-gov-r2-body.md` | — |
| 辩方腿（Sonnet） | `research/prompts/m3-broad-gate-gov-r2-sonnet-output.md`（125 行） | cc338f70 |
| 攻方腿（Opus） | `research/prompts/m3-broad-gate-gov-r2-opus-output.md`（242 行），模型 `m3-broad-gate-gov-r2-opus-model/` | 45124ad9 |

- 快照：冻结副本 `/tmp/claude-1000/m3-gov-r2-freeze/tree/` 对 `research/prompts/m3-broad-gate-gov-r2-snapshot/tree-sha256.txt` 跑 `sha256sum -c`，退 0。
- `cite-check.py` 两份报告一起跑（`--root` 冻结副本与主仓，带背景材料）：「核了 100 处引文，对上 100 处，没判 0 处」；攻方报告开头列的 19 个文件 sha256 前 16 位现算全对。
- 主 agent 在冻结副本里现查：`scripts/detached-run.sh:209` 带进单元的是「SINGLEFS_HEAVY_TESTS PATH HOME CARGO_HOME RUSTUP_HOME」五个（正文第二节 H4 写「只带 PATH、HOME」错了，辩方指出）；`scripts/broad-mutation.py:1907` 内存撞顶分支在 `:1914` 测试没过分支之前；`:317` 经 runner 限 CPU 的只有 one-test 与两段测试步，没有文档测试步；`:2254` 全局键只认五类编译变量；驱动与门禁里 `人工`、`豁免`、`waive`、`--decide` 零命中；cargo-mutants 27.1.0 `src/config.rs:60`「pub exclude_re: Vec<String>,」；驱动 `:2251` 起 `.cargo/` 下的文件进全局键，`:836` 的 `resync_command` 打 `=user-request`。
- 两条腿的数都在冻结副本的拷贝或抽出来的纯函数上量；各格「没打中」都是一次抽样。复跑见第五节。

## 二、逐条判决

| # | 格 | 两条腿 | 判 |
|---|---|---|---|
| 1 | H1-3a 查标记那一侧 | 攻方量：样本上写下合法标记 A，暂存 B、工作区写回 A，`stage-marker-check` 退 0「ok」，同一现场门禁第一格红。`commit-stages.md` 第 19 行照退 0 记复用、第 2b 步不起。整轮 `gate-staged.sh` 兜得住（推的） | **打中（主 agent 复跑坐实）**：第一轮第 3 行的改法只修了写标记，查标记那一侧仍按工作区算指纹 |
| 2 | H1-3b 与别的五道同规矩只做了一半 | 攻方：别的四道输入路径脏就不跑；这一道照跑、照出判定。驱动、`mutation-prune.py`、排除表都读工作区那一份（排除表量过：工作区与暂存区两份 sha256 不同时门禁取工作区的）；驱动或剪枝规则没暂存的改动、不动判法版本时，④ 与整轮都绿（推的） | 打中（形态，假绿那一支推的）。与第 1 行同一改法：第 19 行给这一道也加别的四道那两核，核的路径取登记行全部路径（含导出范围之外的驱动、剪枝规则、排除表、包装脚本、阶段自己），脏就不起 ②、不查标记。零轮。「驱动与剪枝规则内容进全局键」不采：每改一次驱动整版重判，代价没量 |
| 3 | H2-a 样本与被判驱动共用函数 | 攻方量：16 种写错版里 11 种让 26 份样本一份不翻。主 agent 跑攻方交来的驱动自证变异：其中 6 种（`global_key_lines` 丢 toolchain、file、env、tool、limits 行，`fingerprint_of_manifest` 只算内容）驱动 `--selftest` 也是「55 格都对」 | **打中（坐实）**：门禁第 272、321、325 行三个判红分支在这 6 种写错下谁都不盯。改法两样都做（零轮）：一份写死全局键全文与指纹字面值的绿样本（修门禁这一侧）；驱动自证补一格，在 `--sync-tree` 与真跑之间逐类换工具版本、限时、编译变量、Cargo 文件，各要退 7（修驱动沿用这一侧） |
| 4 | H3 用户定案丙的出路 | 两条腿同判：慢、加大就过的超时与撞顶走得通（`--timeout-floor`、`--memory-cap` 不进全局键，不退 7）；确定性的两种走不出去——文档测试真卡住（文档测试步不经 runner，按整步限时）、无界分配撞内存（撞顶分支在测试没过之前）；两种不进结论表，之后每次提交都重选、照旧红；「交用户定」之后门禁没有认这个决定的入口。频率没量（真库新驱动下结果行只 2 行）。辩方另指：③ 重起会重烧本趟已落的超时、撞顶行；「加大限时再 `--item --rerun`」驱动自证没有专格 | **打中，交用户（第四节第 1 条）**。两样小处采：自证补「加大限时再 `--item --rerun`」一格；`commit-stages.md` 第 26 行「已有定论的条不重跑」写明超时、撞顶不算定论 |
| 5 | H3-b / H4-6 ② ④ 与 ③ 环境不同 | 两条腿各自打中同一形：② ④ 在会话 shell 里起、③ 经脱离单元只带五个变量，会话 shell 设了 `RUSTFLAGS` 或 `CARGO_PROFILE_*` 时 ③ 每次退 7，出路重做 ② 还在同一个 shell 里，绕不出去。今天两腿现查都没设 | 打中（条件式，推的）。改法：`commit-stages.md` ③ 那一步起单元之前核会话 shell 里全局键认的那几类变量，设了的写成 `-- env 名=值` 带进单元。零轮 |
| 6 | H1-5 旁支：全局键的环境部分比构建环境窄 | 攻方量（模型）：`CARGO_TARGET_*_RUSTFLAGS`、`RUSTC_WRAPPER`、`CARGO_BUILD_TARGET`、`~/.cargo/config.toml` 改 overflow-checks 时全局键不变，`scripts/admission.py` 的构建环境定义对它们都变；用户级 cargo 配置改了之后旧配置下判的「抓到」照沿用（推的） | 打中（一个让门禁该红不红的输入，正文第五节第一句）。改法：全局键的环境与 cargo 配置部分改调 `admission.py` 的构建环境那一份，不各写一份。零轮 |
| 7 | 第一轮第 10 行「等价变异没有排除」 | 辩方：驱动与门禁没有排除开关是真的，但清单由 cargo-mutants 在导出树里生成，`.cargo/mutants.toml` 的 `exclude_re` 排除得了单条变异；代价是 `.cargo/` 进全局键、整版重判 | 认：第一轮第 10 行够不着一半。出路句改成「等价变异可写进 `.cargo/mutants.toml` 的 `exclude_re`（整版重判）」；它同时是第四节第 1 条的一个候选 |
| 8 | H5 别的会话在 ② 与 ③ 之间换树 | 攻方：③ 读此刻指针、不核是不是 ① 那个提交，白跑别人那一版几小时，④ 红，不出假绿（推的） | 记欠账，不在这一批改（只费时间）；零轮改法「③ 核指针是不是 ① 那个提交」记进下一批治理清单 |
| 9 | 措辞债 | 辩方：重型闸拒绝说明少 `-h`；驱动 `resync_command` 在提交流程里仍打 `=user-request`；驱动文件头退出码一段缺 8 | 采，逐处改 |
| 10 | 第一轮第 1–5、7–9 行复核 | 辩方：站得住；第 5 行「全局键进版本指纹」对「门禁另核」站得住（结果行不带全局键、每趟真跑覆盖 `global-key.txt`、同源码同工具链重做 `--sync-tree` 落回同一版）；第 3 行的写标记一侧比别的五道更严 | 认；第 3 行另见本表第 1、2 行 |
| 11 | 没打中的 | H1-4 钉住（五种形状）、H1-5 门禁与驱动现算的全局键同一份（真库 16 行里 15 行逐字节同，不同的一行是排除表确实改过）、H1-6、H1-7（5 400 种结局组合 0 次误判）、H2-b（20 个开关翻面与文件头逐项相同）、H4 五份文件无整句不符 | 一次抽样，不支撑「没问题」，只记 |

## 三、改法（定下的，进下一批）

- 第 1、2 行：`commit-stages.md` 第 19 行这一道同别的四道核登记行全部路径的工作区等于暂存区、没有未跟踪文件，不过不起 ②、不查标记。
- 第 3 行：写死字面值的绿样本；驱动自证逐类换全局键输入要退 7；自证补「加大限时再 `--item --rerun`」一格（第 4 行）。
- 第 5 行：③ 起单元前核全局键认的变量、设了的带进单元。
- 第 6 行：全局键的环境与 cargo 配置部分与 `admission.py` 构建环境共用一份。
- 第 4、7、9 行：出路句与措辞逐处改；第 26 行写明超时、撞顶不算定论。
- 第 8 行进下一批治理清单。
- 以上都是零轮；改到的定义与规则（`commit-stages.md`、可能的 `agent-common.md`）走第四节第 2 条。

## 四、交用户

| # | 问题 | 候选 |
|---|---|---|
| 1 | 确定性的超时与内存撞顶（文档测试真卡住、无界分配）怎么出得去 | 甲：立一张「用户判」登记（按变异名记 caught / unviable / 等价与理由，驱动与门禁认、登记表进全局键）；乙：用 cargo-mutants 的 `.cargo/mutants.toml` `exclude_re` 排除那一条（不新造机制，代价整版重判，理由写在配置旁）；丙：不加机制，红着提交、在当前里程碑记录记一行，直到补一条第一段就红的测试；三样都是零轮，频率没量 |
| 2 | 这一批改法往下怎么走 | 甲：开第三轮（终轮），攻第三节改法与第四节第 1 条选定的那一样；乙：就此照第三节改，改完的定义与规则由用户逐份豁免 |

## 五、复跑

攻方 `rerun.sh` 在草稿目录 `/tmp/claude-1000/-home-fy5090-code-singlefs/8adc3bc8-1d0b-43bd-9a3f-9e63dae04aaf/scratchpad/gov2-verify/` 里拷模型目录、对冻结副本重跑，退 0：十份输出七份逐字节相同（`samples-base.txt`「判对 26 个、判错 0 个：无」、`switches.txt`、`h2-sweep.txt` 16 行、`h2-mutants.list`、两份 `h1-7-combos`（5000 趟「mismatches=0」）、`h1-5-real-key.txt`），另三份（`exp1-marker-check.txt`、`exp2-probe.txt`、`h1-5-env-gap.txt`）只差临时目录路径，判定行相同（`exp1` 第 16 行「marker-check exit=0」、第 28 行「stage(B staged) exit=1」）。攻方交主 agent 的 `driver-selftest-mutants.sh`（带 `SINGLEFS_HEAVY_TESTS=user-request`）：base「55 格都对」；fop-content-blind 4 格错、vk-no-toolchain 1、vk-no-source 5、vk-no-global 2、vk-no-judge 1、gkl-no-rules 1、gkl-no-exclusions 1、exd-always-none 1；gkl-no-toolchain、gkl-no-files、gkl-no-env、gkl-no-tools、gkl-no-limits、fop-path-blind、limits-constant、export-no-cargo 八种「55 格都对」——后两种样本抓到（5 份、11 份），前六种样本与驱动自证都不翻。H1-3a 真仓复现、H1-3b 现场 2（要驱动真跑）没做。

## 回看决策

不涉及决策：这一轮判的是门禁、钩子、广谱变异驱动与 agent 定义、skill，不碰文件系统的任何决策分项。
