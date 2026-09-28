# 里程碑二收尾：代码轮第二轮判决（2026-09-27）

<!-- doc-lint:not-numbers Y1 Y2 Y3 Y4 Y5 Y6 Y7 Y8 Z3-A P1 Q4 Q7 Q8 X5 -->

正文 `research/prompts/_m2-closeout-code-r2-body.md`，背景 `_m2-closeout-code-r2-background.md`（sha256 73bd9a8f…），被判的 diff `_m2-closeout-code-r2-diff.md`（`refs/sop/m2-closeout-code-r1-snapshot` → `refs/sop/m2-closeout-code-r2-snapshot` = `30c084138f7da7488f27874417902caeb5272a15`，树 `98cbcaf8…`；71 个文件，+38121 / −2073，src 29 份）。三条腿：攻方 `m2-closeout-code-r2-opus-output.md`（Y1、Y2、Y4、Y6、Y7，sha256 be3fc719…，模型目录 `m2-closeout-code-r2-opus-model/`）、正推 `m2-closeout-code-r2-sonnet-output.md`（Y3、Y5，sha256 bb634470…）、本地攻方（Y8 算术；提示 `m2-closeout-code-r2-local-attack.md`，运行记录 `m2-closeout-code-r2-local-attack-runlog.md`，翻译核对表 `m2-closeout-code-r2-local-attack-translation-audit.md`；它的交回只发了一条占位，续做闸不许再联系，主 agent 直接读产物）的每一份样本，主 agent 都读了：`m2-closeout-code-r2-local-attack-output-s1.md`（干净，采）；`m2-closeout-code-r2-local-attack-output-s2.md`（干净，采：runlog 第 9–10 行记两次跑到 `-s2` 都被损坏闸判红、落成 0 字节，第四次才过闸）；`m2-closeout-code-r2-local-attack-output-void1.md`（损坏闸留下的作废副本，参考：六格都写 matches，与 s1 / s2 一致，没有打中）；`m2-closeout-code-r2-local-attack-output-void2.md`（作废副本，参考：第 22 行写第一条流闭式「Mismatch: 16777240 − 16777237 = 3」，它把新段序列第 10 段抄成 1、而登记的序列第 10 段是 2（C577 报告第 75 行 `2+2+1+2+2+1+2+24+2+1+2`），是抄错序列算出的差，s1 第 21 行与 s2 第 18 行按正确序列各算得 16777240；不采、不算打中）。核查员 `m2-closeout-code-r2-verifier-output.md`（sha256 e3715dd4…）：核了 84 处引文（✓ 79、✗ 5）、82 个产物 sha256 全对、`rerun.sh` 4 组日志字段级全对、3 组因机器负载在时间预算内没核完（单故障扫、Y6 没文件那一族、Y6 卸载与回归对比）。✗ 5 处：攻方两处把 `decisions/13-验证路线.md` 的第 71 行写成 72（空行）；正推 `walk.rs` 六个行号全错（两处指到不相关的 I-9.4 / I-9.12 判定）、`invariants.md:12` 应为 11、全篇把「槽」抄成「槎」12 处且三处引文在关键限定语前截断。主 agent 判正推那几格时按 kb 与快照原文现读，不按它的引文。

开工快照 `m2-closeout-code-r2-snapshot/`：正文与 16 份 kb 的 sha256 在 `kb-sha256.txt`，材料员开工前逐行核过；先派材料员、后派腿（`dispatch-time.txt`）。腿交齐后 `sha256sum -c`：7 份对不上——主 agent 的 kb 第八批（`invariants.md`、`checks-owed.md`、D16、D22、D8、D3）与别的会话对 D13 的改动。开工原样倒推进 `kb-at-start/`：6 份按替换反做、D13 取 HEAD，与快照逐份相同；`checks-owed.md` 倒推不出（别的会话同期加了 C580–C582），核查员对它以背景材料附录里整段抄的那一份为准。腿引 kb 条款写的都是 kb 文件自己的行号。

