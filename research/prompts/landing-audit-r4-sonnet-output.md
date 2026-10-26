# landing-audit-r4 正推腿报告（K1：第九批每处改法做没做到它那一行）

日期：2026-10-23。腿：云端正推（three-way-forward）。被判：拆分提案 `records/2026-09-16-subagent拆分提案.md` 第九十节 ⑥ 到 ㉔ 共十九行（行号 1477 到 1495）的第九批改法。没读禁读清单里的攻方腿报告。

## 怎么核的

- 被判文件在开工时与快照一致：`sha256sum -c research/prompts/landing-audit-r4-snapshot/judged-sha256.txt` 对全部文件打 OK、没有一行不符，所以下面读的是主工作区的文件，行号现查。
- 钩子与脚本的自证、开关逐条复跑、门禁样本档，都经 `scripts/hosts/run-with-memory-cap.sh` 起，带范围声明，在主工作区的字面路径上跑（重型闸只放行登记成轻的自证和 `stage-selftest.sh --item <阶段文件名>`）。
- 造输入的在草稿目录 `~/.local/share/singlefs/work/landing-audit-r4-sonnet/`：钩子拷到 `repo/.claude/hooks/`（其余用符号链接指回主工作区），喂造出来的会话记录与 hook 输入，输出落在 `out/`。没写模型、没有模型目录，所以没有 `tools/rerun.sh`；下面是自己写的探针（不是模型）与 sha256：

```
a388ef7c5e2d2c747a101ee0fccf6ecfb43414f31d916408139a36ab8899c5f1  fix/mk.py        造交回闸要的会话记录与 hook 输入（命名检查七例、写回员四例）
79794bc4d458f1f5497ff53b5e8912bf687880d65cf3f15825c6c827a44a8767  fix/det.py       造检出钩子的 hook 输入（mktemp 十例）
9a871236356ef5c35dc42b81a927d3e14588820c8cdbf10769f0e272736a5cbb  fix/replay2.py   照门禁读法逐条设弄坏开关跑自证
9f6d662238cbbf9553a8b500e08cd4f763c4df5247787b5eba5f4bc88b6969c7  fix/bsl.py       从 code-tooling.sh 里原样取出的开关表读法
279de2eeda6b0d489b94bc20d2f98f50221ebd865535607410f05a3e5bddc2e1  fix/probe13.py   调 agent-watch.py 的 memory_threshold 探 ⑬
```

- 复跑：`python3 fix/mk.py` 再 `CLAUDE_PROJECT_DIR=<仓根> bash repo/.claude/hooks/handback-guard.sh < fix/<例>.json`；`python3 fix/det.py` 再 `bash repo/.claude/hooks/bash-command-detector.sh < fix/det-<例>.json`；`python3 fix/replay2.py <钩子或脚本绝对路径> [开关值…]`；`python3 -I fix/probe13.py`。
- 自证读数（主工作区，全部退 0）：交回闸 49 种、派发闸 483 种、检出钩子 764 种、写闸 229 种、`lib_heavy_tests.py` 787 种（记录里写 783，文件与快照 sha256 相同，差的 4 种我没追）、`heavy-test-guard.sh` 1401 种（记录写 1395）、`dispatch-prompt.py` 92 项、`agent-cost.py` 23 项（其中 outcome 的 21 个子 agent）、`merge-and-verify.sh` 45 种、`claim-experiment.sh`、`lib-lag-baseline.py` 9 项、`agent-watch.py` 一整趟（约 4 分钟，退 0）。
- 门禁样本档（`stage-selftest.sh --item`，各点一道）全部判得对：doc-registries 51 个用例、doc-decisions 53、doc-experiments 60、checker-independence-and-sync 16、code-tooling 94。主 agent 说样本档要样本与暂存区相同，这一趟没遇到这个拦：五道都直接跑完了。
- 开关逐条复跑（`fix/replay2.py`，读法用门禁自己那份）：派发闸 98 条、交回闸 19 条、`subagent_transcript.py` 2 条，点名的失败行原文都在失败行里、都退 1，不对 0 条；合入后验证脚本新加的 7 条 checker 档快档开关逐个打开，都退 1（我的简易比对认不了「……」省略，这 7 条只证明红，没逐项核点名）。

## 总表

| 行 | 要拦什么（一句话） | 判定 |
|---|---|---|
| ⑥ 1477 | 交回闸核会话记录里跑没跑命名检查 | 做到 |
| ⑦ 1478 | 写回员规格目标不在又没写「找不到的目标」 | 做到 |
| ⑧ 1479 | 写回员写范围覆盖 `.claude/kb-directories` 登记目录，含子目录 | 做到 |
| ⑨ 1480 | 「实验号住在哪」一份登记，占号脚本与孤儿格都读 | 做到 |
| ⑩ 1481 | 上游 gate-overlap 认 hook-swap 与 mv 写法 | 本批不在范围（上游，交发版会话） |
| ⑪ 1482 | 第八九处「与基准比」并进库 | 本批不在范围（第七批已做），抽查读法还在 |
| ⑫ 1483 | 登记表认顶层 `lib-` 库 | 做到（带 ㉖ 已记的一个副作用） |
| ⑬ 1484 | 看门狗阈值加绝对下限 | 做到；512 是推的，已写明 |
| ⑭ 1485 | 登记成轻的自证临时目录里的 ✗ 不判长跑判红 | 做到（认的范围比这一行写的略宽） |
| ⑮ 1486 | 比对对象表固定形态、格逐行对 | 做到 |
| ⑯ 1487 | 三方腿类型表收进 frontmatter、四处读它 | 四处做到；真派一次没核成；同三个名字在别的表里还抄着 |
| ⑰ 1488 | 检出钩子认草稿目录里的 `mktemp -d` | 做到 |
| ⑱ 1489 | 取基准取不到的两处改调库、判红 | 做到 |
| ⑲ 1490 | 派发闸 19 条点名对齐、加复跑表、主会话按记录时刻筛 | 做到 |
| ⑳ 1491 | ⑰ 按本提示草稿目录判、三方形态整份认、自证拿真提示当样本 | 一半（自证用的是仿写，不是真提示） |
| ㉑ 1492 | ⑰ 加判 SUMMARY、FINDINGS、ANALYSIS 前缀 | 做到 |
| ㉒ 1493 | 结局账落盘、定义点名时机 | 做到；「records/ 下放 .txt 门禁认不认」我读出来了：不判、也不拒 |
| ㉓ 1494 | 合入后验证加 checker 档快档一步 | 一半（脚本与第 52 行做到；主 agent 定义第 3 条的例外句没写） |
| ㉔ 1495 | 三处小尾巴 | 做到 |

下面每一节写：改法的文件:行、判定、依据（命令与输出）、推翻条件。

## ⑥ 交回闸核命名检查（九十节 1477）

行的要求：「交回闸核执行员、实现员、工具员自己的会话记录里跑没跑过那两条命名检查命令」（`records/2026-09-16-subagent拆分提案.md:1477`）。

