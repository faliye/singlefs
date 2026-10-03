# 根环实现第 8 批：代码轮第二轮判决（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 E2 B2 RR4 RS1 RS2 RS3 RS4 RS5 RS6 RS7 L-K1-1 L-K1-2 L-K1-3 L-K3-1 L-K4-1 L-K4-2 -->

批次：`records/2026-09-30-里程碑三开工第一轮.md` 第 8 批（代码轮第二轮）。第一轮判决 `research/prompts/m3-root-ring-code-r1-main-verification.md`。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 / 小节清单 / 附录 / 改动全文 | `research/prompts/_m3-root-ring-code-r2-body.md`、`_m3-root-ring-code-r2-background.md`、`_m3-root-ring-code-r2-checklist.md`、`_m3-root-ring-code-r2-appendix.md`、`_m3-root-ring-code-r2-diff.md` | — |
| 开工快照 | `research/prompts/m3-root-ring-code-r2-snapshot/`（crates 46 份、kb 4 份；冻结副本 `/tmp/claude-1000/m3-root-ring-code-r2/tree/`） | `bash research/scripts/snapshot-check.sh m3-root-ring-code-r2`：「✗ 快照 m3-root-ring-code-r2：50 个文件里 18 个对不上」，处置见下 |
| 云端正推（Sonnet） | `research/prompts/m3-root-ring-code-r2-sonnet-output.md`（466 行，sha256 29865c8a57c4ba8ccf5365499f4424ad3d4521f44266180e4fa8d79bb169aa6b），模型 `research/prompts/m3-root-ring-code-r2-sonnet-model/` | — |
| 云端攻方（Opus） | `research/prompts/m3-root-ring-code-r2-opus-output.md`（200 行，sha256 34818b036a46233e5b215b9091f6595f39eb3c078c9142148f92e95d7ea9a433），模型 `research/prompts/m3-root-ring-code-r2-opus-model/` | 主 agent 复跑，见「复跑」一节 |
| 引文 | 两份腿报告 | `python3 research/scripts/cite-check.py <两份报告> --root <crates 取冻结副本、其余取主仓的拼合根> --background research/prompts/_m3-root-ring-code-r2-background.md`：「✓ 核了 52 处引文，对上 52 处，没判 0 处（逐处列在上面）」 |

快照对不上的 18 份：腿跑着与交回之后，别的两条线往主工作区合了补丁、做了写回，冻结副本没动。两条腿都只读冻结副本（两份报告开头各自写明），`crates/` 的引文按冻结副本核、52 处全对上；kb 的引文按主仓核也全对上，被改的几处没有落在腿引的行上。倒推用的改动清单都存在快照目录里：

| 来源 | 动到的快照文件 | 倒推材料 |
|---|---|---|
| 形式化接入那条线 17-1（新立 I-9.27、I-5.5） | `image.rs`、`walk.rs`、`crash_judge_checker.wgsl`、`crash_judge_gpu.rs`、GPU 对拍测试、`transaction.rs`（一句注释）、`checker_known_bad_images.rs`、`writable_mount_of_a_formatted_pool.rs`、`crates/mutations.tsv`（末尾追加） | `peer-formal-17-1-crates.patch`（`git apply -R`） |
| 形式化接入那条线 18-2 | `crates/singlefs-harness/src/model.rs` | `peer-formal-18-2-crates.patch` |
| 形式化接入那条线 18-3 写回 | `.claude/kb/decisions/16-发布语义.md`、`.claude/kb/invariants.md` | `peer-formal-18-3/`（`report.md`、`pieces/`、`self-edit-d16.md`） |
| 删除与并发那条线第 14 批 | `mount.rs`、`allocator.rs`、`records.rs`、`recovery.rs`、`model_comparison.rs` 等 | `peer-deletion-batch14-crates.patch` |
| 删除与并发那条线第二次写回（甲、乙）与 C506 追加替换 | `16-发布语义.md`、`19-块指针的结构与宽度预算.md`、`invariants.md` | `peer-deletion-writeback2-spec.{json,md}`、`peer-deletion-writeback2-scribe-b-{spec,report}.md`、`peer-deletion-c506-fix.md` |

### 复跑

攻方模型拷到草稿目录 `/tmp/claude-1000/-home-fy5090-code-singlefs/8adc3bc8-1d0b-43bd-9a3f-9e63dae04aaf/scratchpad/r2-recheck/`，照报告「复跑」一节跑 `rebuild-and-run.sh <草稿目录> e2 e2-sweep fh sir avc fz`（第一次因草稿目录不存在 rsync 退 11，建好目录重跑，`rc=0`）。