## 一、各格判定

| 格 | 判定 | 依据 |
|---|---|---|
| Y1 最新根读不出、见证与屏障 | **没打中（一次抽样）**：五个入口单故障逐序号扫 3024 段（含 C577 那道屏障、零单元发布那一支、取号前后的读），checker 红 0、重挂败 0、写了再拒 0、冻结后原样重发败 0。零单元发布屏障报错不冻结、整个挂载返回错误：**替没写的条款做了选择**（D23（journal 的角色与格式） 已定项 14 第 397 行只写冻结），今天结局无害，第三轮 X5 ③ 钉用例。回退时一次读错报成 NotInRing：已知（Q7，合入后验证一在改）。「没打中」只一次抽样，第三轮 X5 ① 再抽 | 攻方 Y1（`m2-closeout-code-r2-opus-output.md` 第 116–165 行）；核查员复跑单故障扫因负载没核完 |
| Y2-a 同池旧快照的盘经挂着之后的入口「洗白」 | **打中**：盘 1 换成第一个文件之后的旧快照，会话发布或正常卸载照常做成、把旧盘的系统配置轮换到「跟得上」，之后可写挂载做成（实例 2）、池级 checker 红 1–6 条 I-2.1（校验和与内容匹配）；对照：同一组盘直接可写挂载被落后那一支拒；用户动作放开扫 16 格都一样。判定「兑现了条款」——D18（块里携带什么信息） 已定项 11 第 314 行字面「不核落后那一支」——但条款给可写挂载放行第二种写的理由（「系统配置跟得上、只缺几份单元」）罩不住这一格。**用户 2026-09-27 定「采纳，四个入口都核」**：会话、卸载、抬 F、回退在「可见」之后再核本盘设备号与落后支，D18 已定项 11 第 314 行改（kb 第九批），实现进第三轮那一批 | 攻方 Y2-a（第 56–90 行，`logs/y2.log`、`logs/y2-sweep*.log`，核查员复跑 y2 字段级全对）；用户定案 |
| Y2-b 会话收下盘体对调的盘表 | **打中，替没写的条款做了选择**：会话只比身份次序与「可见」（快照 `mount.rs:690` `is_the_device_table_of_the_mount`），别的入口还比本盘设备号（`mount.rs:3105` `device_table_disagreeing_with`）；会话照常发布、把错的本盘设备号写进两块盘的系统配置，之后池可写挂不上、checker 0 违例——checker 读本盘设备号却没有一条不变量拿它比盘的身份（越格线索，第三轮那一批补 I-7.14）。同一盘表交卸载在任何写之前拒。用户定案同上 | 攻方 Y2-b（第 91–110 行，`logs/y2.log`）；用户定案 |
| Y2-c 只剩一槽自证、别的池的盘、全零空盘 | **没打中**：第一轮 Z3-A 三形照旧被拒 | 攻方 Y2-c（第 111–115 行） |
| Y3-1 K1 整棵树 + 逐设备合取，两盘不同落点 | **兑现了条款**：K1 对全部设备求和，恒 ≥ 任一设备的实际值，逐设备合取用同一个数比（D28（挂载期承诺量） 已定项 4 用户改定的式子） | 正推 Y3-1（`m2-closeout-code-r2-sonnet-output.md` 第 206 行） |
| Y3-2 C545 准入先拒之后的假性 ENOSPC | **兑现了条款**：唯一实测到的「真有空闲却被拒」（384 槽段内 79 对）已登记在 C571（df 算不算聚簇段内关着的空槽，假性 ENOSPC 待判）；C576（三条用例被准入先拒连带红，改法待选） 管的是另一件事 | 正推 Y3-2（第 207 行） |
| Y4-a core 与 checker 对系统配置几何字段的读法 | **打中，和条款说反话**：core 解槽（快照 `system_configuration.rs:473` `parse_slot`，第 497 行 `skip(4 + 8)`）不读 journal 环起点（325）与根环起点（371）、取编译期常量（`root_ring.rs:94`、`journal.rs:201`），判槽距上界也用常量；checker 按字段读（`singlefs-checker/src/image.rs:241–244`）。根环起点写 0：checker 报 I-7.13（系统配置池级字段在读者收的范围里） 并自称「实现整池拒绝挂载」，core 只读、可写都做成；写 65：core 照挂、checker 报 I-7.1；417 改 +64 或环长改到段边界外：core 整池拒、I-7.13 不红。D22（单元原子性怎么合成） 已定项 16 第 332 行「基址 = 该字段 × 16384」、第 336 行「槽 1 整槽落在同一槽自述的根环起点之前」在 C、E 两格上与代码相反。合法历史走不到（mkfs 只写 64、1024），盘上内容可控时走得到。**用户 2026-09-27 定「读字段，不等于常量就整池拒」**：core 读 325 / 371、与第一版常量不等整池拒、槽距上界用同一槽自述的根环起点，I-7.13 补同一张表；实现进第三轮那一批 | 攻方 Y4（第 174–221 行，`logs/y4.log`，核查员复跑 y4 字段级全对）；用户定案 |
| Y4-b mkfs 环长不是物理块整数倍；读路径 panic 面 | **没打中**：mkfs 写之前被块层拒；panic 面只读了代码（一次抽样） | 攻方（第 222–234 行） |
| Y5-1 新判定的不适用与判绿分得开 | **兑现了条款**：`Judgements::into_report` 的优先级（违例 > 部分判不了 > 已评估 > 显式不适用）只在真有对象时判绿 | 正推 Y5-1（第 208 行；行号按快照现读，不按它的引文） |
| Y5-2 I-7.13 一槽越界其余全报不适用 | **兑现了条款**：是 I-7.13（系统配置池级字段在读者收的范围里） 条款自己的字面（用户 2026-09-27 定整池拒）；代价是同一镜像上另一处真实损坏在这次判定里不可见——已写在那一行的判据里，不另立账 | 正推 Y5-2（第 209 行） |
| Y5-3a I-9.16 不连中央映射树的号比 | **替没写的条款做了选择**：D8（核心索引结构） 已定项 8 ② 的发号名单字面含中央映射树，I-9.16（树表条目按树 ID 严格升序且合发号次序） 只罩树表七棵。主 agent 判：中央映射树的根在根记录里、不进树表，树表排序判不到它是条款对象的边界，不是漏；kb 第八批已把原句写成「中央映射树不进树表」。被攻过一轮（正推），第三轮辩方复核 | 正推 Y5-3a（第 210 行）；树表排序报告第八节第 1 条 |
| Y5-3b I-9.16 只比树表里有的种类 | **替没写的条款做了选择**：缺项由 `tree_table_entries_each_kind_at_most_once` 与缺树判定兜底，不留判定空白。主 agent 接受 | 正推 Y5-3b（第 211 行） |
| Y5-3c I-9.16 排在水位与缺树判定之前 | **兑现了条款**：「拒在任何写之前」的直接推论 | 正推 Y5-3c（第 212 行） |
| Y5-4 checker 与 core 只共享常量模块 | **兑现了条款**：门禁 94 号在 A3-checker、A3-checker-2、树表排序三份报告里都绿 | 正推 Y5-4（第 213 行） |
| Y6-a journal 环转圈之后的撕裂 | **没打中**：1 MiB 环两族历史全量 196665 + 4155 个状态，环转圈之后撕开的记录进了第三态枚举，oracle 与 checker 0 违例、再挂全部做成。有文件那一族覆盖写那一段 1677 万个状态与崩了再挂 39 万个状态没跑（超过一段历史 10⁶ 的线 / 时长）；核查员复跑没文件那一族因负载没核完 | 攻方 Y6-a（第 237–269 行） |
| Y6-b C577 之后的关段规则 | **兑现了条款**：根槽 FUA 自成一段、轮换两写由末尾屏障关段，是 D13（验证路线） 已定项 4 第 71 行的字面（攻方写成第 72 行，核查员核出是 71） | 攻方 Y6-b（第 270–275 行）；核查员 |
| Y6-c、Y6-d 续跑与分片、模型「重读仍读不出」那一格 | **只读了代码，没造输入**（一次抽样，不算没打中） | 攻方（第 276–283 行） |
| Y7 合并点：A2a 同段同盘两次系统配置写、写了再拒 | **没打中**：189 条流 0 次；写了再拒 0（见 Y1 扫）。核查员复跑 a2a 字段级全对 | 攻方 Y7（第 166–173 行） |
| Y8 算术（本地攻方） | **两次样本六格全部对上**：第一条流两态闭式 67108885 → 16777240（`m2-closeout-code-r2-local-attack-output-s1.md` 第 17、21 行，`m2-closeout-code-r2-local-attack-output-s2.md` 第 18 行）；隔离看 33 → 35、暖机 18 → 20（s1 第 28–29 行，s2 第 29–30 行）；单元区起点 1024 + 环长 ÷ 16384 在 768 MiB / 6 MiB / 159 MiB 上各 50176 / 1408 / 11200、都在 64 槽段边界上，lcm(812, 64) = 12992、50176 − 3 × 12992 = 11200（s1 第 45–53 行，s2 第 45–49 行）；1 MiB 环在飞上限 85（s1 第 62 行，s2 第 58 行）。翻译核对表 16 项与事实表 6 组核查员全 ✓ | 本地攻方两次干净样本 `m2-closeout-code-r2-local-attack-output-s1.md`、`m2-closeout-code-r2-local-attack-output-s2.md`；作废副本 `m2-closeout-code-r2-local-attack-output-void1.md`、`m2-closeout-code-r2-local-attack-output-void2.md` 只参考（开头那一段写了各自的内容与去向）；核查员第三节 |

