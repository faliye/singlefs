# 第 10z 批第十、十一块代码轮第一轮：正推腿报告（2026-10-08）

轮名 m3-batch10z-jk-code-r1，立场正推：从已定条款与 `crates/` 今天的实现推「应该是什么」，再与被判对象逐格比。
引代码写冻结副本 `~/.local/share/singlefs/work/m3-batch10z-jk-code-r1/tree/crates/` 下的路径与行号；引 kb 写 kb 文件名加那份文件自己的行号（kb 的行很长，只抄行内的一句，整行见原文件）。
模型与日志：`research/prompts/m3-batch10z-jk-code-r1-sonnet-model/`（下面简写 `model/`；README 写了每一份是什么）。草稿目录 `~/.local/share/singlefs/work/m3-batch10z-jk-code-r1-sonnet/`。

## 0 环境与限度

- 冻结副本核过：`sha256sum -c research/prompts/m3-batch10z-jk-code-r1-snapshot/crates-sha256.txt`（在冻结副本根下跑）336 行 `OK`、0 行不是 `OK`；草稿里的拷贝 `tree-a` 对冻结副本 `diff -rq crates` 无输出。
- 开跑前 `ps` 看到的：别的会话的 `cargo test --offline -p singlefs-harness --test opus_r3_leaf_death_images` 与两件 `e178-leaf-and-record-death-counts` release 运行，没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`；照常 `nice -n 19`，每条命令 `capped.sh` 1 至 15 线程、并行不超过 8 件；机器 loadavg 18 至 36，没有等过锁。
- 没起 GPU 设备，没跑 checker 档测试，没跑门禁阶段。第十一块 checker 档的 4 行变异与 GPU 相关的格，只做了「逐字搬运进小程序」的模型与读码（下面标「模型」「读码」）。
- 谱系、回退、根环的条款：「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」（`.claude/kb/decisions/23-journal的角色与格式.md:390`）；「回退候选集里每一个根」（`.claude/kb/invariants.md:58`）「所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头」（`.claude/kb/invariants.md:58`）（I-7.4）；「管理员回退那次发布追加一条」（`.claude/kb/decisions/18-块里携带什么信息.md:576`）（谱系记录 (T_rb, T_old)）；「新根的用户可见树（inode 树、extent 树）与 `deleted_inodes` 的根指针取 R_old 那一版的」（`.claude/kb/decisions/23-journal的角色与格式.md:378`）。

## 1 我推出来的「应该是什么」（第十块）

从上面两条（D23 第 390 行、I-7.4）推：只要一条被抛弃时间线的根 R 还在根环里，R 引用的每个单元就得一直隔离到这个进程写掉 R 那一槽。
R 走丙（记录齐、账读不出，「txgs_of_the_roots_taking_the_current_account.push(root.checkpoint_txg);」（`crates/singlefs-core/src/mount.rs:2015`））时，R 引用的单元只可能来自四处：
(a) P 的账里仍分配的（「placements_still_allocated_in_the_account(current_records);」（`crates/singlefs-core/src/mount.rs:2017`） 已并）；(b) (P, R] 之间各次发布的记录点名的、账里列的（要靠丙并中间并进 R 那一槽）；
(c) R 自己写的（记录点名，C 里本来就有）；(d) 若 R 或 (P, R) 之间某次发布 Q 是回退：回退目标 R_old 引用、P 的账里记成已释放的数据单元（复活）。
入库代码对 (b) 的做法：「fn with_the_placements_of_the_publishes_in_between_chained_onto_the_newer_roots(」（`crates/singlefs-core/src/mount.rs:2064`），上界 「..newer_root_txg.0)」（`crates/singlefs-core/src/mount.rs:2076`），至多一圈 「.take(ring_length_in_txgs)」（`crates/singlefs-core/src/mount.rs:2078`）；
对 (d) 只管 R 自己是回退：「fn placements_a_rollback_publish_revived_released_in_the_current_account<Device: BlockDevice>(」（`crates/singlefs-core/src/mount.rs:2118`），认回退 「.find(|lineage| lineage.rollback_txg == checkpoint_txg_of_the_publish)」（`crates/singlefs-core/src/mount.rs:2169`），
取 R_old 用 「.filter(|root| !is_abandoned(root) && root.checkpoint_txg == rollback_target_txg)」（`crates/singlefs-core/src/mount.rs:2141`），交 「.filter(|(device, slot, _)| released_in_the_current_account.contains(&(*device, *slot)))」（`crates/singlefs-core/src/mount.rs:2146`） 那一截。
放的时机在分配器：「pub fn record_root_written_by_this_process(&mut self, checkpoint_txg: CheckpointTxg) {」（`crates/singlefs-core/src/allocator.rs:2157`） 里 「.remove(&ring_slot)」（`crates/singlefs-core/src/allocator.rs:2184`） 取走被写那一槽，「let still_protected_by_the_isolation = occupancy.placements_the_isolation_still_protects();」（`crates/singlefs-core/src/allocator.rs:2186`） 对剩下的槽照罩——所以 Q 那一槽先解开时，R 那一槽里并进来的那一份仍罩着。

## 2 K1　洞一 丙并中间

### K1-1 罩不罩得住　判：一致（在我造得出的形里），附两个造不出的形

做法：拷冻结副本到草稿 `tree-p`，把第七八九块实现员的量法探针 `probe_10zj.rs`（`~/.local/share/singlefs/work/m3-r1/batch10z-j/keep/`）改成可配 Q、R 的种类、R 之后再发布、先做几次发布，加故障 `qacct`、`rlin`、`rold`、`roldacct`、`sacct`（`model/src/probe_jk.rs`），release 构建，每件经 `run-with-memory-cap.sh 4G` + `capped.sh 1`。
- 入库用例原样复跑：`cargo test --offline -p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table` 末行 `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.80s`。
- 攻方一形的扩展：Q ∈ {顺序写 s、建 inode n、回退 b} × R ∈ {n、b} × 故障子集（`cfgq` 恒有、`racct` 恒有，另取 `qroot`、`qacct`、`qlast`、`rlin` 的所有子集，R 不是回退时去掉 `rlin`），共 72 个形、每形 2 段随机历史（22 至 26 步，字母表 `ons`，每一步核根落之前那一态与之后的 I-7.4、I-4.8）。
  汇总 `logs/summary-sweep-implemented.txt`：72 形里 39 形挂载被拒（`Recovery(UnitUnreadable{…})`，没写，不算红）、33 形做成；做成的里 7 形红（14 段），7 形全都带 `rlin`（R 是回退而它的谱系片读不出，见 K2-3）；不带 `rlin` 的 26 个做成的形 0 红。
  另一批（字母表 `onsFb`，含抬 F 与会话里回退），臂 `none`（撤掉两处补法）对攻方原形（Q=s、R=n、`cfgq,qroot,qlast,racct`）12 段里 2 段红（`logs/summary-F-and-b-actions.txt`：`fb-chain arm=none … hit=2`；`arm=implemented … hit=0`）。
- 我自己提的一形 H-A：Q 是回退、R 是普通发布、Q 的根槽读不出、Q 的账读不出、R 的账读不出（Q 的记录齐）。丙并中间只并 C 里 Q 那一槽，Q 复活的单元不在 Q 的记录里，也不在 Q 的账里（读不出），R 不是回退所以洞二也不认。
  造出来：`logs/summary-qb-shapes.txt` 里 `qb-cfgq+qroot+qacct+racct` 4 段全部挂载被拒（`Recovery(UnitUnreadable{slot:SlotNumber(1532)})`），没有红。再加 `qlast`（Q 的末条记录读不出、Q 的记录不齐）：`qb-cfgq+qroot+qacct+qlast+racct` 4 段 0 红（Q 走从映射现算）。
  所以这一形今天是拒不是红；推翻它的现象：恢复阶段对「记录齐、根读不出、账读不出」的那条 `UnitUnreadable` 拒绝被放宽（比如改成交空），H-A 就走到丙，复活的单元进不了 R 那一槽。
- R 之后还有较新的根 S 也走丙（`post=o`、`sacct`）：造了两次，挂载被拒（`Recovery(UnitUnreadable{slot:SlotNumber(1542)})`、`Recovery(NoValidSystemConfiguration…)`，`logs/summary-post-attempts.txt` 全是 `mount_err`），没造出 S 与 R 同走丙而做成的形。没做成不等于做不成。
- 推翻 K1-1 的现象：在入库代码上做成挂载、R 走丙、而 Q 的复活或写出的单元不在 R 那一槽里，且 R 还在环里时 I-7.4 红。

### K1-2 放的时机　判：一致

- 一直不放：臂 `implemented` 做成的 405 段历史（`model/runs/*.log` 里 `arm=implemented` 的行）：`refused_steps` 全 0、`r_left_at` 每段都有值（R 离开了根环）；`iso_end`（段末隔离的槽数）为 0 的 371 段；其余 34 段全是 Q 的根槽与 Q 的末条记录都读不出（故障里 `qroot` 与 `qlast` 同时有）的形，隔离数停在 18 或 30 不回 0——同形在撤掉两处补法的臂 `none` 里一样，见下面「旁证」，不是这一补丁带进来的。
- 放早：同样每一步核「最后一条根槽写之前」那一态（`PROBE_PREROOT`），臂 `implemented` 0 红；撤掉补法（臂 `none`）攻方形 2/12 红、回退形 22/24 红，红的都在 R 还在环里的那一态——说明探针看得见放早，臂 `implemented` 没有。
- 机制：分配器 `allocator.rs`:2184 只取走被写那一槽，其余槽继续罩（见 §1）；R 那一槽在这个进程写到 R 的 txg 加一整圈时才解开（384 槽盘上 24 个 txg）。
- 推翻它的现象：某一步 `iso_end` 不回 0，或根落之前那一态红而 R 在环里。

### 旁证（不归这两块，撤掉两处补法的臂 `none` 里同形）

故障 `cfgq,qroot,qlast,racct`、Q=n、R=n、60 步 `o`、臂 `none`：段末 `iso_end=18 c_left=0 abn_left=0`（`model/runs/long-nn60-none.log`），隔离了 18 槽而根环表里已经没有谁要它们；同一个形去掉 `qroot,qlast` 只留 `cfgq,racct`，段末 `iso_end=0`（`long-nn60b-none.log`）。
120 步也是 18（`long-nn120-none.log`）。这一形没有回退、没有丙并中间的补法参与，我没追原因（不在这两块里）。它是「一直不放」的一个实例，交主 agent 判要不要另立一题。
推翻它的现象：把这一形放在入库的 `mount.rs` 原件（不加我的环境变量臂）上跑，段末 `iso_end` 回 0。

## 3 K2　洞二 回退复活

### K2-1 认回退　判：冲突一处（认漏：读不出与没有记录同一个结局）；认错没造出来

两句并排整行：
- 「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」（`.claude/kb/decisions/23-journal的角色与格式.md:390`）
- 「/// 认不出回退（记录没点名谱系片、那一片读不出、里面没有这条根的谱系记录）、根环里没读出 R_old、R_old 的账读不出，都交空：」（`crates/singlefs-core/src/mount.rs:2112`），接着 「/// 那几形照丙原样，少护的那一截交主 agent」（`crates/singlefs-core/src/mount.rs:2113`）（该行后半句：第十块报告「停下交主 agent」。只读盘，不改分配器。）

认漏的机理（读码）：谱系片读不出 「read_unit_via_locations(devices, &named.locations, packed_unit_bytes).ok()」（`crates/singlefs-core/src/mount.rs:2163`）、校验和对不上或解不开 「.filter_map(|bytes| crate::unit::parse_packed_unit(&bytes).ok())」（`crates/singlefs-core/src/mount.rs:2165`），两条都被 `.ok()` 吞成「这一片不存在」，
与「读得出、确实没有这条谱系记录」（R 不是回退）落在同一个 `None`。调用方交空。

认错（并进不相干的落点）没造出来：
- R 不是回退却点名了更早一次回退的谱系片：Q=回退、R ∈ {n, o, s}、故障 `cfgq,qlast,racct`，加臂 `anylin`（认回退时不比 `rollback_txg`，任何谱系记录都认）对臂 `implemented`：三个 R 的 `iso_mount` 逐格相同（18、24、34），`c_r` 相同，0 红（`logs/summary-anylineage.txt`，各 2 段）。读码：「.find(|lineage| lineage.rollback_txg == checkpoint_txg_of_the_publish)」（`crates/singlefs-core/src/mount.rs:2169`） 要等于这条根自己的 txg；同一个环里 txg 不重复，普通发布的记录不点名谱系片。
- 谱系片里不止一条谱系记录：`pre` 取 `bo`、`boo`、`bob`、`obo`、`oobo`（R_old 自己是回退、或中间隔着几次覆写）的 R=回退，`logs/summary-rollback-sweep-implemented.txt`：144 个形里 128 个做成（16 个没录成：回退那一步被拒或录下来的根数对不上，原因没逐个看）、0 红。
- 回退目标不在一圈窗口里：入库用例 `rollback_target_left_in_a_hole_outside_the_one_revolution_window_is_a_candidate.rs` 证明环里一圈窗口之外的根是合法候选；这时 R_old 读得出就认得对，读不出是下面 K2-3 的 `rold`。
- 推翻「认错没造出来」的现象：某个 R 不是回退、`implemented` 与 `anylin` 的 `c_r` 或 `iso_mount` 不等。

### K2-2 罩不罩得住　判：一致

- 回退扫：`k` 取 2 至 5、回退目标取倒数第 1 至 6 版、先做 `pre` 六种（空、`bo`、`boo`、`bob`、`obo`、`oobo`），故障 `racct`，每形 2 段、22 至 28 步：128 个做成的形、256 段，0 红；120 个形里复活的单元非空（`revived` > 0），全部在 R 那一槽里（`logs/summary-rollback-sweep-implemented.txt` 每行的 `rev_in_c_r` 与 `revived` 两列逐形相等）。
- 带 `F`（抬 F 到准入上限）与 `b`（会话里再回退）的字母表 `onsFb`：`logs/summary-F-and-b-actions.txt` 的 `fb-rb arm=implemented n=24 … hit=0`；臂 `none` 22/24 红。
- R_old 本身是回退：`pre=bo` 起的各形包括在内（R_old = 那次回退发布，它的账把复活的单元记成已分配，所以在「R_old 账里仍分配」那一截里），0 红。
- 推翻它的现象：某个做成的形里复活的单元不在 R 那一槽、且 R 在环里时 I-7.4 红。

### K2-3 认不出回退（交用户的残余）　判：冲突（同 K2-1 的两句）；可达面与读得出的信息如下

我造出的三形（回退目标 = 第一个文件那一版，k=4，R=回退，故障 `racct` 加下面一项，每形 4 段、22 至 34 步，`logs/summary-residual-shapes.txt`）：

| 形 | 额外故障 | 臂 `implemented` | 臂 `today`（撤两处补法） | 臂 `control`（并现行账里全部已释放的） | 臂 `tri`（见下） |
|---|---|---|---|---|---|
| 谱系片读不出 | `rlin`：R 写的谱系打包单元读回全零 | 4/4 红（根落之前那一态，R 在环里） | 4/4 红 | 0 红，挂载 116 槽 | 0 红，挂载 116 槽 |
| R_old 的根槽读不出 | `rold` | 4/4 红 | 4/4 红 | 0 红，116 | 0 红，116 |
| R_old 的账读不出 | `roldacct`：R_old 的树表与分配记录树根读回全零 | 4/4 红 | 4/4 红 | 0 红，116 | 0 红，116 |

红法与攻方形同一个：R 复活的数据单元（2 个落点）没进 R 那一槽，会话覆盖写到抬 F 或回收时发出去，池级 checker 在被抛弃的 R 上报 I-7.4。三形里 `implemented` 与 `today` 逐格同数，说明补法在这三形上一点不起作用（它们都落在「交空」上）。

可达面（读码加实测）：
- 三形都要两件事同时成立：R 是回退发布、走丙（R 的账读不出而记录齐）；加上「谱系打包单元」「R_old 的根槽」「R_old 的树表或分配记录树根」三者之一读不出。其余回退组合都被认出（K2-2）。
- 回退目标取哪一版不挑：认回退与找 R_old 的两步不看目标是哪一版；我只在 k=4 的第一个文件那一版上跑了这三形，72 形扫（`logs/summary-sweep-implemented.txt`）里 `rlin` 在 Q=s、n 之后的回退（目标回到第一个文件那一版）上同样红：7 个形、14 段。`rold`、`roldacct` 没做 Q 之后的形。
- `rold` 是入库用例承认的合法情形：回退候选取整个根环（RGall），目标可以是一圈之前的根（上面那份用例）；挂载时那一槽读不出的目标一样走到交空。
- `roldacct` 时回退那一刻 R_old 的账是读得出的（「// R_old 那一版整个从盘上重建（它的账、它的三棵用户可见树、记账树、中央映射树都读回来）：有一个节点读不出就拒。」（`crates/singlefs-core/src/mount.rs:7803`）），所以是回退之后才坏的读。
- 没造出来的：R_old 被 R 之后的发布覆写出根环（要 R 之后有同样走丙的 S，且 384 槽盘上 F 抬着，目标最远只能退几版；我试 k=22、目标取倒数第 6、9、12、15、18、20、21、22 版，都被 `BelowEffectiveFloor` 或 `NotInRing` 拒在回退那一步，没录成；`model/runs/old-*.log` 有拒绝的原话）。

不改格式、读得出的信息（回答「有没有」）：有。
1. R 的记录齐（丙的入口条件就是齐），每条记录的 `named` 列出 R 写的单元和类标签：「.filter(|named| named.unit_class == crate::unit::UNIT_CLASS_PACKED)」（`crates/singlefs-core/src/mount.rs:2161`）。回退发布必然追加一条谱系记录（D18 「管理员回退那次发布追加一条」（`.claude/kb/decisions/18-块里携带什么信息.md:576`）），所以 R 是回退 ⇒ R 的记录点名了一个谱系打包单元。反过来：R 点名的打包单元全读得出、且里面没有谱系记录 ⇒ R 肯定不是回退；点名的打包单元有读不出的 ⇒ 不知道。代码今天把「不知道」当「肯定不是」。
2. 认出是回退（谱系记录读得到）而根环里没有 R_old、或 R_old 的账读不出 ⇒ 肯定是回退、复活集不知道。
3. 臂 `tri`（`model/src/probe-arms-on-mount.diff`）：上面 1、2 的「不知道」与「肯定是回退而算不出」都改并现行账里全部已释放的落点（即对照臂），其余不变。结果：三形 12 段 0 红；72 形扫里与 `implemented` 的 `iso_mount` 不同的只有 11 个形，全带 `rlin`（`logs/summary-sweep-tri.txt`），做成的 33 个形 0 红；做成的 33 形里另外 22 形（R 不是回退）的 `iso_mount` 与 `implemented` 逐格相同，一槽不多。
   代价：那些形里挂载那一刻隔离 92 至 126 槽、推过第一串抬 F 144 槽（384 槽盘），拒 0 段。只量了 384 槽那一档盘。
4. 回退目标账里的复活集不用读：R 的树表里 inode、extent 树根指针取 R_old 的（「新根的用户可见树（inode 树、extent 树）与 `deleted_inodes` 的根指针取 R_old 那一版的」（`.claude/kb/decisions/23-journal的角色与格式.md:378`）），但树表是 R 写的单元，在 `racct` 与 `rlin` 里一起读不出，用不上。
- 推翻「有读得出的信息」的现象：`tri` 的判据（点名的打包单元读不出）在某个 R 不是回退、R 的打包单元全读得出的形上多隔离了；或某个 R 是回退而点名的谱系片读不出时，`named` 里没有类标签 3 的单元。

### K2-4 多护的那一截放不放、D28 第九项　判：一致（放与拒）；规则没说一处（第九项字面）

- 量：R 是回退时挂载那一刻隔离 42 槽（臂 `today` 18，臂 `control` 116），推过第一串抬 F 70 槽（`today` 46，`control` 144），384 槽盘；多出的 24 槽是 R_old 自己的固定点单元加复活的 2 个落点（跨度 2）。`logs/summary-rollback-sweep-implemented.txt` 的 128 形 `iso_mount` 取值：42（232 段）、36、30、48（各 8 段，是 pre 里带多次回退的形）。
- 放：回退扫 256 段全部 `iso_end=0`、`r_left_at` 有值；`refused_steps` 在臂 `implemented` 的 405 段里全 0，没有一段因为这 24 槽被拒。
- 准入与卸载：我只复跑了 `raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table` 13 个全过（含抬 F 与卸载几条），`admission_formula_on_the_mounted_state` 等第九项相关的二进制没复跑（实现员报告第五节写过 15 个全绿，我没复核）。
- 第九项字面：「可写挂载时预隔离的槽：水位之上没施加的 journal 记录点名的落点、那次发布的账里的落点与根环里比所选那一版新、按实例表有效、读得出的根的账里的落点」（`.claude/kb/decisions/28-挂载期承诺量.md:27`）；
  代码并进来的是 「.filter(|(device, slot, _)| released_in_the_current_account.contains(&(*device, *slot)))」（`crates/singlefs-core/src/mount.rs:2146`） 那一截——R_old（比所选那一版旧）账里的落点，字面没说到。结论：规则没说，是写回项（实现员报告也这么记）。
- 推翻它的现象：某段 `iso_end` 不回 0 且 `c_left`、`abn_left` 非 0（即补法并进去的没解开），或 `refused_steps` 非 0 而 `today` 臂同一段为 0。

## 4 K3　第十一块 K1-1 列卡带借据号

### K3-1　判：一致（造不出多给）；前提无会红的检查，一致于前一轮

读码推（账层单测 108 个全过，`gpu_memory_ledger::tests`）：
- 销账只有四条路：列卡后 poll 交回队列空了只销列出来的 「fn return_the_listed_leases_kept_after_a_failed_wait_on(」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:904`）；某笔借据等成了销这一台上全部没等成的 「books_of_the_card.return_the_leases_kept_after_a_failed_wait_on(device.device);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1551`）；
  设备记成丢了 「for kept_lease in std::mem::take(&mut self.leases_kept_after_a_failed_wait).into_keys() {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:869`）；设备放掉。
- 列卡之后才记成没等成的一笔 A，在下一次列卡、poll 之前被销的路只剩第二条（别的借据 B 等成了）和后两条。第二条：A 没等成靠的是超时（「timeout: Some(WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE),」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:50`）），
  A 的 poll 在 t_a − T 开始、A 的活都在那之前提交；B 在 t_b > t_a 等成，等成要在自己的超时之内，B 的 poll 开始在 t_b − T > t_a − T，晚于 A 的全部提交，B 等到的「到此刻为止的全部提交」包括 A 的。
  非超时的没等成（「fn device_wait_before_returning_a_lease(」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:32`） 里 `Err(_)`、panic）可以立刻发生，那时 A 的活可以晚于 B 的 poll 开始；但一台设备上 poll 报错或 panic 的多半是设备丢了（推的），同一台设备上 B 等不成。没造出这一形。
- 后两条销的是整张卡的设备已丢或已放，缓冲随设备走，不多给。
- 「借据先于缓冲丢」：`create_buffer` 的位置都在函数内部（`crash_verify_gpu.rs`、`gpu_unit_checks.rs`:643 至 676、`crash_judge_dispatch.rs`:548、776 至 799），GPU 判器对拍那条路写成先丢判器再还借据（「drop(judge);」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:948`）、949 `drop(lease);`）。
  没有一条会红的检查证明它（与前一轮判决「前提今天没有会红的检查」一致）；我也没造出破坏它的产品路径。
- 推翻它的现象：账层测试里造出 A（列卡之后记成没等成）被 B 的等成销掉而 A 的缓冲还在卡上的序列；或产品路径里 wgpu 缓冲活过借据。

### K3-2　判：一致（多销没有）；永远销不掉有一形，归 K4-2

- 列出来的、poll 的、换下来的三台：列出来时记的是借据号→设备号，销时核 「if *listed_on_the_device == device」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:911`），设备号不同不销；重起时 `take_back_the_driver_share_of_a_lost_device`（`gpu_memory_ledger.rs`:864）已把整张卡的没等成的清掉，所以列出来的借据在设备换了之后 `remove(..).is_some()` 为假，不会多销。
- 变异 G1（去掉 `listed_on_the_device == device` 那个条件）在账层 108 个单测里照绿——生产里等价（设备换了借据早被清了），但没有测试钉着这个条件。
- 永远销不掉：见 K4-2。

## 5 K4　第十一块 K4-2 拆锁

### K4-1 死锁、两条线程各起一台　判：一致

- 锁序：`starting` → 一瞬 `started`（`gpu_memory_ledger.rs`:1719 起的 `replace`）→ 起设备闭包里的账锁（`prepare_the_driver_share_before_starting_the_device`，`gpu_memory_ledger.rs`:1318；`record_the_device_started_on_the_card`）。反方向（持账锁再要 `starting` 或 `started`）没有：`started_or_start` 的调用只有 「card.started.started_or_start(」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1371`），`started_now` 只有 `gpu_unit_checks.rs`:1338 那一处，两处都不在账锁里。
- 同一张卡两条线程各起一台：拿到 `starting` 之后再看一次（`gpu_memory_ledger.rs`:1712 起），入库用例 `a_second_start_on_the_same_card_waits_for_the_restart_in_progress_and_takes_its_device`（`gpu_memory_ledger.rs`:4224）。
  把那个再看一次的块删掉的变异（表行「拆锁之后拿到起设备的锁不再看一次留着的（等完照样另起一台）」，红在这条用例，`logs/mutation-harness-ledger-last-row.txt`）与整个去掉 `starting` 的变异（表行「拆锁之后同一张卡上同时要起的不等这一次起完（各起一台）」，`logs/mutation-harness-ledger-rows-and-extras.txt` 第 3 行）在账层都红；删掉 `starting` 的变异我又在 16 路并行、机器 loadavg 20 以上的条件下连跑 48 次：48 次全红（`logs/race-second-start-mutated-48x.txt` 里 `48 101`）。
- 毒：`starting` 与 `started` 都 `unwrap_or_else(PoisonError::into_inner)`（`gpu_memory_ledger.rs`:1710）。起设备闭包 panic 时 `started` 还是旧的，别的线程再起。
- 推翻它的现象：账层单测里两条线程先后拿到各自起的设备（`starts_by_the_second` 非 0），或两把锁互等挂住。

### K4-2 借之前碰上正在重起　判：账多给没有；少给出不来有一形（冲突，窄；读码加账层实测，真卡没试）

- 账多给：重起中 `started_now()`（`gpu_memory_ledger.rs`:1740）交起之前留着的那一份（丢了的），`take_in_the_devices_found_idle` 对丢了的不 poll（「if started.device_condition().is_lost() {」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1342`）），不销；没销就是少给，不是多给。重起开头 `take_back…` 把这张卡没等成的都清了（缓冲在丢了的设备上，放掉之前这个进程占着的照分不出算）。没造出多给。
- 少给出不来（自己造的账层探针 `model/src/probe_kept_leases_on_a_lost_device.rs.txt`，在草稿副本的 `mod tests` 里跑）：
  两句并排整行：
  - 「            /// （wgpu 在丢了的设备上 poll 会 panic）：那几笔到重起时照重起记销。取留着的设备不等别的线程起设备（`DeviceStartedOnDemand::started_now`，」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1332`）
  - 账层探针原样输出：`PROBE room_before=29550 kept=1 room_now_after_lost=1000`、`PROBE booking_4000_after_the_device_is_lost_and_the_buffers_are_gone=Some(NoCardHasRoomNow { need_mebibytes: 4000, card_rooms_mebibytes: [1000] })`、`PROBE prepare=Ok(()) room_after_prepare=29550`、`test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 195 filtered out; finished in 0.01s`。
  机理：丢了的设备上等设备没等成的借据记在账上（`take_in_the_wait_before_returning` 在账不知道设备丢了时照记），池里只有借到之后才走到重起（「fn started_card(」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1365`） 里的 `.prepare_the_driver_share_before_starting_the_device(card_position)` 在 `gpu_unit_checks.rs`:1379），重起才清它们。
  这些借据占着的量让后面的借都放不下时，借不到、走不到重起，那张卡这个进程里不再借出（报「本次未跑」，卡上其实空着）。设备丢了的时候这张卡上同时在飞的几笔借据都等不成是常态（推的），它们加起来超过卡容量减下一笔需要的，就走到这一形。
  这不是第十一块带进来的（拆锁前 `started_now` 同样交丢了的、同样跳过），但它正是 K4-2 问的「本该销的永远销不掉」。
- 推翻它的现象：池层有一条路在丢了的设备上也告诉账「设备丢了」（不等借到），或真卡上 `NoCardHasRoomNow` 之后下一次借照样能走到重起。

### K4-3 旧设备在放开 `started` 之后才丢　判：一致（读码），变异 G2 照绿

- 顺序：「let replaced = self」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1719`） 在 `started` 的锁里取出旧设备、锁放开，「drop(replaced);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1724`） 丢它——此时仍持 `starting`（`_starting` 到函数尾，`gpu_memory_ledger.rs`:1710）。所以丢旧设备的几秒里：拿新设备的快路（只拿 `started`）不受影响；要重起同一张卡的线程要等；做重起的这条线程自己晚返回。
- wgpu 这一段是转述，不是引文：`wgpu-core-30.0.1/src/device/queue.rs`（`~/.cargo/registry/src/*/`）的 `impl Drop for Queue` 先对自己的 raw 队列 `wait_for_idle`，再取自己设备的 `snatchable_lock` 读锁做 `maintain`，设备丢了走 `handle_hal_error(Lost)` 不 panic；等和锁都落在被丢的那台设备上，不碰别的设备。真设备上等多久没量（不起 GPU）。
- 变异 G2（把 `replace` 改回在 `started` 锁里丢）账层 108 个单测照绿：顺序没有测试钉着。
- 推翻它的现象：真设备上丢旧设备时，别的卡或这张卡新设备上的提交被压住。

## 6 K5　测试判别力

### 第十块　判：10 行都红（实现员只证了 4 行）；改回去的方向都拦住；「多护」方向的四处改动照绿

- 10 行变异表行我全跑了（拷贝 `tree-m1` 至 `tree-m4`，各自的编译目录，`model/scripts/mutdrv.py`，原文在表里命中恰好一次），`logs/mutation-core-rows-and-extras.txt` 第 0 至 9 行：每一行都 `rc=101` 红，红在表里点名的那条用例上
  （丙并中间的行 1、3 是 `units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written`；行 2、4、5、10 是库单测 `the_ring_slot_of_a_root_taking_the_current_account_takes_what_the_publishes_in_between_named` 与 `a_root_taking_the_current_account_right_after_the_selected_version_takes_nothing_more`；回退复活四行 6 至 9 是 `units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount`）。
  所以「其余 6 行读码判」的那 6 行（3、4、5、7、8、9）实测都红。
- 我另加的改动（`logs/mutation-core-rows-and-extras.txt` 第 10 至 16 行，`scripts/extra-core.py`）：

| 改动 | 结果 | 说明 |
|---|---|---|
| E1 回退目标取所有有效根（去掉 `txg == T_old`，`mount.rs`:2141） | **绿**（`1 passed`） | 多护，用例不查多出来的 |
| E2 认回退时任何谱系记录都认（`mount.rs`:2169 改成 `.find(|_lineage| true)`） | **绿** | 认错方向没有用例 |
| E3 认出回退就并现行账里全部已释放的落点（正是实现员量过、因代价而不取的对照 `control`） | **绿** | 「窄」没有测试钉着 |
| E7 只并最后一条走丙的根（`.iter().rev().take(1)`） | **绿** | 用例里只有一条走丙的根 |
| E4 去掉 `!= R 那一槽` 的过滤、E5 取最旧一圈而不是最近一圈 | 绿 | 等价改动（去重、一圈内每槽都取到），不算漏 |
| E6 上界放到 R 之后一圈 | 红（库单测） | |

  E3 是最要紧的一条：用户定「回退那一形的补法代价先量，隔离数或拒数与乙同一量级时停下」，实现员取窄形的理由是隔离 42 对 116；把窄形改回对照臂（116），点名的那条回退用例照绿；文件里与回退有关的只有这一条（我读过，别的 12 条我没逐条在这一改动上跑），没有一条钉着「R 不是回退时一槽不多」「R 是回退时只多这一截」。
- 推翻「E1/E2/E3/E7 照绿」的现象：在入库的测试里有一条在这四处改动之一上红（我跑的是 `raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table` 里被点名的那一条与库单测那一组，没跑整份文件，整份文件有 13 条；基线整份 13 条全过）。

### 第十一块　判：账层 10 行都红；checker 档 4 行在逐字搬运的小程序里红；`take_in_the_devices_found_idle` 里面和下游各层的叫停没有用例

- 账层：实现员表里 `-p singlefs-harness` 的 9 行加补跑的 1 行（表末的「拿到起设备的锁不再看一次留着的」）全红（`logs/mutation-harness-ledger-rows-and-extras.txt` 第 0 至 8 行、`logs/mutation-harness-ledger-last-row.txt`）。
  `a_second_start_on_the_same_card_waits_for_the_restart_in_progress_and_takes_its_device` 的 500 毫秒：该用例 「second.recv_timeout(std::time::Duration::from_millis(500));」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:4260`） 之后还有两处断言（第二条拿到的是第一条起好的那台、第二条没另起），
  去掉 `starting` 的变异在第二条线程晚排上时仍会在后两处红；只有第二条线程晚到 500 毫秒以上、且第一条已经把新设备放进去时才会假绿。16 路并行加机器 loadavg 20 以上连跑 48 次全红（`logs/race-second-start-mutated-48x.txt`：`48 101`）。
