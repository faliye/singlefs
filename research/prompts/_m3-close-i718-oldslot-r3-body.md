# m3-close-i718-oldslot-r3：满环时根槽写失败、恢复之前的镜像上 I-7.18 该怎么判，丁甲 / 丁乙 / 丙三选一（推论轮第三轮，最终轮）正文（2026-10-23）

## 一、题面

前两轮判决：`research/prompts/m3-close-i718-oldslot-r1-main-verification.md`、`research/prompts/m3-close-i718-oldslot-r2-main-verification.md`。被判的是：根环装满时一次发布的根槽写落在环里最旧那条根住的槽上、写成 0 或撕裂、进程在重发之前崩，恢复之前那张镜像上 I-7.18（根记录的回收门槛等于现算的门槛）判红；写者照 D16（发布语义） 已定项 1「写失败的槽按旧内容算」没错。这是欠账 C560 那一形。用户 2026-10-23 弹窗定「开一轮三方定（推荐）」，原话在调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md`。

第一轮打穿甲（环洞）、乙（钉槽轮到下一槽）；第二轮打穿丁甲的两处读法（前提同实例、第 2、3 种按值），定下修法（任一实例、按现算），两腿一致认出「合法写坏镜像」与「p 写 255、N+1 在飞」的坏写者在单张镜像上判不开（崩溃点重放的枚举不产出撕裂的根槽，`crates/singlefs-checker-tier/src/crash.rs` 里 `TearableInPlaceOverwrites::of` 的 Panics 段）。

这是最后一轮：分工照 `.claude/rules/three-way-inference.md` 第三轮——「已定」项撑不撑得住 / 未定项清单的完整性 / 那一轮报出的数字能不能核。这一轮新冒出来的改法算零轮，写进交用户表，不开第四轮。

## 二、实现今天的样子（主 agent 的观测，2026-10-23，行号按主工作区现查）

- 写者：`crates/singlefs-core/src/allocator.rs` 第 1802 行 `reclaim_threshold_of_a_root_carrying` 取 `self.reclaim_floor.max(rollback_floor_of_the_root)`；`reclaim_floor` 跟着环里最旧有效根走（同文件第 1588 行 `oldest_valid_root_in_the_ring`、第 1756 行起）。
- 池级 checker：`crates/singlefs-checker/src/walk.rs` 由 `deferred_slots_reclaim_threshold` 现算、判 `newest_root.reclaim_threshold == recomputed_threshold`；journal 记录表在同一个函数里可用（`scanned_journal_records_by_device` 一类，现查）。
- GPU 走读内核：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 里 `judge(INVARIANT_I_7_18, eq64(reclaim_threshold, recomputed_threshold))`；第二轮两腿都推「GPU 没有现成的点名 N+1 的记录交接」。
- 不变量条文：`crates/singlefs-checker/invariants/02-根环.md` I-7.18、I-7.19。
- 场景与造法：`crates/singlefs-harness/tests/common_images_for_the_checker_tier/rollback_by_a_forward_publish.rs`、`crates/singlefs-harness/tests/common_root_ring_holes/mod.rs`。
- 前两轮四条腿的模型：`research/prompts/m3-close-i718-oldslot-r1-{opus,sonnet}-model/`、`research/prompts/m3-close-i718-oldslot-r2-{opus,sonnet}-model/`（只读，要改拷进自己这一轮的模型目录）。第二轮辩方的 `out/raw/` 出自较早一版探针，复跑张数不同、归类相同（第二轮判决第六节）。

## 三、被判的形态（照抄第二轮判决第三节）

**前提**（丁甲、丁乙共用）：最新根（txg N）之后那一槽读不出自证过的根，且 journal 里有**任一实例**点名 N+1 那次发布的记录，且 N+1 ≥ 3S。

- **丁甲**：前提成立时，I-7.18 判 t 等于下面三种**现算**之一就成立：今天的现算；把那一槽按它在更早任一圈住过的根补进环之后照今天的算法现算；被抛弃的根住在那一槽时照今天的算法现算。
- **丁乙**：前提成立时，I-7.18 这一态报不适用（照 I-7.18 条文里「不判」那一类的写法记）。
- **丙**：不改判器，这一态认成已知红，场景期望写成 I-7.18 红，留给 C560。

前提不成立照今天判。三者共有的取舍写明：丁甲、丁乙收下那个「p 写 255、N+1 在飞」的坏写者（漏报），丙把合法写坏镜像判红（误红）。

## 四、格（照抄第二轮判决第三节「第三轮要答的格」）

| 格 | 问什么 |
|---|---|
| Z1「已定」项撑不撑得住 | D16（发布语义） 已定项 1「写失败的槽按旧内容算」与已定项 7 持久顺序、I-7.18 与 I-7.19 条文、C773 与 C863 的用户立场：三个形态各要改哪一句，改了会不会和别的已定项字面打架；丁甲、丁乙收下的那个坏写者，按 C863「判不出的坏实现」那一类该怎么登记 |
| Z2 未定项清单的完整性 | 两轮都没量的：环里同时有几个洞；洞与抬 F 叠在一起；`WithTreesAndNoFile` 之后的写行；多实例交替；p 的其余取值；GPU 交接的写法与对拍要加的盘 |
| Z3 两轮报出的数能不能核 | 第一轮「格 B 36/36」「格 F 11/11、12/12」、第二轮「格 G 3/3」「BR 14 张、50 张」「33 个键分类不变」与第二轮判决第六节的复跑差异：用独立于原腿的办法逐个复核 |

## 五、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | Z1–Z3 | 复核两份判决：替丙（不新增漏报）与丁乙（不在写坏镜像上求等）各自辩护，判前两轮对它们的打中够不够得着、是不是同样打中丁甲；Z3 用独立的办法复核数 |
| 云端攻方（Opus） | Z1–Z3 | 找反例：三个形态各自与已定条款打架的地方、Z2 列的没量过的形上谁误红谁漏报、Z3 前两轮核不上的数。不重复前两轮攻过的角度（格 A、B、E、F、G、BR、Y 本身，甲的括注，乙的第一圈闸，前提同实例，按值读，兄弟镜像） |

方案按 `crates/` 今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
