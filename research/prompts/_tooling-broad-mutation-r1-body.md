# 广谱变异驱动与重型闸的改动：第一轮正文（2026-09-30）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

## 一、题面

用户 2026-09-30 定：「钩子的代码要三方 打一次」。被判的是工具实现员同日交回的一批改动（报告 `/tmp/claude-1000/m3-r1/broad-mutation-tooling/report.md`，sha256 198730467b8ec8bb56658ef59d106b36b51be32640403ca0bd7311604ac5aa3f），它关 `.claude/kb/checks-owed.md` 的 C489（singlefs-core 那 1786 条广谱变异没跑） 里「跑不起来」那一半：

- `.claude/hooks/lib_heavy_tests.py`：重型测试分类加一类「广谱变异」（kind `broad-mutation`），自检表加 9 例，弄坏开关 `LIB_HEAVY_TESTS_BREAK=broad-mutation-ignored`。
- `.claude/hooks/heavy-test-guard.sh`：文件头分类表加两行注释。
- `.claude/gate.d/code-tooling.sh`：`SELFTEST_RUNNERS` 加一条。
- 新文件 `research/scripts/broad-mutation.py`（1343 行）：逐条、可点名、中断可续接、两段式的广谱变异驱动。

diff 在 `research/prompts/_tooling-broad-mutation-r1-diff.md`；四份文件的 sha256 在 `research/prompts/tooling-broad-mutation-r1-snapshot/crates-sha256.txt`。腿直接读仓里这四份文件（这一轮判完之前主 agent 不改它们），引行号先 `sha256sum -c` 核过再引。

用户对驱动的三条验收（原话）：「这个没什么夜间不夜间的 需要的话就可以一直跑 31核的跑」「但是你记得 中断要续接。 不能中断后从头开始」「最好把测试做成一条一条的 可以指定启动那一条 不要做成一个大块」。

## 二、实现今天的样子（主 agent 的观测，2026-09-30，HEAD fcc4d340 加工作区）

- 被变异的对象是 `crates/singlefs-core/src` 与 `crates/singlefs-checker/src`；驱动在 `git archive` 导出的专用树上就地改、复用编译目录，库目录 `~/.local/share/singlefs/broad-mutation`，清单 5091 条（core 3326、checker 1765，提交 fcc4d3404c80）。
- 主 agent 验收过的（带 `SINGLEFS_HEAVY_TESTS=user-request`）：`python3 research/scripts/broad-mutation.py --selftest` 退 0；七个弄坏开关 `BROAD_MUTATION_BREAK=` rerun-done、interrupted-as-done、no-row-flush、stage-two-missed-as-caught、summary-mismatch-silent、no-restore、accept-unknown-item 逐个退 1（日志 `/tmp/claude-1000/m3-r1/main/broad-break-*.log`、`broad-selftest.log`）；`--summary` 原样「BROAD_MUTATION_SUMMARY fingerprint=19c45e021b5638f6 commit=fcc4d3404c80 total=5091 caught=2 missed=0 unviable=1 timeout=0 memory_cap=0 stage_one_only=0 not_run=5088」。
- 没验过的：`lib_heavy_tests.py` 的自检这一轮没人跑过（工具实现员只做了 `py_compile`）；第二段一步都没在真 cargo 上跑过；两台机器结果的合并导入没做。
- 分类的新规则（`lib_heavy_tests.py` 的 diff）：命令名是仓里 `research/scripts/` 下的 `broad-mutation.py` 时，参数里有 `--sync-tree`、`--list-items`、`--summary`、`--help`、`-h` 之一就不算重型；同时有 `--item` 与 `--skip-stage-two` 也不算；`--selftest` 归「提交时才跑的检查」；其余算重型。
- 已知的规格缺陷（主 agent 自己的）：第二段用完整的 `cargo test`，把第一段跑过的 harness 档测试不经 eatmydata 重跑一遍；该只补文档测试与 checker 档快档。判决之后改，不在这一轮里动。