## 二、改法（被攻过零轮，第三轮攻）

| 改法 | 做什么 | 修哪一格 |
|---|---|---|
| 四个入口再核两样 | 会话发布、正常卸载、抬 F、管理员回退在「可见」之后再核每块盘自证槽里的本盘设备号等于盘表给的身份、这块盘最新那份系统配置不落后于现行那一版；任一不过在任何写之前拒，只剩一槽自证的盘不误拒。形态照攻方原型补丁 `fix_prototype_swapped_or_behind.patch`，不照抄（它没打回退、借了成员名）。代价（推的，没量）：本盘设备号在「可见」那一判已读进来的两槽里、不多读盘；落后支只在盘被换过时才读单元 | Y2-a、Y2-b |
| core 读 325 / 371 | `parse_slot` 读两字段，择系统配置时与第一版常量不等整池拒；槽距上界用同一槽自述的根环起点；I-7.13 补同一张表（417 = 环长现算、起点在段边界、325 / 371 等于常量），checker 另写一份 | Y4-a |
| checker 补 I-7.14 | 每块盘每个自证过的系统配置槽里的本盘设备号等于这块盘在池里的身份，不等报违例；坏镜像一份 | Y2-b 越格线索 |
| D18 已定项 11 第 314 行、D22 已定项 16 现状、I-7.13 那一行、I-7.14 立号、D23 已定项 14 零单元那一支的选择 | kb 第九批（改法落地之后写现值） | Y2、Y4、Y1 |
| 合入后验证一（在飞） | P1（D16 第 37 行读法）、Q7 两处读不出、Q4 归族、钉值统一修正、层 0 文件静态改 | 第一轮之后的欠 |
| Y5-3a 的取法 | I-9.16 不连中央映射树的号比：主 agent 接受实现边界，射程句已补 | Y5-3a |
| C577 结清 | 第三轮 X5：Y1 再抽一次样、litmus「轮换之后屏障」、零单元发布屏障报错的用例、屏障代价问题单、双故障形两条钉已知 | Y1、用户定 |

