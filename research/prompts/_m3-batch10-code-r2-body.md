# 根环实现第 10 批：代码轮第二轮正文（2026-10-05）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 Q3 Q4 rr2 G3 G4 -->

## 一、题面

第 10 批代码轮第一轮判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第三节与第六节交出的五条，由第 10g 批补丁改进 `crates/`（交回与主 agent 的判定在 `records/2026-09-30-里程碑三开工第一轮.md` 第 10g 批那一条）。这一轮攻的是 10g 合入之后的代码与它的测试；第一轮已攻过的角度（第一轮判决第二节）不重复。岔路单 `research/prompts/m3-batch10-code-r1-forks.md` 四行用户都已定：Q1、Q2 照今天，Q3 另起拒因、单子加一行，Q4 采用 rr2。

改动全文在 `research/prompts/_m3-batch10-code-r2-diff.md`，实现员报告 `/tmp/claude-1000/m3-r1/batch10g/report.md`，冻结副本 `/tmp/claude-1000/m3-batch10-code-r2/tree/`：

| # | 改动 | 主要落点 | 压着的条款或定案 |
|---|---|---|---|
| A | Q3：重建分配器读根环重读那一遍整个环仍一条不被抛弃的自证有效根都没有，报新成员 `MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`；挂载透明的单子加一行，镜像上复算「按最新那条根的实例表，环里一条不被抛弃的自证有效根都没有」 | `crates/singlefs-core/src/mount.rs`，`crates/singlefs-harness/src/mount_transparency.rs`，`crates/singlefs-checker-tier/src/crash_injection.rs` 第三截 | `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14；用户 2026-10-05 定 Q3 |
| B | Q4 rr2：重建分配器读根环第一遍有一块盘一条有效根都没读出就整环重读，重读之前调钩子；根环表、回收门槛与影子账用重读的读数 | `mount.rs` 的 `rebuilt_allocator` | 用户 2026-10-05 定 rr2、甲二「重读一次再拒」 |
| C | 影子账候选与 I-3.1 丁-defer 门槛退回「环里最旧有效根」的两处覆盖：各一条用例、一行变异 | 新测试 `shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs` | `.claude/kb/decisions/16-发布语义.md` 已定项 1「环里最旧有效根」；用户 2026-10-04 定 RGall |
| D | 第 927 行变异改按第三类（取样点不敏感）留档；过时注释 | `crates/mutations.tsv`、`mount.rs` | `.claude/rules/mutation-sampling.md` |

## 二、实现今天的样子（主 agent 的观测，2026-10-05）

- 冻结副本取自 10g 合入之后的主工作区；删除与并发那条线第 21 批已合入。被判文件的 sha256 在 `research/prompts/m3-batch10-code-r2-snapshot/crates-sha256.txt`。
- 合入后验证（harness 档全部测试二进制、fmt、clippy）的日志在 `/tmp/claude-1000/merge-and-verify/` 下，目录名写进调度记录第 10g 批合入那一条。
- 实现员交主 agent 的三处读法（腿判对不对、代价多大，主 agent 量过之后交用户）：① rr2 照字面让新池与年轻的环每次可写挂载都整环重读一次（只有 mkfs 那条根、落在一块盘上），`the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread` 改钉 `OnTheOneReread`；② 第一遍一块盘读出、重读那一遍整个环一条都没有（第二遍比第一遍坏），现在报 Q3 那一拒，rr2 之前它照第一遍的读数走 m 取不到那一支；③ Q3 复算在最新那条根的实例表读不出时取「不按表滤」。
- 已知没做、不在这一轮被判范围的（腿报出来不算打中，判决照原去向写）：第一轮判决「回看决策」与第五节列的欠账（I-7.9 盲带、影子账抛弃判定与择根不是同一次读、进程内重读读出被抛弃的根不隔离就丢、GPU 偏移不相交没断言、记录核对器判据没写进 kb、E139 F10/F11）；钩子不区分调用位置；崩溃注入换恢复策略的开关。
- 用户已认的代价（腿报出来不算打中，只核「代价是不是只有这么大」）：第一轮正文第二节那两条；Q1 照今天（读坏槽卡住上限就在这次挂载里报 ENOSPC）；Q2 照今天不并（F 可能越过住在全零槽里的有效根），E173 第一、二段量过（`.claude/kb/experiments/173-上限卡在读坏槽的三种处置.md`）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　Q3 拒因与它的复算（A）
新拒因只在「重读那一遍整个环仍一条都没有」时交回吗；有没有一条路径把本该报 `Recovery(NoValidRoot)` 的（读阶段择不到根）或别的拒因报成它，或者反过来。镜像上复算的判据与实现判拒的读数是不是同一个定义（被抛弃按哪条根的实例表判、全零与读坏怎么算）：有没有一张没注读故障的合法崩溃镜像让复算为真（挂载透明假红），或一次该拒的挂载复算为假（漏红）。实例表读不出时「不按表滤」那一读法会不会让某一格判反。崩溃注入第三截的计数与报红有没有漏一形。

### K2　rr2（B）与它的两个后果
第一遍「每块盘都读出有效根」的判法与当场放开到哪儿那一步取 m 是不是同一处定义；重读那一遍仍有一块盘没读出时照 m 取不到那一支往下走，回收门槛、影子账、根环表三样用的是不是同一遍读数。重读比第一遍坏时整份换成重读的读数：有没有一段合法历史因此多拒，或让回收门槛越过一条盘上真住着、可回退到的有效根（不安全方向）。新池与年轻的环每次可写挂载都重读一次：除了多一遍读与一次钩子，还改了什么（读序号、钩子次数、checker 档按读序号注故障的装置）；有没有一条用例因此钉错。

### K3　两处覆盖用例（C）
影子账候选「每块盘隔离 14 槽、期望 12」与丁-defer 门槛那一形：用例钉住的是不是条款要的那件事，改回一圈窗口时红是因为门槛还是因为别的；K1 那一形为什么钉不住门槛（txg 24 那一版由记录施加出来、照候选走），换的那一形会不会同样被别的路径掩护。

### K4　checker 档装置（A 的第三截、B 的读序号）
`crash_injection.rs` 第三截新加的计数与报红、`crash_injection_campaign.rs` 改的那几处，与第二截靠判官直接接上那一支是不是同口径；按读序号注读故障的坏盘输入、崩溃注入与 E158、E171 装置，读序号后挪一遍根环之后，原先打中的槽换没换、钉值还对不对（主 agent 跑 checker 档之后把结果补进判决）。

### K5　数能不能核
「隔离 14 槽、期望 12」、攻方 rr2 臂 8 段照常挂上与拿掉 rr2 时 8 条红在 `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`、手工盘面「根环每一槽清 0」那条用例：每一个从哪一段代码推出来、钉它的测试与变异行改回旧值时红不红；有没有一个数只在实现员的临时探针上核过。

### K6　欠账与读法清单全不全
10g 报告里「停下交主 agent 的」与「取了读法的」每一条，在调度记录、`.claude/kb/checks-owed.md` 或判决里都有去向吗；第一轮判决第三节五条是不是都关了。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3、K5、K6 | 逐条从条款或定案原文推到冻结副本里的行，再推到钉它的测试与变异行 |
| 云端攻方（Opus） | K1、K2、K4，重点 K2 | 在冻结副本的拷贝上造反例，只跑 harness 档；不重复第一轮判决第二节已攻过的角度；用户认下的代价只攻「是不是只有这么大」 |

## 五、跑前写死的判据

- 两腿都没打中 ⇒ 站住（一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记）。
- 打中 ⇒ 主 agent 现查坐实，改法进下一批；腿提的收严记「零轮」，交用户或登记实验。
- 第二节列为已知没做、用户已认的几样，腿报出来不算打中，判决照原去向写。
- 第二节实现员交出的三处读法，腿判出哪一读法错或代价比写的大，算打中；判出与写的相同，交用户时附上腿的读数。

交付：腿的报告写 `research/prompts/m3-batch10-code-r2-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-batch10-code-r2-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）；引代码写冻结副本里的行号。方案按 `crates/` 今天的实现来谈。
