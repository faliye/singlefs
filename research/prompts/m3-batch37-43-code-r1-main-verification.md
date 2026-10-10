# 删除线第 37 批 X、第 38 批 Y、第 39 批 Z、第 41–43 批合入的代码：代码轮第一轮判决（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 X1 X2 X3 X4 X5 Y1 Y2 Y3 Y4 Y5 Y6 Z1 Z2 A1 A2 A3 A4 B1 C1 H1 H2 H3 H4 F1 F2 R P Pm V -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文、diff、背景材料 | `research/prompts/_m3-batch37-43-code-r1-body.md`、`_m3-batch37-43-code-r1-diff.md`（4907 行）、`_m3-batch37-43-code-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-batch37-43-code-r1-snapshot/crates-sha256.txt`（23 份）、`kb-sha256.txt`；冻结副本 `~/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/`（第 42、43 批合后） | 腿交齐后冻结副本 `sha256sum -c` 23 份全 OK。主工作区里有两份对不上：`crates/singlefs-core/src/mount.rs` 被根环线 10z 第四块改了（抬 F 那一串加一个参数和 H1 那一圈；hunk `~/.local/share/singlefs/work/m3-r1/batch10z-d/patch/hunks/mount.rs.diff`，主 agent 看过）；`crates/mutations.tsv` 被根环线 10z 第四、五块追加和替换，第 825 行的测试名被主 agent 改了。两条腿引的都是冻结副本（cite-check 带 `--root <冻结副本>`），落在被改那几处的引文 0 处。快照外，主 agent 改了 `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` 两条单测（第四节） |
| 云端正推（Sonnet） | `research/prompts/m3-batch37-43-code-r1-sonnet-output.md`（347 行，sha256 caced9e3…），模型 `…-sonnet-model/` | — |
| 云端攻方（Opus） | `research/prompts/m3-batch37-43-code-r1-opus-output.md`（210 行，sha256 e6b66643…），模型 `…-opus-model/` | — |
| 引文 | 两份腿报告 | `cite-check.py <两份> --root <冻结副本> --root . --background <背景材料>`：「✓ 核了 52 处引文，对上 52 处，没判 0 处」 |
| 主 agent 复跑：攻方 | 模型拷到 `~/.local/share/singlefs/work/m3b3743-opus-model/`，`SINGLEFS_REPO=<冻结副本> SINGLEFS_TOOLS=<仓根> THREADS=15 rerun.sh ~/.local/share/singlefs/work/m3b3743-opus-rerun`，再 `compare.sh <summary.txt>` | 「COMPARE same lines=2324」。原样有 `l0-hit-ow-k5	TALLY kind=ow k=5 root_hidden=false M1=Ok r_units=hit cells=1` 与 `l0-ctrl-ow-k5	TALLY kind=ow k=5 root_hidden=false M1=Ok r_units=- cells=1` |
| 主 agent 复跑：正推 | 模型拷到 `~/.local/share/singlefs/work/m3b3743-sonnet-model/`，跑 `rerun.sh` 的默认三段（static、f2diff、baseline），再 `compare.sh <复跑> <它的 expected>` | 「✓ static.txt 全同（21 行）」「✓ f2diff.txt 全同（8 行）」「✓ baseline.txt 全同（11 行）」；mutations、adhoc 两段没复跑（harness 档 39 行变异各证一遍，归提交时的变异整表复跑） |
| 主 agent 上卡 | 第 42 批副本（主工作区加第 41、42 批）带 `--nocapture`：「test result: ok. 163 passed; 0 failed; 2 ignored」，`NOT_RUN_NO_GPU_ADAPTER` 0 行；`CRASH_JUDGE_GPU_MISMATCH` 15 行，全在五条故意判不了、交回 CPU 的流上，每条流打出的条数与交回 CPU 的状态数逐条相同（日志 `~/.local/share/singlefs/work/m3-batch42/gpu-agree-41-42-nocapture.log`） | Z1、A1、B1 在登记的对拍流上两边同判；第 43 批没动 GPU 一侧 |

两条腿的数都是在腿的副本上量的，不进 kb。

## 二、逐格判决