改法落点：
- 判法在 `.claude/hooks/handback-guard.sh` 的 `naming_source_tree`（331 行）、`naming_check_runs`（351 行）、`naming_problems`（409 行）、`naming_refusal`（431 行），文件头 ⑤ 一段在 26 到 33 行。
- 判的类型是 implementation-writer、experiment-runner、tooling-writer，与那一行点名的三种一致。
- 「真跑过」只看命令：「只看命令，不看报告里写了什么」（`.claude/hooks/handback-guard.sh:30`）。
- rsync 副本那一支不判：「rsync 拷的副本，共用约束写明两条命令交主 agent」（`.claude/hooks/handback-guard.sh:32`），与 `.claude/agent-common.md:99` 那一条对得上。
- 共用约束一侧：交回正文闸那一条补了「那棵树最后一次改 `.rs` 之后没真跑过」这一样（`.claude/agent-common.md:94`）。

我造的输入（`fix/mk.py`，会话记录里 Edit 了主工作区 `crates/singlefs-core/src/lib.rs`），拷来的钩子，hook 输入走 stdin，原样退出码：

| 例 | 会话记录里 | 期望 | 实测退出码 |
|---|---|---|---|
| n-unrun | 只有 Edit，一条 Bash 都没有 | 拒 | 2（说明里点名「改完之后没跑：code-source-discipline.sh 那四格、naming-lint」） |
| n-run | Edit 之后跑两条，根写绝对路径，naming-lint 退 1（Exit code 1） | 放 | 0 |
| n-dot | Edit 之后跑两条，根写 `.` | 放 | 0 |
| n-fullrun-before | 两条先跑、Edit 在后 | 拒 | 2 |
| n-toplevel | 两条的根写 `$(git rev-parse --show-toplevel)` | 九十节记为口子 | 2（误拒，已记 ㉕） |
| n-mv | 只有 Bash `mv` 换了 `.rs` | 九十节记为口子 | 0（放过，已记 ㉕） |
| n-sedi | 只有 Bash `sed -i` 改了 `.rs` | 同上 | 0（放过；`sed -i` 没单独点名，同属「Bash 里写 `.rs`」） |

自证：`bash .claude/hooks/handback-guard.sh --selftest` 退 0，「查了 49 种」。19 条弄坏开关逐条设上跑自证（`fix/replay2.py`）：`handback-guard.sh switches 19 bad 0`。

判定：做到。没越出这一行：没有碰别的类型。漏的两种（mv、cp、重定向写的 `.rs`；根写命令替换）是九十节自己在 ㉕ 里记着的。
推翻条件：实现员在主工作区 Edit 了 `.rs`、之后两条命名检查都真跑过（Bash 有结果）还被拒；或只跑了一条、跑在改动之前，却被放行。

## ⑦ 写回员规格目标（九十节 1478）

行的要求：规格点名的每个目标文件「在主工作区存在，或报告「找不到的目标」一节里有它」（`records/2026-09-16-subagent拆分提案.md:1478`）。

改法落点：`.claude/hooks/handback-guard.sh` 的 `scribe_spec_entries`（466 行）、`missing_targets_listed`（504 行）、`scribe_target_problems`（515 行），文件头 ⑥ 在 34 行起；共用约束 `.claude/agent-common.md:94` 补了「写回员规格里的目标文件在主工作区不在」这一样；对应的定义句在 `.claude/agents/kb-scribe.md:15`（「整份不写，停下」那一句，原来就有）。

我造的输入（规格 `spec.json` 两条：`.claude/kb/decisions.md` 在、`.claude/kb/r4-no-such-target.md` 不在；派发提示里写「规格：…/spec.json」与「报告：…」）：

| 例 | 报告里 | 期望 | 退出码 |
|---|---|---|---|
| s-none | 没有「找不到的目标」 | 拒 | 2，说明点名 `.claude/kb/r4-no-such-target.md` |
| s-listed | `## 找不到的目标` 一节写了它 | 放 | 0 |
| s-other | 只在 `## 别的` 一节提到 | 拒 | 2 |
| s-inline | 正文一行「找不到的目标：路径」，没有标题 | 拒（只认标题行里含这几个字的一节） | 2 |

判定：做到。s-inline 这一例说明认法是「标题」，行文里写一句不算；这和行里「一节」的字面一致。
推翻条件：目标不在、报告里有标题含「找不到的目标」且节里写着它，却被拒；或目标不在、报告没有这一节，却放行。

## ⑧ 写回员写范围覆盖 kb 目录登记（九十节 1479）

行的要求：门禁 code-tooling 加一格，登记目录在 `.claude/hooks/agent-write-scope.tsv` 写回员那几行里有覆盖它的模式，先定子目录算不算覆盖（`records/2026-09-16-subagent拆分提案.md:1479`）。

改法落点：
- 新格 `kb-directories-write-scope`：登记 `.claude/gate.d/code-tooling.sh:49`，阶段表 `.claude/gate.d/code-tooling.sh:218`，格函数 `cell_kb_directories_write_scope` 在 `.claude/gate.d/code-tooling.sh:4238`；探两条造出来的路径（`PROBE` 在 `.claude/gate.d/code-tooling.sh:4257`）：目录下一层与子目录里各一份 `.md`。
- 子目录算覆盖。真仓上因此红的是不变量清单目录，于是写范围表加一行 `.claude/hooks/agent-write-scope.tsv:13`（`crates/singlefs-checker/invariants/**/*.md`）、定义 `.claude/agents/kb-scribe.md:26` 与 `.claude/agents/kb-scribe.md:42` 同步。
- 写闸自证里 kb-scribe 不变量清单那一格由主 agent 换成「子目录里的 .md 放行、.rs 越界」两格。

核：
- `stage-selftest.sh --item code-tooling.sh`：94 个用例判得对，其中 `kb-directories-write-scope-green`、`kb-directories-write-scope-red` 判得对。
- 真仓：我没法在真仓上起这一格（扫仓的门禁阶段只许提交时跑），改用写闸钩子读表的同一份库 `lib_write_scope.py` 算 kb-scribe 的模式：

```
.claude/kb/kb-directories-write-scope-probe/kb-directories-write-scope-probe.md True
crates/singlefs-checker/invariants/kb-directories-write-scope-probe/kb-directories-write-scope-probe.md True
crates/singlefs-checker/invariants/a/b/c.md True
crates/singlefs-checker/invariants/x.rs False
crates/singlefs-checker/invariants/a/x.rs False
```
  两个登记目录的下一层与子目录里的 `.md` 都盖住，`.rs` 不盖。所以真仓上这一格现在应当绿；这是我复算的，不是门禁本身的输出。
- 写闸自证 `bash .claude/hooks/write-guard.sh --selftest` 退 0，「查了 229 种」，没有 `✗ 自检`。

