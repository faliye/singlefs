# 里程碑二收尾：代码轮第三轮正文（2026-09-28）

<!-- doc-lint:not-numbers X1 X2 X3 X4 X5 X6 Y1 Y2 Y4 Y5 Z1Q-A Z1Q-B Z3-A P1 Q4 Q7 -->

## 一、这一轮要判什么

第三轮，也是最后一轮（`.claude/rules/three-way-inference.md`「第三轮之后停」）。第二轮判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第三节写明这一轮四件事：

1. **复核前两轮的判决**（辩方腿）：第一轮 `research/prompts/m2-closeout-code-r1-main-verification.md` 与第二轮各格的判定够不够得着、打中的是不是同样打中所有替代方案、「已知」那几格有没有被拿来挡住真打中。
2. **攻第二轮之后合入的代码**（攻方腿）：合入后验证一（报告 `research/prompts/m2-impl-merge-verify-1-implementer-report.md`）与第二轮改法两半（报告 `research/prompts/m2-impl-r2-fixes-a-implementer-report.md`、`research/prompts/m2-impl-r2-fixes-b-implementer-report.md`）。
3. **C577 加屏障结清**（用户 2026-09-27：「开第三次代码三方的时候 解决 C577 加屏障的问题」）。
4. **算术**（本地腿）：钉值统一修正之后各测试文件里新钉的数与层 0 文件的静态改。

共用问句与三种结论照第一轮正文，这里不抄第二份。这一轮里新冒出来的零轮形态不再为它开一轮：写进判决的交用户表、标「零轮」。

## 二、实现今天的样子（主 agent 的观测，2026-09-28 现查）

- **腿读代码一律读快照**：提交 `0cc2a238a57d297f511f9e08cedbbc36c6d56d67`（ref `refs/sop/m2-closeout-code-r3-snapshot`，树 `8bd5b978c1bf6342120c92cbbaabd4d89fe21551`），不在任何分支上。它是 2026-09-28 取快照时 HEAD `bc57af7a` 加主工作区 `crates/` 与 `litmus/` 的原样（含别的会话 singlefs-8b 已暂存没提交的改动与未跟踪文件），再打上合入后验证二第一段补丁（改法 A 与挪到改名后路径上的改法 B，`research/prompts/m2-closeout-code-r3-snapshot/merge-verify-2-stage1-crates.patch`，sha256 fbafe111…）。取法：`git archive refs/sop/m2-closeout-code-r3-snapshot crates litmus | tar -x -C <草稿目录>`。主工作区此刻还没打第一段、而且别的会话还在改，别读主工作区。
- 快照里 `crates/*/src/**/*.rs` 59 份的 sha256 在 `research/prompts/m2-closeout-code-r3-snapshot/crates-src-sha256.txt`，`crates/` 与 `litmus/` 全部 186 份在同目录 `crates-sha256.txt`。
- 被判的范围两块：
  - **合入后验证一**（P1、Q7、Q4 与钉值）：补丁副本 `research/prompts/m2-closeout-code-r3-snapshot/merge-verify-1-crates.patch`（sha256 24ce451b…，路径是改名之前的），src 5 份：`crates/singlefs-core/src/mount.rs`、`recovery.rs`、`transaction.rs`，`crates/singlefs-harness/src/history.rs`、`model_comparison.rs`。它已随别的会话的提交 `bc57af7a` 进了 HEAD，快照里就是它合入之后的样子。
  - **合入后验证二第一段**（改法 A + B）：`git diff refs/sop/m2-closeout-code-r3-base refs/sop/m2-closeout-code-r3-snapshot`（基准 `9a9cfb6c` 是同一刻不打第一段的样子），33 个文件，+2594 / −684。其中 src 13 份：`crates/singlefs-core/src/mount.rs`、`mounted_session.rs`、`recovery.rs`、`system_configuration.rs`、`transaction.rs`、`make_filesystem.rs`，`crates/singlefs-checker/src/image.rs`、`lib.rs`、`walk.rs`，`crates/singlefs-harness/src/history.rs`、`model_comparison.rs`，`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs`、`new_pool_file_creation_on_device.rs`；`litmus/` 新加两份。
  - **不判**：别的会话 singlefs-8b 的包重组与按内容改名（提交 `973e1f58`、`bc57af7a` 与它暂存的后续，文件从 harness 挪进 `crates/singlefs-checker-tier/`、61 份测试与 5 个装置改名、术语改名改进注释），有它自己的判决；腿打中的格落在这些改动上时照写，判决里写明归属。合入后验证二第二段（双故障两条、`writable_mount_takes_a_second_instance.rs` 那一格、`random_histories.rs` 两条、「F_生效 取 max」新用例、core 与 checker 收同一张表的交叉测试、证红）只动测试、还在做，不在快照里，归提交时的门禁与崩溃验证员。
