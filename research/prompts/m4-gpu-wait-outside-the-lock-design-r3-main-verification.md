# m4-gpu-wait-outside-the-lock-design-r3：判决（2026-11-18）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 -->

正文 `research/prompts/_m4-gpu-wait-outside-the-lock-design-r3-body.md`，开工快照 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-snapshot/`。这是这一题的第三轮（设计推论轮的最终轮），前两轮判决 `research/prompts/m4-gpu-wait-outside-the-lock-design-r1-main-verification.md`、`research/prompts/m4-gpu-wait-outside-the-lock-design-r2-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端正推（Sonnet） | `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-sonnet-output.md` | Q1 前提原样不成立（隔离记号），补成比整条记录加重取隔离集后八条路径读码没找到反例；岔路单十一行；用户定的两条在今天的 `crates/` 上都做得成，先决三处 |
| 云端攻方（Opus） | `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-opus-output.md` | Q1 字面打中（同一处隔离记号），后果被发布路径兜住；回锁不重判回退门打中；签发的量照哪一版取没写、照票取 I-5.6 红；Q5 没打中；「读最坏等 1.10 秒」撑不住 |

核查：

- 引文：任务 19856 `python3 scripts/agents/cite-check.py research/prompts/m4-gpu-wait-outside-the-lock-design-r3-sonnet-output.md research/prompts/m4-gpu-wait-outside-the-lock-design-r3-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_m4-gpu-wait-outside-the-lock-design-r3-background.md`，末行「✓ 核了 85 处引文、109 处裸行号，对上 85 处，没判 109 处（其中裸行号没有引文可比 108 处；逐处列在上面）」。快照里八份 `crates/` 文件写判决之前 `sha256sum -c` 全对。裸行号没逐处核。
- 攻方探针：任务 19778（第三次重排后退 0），`~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r3-opus-run/out/issue-after-the-window.out`：`RESULT name=issue_after_the_window arm=ticket histories=16 landed=1 issue_published=16 i56_red_after_issue=1 i56_red_where_landed=1 i56_red_where_not_landed=0 other_red_after_issue=0 setup_failed=0 panicked=0`；`RESULT name=issue_after_the_window arm=current histories=16 landed=1 issue_published=16 i56_red_after_issue=0 i56_red_where_landed=0 i56_red_where_not_landed=0 other_red_after_issue=0 setup_failed=0 panicked=0`；`SELF name=self_checks checked=3 failed=0`。16 段历史里锁外期间有用户数据落进选中那一段的只有 1 段。
- 正推点名的现成用例：任务 19777，`test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out`（释放前读盘核没过时记录留在已分配）。
- 忙窗口补量：任务 19776 **退 1**。30 趟都跑完、30 行 `name=busy_window_foreground_wait` 在 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r3-sonnet-run/out/`，脚本末尾数行数那一步打的是 `name=rerun_done busy_rows=0 expected_rows=30 failures=0`（数成 0），所以判失败。这一组只当线索，不当结局。线索里看得到的：
  - 四个读线程、没有写线程，被挡的取样最坏等（微秒，每档三趟）：替身睡 0 毫秒 51075、87150、762954；睡 5 毫秒 209131、272595、312500；睡 50 毫秒 898635、1030942、1447362；CPU 路径 78924、238506、264947。同一档三趟之间差到十几倍。
  - 加一个写线程，被挡的取样中位（微秒，每档三趟）：睡 5 毫秒 31887、42322、61645；睡 10 毫秒 45072、53769、81583；睡 20 毫秒 98895、124386、127909；睡 50 毫秒 167505、175080、287302。中位随替身的睡眠涨。
  - 模板自证那一条没交。

## 二、判决

三轮合起来判，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」：

| 结论 | 三轮里怎样 | 判 |
|---|---|---|
| 回锁里拿旧一版当发布的底不行 | 第一轮打中，第二轮辩方复核后依据换成 42 段历史（旧底上的签发不落盘、整理白算），第三轮攻方说这 42 段量到的只是签发、射程写宽了 | 形态不再拿旧一版当底，这一句不承重；射程收成「没有活意图时旧底上的签发被拒」 |
| 比 txg 作废再加下限 | 第一轮提出，第二轮两腿打中（下限把卡挂住的那一批带回锁里），第三轮没人翻 | **出局** |
| 三段的骨架、让路只在取输入之前问、回锁之前放 GPU 实现那把锁 | 三轮都没被打穿 | **站住** |
| 切到 `gpu` 的校验不能在会话锁里取 GPU 实现那把锁 | 第一轮量到，后两轮没人翻 | **站住**；怎么改是岔路 |
| 回锁照现行那一版重排计划、不作废 | 第二轮提出，第三轮两腿都打中同一处：只比盘、槽、跨度、分配代漏掉隔离记号。两腿又一致：后果被发布路径拒整批兜住，不写坏盘；正推补成比整条记录加回锁重取隔离集后读码八条路径没找到反例，攻方七种前台路径读码没打中 | 原样**打中**；补过的形态两腿都没打中，但它是第三轮里新成形的，**零轮** |
| 签发那一支的量照哪一版取 | 第三轮攻方打中、探针两臂量到（照票取 1 段 I-5.6 红，照现行那一版取 0 段） | 采「照现行那一版取」；取样只有 16 段、落进选中段的只有 1 段 |
| 回锁那一次发布要重判回退门 | 第三轮攻方打中（推的），正推没覆盖 | 只有一条腿：记线索，进形态，零轮 |
| 崩溃语义 | 第一轮严判据下没打中；第三轮攻方读码没打中、没枚举，正推「推不出」（缺现有崩溃流罩不罩得到「旧版分类、新版发布、子集计划」的枚举计数） | **没定**：两条腿不一致，不算都没打中。实现时照 `.claude/rules/implementation-workflow.md` 代码轮检查单「新崩溃流多罩了什么」那一格答 |
| 今天锁内等的代价 | 第二轮一趟读数「睡 50 毫秒读最坏等 1.10 秒」；第三轮两腿都说它不能当真卡的数（替身睡的比 E185（GC一批搬迁的挂钟拆分） 真卡一次往返大 4 到 10 倍） | 第二轮判决「今天锁内等的代价量到了」那一句**收窄**：量到的是「前台被挡的时长随 GPU 一次调用的时长涨」这个方向（补量线索里加写线程那一组的中位从睡 5 毫秒的 32 到 62 毫秒涨到睡 50 毫秒的 168 到 287 毫秒）；真卡上的绝对值没量。替身睡 0 毫秒与 CPU 路径最坏也有 51 到 763 毫秒，锁里的 CPU 活占一大块，拆 GPU 等拿不走这一块 |
| 缺口 2（放锁后前台抢不到） | 第一轮撤回，第二轮改成一半成立 | 维持第二轮：被连着越过 4 次以上有，一拍 64 次没出现 |
| 卸载等不等 | 前两轮写成候选；第三轮正推指出用户 2026-10-28 已定「卸载不等」（`.claude/kb/decisions/26-后台整理与放置回收.md` 第 269 行），今天的运行库没做到 | 不是岔路：照已定的做，改 GPU 包里等的那一处查停标记。过期 2 的判法随之改：条款没错，是运行库没做到 |

## 三、改法

最终形态（十四条，实现员照它落；除标「量过」的都是零轮，写进产品代码之前先在探针里做 A/B）：

1. 核心层后台入口拆三段：锁里取输入、锁外算、回锁里发布；GPU 与 CPU 两条路径都走；会话上记「有一批在锁外」，取输入第一步就判、在取 GPU 锁与换设置之前拒第二个后台调用者。
2. 取输入：问让路、排空待发、`begin_a_batch`、现行那一版拷 `Arc` 指针。
3. 锁外算完先放 GPU 实现那把锁再回会话锁（**量过**，任务 19650）。
4. 回锁里出错先于一切。
5. 回锁里不作废：先排空待发，重取隔离集；有活意图那一支比起点在 R 里的未释放记录的整条记录与集合相等，变了的单元从计划里剔掉，其余在现行那一版上发布；剔完一个不剩不发。A/B **量过**（任务 19924，探针 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-ab-model/`，四臂、四种前台改动、17 段历史一臂，自证 `SELF checked=48 failed=0`）：`RESULT arm=replan_pruning histories=17 published=11 batches_with_units_moved=11 units_moved=18 units_pruned=16 refused_by_the_publish_path=0 checker_red_histories=0 readback_bad_histories=0 setup_failed=0 panicked=0`；`RESULT arm=drop_when_the_txg_moved histories=17 published=3 batches_with_units_moved=3 units_moved=6 units_pruned=0 refused_by_the_publish_path=0 checker_red_histories=0 readback_bad_histories=0 setup_failed=0 panicked=0`；`RESULT arm=recompute_in_the_lock_as_today histories=17 published=17 batches_with_units_moved=11 units_moved=18 units_pruned=0 refused_by_the_publish_path=0 checker_red_histories=0 readback_bad_histories=0 setup_failed=0 panicked=0`；`RESULT arm=replan_without_pruning histories=17 published=7 batches_with_units_moved=7 units_moved=14 units_pruned=0 refused_by_the_publish_path=10 checker_red_histories=0 readback_bad_histories=0 setup_failed=0 panicked=0`。回锁重排搬的单元数与今天锁内重算相同（18 对 18），txg 变了就丢只搬 6 个；不剔的那一臂 10 批被发布路径拒。今天锁内重算多出的 6 次发布不搬单元（覆盖写与删文件把计划里的单元都释放掉之后的那几批），回锁重排在这几段不发、留给下一批重扫。没量的：隔离记号那一格（没注入坏块）、回退复活与快照。
6. 签发那一支：回锁里对选中的段在现行那一版上重判资格与签发前那一判，各个量照现行那一版取（**量过**，任务 19778，两臂）。
7. 回锁那一次发布重判回退门。A/B **量过**（任务 19922）：`RESULT name=rollback_gate_after_the_window arm=regate histories=2 published=1 gate_closed_before_relock=1 states_after_publish=[5,4] fewer_than_four_states_after_publish=0`；`RESULT name=rollback_gate_after_the_window arm=no_regate histories=2 published=2 gate_closed_before_relock=1 states_after_publish=[5,5] fewer_than_four_states_after_publish=0`。门关的那一段重判就不发、不重判就发；不重判那一臂发完之后状态也没少于 4 个，攻方说的违例在这两段历史上没出现。重判是今天每次整理发布之前本来就有的那一判，留着，不算新改法。
8. 回锁时有冻结的发布：这一批丢掉，单独的停因。
9. 让路：任何前台入口等会话锁都让，计数在锁外、`try_lock` 失败才登记，问答与停因各加一个成员，破例批不让；连着让路 N 次后下一批不让，N 一批发成才清零。新停因进运行库 `keeps_going` 的停下名单。
10. 维护线程自己的 `publish_when_due` 与重发冻结在锁外那一段期间照跑。
11. 切到 `gpu` 的校验不取 GPU 实现那把锁。
12. GPU 把手锁中毒交回错误。
13. 卸载与 Drop 不等锁外那一批：GPU 包里等的那一处查停标记，那一次提交留在卡上、那张卡整张泄漏（用户 2026-10-28 已定）。
14. 观测：让路、剔掉了几个单元、没资格换段、冻结丢批，各有计数或停因成员。