判定：做到。有一处超出这一行的细节：这一批同时改了 `.claude/agents/kb-scribe.md` 与写范围表，不止加门禁一格；那是这一行去向列自己点名的后半（「写范围表与 kb-scribe 定义加 `**/*.md` 一行」），不算越位。
推翻条件：登记目录下的 `.rs` 被覆盖，或子目录里的 `.md` 没被覆盖，而这一格仍绿；或 `kb-scope-subdirectory-unchecked` 打开后 `kb-directories-write-scope-red` 以外的样本判错。

## ⑨ 实验号住在哪（九十节 1480）

行的要求：「一份「实验号住在哪」的目录登记，`scripts/kb/claim-experiment.sh` 与 experiment-orphans 格都读它」（`records/2026-09-16-subagent拆分提案.md:1480`）。

改法落点：
- 登记 `.claude/gate.d/experiment-number-directories.tsv`（五行：`research`、`scripts`、`crates/singlefs-checker-tier/src/bin`、`.claude/kb/experiments`、`.claude/kb/experiments.md`），读法只有 `scripts/kb/lib_experiment_number_directories.py`。
- 占号脚本读它：`REGISTRY_RELATIVE_PATH=.claude/gate.d/experiment-number-directories.tsv`（`scripts/kb/claim-experiment.sh:42`）。
- 孤儿格读它：`REGISTRY=.claude/gate.d/experiment-number-directories.tsv`（`.claude/gate.d/doc-experiments.sh:1700`，格函数 `cell_experiment_orphans` 在 1698 行）。

核：
- `bash scripts/kb/claim-experiment.sh --selftest` 退 0，成功句含「目录登记：入库装置目录里的 E40 被拒，登记里加 notes 一行之后 --next 从 E41 涨到 E51」。设 `CLAIM_EXPERIMENT_BREAK=registry-ignored` 再跑：退 1，输出「selftest: 目录登记：入库装置目录里有 E40、登记里还没有 notes，--next 该是 E41，拿到 E1」。
- `bash scripts/kb/claim-experiment.sh --next` 在真仓输出 `E198`；读法库 `numbers` 在真仓打 5 行 `entry`、1424 行 `number`。
- `stage-selftest.sh --item doc-experiments.sh`：60 个用例判得对，含 `experiment-orphans-registry-red`、`experiment-orphans-green`、`experiment-orphans-red`。

判定：做到。登记放在 `.claude/gate.d/` 而不是 `.claude/` 顶层，是交回报告里写明的取舍（写范围表里工具员到不了 `.claude/` 顶层）；这一行没规定位置，所以不算越位，是否合 `.claude/rules/directory-layout.md` 归 K2。
推翻条件：往登记里加一行新目录，占号 `--next` 或孤儿格有一处不跟着认；或登记写坏时占号照样给号。

## ⑩ 与 ⑪（九十节 1481、1482）

- ⑩ 是上游 `gate-overlap.py`，去向列写「上游填内容，交发版会话」；这一批的被判文件里没有它，没有可判的改法。
- ⑪ 去向列写「第七批已做」，不属这一批。我只抽查读法还在：`.claude/gate.d/doc-experiments.sh:1099` 调 `lag_baseline.baseline_rows`；`.claude/gate.d/doc-decisions.sh:3393` 调 `lag_baseline.baseline_text`。

## ⑫ 登记表认顶层 `lib-` 库（九十节 1483）

行的要求：库挪进子目录并登记，「或登记表认 `.claude/gate.d/` 下以 `lib-` 起头的库」（`records/2026-09-16-subagent拆分提案.md:1483`）。去向列写这一批做了后一种。

改法落点：`GATE_TOP_LIBRARY_PREFIX = "lib-"`（`.claude/hooks/lib_heavy_tests.py:695`）、`gate_top_library_key`（`.claude/hooks/lib_heavy_tests.py:1714`）；登记一行 `.claude/hooks/selftest-registry.tsv:177`（`.claude/gate.d/lib-lag-baseline.py` 登记 light）。

核：
- `python3 .claude/gate.d/lib-lag-baseline.py --selftest` 经重型闸放行，输出「✓ lib-lag-baseline.py 自证：9 项都过」。
- 同目录没登记的 `lib-forbidden-notations.py`（文件里有 `--selftest`）：`python3 .claude/gate.d/lib-forbidden-notations.py --selftest` 被重型闸当场拒，原因「这份自证没在登记表 .claude/hooks/selftest-registry.tsv 里」。这就是九十节 ㉖ 记的副作用，真发生了。
- `LIB_HEAVY_TESTS_BREAK=gate-top-library-as-stage python3 .claude/hooks/lib_heavy_tests.py --selftest` 红三格：「样本表：门禁目录顶层 lib- 起头的共用库没登记的 --selftest 照拒」、「样本登记表里写坏的几行逐条报出来」、「仓里的登记表……没有写坏的行」。
- `lib_heavy_tests.py --selftest` 退 0，查了 787 种；`heavy-test-guard.sh --selftest` 退 0，「查了 1401 种，其中该拒 266 种」（记录里写 1395，同样是别的会话后来加了格，我没追）；`scripts/gate/light-selftest-guard.py --selftest` 退 0，14 格都对。

判定：做到。副作用（一份带自证没登记的库从放行变被拒）是行自己没预见、㉖ 记了的；它不是改法没做到。
推翻条件：另一份顶层 `lib-` 库登记成 light 后单独 `--selftest` 仍被拒，或没登记的反而放行（现在是被拒，符合）。

## ⑬ 看门狗阈值下限（九十节 1484）

行的要求：峰值表里不到可分辨下限的条目「不当阈值、阈值设一个绝对下限（例如 0.5 GiB）或退回 `--process-rss-gibibytes`」（`records/2026-09-16-subagent拆分提案.md:1484`）；自证造峰值 0 的条目与常驻 0 的进程应当不报。

改法落点：`memory_threshold`（`scripts/agents/agent-watch.py:2659`），下限常数 `MEMORY_PEAK_FLOOR_MEBIBYTES_DEFAULT = 512`（`scripts/agents/agent-watch.py:655`），选项 `--process-rss-floor-mebibytes`（`scripts/agents/agent-watch.py:6056`），`scripts/agents/watch.conf:52` 写 512，`scripts/agents/watch.sh:37` 的键表加了键。

我的探针（`fix/probe13.py` 调真的 `memory_threshold`，固定线 8 GiB、下限 512 MiB）：

```
peak 0       -> 阈值   8192.0 MiB 来源 fixed
peak 4KiB    -> 阈值    512.0 MiB 来源 peakfloor
peak 100MiB  -> 阈值    512.0 MiB 来源 peakfloor
peak 340MiB  -> 阈值    512.0 MiB 来源 peakfloor
peak 400MiB  -> 阈值    600.0 MiB 来源 peak
peak 2GiB    -> 阈值   3072.0 MiB 来源 peak
固定线 0.125GiB、峰值 4KiB -> 128.0 MiB peakfloor
```
峰值 0 的条目走固定线、小峰值取下限、大峰值照乘 1.5：峰值表里真大的命令（400 MiB 起）阈值不变，仍报。`agent-watch.py --selftest` 整趟退 0，含新加的两格。

