# m3-inv-b-r1 攻方腿报告（2026-09-30）

<!-- doc-lint:not-numbers B1 B2 B3 B4 B5 B6 B7 -->

批次：records/2026-09-30-里程碑三开工第一轮.md#2。这是接续的腿：上一条同立场的腿撞限额中断，它落盘的原型（副本上的谓词模块、三份用例）与六份日志核过能用，接着用；这一条补了 V1 改法、I-3.9 两种守卫的逐镜像对照、多条记录发布的崩溃状态那一份用例，重跑了两批随机历史，写了报告。

## 复跑与文件

复跑：`bash research/prompts/m3-inv-b-r1-opus-model/rerun.sh /tmp/claude-1000/m3-ring-r1/tree <草稿目录> 6`（把冻结副本拷到草稿目录、打 `proto.diff`、逐条经内存包装跑；只跑 `singlefs-harness` 的测试目标，不碰 checker 档包）。`proto.diff` 对冻结副本 `patch -p1 --dry-run` 六个文件都过。

```text
054e7fe80ac78de67be92f7e938bc6fe39e60c5ddcee091e2abebe26f10c956c  proto.diff
a3f4d20267d2f9e336d80419afdaa1e0d49ce9fae6dae499b6cbc7be1ba9ded7  rerun.sh
0392a44f8de6b37e4d0e50e521c18db2fe5f91e30b1fc149c6ab32c1dae2ff6a  logs/s1-b3-rollback.log
3cb456b32e8ee735815bce73570bcc336a95dc76ecc84470ce25833022cc8fe7  logs/s1-b3-sweep.log
2b78c67128bc875ea9d15e35eb7cb3084aa57d8f1248f026964ef3c7cbe9ca00  logs/s1-b7.log
bd99ffcb747ef34effdc63744260789f7bd9bd3d75e6c77d9bc9dcac896e65f5  logs/s1-broad48.log
bbdc1d2b774248d7fb4fbf1e50f6dd34764dd8605fad2bc3b7d4abb994d8d63d  logs/s1-broad4.log
260a7d85206a25781a6e6fa2d171275f41f064bf7de8800d0339a0e43784734c  logs/s1-rollback48.log
3847c033d198c38035018757043ab118f336d4764ab41b70d68c6f8ed3b9fa63  logs/s2-b3-rollback.log
9119c0592112bd2b9e34dc07e26e48381b28fcf0d9bd9ac65b10095bf9a3c53e  logs/s2-broad48.log
35ba74b542e87bc18303b2c7df068aa7ff7ec38a44df29e9b0150adc14c71f4e  logs/s2-multi-record-12-dump.log
c4d3ef3cb0b1b59be120ac6db0a94e054e1a0cdd744a89d914583922d3c3f565  logs/s2-multi-record-12.log
98fd4b986d95944e68f7325bb437de5dd2a6a51fa8d041338dd09b451a988b8e  logs/s2-multi-record-3.log
c6ada65884fb8b3219a2c6e2b92eed0303dcc62f7ca72d53ff0bfffefd886af7  logs/s2-rollback48.log
```

`logs/s1-*` 是上一条腿跑的（那时还没有 V1 与守卫对照两样），`logs/s2-*` 是这一条跑的。全部数都是副本上的数，不算入库装置上的数。

装置是什么：`proto.diff` 在副本的 checker 里加一个只读模块 `walk/proto_b.rs`，把正文 B1–B6 的候选措辞各写成一个谓词（不进 `IMPLEMENTED_INVARIANTS`），给 I-3.9 的守卫加一个三档开关（今天的、正文 B7 收窄的、完全不设防的）；四份 harness 用例在合法历史的每张镜像上数每个谓词判了几次、红了几次。盘全是内存稀疏盘。

## 各格判定一览