## 三、格

### T1　分类放不放得过该拦的
找一条子 agent 能发出、分类交回「不算重型」而实际会起重型测试或整趟广谱变异的命令：查询类参数与运行参数并用（例 `--summary` 加 `--shard`、`--sync-tree` 之后脚本接不接着跑——要读驱动的参数处理核）；`--item` 给多次或给一个范围加 `--skip-stage-two`（点名 5091 条等于整趟）；经 `bash -c`、`env`、`nice`、`python3 -m`、相对路径 `./`、符号链接、改名拷贝、`python3 - < 脚本` 起；`--stage-two-only` 与 `--skip-stage-two` 并用时驱动怎么办。每条给命令原文、分类的返回、驱动实际会做什么。

### T2　分类拦不拦错不该拦的
主 agent 带前缀的全量、`--item … --skip-stage-two` 的小样、`--list-items` / `--summary` 各自走得通；分类改动有没有改变已有类的判定（自检表原有各例逐例对照）。

### T3　驱动的三条验收站不站得住
逐条攻：（a）中断续接——在哪几个时刻被 SIGKILL 会丢已有结果、重跑已有结果的条、把被打断的条当成已完成、或留下改坏的源文件而下一条在坏树上判（列时刻：落行之前、落行与 fsync 之间、恢复原件之前、第二段编译中、持锁进程死掉之后）；（b）一条一条——结果行的键与清单序号在源码指纹变了之后还对不对得上、`--item` 重跑追加之后 `--summary` 取哪一行；（c）判定对不对——「抓到」会不会来自环境（超时、内存撞顶、编译缓存串了上一条变异、nextest 与 eatmydata 下判定与 `cargo test` 不同的测试），「没红」会不会来自测试根本没跑到（过滤词、`#[ignore]`、fail-fast 之后的条）。自证 18 格与 7 个开关罩不罩得住上面每一条，罩不住的列出来。

### T4　登记与规矩
`code-tooling.sh` 的登记、`lib_heavy_tests.py` 自检表新增各例、`heavy-test-guard.sh` 文件头，与 `.claude/singlefs-ai-sop/rules/sop-first.md`「加门禁或钩子之前，先找已有的」、`.claude/singlefs-ai-sop/rules/preflight-discipline.md`、`.claude/singlefs-ai-sop/rules/command-safety.md` 对不对得上；驱动与已有的 `research/scripts/mutate.sh`、`research/scripts/crates-mutation-rows.py`、`.claude/gate.d/checker-tier-crates-mutation-replay.sh` 有没有整段重复该抽成共用的。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T4 | 核「代码做的是不是题面与用户验收说的」，逐格给依据行 |
| 云端攻方（Opus） | T1、T3 为主，T2、T4 找反例 | 造绕过分类的命令与打断续接的时刻；能在草稿目录里用假清单、假命令实测的就实测（不起真 cargo、不跑驱动的真变异、不跑钩子与脚本的 `--selftest`） |

第一轮，不派辩方。

## 五、跑前写死的判据

- T1 给出一条子 agent 发得出、分类放过、实际起重型的命令 ⇒ 分类必须改，那条命令做成自检表的一例。
- T3 给出一个时刻让已有结果丢失、被重跑、或被打断的条被当成已完成 ⇒ 驱动必须改，那个时刻做成自证的一格加一个弄坏开关。
- T3（c）给出一条让「抓到」或「没红」失真的路径 ⇒ 记进判决；已经跑出的结果行里受影响的要点名重跑。
- 「没打中」只抽一次样不算（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」）。
- 腿提的改法算被攻过零轮。

交付：腿的报告写 `research/prompts/tooling-broad-mutation-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；攻方的脚本与产物放 `research/prompts/tooling-broad-mutation-r1-opus-model/`；草稿与临时文件放 `/tmp/claude-1000/<腿名>/`。方案按 `crates/` 与这四份文件今天的样子来谈。