读现在的峰值表 `~/.local/share/singlefs/memory-peaks.tsv`（31120 条，一条一个键）：峰值不到 341 MiB（乘 1.5 不到 512 MiB）的 17873 条，约 57%，它们的阈值都被抬到 512 MiB；这比「0.00 GiB 与 0.01 GiB」宽得多，但行自己给的做法就是绝对下限（例 0.5 GiB），不算越位，只是影响面要知道。

推的、没量过：写进了文件。「512 本身是推的，没量过」（`scripts/agents/watch.conf:25`），选项帮助里同一句「512 是推的（没量过）」（`scripts/agents/agent-watch.py:6060`）。
判定：做到。
推翻条件：峰值 4 KiB 的条目配 `sleep` 仍报「进程常驻内存过线」；或峰值 400 MiB 的命令常驻 700 MiB 不报（阈值应当是 600 MiB）。

## ⑭ 自证临时目录里的 ✗ 不判长跑判红（九十节 1485）

行的要求：登记成 light 的自证进程的临时目录底下写的文件不判「长跑判红」；自证造一个开关轮输出 ✗ 的自证进程应当不报；并写明「与门禁阶段共用 TMPDIR、又自己起名建 mktemp 目录写 ✗ 的照报，没罩」（`records/2026-09-16-subagent拆分提案.md:1485`）。

改法落点：`light_selftest_scratch_owner`（`scripts/agents/agent-watch.py:2159`）从写文件的进程往上找登记成 light 的自证（用的是 `registered_light_selftest`，同一份登记表），`private_temporary_base`（`scripts/agents/agent-watch.py:2199`）判临时目录是不是单给这条自证的；自检格 `selftest_verdict_selftest_scratch`（`scripts/agents/agent-watch.py:4658`）真起三个进程、造四份带 ✗ 的文件。

核：`python3 scripts/agents/agent-watch.py --selftest` 整趟退 0（约 4 分钟，被工具挪进后台，完成通知回来后读的输出），成功句里有「✗ 不报」一语的那一行。自检里：共用 TMPDIR 的自证写在 mktemp 起的目录与探查目录里的两份不报、单给 TMPDIR 的自证写在 `sample-wheel-selftest.XXXXXX` 里的不报；直接写在 TMPDIR 底下的、没有自证上层的进程写的，照报。

判定：做到，范围比这一行写的略宽。这一行说共用 TMPDIR 又自己起名建 mktemp 目录写 ✗ 的照报；现在共用 TMPDIR 下名字像 mktemp 的（`SELFTEST_TEMPORARY_NAME`，`scripts/agents/agent-watch.py:2163` 起的说明）和探查前缀的一层不报，单给 TMPDIR 的整个都不报。即：行里写的「没罩」那一类里，名字像 mktemp 的现在罩住了。这是放宽，不是漏判。
推翻条件：一趟门禁 code-tooling 的 research-script-selftests 格里看门狗仍报 `light-selftest-guard-*/tmp/…/switch-wheel-*.out` 的长跑判红；或一条真红的长跑日志被写成「在登记成轻的自证 … 的临时目录里，✗ 不报」。我没在真门禁整趟里验这一条（只有自检里的造出来的进程树）。

## ⑮ 比对对象表固定形态（九十节 1486）

行的要求：比对对象表「表头逐字、一行一个正式跑」，格解析它、与登记表加文件头点名不登记的单元「逐行对；对不上判红」（`records/2026-09-16-subagent拆分提案.md:1486`）。

改法落点：
- 固定形态 = E165 r5 那张表的表头（`research/prompts/e165-r5-prereg.md:664`）。`.claude/agents/experiment-designer.md:30` 点名它。
- 登记表文件头写对法与点名：`.claude/gate.d/replay-comparison-units.tsv:14`（固定形态在 designer 第 1 步），五行不登记：`.claude/gate.d/replay-comparison-units.tsv:17` 到 21 行。
- 格：`.claude/gate.d/doc-experiments.sh:2640` 起的「④ 比对对象表与登记表逐行对」，`covers`（`.claude/gate.d/doc-experiments.sh:2576`）认旧登记「重跑产物名只写前缀之后一段」。
- 样本 `replay-comparison-units-table-red`、`replay-comparison-units-table-green`。

核：
- `stage-selftest.sh --item doc-experiments.sh`：60 个用例判得对，含 `replay-comparison-units-table-green`、`-table-red`、`-rekey-red`、`-zero`、`-green`、`-red`。
- 我独立重算真仓（python 解析 `research/prompts/e165-r5-prereg.md` 的表与登记表 14 行，用「留存产物名等于比对对象或重跑产物名，或以 -<重跑产物名> 收尾」）：

```
table rows 19 ['1', '2', '3'] ... 19 cols 7
registry rows 14
matched 14 unmatched ['15', '16', '17', '18', '19']
```
  未对上的正是文件头点名不登记的 #15 到 #19，所以真仓上这一格的预期是绿；这是我复算的，不是门禁本身的输出。

判定：做到。边角：只有写了这个固定表头的登记会被解析；别的登记（r1 到 r4 的表头不是固定形态）这一格看不见，文件头也写明了（`.claude/gate.d/replay-comparison-units.tsv:22` 一带的说明）。这是行本身「定一个固定形态」的射程，不是漏改。
推翻条件：往 E165 r5 的表里加一行既不在登记表、也没点名不登记的，这一格仍绿；或表头改一个字，这一格不报「登记出处里没有固定形态的表」。

## ⑯ 三方腿类型表（九十节 1487）

行的要求：三份定义 frontmatter 加一个键，三处改读它，「`scripts/agents/agent-cost.py` 是第四处」；自证核各处读到的类型集合与 frontmatter 逐个相同（`records/2026-09-16-subagent拆分提案.md:1487`）。

改法落点：三份定义第 9 行都是 `leg: three-way`（`.claude/agents/three-way-forward.md:9`、`.claude/agents/three-way-attack.md:9`、`.claude/agents/three-way-defense.md:9`）；读法只有 `.claude/hooks/lib_three_way_legs.py`；四处：`citation_checked_agents`（`.claude/hooks/handback-guard.sh:171`）、`three_way_report_types`（`.claude/hooks/runner-dispatch-guard.sh:1451`）、`three_way_report_types`（`scripts/agents/dispatch-prompt.py:130`）、`three_way_leg_types_of`（`scripts/agents/agent-cost.py:1053`）。四份自证各有「三方腿类型表」一格，新开关 `leg-types-hardcoded`（三处）与 `leg-types-by-prefix`（账表）。