| 格 | 被攻的措辞 | 判定 | 依据（副本上量的） |
|---|---|---|---|
| B3 下界 | I-3.5 ② 下界按回退候选集判 | **打中：合法镜像上红** | 管理员回退之后回退目标离开候选集的那张镜像；两批随机历史 173 张镜像红（2976 张里），池级 checker 同一张上 0 条红 |
| B3 上界 | birth ≤ R_first.txg | 没打中 | 667872 次判、0 红 |
| B3 点名项 | 校验和相符则头逐字段等于点名项 | 没打中 | 1498556 次判、0 红；崩溃状态上另 45080 次 0 红 |
| B2 | 同 (实例, txg) 的根与记录新根段相等（只比计数器最大、比每一条、只比带末条标志的三种写法） | 没打中；条款那一半有一处空 | 三种写法各 0 红；多条记录的发布今天每条都带同一份新根段，但这句只在代码注释里、决策正文没有 |
| B5 (a) | T < 引用这张表的根的 txg | 没打中 | 40363 次判、0 红 |
| B5 (b) | 同一实例的行在候选集两张表里相同 | 没打中；按今天的条款推得成立，但我的历史里没有一段改写过已有的行 | 40363 次判、0 红 |
| B7 守卫 | 比 L 晚的候选根都走得全才判 | 没打中 | 2976 张合法镜像上与今天的守卫判定逐张相同；组合坏镜像红、对照格不红 |
| B7 对照格 | 底稿说现成镜像造不出 | **造出来了** | 抬 F 的空发布之后再覆盖写，把最后一次空发布那条根的树表翻坏：不设防的守卫在它上面误红，收窄的不红 |
| B1 | 指针出生树等于被指单元头（树表根指针、树根指针、数据指针、实例表指针） | 没打中 | 五个谓词各 0 红 |
| B4 | previous_snapshot_txg、记账代、改动计数上下界 | 没打中 | 四个谓词各 0 红 |
| B6 | 系统配置世代号、根槽位置、journal 记录 txg 次序 | 没打中；我推的一个反例被实测否掉 | 六个谓词各 0 红 |

「没打中」都是这一条腿一次抽样的结果，取样范围在末尾「没打中的形状」一节逐格列。

## B3　诞生代号下界（打中）与上界、点名项（没打中）

### 打中的历史

被攻的措辞是正文第三节 B3 的第一句（`research/prompts/_m3-inv-b-r1-body.md:31`：「I-3.5 ② 按回退候选集判（R_first、R_prev 的写法），射程收在候选集」），写法照底稿主表第 3 行：R_first 是候选集里引用落点的根中 txg 最小的那条，R_prev 是候选集里 txg 严格小于 R_first 的根中最大的那条，R_prev 存在且数得全时要 birth > R_prev.txg。

两段写死的历史，起点都是第一个文件之后（A，txg 3），全程一个实例、不崩溃、不注入故障：

1. 「抬 F」那一段：覆盖写四次（txg 4–7）→ 管理员回退到 txg 4 那条根（回退那次发布是 txg 8）→ 抬 F 到 5。
2. 「轮出环」那一段：覆盖写两次（txg 4、5）→ 回退到 txg 4（回退那次发布是 txg 6）→ 再覆盖写 24 次，txg 4 那条根轮出根环。

`logs/s2-b3-rollback.log` 里红的两步，原样：

```text
PROTO_B3 raise step=6 op=Some(RaiseRollbackFloor) checker_violated=[] b3_lower_evaluated=104 b3_lower_red=8 v1_evaluated=80 v1_red=0 v1_exempted=24 first=盘 0 槽 50182 类 1 诞生代号 4 ≤ R_prev (实例 1, txg 7)，R_first (实例 1, txg 8)
PROTO_B3 turn step=25 op=Some(PublishOverwrite) checker_violated=[] b3_lower_evaluated=624 b3_lower_red=8 v1_evaluated=600 v1_red=0 v1_exempted=24 first=盘 0 槽 50182 类 1 诞生代号 4 ≤ R_prev (实例 1, txg 5)，R_first (实例 1, txg 6)
```

两张镜像上池级 checker 的 49 条一条不红（`checker_violated=[]`）。

每一步由哪一句许可：

- 回退之后新根重新引用旧单元：`.claude/kb/decisions/23-journal的角色与格式.md:376`：「新根的用户可见树…的根指针取 R_old 那一版的」；实现 `crates/singlefs-core/src/mount.rs:5036`：「复活：R_old 引用的用户可见单元里 cur 的账记成已释放的，改回已分配，分配代写回 R_old 那一版账里的」。所以 txg 4 写的数据单元（槽 50182）、extent 树与 inode 树的单元，被 txg 4 的根与回退那条根引用，夹在中间的根不引用。
- 回退不换实例、不写行：同一行「不新增实例、不取号、不写实例表行」。中间那几条根没有被抛弃，留在候选集里。
- 之后 R_old 离开候选集有两条合法的路：抬 F 越过它（F 的上限由 I-7.9 判，这张镜像上不红），或它轮出根环。离开之后候选集里最早引用那个单元的根变成回退那条根，R_prev 变成回退之前最后一条根，单元的诞生代号（4）不大于它。