- `results/out/` 102 份里 66 份、`results/out-fixA/` 30 份里 28 份逐字节相同，含判决要引的 `e2-s4-p20-b2-flip-once.out`、`e2-s4-p20-b2-clean.out`、全部 `sw-*`、`fa-*` 与 `out-fixA/fh-s4-h9-mount.out`。
- 对不上的 33 份是报告第 15 行自报的那几类（「`results/out/` 里的输出是边改原型边跑的……用今天的源码复跑会多出那几行或字段，其余逐字段应相同」）：主 agent 逐份核过，腿留存的每一行都按次序出现在复跑里，`FUZZ` 行只多一个 `i35_holds_na_red_holestates_holeholds=` 字段（例：种子 100 复跑那一行多「i35_holds_na_red_holestates_holeholds=[33, 0, 0, 7, 7]」，其余逐字相同），另多 `REFUSED` / `PUBLISHED` 打印行。
- 另 3 份（`e2-s4-p20-b2-flip-once-enum.out`、`fh-s4-h6-mount.out`、`fh-s4-h6-none.out`）是今天的脚本不再产出的旧件，报告没引。

主 agent 另对着冻结副本读了 E2 重读那一段：`crates/singlefs-core/src/mount.rs` 的 `reread_of_a_root_ring_slot_unreadable_at_mount`，`RootRingSlotReading::SelfVerified(_) | RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified)` 交 `NoValidRoot`；`crates/singlefs-core/src/allocator.rs` 第 1906–1907 行对 `NoValidRoot` 做 `occupancy.occupants.remove(&ring_slot);`。条款 `.claude/kb/decisions/16-发布语义.md` 第 35 行那一句是「先重读那一槽，读得出且自证过就照读出来的根算，仍读不出才按记的最大 txg 算」，「读得出而自证不过」没写。