核：四份自证退 0（交回闸 49、派发闸 483、骨架 92、账表 23）。派发闸与交回闸的 `leg-types-hardcoded` 开关都在 `fix/replay2.py` 的逐条复跑里退 1。样本我自己也喂了：三种腿的骨架填好放行（`--check-prompt` 退 0），报告行换成草稿目录下的 `result.md` 退 2。

三件要单独写：
1. 已做到的是「三方腿报告形态」这一张表。同样的三个名字在别的两张表里仍然是手抄的：任务带重活要线程上限行的类型 `THREAD_LIMIT_REQUIRED_AGENTS`（`.claude/hooks/runner-dispatch-guard.sh:1103`）与预计挂钟的 `LONG_RUN_AGENTS`（`.claude/hooks/runner-dispatch-guard.sh:1600`，由前一张推出），骨架一侧 `HEAVY_WORK_TYPES`（`scripts/agents/dispatch-prompt.py:148`）与 `LONG_RUN_TYPES`（`scripts/agents/dispatch-prompt.py:151`）；两边靠自证互比，不读 frontmatter。这四处不在这一行点名的范围里（行写的是「三方腿的类型表」），我记为边角：以后给定义加第四种 `leg: three-way`，这几张表不会跟着加。
2. 「加键之后还没真派过一次三方腿，landing-audit-r4 开轮时顺带核」（`records/2026-09-16-subagent拆分提案.md:1487`）。我这一趟就是三方腿，能正常派出，但不能当作核过：主会话 2026-10-22 开的（会话记录首行时间戳），三份定义是同一天晚些时候改的（`stat` 的修改时刻，晚于会话开头），也就是定义改在会话开始之后，我没有办法证明 Claude Code 读的是改后的定义还是会话开头缓存的。结论：没核成；要核得在改定义之后新开的会话里派一次。
3. 另：`git diff --cached` 里 `.claude/agents/three-way-attack.md`、`three-way-defense.md` 的 `required-inputs` 末尾多了「报告」，不在 b9-leg-types 的改动清单里，暂存区里就有，不是这一批的，我不判。

判定：四处做到；真派一次没核成；边角如上。
推翻条件：往某份定义 frontmatter 加或删 `leg: three-way` 后，四处读到的集合有一处不跟着变；或加键后三方腿派不出去。

## ⑰ 检出钩子认草稿目录里的 `mktemp -d`（九十节 1488）

行的要求：`mktemp -d` 的模板前缀落在这个子 agent 自己认领的草稿目录里时，目标按「草稿目录内」判、不报；自造命令应当不报，模板前缀在草稿目录外的照报（`records/2026-09-16-subagent拆分提案.md:1488`）。

改法落点：`own_scratch_directory`（`.claude/hooks/bash-command-detector.sh:1521`）两样都要对上（第一层目录 `.owner` 的会话 + 最近一把写者锁归这个 agent_id，或还没有认领文件时主会话记录里的派发回执对得上）；`mktemp_template_directory`（`.claude/hooks/bash-command-detector.sh:1555`）；`mktemp_substitutions_expanded`（`.claude/hooks/bash-command-detector.sh:1576`）。

我造的输入（`fix/det.py`：agent_type clerk，草稿目录 `lockdir/` 里放写者锁和认领文件给 `fakeagent1`，第一层 `.owner` 写会话号）。注意 ⑫ 已被别的会话收严成「算不出的一律执行前拒」，所以「报」现在是退 2：

| 例 | 命令形状 | 退出码 |
|---|---|---|
| own | `T=$(mktemp -d <草稿目录>/x.XXXXXX); echo x > $T/a.md` | 0 |
| own-inline | `echo x > $(mktemp -d <草稿目录>/x.XXXXXX)/a.md` | 0 |
| tmp | 模板在 `/tmp` | 2 |
| other-agent | 同形，agent_id 换成别人 | 2 |
| other-session | 同形，会话号不是 `.owner` 里那个 | 2 |
| dash-p | `mktemp -d -p <草稿目录> x.XXXXXX` | 2（已记 ㉕） |
| tmpdir | `--tmpdir=<草稿目录>` | 2（已记 ㉕） |
| nested | `$(cd $(mktemp -d …) && pwd -P)` | 2（已记 ㉕） |
| file-mktemp | 不带 `-d` | 2 |

再用这一趟自己的真身份（会话 412f440a…、我的 agent_id、真主会话记录，没有认领文件、靠派发回执）喂：模板在本草稿目录 → 退 0；模板在别的目录 → 退 2。说明没有认领文件的 Bash 专用腿（三方腿只有 Read 和 Bash，不会触发写闸建认领文件）走的是回执这一支，这一支在真场景里通。

自证：`bash .claude/hooks/bash-command-detector.sh --selftest` 退 0，764 种。`BASH_COMMAND_DETECTOR_BREAK=mktemp-scratch-template-unknown` 红 5 格，`=mktemp-template-owner-unchecked` 红 3 格，与文件头点名的「5 格」「3 格」一致。

判定：做到。小处：自证里这几格的标签仍写「不记／记」，实际现在是「放行／拒」，是别的会话收严 ⑫ 之后留的措辞，不影响判。
推翻条件：子 agent 在自己草稿目录里 `$(mktemp -d <草稿目录>/x.XXXXXX)` 仍被报算不出；或别的 agent 的目录、`/tmp` 下的模板被放行。

## ⑱ 取基准取不到的两处（九十节 1489）

行的要求：两处逐处核用途，「与基准比」的改调 `baseline_text`、取不到判红、各加一份「基准取不到」样本（`records/2026-09-16-subagent拆分提案.md:1489`）。

改法落点：
- `.claude/gate.d/checker-independence-and-sync.sh:678` 的 `baseline_text` 改调库（`lag_baseline.baseline_text`，682 行），取不到时记进 `baseline_problems`、返回 None，不再交空串；
- `.claude/gate.d/doc-decisions.sh:3383` 的 `base_ledger_text`（3393 行调库）与 history-loss 格改起库的 `show`；
- 样本 `layout-checker-sync-base-unreadable-red`、`history-loss-base-unreadable-red`。

核：`stage-selftest.sh --item checker-independence-and-sync.sh` 16 个用例判得对（含 `layout-checker-sync-base-unreadable-red`）；`--item doc-decisions.sh` 53 个用例判得对（含 `history-loss-base-unreadable-red`、`history-loss-green`、`history-loss-red`）。
判定：做到。核用途的结论（两处都是「与基准比」）写在交回报告里，我读了代码，一致。
推翻条件：基准那一版读不出（对象库缺那一份）时这两格仍绿或退 77；或基准里没有这份（新加的）时反而红。

## ⑲ 派发闸点名对齐、复跑表、主会话按时刻筛（九十节 1490）

行的要求有三件：19 条点名对齐失败行；复跑表加派发闸（120 秒）与 `subagent_transcript.py`（60 秒）；`agent-cost.py` 主会话按每条记录时刻筛（`records/2026-09-16-subagent拆分提案.md:1490`）。