- **已知、不算打中**（腿打中这几格时写「已知」并点名去向）：
  - C579（判据在单故障合法状态上也拒可写）：用户 2026-09-27 定维持乙字面（判不出按真、拒可写、只读挂载照常），`warm_up_journal_counter.rs` 的 c366 改钉拒可写；腿要攻的是「判不出」那一支有没有一条单故障历史被拒成永久只读而条款没给出路，不是这一格本身。
  - fsync_drop 两条改钉乙的成员（乙之后逐盘核不可达，调查员报告 `research/prompts/m2-investigate-three-reds-report.md` 第七节）。
  - C572、C573–C576、C578、C579：`.claude/kb/checks-owed.md` 各行。
  - C580（两处读不出无声放过没动）：合入后验证一已改，腿打中它的**改法**算真打中，不算已知。
  - C583（双故障形：每盘一槽系统配置坏加最新根一时读不出）：用户 2026-09-27 定不在射程；`random_histories.rs`、`reuse_after_raising_the_floor.rs` 那两条同形用例钉「红在 I-7.4 是已知」。腿要攻的是「这一形之外的单故障形有没有用例罩着」，不是这两条本身。
  - 第二轮判决「已知」段（宽 0 条数非 0、指针头部 MAC / nonce、I-9.16 取法、屏障报错照 D23 冻结）沿用；其中屏障报错那一格这一轮由 X5 ③ 的用例钉住，攻方攻用例钉的结局对不对。
  - `new_pool_file_creation_on_device` 两条差 1（装置原名 `first_transaction_on_device`）（发布末尾一道屏障、新 `PoolWriter` 开头又一道，录制器 `SharedStream::push` 并掉相邻屏障、设备照收两次 FLUSH）：主 agent 定不改装置、照今天盘上的数改钉，归第五步 55 号六档重录时定。腿打中「录制流与设备流对不上」不算已知——那正是 55 号判的量，若腿能在快照上给出一条设备流比录制流多的 FLUSH 影响判定的历史，算真打中。

## 三、六格

### X1　读不出的根槽怎么算：P1、Q7、Q4 与 D16 第 37 行全句（合入后验证一第一节、第二轮改法 B 第 3 件）

`recovery.rs` 的 `readable_roots_rereading_unreadable_root_ring_slots_once`（已知槽集参数）、`mount.rs` 抬 F 与 `rollback_candidate`、`transaction.rs` 写系统配置前算生效 F、`StillUnreadableAfterOneReread` 一族与 `model_comparison.rs`。压着 D16（发布语义） 第 37 行全句（挂载那一刻读得出、或这个进程写过且 FUA 返回过的槽，这一次读不出**或自证不过**就重读一次，仍坏就拒这次抬 F；挂载那一刻就读不出或自证不过的槽当没有根）、D23（journal 的角色与格式） 已定项 14。问：已知槽集取自分配器上的根环表，挂载之后这个进程写过的槽有没有进集合；「自证不过」那一半四个入口都走到了没有；挂着时的管理员回退算候选与 F 生效值也照这条处置（合入后验证一 P1 与 B 第 3 件，D16 字面只写了抬 F 上限、回退候选集写的是「没判」，kb 第十批按实现扩写、被攻过零轮），攻这一扩写在哪条历史上拒错或放错；另有两处读不出仍当没有根（kb 第十批为它们另立一笔欠账，接在 C580（两处读不出无声放过没动） 之后）：`transaction.rs` 写系统配置之前那一处重读一次之后仍走原来的退路、管理员回退算树 ID 与 inode 号水位那一遍——各有没有一条单故障历史让写进盘的 F 或水位错；`transaction.rs` 写系统配置前拿不到已知槽集时读不出当没有根，有没有一条单故障历史让写进系统配置的 F 比生效值低；`rollback_by_a_forward_publish.rs` 与 `reuse_after_raising_the_floor.rs` 里原来「挂载后改坏一条根」那两条改成哪一形、改对没有；原来 `reuse_after_raising_the_floor.rs` 那条守的「F_生效 取 max(根上带的, 系统配置里的)」挪到哪条新用例、罩没罩住。

### X2　钉值统一修正：改的数是算出来的还是抄出来的（合入后验证一第二节、B 第 4–6 件）

每处新钉值的算式（段序列、闭式、撞墙步数、σ、A3b 小盘几何：三档小盘环 6 MiB、盘宽 = 起点 + 单元区槽数）；`crash_enumeration_sharded_across_processes` 的金样（checker 档，不跑：合入后验证二第二段只改静态推得出的打印行数与进度文件行数，两个 SHA-256 与计划哈希待第五步 54 号跑出来重录，推的行数 58 / 47 对不对由本地腿算）；两条双故障形用例钉已知之后单故障形还有没有用例罩着；`new_pool_file_creation_on_device` 两条照盘上的数改钉，注释说的成因（录制器并掉相邻屏障）与代码对不对得上；16 行变异里证红的结果。

### X3　层 0 文件的静态改（不跑）

8 份层 0 文件（现在在 `crates/singlefs-checker-tier/tests/`）的段数组与闭式按 C577 报告「钉值怎么变」的算法改：第一条流 `2+2+1+2+2+1+2+24+2+1+2`、33→35 步、σ 2^18→2^16；本地腿算一遍。