正推腿交回的「要主 agent 跑的命令」（GPU 对拍，checker 档）不在这一轮跑：GPU 那两条线索的改法进第 9 批，提交前 checker 档照跑。

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| K1 三句写回条款与代码 | 一致；「读得出而自证不过」「被抛弃」两种结局条款没点名，实现当没有根（规则没说） | — | **站住**（条款写到的两种结局逐字对上） |
| K1 E2 重读读得出而自证不过 | 同上一格的「规则没说」 | **打中**（量过）：有效根 h 所在的槽挂载那一刻读错、h 成了窗口最小那一刻那一次重读读坏（盘上字节完好），实现交 `NoValidRoot`、根环表删掉那一项，回收门槛越过 h；txg 29 那一版全落无崩溃，池级 checker 判 I-3.1 红，记账少 196608 字节（不安全方向）；12 组参数 12 组红，同参数重读读得出 12 组 0 红，挂载也读得出 1 组 0 红 | **打中**（主 agent 现查代码与复跑都对得上）。判别子观测不到：重读那一刻读到「自证不过」既可能是一次读坏的有效根，也可能是从没写过的槽（前洞）。改法 A（自证不过与仍读不出同一处置，按记的最大 txg）在 12 组上 0 红，代价是前洞加挂载读错 2 组各一个盘面 I-3.1 红、记账多（安全方向，`out-fixA/fh-s4-h9-mount.out`）。交用户：第四节 RS1 |
| K1 记下 txg 高于真住在那一槽的根 | 没找到：引理逐点枚举 17399790 点 0 违例 | 推的：进程内重读能读出的只有记的最大 txg 或窗口外 | **站住**（两腿一致） |
| K1 E2 选的实例表（写行前后） | 一致（推的）；没有用例 | 没造出两张表判法不同而那一槽正被重读的历史（推的） | **站住**（推的）；没有钉：并进 RS2 |
| K1 E2 三种结局、记根四处调用点的钉 | **打中覆盖缺口**（量过）：探针 p1（有效根一律当没有）、p2（拿掉按表有效）、p3（自证不过分到仍读不出）、q1–q3（三处记根点换成不重读）在 harness 档全绿；对照 p4 红 | — | **打中覆盖缺口**（取样点不敏感，`.claude/rules/mutation-sampling.md` 第三类）：RS2 |
| K1 B2：F 越过窗口里一条读不出的有效根 X | 条款字面一致；**线索**：探针上 X 读不出 F = 22、读得出 F = 21，I-7.9 照第 9 批改成只取窗口之后，前一格会判红（推的） | 条款字面许可，不算打中 | **条款站住，留一处冲突**：D16 已定项 1「根槽这一次读坏」一行（`.claude/kb/decisions/16-发布语义.md:38`）许可 F 越过 X，而只取窗口的 I-7.9 在镜像上看得见 X、看不见挂载那一刻的读错，判别子观测不到。第 9 批改 I-7.9 之前要定哪边让：第四节 RS3 |
| K1 RR4 抬到 B2 上限那一形 | 一致；该形门槛恒等于新 F，RR4 无区分力 | 同（推的） | **站住**：不是缺口，RR4 用例抬到 6 才分得出 |
| K1 E2 重读读出有效根之后那一槽算不算「这个进程知道住着根的槽」 | 规则没说（`crates/singlefs-core/src/allocator.rs:1027` 不收它） | — | **线索，进写回问句**：随 RS1 一起交用户 |
| K2 回退候选集的窗口 | — | 没打中（一次抽样）：窗口外目标 15 次全报 `OutsideTheOneRevolutionWindow`；窗口里被拒 240 次全是 `VersionWithoutFile`（攻方装置的候选没滤带不带文件） | **站住**（一次抽样） |
| K3 I-3.5 出现规则逐要素 | 一致 | 没打中（一次抽样）：1000 段随机合法历史，带计数的 22696 个盘面全成立，其中 2181 个候选集有洞 | **站住**（两条腿各一次抽样，立场不同） |
| K3 gap-row 子条件、引用不全跳过、同 txg 并组的钉 | **打中覆盖缺口**（量过）：探针 bb、bc、bd、be、bf、bg 放松之后 0 红；对照 ba 红 2 条 | — | **打中覆盖缺口**：RS4 |
| K3 8-2 自取读法 | 合法历史误红没找到；漏红只在 ③ ④（已知没做） | ③ 碰不到（实现不留缺行的跳号）；② 没造出同 txg 候选 | **站住**（一次抽样） |
| K3 洞判法三态、走读两处修正 | 一致；I-3.1、I-3.10、I-5.4 读数不变（推的） | 五批随机历史里这三条一次没红 | **站住** |
| K4 七段 CPU 与 GPU、GPU 旧格式常量 | 一致；GPU 后三段与 RR1 没有变异行 | 一致（偏移逐个对过） | **站住**；GPU 侧变异行缺口并进 RS5 |
| K4 RR1 三处口径 | core、CPU checker、GPU checker 一致；**线索 L-K4-1**：GPU 恢复内核 `parse_record` 不判卸载记号其余位 | — | **线索坐实为第二节已知项的同族**：与 8-2 报告 S7（GPU 恢复内核拼新根七段与 core 不一致）同一个内核，并进 RS5 |
| K4 GPU 走读内核取「由记录施加出来的那一版」的实例表与分配记录树根 | **线索 L-K4-2**：仍从所选根取，CPU 从记录新根段取 | **线索**：同一处（`crash_judge_checker.wgsl` 第 2594、2609、3020 行），合法历史上一段量过两边指针相同 | **两腿一致的线索**：今天合法历史上取到同一个值（攻方 `avc-mount` 量过一段），两边口径不同；照 D23（journal 的角色与格式） 已定项 15「施加一条记录 = 把所选根的这七个字段换成记录里的」GPU 那一侧要改成取记录：RS6 |
| K5 七类读者、两种拒法分界、影子账读者 | 一致；「无法解引用」与「损坏」实现不分（规则没说）；加密压缩四个检查各有钉 | — | **站住**；「不分」那一句照 8-1 报告第五节第 4 点取 D9（加密） 已定项 14 原文，不另立 |
| K6 两张不适用表与五行变异 | 一致（打印核实依据是「一个单元都没判到」；5 行现跑都抓到） | — | **站住** |

## 三、打中的改法（零轮）与去向

| 号 | 改法 | 被攻过 | 去向 |
|---|---|---|---|
| RS1 | E2 重读读得出而自证不过怎么记 | 改法 A 零轮，攻方模型上量过 | 交用户（第四节） |
| RS2 | E2 三种结局（读得出有效在窗口里、自证不过、被抛弃）、「按哪张表判有效」、`mount.rs` 两处与 `transaction.rs` 一处记根调用点各补一条会红的用例与变异行；攻方「要主 agent 跑的命令」第 2 条那个读坏一次的读包装一起做 | — | 第 9 批 core 簇 |
| RS3 | B2 下 F 越过读不出的有效根与只取窗口的 I-7.9 | — | 交用户（第四节）；第 9 批池级 checker 改 I-7.9 之前定 |
| RS4 | gap-row 的「T = R⁻.txg」「中间每个实例都有行」「中间行都是 (0, 0)」三个子条件、前后两版引用不全跳过、同 txg 并组，各补一条坏镜像用例与变异行 | — | 第 9 批池级 checker 簇 |
| RS5 | GPU 恢复内核 `parse_record` 判卸载记号其余位、拼新根七段取记录（并 8-2 报告 S7）；GPU 侧 I-8.10 后三段与 RR1 补变异行 | — | 第 9 批 GPU 簇（与 GPU 实现 I-3.5、删临时表同批） |
| RS6 | GPU 走读内核对「由记录施加出来的那一版」从记录新根段取实例表与分配记录树根，与 CPU 同口径 | — | 第 9 批 GPU 簇 |