- 我另加的账层改动：G1（去掉 `gpu_memory_ledger.rs`:911 的设备号条件）绿、G2（旧设备在 `started` 锁里丢）绿、G4（去掉 `take_in_that_the_work_of_the_listed_leases_finished` 开头的卡位置断言）绿；G3（起着时 `started_now` 交 `None`）红、G5（列卡不带借据）红。三处绿的都是读码上等价或顺序类的，没有测试钉着。
- checker 档 4 行（X4、X4b、X1、X2）：我不跑 checker 档测试，把两条扫源码用例的函数体逐字搬进小程序（`model/scripts/scan-model/`，只把 `include_str!` 换成读文件、断言换成交回 `RED …`），对冻结副本的源文件套表里的 `old`→`new`：
  基线 `GREEN`/`GREEN`，X4 `RED fn lease_card( …`，X4b `RED pub fn card_rooms_now_and_when_empty_of( …`，X1 `RED … 最后一个参数是 Some("&|| false,")`，X2 同形（`logs/scan-model-checker-tier-rows.txt`）。这是模型上的红，不是真用例跑出来的。
- 照绿的（模型）：「if device_found_idle_by_a_poll(&polled) {」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1348`） 改成 `if true || …`、`if false && …`（Y1、Y2）；`gpu_unit_checks.rs`:1336 起那个循环只看第一张卡（Y3）；「if started.device_condition().is_lost() {」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1342`） 改成 `if false {`（Y4）——四处扫源码用例都不看函数体里面。
  `take_in_the_devices_found_idle` 在整个仓里只有 `gpu_unit_checks.rs` 自己出现（`grep -rln` 只返回 `gpu_unit_checks.rs` 与 `gpu_memory_ledger.rs`），要真卡的用例也没有造「没等成再列卡 poll」的序列；这一段只有读码。
- 两句并排整行（下游各层的叫停）：
  - 「    /// （两处调用的最后一个参数整行比）。下游各层把收到的叫停原样往下交，换掉就是那一层的参数没用上、编译器报没用的变量。扫这份源码，不要卡。」（`crates/singlefs-checker-tier/src/crash_amplification.rs:4788`）
  - 实测：把 「        stop_is_requested,」（`crates/singlefs-checker-tier/src/crash_amplification.rs:6345`）（`judge_my_blocks_on_gpu` 的 gpu 特性那一份，`crash_amplification.rs`:6327 起）改成 `&|| false,`，`cargo check --offline -p singlefs-checker-tier --features verdict-store,gpu --lib`（`BINDGEN_EXTRA_CLANG_ARGS` 指到 gcc 的 include 目录绕过本机 clang 找不到 `stdbool.h`）原样：
    `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 2m 13s`、`warning: unused variable: \`stop_is_requested\``、`warning: \`singlefs-checker-tier\` (lib) generated 1 warning`——是警告，编译过。
    变异跑道不加 `-D warnings`：`grep -n 'RUSTFLAGS\|-D warnings\|unused' scripts/mutate.sh .claude/gate.d/checker-tier-crates-mutation-replay.sh scripts/replay.sh` 无输出。X1、X2 钉的是外层调用 `crash_amplification.rs`:5775、5810 两处；
    下游各层（`crash_amplification.rs`:1420、2672、5365、6345 的 `stop_is_requested,`）我没找到另有用例钉。
- 推翻它们的现象：入库的 checker 档里有一条用例在 Y1 至 Y4 或下游各层 `&|| false` 上红（我跑不了那一档）。

## 7 K6　只防 OOM、不压并发（第十一块）

### K6-1 账给的量大于此刻真能用的　判：造不出（账层）；Poll 之后缓冲交回驱动这一前提复核不了

- 列卡带借据号只让销得更少（只销列出来的），不会比改前多给；`started_now` 不等重起，交的是丢了的那台，不 poll，也不会多给。账层的 108 个单测与表里 10 行都红（K5）说明这条线上的改动都有测试。
- 复核不了的：销账靠的是「poll 交回 `QueueEmpty` ⇒ 丢掉的缓冲已交回驱动」（`gpu_unit_checks.rs`:54 起的注释，文件里写 wgpu-core 30 `Device::maintain`），文件头写的 E174 量的是等设备（Wait）那一臂，我没看到 Poll 那一臂在真卡上的量（我不起 GPU，也没复核）。要主 agent 在真卡上量：借、建满、没等成、丢掉、poll 到 `QueueEmpty` 之后 `nvidia-smi` 读数里那笔还在不在。
- 推翻它的现象：真卡上 poll 到 `QueueEmpty` 之后缓冲仍占着显存，账已销。

### K6-2 卡放得下却报本次未跑、整趟弃卡、并发被压　判：有一形（同 K4-2），其余没有

- K4-2 那一形：设备丢了、同卡几笔借据没等成，记着的量超过「容量减下一笔要的」，之后每次借这张卡都 `NoCardHasRoomNow`，打「本次未跑」，卡上其实空着；每张卡都这样就整趟弃卡。账层探针实测见 K4-2（挂载 1000、借 4000 被拒）。不在用户已认的清单里（已认的是两进程同读一份空闲的竞态、G2 乙、H2 丙）。
- 只销列出来的带来的瞬时少给：A 在列卡之后才记成没等成，下一次借的列卡会列到它；两次借之间 A 的缓冲其实已交回驱动时，这一窗口里的借按账少给，可能让放得下的借被拒。窗口 = 一次列卡到记账之间，A 是等设备超时才记的（每笔至少等满一个超时），我没量到被拒的实例；这是修 K1-1 的设计代价。
- 并发被压：拆锁后快路只拿 `started` 一瞬；重起同一张卡的线程互等（与改前一样）；`starting` 里丢旧设备的那几秒只压同卡的重起（K4-3）。没找到别的压并发的形。
- 推翻它的现象：账层或池层造出一次借被拒而这时真卡上有空，原因不是 K4-2 那一形。

## 8 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 罩不罩得住 | 一致（造得出的形里）；H-A、S 与 R 同走丙造不出 | 72 形扫、攻方原形与扩展：做成的 33 形里红的 7 形全带 `rlin`（K2-3）；H-A 被恢复阶段拒，不是红 |
| K1-2 放的时机 | 一致；旁证一处不归这两块 | 405 段 `implemented` 里 371 段 `iso_end` 回 0；34 段停在 18/30 的是 `qroot+qlast` 形，撤掉补法的 `none` 臂同数 |
| K2-1 认回退 | 冲突一处（读不出与没有记录同一个结局）；认错没造出 | `mount.rs`:2163、2165 的 `.ok()` 吞掉读不出；`anylin` 对 `implemented` 逐格相同 |
| K2-2 罩不罩得住 | 一致 | 128 形 256 段 0 红，120 形复活集全在 R 那一槽，含 R_old 自己是回退 |
| K2-3 认不出回退 | 冲突（同 K2-1 两句）；可达面三形各 4/4 红；有读得出的信息 | 谱系片 / R_old 根槽 / R_old 账读不出；点名的打包单元读不出 ⇒ 当「可能是回退」并现行账里全部已释放的，三形与 72 形扫 0 红，代价同对照臂（116 槽） |
| K2-4 多护的放与第九项 | 一致；第九项字面规则没说 | +24 槽、放得掉、0 段被拒；D28 第 27 行没写 R_old 账里的落点 |
| K3-1 列卡带借据号 | 一致 | 销账的四条路都核过，多给造不出；「借据先于缓冲丢」仍无会红的检查 |
| K3-2 三台不同台 | 一致（多销没有）；永远销不掉归 K4-2 | 设备号核对在 `gpu_memory_ledger.rs`:911，没有测试钉它（G1 绿） |
| K4-1 死锁 / 两台设备 | 一致 | 锁序 `starting`→`started`→账锁无反向；去 `starting` 的变异 48/48 红 |
| K4-2 重起时不等不 poll | 账多给没有；少给出不来有一形（冲突，窄） | 丢了的设备上没等成的借据占着量，后面的借放不下就走不到重起：账层探针 1000 对 4000 被拒 |
| K4-3 旧设备丢在 `started` 之外 | 一致（读码） | 只压同卡重起，快路不受影响；顺序没有测试钉（G2 绿）；真设备上多久没量 |
| K5 判别力 | 第十块 10 行全红；E1/E2/E3/E7 照绿；第十一块账层全红；`take_in_the_devices_found_idle` 里面与下游各层叫停无用例 | 窄取法的「窄」（E3）没有测试钉；下游各层 `&|| false` 只是警告 |
| K6-1 账多给 | 造不出；Poll 后缓冲交回驱动的前提复核不了 | 要真卡量 |
| K6-2 放得下却未跑 / 弃卡 / 压并发 | 有一形（同 K4-2）；其余没有 | 不在用户已认清单里 |

## 9 没做什么

- 不替主 agent 采纳或出判决。上面「判」都是我这一条腿的判定。
- 没起 GPU 设备；第十一块的真卡行为（Poll 之后缓冲交回驱动、旧设备丢时 wgpu 等多久、`nvidia-smi` 读数）一条都没量。
- 没跑 checker 档任何测试（含带 gpu 特性的）、没跑崩溃枚举、没跑门禁阶段。checker 档的 X1、X2、X4、X4b 只在逐字搬运的小程序里判了红，不是真用例。
- 层 0 全量没跑（超过 2.6 × 10⁸ 个状态）；所有「红多少」都是随机历史的抽样（每形 2 至 24 段），不是枚举。
- `admission_formula_on_the_mounted_state` 等第九项相关的二进制没复跑，只跑了 `raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table` 整份（13 全过）。
- 1536 槽盘没做；`tri` 臂只在 384 槽盘上量过。
- S 与 R 同走丙、R_old 被 R 之后的发布覆写出根环两形没造出（挂载被拒或回退目标被 F 拒），不等于做不成。
- 旁证那一形（`qroot+qlast` 之后隔离停在 18）没追原因，也没在不带环境变量臂的原件上复现；探针的环境变量臂都只在草稿副本里。
- 拷贝里的数不进 kb（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」：副本上量的数要在入库装置上重做才能引）。

## 10 要主 agent 跑的命令（我不跑的）

- 真卡：账层探针 `probe_kept_leases_on_a_lost_device` 对应的池层序列——借一笔建满、让它等设备超时（设备上留一段长活）、杀掉设备、再借 4000 MiB：看是否 `GPU_CARD_LEASE` 前就打「本次未跑」。工作目录仓根，`SINGLEFS_HEAVY_TESTS=user-request bash scripts/run-with-memory-cap.sh 16G …`，跑完看那一行。
- checker 档：`cargo test --offline -p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_amplification::tests::the_stop_request_remembered_in_this_run_is_the_one_handed_to_the_gpu_judge_and_the_cpu_checker gpu_unit_checks::tests::the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first`，对 X1、X2、X4、X4b 的 4 行变异各跑一遍，看是否与我的模型一致。
- 复跑我的数：把 `research/prompts/m3-batch10z-jk-code-r1-sonnet-model/` 拷到草稿目录，按 README 把 `src/probe_jk.rs` 放进冻结副本拷贝的 `crates/singlefs-harness/tests/`、`src/probe-arms-on-mount.diff` 打在 `mount.rs` 上，`scripts/jobs-sw.txt`、`jobs-swt.txt`、`jobs-rb.txt` 用 `scripts/runjobs.sh` 起，`scripts/agg.py` 汇总；比 `logs/summary-*.txt`。

## 11 草稿目录与清理

开跑前拷了冻结副本：`~/.local/share/singlefs/work/m3-batch10z-jk-code-r1-sonnet/tree-a`（基线）、`tree-p`（探针与环境变量臂）、`tree-m1` 至 `tree-m4`（变异驱动的四个工作树），编译目录 `target-a`、`target-p`、`target-m1` 至 `target-m4`、`target-m3`（含 gpu 特性的 `cargo check`）、`target-scan`、扫源码小程序 `scan-model`。
交回前按 `.claude/singlefs-ai-sop/rules/session-wrapup.md`「5. 子 agent 交回之前，删掉自己建的编译目录与仓副本」逐个删除（下面「清理结果」一行写删了哪些）；脚本、日志、探针留在模型目录，不依赖这些副本。

清理结果：已删 `~/.local/share/singlefs/work/m3-batch10z-jk-code-r1-sonnet/` 下的 `tree-a`、`tree-p`、`tree-m1`、`tree-m2`、`tree-m3`、`tree-m4`（仓副本）与 `target-a`、`target-p`、`target-m1`、`target-m2`、`target-m3`、`target-m4`、`target-scan`（编译目录）；该目录下剩的只有脚本、日志与 `scan-model` 源码（3.9M），没有编译目录和仓副本。