| 格 | 正推 | 攻方（一次抽样） | 判决 |
|---|---|---|---|
| K1 X1–X5、Y1–Y6 | 一致 | Y1、Y3 没打中：账读得出那一支与映射现算那一支，隔离的槽数逐格相等（12/12、17/17）；F1_AFTER_F2 在 295 格上都拒了，复算全合法 | **采** |
| K2 Y2「映射算不出不拒」 | — | **打中**（三故障）：R 的全部记录、中央映射树根、账上一个节点同时读回全零。M1 落到 P，C 隔离 0 槽，写行那次根落盘之前写掉 R 的 14 份单元，池级 checker 红 I-2.1、I-4.8、I-7.2、I-7.4。ow k=1、3、5 各 4 格，sw3 k=5 共 4 格；层 0 全量打中格 20504 态红，对照格 0 态红 | **坐实**（复跑原样同上）。改法照攻方臂 2：只在系统配置里取过号的实例代号不比 R 新时，照「记录不齐」那一支办，仍不成就拒。攻方量过入库 38 条全过；臂 1 一律拒会让入库 `urJ` 红 |
| K2 Y4 F2_NEWEST | 一致（D23 第 396 行逐句对上代码，三处同一函数） | **打中**（三故障）：R 的单元、较新的系统配置槽、P 的根槽同时读回全零。恢复、只读、可写三处都跳过已确认的 P，落 Pm（4/4 组）。另一形（P 的根槽换成 R 指着的实例表那一片）只有只读挂载落 Pm | **坐实**。改法照攻方臂 `F2_NO_SKIP=1`：退到的那一版与所选根之间，若有一次发布带末条标志的记录读得出、根读不出、不被所选根的实例表抛弃，就不退。攻方量过 38 条全过。`=2` 会让入库那条「实例表读不出只认恰好相等」红，不采。只读挂载那一形不在 `=1` 罩着的范围里，记欠账。GPU 恢复内核同改 |
| K1 Z1、Z2、A1–A4 | 一致；Z1 差分 400000 个小状态 0 不一致，四个弄坏开关都红；钉值 328970 / 71030 算术对，naga 展开数没量 | 读码没打中 | **采**；钉值由合入后 checker 档库单测核（归第 45 批合后那一趟） |
| K1 B1 | 一致（读码） | 读码没打中 | **采**；上卡那一趟 `states_falling_back_to_(1, 3)=3 states_falling_back_to_(1, 4)=0` |
| K1 / K2 C1 | 一致：三样都照正文判，没调实现判拒的函数（grep 零命中） | 没打中新形：630 次拒在置零镜像上复算，C_MAPPING 拒 40 次，20 次合法、20 次判不合法，后者全是「账根以下的节点读不出」那一形（正文已列已知）；F1 590 次全合法；C393 那一成员 0 次，没有报串；二次崩溃层 0 共 41037 态 | **采**。「较新可读根」那一臂没有 harness 用例（正推小项 3），记欠账 |
| K1 变异行 | harness 档 39 行，各证一遍全红；checker 档 9 行锚点各命中一次、没跑 | — | **采**；checker 档 9 行归提交时的变异整表复跑 |
| K3 | 23 份 sha256 全 OK | — | **采** |

正推另报的小项，去向：
1. X2、Y6 的慢档用例钉的是「整个 C 在」，去掉 C 的任一份它们都不红；变异第 3258、3284 行的名字读成「只有记录点名那一份」会读窄。去向：记欠账，两行变异的名字随下一次碰这两行时改准。
2. 第 3273 行的替换文拒的时候报 C393 的成员，不是新成员；能红，不改。
3. 同 C1 那一行。
4. F1_AFTER_F2 退回之后用读阶段扫出的那一份记录，不重扫环；D23 没写读故障下要不要重扫。去向：记欠账。

攻方第五节的观察（恢复、只读、可写三处触发退回的失败不同：R 只有一个数据单元读回全零时，恢复落 P，只读与可写落 R）不算这一轮打中，记欠账另立一题。

## 三、去向