核：
- 点名：`fix/replay2.py` 用 `code-tooling.sh` 里原样取出的开关表读法（`fix/bsl.py`），逐条设上跑自证：`hook runner-dispatch-guard.sh switches 98 bad 0`、`hook handback-guard.sh switches 19 bad 0`、`hook subagent_transcript.py switches 2 bad 0`；点名的失败行原文都在失败行里、都退 1。
- 复跑表：`.claude/gate.d/break-switch-replay-files.tsv:8`（派发闸 120）与 `.claude/gate.d/break-switch-replay-files.tsv:15`（`subagent_transcript.py` 60）。`stage-selftest.sh --item code-tooling.sh` 判得对 `break-switch-replay-green`、`break-switch-replay-red`。
- 主会话按时刻筛：`agent-cost.py usage|dispatch|timing|outcome --since 2026-10-23 --until 2026-10-23` 四个子命令都接受 `--until`、退 0；`dispatch` 的标题行写「主会话按每条记录的时刻筛」。

判定：做到。门禁 break-switch-replay 一格本身我没跑（扫仓的门禁阶段只许提交时跑），用同一份读法的复算代替。
推翻条件：门禁 code-tooling 的 break-switch-replay 一格复跑这几份时有一条记成「不符」「没红」「超时」或「没点名」。

## ⑳ 派发闸报告行按草稿目录判（九十节 1491）

行的要求有四件（`records/2026-09-16-subagent拆分提案.md:1491`）：⑰ 与报告形态只看落在本提示「草稿目录：」下的路径与裸文件名；三方形态对整份提示里不在引号里的 `research/prompts/<名>-output.md` 也认；放行 3 份转述、拒 4 份逗号后真报告行、放行 27 份「交付：报告」写法；自证拿第三轮攻方 `research/prompts/landing-audit-r3-opus-model/` 里那几份真提示当样本。

改法落点：`REPORT_SCRATCH_MENTION`（`.claude/hooks/runner-dispatch-guard.sh:1310`）、`report_scratch_directories`（`.claude/hooks/runner-dispatch-guard.sh:1361`）、`own_report_token`（`.claude/hooks/runner-dispatch-guard.sh:1375`）、`report_line_mentions`（`.claude/hooks/runner-dispatch-guard.sh:1383`）、`THREE_WAY_REPORT_MENTION`（`.claude/hooks/runner-dispatch-guard.sh:1462`）。

核：把 `research/prompts/landing-audit-r3-opus-model/prompts/` 里的真提示逐份喂 `bash .claude/hooks/runner-dispatch-guard.sh --check-prompt <文件> --type <类型> --root <仓根>`，原样退出码：

| 组 | 提示 | 期望 | 实测 |
|---|---|---|---|
| 转述别人报告，自己的报告行改成 result.md | `a4737c6f3660f4f7e-own-result`、`aa9ec3423c1f54bba-own-result` | 放 | 0、0（第三份 `a28fe10b58d88beec-own-result` 在目录里没有，我没造） |
| 同三份原样（报告行写 report.md） | `a28fe10b58d88beec`、`a4737c6f3660f4f7e`、`aa9ec3423c1f54bba` | 拒「报告文件名」 | 2、2、2，都点「报告文件名」 |
| 逗号后的真报告行 | `a0a2dfee322e26786`、`a789e7091d91430aa`、`a8afb2185cfa840cc`、`a9717cf07fb6e7c21` | 拒 | 2 ×4，都点「报告文件名」 |
| 三方「交付：报告」「节标题冒号」形态 | `a3652270eb80e4fd0`、`-delivery-colon`、`-section-colon`；`a303d33b0ccd5e68f`（工具员） | 放 | 0 ×4 |
| 骨架 | 三种腿的骨架填好 | 放 | 0 ×3；报告行改成草稿目录下的 `result.md` 退 2 ×3 |

那 27 份真提示是从主会话记录里取的，我没有它们，所以「27 份放行」没复跑；上表「交付：报告」那几份是同一写法。自证：`runner-dispatch-guard.sh --selftest` 退 0，483 种。

判定：一半。前三件做到。第四件「自证拿真提示当样本」没做到字面意思：钩子的自证里没有任何一格读 `research/prompts/landing-audit-r3-opus-model/`（`grep -c 'a9717cf07fb6e7c21\|opus-model'` 在钩子里命中的两处是别的内容），新加的格是照那几份的写法仿写的（例如「报告名:句中逗号之后、落在本提示草稿目录下的报告标签照认」，见 `.claude/hooks/runner-dispatch-guard.sh:2720`），真提示的回放只在工具员草稿目录里的一次性脚本里做过，不在仓里、门禁不复跑。仿写不依赖会被归档删掉的快照目录，这是它的好处；代价是真提示的形状变了，自证看不见。
推翻条件：把上表任一份真提示喂当前的派发闸，退出码与「实测」列不同；或自证里加一格读真提示而它红了。

## ㉑ 派发闸 ⑰ 加判三个前缀（九十节 1492）

行的要求：「⑰ 加判这三个前缀开头的 `.md` 报告名」（`records/2026-09-16-subagent拆分提案.md:1492`），共用约束那一句照实测不改。

改法落点：`REPORT_REFUSED_PREFIX_NAME = re.compile(r"(?i:summary|findings|analysis)[^/]*\.md")`（`.claude/hooks/runner-dispatch-guard.sh:1322`），判在 1432 行；开关 `report-name-prefixes-unchecked`。共用约束那一句没动：这一批的交回报告都没碰 `.claude/agent-common.md:110`；它相对 HEAD 的差（句内一个逗号、加「报告文件名里不带 report」一句）来自第七批 `research/prompts/subagent-landing-audit-2026-10-22/r2fix-definitions.md`，不属这一批。

我造的提示（clerk 骨架，只换报告行的文件名；`--check-prompt`）：

| 报告文件名 | 退出码 |
|---|---|
| `result.md` | 0 |
| `summary.md` | 2，点「报告文件名」 |
| `SUMMARY.md` | 2 |
| `FINDINGS-x.md` | 2 |
| `analysis-1.md` | 2 |
| `x-summary.md` | 0 |
| `summary.txt` | 0 |
| `SUMMARY.MD`（扩展名大写） | 0 |
| `report.MD`（扩展名大写） | 2 |

判定：做到。边角：新加的前缀判只认小写 `.md`，所以 `SUMMARY.MD` 放过，而原有的 report 那一判大小写都拒（`report.MD` 退 2）；交回报告自己写了「`.md` 只认小写：探针只测了小写 `.md`」，工具层对大写扩展名拦不拦没测（`research/prompts/subagent-landing-audit-2026-10-22/b9-prefix-probe.md` 的「没做什么」也写了没测混合大小写）。
推翻条件：草稿目录下 `summary*.md` 之类被放行；或 `x-summary.md`、`summary.txt` 被拒。

## ㉒ 结局账落盘与时机（九十节 1493）

