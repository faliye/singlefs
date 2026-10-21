# m3-close-rootring-r3 云端辩方腿报告（Sonnet）

日期 2026-10-20。立场：正文第四节「云端辩方」——复核前两轮判决够不够得着、是不是同样打中所有替代方案；替前两轮被判出局的一方（第一轮「统一成败」「只读照报失败」「改式不看引用」，第二轮「同实例一律收」）找它站得住的理由。方案按 `crates/` 今天的实现谈，系统配置落地补丁没打。

## 约定

- 「量过」：我在草稿目录的仓副本上跑的数，不是入库装置上的数。副本 = 开工时 `crates/` 的拷贝（`.rs` 清单再取 sha256 是 b523d85891b7327b57d1a760417241421b4749bd7891d00f019189826e375eb2，与第二轮攻方的开工拷贝相同，即 `crates/` 自第二轮起没变），其中第二轮正推腿的核心层开关补丁（`research/prompts/m3-close-rootring-r2-sonnet-model/tools/core-switch.patch`）只在要开公式核的几格里用，环境变量不设时等于今天。
- 「推的」：读码或算术，没跑。「零轮」：这一轮新冒出的形态，不为它再开一轮。
- 判定三档：站住（判决够得着、替代方案不被同一句打中）/ 够不着（判决引的依据对不上它说的那件事）/ 同样打中别的臂（判决打中被判出局那一方的理由，对留下的形态也成立）。
- 没读禁读清单；引 kb 条款写 kb 文件名加那份文件自己的行号，行号在文件里现查。

## 复跑与 sha256

```
cd /home/fy5090/code/singlefs && bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-rootring-r3-sonnet-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-rootring-r3-sonnet/frozen --work <新的空草稿目录>
```