1. 删除线第 45 批（用户 2026-10-08 定「两形都修」）：core 照臂 2 与 `F2_NO_SKIP=1` 改，GPU 恢复内核同改、改钉值，对拍加一条流，harness 各加一条用例和一行变异。
2. 照 `.claude/rules/three-way-inference.md`「第三轮之后停」：C694 / F2 这一题已过两轮设计、三轮代码（m3-batch33t、m3-batch35-36、这一轮），第 45 批不再开代码三方。两条改法只在攻方模型上量过、被攻过零轮，带着零轮进形式化线的冻结树。弹窗写明了这一点。
3. kb：
   - D23 已定项 14 写进两条收严，以及 C1 的镜像复算判据（顺树表指针找账根，只判必要条件）。
   - 新立欠账：
     - Y4 只读挂载那一形；
     - C1「较新可读根」一臂没有用例；
     - X2、Y6 用例只钉整个 C；
     - F1_AFTER_F2 不重扫环；
     - 三处触发退回的失败不同；
     - E158（择根与修复四岔路）第 2 次跑修订第 1、2 条的依据被 C694 改掉（第四节）。
4. 第 45 批 2026-10-08 合入：「✓ 测试全绿（185 个测试二进制）」「✓ fmt 过」「✓ clippy 过」（日志 `~/.local/share/singlefs/work/merge-and-verify/2026-10-08-1037577/`），合后 sha256 `~/.local/share/singlefs/work/m3-batch45/after.sha256`，static-check `CHECKED 3349 320 9`。合后主 agent 清场上卡，带 `--nocapture` 跑 checker 档库单测与对拍文件（`~/.local/share/singlefs/work/m3-batch45/gpu-agree-after-45.log`）：库单测「test result: ok. 219 passed; 0 failed; 2 ignored」，其中 `each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget ... ok`；对拍「test result: ok. 164 passed; 0 failed; 2 ignored」。新流原样 `CRASH_JUDGE_GPU_F2 flow=two-overwrites-second-tree-table-unreadable-first-rotation-zeroed-first-root-unreadable states=69 states_reporting_r_unreadable=8 states_falling_back_to_v_minus=0`。`NOT_RUN_NO_GPU_ADAPTER` 2 行不是真的本次未跑：不要卡的库单测 `a_device_short_of_gpu_memory_on_the_card_lent_is_reported_not_run_like_no_room_now`（`crash_amplification.rs`）自己造了「起设备时」与「900 MiB / [700]」两种结局，被测函数 `print_the_not_run_line_or_refuse` 顺手打出那一行（根环线 2026-10-08 查明，主 agent 先前推成显存抢占是错的）；门禁按子串收会把它们记成本次未跑，归根环线拆开。对拍文件里本次未跑 0 行。`CRASH_JUDGE_GPU_MISMATCH` 15 行，全在故意判不了、交回 CPU 的流上。之后回形式化线「三·3 收住、C692、C694、C695 都合完」。

## 四、快照之外：e158 两条单测

根环线报 `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` 两条内联单测在主工作区红。主 agent 在三份冻结树上二分（`~/.local/share/singlefs/work/e158-bisect/`）：33s 两条 ok；33t 第二条红（C 预隔离护住了被藏那条根的单元）；35-36 两条都红（F1_ANY 第一份读坏去读第二份一次）。两条都是 C694 有意改的行为改掉了装置钉的前提。主 agent 只改两条单测的名字、断言与注释，`crates/mutations.tsv` 第 825 行换测试名；主工作区拷贝上「test result: ok. 95 passed; 0 failed」，第 825 行变异对新测试证红「抓到」。根环线、形式化线看过 hunk 都回了「可以」。这份改动只经主 agent 核，没进两条腿的材料。

## 五、交用户

| 岔路 | 用户定案 |
|---|---|
| 攻方打中的三故障两形修不修 | 2026-10-08「两形都修」（候选：两形都记欠账不修 / 两形都修 / 只修 Y4；弹窗写明都在单故障承诺之外、改法零轮、时间推的） |
| C_MAPPING 那一拒怎么归进 D23 单子（第 41 批停下那一条） | 2026-10-08「乙：core 另起错误成员」 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-08 不受影响：Y2、Y4 两条收严与 C1 的镜像复算判据随第 45 批写回 |
| D22（单元原子性怎么合成） 已定项 8 | 不影响 | 2026-10-08 不受影响：两形都在单故障承诺之外，用户定修，承诺本身不改 |
| D28（挂载期承诺量） 已定项 1 | 不影响 | 2026-10-08 不受影响：这一轮没改隔离集合 |