行的要求：定一个时机跑闭窗口 `outcome --summary`，「输出落进那一轮的调度记录目录；主 agent 定义「一轮怎么开、怎么收」第 8 条点名它」（`records/2026-09-16-subagent拆分提案.md:1493`）。

改法落点：
- 脚本一侧：`agent-cost.py outcome --out <文件>`，用法在文件头 10 行，行为说明在 13 到 15 行（排他新建、文件已在就拒退 2、写完回读逐字相同才退 0；每批收尾落账的写法在 15 行）；自检 `selftest_outcome_out`（`scripts/agents/agent-cost.py:1604`）；开关 `outcome-out-clobber`（`scripts/agents/agent-cost.py:69`）。
- 定义一侧：`.claude/main-agent.md:27` 第 8 条，「`--out` 只排他新建」写进去了，命令是 `outcome --since <开工日期> --until <这一天> --summary --out records/<轮>-outcome-<这一天>.txt`。

我跑的（草稿目录里，经内存包装）：
- 第一次 `outcome --since 2026-10-23 --until 2026-10-23 --summary --out <新文件>` 退 0，成功句末写「这份输出也落进了 …（排他新建，回读与标准输出逐字相同）」；
- 同一路径再跑一次：退 2，「✗ --out 指的文件已经在：…（结局账只排他新建，不覆盖已落的账）」；
- 在仓根相对路径 `records/r4-outcome-2026-10-23.txt`（相对草稿目录里的 `records/`）写：退 0，文件里没有 `/home/` 绝对路径；
- `usage`、`dispatch`、`timing`、`outcome` 四个子命令带 `--until` 都退 0。

这一行留的口子「`records/` 下放 `.txt` 门禁认不认没核，进 landing-audit-r4」（`records/2026-09-16-subagent拆分提案.md:1493`），我读出来的结果：
- 扫 `records/` 的格只取 `.md`：门禁 doc-text 的 account-quota 与 date-once 格里 `find records .claude/kb -name '*.md'`（`.claude/gate.d/doc-text.sh:743`）和 `records/*.md) ;;`（`.claude/gate.d/doc-text.sh:749`），射程函数 `path.endswith(".md")`（`.claude/gate.d/doc-text.sh:1102`）；`.claude/rules/directory-layout.md` 里没有提 `records/`（`grep -n records` 零命中）；没有一格点名 `records/` 下的 `.txt` 并拒它。
- 全仓文本格（prime-marks、clock-times、private-values 等）按「git ls-files 列出的每份文本文件」逐行扫，射程说明在 `.claude/gate.d/doc-text.sh:30` 起，所以这个 `.txt` 照样被扫钟点和角标；我这份输出里钟点形态零命中，时区词表是 `.claude/gate.d/lib-forbidden-notations.py:84` 的 `ZONE_WORDS`（两个大写缩写），输出里的「Asia/Tokyo」不在其内。
- 没有一格核「收拢表引了这份账」。
- 我没法起门禁阶段本身来实测这一条（扫仓的阶段只许提交时跑），上面是读代码得出的。
判定：做到（脚本一侧、定义一侧都在）；「.txt 门禁认不认」：不拒，不扫 `records/` 专用的两格，全仓文本格会扫。
推翻条件：提交时门禁因 `records/<轮>-outcome-<日期>.txt` 红；或 `--out` 覆盖了已有文件。

## ㉓ 合入后验证加 checker 档快档一步（九十节 1494）

行的要求：`.claude/main-agent.md` 验证段与 `scripts/gate/merge-and-verify.sh` 加一步：带 `SINGLEFS_HEAVY_TESTS=user-request` 只跑单元键对不上的 checker 档快档测试，红了进写段下一批（`records/2026-09-16-subagent拆分提案.md:1494`）；去向列自己记着三处口子进这一轮（主 agent 定义第 3 条没写例外、`--copy` / `--peer` 报本次未跑、闭包外被测试读的文件认不出）。

改法落点：
- 脚本：`checker_verdict`（`scripts/gate/merge-and-verify.sh:457`，判补丁碰没碰 checker 档包与它的依赖闭包）、`checker_quick_step`（`scripts/gate/merge-and-verify.sh:667`，起 `checker-tier-crash-point-replay.sh --check crash-case-replay,feature-gated-tests`，键与复用由那一道算）；文件头说明从 99 行起，`--peer` 与 `--copy` 的两种「本次未跑」在 108 行。
- 定义：验证段第 1 步「没带前缀这一步拒、整条退非 0」（`.claude/main-agent.md:52`）。

核：
- `bash scripts/gate/merge-and-verify.sh --selftest` 退 0，「查了 45 种」，成功句里含「没带 SINGLEFS_HEAVY_TESTS 时不起并给出路、补丁没碰 checker 档包与它的依赖闭包时报本次未跑、那一道退 3 只在样本根上算过、--copy 与 --peer 时报本次未跑」。
- 7 个新开关（`checker-quick-skipped`、`-always-run`、`-prefix-unchecked`、`-dependencies-ignored`、`-sample-exit-anywhere`、`-red-ignored`、`-elsewhere-run`）逐个设上跑自证，都退 1（红的格数 7、2、1、1、1、2，最后一个单独跑见「不对 0」；我的比对认不了文件头的「……」，所以只证红，没逐项核点名）。
- 那一道的两格确实存在：`.claude/gate.d/checker-tier-crash-point-replay.sh` 的 `# gate-cell:` 登记里有 `crash-case-replay`、`feature-gated-tests`。
- 没真跑一次合入后验证（要 cargo 和补丁，属提交时主 agent 的活）。

判定：一半。脚本一侧与第 52 行做到，红了进写段下一批也写了。口子的现状：
1. 主 agent 定义第 3 条仍是「提交之外任务确实要跑，先弹窗问用户，同意了才跑」（`.claude/main-agent.md:22`），第 52 行却要求每趟合入后验证都带 `SINGLEFS_HEAVY_TESTS=user-request`；用户定案（`research/prompts/landing-audit-r3-main-verification.md@用户定案（2026-10-23，弹窗）`）是授权，但第 3 条没写这一步的例外：没改，口子还在。
2. `--peer`、`--copy` 报本次未跑：设计如此，口子还在（只是知道了）。
3. 闭包外被测试读的文件：判闭包靠成员 `Cargo.toml` 的依赖表（`scripts/gate/merge-and-verify.sh:457` 一带），读 `crates/` 外文件的测试认不出：没改。
这三处行自己说「进 landing-audit-r4」，不算这一批漏改，但这一行的效果在这三处之内不是完整的。
推翻条件：补丁改了快档测试函数闭包里的函数，这一步却报本次未跑；或补丁只改闭包外的包，这一步却起了那一道。

## ㉔ 三处小尾巴（九十节 1495）

行的要求：文件头改成四元键；doc-registries 两格把「起不来」判红并加样本；`.claude/agents/implementation-writer.md` 第 4 步列举补命名检查（`records/2026-09-16-subagent拆分提案.md:1495`）。