根上的原因不在「候选集」这个射程，在 I-3.5 自己的前提：`.claude/kb/invariants.md:137`：「① `Reach(b)` 在 txg 序上**连续无洞**」。回退是一次向前发布之后（2026-09-26 定），一个单元的引用集合可以有洞；R_old 还在候选集里时 ② 碰巧不红（R_first 就是 R_old），但 ① 在同一张镜像上已经不成立。底稿主表第 3 行「推不出的」那一格的第 ① 处（`research/prompts/c395-invariants-draft.md:21`：「把根环候选根当成 QB1 里的快照 S 用」）不成立：候选根不是快照，回退根是一条重新引用旧版本的根。

### 用户动作放开扫

照定义第 3 条，前缀固定为 A 之后覆盖写 k 次，回退目标、抬 F 的步数、回退之后写几次放开（k ∈ {2, 4}，回退目标取根环从新到旧第 0–5 条，之后抬 F 0 / 2 / 4 / 6 / 8 步或不抬再写 0 / 3 次）。`logs/s1-b3-sweep.log`：

```text
test proto_b3_sweep_the_user_steps ... PROTO_B3_SWEEP red k=4 rollback_index=4 tail=raise4 red_steps=1 completed=true
PROTO_B3_SWEEP cells=84 red_cells=1 not_completed=0
```

84 格里只 1 格红，说明写死的后缀很容易避开落点：要红得「回退目标离开候选集而它之后的旧根还在」，这 84 格里只有一格的抬 F 恰好落进那个窗口。随机历史不固定后缀，两批各 48 段 × 30 步（`logs/s2-rollback48.log`、`logs/s2-broad48.log`）：

```text
PROTO name=B3-lower evaluated=325940 violations=1464 images_red=150 first=seed=1000004 step=28 start=AfterFirstFile ops=[PublishOverwrite>RaiseRollbackFloor>RollBackWhileMounted>PublishOverwrite>RaiseRollbackFloor>RollBackWhileMounted>RaiseRollbackFloor>RollBa
PROTO name=B3-lower evaluated=310068 violations=184 images_red=23 first=seed=2000015 step=15 start=AfterFirstFile ops=[RaiseRollbackFloor>RaiseRollbackFloor>CloseAndMountWritable>PublishOverwrite>RollBackWhileMounted>PublishOverwrite>RaiseRollbackFloor>Publish
```

（两行在 `first=` 之后截在 260 字符，全行在日志里。）偏回退的那一批 1488 张镜像里 150 张红，各类操作都抽的那一批 1488 张里 23 张红；两批 96 段都跑完（`endings={"completed": 48}` 两行）。

### 打中之后的四句

1. 分不分辨臂：B3 这一格里三条措辞（下界、上界、点名项）分得开——只有下界红，上界与点名项在同样的镜像上 0 红。下界自己没有第二条臂可比。
2. 被判的系统看不看得到判别它的东西：**看不全**。要分的是「回退根合法地重新引用旧单元」与「一个单元被盖了偏小的诞生代号」。回退不写行、不留标志；R_old 轮出根环之后盘上没有它；分配记录的分配代被写回 R_old 那一版的，与头一致。盘上剩下的痕迹只有一样：回退根的树表里用户可见树的根指针出生 txg 早于这条根自己、而且前一条根不引用那个落点（下面 V1 用的就是它）。只改一个数据单元的偏小代号分得出；整条引用链一起偏小、连 inode 树根也偏小的，与回退分不出。
3. 满足的是判据字面的哪一句：正文第五节第一条（`research/prompts/_m3-inv-b-r1-body.md:56`：「一条措辞在任一合法镜像上判红 ⇒ 出局」）的前半句。不是「推不出条款支撑」那半句——下界的条款在，是条款自己与回退的定案对不上。
4. 跑前条款给的改法：正文没给改法，只给了「出局」与「推不出的记欠账」两个去向。按字面，下界这一半出局；I-3.5 ① 与回退定案的矛盾是另一笔账（现有的 I-3.5 那一行还是「未实现」，没有在跑的判定会因此红）。

### 我提的改法 V1（只在我的模型上量过、被攻过零轮）

V1：R_first 的树表里 inode 树根指针的出生 txg ≤ R_prev.txg，且那个根指针的落点 R_prev 不引用——这条 R_first 的用户可见树是整棵从候选集之外的一版继承来的——这时它名下「最早由它引用」的落点都不判下界；否则照原措辞判。

