# m3-close-rootring-r3 判决（第三轮，最终轮，2026-10-20）

正文 `research/prompts/_m3-close-rootring-r3-body.md`，背景材料 `research/prompts/_m3-close-rootring-r3-background.md`（sha256 ae3d54b7f67960e8ea0aad28cdc32958f783c088541b0b6b1500866558675300），开工快照 `research/prompts/m3-close-rootring-r3-snapshot/kb-sha256.txt`。前两轮判决 `research/prompts/m3-close-rootring-r1-main-verification.md`、`research/prompts/m3-close-rootring-r2-main-verification.md`。

两条腿：
- 辩方（Sonnet）：`research/prompts/m3-close-rootring-r3-sonnet-output.md`，sha256 68c6fd3179aaf303079d540f80356d973b2ed0b743c175afd5343829b7e2c019。模型 `research/prompts/m3-close-rootring-r3-sonnet-model/`。
- 攻方（Opus）：`research/prompts/m3-close-rootring-r3-opus-output.md`，sha256 d39f7f08f63d6acc1874f92ab169b3d9d6ce61fcf70d88be671d3fb6cca61380。模型 `research/prompts/m3-close-rootring-r3-opus-model/`。

这是最终轮：不再开第四轮。这一轮腿提的改法与主 agent 合成的组合都算被攻过零轮，写进第三节的交用户表。

开工快照：`research/prompts/m3-close-rootring-r3-snapshot/`（crates-sha256.txt 于 2026-10-21 事后补：取 511fe067b（2026-10-21）上的版本，腿派出时没记）

## 一、核查

1. 引文：`cite-check.py <两份报告> --root . --background <背景材料>` 末行「✓ 核了 54 处引文、23 处裸行号，对上 54 处，没判 23 处（其中裸行号没有引文可比 23 处；逐处列在上面）」。裸行号里判决要用的几处主 agent 逐行现查：`crates/singlefs-core/src/allocator.rs:2611`（`occupancy.effective_floor,`）、`allocator.rs:1266`（「挂载时生效的 F。抬 F 生效之后不跟着抬」）、`crates/singlefs-core/src/recovery.rs:1877`、`crates/singlefs-checker/src/walk.rs:9795`（`fn judge_blocks_referenced_by_abandoned_roots(`）、`walk.rs:18297`（I-3.1 在没有记账树时报不适用）、`crates/singlefs-core/src/mount.rs:8023`、`mount.rs:8029`、`crates/singlefs-core/src/instance_table_sweep.rs:721`、`crates/singlefs-core/src/admission.rs:675`，都对得上。辩方写的 `.claude/kb/decisions/22-单元原子性怎么合成.md:341`（`physical_block_size` 区间）行号错，那一句在同一文件第 74 行（「`physical_block_size`）∈ [根记录宽 457 字节（已定项 7）, 槽距]」），内容对。
2. 快照：在仓根 `sha256sum -c research/prompts/m3-close-rootring-r3-snapshot/kb-sha256.txt` 9 份全部 OK。
3. 攻方复跑：开工拷贝拷进 `~/.local/share/singlefs/work/m3-close-rootring-r3-main-checks/`，经 2G 包装跑它的 `tools/rerun.sh`，退 0：`narrow.out same=yes 43 行`、`zero.out same=yes 14 行`，第二轮 `same.out` 交叉核 `agree=26`。
4. 辩方复跑：开工拷贝拷进 `~/.local/share/singlefs/work/m3-close-rootring-r3-main-checks/frozen-sonnet`，经内存包装跑它的 `tools/rerun.sh`，退 0：

```
name=rerun_compare file=sweep.out same=yes lines=5
name=rerun_compare file=narrow_r2.out same=yes lines=27
name=rerun_compare file=formula_red4.out same=yes lines=15
```

5. 第二轮正推腿的模型主 agent 复跑仍在排队；辩方在自己的副本上复跑了第二轮 Y1（122 张镜像、2094 个根、裸公式拒 19、例外拒 0），对上。第二轮正推的数仍标「正推副本、辩方副本复跑对上，主 agent 未复跑」，主 agent 复跑出来补进第二轮判决第一节。
6. 核不动：C773 合成形态的转绿、误红都没有入库镜像（checker 与根记录的改动没打进任何副本），两腿都标推的；C773 登记那一形的复现目录 `~/.local/share/singlefs/work/i31-abandon-investigation` 已不在，「那 44 槽不被被抛弃根引用」这一前提核不动。

## 二、逐格判定