1. 四元键：「格拿它与 gate 基准那一版里复跑写 none: 的行比（按（实验号，登记出处，单元名，留存产物名）四样一起当键」（`.claude/gate.d/replay-comparison-units.tsv:9`）；格里的键函数 `none_key`（`.claude/gate.d/doc-experiments.sh:2478`），开关 `comparison-none-key-pair` 退回二元（`.claude/gate.d/doc-experiments.sh:96`）。`-rekey-red` 样本判得对。
2. doc-registries：样本 `kb-directories-unreadable-red`，setup 往 `.claude/kb-directories` 写一个不是 UTF-8 的字节，期望退 1，且 `inline-sums`、`invariant-count-elsewhere` 两格各「没判：kb 目录登记位读不出」并「红（退出 1）」。`stage-selftest.sh --item doc-registries.sh`：51 个用例判得对，含这一份。
3. 第 4 步：「共用约束「门禁」一节「命名检查交回前跑」那两条命名检查」（`.claude/agents/implementation-writer.md:40`）进了列举，同一行后面的例外改成「那两条命名检查除外」，与列举指同一样；`git show HEAD:` 的同一行没有这两处。

判定：做到。
推翻条件：`.claude/kb-directories` 读不出时两格仍退 77；或第 4 步的列举与例外又不指同一样。

## K1 第 3 条：「推的、没量过」有没有当成量的写进别处

| 项 | 交回报告怎么说 | 写进了哪里 | 有没有写成量 |
|---|---|---|---|
| ⑬ 的 512 MiB | 推的、没量过 | `scripts/agents/watch.conf:25`（「512 本身是推的，没量过」）、`scripts/agents/agent-watch.py:6060`（选项帮助）、九十节 1484 去向列、调度记录任务 29 一行（`records/2026-10-22-子agent结果落地审计.md:240`） | 没有；peaks 表数（不到 8 MiB 的 2787 条、不到 512 MiB 的 19163 条）是当时读的数，我现读是 2826 与 19586，同量级 |
| ⑯ 加键后定义照常派出 | 按先例推的，没真派 | 九十节 1487 去向列（「加键之后还没真派过一次三方腿，landing-audit-r4 开轮时顺带核」）、b9-leg-types.md 第 1 节 | 没有写成量；调度记录任务 30 一行（`records/2026-10-22-子agent结果落地审计.md:243`）写「先例：14 份定义都带 Claude Code 不认的 `required-inputs:`，一直照常派」是先例的事实，没有写「已验证加键可派」。我这一趟没能把它核成（见 ⑯ 第 2 点） |
| 任务 30 的桶变化 | 没在真会话记录上比改前改后 | 调度记录任务 30 一行「真会话记录上桶变多少没量」、b9-leg-types.md「没做什么」 | 没有；`agent-cost.py` 里三方腿桶现按 `leg: three-way` 收，three-way-materials 与已没有定义的类型不再进桶，这个效果没量 |

另一处值得说的：⑭ 的「认的范围」这一批比行里写的宽，宽了多少没有量过，只在自检里造了进程树。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| ⑥ | 做到 | 七例输入里该拒的拒、该放的放；mv/cp/重定向写的 `.rs`、命令替换的根是九十节记着的口子 |
| ⑦ | 做到 | 四例：缺节拒、写了放、只在别节提拒、行文里写一句不算 |
| ⑧ | 做到 | 样本判得对；用同一份库复算真仓两个登记目录下一层与子目录的 `.md` 都盖住 |
| ⑨ | 做到 | claim 自证与孤儿格样本都过，开关红在「目录登记」一行；真仓 `--next` 为 E198 |
| ⑩ | 本批不在范围 | 上游，交发版会话 |
| ⑪ | 本批不在范围 | 第七批已做，读法还在 |
| ⑫ | 做到 | 登记表认顶层 `lib-` 库；`lib-forbidden-notations.py` 现被拒，㉖ 已记 |
| ⑬ | 做到 | 探针：峰值 0 走固定线、小峰值取 512 MiB、400 MiB 起照乘；512 是推的，已写明；影响约 57% 的键 |
| ⑭ | 做到 | 自检真起进程树，不报的不报、该报的照报；范围比行里写的宽一点 |
| ⑮ | 做到 | 我独立重算：19 行表，14 对上登记、#15 到 #19 是点名不登记的 |
| ⑯ | 一半 | 四处读库做到；真派一次没核成；同三个名字在线程上限与预计挂钟的表里仍手抄 |
| ⑰ | 做到 | 本草稿目录放行，别处、别的 agent、`/tmp`、`-p` 与 `--tmpdir` 都拒；真身份走回执这一支通 |
| ⑱ | 做到 | 两处都改调库，取不到判红，两份样本判得对 |
| ⑲ | 做到 | 98、19、2 条开关点名全对；复跑表两行在；`--until` 四个子命令都接受 |
| ⑳ | 一半 | 真提示放行拒绝都对；「自证拿真提示当样本」没做到字面，自证里是仿写 |
| ㉑ | 做到 | 三个前缀拒，`x-summary.md`、`summary.txt` 放；大写扩展名 `SUMMARY.MD` 放过（没测过） |
| ㉒ | 做到 | 脚本与第 8 条都在；records 下 `.txt` 不拒、专用两格不扫、全仓文本格扫 |
| ㉓ | 一半 | 脚本与第 52 行做到；第 3 条的例外句没写，另两处口子也还在 |
| ㉔ | 做到 | 四元键、两格判红加样本、第 4 步列举补命名检查，三处都在 |

## 没做什么

- 没起门禁阶段本身（扫仓的阶段、break-switch-replay 一格、kb-directories-write-scope 一格在真仓上），只跑了样本档 `stage-selftest.sh --item` 和复算；真仓读数写明是复算。
- 没真跑合入后验证，没编译，没跑重型测试，没跑任何 checker 档。
- 没验 ⑯ 加键后真派三方腿（见 ⑯）；没验 ⑭ 在整趟门禁里的表现；没验 ⑳ 的 27 份「交付：报告」真提示（只有三种形态的代表）。
- 没判 K2、K3（攻方的格），也没替主 agent 采纳或出判决；上面「一半」只是改法与那一行的对照。
- 没做模型，没有 `tools/rerun.sh`；探针脚本在草稿目录 `fix/` 下，sha256 在开头。
- 清理：草稿目录 `~/.local/share/singlefs/work/landing-audit-r4-sonnet/` 里自己建的 `repo/`（钩子拷贝加指回主工作区的符号链接，`du -sh` 6.2M）、`lockdir/`（12K，造检出钩子样本用的假写者锁）、`split/`（1.9M，快照 diff 的拆分）已删，符号链接的目标没动；留下 `fix/`（探针与造出的输入）、`out/`、`outcome/`、`prompts/` 共 572K，没有仓副本与编译目录。