以上代码改法合成一批（`/tmp/claude-1000/impl-r2-fixes/spec.md`），等合入后验证一交回、合入之后派一个实现员做（同一批文件）。

## 三、第三轮

正文 `research/prompts/_m2-closeout-code-r3-body.md`（稿已备）：辩方复核第一、二轮判决（X4）；攻方攻合入后验证一与第二轮改法合入后的代码（X1、X2）、C577 结清（X5：Y1 换历史再抽一次、零单元发布屏障报错那一支）；本地腿算钉值统一修正与层 0 静态改的数（X3、X2）。快照在这两批合入之后重取。第三轮之后停。

## 四、按路径点名被判的文件（门禁 56 号）

被判的 src 28 份：`crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker/src/lib.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-core/src/admission.rs`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/code_two_tree.rs`、`crates/singlefs-core/src/inode_tree.rs`、`crates/singlefs-core/src/journal.rs`、`crates/singlefs-core/src/make_filesystem.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/pointer.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/system_configuration.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/unit.rs`、`crates/singlefs-format/src/lib.rs`、`crates/singlefs-harness/src/bad_disk_input.rs`、`crates/singlefs-harness/src/crash.rs`、`crates/singlefs-harness/src/crash_injection.rs`、`crates/singlefs-harness/src/fault_injection.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/layer0_progress.rs`、`crates/singlefs-harness/src/model.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-harness/src/bin/first_transaction_on_device.rs`、`crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`、`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`（后两份是实验装置，对不对由各自实验页的变异表与单测判）。

按路径点名、不判、归别的会话：`crates/singlefs-harness/src/bin/e161_crash_state_dedup_and_time_split.rs`（singlefs-e1 的 E161 装置）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D18（块里携带什么信息） 已定项 11 | 改了 | 2026-09-27 改了：挂着之后的入口在「可见」之后再核本盘设备号与落后支（用户定），第 314 行「不核落后那一支」随 kb 第九批改 |
| D22（单元原子性怎么合成） 已定项 16 | 改了 | 2026-09-27 改了：core 读 325 / 371、与常量不等整池拒，槽距上界按同一槽自述的根环起点（用户定），现状句随 kb 第九批写 |
| D23（journal 的角色与格式） 已定项 14 | 不受影响 | 2026-09-27 不受影响：Y1 零单元发布屏障报错不冻结是替没写的条款做的选择，条款不改，第三轮 X5 ③ 钉用例后由 kb 第九批补一句 |
| D16（发布语义） 已定项 7 | 不受影响 | 2026-09-27 不受影响：Y1 单故障扫 3024 段没打中（一次抽样），Y6-b 关段兑现 |
| D13（验证路线） 已定项 4 | 不受影响 | 2026-09-27 不受影响：Y6-a、Y6-b 兑现字面 |
| D13（验证路线） 已定项 5 | 不受影响 | 2026-09-27 不受影响：Y5-4 兑现 |
| D28（挂载期承诺量） 已定项 4 | 不受影响 | 2026-09-27 不受影响：Y3-1 兑现 |
| D2（RAID 条带策略） 已定项 13 | 不受影响 | 2026-09-27 不受影响：Y3-2 兑现 |
| D8（核心索引结构） 已定项 8 | 不受影响 | 2026-09-27 不受影响：Y5-3a 是实现边界的选择，射程句已由 kb 第八批补「排序契约由 I-9.16 判」 |
| D3（空间分配） 已定项 10 | 不受影响 | 2026-09-27 不受影响：Y4 四处读的单元区起点在合法镜像上是同一个数（Y8 算过） |