| 格 | 原措辞 | V1 | 量过 / 推的 |
|---|---|---|---|
| 「抬 F」那一段第 6 步 | 红 8 处 | 0 红，豁免 24 处 | 量过：上面 `PROTO_B3 raise step=6` 那一行 |
| 「轮出环」那一段第 25 步 | 红 8 处 | 0 红，豁免 24 处 | 量过：上面 `PROTO_B3 turn step=25` 那一行 |
| 偏回退的 48 段 | 150 张红 | 0 张红 | 量过：`PROTO name=B3-lower-v1 evaluated=304628 violations=0 images_red=0 first=-`，豁免 `PROTO name=B3-lower-v1-exempted evaluated=21312 violations=0 images_red=0 first=-` |
| 各类操作都抽的 48 段 | 23 张红 | 0 张红 | 量过：`PROTO name=B3-lower-v1 evaluated=301192 violations=0 images_red=0 first=-`，豁免 `PROTO name=B3-lower-v1-exempted evaluated=8876 violations=0 images_red=0 first=-` |
| 放开扫的 84 格 | 1 格红 | 没量 | 那一份日志是上一条腿跑的，当时还没有 V1 |
| 底稿的下界坏镜像（B 的数据单元 4 → 3，指针与分配代一起改） | 该红 | 该红：B 的 inode 树根是 B 自己写的，出生 txg 4 > R_prev 的 3，不豁免 | 推的，没造这份镜像 |
| 回退根的根槽没落盘、由记录施加、下一次挂载的写行根接在它后面 | 会红 | 不红：写行根照抄用户可见树，它的 inode 树根同样满足豁免条件 | 推的 |
| 多文件之后：回退根与它后面一条根的根槽都读不出，再后面一条根只重写了别的文件 | 会红 | 会红：那条根的 inode 树根是新写的，不豁免，而它继承的别的文件的单元诞生代号偏早 | 推的；今天一个文件、走不到，里程碑三删除与多文件落地之后才有 |
| 整条引用链连 inode 树根一起盖了偏小代号的坏镜像 | 红 | 不红（被当成回退豁免） | 推的：V1 丢掉的判别力 |

V1 的代价是最后一行：它把「回退」与「整棵树一起偏小」看成一回事。要不要下界、用不用 V1，是判决的事。

## B2　记录新根段与根（没打中；条款有一处空）

三种写法都量了：只比这块盘上同 (实例, txg) 那一组里计数器最大的一条（B2-max，底稿的写法）、比那一组每一条（B2-all）、只比带「本次发布末条」标志的那条（B2-last）。