`--frozen` 是上面那份开工拷贝（只含 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`，约 35M）。我在新目录里完整跑过一次，退 0，原样三行：

```
name=rerun_compare file=sweep.out same=yes lines=5
name=rerun_compare file=narrow_r2.out same=yes lines=27
name=rerun_compare file=formula_red4.out same=yes lines=15
```

| 文件 | sha256 |
|---|---|
| `tools/rerun.sh` | 7ef1b31a8a8988dd6f2f6336ff45d974b35153607300aef88fac7f5c9f692912 |
| `tests/rootring_r3_defense_sweep_shape.rs` | bdecf8891b50de3d06e98b2bbeb51b10d1634466cdc353ed9536055173e0831b |
| `tests/rootring_r3_defense_narrow_on_the_r2_shapes.rs` | 56850bdee403c23abfb1a9bc21a07656692aca11f061932b537e94bcdaeb8bfc |
| `tests/rootring_r3_common/mod.rs` | 560ceabe57c2ece0cf82f55be6301eb5b073a916092f683973c63a248540f240 |
| `out/sweep.out` | 3a81f6998db35bd545217ace2474a7750deccae6d9af6d9b7e990925c19663a9 |
| `out/narrow_r2.out` | 5a23ee6de922b2cc24f726161ae4bebb75629bd6101f92279af220c5de14e990 |
| `out/formula_red4.out` | 19046310b3cbf9aeee0b9aa6a52ed630b0a5062b49caeb2b5a214f834419ae58 |

模型目录 `research/prompts/m3-close-rootring-r3-sonnet-model/`。`tests/rootring_r3_common/mod.rs` 从第二轮攻方模型的共用件拷来，只把 F2 照抄判法的「同实例」开关由 bool 改成三档（0 今天 / 1 同实例一律收 / 2 只在见证到的那一版与所选根同实例时才收）。

另：第二轮正推腿的模型在这份副本上也复跑过（`rerun.sh` 退 1，原因是它抽输出的 `grep '^Y1 '` 漏掉被 `test … ... ` 前缀吃住的每份第一行；其余行逐行相同）：`Y1_SUMMARY images=122 self_verified_roots_seen=2094 rejected_by_bare_formula=19 rejected_with_genesis_exception=0 images_whose_outcome_differs_under_bare=0 images_whose_outcome_differs_under_exception=0`；`out/y3.out`、`out/derive_y3.out` 的 sha256 与原件逐字相同（f5c70d9d…、3f5c1f26…）。

## 各格判定一览

| 判决条目 | 辩方判定 | 一句话（证据在下文各节） |
|---|---|---|
| 第一轮 X1 / 第二轮 Y1 C743 补第 0 代例外 | 站住 | 合法写者只有发布路径与 mkfs（读码）；122 张合法镜像 2094 个根例外拒 0（复跑对上）；22 个手摆根 harness 目标里 4 个目标 7 条用例变红（量过，比第一轮报的「3 个」多 1 个目标）；checker 档另有 6 个手摆根文件没跑 |
| Z1 D22 平局括号与公式句 | 判决够不全 | 要改的是 D22:140 那句；同一个「设备失而复得 ⇒ 两条同 txg 合法根」另住在 D16:184、checks-owed:80（C88）、checks-owed:786（C85），判决只点了前一处 |
| 第一轮 X3「统一成败」出局 | 站住，但引的两句够不着 | 量过的 34 / 50 格站得住；撞的 D19:115（数据单元）与 D23:421（记录）是对的；D23:392、D23:396 说的是别的闸，不是所选那一版的读者 |
| 第一轮 X2「只读照报失败」出局 | 站住 | 它与 `:656` 的冲突是真的；「`:656` 在单故障承诺之外」对 C751 登记那一形同样成立，替它辩不出差别 |
| 第一轮 X4「改式不看引用」出局 | 同样打中别的臂 | 「判不出钉低不解开」对合成形态同样成立（四类坏实现里它只罩两类）；出局的真依据是用户对 C684 的定案，不是判别力 |
| 第二轮 Y2「同实例一律收」出局 | 站住；它的前提对、结论错 | 「同实例的根按表恒不被抛弃」读码核实（crates/singlefs-core/src/recovery.rs:1877）；错在「同实例 ⇒ 共用读不出的那张表」。复跑 26 格：一律收 8 处由成变败，收窄形态 0 处 |
| 第二轮 Y3 并集只判退 | 站住，带两处判决没带的条件 | 16 → 46、0 处挂不上复跑对上；「改后成功 ⊇ 今天成功」的证明要「退不成留在所选版」那一句，判决与正文都没写；「记录同一份」在瞬时读故障下说大了 |
| Z2 C751 收窄形态 | 站住一半 | 第二轮攻方 26 格、登记形、`:656` 上落点量过，0 处比今天差；但清扫发布在同一实例里重写实例表，收窄形态在这一形上让恢复由成变败（量过，零轮） |
| Z3 C755 记录同一份 | 说大了 | 单一持久故障下重扫不改落点；瞬时故障下三处各扫一遍，逐条相同不保证 |
| Z4 C773 合成形态 | 站住，带三处缺口 | 字段宽度与落点算术对上；I-7.18、I-7.19 少了「只判建账的发布」一句，无文件版本上的复制根会误红；坏实现判得出两类、判不出两类；折中并法要 checker 走被抛弃根（已有 `crates/singlefs-checker/src/walk.rs:9795` 可用） |
| Z5 清单与数 | 数能核；清单有漏 | 50 / 100 格、122 / 2094、16 → 46 都对上；137 902 不是不同镜像数；漏列项见 Z5 |

## Z1　「已定」项撑不撑得住

### Z1.1 C743 补第 0 代例外：第二轮「站住」的理由够不够得着

第二轮说的那件事是「合法写者写出的根都过公式核加第 0 代例外，核只在读错位时起作用」。逐句现查：

- 写根槽只有一个落盘步骤：`crates/singlefs-core/src/transaction.rs:1533`「writer.perform_commit_step(CommitStep::WriteRootRecordForceUnitAccess {」，步骤里取槽的一行是 `crates/singlefs-core/src/transaction.rs:508`「let target = target_for_publish(」；`root.to_slot(` 的其余非测试调用点（`transaction.rs` 的三处发布计划）都装进 `PublishWrites`，经同一个步骤落盘。主工作区里另一处写根的 `root_left_newest.to_slot(root_slot_bytes)` 在 `crates/singlefs-core/src/mount.rs` 的 `#[cfg(test)]`（`crates/singlefs-core/src/mount.rs:9355`）之后，是测试代码。mkfs 第 0 代：`crates/singlefs-core/src/make_filesystem.rs:621`「for ring_slot in genesis_root_ring_slots() {」，槽集合是 `make_filesystem.rs:677`「(0..ROOT_RING_REGIONS).map(|region| RootRingSlot { region, slot: 0 })」。
- 公式与写者是同一个函数：`crates/singlefs-core/src/root_ring.rs:114`「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」。checker 已经按同一个例外判：`crates/singlefs-checker/src/image.rs:1272`「let (expected_region, expected_slot) = if txg == GENESIS_ROOT_CHECKPOINT_TXG {」；`crates/singlefs-checker/invariants/02-根环.md:24`「时只判 s == 0，不判 r」。
- 复跑第二轮正推的数（上面「复跑与 sha256」末段）：122 张合法镜像、2094 个根，例外拒 0，裸公式拒 19，三处结局与今天不同的镜像两种都是 0。数能核。
- **我补量的（第二轮正推列为「没跑」）**：把第二轮的开关补丁装到副本，环境变量 `R2_FORMULA=2`（带例外）下逐个跑「手摆根」候选。第一轮报的 22 个 harness 文件（命令原样重跑得 22）里，`common/mod.rs` 被几乎所有目标包含、没单独跑；另 21 个是 15 个顶层目标加 6 个子模块（各有一个同名顶层目标），再加 F2 用例文件一个，共跑 22 个测试目标（名单在草稿目录 `targets.txt`）：18 个目标全绿，4 个目标变红，共 7 条用例；开关 0 下这 4 个目标全绿。原样（`out/formula_red4.out`）：

```
target=core_review_tree_table_duplicates_and_slot_one_search mode=2 test result: FAILED. 12 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
target=corrupt_on_disk_content_is_refused_instead_of_panicking mode=2 test result: FAILED. 42 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
target=root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot mode=2 test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
target=root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance mode=2 test result: FAILED. 2 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out
```

  第一轮正推只核实了 3 个目标；多出来的是 `corrupt_on_disk_content_is_refused_instead_of_panicking` 里 `an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area_refuses_the_writable_mount_instead_of_panicking`。这 7 条都是把根手摆进非公式槽的用例，不是合法历史上的根被误判。**没跑的**：`crates/singlefs-checker-tier` 里同时含 `slot_offset(` 与 `to_slot(` 的 6 个文件（`e158_root_choice_repair.rs`、`e165_replay_bounded_enumeration.rs`、`e170_o3_spec_executor_export.rs`、`common_allocation_record_trees_copied_under_other_generations/mod.rs`、`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs`）属 checker 档，按定义不跑；第一轮「22 个候选」只数了 harness，这 6 个同样手摆根，要主 agent 在实现批里一并核。

判定：**站住**。推翻现象：合法历史上出现一条开关 2 拒掉的根（第二轮的 `Y1_SUMMARY` 非 0，或 `formula_red4` 之外的 harness 目标变红且盘面是合法写者造的）。

### Z1.2 D22 的两句并排，公式核之后哪一句要改

两句（行号现查）：`.claude/kb/decisions/22-单元原子性怎么合成.md:140`「设备失而复得会造出两条同 txg 的合法根」；`.claude/kb/decisions/22-单元原子性怎么合成.md:335`「无状态、只由 txg 与 (R, S) 决定」。

判定：**要改的是 140 那一句，335 留着**。依据：

- 335 的公式已经由 I-7.17 在 checker 里判成红线（`crates/singlefs-checker/invariants/02-根环.md:24`，实现 `crates/singlefs-checker/src/image.rs:1272` 一带）：同 txg 的根住进别的槽，镜像上本来就是红。改 335 等于推翻已实现的 I-7.17，不止改一句话。
- 140 的前提「同 txg 两条根并存」在公式下造不出：一个 txg 只有一个公式槽，区域落在哪块盘由 mkfs 写死（`crates/singlefs-core/src/make_filesystem.rs:63`「pub const FIRST_VERSION_REGION_DEVICES: [DeviceIdentity; 3] =」），丢盘期间新时间线写不进那块盘上的区域。我另按两盘三区域走了一遍丢盘回退（推的）：旧时间线在丢失盘上比幸存根最多多一个 txg，新时间线第一个落在丢失区域的 txg 写败、推进一格，所以新线的 txg 恒高于旧线留在丢失盘上的那一条；盘回来之后同 txg 的写落在同一个槽，后写的盖掉先写的。与前两轮「没找到走得到的盘面」一致，不是新证据。
- **判决没点到的同源句**：同一个「设备失而复得 ⇒ 两条同 txg 合法根」还写在 `.claude/kb/decisions/16-发布语义.md:184`「丢盘回退 + 盘失而复得会造出两条同 txg 的自证合法根」、`.claude/kb/checks-owed.md:80`（C88）「设备失而复得会让根环里并存两条同 checkpoint_txg 的自证合法根」、`.claude/kb/checks-owed.md:786`（C85 的连带行），以及 `.claude/kb/milestone/09-ninth-txn.md:59` 里「时间线判别」这个作业项名。只改 D22:140 会留下 D16 与 C88 两处与公式矛盾的原话。
- 我没核的：C88 的另一半（checker 识别「已丢弃时间线」本身）。上面只核了平局；盘回来之后旧线留下的更高 txg 根是否压过新线，我按两盘三区域算是不会（旧线最多多 1 个 txg，见上），但别的盘数、区域归属没算。C88 整条销不销，由主 agent 判。
- 择根键 `(checkpoint_txg, 实例代号)` 不用动：平局只剩第 0 代三份同字节，实例代号在别处（抛弃判断）仍有用。入库的平局用例（上面 4 个红目标里的 `root_choice_on_a_tied_…`）要按 D22 改后的语义重写。

### Z1.3 C755 并集只判退：第二轮「站住」的理由够不够得着

第二轮说的那件事是「取并集只判退，成败各看各，没有一处比今天少挂上；三处同版 16 → 46」。

- 数能核：在这份副本上复跑第二轮正推的模型，`out/y3.out`、`out/derive_y3.out` 与原件逐字相同；推算表里「unwitnessed any 严」与「宽」两行都是「三处都成且同版=46 成的几处同版=50 今天成改后不成=0处(0格) 换版=42处」（`research/prompts/m3-close-rootring-r2-sonnet-model/out/derive_y3.out:20`、`:21`），108 格校准 0 格对不上。
- 判决够得着的：取走读或并集，100 格里 0 处少挂上；单取打开 / 交集 52 处（30 格）、单取建上一版 8 处（8 格）少挂上（`research/prompts/m3-close-rootring-r2-opus-model/out/unify.out` 的 `R2UNIFY_TALLY` 行，主 agent 已复跑对上）。「触发必须是并集」这一步我也核了推理：今天任何一处靠退回挂得上，当且仅当它自己的读者在所选根上不通，所以触发集合要罩住每一处自己的不通，只有并集罩得住。
- **判决与正文都没带的条件一：「退不成留在所选版」**。第二轮正推的证明（`research/prompts/m3-close-rootring-r2-sonnet-output.md` 的 Y3.1 末段）靠这一句才有「改后成功 ⊇ 今天成功」；没有它，触发之后退到的那一版自己的读者读不通，就会把今天靠留在所选根挂得上的一处变成失败。正文 C755 行写的是「落到那一版之后成不成各照今天」，不含这一句。单故障下两种写法逐格相同（`y3.out` 里 `retreat_target_failure=report_failure` 与 `stay_on_chosen` 的 `Y3_SUMMARY` 行除标签外相同），所以 100 格的数不分辨它；它只在双故障下分叉，例（推的，没造盘面）：R 的数据单元两份读回全零（走读败、打开与建上一版不败）加 P 的 inode 树根单元读回全零，只读挂载今天打开 R，无保底的并集触发退到 P、P 打不开、照报失败。
- **条件二：换版的 42 处都在「R 没见证」的盘面**；R 已确认而较新系统配置槽读回全零（两样故障）的盘面上，第二轮攻方量到同样的只读 34 格、可写 8 格落回上一版（`research/prompts/m3-close-rootring-r2-opus-model/out/unify.out:175`、`:176`）。这笔代价 kb 里已经认了且写的是三处：`.claude/kb/decisions/23-journal的角色与格式.md:423`「恢复、可写挂载与只读挂载悄悄读回上一版」。所以这一格撑得住，但判决该引这一句，不是只说「第二轮正推的 42 处全是未见证」。
- **条件三：「记录同一份」说大了**，见 Z3。

判定：**站住**（数与推理都对上），条文要补上面三处。推翻现象：找到一个盘面，并集触发加保底之后某一处由成变败；或不注读故障的崩溃态上并集失败而走读成功（别的会话量过 0，见 Z5 末项）。

### Z1.4 替被判出局的四条臂辩护

**统一成败**（第一轮出局）。它最强的理由：D23 的 F2 触发词本来就是「所选根走读失败」（`.claude/kb/decisions/23-journal的角色与格式.md:398`「所选根走读失败、系统配置没见证它时退回见证到的那一版」），C755 登记行的第一条路「管到触发条件」是用户选的，统一之后三处结局一致、没有「恢复报失败而只读照常」的分叉。**判定：站住，但判决引的两句够不着一半。** 量过的后果（`research/prompts/m3-close-rootring-r1-opus-model/out/union.out` 我重新数：已确认盘面 50 格、今天只读落所选根 34 格、`mount_lost=true` 34 格；没见证盘面 `mount_lost=true` 4 格）是真的，零断电、单故障，少挂上。撞的 kb 句子里，`.claude/kb/decisions/19-块指针的结构与宽度预算.md:115`「读不出的数据单元照抄它的位置项、不读内容，挂载照常，读到那个文件时才报错」（可写 8 格，数据单元）和 `.claude/kb/decisions/23-journal的角色与格式.md:421`「只读挂载照常择 (实例 1, txg 4)」（记录那一格）是对的。判决还引的 `.claude/kb/decisions/23-journal的角色与格式.md:392`「仍不成在取号之前拒可写、只读照常」说的是被抛弃根的账读不出，`:396`「仍为真就在取号之前拒可写挂载、盘上逐字节不变，只读挂载照常」说的是见证判不出时的可写挂载闸；两句都不是所选那一版的读者，只读 34 格（实例表、分配记录树、记账树这类元数据单元）真正的依据是量过的「本来挂得上的池挂不上」，不是这两句。结论不变，引文要换。

**只读照报失败**（第一轮出局）。最强的理由：改动只在只读一处，与 C751 登记行里被判不采的 `F2_NO_SKIP=2` 同类；`:656` 钉的是三样故障同时的盘面，而 C751 登记行自己对它的形写的是 `.claude/kb/checks-owed.md:503`「三样故障同时，在单故障承诺之外」——同一个预算，翻 `:656` 与不翻登记那一形是对称的。**判定：站住。** 对称性替它辩不出差别：`:656` 里 P 是对的版本，只读照报失败丢掉它，恢复仍退到 P，三处分叉；而收窄形态（Z2）在 `:656` 上一个字不动（复跑对上），所以这一条臂的冲突不会打到留下的形态。

**改式不看引用**（第一轮出局）。最强的理由：它就是记账本身的语义（释放代大于门槛的已释放槽，记账都算已分配），所以永不误红；checker 不再自己算门槛，I-3.1 一侧删掉约 60 行选根代码；C773、C684 两形一起转绿。判决打它的技术理由是「加上界之后判不出钉低不解开」。**判定：同样打中别的臂。** 合成形态（根记录加 t 与 p）在第二轮正推的坏实现表里只判出两类：恒取 F（I-7.18 求等）、写掉钉着的槽不解钉（I-7.19）；判不出两类：钉着的槽重读读出了有效根而写者没解钉、凭空钉一个读得出的槽（`research/prompts/m3-close-rootring-r2-sonnet-output.md` 的 Y4.5 表后两行，第二轮攻方 Y4.3 同判）。所以「判不出泄漏」不是改式独有的缺点，合成形态只多罩住一半；泄漏两边都有界（钉在这个进程写掉那一槽时必解，最多 3S 次发布）。**改式真正出局的依据是用户对 C684 的定案**（`.claude/kb/checks-owed.md:447`「读法甲：照现行原式，这一形归读不出的根那一类已知形态去判，测试钉成例外」，2026-10-20 定「不并」），改式让 C684 那一形转绿；判决写成「判不出」会让人以为合成形态没有这个缺点，应把理由改写成用户定案。

**同实例一律收**（第二轮出局）。最强的理由：前提是对的——抛弃的判法只看表里有没有这个根自己的实例的行（`crates/singlefs-core/src/recovery.rs:1877`「.any(|row| row.instance == instance && checkpoint_txg > row.selected_root_txg)」），行只写给比表的主人旧的实例（`.claude/kb/decisions/23-journal的角色与格式.md:406`「行只落在 [这个根的实例, 新实例)」），同实例的根按表恒不被抛弃；它还把登记形的只读由 Pm 修到 P。**判定：站住，但出局的理由是「共用读不出的那张表」，不是「被抛弃」。** 复跑第二轮攻方 26 格（`out/narrow_r2.out`）：一律收（`union/same_all`）与取法不动（`union/exact`，F2 只认恰好等于见证到的那一版，同样用并集触发）逐格比，8 处由成变败（`cross` 族 `crash_after_mount_root#2 fault=table+cfg` 的恢复与可写，四组各 2 处；可写那一列只量建上一版，不含读阶段先拒的那一道，第二轮攻方已注明）。这条臂的前提对、结论错；Z2 里的收窄形态把结论错的一半修了，但同一个错的根源（同实例 ⇒ 共用那张表）在清扫发布下仍成立。

## Z2　C751 收窄形态

收窄形态：所选根的实例表读不出时，只在见证到的那一版与所选根同实例时，F2 才把与所选根同实例的根当作不被抛弃。我在副本里给第二轮攻方的照抄判法加了第三档（`mode == 2`：`root.instance == chosen.instance && witnessed_publish.1 == chosen.instance`，`tests/rootring_r3_common/mod.rs`），原样重造第二轮攻方 `same.out` 那 26 格（`out/narrow_r2.out`，`replica_mismatches=0`：关着开关时与 core 的函数逐格交回同一条根），触发取并集。

**三个形上各落哪一版**（量过，副本）：

| 盘面 | 格数 | F2 目标：今天 / 一律收 / 收窄 | 并集触发下三处结局：收窄 vs 今天 |
|---|---|---|---|
| 第二轮攻方 `cross`（见证到的那一版属更旧的实例） | 20 | 20 格里 6 格一律收选的目标与今天不同：4 格（`crash_after_mount_root#2 fault=table+cfg`，ow 与 sw3 各 k = 3、5）一律收退到与所选根共用读不出的表的 Row，收窄与今天选同一条；另 2 格（k = 3 的 `crash_after_mount_root#3 fault=table+cfg`，ow 与 sw3 各一）一律收与收窄都多收一条同实例的根，今天 (2,8) 对 (2,9)，两边恢复都败 | 成→败 0 处，败→成 0 处 |
| C751 登记形（`registered`，R、P、Pm 同实例 1） | 4 | Pm / P / P（k = 3：(1,5) → (1,6)；k = 5：(1,7) → (1,8)） | 只读由 Pm 改落 P（修到了 C751 要修的越过 P）；恢复、可写在 P 上读同一张读不出的表，仍败 |
| 入库用例 `:656`（`pinned656`，见证到的 P 属实例 1、所选 Row 属实例 2） | 2 | 三档都落 P：(1,6) / (1,8) | 三处一个字不变 |

结论：第二轮攻方那一形上收窄形态不再让恢复由成变败；登记形上只读修到 P；`:656` 不动。判决「收窄形态是零轮改法」这一半站住。

**有没有一形，同实例见证而同实例的根其实已被抛弃**：被抛弃本身没有——抛弃只看表里这个根自己的实例的行（`crates/singlefs-core/src/recovery.rs:1877`），行只写给更旧的实例，同实例的根不会被同实例的表抛弃；唯一能造出的是实例代号重用（C555），它不分辨取法。**但收窄形态背后那句「同实例 ⇒ 共用读不出的那张表」有例外，例外就是清扫发布**（零轮，量过）：清扫发布用同一个实例重写实例表，`crates/singlefs-core/src/instance_table_sweep.rs:721`「instance_table: InstanceTablePlan::Rewrite(self.instance_table.clone()),」，文件头 `instance_table_sweep.rs:3`「把那一行删掉。」。于是同一实例里，清扫发布之前的根用旧表、之后的根用新表。

盘面（`rootring_r3_defense_sweep_shape.rs`，用入库的 `pool_with_a_dead_instance` 与 `sweeping_mount_on`）：实例 1 死了、实例 2 覆盖写到 txg 19 → 实例 3 可写挂载，末尾清扫发布删实例 1 的行 → 同一次挂载再覆盖写一版 R → 崩在 R 的根槽之后、轮换之前。读的那一刻 R 的实例表两份读回全零，再加每块盘较新的系统配置槽读回全零（与登记形同一个故障预算：一个单元两份、较新配置槽、一次断电）。`out/sweep.out` 原样：

```
R3SWEEP roots before_sweep=(3, 22) sweep=(3, 23) r=(3, 24) table_same(before,sweep)=false table_same(sweep,r)=true
R3SWEEP fault=table+cfg chosen=(3,24) table_readable=false today rec=Ok(3,22) ro=Ok(3,24) rw=Fail | f2_core=(3,22) f2_exact=(3,22) f2_same_all=(3,23) f2_same_if_witness_same_instance=(3,23) replica_matches_core=true target_shares_the_chosen_table(exact,all,narrow)=(false,true,true) | union/exact: rec=Ok(3,22) ro=Ok(3,22) rw=Ok(3,22) | union/same_all: rec=Fail@(3,23) ro=Ok(3,23) rw=Fail@(3,23) | union/narrow: rec=Fail@(3,23) ro=Ok(3,23) rw=Fail@(3,23)
```

读法：见证到的那一版是清扫前的 (3,22)，与所选根 (3,24) 同实例，收窄形态的条件成立；它收进的 (3,23) 是清扫发布，与所选根共用读不出的那张表，恢复由今天的 `Ok(3,22)` 变成 `Fail@(3,23)`。可写那一列只量建上一版，今天读阶段就拒（`rw=Fail`）。只有 `fault=table`（没有配置槽那一样）时见证到的就是清扫发布，两种取法都败，不分辨。**这是收窄形态在 C751 自己的故障预算里的一处退化**，不是「同实例的根被抛弃」；Z2 的问法没点到，按零轮记。

可选的修法（零轮，没量，推的）：(一) 条件里加「候选与所选根的实例表指针不同」——根记录里就有指针，不读表；代价是登记形的只读收益也没了（R、P、Pm 共用一张表）；(二) F2 退到的目标自己的读者读不通就再往下找一条——要改成按各处自己的读者判，改动比一行大。要不要修、修哪条，主 agent 与用户定。

判定：**站住一半**——第二轮攻方的 26 格、登记形、`:656` 上都量过不差于今天；清扫发布那一形不站住。推翻「清扫形」的现象：清扫发布不是用 `InstanceTablePlan::Rewrite` 发的根（读码已核 `instance_table_sweep.rs:721`），或 `sweep.out` 复跑不再出现 `table_same(sweep,r)=true` 且 `union/narrow` 的恢复败。

## Z3　C755 记录同一份

- 今天三处交 F2 的记录来路不同（读码，行号现查）：恢复在走读失败那一刻重扫：注释在 `crates/singlefs-core/src/recovery.rs:7115`「走读失败才再扫一遍环拿记录给退回的判法」，调用在 `crates/singlefs-core/src/recovery.rs:7124`「&scan_journal(reader, &system_configuration),」；只读挂载用挂载开头扫的那一份，`crates/singlefs-core/src/mounted_read.rs:628`「let records = scan_journal(reader, &system_configuration);」；可写挂载把读阶段的 `&records` 传给 F2（`crates/singlefs-core/src/mount.rs:8000` 一带的 `match rebuild_previous_version(` 之后）。
- 「触发之后各重扫」做成后，三处各调一次 `scan_journal`。**单一持久故障**（读不出的那片一直读不出）：`scan_journal` 对同一盘面是纯读，重扫与第一遍逐条相同，三处交 F2 的记录逐条相同。**瞬时故障**（读第一遍败、再读成）：三处各扫各的，读到第几次不同，逐条相同不保证。第二轮正推、攻方都写了这一点；正文那一句「记录同一份」说的是来源，该写成「触发之后各重扫一遍、用重扫的那一份」，不写「同一份」。
- 重扫多读的环会不会改变落点：持久单故障下不会（同输入同输出）；瞬时单故障下会，方向是纠正第一遍的漏读——例：计数器 = c_见证 的那条记录第一遍读不出，F2 因「那条记录读不出」不退（`.claude/kb/decisions/23-journal的角色与格式.md:398`），重扫读出来才退。这正是 C754 想要的，没有哪个方向比今天差；没造瞬时故障盘面（推的）。
- 可写挂载补丁里 F1 再判前那一次重扫（`~/.local/share/singlefs/work/m3-close-w8-sysconf-landing/patch/crates.patch`）可以和 F2 前这一次并成一次（第二轮正推 Y3.3 已列），不并也只是多读一圈环。

判定：**站住，措辞要改**（「同一份」→「同一种来源」）。推翻现象：持久单故障下重扫出的记录与第一遍不同（说明 `scan_journal` 不是纯读）。

## Z4　C773 合成形态（根记录加门槛 t 与钉槽 p，I-7.18、I-7.19，I-3.1 只并钉带出来的那一截）

这一格全是读码加算术，没有模型：checker 与根记录的改动没打进副本，造不出合成形态的镜像。凡「转绿」「会红」都是推的。

**Z4.1 C773 登记形转不转绿（推的）。** 登记行 `.claude/kb/checks-owed.md:523`「差的 44 槽释放代全在 (21, 25]」；复现盘面目录 `~/.local/share/singlefs/work/i31-abandon-investigation` 现在不存在（`ls` 报无此目录），只能照登记行与第二轮正推的推法重造：挂载把某槽钉在 21（钉值 = 3 × 槽 + 区域，`target_for_publish` 反推得到槽，所以 p 一个字节同时认槽与钉值；钉值 21 要求 S ≥ 8，登记行没写 S，这一条登记行没给的前提我没法核）；之后这个进程写的根带 t = 21、p = 21。checker：不带钉的现算门槛 t* = max(根自己带的 F, 环里最旧有效根 25) = 25；I-7.18 现算 max(F, min(25, 钉值 21)) = 21，等于 t，绿；I-3.1 把释放代落在 (21, 25] 的已释放槽并进，258 + 44 = 302 等于记账，绿。**前提**：这 44 槽不被任何被抛弃的根引用（被抛弃根引用着的不并）；登记行没写，复现盘面不在，核不动。

**Z4.2 C684 那一形的结局（推的）。** C684 那一形里换下某一槽的那一版的根在环里读不出，挂载时那一槽读坏，钉着，所以 p 不是 255，「钉带出来的那一截」非空；那一槽在最新根的账里还在 defer，只剩一条读得出的被抛弃根引用它，形态写的是「被抛弃根引用着的槽不并」，所以它不进那一截，I-3.1 照旧红，与 `.claude/kb/checks-owed.md:447`「读法甲：照现行原式，这一形归读不出的根那一类已知形态去判，测试钉成例外」一致。代价：checker 要拿被抛弃根的引用集合当排除项；这一步现成的底子是 `crates/singlefs-checker/src/walk.rs:9795`「fn judge_blocks_referenced_by_abandoned_roots(」（它已经为别的不变量走被抛弃的根），不是从零写。第二轮正推点的「被抛弃根独占、又被钉扣着没回收」的槽，形态已经写成「不并」，所以红，是明写的取舍，不是漏。

**Z4.3 合法历史上 I-7.18、I-7.19 会不会误红。**

- 跨挂载、取号之后、回退、卸载抬 F：这些发布在有文件的版本上全是建账的发布——`crates/singlefs-core/src/mount.rs:5023`「记账树已经存在 ⇒ 空发布也重写固定点单元」，卸载那一串走 `crates/singlefs-core/src/mount.rs:4813`「let published = publish_version_with_unmount_marker(」同一套；管理员回退是向前发布（`crates/singlefs-checker/invariants/02-根环.md:11`「管理员回退是挂着时的一次向前发布，不抛弃任何根」）。t 与 p 在每一次这样的发布里现算，最新一条根的环就是建账时的环，求等不误红（推的）。校验和罩整槽（`crates/singlefs-core/src/root_record.rs:115`「let digest = wide_checksum_with_field_zeroed(&bytes, slot_bytes, ROOT_CHECKSUM_OFFSET);」），新字节被罩。**没核的**：`physical_block_size` 大于 512 时根槽写可以撕在扇区边界，撕坏的槽自证不过，等于丢了它盖掉的最旧那条根，此时 checker 现算的 t* 比最新根建账时的环高一截，I-7.18 求等会不会红我没核（今天的 I-3.1 在同一状态上怎么判也没核）；入库的环洞测试池用 512（`crates/singlefs-harness/tests/common_root_ring_holes/mod.rs:45`「physical_block_size: 512,」），这一格没有被量过。
- **判决与正文都没带的缺口：无文件版本上的复制根。** 树表 0 条的版本上的零单元发布，`crates/singlefs-core/src/transaction.rs:1545`「空记录不点名任何单元、事务号 0、提交标记 1，新根段照上一版的根，根记录照上一版的根、只换 checkpoint_txg、实例代号、回退下界」——新字段只能抄上一条根的，t 与 p 属于上一个进程；那一版没有记账树（`crates/singlefs-core/src/admission.rs:675`），I-3.1 无物可比。只做过 mkfs 的池反复挂载卸载，环转过一圈之后现算的 t* 往上走，抄来的 t 不动：严格求等会红，I-7.19 的 p 抄来后也可能指到这个进程自己刚写掉的槽。第二轮正推的条文里有「这一版的账是这条根自己那次发布建的时求等，否则只要求 t 不高于 t*；I-7.19 只判建账的发布」（`research/prompts/m3-close-rootring-r2-sonnet-output.md` 的 Y4.3），合成时这一句掉了：正文 C773 行与第二轮判决的 Y4 行都是无条件求等。要补回。
- 写失败但落了盘、读故障加写失败的 I-7.19 误红：第二轮正推已列（两样故障，超出单故障），照收。

**Z4.4 「钉低不解开」「写掉钉着的槽却不解钉」能不能判出（推的，没造坏镜像）。** 沿用第二轮正推的坏实现表：恒取 F 判得出（I-7.18）；写掉钉着的槽不解钉判得出（I-7.19：那一槽现在住着本实例写的根）；p 写 255 但其实还钉着槽判得出（现算的 t* 没有钉值，更高，求等红）；**判不出两类**：钉着的槽重读读出了有效根而写者没解钉、凭空钉一个读得出的槽——这两类在镜像上与合法的钉逐字节相同（读故障不留在盘上）。泄漏有界：钉在这个进程写掉那一槽时必解，最多 3S 次发布。这两类该写进交用户表的「已认代价」，现在判决里没有。

**Z4.5 字段宽度、落点、同步件。**

- 算术（命令数出来）：`python3 -c 'print(457+9, 487+9, 364+9, (4096-496)//56, (4096-487)//56, 3*16)'` → `466 496 373 64 64 48`：根记录 466 ≤ 512，记录头 496 ≤ 512，点名项个数 64 不变，钉值最大 48 < 255 装得进 1 字节（S 上界 16）。
- 落点 428 起 9 字节：428 之前字段偏移都不动，`crates/singlefs-core/src/root_record.rs:13` 的 `ROOT_CHECKSUM_OFFSET` 仍是 138；改的常量是 `crates/singlefs-format/src/lib.rs:234`「pub const ROOT_RECORD_BYTES: u64 = 457;」与 `crates/singlefs-format/src/lib.rs:247`「pub const JOURNAL_HEADER_BYTES: u64 = 487;」及新根段 364（`crates/singlefs-format/src/lib.rs:244`）。
- 涉及面（我重数，与第一轮报的一致）：数字 457 在 `crates` 下 `.rs` / `.wgsl` 17 个文件、`.claude/kb` 34 个；487 是 23 与 66；364 是 8 与 51。
- **第一轮、第二轮没列的：** (一) 施加记录拼根的字段数：`.claude/kb/decisions/23-journal的角色与格式.md:465` 一带写的是「把所选根的这七个字段换成记录里的」，第一轮的字段只有一个，写「七改八」；合成形态是 t 与 p 两个字段，该是「七改九」，I-8.10 逐字节比的段数同理。(二) `.claude/kb/decisions/22-单元原子性怎么合成.md:341` 的区间「`physical_block_size` ∈ [根记录宽 457, 槽距]」要改 466。(三) GPU 判器里 I-7.18、I-7.19 和折中并法都要有一份（折中并法要走被抛弃根，GPU 一侧没有现成件，我没数处数）。(四) 上面 Z4.3 的复制根豁免两条判据也要进 GPU。

判定：**站住，带四处缺口**（复制根豁免、两类判不出的已认代价、字段数七改九、被抛弃根引用集合的 checker 工作量）。推翻现象：有文件的版本上出现一条非建账的根（则复制根豁免不止无文件一类，I-7.18 要更宽的豁免）；或 C773 登记形上这 44 槽被被抛弃根引用（则折中并法仍红）。

## Z5　未定项清单的完整性与数能不能核

### Z5.1 前两轮报出的数，在入库代码上核（副本）

| 数 | 出处 | 我怎么核 | 结果 |
|---|---|---|---|
| 第一轮 100 格；已确认盘面 50 格、今天只读落所选根 34 格、可写落所选根 8 格；没见证盘面 `mount_lost` 4 格 | 第一轮判决、攻方 `out/union.out` | 对入库的 `research/prompts/m3-close-rootring-r1-opus-model/out/union.out` 逐行数（`grep`、`awk`，不用模型）；主 agent 已复跑模型对上 | 100 行；已确认盘面 `ro==chosen` 34、`rw==chosen` 8、`mount_lost=true` 34；没见证盘面 34 / 8 / 4。对上 |
| 第二轮攻方 `unify.out` 183 行、`same.out` 27 行；`R2UNIFY_TALLY` 16 行（只读 34、可写 8、打开取法 30 / 22、建上一版 8 / 26） | 第二轮判决 | 数入库输出文件（`wc -l`、`grep -c`），主 agent 已复跑模型对上 | 183 行、`R2UNIFY ` 150 行、`rule=exact` 的 `TALLY` 16 行；`research/prompts/m3-close-rootring-r2-opus-model/out/unify.out:159`、`:160`、`:175`、`:176` 的 34 / 8 对上 |
| 第二轮 Y1：122 张镜像、2094 个根、裸公式拒 19、例外拒 0 | 第二轮正推；判决写「正推副本，主 agent 未复跑」 | 我在副本上跑它的 `rerun.sh`（见开头） | `Y1_SUMMARY` 逐字对上（122 / 2094 / 19 / 0 / 0 / 0）。**判决那一节「复跑出来之后补进」的欠账可以补了** |
| 第二轮 Y3：三处同版 16 → 46、换版 42、打开取法 52 处（30 格）、建上一版 8 处 | 第二轮正推 | 同一次复跑：`y3.out`、`derive_y3.out` 的 sha256 与原件逐字相同 | 对上 |
| 手摆根旧用例 22 个候选、已核实 3 个会红 | 第一轮正推 X1.7 | 第一轮的 `grep` 命令原样重跑；开关 2 下逐目标跑 | 22 对上；会红的是 4 个目标 7 条用例，不是 3 个（Z1.1）；checker 档另有 6 个同形文件没跑 |
| 457 / 487 / 364 的涉及面 | 第一轮正推 X4.6 | `grep -rlw` 重数 | 17 与 34、23 与 66、8 与 51，全部对上 |
| 别的会话量的「137 902 个不注读故障的崩溃态，三处各 137 902 次 F2 没被调到」 | 第一轮正推 X3.4 | 报告文件在，sha256 `c2f4ac242d70ca1c6ed960c3d070a76fd3a113b3120f866f568c14b96ed89a1f` 与第一轮引的相同 | 数在原报告里有；**原报告自己写「两档的态有重叠，不是 137 902 个不同的镜像」**（该报告第 66 行：quick 29 028 + hybrid14 108 874 相加），第一轮正推写成「137 902 个……崩溃态」措辞说大了；复跑是重型，没跑 |

### Z5.2 前两轮报出而没进任何一格（或进格时措辞丢了）的问题

四笔之外，我逐份读了四份报告，下面是没占到 Z1–Z4 任何一格、或判决合成时丢掉的项。「状态」写我的建议去向，不替主 agent 定。

| 项 | 出处 | 状态与建议 |
|---|---|---|
| 别的自证结构（系统配置槽、journal 记录、单元）的读错位 | 第一轮正推 X1.2 末行 | 没进格、没人查。D17 已定项 1 加的那句限定「根环槽」；写进交用户表标「零轮」，另立一题 |
| 「退不成留在所选版」（保底） | 第二轮正推 Y3.1、Y3.4 | 判决与正文都没带，Z1.3；单故障下无差别，双故障才分叉。写进条文或明写不写 |
| 无文件版本上复制根的 I-7.18、I-7.19 豁免 | 第二轮正推 Y4.3、Y4.6 | 合成时丢，Z4.3；要补回 |
| 判不出的两类坏实现作为「已认代价」 | 第二轮正推 Y4.5、攻方 Y4.3 | 没进交用户表，Z4.4 |
| 折中并法要 checker 走被抛弃根 | 第二轮正推 Y4.4 | 形态已写「不并」，工作量没写；现成件 `crates/singlefs-checker/src/walk.rs:9795`，Z4.2 |
| 字段数「七改九」、`22-…md:341` 的区间改 466 | 第一轮正推 X4.3（写成七改八） | Z4.5，实现批要改 |
| 同源的「设备失而复得 ⇒ 同 txg 两根」句：D16:184、C88、C85、里程碑九二·3 | 第二轮正推 Y1.4 只点了 D22:140 | Z1.2；C88 的「时间线判别」另一半没核 |
| 清扫发布在同一实例里重写实例表 | 无（本轮新冒出） | 零轮，Z2 |
| GPU 判官只按恢复那一路判：并集触发、公式核、F2 一行、I-7.18 / I-7.19 都不进 GPU 时的射程句 | 第一轮正推 X3.6、第二轮正推 Y3.3 | 没进格。无读故障的崩溃态上并集等于走读，GPU 不搬与 CPU 一致（别的会话量过 0，见上表末行）；将来注读故障才要搬，射程句要写进对拍用例 |
| E165 读槽对拍（停机条款）加公式核后重跑 | 第一轮正推 X1.7、「要主 agent 跑的命令」 | 重型，欠；实现批提交时跑 |
| `BadRootRingSlotReading` 复用 `NotSelfVerified` 还是加第三成员 | 第一轮正推 X1.7 | 实现取舍，没进格；先复用 |
| checker 一侧在 I-7.17 红的镜像上，其余不变量不作保 | 第一轮正推 X5 甲 / 乙；第二轮 Y5.2 | 第二轮写「checker 一侧不变」但没写进 I-7.17 那一行的那句话；实现批补 |
| 「并集」里记录（末条记录读不出）算不算 | 第一轮攻方 X3.3 | 正文写「单元的并集」；第二轮正推的模型含记录那 4 格，0 少挂上，但条文没说记录算不算 |
| D19 已定项 5 的回看：没见证的 R 上数据单元读不出，可写挂载原来「照常」挂 R，现在退回上一版 | 第二轮 Y3.4 末项只算了已确认格 | 第一轮判决回看表写「不受影响」；并集触发后这一句在没见证的格上要改成「没见证时除外」。该重判 |
| 表读不出、见证到的那条根读不出而区间里有同实例的根：今天不退，收窄后退到同实例的根 | 第二轮正推 Y2.4 末项 | 新结局，用例要钉；没进格 |

### Z5.3 结论

数能核（上表前六行在入库代码或入库输出文件上对上；Y1 / Y3 此前只有作者自己副本里的数，这是它们第一次被别人复跑）。清单不完整：上表 Z5.2 的 15 行里，真正会改结局的是前四行与清扫发布一行；其余是措辞或实现件。

## 这一轮新冒出的形态（零轮）

- 清扫发布在同一实例里重写实例表，使收窄形态的 F2 在 C751 自己的故障预算里让恢复由成变败（Z2，量过，副本）。
- 手摆根旧用例是 4 个目标 7 条用例，不是 3 个；checker 档另有 6 个文件没核（Z1.1，量过）。
- 同源的「设备失而复得」句不止 D22:140 一处，C88 / C85 / D16:184 / 里程碑九二·3 同改（Z1.2，读码）。
- I-7.18、I-7.19 无文件版本上的复制根豁免（Z4.3，读码）；字段数七改九（Z4.5）。
- 「退不成留在所选版」没带，双故障下并集触发会让只读由成变败（Z1.3，推的）。

## 要主 agent 跑的命令

没有必须的。判决要引复跑数时：

```
cd /home/fy5090/code/singlefs && bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-rootring-r3-sonnet-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-rootring-r3-sonnet/frozen --work <新的空草稿目录>
```

看 `name=rerun_compare` 三行都是 `same=yes`；线程 31；4G 够用（我这边排队等了半分钟到一分钟）。若 `--frozen` 的目录被清掉，重建法：把 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default` 拷成一份，`.rs` 的 sha256 清单再取 sha256 要是 `b523d85891b7327b57d1a760417241421b4749bd7891d00f019189826e375eb2`。

## 进程与负载

开跑前 `ps` 看到别的会话的 `cargo`（编译、带覆盖率的测试）与多条 4G 内存包装在跑，没有 `qemu-system`、`fio`、`vm-bench.sh`、`e152-file-system-benchmark`；全部 `nice -n 19`，经 `capped.sh 31`。内存包装每次排队 16 秒到 64 秒。没起后台任务。

## 草稿目录

`/home/fy5090/.local/share/singlefs/work/m3-close-rootring-r3-sonnet/`（约 46M）：
- `frozen/`：开工时 `crates/` 等的拷贝（35M），**留着**，是上面复跑命令的 `--frozen`；主 agent 用完可删。
- 两个编译用的工作树（`rerun-r2-sonnet/tree`、`rerun-r3b/tree`，各约 2–9G）已在交回前删掉，只留日志（`rerun-r2-sonnet.log`、`rerun-r3.log`、`rerun-r3b.log`）和 `logs/`（手摆根目标的逐个输出、两份模型的 `--nocapture` 输出）。
- `targets.txt`、`files22.txt`、`run-targets.sh`、`run-one-nocapture.sh`：开公式核逐目标跑的名单与脚本。第二份脚本的范围声明「为什么」栏写的是清扫那一形，也被我拿去跑了 26 格的那一份；两份都是同一类 harness 测试目标，没有别的范围。

## 没做什么

- 没判主 agent 的判决要不要撤销或维持，只给复核结果；没替用户选 C751 的修法、没选 C773 取舍。
- 没跑 checker 档、崩溃点重放、GPU 判官、E165、变异、门禁；`crates/singlefs-checker-tier` 里 6 个手摆根文件没核。
- 没实现 C773 的字段与 I-7.18 / I-7.19，没在 checker 上造合成形态的镜像，所以 Z4 全是推的；C773 登记形的复现盘面目录不在，核不动。
- 没造瞬时读故障的盘面（Z3）、没造双故障的「退到的那一版也读不通」盘面（Z1.3）。
- 没改 kb、`crates/`，没打系统配置落地补丁；没跑整个 harness 测试包，只跑了点名的 22 个目标加 2 份自己的模型目标。
- 副本上的数不是入库装置上的数；模型目录是新写的，要引进 kb 的数请主 agent 照三方规则在入库装置上重做。
- 没读禁读清单里的文件。

## cite-check 末行

`python3 scripts/agents/cite-check.py research/prompts/m3-close-rootring-r3-sonnet-output.md --root . --background research/prompts/_m3-close-rootring-r3-background.md` 末行原样（退 0）：

```
  ✓ 核了 38 处引文、20 处裸行号，对上 38 处，没判 20 处（其中裸行号没有引文可比 20 处；逐处列在上面）
```