## 四、交用户的岔路

| 行 | 候选 | 代价与依据 |
|---|---|---|
| RS1 E2 重读读得出而自证不过 | 今天：当没有根；改法 A：与仍读不出同一处置，按记的最大 txg 算；改法 B：再重读一次，仍自证不过才当没有根 | 今天：一次读坏的有效根上记账少、门槛越过盘上有效根（不安全方向，12 / 12 组，量过）。改法 A：12 组 0 红；从没写过的槽加挂载读错上一个盘面记账多（安全方向，2 / 2 组，量过），零轮。改法 B：读故障跨整次发布时照样红（推的） |
| RS3 B2 下 F 越过挂载那一刻读不出的有效根 | 甲：条款照旧许可，第 9 批的 I-7.9 在这一形上判红（checker 看不见挂载那一刻的读错）；乙：抬 F 的上限把窗口里挂载那一刻读不出、之后没写过的槽当成「可能是一条非空有效根」（与 RR4 把它们并进回收门槛同一个思路），F 不越过它；丙：I-7.9 只取窗口，但窗口里有挂载那一刻读不出的槽留下的空当时整条报不适用 | 甲：合法历史上 I-7.9 误红（正推探针量过 F = 22 那一形，改 I-7.9 之后会红是推的）。乙：读故障时 F 抬得少一点（探针上 22 → 21），D16 已定项 1「根槽这一次读坏」一行与抬 F 上限那一行要改，零轮。丙：checker 从镜像上看不出哪个空当来自读错，只能按「窗口里有空槽」判，判别力变弱（推的） |

## 五、第二节已知没做的几样（照原去向）

池级 checker 的 I-7.9「之前的根」取全环（攻方五批随机历史里每批 27–46 段红，第 9 批）；GPU 不判 I-3.5、临时表（用户定改回全覆盖，第 9 批）；GPU 恢复内核七段（RS5 并入）；harness 判法 `mount_transparency.rs` 改 R2（第 9 批，形式化接入那条线同意由本线改）；I-3.5 ③ ④；`root_record.rs` 读者；e171、e165 注释；清扫器；I-7.8 认领规则；洞上判法两件（第 9 批三方）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-04 不受影响：RS1、RS3 交用户，定之前不改条款；其余格照条款站住 |
| D23（journal 的角色与格式） 已定项 15 | 不影响 | 2026-10-04 不受影响：RS6 是 GPU 一侧没照条款，条款不改 |
| D19（块指针的结构与宽度预算） 已定项 3 | 不影响 | 2026-10-04 不受影响：七类读者照条款站住 |
| D9（加密） 已定项 14 | 不影响 | 2026-10-04 不受影响：算法与压缩段的处置照原文 |

## 六、用户定案（2026-10-04）

| 行 | 用户选的 | 落成什么 |
|---|---|---|
| RS1 E2 重读读得出而自证不过 | 「改法 A：按记的最大 txg 算 (Recommended)」 | 重读读得出而自证不过与仍读不出同一处置，按根环表记的最大 txg 算；重读读出有效根的那一槽算进「这个进程知道住着根的槽」（选项说明里一并写明）。D16 已定项 1 窗口那一行的 E2 句与「根槽这一次读坏」一行跟着改 |
| RS3 B2 下 F 越过挂载那一刻读不出的有效根 | 「上限把读不出的槽算进去 (Recommended)」 | 抬 F 的上限把窗口里挂载那一刻读不出、之后没写过的槽当作可能是一条非空有效根，F 不越过它（与 RR4 把它们并进回收门槛同一个思路）；D16 已定项 1 抬 F 上限那一行与「根槽这一次读坏」一行跟着改。第 9 批 I-7.9 只取窗口时照这个上限判 |

零轮提醒：两条都被攻过零轮，第 9 批实现落地后的代码轮第三轮攻它们。