| 笔 | 三轮 | 这一轮 | 判定 |
|---|---|---|---|
| C743 补第 0 代例外 | 第一轮打中（裸公式）→ 改；第二轮没打中；第三轮辩方站住，攻方没攻 | 辩方：合法写者只有发布路径与 mkfs（读码）；手摆根旧用例开公式核 22 个目标里 4 个目标 7 条变红（量过，`formula_red4.out`，第一轮报的 3 个少数了 1 个），checker 档另有 6 个手摆根文件没跑。D22 实例代号行（`.claude/kb/decisions/22-单元原子性怎么合成.md:140`「择新在 checkpoint_txg 平局时按它高者赢（设备失而复得会造出两条同 txg 的合法根…」）与已定项 16 公式句冲突，要改的是 :140 那句；同源句另住 `.claude/kb/decisions/16-发布语义.md:184`、C88、C85 与里程碑九二·3 | **站住**。写回时 D22:140 与四处同源句一起改；实现批把 4 个目标 7 条旧用例改成照公式摆根，checker 档 6 个文件随实现批核 |
| C751 收窄形态 | 第一轮、第二轮都打中；第三轮收窄形态被两腿各自打中（同一形） | 攻方 `out/narrow.out` 原行「`R3NARROW shape=sweep crash_after_root=(3,24) … witnessed=(3,22) today rec=Ok(3,22) … narrow: f2=(3,23) shares_chosen_table=true rec=Fail@(3,23) …`」，辩方 `out/sweep.out` 原行「`R3SWEEP fault=table+cfg chosen=(3,24) … today rec=Ok(3,22) … union/narrow: rec=Fail@(3,23) …`」，两份主 agent 都复跑对上：同一实例里清扫发布（`instance_table_sweep.rs:721` 用同一实例重写实例表）落在见证与所选根之间，收窄形态收进清扫发布 (3,23)，它与所选根共用读不出的新表，恢复由 `Ok(3,22)` 变 `Fail@(3,23)`。第二轮 `cross` 26 格、登记形、`:656` 上收窄形态不差于今天（量过） | **打中成立**（三轮里三次打穿）。收窄形态不写回。交用户在两个零轮改法与「承诺外」之间定，见第三节 |
| C755 并集只判退 | 第一轮打中；第二轮站住；第三轮辩方站住 | 辩方：16 → 46、0 处挂不上复跑对上；「改后成功 ⊇ 今天成功」要靠「退不成就留在所选版」那一句，判决与正文都没写，双故障下才分叉（推的）。两腿一致：「三处交 F2 的记录同一份」说大了，「各重扫」给的是同一种来源的三份（读码，`recovery.rs:7124`、`mounted_read.rs:628`、`mount.rs:8023`）；单故障加一次断电之内重扫不改落点（推的）。攻方另指：可写挂载里 F2 认的见证与建退到那一版取的末条记录（`mount.rs:8029`）可以出自不同两遍 | **站住**，条文补三句：「退不成就留在所选的那一版」；「记录同一份」改成「同一种来源：触发之后重扫一遍、用重扫的那一份」；可写挂载里 F2、建退到那一版要的那条记录、F1 再判同用触发之后那一遍。三句都是零轮措辞，交用户认 |
| C773 合成形态 | 第一轮、第二轮都打中；第三轮辩方站住带缺口，攻方条文层打中两处 | 攻方 Z4.2（前提量过 `out/zero.out`，后果推的）：写者的 t 取哪个数没写；分配器记根时的门槛取挂载时的 F（`allocator.rs:1266`、`:2611`），正常卸载抬 F 之后每一条根 I-7.18 求等误红。攻方 Z4.3 与辩方 Z4.3（推的）：零单元发布与无文件版本上的复制根，t、p 抄自上一版，环转满后 I-7.18 误红、I-7.19 也可能误红；I-3.1 今天在没有记账树时报不适用（`walk.rs:18297`）。辩方 Z4.4（推的）：坏实现判得出三类、判不出两类（钉着的槽重读读出有效根而没解钉、凭空钉一个读得出的槽），泄漏有界（最多 3S 次发布）。C684 那一形两腿都推「不并、照旧红」，与用户定案一致 | **形态站住，带零轮补法**：t = max(这条根自己带的 F, 分配器记这条根时的回收门槛)；I-7.18、I-7.19 只判建账的发布，最新根下面没有记账树时报不适用；两类判不出的坏实现记成已认代价。交用户认，见第三节 |

格式件补漏照收（读码，命令数的）：根记录 457 → 466、journal 新根段 364 → 373、记录头 487 → 496；要同步的有 `crates/singlefs-format/src/lib.rs` 三个常量、引它们的 17 份 `.rs`、GPU 判器 2 份 WGSL、`.claude/kb/layout/01-first-txn.md` 三语的「457 / 512 槽」行、D23 已定项 15 施加记录拼根的字段数（「七个字段」改九个，I-8.10 逐字节比的段数同改）、D22 第 74 行 `physical_block_size` 区间下界改 466。I-3.1 折中并法要 checker 走被抛弃根，现成底子 `walk.rs:9795`；GPU 判器那一侧没有现成件。

## 三、交用户