- 随机历史两批（回退、抬 F、可写挂载、冷启动、零单元发布都在里面）：`PROTO name=B2-all evaluated=40698 violations=0 images_red=0 first=-` 与 `PROTO name=B2-all evaluated=39304 violations=0 images_red=0 first=-`，B2-max、B2-last 同数同为 0。三个谓词判的次数相同，说明这两批里每一组都只有一条记录——随机历史的内容长度最多装满一个数据单元（`crates/singlefs-harness/src/history.rs:250`：「InsideOneDataUnit {」那一族），一次发布切成多条记录的那一格随机历史取不到。
- 所以另写了一份用例专取那一格（`inv_b_proto_multi_record_crash.rs`）：产品路径上顺序写 N 个数据单元就是 N 条记录。历史 mkfs → 暖机 → A → 顺序写两个数据单元的 B（两条记录）→ 顺序写 N 个数据单元的 C，C 的写按 `writes_and_segments` 切段、段内整写子集枚举；每个崩溃状态判三张镜像（崩溃镜像、可写挂载之后、再覆盖写一次加顺序写两个数据单元之后）。N = 3 时（`logs/s2-multi-record-3.log`）：`PROTO_MR name=B2-all evaluated=3116 violations=0 images_red=0 first=-`、`PROTO_MR name=B2-max evaluated=2516 violations=0 images_red=0 first=-`；N = 12 时（`logs/s2-multi-record-12.log`）：`PROTO_MR name=B2-all evaluated=2056 violations=0 images_red=0 first=-`、`PROTO_MR name=B2-max evaluated=1472 violations=0 images_red=0 first=-`。B2-all 的次数多于 B2-max，多条记录的组确实判到了。

条款那一半：正文问「一次发布切成多条记录时前几条的新根段该写什么（今天的条款有没有）」。**没有。** `grep -n '新根段' .claude/kb/decisions/23-journal的角色与格式.md` 命中第 21、32、82、154、155、156、158、443、449 行，没有一行说前几条写什么；定案那一句只说施加（`.claude/kb/decisions/23-journal的角色与格式.md:443`：「施加一条记录 = 把所选根的这四个字段换成记录里的」）。「每条都带同一份」只在实现的注释里（`crates/singlefs-core/src/transaction.rs:6728`：「每条都带整次发布的新根段（已定项 15）」）。所以：

- B2-max（只比计数器最大那条）有条款支撑（新根段是那次发布的根的四个字段），量过 0 红。
- B2-all 今天的实现满足、量过 0 红，但要立成不变量缺一句条款；立之前先补条款，是条款变更。
- 「根槽没落盘、只由记录施加出来的那一版」没有根可比，三种写法都不判它；崩溃状态里这类状态在记录段的 63 + 34 个子集里都有，谓词在它们上面没有对象、也没有误红。

我试过而没成的反例：同一个 (实例, txg) 出两份内容不同的记录。落盘中途失败的发布是原样重发（`crates/singlefs-core/src/transaction.rs` 的 `resend_the_frozen_publish`，checkpoint_txg、计数器、字节都不变），崩溃之后换实例、txg 取环里见过的最大值加一，两条路都造不出；这一句是读代码推的，失败注入的历史没跑。

## B5　实例表行 T（没打中）

(a)：`PROTO name=B5a evaluated=18266 violations=0 images_red=0 first=-`、`PROTO name=B5a evaluated=22097 violations=0 images_red=0 first=-`；把射程从候选集放到根环里全部自证过的根（B5a-all-roots）也是 0 红（19639、23300 次）。

(b)：`PROTO name=B5b evaluated=18266 violations=0 images_red=0 first=-`、`PROTO name=B5b evaluated=22097 violations=0 images_red=0 first=-`。

按今天的 D18 已定项 11 原文重判 (b)：

- 回退不再写行（`.claude/kb/decisions/18-块里携带什么信息.md:305`：「按实例代号唯一、后写覆盖，只在恢复与实例切换时写」），底稿担心的「回退行」那一支没有了。
- 「后写覆盖」什么时候真的改一行：一次恢复择到实例 i 的根而表里已有 i 的行，只可能是写那一行的实例 j（j > i）的根这一次都读不出。这次恢复给 [i, 新实例) 每个实例写行，j 在其中、拿到的是中间实例行（`.claude/kb/decisions/18-块里携带什么信息.md:306`：「严格介于所选根的实例与新实例之间的实例写 (i, 0, 0)」）。按最新根的表判，j 的根（txg > 0）全被抛弃、不在候选集里；带着旧行的那张表只被 j 与 j 之后、新实例之前的实例的根引用，它们同样拿到中间实例行。所以候选集里不会同时有两张表对同一实例记着不同的行。(b) 按今天的条款**推得成立**，射程必须写「候选集」，放到「根环里全部自证过的根」就不成立（被抛弃的根带着旧行）。
- 这一段是推的。我的历史里没有一段真的改写过已有的行：随机历史的「崩溃恢复抛弃根」那一步今天被可写挂载在取号之前拒掉、造不出被抛弃的根（`crates/singlefs-harness/src/history.rs` 那一格的文档注释原话是「这一步不再造出被抛弃的根」），B5b 判到的全是同一行被照抄进下一片表。要量这一格得照 `tests/common/mod.rs` 的 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 连造两次，没做。

## B7　I-3.9 的空当

### 组合坏镜像（实测）

造法按今天的多层分配记录树改了两处，其余照底稿：释放代 4 → 3 用现成的 `mutate_allocation_leaves` 改覆盖写之后那一版两片叶里的 24 条已释放记录（不是底稿的 16 条、也不是槽 50253）；让 A 那条根数不全，翻的是 A 的分配记录树盘 0 那片叶（槽 50245）两份各一个字节、不重封。`logs/s1-b7.log` 原样：

```text
PROTO_B7 image=combined guard=Today i39=NotApplicable("有候选根走不完（单元读不出，或走到条目格式没有条款的形状）：引用集合数不全") violated=["I-2.1", "I-4.8", "I-7.4"]
PROTO_B7 image=combined guard=Narrowed i39=Violated("盘 1 槽 50180 的分配记录释放代 3 不在 (3, 4] 里：还引用它的最新一条有效根是 txg 3，最早不再引用它的是 txg 4") violated=["I-2.1", "I-3.9", "I-4.8", "I-7.4"]
PROTO_B7 image=A-leaf-50245-unreadable-only guard=Narrowed i39=Holds violated=["I-2.1", "I-4.8", "I-7.4"]
PROTO_B7 image=release-4-to-3-only guard=Today i39=Violated("盘 1 槽 50180 的分配记录释放代 3 不在 (3, 4] 里：还引用它的最新一条有效根是 txg 3，最早不再引用它的是 txg 4") violated=["I-3.9"]
```

今天的守卫（`crates/singlefs-checker/src/walk.rs:3282`：「有候选根走不完（单元读不出，或走到条目格式没有条款的形状）：引用集合数不全」）在组合镜像上报不适用，收窄之后判违例；只翻叶、不改释放代的那份收窄之后 I-3.9 成立。同时翻 A 的两片叶（50245 与 50246）结果相同（同一份日志 `image=combined-both-A-leaves` 三行）。一起红的 I-2.1、I-4.8、I-7.4 是翻坏单元带来的，与底稿预期一致。

### 对照格（造出来了）

底稿说这一格现成镜像造不出（`research/prompts/c395-invariants-draft.md:45`：「现成镜像造不出」），给的路子是「先在带文件的一版上推一次空发布再覆盖写」。抬 F 的空发布就是这样一次发布：用步 5 现成的 `pool_before_raising_the_floor`（`crates/singlefs-harness/tests/checker_known_bad_images.rs:3775`）抬 F 到 11（末一次空发布 txg 16，它不换用户可见树），再覆盖写一次（txg 17，释放 txg 14 那一版的用户可见单元，释放代 17，真正的 L 是 16）；把 txg 16 那条根自己的树表（槽 50438）两份翻坏。那几个落点藏在读不出的树表下面，算出来的 L 是 15、T 是 16。

```text
PROTO_B7 control last_raise_txg=CheckpointTxg(16) its_tree_table_slot=50438 overwrite_txg=CheckpointTxg(17)
PROTO_B7 image=control-later-root-tree-table-unreadable guard=Narrowed i39=Holds violated=["I-2.1", "I-3.1", "I-4.8", "I-7.4"]
PROTO_B7 image=control-later-root-tree-table-unreadable guard=Unguarded i39=Violated("盘 1 槽 50409 的分配记录释放代 17 不在 (15, 16] 里：还引用它的最新一条有效根是 txg 15，最早不再引用它的是 txg 16") violated=["I-2.1", "I-3.1", "I-3.9", "I-4.8", "I-7.4"]
```

完全不设防的守卫在这份上误红（合法的释放代 17 被判出区间），收窄的守卫不红：对照格分得出「收窄」与「摘掉」。这份镜像可以直接当「守卫摘掉」那条变异的用例。

### 收窄之后会不会在合法镜像上多红

收窄与今天的守卫只在「有候选根走不全」的镜像上不同。两批随机历史 2976 张镜像逐张对照：

```text
PROTO name=B7-narrowed-differs-from-today evaluated=1488 violations=0 images_red=0 first=-
PROTO name=B7-today-not-applicable-for-an-incomplete-root evaluated=1488 violations=0 images_red=0 first=-
```

（两批各一组，数相同。）合法镜像上一张「候选根走不全」的都没有，两种守卫判定逐张相同。按写法推：收窄只在「比 L 晚的根都走得全」时判，真正的 L 若比算出的大，它就是一条比算出的 L 晚、走不全的根，那一条会被跳过；所以收窄不会因为 L 算小而误红。没量的一格：崩溃状态上两种守卫的对照（多条记录那份用例只按今天的守卫判）。

一处实现上要留意的（不是误红）：副本里收窄之后若每条已释放记录都被跳过，报不适用、理由另写一句；若只跳过一部分，其余照判、整条可以报成立。坏镜像上「成立」只说明判到的那几条成立。

## B1、B4、B6（找反例，没打中）

B1 量了五个谓词：实例表指针的出生 txg、实例、尾段等于实例表单元头；实例表指针、数据指针的出生树等于头里偏移 43 的出生树；树表指针、树表条目根指针的出生树等于被指码 2 节点头里的树 ID。两批各 0 红（偏回退那一批 15765 / 18505 / 18505 / 63060 / 18505 次，另一批 14635 / 18742 / 18742 / 58540 / 18742 次）。码 2 那一类今天的实现里「指针出生树 = 头里的树 ID」处处成立，但头里取哪个字段的条款没定（C289），量到的只是实现的样子，不替条款作答。码 2 内部节点的子指针、inode 树内部与叶的指针这一轮没写成谓词。

B4：`previous_snapshot_txg`（非 inode 树恒 0、inode 树 0 或 ≤ 根 txg）、记账代 = 根的 txg、对象出生代 ≤ 改动计数 ≤ 叶容器诞生代号，四个谓词两批各 0 红（记账代 236475 + 219525 次）。回退根的记账行按回退那次发布现算、代 = 回退那次的 txg，没有成为反例。今天没有快照，`previous_snapshot_txg` 全是 0，inode 树那一支「≤ 根 txg」没有被真正判到过。由记录施加出来的那几版谓词没判（只判了根）。

B6：系统配置世代号奇偶与两槽差（5936 + 5848、2976 + 2976 次）、根槽位置（21596 + 20974 次）、journal 记录 txg 不减与换实例严格大（39878 + 38306、3614 + 4766 次），全 0 红；多条记录那份用例的崩溃状态上同样 0 红。

我推过一个 B6 journal txg 的反例，实测否掉了，记在这里免得下一轮再走：一次发布 12 条记录，崩在只有最后一条记录落盘（前 11 条没落、根没落）；恢复一条都不施加，我以为新实例会从「前缀末 + 1」接着写、把计数器排在那条残留记录前面，于是按计数器排 txg 先大后小。实际新实例接在环里读得出的最大计数器后面（`.claude/kb/decisions/23-journal的角色与格式.md:369`：「前缀末的计数器取环里读得出的最大那一个」），`logs/s2-multi-record-12-dump.log` 那个状态三张镜像的盘 0 记录（计数器, 实例, txg, 标志）：

```text
PROTO_MR_DUMP seg=1  mask=110000000000000000000000 stage=crash journal=[(1, 1, 1, 1), (2, 1, 2, 1), (3, 1, 3, 1), (4, 1, 4, 0), (5, 1, 4, 1), (17, 1, 5, 1)]
PROTO_MR_DUMP seg=1  mask=110000000000000000000000 stage=mounted journal=[(1, 1, 1, 1), (2, 1, 2, 1), (3, 1, 3, 1), (4, 1, 4, 0), (5, 1, 4, 1), (17, 1, 5, 1), (18, 2, 6, 1), (19, 2, 7, 1)]
```

（两行里 `kinds=[…]` 那一段 24 项我删掉了，其余原样；全行在日志里。）残留的 (17, 实例 1, txg 5) 排在新实例的 18 之前，txg 5 < 6，谓词成立。

## 没打中的形状

| 形状 | 取样范围 | 打的是哪几格 |
|---|---|---|
| 随机历史，偏回退的比重 | 48 段 × 30 步、1488 张镜像，种子 1000000 起 | 全部谓词 |
| 随机历史，各类操作都抽 | 48 段 × 30 步、1488 张镜像，种子 2000000 起 | 全部谓词 |
| 多条记录的发布，C 三条记录 | 69 个崩溃状态 × 3 张镜像：记录段 6 个写的 63 个非空子集全枚举，根槽段 1 个，系统配置轮换段 3 个，单元段只取全不落与全落 | 全部谓词；池级 checker 207 张 0 红 |
| 多条记录的发布，C 十二条记录 | 40 个崩溃状态 × 3 张镜像：记录段 24 个写取 34 个子集（只有后 k 条、只有前 k 条、只有首尾两条、缺中间一条，两盘同一子集；加全落） | 同上，120 张 0 红 |
| 回退之后用户动作放开 | 84 格 | 只 B3 下界 |
| B7 两份坏镜像 | 6 张镜像 × 3 种守卫 | 只 I-3.9 |

每批的段数与估时：先跑 4 段（四线程 37 秒），按它估 48 段约 7.5 分钟；实际四线程 391 秒与 597 秒，六线程 478 秒与 508 秒。没给批设限时。

没取到的：

- 被崩溃恢复抛弃的根（随机历史今天造不出，见 B5）；连造两次、改写已有行的那一格。
- 实例切换：今天的实现里没有「挂着时换实例」这个动作（只有预留与原样重发），底稿第 3 行「推不出的」第 ② 处（切换照旧的数据单元）与第 4 行「点名项的诞生代号不等于记录的 txg」没有镜像可判；今天每个点名项的出生 txg 都等于记录的 txg，那只是实现的样子。
- 落盘中途失败、原样重发的历史；设备失而复得；读故障下的可写挂载。
- 层 0 那两条固定脚本流本身没跑（它们在 checker 档包里）。

## 这条腿自己的限度

- **崩溃状态没有照层 0 的枚举域取全。** 定义要每段历史的崩溃状态一个不落、调 `enumerate_layer0`。我没调它：它在 `singlefs-checker-tier` 包里，那个包的测试二进制这一轮不许跑，我也没另建一个依赖它的原型包。我在 harness 里用同一条切段规则（`writes_and_segments`）自己枚举整写子集，没有「原地覆写撕裂」那第三态。C 那次发布的全量按 `closed_form_state_count` 是 1073741891（三条记录）与 281474993487875（十二条记录），都超过约 10⁶ 的线，按定义这两段不该整段跑；我跑的是其中 69 与 40 个（另有一遍带打印的重跑 40 个），合计 149 个状态，没枚举的主要是单元段（30 与 48 个写）的子集与十二条记录那一段记录子集的绝大部分。这一条交主 agent：要照层 0 的域判这些谓词，得在 checker 档里做。
- 前缀（mkfs、暖机、A、B）没有枚举崩溃状态，只枚举了 C 那一次发布。
- 随机历史只看每一步做完之后的镜像，不看步内的崩溃状态。
- 每个「没打中」都只抽了一次样。
- V1 只在我的模型上量过、被攻过零轮；它的表里四行是推的。
- 内存包装：有一次按峰值表允许的 226M 跑，退出码 250（撞了这一条自己的上限），那一次的输出没用；之后给了理由按 4G 重跑，退出码 0。命令是 `run-with-memory-cap.sh 226M bash capped.sh 6 cargo test -p singlefs-harness --test inv_b_proto_multi_record_crash -- --nocapture`。另有两次 252（排不上）与一次 251（上限超过峰值两倍没给理由），都一行没跑。
- 开跑前 `ps` 看到别的会话的 `cargo build --release -q -p singlefs-core --example formal_probe_replay_bounded` 在跑，没有性能测量在跑；加 `nice -n 19` 照常跑。

## 什么现象会推翻这些结论

- B3 下界「打中」：若回退那次发布不再重新引用 R_old 的单元（条款或实现改成重写一份），或回退改成把中间的根判成被抛弃，这个打中就不成立；在入库装置上复跑「抬 F」那一段第 6 步 B3 下界不红，同样推翻。
- B7「收窄不会多红」：出现一张合法镜像（含崩溃状态）有候选根走不全，而收窄之后 I-3.9 判违例。
- B2「三种写法都不红」：出现同一个 (实例, txg) 的根与记录新根段不等的合法镜像——最该找的地方是失败重发与实例切换落地之后。
- B5 (b)「按条款成立」：造出一张镜像，候选集里两条根的表对同一实例记着不同的 (T, W)。
- B6「没打中」：新实例的第一条记录排在一条残留的旧实例记录之前（计数器更小）而 txg 更大。

## 没做什么

- 不判正推那几格，不替主 agent 采纳；副本上的数不算入库装置上的数。
- 没跑任何 checker 档测试、门禁、QEMU；没改仓里 `crates/` 一个字，改动都在草稿目录的副本上，以 `proto.diff` 入库。
- 没造底稿各行的「只红它」坏镜像（B7 之外），没写变异。
- 没为 B5 (b) 造改写已有行的历史；没为 B1 的码 2 子指针、inode 指针写谓词；没在由记录施加出来的那几版上判 B4。
- 没核正推腿的报告（禁读）。
- 草稿目录：上一条腿的 `/tmp/claude-1000/m3-inv-b-r1-opus/`（仓副本与编译目录已删，只剩六份日志与两个退出码文件）与这一条的 `/tmp/claude-1000/m3-inv-b-r1-opus-2/`（日志与退出码文件），都已拷进模型目录的 `logs/`。

## 引文核对

`python3 research/scripts/cite-check.py research/prompts/m3-inv-b-r1-opus-output.md --background research/prompts/_m3-inv-b-r1-background.md` 末行：

```text
  ✓ 核了 14 处引文，对上 14 处，没判 0 处（逐处列在上面）
```