### X4　前两轮判决的复核（辩方）

第一轮 Z1Q-A / Z1Q-B / Z3-A、第二轮 Y5-3a / Y5-3b 与「已知」那几格；C577 加屏障之后 Z3-A 乙的三形有没有变；第二轮 Y1 攻方在 3024 段单故障扫上「没打中」是不是只读了代码。

### X5　C577 加屏障结清（用户定第三轮解决）

`transaction.rs` 的 `persist_the_root_then_rotate_the_system_configuration`（根槽 FUA → 系统配置轮换 → 池屏障）。压着 D16（发布语义） 已定项 7（序点四个）、D23（journal 的角色与格式） 已定项 14、D13（验证路线） 已定项 4。五件：① 第二轮 Y1 攻方「没打中」只是一次抽样（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」），这一轮攻方换一组历史（不同种子、含回退与抬 F 之后立刻掉电）再抽一次；② `litmus/` 新加的两份「轮换之后屏障」（正例与 `-nofence` 对照，改法 A 第 3 件）：形态、锚点绑到 `persist_the_root_then_rotate_the_system_configuration` 对不对、`publish_order_matches_litmus.rs` 步数 35 罩进去没有（herd7 归提交时 57 号，腿不跑）；③ 屏障报错时零单元发布不冻结、整个挂载返回错误（D23 已定项 14 射程今天写成选择）：A 第 4 件的用例钉的结局（错误成员、盘上哪几步已落、之后可写挂载怎么判）对不对；④ 屏障的代价（每次发布多一次 FLUSH）：不在这一轮的腿里；第五步 55 号六档重录时报每次发布的 FLUSH 次数与挂钟，数出来再由用户定要不要立实验；⑤ 双故障形两条钉已知（B 第 5 件，见 X2）。

### X6　第二轮改法：四入口再核两样、core 读 325 / 371、checker 收同一张表、I-7.14（改法 A 第 1–2 件、B 第 1–2 件）

压着 D18（块里携带什么信息） 已定项 11 第 314 行、D22（单元原子性怎么合成） 已定项 16 第 1 条、I-7.13（系统配置池级字段在读者收的范围里） 八项表。问：会话发布、正常卸载与抬 F、管理员回退四个入口在「可见」之后是不是都核了本盘设备号与落后支，任一不过是不是在任何写之前拒，只剩一槽自证的盘有没有被误拒；落后判「(实例, tail) 不落后于现行那一版末条记录」与乙那一判是不是同一级、有没有一条合法的单故障历史（回退之后立刻建对象、写几次）被它拒；core 的 `parse_slot` 读 325 / 371、择系统配置时不等常量整池拒，与 checker 的 I-7.13 三项（417 = 环长现算且在 64 槽段边界、325 = 1024、371 = 64）是不是同一张表——攻方 Y4 用例的 A–E 五格 core 与 checker 结局是否一致（合入后验证二把它写成入库测试 `system_configuration_fields_core_and_checker_refuse_the_same_table.rs`）；落后支今天的判法是「落后**且**缺现行那一版的单元」（与可写挂载落后支同一判，B 报告第四节第 6 条），不是 D18 字面的「不落后」：现行那一版是零单元发布、旧快照上现行那一版的单元一个不缺时旧快照不被拒——攻这个取舍在哪一条历史上让旧快照混进来之后 checker 红或读回旧数据；`recovery.rs` 判槽距上界改用同一槽自述的根环起点，但「逐档找槽 1 那一处仍用常量」（B 报告第三节第 2 行），两处口径不同是不是会让一份 371 合法、槽距在两种算法下一收一拒的系统配置两边判得不一样；I-7.14 在池级 checker 上盘体对调是否报违例、坏镜像只红 I-7.14。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | X4 | 复核前两轮判决：替被判出局的一方辩护，判每格够不够得着、是不是同样打中所有替代方案 |
| 云端攻方（Opus） | X1、X5 ①②③、X6 | 在第三轮快照副本上写用例找反例；与前两轮不重叠的攻击面：第二轮的修补本身（四入口、325 / 371、I-7.14、D16 第 37 行全句）、「已定」项撑不撑得住（D16 已定项 7 序点四个在屏障报错那一支）、Y1 换历史再抽一次 |
| 本地攻方 | X3 与 X2 的算式 | 只算数，不造历史；与云端攻方不重叠。派攻方不派辩方：这一轮辩方那一面已由云端辩方腿做，本地腿要算的是能落成数、逐格判的钉值 |

本地腿：攻（辩方那一面这一轮由云端辩方腿复核前两轮判决，本地腿要做的是能落成数、逐格判的钉值与闭式，派攻方找算错的格）。

第三轮之后停，不开第四轮。

## 五、交付

每条腿的报告按格写：结论（三种之一）、原文、代码位置（快照里的 `文件:行号`）、打中的给复现（命令、种子、原样输出）。攻方写的用例与模型放各自的模型目录，在快照副本上跑，不改主工作区。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。

## 六、回看决策

判决里写。