| 笔 | 候选 | 被攻过几轮 | 代价与出处 |
|---|---|---|---|
| C751 | 甲：只在「见证到的那一版与所选根共用同一个实例表指针」时，F2 才收同实例的根（攻方 Z2.4） | 零轮（攻方在自己的模型输出上逐格推过，没改模型重跑） | 推的：登记形只读修到 P；`cross`、`:656`、清扫那一格照今天；两样都在根记录里，不读表 |
| | 乙：候选与所选根的实例表指针不同才收（辩方 Z2 修法一） | 零轮（没量） | 推的：清扫那一格修到；登记形的只读收益也没了（R、P、Pm 共用一张表） |
| | 丙：退回「承诺外，并进 F2 的代价」，不改代码 | 第二轮判决已列 | 登记形只读照今天越过 P 落 Pm（第一轮量过） |
| C755 | 补三句（第二节 C755 那一格） | 零轮措辞 | 单故障下结局不变（推的）；双故障下「留在所选版」保住只读（推的） |
| C773 | 补三样（第二节 C773 那一格）：t 的取法、I-7.18 / I-7.19 只判建账的发布、两类判不出的坏实现记成已认代价 | 零轮 | `physical_block_size` 大于 512 时根槽撕在扇区边界、I-7.18 会不会红没人核（辩方 Z4.3，推的），实现批的用例要钉这一格 |

C743 不交用户，照第二节写回。

用户 2026-10-20 弹窗定（原话在 `records/2026-10-18-里程碑三收尾与近期欠账.md` 末尾）：C751 取甲；C773 认 t 的取法与只判建账的发布两样，两类判不出的坏实现不记成已认代价、另立一笔欠账做成判得出；C755 三句都补。

## 四、没进任何一格的，去向

两腿 Z5 合起来列的项，逐条去向（「下一批实现」指根环这一簇在系统配置落地补丁合入之后的实现批）：

| 项 | 出处 | 去向 |
|---|---|---|
| 别的自证结构（系统配置槽、journal 记录、单元）的读错位 | 第一轮正推 X1.2，辩方 Z5.2 | 新欠账登记进 `.claude/kb/checks-owed.md`，不在本轮开题 |
| 表读不出、见证到的那条根读不出而区间里有同实例的根：今天不退，改后退到同实例的根 | 第二轮正推 Y2.4，两腿 Z5 | 随 C751 定案：选甲或乙时下一批实现钉一条用例 |
| GPU 判官要搬的：C751 一臂、C755 并集、C773 三条新判 | 两腿 Z5 | 下一批实现，与 CPU 一侧同批；没搬之前对拍用例写射程句 |
| E165 读槽对拍加公式核后重跑 | 第一轮正推 X1.7 | 重型，实现批提交时跑 |
| `BadRootRingSlotReading` 复用 `NotSelfVerified` | 第一轮正推 X1.7 | 实现取舍，先复用 |
| I-7.17 红的镜像上 checker 其余不变量不作保 | 第一轮正推 X5，第二轮 Y5.2 | 写回 I-7.17 那一行 |
| 「并集」里记录（末条记录读不出）算不算 | 第一轮攻方 X3.3 | 随 C755 写回：写明算（第二轮正推含记录那 4 格 0 少挂上，正推副本） |
| D19 已定项 5 回看：没见证的 R 上数据单元读不出，可写挂载由「照常挂 R」改成退回上一版 | 辩方 Z5.2 | 写回时 D19 已定项 5 加「没见证时除外」，回看表改 |
| C755 取并集时两种挂载判退前先跑走读的耗时 | 第二轮攻方 Y3.2 | 实现批量，不挡定案 |
| I-7.19 在两样故障下的误红（写失败而实际落了盘） | 第二轮正推 Y4.6 | 超出单故障，记进 C773 射程 |
| 被抛弃根独占又被钉扣着没回收的槽在合成形态下红 | 第二轮正推 Y4.4 | 与 C684「不并」同一取舍，写进 C773 射程 |
| C773 登记形 44 槽是否被被抛弃根引用 | 两腿 Z4 | 核不动（复现目录不在）；下一批实现用攻方 `zero.out` 那一形（同机理）当回归用例 |
| 第二轮正推 Y1、Y3 的数 | 辩方 Z5.1 | 主 agent 复跑排队中，出来补进第二轮判决 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D17（实现分层与第三方管道） 已定项 1 | 备料 | 2026-10-20 不受影响：C743 错位读条文定了，写回批写进失败模型 |
| D22（单元原子性怎么合成） 已定项 7 | 备料 | 2026-10-20 不受影响：实例代号行平局句与根记录宽度随写回批改，待 C773 用户认 |
| D16（发布语义） 已定项 1 | 备料 | 2026-10-20 不受影响：回收门槛落盘形态待用户认 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-20 不受影响：F2 段条文待 C751、C755 用户定 |
| D23（journal 的角色与格式） 已定项 15 | 备料 | 2026-10-20 不受影响：施加记录拼根字段数随 C773 写回 |
| D19（块指针的结构与宽度预算） 已定项 5 | 备料 | 2026-10-20 不受影响：「没见证时除外」随 C755 写回 |