## 交用户的

岔路单 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-forks.md`。要用户定的四条：

1. 做不做：最终形态相对今天多出六个状态、八个分支、一到三个参数（正推腿数的），换到的是卡挂住时前台不再被挂约 40 秒（读码算的）与平常少等 GPU 那一段（真卡上没量；替身上方向量到了）。
2. 读盘放哪：选段循环每圈回锁里读，或盘表与会话分锁（真盘块设备经不经得起读写并发没人读过）。
3. 锁外那一段含多少：只含 GPU 或 CPU 核的调用，或把建索引、重扫、算计划这些纯计算也挪出去（替身睡 0 毫秒最坏也等 51 到 763 毫秒，线索）。
4. 让路的下限 N 取次数还是累计时长。

其余岔路（切换校验的三候选、冻结时丢批还是先重发、签发没资格记不记、维护线程自己的发布怎么照跑）主 agent 取默认写进实现条款，判决里标零轮。

## 这一轮被判过的文件

- crates/singlefs-mount-runtime/src/lib.rs：判了卸载 join 与用户已定的「卸载不等」相左、`keeps_going` 要认新停因、忙窗口补量量的是它今天的行为。
- crates/singlefs-core/src/mounted_session.rs：判了回锁里比什么、签发的量照哪一版取、回退门要重判。
- crates/singlefs-core/src/background_compute.rs：判了切换校验取 GPU 实现那把锁是先决要改的一处。
- crates/singlefs-core/src/compaction_plan.rs：判了新停因与问答成员、备忘照哪一版记。
- crates/singlefs-core/src/transaction.rs：判了发布路径逐条核计划兜得住剔漏的单元（拒整批、不写坏盘）。
- crates/singlefs-core/src/block_device.rs：判了盘表分锁要的读写并发两轮都没读过真盘实现。
- crates/singlefs-core/src/region_scan.rs：判了重扫的分类在记录不变时照旧成立的八条路径（读码）。
- crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs：判了等的那一处要查停标记，卸载不等与维护线程自己的发布是同一处改动。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D24（后台重活能不能卸给 GPU） 已定项 2 | 推翻 | 2026-11-18 改了：射程里「第一版一批之内持锁等 GPU，里程碑六拆」那一句待用户定「做不做」之后写回 |
| D26（后台整理与放置回收） 已定项 13 | 推翻 | 2026-11-18 改了：第 6 条让路范围、第 5 条破例那一句待写回；第 4 条「卸载时不等还在跑的提交」不改，运行库照它改 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-11-18 不受影响：回退门照旧守，回锁那一次发布重判 |
