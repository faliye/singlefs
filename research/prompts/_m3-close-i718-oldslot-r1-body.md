# m3-close-i718-oldslot-r1：满环时根槽写失败、恢复之前的镜像上 I-7.18 该不该红（推论轮第一轮）正文（2026-10-23）

## 一、题面

根环装满时，一次发布的根槽写落在环里最旧那条根住的槽上；这次根槽写写成 0 或撕裂，进程在重发之前崩。恢复之前那张镜像上：最新根（实例 2、txg 27）带的回收门槛 t 是 4，池级 checker 照镜像上的环现算出 5，I-7.18（根记录的回收门槛等于现算的门槛）判红。

写者没有写错：D16（发布语义） 已定项 1 的「环里最旧有效根」写着「写失败的槽按旧内容算」，照这一句 t = 4。镜像上的环却已经不是建账时的环：最旧那一槽在镜像上读出来是 0 或坏的。

这是欠账 C560（满环时根槽写坏的恢复前镜像上记账多于遍历）那一形：以前红在 I-3.1，C773（读坏与全零根槽的回收门槛钉只在内存里）实现之后红在 I-7.18。根环三方第三轮辩方推过「最新一条根的环就是建账时的环，求等不误红」（`research/prompts/m3-close-rootring-r3-sonnet-output.md` 第 159 行起 Z4.3），这张镜像是反例。

用户 2026-10-23 弹窗定：「开一轮三方定（推荐）」，原话在调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md`。

这一轮要回答：这一态上 I-7.18 该怎么判，条文与两侧判器（池级 checker、GPU 走读内核）跟着怎么改。恢复之后的各张镜像按 C773 已定的改成全绿，不在这一题里。

## 二、实现今天的样子（主 agent 的观测，2026-10-23，行号按主工作区）

- 写者：`crates/singlefs-core/src/allocator.rs` 第 1802 行 `reclaim_threshold_of_a_root_carrying` 取 `self.reclaim_floor.max(rollback_floor_of_the_root)`；`reclaim_floor` 跟着环里最旧有效根走（同文件第 1588 行 `oldest_valid_root_in_the_ring`、第 1756 行起）。
- 池级 checker：`crates/singlefs-checker/src/walk.rs` 第 8507 行起由 `deferred_slots_reclaim_threshold`（第 8366 行）按镜像上的环现算，第 8523–8525 行判 `newest_root.reclaim_threshold == recomputed_threshold`。
- GPU 走读内核：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 第 4600 行一带 `judge(INVARIANT_I_7_18, eq64(reclaim_threshold, recomputed_threshold))`，现算的写法与 CPU 一侧对应。
- 不变量条文：`crates/singlefs-checker/invariants/02-根环.md` 第 25 行 I-7.18。
- 被红的场景：`crates/singlefs-harness/tests/common_images_for_the_checker_tier/rollback_by_a_forward_publish.rs`（plain、torn、zeroed 三个场景）；checker 档的用例在 `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`；变异 `crates/mutations.tsv` 第 2596、2597、702、2628 行。
- 调查员报告：`~/.local/share/singlefs/work/m3-close-b16-i718-baseline/result.md`（sha256 7989c94e…）：plain 场景 23 张里 22 张、torn 场景 24 张里 22 张与期望对不上；恢复之前那张只红 I-7.18（t 4、现算 5），红只出自 walk.rs 那一判；写者没错（txg 27 发布时环里是 txg 4–27，场景自己断言过）。候选甲的 CPU 一半试改补丁在同目录 `prototype-i718-next-slot-by-its-old-content.patch`：三个场景 71 张镜像全绿、I-7.18、I-7.19 的 harness 回归 9 条过、池级 checker 库单测 101 条过；GPU 一半没写，判别力代价没量。

## 三、候选（只写定义）

| 候选 | 定义 |
|---|---|
| 甲 | 最新根之后那一槽 N+1 上没有自证过的根时，按它的旧内容（N+1−3S 那一条根）再现算一次；t 等于两种现算之一就成立 |
| 乙 | 这一态（最新根之后那一槽读不出自证过的根）上 I-7.18 报不适用 |
| 丙 | 这一态认成已知红：场景期望写成 I-7.18 红，条文与判器不动，留给 C560 |

## 四、格

| 格 | 问什么 |
|---|---|
| V1 | 判别力：三个候选各自放过了哪些真错的写者（t 写错、钉错、拿错环）；甲的「两种现算之一」在哪些镜像上会把错的 t 判成对；乙的「不适用」判据是不是池级 checker 只看镜像就判得出来 |
| V2 | 这一态的范围：除了「下一次发布的根槽写失败」，还有哪些合法历史会让镜像上最旧那一槽不是建账时的那条根（纯掉电、挂载时读坏、撕裂域里别的槽）；三个候选各自罩不罩得住 |
| V3 | 与已定条款的关系：D16（发布语义） 已定项 1「环里最旧有效根」那一行（含「写失败的槽按旧内容算」与它点名的六处同取）、I-7.18、I-7.19 条文、C560、C773 的定案；哪个候选要改哪一句，改了会不会让别的分项不自洽 |
| V4 | 两侧判器：GPU 走读内核照同一候选改的写法与代价；CPU 与 GPU 对拍要加哪几盘 |

## 五、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | V1–V4 | 从附录条款与第二节的代码今天的样子正推：三个候选各自要改哪几处、各自的判别力，推出该取哪一个 |
| 云端攻方（Opus） | V1–V4 | 找反例：甲的「两种现算之一」放过的错 t、乙的判据判不出的镜像、丙留下的代价；第二节没列到的合法历史；候选与已定条款打架的地方 |

方案按 `crates/` 今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
