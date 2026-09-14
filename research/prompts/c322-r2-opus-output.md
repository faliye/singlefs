# C322（取号那一步的屏障怎么放没有条款） 第二轮 · Opus 攻方腿报告（2026-09-14）

- 立场：换攻击面，攻甲″（取号那两次超级块写之后、本实例第一个非超级块写之前至少一道完成了的屏障）为主、乙′（盘 0 超级块 → 屏障 → 盘 1 超级块 → 屏障，加同一条 I-7.7 改写）为辅；问 K2（回退）、K3（第四条路径）、K4（新条款的误报漏报）、K5（世代号），以及一起要定的三件。
- 输入：提示 `research/prompts/c322-r2-opus.md`；背景材料 `research/prompts/_c322-r2-background.md`；第一轮 `research/prompts/c322-r1-main-verification.md`、`research/prompts/c322-r1-opus-output.md`、`research/prompts/c322-r1-opus-model/`（只读；模型 K 的 A 段按第一轮模型 B 的切段与枚举重写，扩了臂与路径）。
- 模型在 `research/prompts/c322-r2-opus-model/`，构建与运行都在临时目录：
  - 模型 R `real_code/`：依赖仓里四个 crate（只读、不改），在内存盘上跑 mkfs → 取号 → 暖机 → 第一个事务（与层 0 同一串调用），再当成第二次可写挂载：取实例 2、补一道屏障（甲″）、借 `publish_first_file` 以实例 2 发 8 个单元（恢复写行那次发布没实现，这 8 个单元是它的替身，只留到它第一道屏障之前）；读每盘两槽、恢复与 checker 各自择到的超级块、根环、单元头写序，跑 checker 的 I-7.7。产物 `real-code.out`（4 行）。
  - 模型 K `k_model/`：只用 std。A 段五条路径 × 四条臂，按层 0 同一条切段规则（屏障关段、FUA 写关自己那一段、前面的段全持久 + 当前段任意真子集 + 全持久那一个）逐状态枚举；B 段三种世代号规则 × 四种读法 × 三条臂，第一个事务之后连着三次挂载的历史；C 段 18 个点名镜像过四种检查。产物 `k-model.out`（101 行）。
- 两个模型都是确定性的（没有 I/O、随机源、并发），跑一遍与跑 N 遍给的信息一样。判别力靠对照：模型 R 同一条流只换取号那次的世代号，撞号 true → false；模型 K 同一套路径只换臂，撞号 15 → 0。

## 〇、结论表

| 问 | 结论 | 跑前条款 | 细节 |
|---|---|---|---|
| K2 回退 | 「超级块已带新号、回退没生效」的状态在甲″ 下可达：回退路径 21 个，其中一个是 R1 = {acquire_superblock@disk0, acquire_superblock@disk1}（两盘取号写都落、回退那次发布一个字节没落）；乙′ 20 个，甲 51 个。按字面「之前没有持久效果」它违反，但三条臂都违反——D18（块里携带什么信息） 第 879 行与 D23（journal 的角色与格式） 已定项 16 早就要求「取号先写进每一份超级块、之后才动单元」，回退走的是同一个取号。按语义（下一次挂载选哪个根、施加哪些记录、已发布谓词给回退的孤儿判什么）甲″ / 乙′ 下 0 个状态有可观测后果，甲下 15 个 | 反向接受条款第 2 句不触发：甲″ 与乙′ 在这一格同形 | 二 |
| K2 让哪边 | 改那一句，不改取号。「回退把新号只写进它自己那次发布」在回退、恢复、切换三条路径上各 18 个状态撞号、18 个状态孤儿在下一次挂载后判成已发布，首次挂载路径 3 个撞号，还让今天 I-7.7「≥ 根环」那半句在 3 个合法状态上红 | — | 二 |
| K3 路径结构 | 没找到第四条：五条路径（第五条是新加的「写行之前先推一次抬 F 的空发布」）在甲″ / 乙′ 下撞号都是 0 | 不触发 | 三 |
| K3 字面那一句 | 甲″ 下有撞号的可达状态，来源是世代号，不是路径：仓里今天的代码上，第二次挂载取号写世代号 2（常量）+ 屏障 + 8 个单元带实例 2 落盘，两盘择到的都是 (世代号 5, 实例 1)，按择到的那一份下一次取号得 2，撞上那 8 个单元，checker 的 I-7.7 判 Holds。乙′ 下同样（模型 K：甲″ 153 / 729、乙′ 120 / 512 条历史） | 字面触发；不分辨臂，按「打中不分辨臂」归到 K5 的共同前提先修，不拿它判甲″ | 三、五 |
| K3 附带 | 「写行之前先推抬 F 的空发布」撞号 0，但四条臂下都有 67 个状态让旧实例的孤儿在下一次挂载后按「i < i_now 且无行」判已发布。D18 第 879 行「与第一个新根同一次发布」已经不许这个次序，没有检查盯着 | 不分辨臂 | 三 |
| K4 条款 | Q7 ① 是唯一抓得住撞号的：甲下 15 / 15、「改取号」下 18 / 18、模型 R 那一格也红；甲″ / 乙′ 全部路径 0 误报。它要两条限定：只数 fsid 与本池相同的单元（不然重 mkfs 留下的旧单元会误报），读不全时报不适用。第一轮甲的改写在甲下 15 个撞号状态上全绿，当不了「摘掉那道屏障必须红」的检查。Q7 ② 一个撞号都抓不到，它管的是「新号只有一份见证」（甲下 30 个不撞号的状态）；三块盘以上「从未被这次挂载打开的盘」上它误报 | 触发（写出误报与漏报的镜像） | 四 |
| K5 世代号 | G1 / G2 × 两种读法四种组合，在甲″ / 乙′ 下每一次后来有东西用到的取号都看得见（撞号 0）。择到的那一份唯一漏看的是回卷写（甲″ 153 / 729 条历史，漏看的号没有任何东西带着）。会撞号的是今天实现的写法（取号恒写 2、发布写 txg + 2），它既不是 G1 也不是 G2。G1 在滞后盘上覆写它最新的那一槽（甲″ 402 / 729 条历史），G2 0 次 | 触发（今天的写法 × 择到的那一份，写出历史）；G1 / G2 四种组合不触发 | 五 |

攻方的倾向（不是判决）：甲″ 留下，但它单独关不住撞号，要同一次写死三句——世代号取 G2；「全部超级块」取每盘两槽里全部自证过的槽，取号与 checker 用同一个读法；I-7.7 改成 Q7 ① ②（带 fsid、读不全、打开集合三条限定）并删掉回卷例外。D23 已定项 14 那一句按二改写。

## 一、现查清单

| 查什么 | 命令 | 结果（行号） |
|---|---|---|
| D23（journal 的角色与格式） 已定项 14 回退那一句 | `grep -n '已定项 14' .claude/kb/decisions/23-journal的角色与格式.md` | 索引行 691（「取新实例代号、写回退行，第一个新根的 checkpoint_txg = 根环里全部根记录 txg 的最大值 + 1，回退与第一个新根同一次发布、之前没有持久效果」）；正文小节 1204，显式例外在 1206（「回退与它的第一个新根同一次发布，之前没有持久效果，崩了就重做」）；1240 有一条 ⚠️ 写着这一句「按今天的条文不成立」（附录整行抄） |
| D18（块里携带什么信息） 第 879 行 | `sed -n '879p' .claude/kb/decisions/18-块里携带什么信息.md` | 7920 字节；与背景材料第 1227 行逐字节相同，也与第一轮报告附录那一份相同（`cmp`） |
| D22（单元原子性怎么合成） 已定项 16 | `grep -n '已定项 16' .claude/kb/decisions/22-单元原子性怎么合成.md` | 索引 232、正文 1003–1019；定案句 1005（「槽世代号从 1 起、每写一次 +1，下一次写的槽 = 世代号 mod 2，择槽取校验和过且世代号最大的」） |
| D16（发布语义） 已定项 1（抬 F） | `grep -n '^| 准入 ' .claude/kb/decisions/16-发布语义.md` | 小节从 358 行起；准入那一行在 377（附录整行抄） |
| D16（发布语义） 已定项 8（暖机） | `sed -n '205,207p' .claude/kb/decisions/16-发布语义.md` | 定案句 207 |
| first-txn-layout.md 第 389–393 行 | `sed -n '389,393p' .claude/kb/first-txn-layout.md` | 389 取号、390 空发布、391 普通发布、392 实例切换 / 管理员回退、393 抬 F 的空发布 |
| D28（挂载期承诺量） 已定项 3 | `sed -n '78,95p' .claude/kb/decisions/28-挂载期承诺量.md` | 切换预留是内存里的量；D18 第 879 行把「实例切换的预留拿得到」列在可写挂载的准入里、拿不到就只读，而「只读挂载不取号」 |
| 今天的取号 | `grep -n 'SUPERBLOCK_GENERATION_AT_INSTANCE_ACQUISITION\|fn superblock_generation_for_publish\|fn acquire_instance\|let slot_index' crates/singlefs-core/src/transaction.rs` | 49 常量 2；129 槽 = 世代号 mod 2；176–177 发布之后 = txg + 2；181–187 `acquire_instance` 写那个常量 |
| 今天择超级块 | `grep -n 'pub fn choose_superblock\|if one.slot_generation' crates/singlefs-core/src/recovery.rs` | 191 起；210 `one.slot_generation > zero.slot_generation`（相等取槽 0）；各盘只比 fsid 与设备数，返回第一块盘择到的那一份 |
| checker 的 I-7.7 | `grep -n '"I-7.7",' crates/singlefs-checker/src/walk.rs`；`grep -n 'pub fn chosen_superblocks' crates/singlefs-checker/src/image.rs` | walk.rs 546 各盘择到的实例代号全相等、583 都 ≥ 根环最大；image.rs 133 起每盘择世代号大的那一槽 |
| checker 的根与记录先比 fsid | 读 `crates/singlefs-checker/src/image.rs` 186、`crates/singlefs-core/src/recovery.rs` 的 `scan_journal` 注释 | 根：`check_root_slot(&bytes, &geometry.filesystem_identifier)`；记录：「fsid 不符的不算数」 |
| checker 有没有已发布谓词 | `grep -n 'I-1.2' crates/singlefs-checker/src/*.rs` | 0 行 |
| 层 0 把 I-7.7 钉成 2 | `grep -n 'expected_violated' crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs` | 107 |

背景材料附录与今天的 kb 逐字节核对（命令 `diff <(sed -n 'Np' kb 文件) <(sed -n 'Mp' 背景材料)`，9 处全 `SAME`）：D18:879 = 背景 1227；D22:1005 = 910；D23:691 = 800；D23:1206 = 815；D23:1240 = 849；D16:207 = 932；D16:289 = 980；invariants.md:53 = 1016；first-txn-layout.md:389–393 = 1107–1111。下面引这几行时只写「kb 文件:行」，整行就是背景材料附录那一份；背景材料没抄的（D16:377、fs-design.md:23）在文末「附录」整行抄。

## 二、K2：回退途中崩溃，超级块已带新号

### 条款

- D23:691 与 D23:1206（引文见一）：回退「取新实例代号」，「与第一个新根同一次发布、之前没有持久效果」。
- D18:879 可写挂载的顺序：「再取新代号 = max(**这次挂载独占打开成功的那个集合**里各超级块的代号, 根环里全部根记录的实例代号) + 1 并写进**那个集合**里的每一份超级块、**全或无**」，括注之后是「之后才动任何单元」。D23:695（已定项 16）把同一件事写成「并写进每一份超级块（一次超级块槽写，世代号 +1）之后才动单元」。

### 构造（甲″，管理员回退）

- 前态：第一个事务写完之后实例 1 非干净结束。两盘超级块槽 0 = (世代号 4, 实例 1)、槽 1 = (5, 1)；根环 (0, 0) (1, 1) (1, 2) (1, 3)；journal 里是实例 1 的记录。管理员选 R_old = (1, 1)。
- 写流（模型 K 按 first-txn-layout.md:392 的预想段序列建）：[acquire_superblock × 2 盘] 屏障 [rows_unit × 2 盘（回退行 (1, 1, 0)，flags bit0）+ rollback_cow_unit × 2 盘] 屏障 [rows_record × 2] 屏障 [rows_root FUA，txg 4，区域 1 → 盘 1] [rows_superblock × 2] 屏障 [暖机记录 × 2] 屏障 [warm_up_2_root，txg 5，盘 0] [超级块 × 2]，段序列 `[2, 4, 2, 1, 2, 2, 1, 2]`。
- 状态 R1 的持久写集合：{acquire_superblock@disk0, acquire_superblock@disk1}。两盘择到的超级块都是实例 2；根环、journal、单元里没有任何带 2 的东西；回退行、回退根都没落。
- `k-model.out` 第 27–35 行整行抄：

```text
PATH path=administrator_rollback arm=jia segments=[6, 2, 1, 2, 2, 1, 2] barriers=4 states=78 superblocks_unequal=32 collisions=15 new_number_in_superblock_without_new_root=51 new_orphans_published_at_next_mount=15 previous_orphans_published_at_next_mount=0 CHECKS(red/red_on_collision/red_without_collision/green_on_collision) literal=32/0/32/15 jia_rewrite=0/0/0/15 q7_one=15/15/0/0 q7_two=30/0/30/15
PATH_EXAMPLE path=administrator_rollback arm=jia first_collision=["rows_unit@disk0"]
PATH_EXAMPLE path=administrator_rollback arm=jia first_new_number_in_superblock_without_new_root=["acquire_superblock@disk0"]
PATH path=administrator_rollback arm=jia_double_prime segments=[2, 4, 2, 1, 2, 2, 1, 2] barriers=5 states=33 superblocks_unequal=2 collisions=0 new_number_in_superblock_without_new_root=21 new_orphans_published_at_next_mount=0 previous_orphans_published_at_next_mount=0 CHECKS(red/red_on_collision/red_without_collision/green_on_collision) literal=2/0/2/0 jia_rewrite=0/0/0/0 q7_one=0/0/0/0 q7_two=0/0/0/0
PATH_EXAMPLE path=administrator_rollback arm=jia_double_prime first_new_number_in_superblock_without_new_root=["acquire_superblock@disk0"]
PATH path=administrator_rollback arm=yi_prime segments=[1, 1, 4, 2, 1, 2, 2, 1, 2] barriers=6 states=32 superblocks_unequal=1 collisions=0 new_number_in_superblock_without_new_root=20 new_orphans_published_at_next_mount=0 previous_orphans_published_at_next_mount=0 CHECKS(red/red_on_collision/red_without_collision/green_on_collision) literal=1/0/1/0 jia_rewrite=0/0/0/0 q7_one=0/0/0/0 q7_two=0/0/0/0
PATH_EXAMPLE path=administrator_rollback arm=yi_prime first_new_number_in_superblock_without_new_root=["acquire_superblock@disk0"]
PATH path=administrator_rollback arm=number_only_in_own_publish segments=[4, 2, 1, 2, 2, 1, 2] barriers=4 states=30 superblocks_unequal=2 collisions=18 new_number_in_superblock_without_new_root=0 new_orphans_published_at_next_mount=18 previous_orphans_published_at_next_mount=0 CHECKS(red/red_on_collision/red_without_collision/green_on_collision) literal=3/0/3/18 jia_rewrite=3/0/3/18 q7_one=19/18/1/0 q7_two=2/0/2/18
PATH_EXAMPLE path=administrator_rollback arm=number_only_in_own_publish first_collision=["rows_unit@disk0"]
```

- 21 的来历（算术）：第 0 段只落一盘 2 个 + 第 1 段四个单元写的真子集 15 个（空子集就是 R1）+ 第 2 段记录的真子集 3 个 + 第 3 段根的空子集 1 个 = 21。乙′ 20：第 0 段在乙′ 下拆成两段，只剩「只落盘 0」一个。

### 按字面判：违反，而且三条臂都违反

R1 里两份超级块带实例 2，是回退那次发布之前落了盘的字节，按字面「之前没有持久效果」它违反。但它不是甲″ / 乙′ 带来的：甲下取号两写与回退那次发布的四个单元写同在第 0 段（第 27 行 `segments=[6, …]`），段内「两份超级块落了、四个单元一个没落」就是 R1，甲那一行 `new_number_in_superblock_without_new_root=51`。它也不是这一轮才有的：D18:879 与 D23:695 要求取号先写进每一份超级块、之后才动单元，回退「取新实例代号」走的是同一个取号。⇒ 字面冲突在今天的条文之间就有，与臂无关。
D23:1240 已经有一条 ⚠️ 说这一句「按今天的条文不成立」，理由是回退那次发布的 COW 单元可以复用最新根引用的单元（C314（回退可以复用被抛弃的根引用的单元），已由影子账收口）。那是同一句话的另一个毛病，与取号无关。

### 按语义判：甲″ / 乙′ 下没有可观测后果，甲与「改取号」下有

R1 这类状态（超级块带新号、新号的根一个没落）之后的下一次普通挂载，逐项：

1. 择根：新号的根没落，恢复照旧选实例 1 的最新根，与没发起回退时相同（「崩了就重做」）。
2. 取号：max(超级块 2, 根环 1) + 1 = 3，跳过 2。
3. 写行：D18:879「给 [所选根的实例, 新实例) 每个实例写 (i, **所选根的 checkpoint_txg**, …)」⇒ [1, 3)，被跳过的 2 得到一行 (2, 3, 0)；重做回退时它是中间实例，按「回退写 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)」得 (2, 0, 0)。
4. 谓词：回退那次发布留下的孤儿（写序实例 2、诞生 txg 4）对行 (2, 3, 0)：码 1「b ≤ T_pub ∨ n ≤ W」= 4 ≤ 3 ∨ 1 ≤ 0，假；码 2 / 3「b ≤ T_pub」，假 ⇒ 未发布，对。

模型 K 按 D18:879 的四支谓词逐状态算下一次挂载之后的判定（`next_mount_orphan_verdicts`）：甲″、乙′ 的回退路径 `new_orphans_published_at_next_mount=0`；甲 15，正是撞号的那 15 个——下一次取号又发 2，挂上的第一个根是 (2, 4)，孤儿落进「i == i_now ⇒ b ≤ 挂载根的 txg」，4 ≤ 4，已发布。
⇒ 甲″ / 乙′ 下「超级块带新号」只留下两件事：一个被跳过的号，实例表里多一行 (2, 3, 0) 或 (2, 0, 0)。选根、施加、谓词三样都与没发起回退时逐项相同。

### 让哪一边，代价各是什么

| 让法 | 改什么 | 代价 |
|---|---|---|
| 改那一句（攻方推荐） | D23:691 与 D23:1206 的「之前没有持久效果」改成：「回退行、回退那次发布的单元与回退根在那次发布之前都不生效——崩在它之前，下一次挂载选的根、施加的记录、已发布谓词的判定都与没发起回退时相同；取号写进超级块的新号是唯一留下的字节，它让下一次取号跳过一个号，被跳过的号按 D18（块里携带什么信息） 已定项 11 的写行规则得一行」 | 零字节、零屏障。要补一条检查：崩溃点重放在回退路径上逐状态比「下一次挂载的所选根与施加集合」与「没发起回退」相同；今天层 0 只有首次挂载那一条流，这条检查没有输入 |
| 改取号：新号只随回退那次发布写进超级块 | 取号不先写超级块；回退那次发布的单元、记录、根先带新号，发布末尾的超级块写才带 | 回退、恢复、切换三条路径各 18 个状态撞号、18 个状态孤儿在下一次挂载后判已发布（第 34 行；恢复第 16 行、切换第 25 行同数）；首次挂载路径 3 个撞号（w1 记录带实例 1 而超级块还是 0，第 7、8 行）；今天 I-7.7「≥ 根环」那半句在 3 个合法状态上红（根落了、发布末尾的超级块写没落，第 34 行 `literal=3/0/3/18`）。补救要让取号的 max 看得见单元，那是运行时遍历，fs-design.md:23 判「不许」 |

「改取号」在回退路径上比第一轮那个撞号更坏一层（推理，模型只量了谓词翻面）：翻成已发布的是回退那次发布的 COW 单元，内容按 R_old 那棵账写成；撞号之后继续走的却是没回退的那条时间线，扫描重建时这些单元会被当成它的已发布版本。

⚠️ 射程：模型 K 只建了一次回退、R_old = (1, 1)、两个单元写；影子账（C314）没进模型；下一次挂载假设一条记录都不施加（W = 0）；谓词翻面之后「扫描重建复活错版本」那一步没建模，与第一轮同。

## 三、K3：有没有第四条路径

### 路径结构：五条路径在甲″ / 乙′ 下撞号 0

`k-model.out` 各 `PATH` 行的 `collisions=`：

| 路径（取号之后本实例的第一次发布） | 甲 | 甲″ | 乙′ | 改取号 |
|---|---|---|---|---|
| mkfs 之后第一次可写挂载（暖机空发布） | 0 | 0 | 0 | 3 |
| 非干净结束之后的可写挂载（写行） | 15 | 0 | 0 | 18 |
| 实例切换（写行 + 重发的单元） | 15 | 0 | 0 | 18 |
| 管理员回退（回退行 + COW 单元） | 15 | 0 | 0 | 18 |
| 写行之前先推一次抬 F 的空发布（这一轮新加） | 0 | 0 | 0 | 3 |

提示点名的几种，逐个看：

- 抬 F 的空发布（D16:377 准入行「准入不够时先推空发布抬 F」）：本实例之内的发布，开头就是屏障。唯一会排到取号之后的次序是「恢复那次挂载在写行之前推它」，即第五条路径；开头照样是屏障，所以甲下也是 0。
- 准入不够时的回落：同一件事，本实例之内。
- 切换预留：D18:879 把「实例切换的预留拿得到」列进可写的准入，拿不到就只读，而「只读挂载不取号」。
- 别的恢复分支：扫描重建的产物只读、实例表不可读只读（D18:879「表不可读时：只读挂载」）；切换的探针写（D23:691「失败发生时先对目标设备的固定落点做一次探针写」）排在切换取号之前、带旧号；回卷写是超级块写，只在全或无失败时发生，那时单元还没发。
- 甲″ 按「本实例第一个非超级块写」下定义，不按路径列举，哪条路径照定义实现都关得住。它带一条实现义务：那道屏障完成之前，本实例的全部写者都不发单元、记录、根。第一版串行提交没有后台写者；打开并行提交或后台整理之后要重核。

### 甲″ 下撞号的可达状态：在今天的实现上，来源是世代号

`real-code.out` 整行抄：

```text
REAL second_acquisition=TodayAcquireInstance second_mount_superblock_writes=2 second_mount_unit_writes=16 slots(generation,instance)=[[(2, 2), (5, 1)], [(2, 2), (5, 1)]] recovery_choose_superblock=(5, 1) checker_chosen=[(5, 1), (5, 1)] root_instances={0, 1} second_mount_unit_instances={2}
REAL second_acquisition=TodayAcquireInstance next_acquisition by_recovery_choice=2 by_checker_chosen=2 by_every_slot=3 collides_with_units_or_roots by_recovery_choice=true by_checker_chosen=true by_every_slot=false checker_I-7.7=["Holds"]
REAL second_acquisition=GenerationSixFromEitherRule second_mount_superblock_writes=2 second_mount_unit_writes=16 slots(generation,instance)=[[(6, 2), (5, 1)], [(6, 2), (5, 1)]] recovery_choose_superblock=(6, 2) checker_chosen=[(6, 2), (6, 2)] root_instances={0, 1} second_mount_unit_instances={2}
REAL second_acquisition=GenerationSixFromEitherRule next_acquisition by_recovery_choice=3 by_checker_chosen=3 by_every_slot=3 collides_with_units_or_roots by_recovery_choice=false by_checker_chosen=false by_every_slot=false checker_I-7.7=["Holds"]
```

- 持久写集合：{a2@disk0（槽 0，世代号 2，实例 2）, a2@disk1（同）}，屏障完成，之后 16 个单元写（8 个单元 × 2 盘，写序实例 2）全落，下一道屏障没过。
- 每盘两槽：槽 0 (2, 2)、槽 1 (5, 1)。取号的世代号 2 比盘上已有的 5 小，又落在槽 0 上把 w6（世代号 4）盖掉了；择槽取世代号大的 ⇒ 两盘都择到 (5, 1)。
- 下一次取号：按恢复择到的那一份 max(1, 1) + 1 = 2，撞；按 checker 择到的，2，撞；按每盘两槽全部自证过的槽，3，不撞。checker 的 I-7.7 两半都成立（`["Holds"]`）。
- 对照（第 3、4 行）：同一条流只把取号那次的世代号换成 6（两盘择到的都是 5，G1 与 G2 在这一格都给 6），两盘择到 (6, 2)，三种读法都得 3，不撞。
- 层 0 看不到它：层 0 只有 mkfs 之后第一次挂载，那一次取号写的 2 正好大于 mkfs 写的 1。

它满足 K3 触发句的字面（「写出甲″ 下一个撞号的可达状态」），但不在「取号之后第一次写是单元」这一族里：屏障照样在单元之前完成，是那两次取号写被择槽规则藏住了。按 evidence-discipline「打中之后先问三句」：分不分辨臂——不分辨，乙′ 同样发生（`k-model.out` 第 81 行，120 / 512 条历史）；系统当时看不看得到——看得到，槽 0 就在盘上，只是择到的那一份不看它；满足的是哪一个分句——「撞号的可达状态」，而 K3 的问句问的是「取号之后先写单元」的路径。⇒ 不拿它判甲″ 出局，归到五的共同前提。

### 附带：写行之前先推抬 F 的空发布

`k-model.out` 第 39、41 行整行抄：

```text
PATH path=empty_publish_before_recovery_rows arm=jia_double_prime segments=[2, 2, 1, 6, 2, 1, 2] barriers=4 states=78 superblocks_unequal=2 collisions=0 new_number_in_superblock_without_new_root=6 new_orphans_published_at_next_mount=0 previous_orphans_published_at_next_mount=67 CHECKS(red/red_on_collision/red_without_collision/green_on_collision) literal=2/0/2/0 jia_rewrite=0/0/0/0 q7_one=0/0/0/0 q7_two=0/0/0/0
PATH_EXAMPLE path=empty_publish_before_recovery_rows arm=jia_double_prime first_previous_orphan_published=["acquire_superblock@disk0", "acquire_superblock@disk1", "f_raise_record@disk0", "f_raise_record@disk1", "f_raise_root"]
```

- 最小持久写集合：{acquire_superblock@disk0, acquire_superblock@disk1, f_raise_record@disk0, f_raise_record@disk1, f_raise_root}。新实例的根 (2, 4) 落了，写行那次发布没落。
- 下一次挂载选 (2, 4)，写行只给 [2, 3)；实例 1 从此没有行，它非干净结束时留下的孤儿（写序 (1, 1)、诞生 txg 4）落进「i < i_now 且无行 ⇒ 已发布」。67 = 63 + 3 + 1：抬 F 那个根落了之后、写行那个根落之前的全部状态。
- 四条臂都是 67，与取号那一步无关。D18:879「与第一个新根同一次发布」已经不许这个次序；D16:377 的准入行没说恢复那次挂载除外；first-txn-layout.md:393「抬 F 的空发布」一行只写「与暖机的空发布同型」，没写它与写行那次发布谁先。建议八那张表的「实例切换 / 管理员回退」一行与要补的「非干净结束之后的可写挂载」一行写明「写行那次发布是本实例的第一次发布，抬 F 只排在暖机之后」；层 0 覆盖这几条路径之后加一条计数：本实例的第一个根落了而旧实例无行的状态数必须为 0。

## 四、K4：三种 I-7.7 改写各判什么

四个检查：字面（invariants.md:53）、甲的改写（第一轮甲：「相等，或不等时根环与 journal 里没有带最大那个实例代号的根或记录」，保留「≥ 根环」那半句）、Q7 ①、Q7 ②。

### 全部路径枚举（A 段）

`CHECKS` 的四个数依次是：红 / 红且撞号 / 红而不撞号 / 撞号而绿。三条非首次挂载路径逐字相同（`k-model.out` 第 9、18、27 行是甲，第 12、21、30 行是甲″，第 14、23、32 行是乙′，第 16、25、34 行是改取号）。

| 路径 · 臂 | 字面 | 甲的改写 | Q7 ① | Q7 ② |
|---|---|---|---|---|
| 首次挂载 · 甲 / 甲″ | 2/0/2/0 | 0/0/0/0 | 0/0/0/0 | 0/0/0/0 |
| 首次挂载 · 乙′ | 1/0/1/0 | 0/0/0/0 | 0/0/0/0 | 0/0/0/0 |
| 非首次挂载三条 · 甲 | 32/0/32/15 | 0/0/0/15 | 15/15/0/0 | 30/0/30/15 |
| 非首次挂载三条 · 甲″ | 2/0/2/0 | 0/0/0/0 | 0/0/0/0 | 0/0/0/0 |
| 非首次挂载三条 · 乙′ | 1/0/1/0 | 0/0/0/0 | 0/0/0/0 | 0/0/0/0 |
| 非首次挂载三条 · 改取号 | 3/0/3/18 | 3/0/3/18 | 19/18/1/0 | 2/0/2/18 |

1. 甲″ / 乙′ 的可达集合上，甲的改写、Q7 ①、Q7 ② 一个都不红；字面只在「取号没做完」那 2 / 1 个状态上红，就是 Q2 记的那个误报。
2. 自证「摘掉甲″ 那道屏障必须由绿转红」（甲″ → 甲）：Q7 ① 0 → 15，红的 15 个恰是撞号的 15 个（红而不撞号 0、撞号而绿 0）；Q7 ② 0 → 30，全不撞号；甲的改写 0 → 0，15 个撞号状态上全绿。⇒ 甲的改写当不了这条自证的检查，Q7 ① 当得了。
3. Q7 ② 在甲下红的 30 个，是「新号已被单元带着、只有一块盘的超级块见证它」，例如 {acquire_superblock@disk0, rows_unit@disk1}：按两盘全部超级块取号不撞，那块盘的超级块一坏就撞。① 管撞号本身，② 管见证有没有两份，二者互不替代。
4. 「改取号」下 Q7 ① 有 1 个红而不撞号：根已落（根环带 2）、超级块还是 1，下一次取号 = max(1, 2) + 1 = 3，不撞。这个状态在「改取号」的设计里合法，① 在它上面误报；字面与甲的改写也红（「≥ 根环」那半句）。

### 点名镜像（C 段）

`k-model.out` 第 84–101 行整行抄：

```text
IMAGE name=jia_recovery_path_collision_rows_unit_disk0 superblocks=[1, 1] next_acquisition_collides=true literal=true jia_rewrite=true q7_one(fsid_filtered)=false q7_one(counting_foreign_units)=false q7_two(fsid_filtered)=true q7_two(counting_foreign_units)=true
IMAGE name=first_mount_acquisition_disk0_only superblocks=[1, 0] next_acquisition_collides=false literal=false jia_rewrite=true q7_one(fsid_filtered)=true q7_one(counting_foreign_units)=true q7_two(fsid_filtered)=true q7_two(counting_foreign_units)=true
IMAGE name=first_mount_acquisition_disk1_only superblocks=[0, 1] next_acquisition_collides=false literal=false jia_rewrite=true q7_one(fsid_filtered)=true q7_one(counting_foreign_units)=true q7_two(fsid_filtered)=true q7_two(counting_foreign_units)=true
IMAGE name=second_mount_acquisition_disk0_only superblocks=[2, 1] next_acquisition_collides=false literal=false jia_rewrite=true q7_one(fsid_filtered)=true q7_one(counting_foreign_units)=true q7_two(fsid_filtered)=true q7_two(counting_foreign_units)=true
IMAGE name=second_mount_acquisition_disk1_only superblocks=[1, 2] next_acquisition_collides=false literal=false jia_rewrite=true q7_one(fsid_filtered)=true q7_one(counting_foreign_units)=true q7_two(fsid_filtered)=true q7_two(counting_foreign_units)=true
IMAGE name=rollback_in_flight_superblocks_only_jia_double_prime superblocks=[2, 2] next_acquisition_collides=false literal=true jia_rewrite=true q7_one(fsid_filtered)=true q7_one(counting_foreign_units)=true q7_two(fsid_filtered)=true q7_two(counting_foreign_units)=true
IMAGE name=rollback_in_flight_rows_unit_disk0_jia_double_prime superblocks=[2, 2] next_acquisition_collides=false literal=true jia_rewrite=true q7_one(fsid_filtered)=true q7_one(counting_foreign_units)=true q7_two(fsid_filtered)=true q7_two(counting_foreign_units)=true
IMAGE name=rollback_number_only_in_own_publish_rows_unit_disk0 superblocks=[1, 1] next_acquisition_collides=true literal=true jia_rewrite=true q7_one(fsid_filtered)=false q7_one(counting_foreign_units)=false q7_two(fsid_filtered)=true q7_two(counting_foreign_units)=true
IMAGE name=foreign_disk1_same_instance_1 superblocks=[1, 1] next_acquisition_collides=false literal=true jia_rewrite=true q7_one(fsid_filtered)=true q7_one(counting_foreign_units)=true q7_two(fsid_filtered)=true q7_two(counting_foreign_units)=true
IMAGE name=foreign_disk1_higher_instance_5 superblocks=[1, 5] next_acquisition_collides=false literal=false jia_rewrite=true q7_one(fsid_filtered)=true q7_one(counting_foreign_units)=true q7_two(fsid_filtered)=true q7_two(counting_foreign_units)=false
IMAGE name=foreign_disk1_lower_instance_1_home_3 superblocks=[3, 1] next_acquisition_collides=false literal=false jia_rewrite=false q7_one(fsid_filtered)=true q7_one(counting_foreign_units)=true q7_two(fsid_filtered)=false q7_two(counting_foreign_units)=false
IMAGE name=remkfs_residue_old_filesystem_unit_instance_57 superblocks=[1, 1] next_acquisition_collides=false literal=true jia_rewrite=true q7_one(fsid_filtered)=true q7_one(counting_foreign_units)=false q7_two(fsid_filtered)=true q7_two(counting_foreign_units)=true
IMAGE name=roll_back_written_disk0_chosen_reading superblocks=[1, 1] next_acquisition_collides=false literal=true jia_rewrite=true q7_one(fsid_filtered)=true q7_one(counting_foreign_units)=true q7_two(fsid_filtered)=true q7_two(counting_foreign_units)=true
IMAGE name=roll_back_written_disk0_every_slot_reading superblocks=[2, 1] next_acquisition_collides=false literal=false jia_rewrite=true q7_one(fsid_filtered)=true q7_one(counting_foreign_units)=true q7_two(fsid_filtered)=true q7_two(counting_foreign_units)=true
IMAGE name=roll_back_written_disk0_disk1_landed_despite_error_chosen_reading superblocks=[1, 2] next_acquisition_collides=false literal=false jia_rewrite=true q7_one(fsid_filtered)=true q7_one(counting_foreign_units)=true q7_two(fsid_filtered)=true q7_two(counting_foreign_units)=true
IMAGE name=three_disks_disk2_never_opened_by_this_mount superblocks=[2, 2, 1] next_acquisition_collides=false literal=false jia_rewrite=false q7_one(fsid_filtered)=true q7_one(counting_foreign_units)=true q7_two(fsid_filtered)=false q7_two(counting_foreign_units)=false
IMAGE name=today_code_hidden_acquisition_units_durable_chosen_reading superblocks=[1, 1] next_acquisition_collides=true literal=true jia_rewrite=true q7_one(fsid_filtered)=false q7_one(counting_foreign_units)=false q7_two(fsid_filtered)=true q7_two(counting_foreign_units)=true
IMAGE name=today_code_hidden_acquisition_units_durable_every_slot_reading superblocks=[2, 2] next_acquisition_collides=false literal=true jia_rewrite=true q7_one(fsid_filtered)=true q7_one(counting_foreign_units)=true q7_two(fsid_filtered)=true q7_two(counting_foreign_units)=true
```

- 撞号镜像（甲，{rows_unit@disk0}：两盘超级块实例 1、单元带 2）：字面、甲的改写、Q7 ② 都成立，漏报；只有 Q7 ① 红。
- 取号没做完（首次挂载 [1, 0] / [0, 1]，第二次挂载 [2, 1] / [1, 2]）：字面红，误报；另三种都成立。
- 回退途中（甲″）的 R1 与 R1 + rows_unit@disk0：四种都成立，下一次取号不撞。「改取号」的 {rows_unit@disk0}：撞，只有 ① 红。
- 外盘替换：同号 1 四种都成立（第一轮判决四第 5 条已让 I-1.4 逐盘比 fsid，那才是它该归的地方）。外盘号 5：字面红；甲的改写成立（checker 的根与记录先比 fsid，外盘的看不见）；① 成立；② 只在把 fsid 不同的单元也数进去时红。外盘号 1、本池 3：字面、甲的改写、② 红，① 成立——红是对的，病因却是 I-1.4。
- 重 mkfs 残留（旧 fsid、实例 57 的单元还躺在单元区）：只有「把 fsid 不同的单元也数进去」的 ① 红，误报。⇒ ① 必须写成「fsid 与本池相同的单元」。
- 回卷（盘 1 的取号写报错、盘 0 回卷成旧代号且回卷落了）：择到的那一份 [1, 1] 四种都成立；全部槽 [2, 1] 与「盘 1 其实落了」[1, 2] 只有字面红。⇒ D18:879 那条「回卷期间……是 I-7.7 的一条显式例外」只有字面需要；换成 ① ② 或甲的改写，例外直接删掉，Q5 ③ 的合并就是删。
- 三块盘、盘 2 从未被这次挂载打开 [2, 2, 1]：字面、甲的改写、② 红，① 成立。D18:879 逐字「**从未被这次挂载独占打开的**，或缺号期间池里一次发布都没有的，不作废——它没有分叉的物理可能」⇒ 这是设计上合法的镜像，②、甲的改写、字面在它上面误报。今天的 I-7.7 用「独占打开成功的那个集合」挡这一格，可镜像里没记这次挂载打开了哪几块盘，checker 分不出。第一版两盘不可达（掉一块只能只读挂载）；D2（RAID 条带策略） 允许在线加盘之后可达，与 C120（分叉盘回归的判定与重同步） 同时到期。
- 今天代码的藏号状态（模型 R 那一格）：按择到的那一份 [1, 1] 撞，只有 ① 红；按全部槽 [2, 2] 不撞、四种都成立。⇒ ① 与取号必须用同一个读法（五第 2 条）。

### 条款形态（攻方的读法，不是判决）

I-7.7 改成两句，替掉「各超级块的实例代号相等」：

- ① fsid 与本池相同的每个根记录、journal 记录、单元写序的实例代号 ≤ 各盘超级块实例代号的最大者；「各盘超级块」与取号的「全部超级块」取同一个读法。任一根环区域、任一份 journal、任一单元头读不出时这一半报「不适用」，不算成立（第一轮判决二对甲提过的同一条要求）。
- ② 各盘超级块实例代号不等时，较大者不出现在任何根、记录、单元写序里；限定到这次挂载独占打开的那个集合，第一版两盘就是全部盘；三块盘以上之前要先让镜像记下那个集合，记不下就只能报「不适用」。
- 删掉 D18:879 的回卷例外：① ② 在回卷状态上本来就成立。
- 输入：① 的单元那一半要扫描方向逐个读单元头。checker 算 I-7.8 时已经在读码 2 头（第一轮报告五写的 walk.rs 475–503）；码 1 / 码 3 头今天没读，是新增的读，不改格式。

## 五、K5：世代号规则 × 「全部超级块」的读法

- G1：每次超级块写（取号、发布末尾的轮换、回卷）都写「两盘择到的世代号的最大者 + 1」，两盘同值，是实现今天「一次轮换给每块盘写同一个值」的推广。
- G2：每盘写「这块盘择到的世代号 + 1」，即 D22:1005 字面「槽世代号从 1 起、每写一次 +1，下一次写的槽 = 世代号 mod 2」。
- 两种读法：每盘两槽里全部自证过的槽；每盘择到的那一份，再跨盘取最大。另外量了今天真实存在的两种单份读法（只看世代号跨盘择一份，即 C322 那条自证的变异；第一块盘择到的那一份，即 `choose_superblock` 今天返回的）和一种规则（今天的实现：取号恒写 2、发布写 txg + 2）。
- 模型 K B 段：从第一个事务写完起（两盘槽 0 = (4, 1)、槽 1 = (5, 1)，根环 (0, 0) (1, 1) (1, 2) (1, 3)）连着三次挂载，每次按读法取号、按规则写，崩在下列结局之一：取号两写各落没落；两写都落之后单元、根、发布末尾的轮换各落到哪；「盘 1 的取号写报错、盘 0 回卷成旧代号且回卷落了」。甲″ 九种，乙′ 去掉「只落盘 1」八种，甲再加三种「单元先落、取号写没落全」共十二种。一条历史里只要有一次取号拿到盘上已有的号就记撞号；「漏看」= 某次取号时读法给的号比全部自证过的槽里最大的号小；「覆写最新槽」= 某次写落在那块盘此刻择到的那一槽上。

`k-model.out` 第 60–83 行整行抄（甲″、乙′）：

```text
GENERATION arm=jia_double_prime rule=PoolWideHighestPlusOne reading=EverySelfVerifiedSlot histories=729 collision=0 reading_missed_a_slot=0 newest_slot_overwrite=402 final_q7_one(chosen_reading)_false_red/missed=0/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=jia_double_prime rule=PoolWideHighestPlusOne reading=ChosenSlotPerDisk histories=729 collision=0 reading_missed_a_slot=153 newest_slot_overwrite=402 final_q7_one(chosen_reading)_false_red/missed=0/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=jia_double_prime rule=PoolWideHighestPlusOne reading=SingleSlotByGeneration histories=729 collision=0 reading_missed_a_slot=153 newest_slot_overwrite=402 final_q7_one(chosen_reading)_false_red/missed=0/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=jia_double_prime rule=PoolWideHighestPlusOne reading=FirstDiskChosen histories=729 collision=0 reading_missed_a_slot=288 newest_slot_overwrite=402 final_q7_one(chosen_reading)_false_red/missed=0/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=jia_double_prime rule=PerDiskPlusOne reading=EverySelfVerifiedSlot histories=729 collision=0 reading_missed_a_slot=0 newest_slot_overwrite=0 final_q7_one(chosen_reading)_false_red/missed=0/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=jia_double_prime rule=PerDiskPlusOne reading=ChosenSlotPerDisk histories=729 collision=0 reading_missed_a_slot=153 newest_slot_overwrite=0 final_q7_one(chosen_reading)_false_red/missed=0/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=jia_double_prime rule=PerDiskPlusOne reading=SingleSlotByGeneration histories=729 collision=0 reading_missed_a_slot=171 newest_slot_overwrite=0 final_q7_one(chosen_reading)_false_red/missed=0/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=jia_double_prime rule=PerDiskPlusOne reading=FirstDiskChosen histories=729 collision=0 reading_missed_a_slot=288 newest_slot_overwrite=0 final_q7_one(chosen_reading)_false_red/missed=0/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=jia_double_prime rule=TodayConstantTwoThenTxgPlusTwo reading=EverySelfVerifiedSlot histories=729 collision=0 reading_missed_a_slot=0 newest_slot_overwrite=391 final_q7_one(chosen_reading)_false_red/missed=304/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=jia_double_prime rule=TodayConstantTwoThenTxgPlusTwo reading=ChosenSlotPerDisk histories=729 collision=153 reading_missed_a_slot=513 newest_slot_overwrite=391 final_q7_one(chosen_reading)_false_red/missed=171/0 final_q7_one(every_slot_reading)_false_red/missed=0/133 first_collision=Some(([NothingDurable, BothThenUnits, NothingDurable], [[SlotContent { generation: 2, instance: 2 }, SlotContent { generation: 5, instance: 1 }], [SlotContent { generation: 2, instance: 2 }, SlotContent { generation: 5, instance: 1 }]]))
GENERATION arm=jia_double_prime rule=TodayConstantTwoThenTxgPlusTwo reading=SingleSlotByGeneration histories=729 collision=153 reading_missed_a_slot=513 newest_slot_overwrite=391 final_q7_one(chosen_reading)_false_red/missed=171/0 final_q7_one(every_slot_reading)_false_red/missed=0/133 first_collision=Some(([NothingDurable, BothThenUnits, NothingDurable], [[SlotContent { generation: 2, instance: 2 }, SlotContent { generation: 5, instance: 1 }], [SlotContent { generation: 2, instance: 2 }, SlotContent { generation: 5, instance: 1 }]]))
GENERATION arm=jia_double_prime rule=TodayConstantTwoThenTxgPlusTwo reading=FirstDiskChosen histories=729 collision=153 reading_missed_a_slot=603 newest_slot_overwrite=391 final_q7_one(chosen_reading)_false_red/missed=171/0 final_q7_one(every_slot_reading)_false_red/missed=0/133 first_collision=Some(([NothingDurable, BothThenUnits, NothingDurable], [[SlotContent { generation: 2, instance: 2 }, SlotContent { generation: 5, instance: 1 }], [SlotContent { generation: 2, instance: 2 }, SlotContent { generation: 5, instance: 1 }]]))
GENERATION arm=yi_prime rule=PoolWideHighestPlusOne reading=EverySelfVerifiedSlot histories=512 collision=0 reading_missed_a_slot=0 newest_slot_overwrite=248 final_q7_one(chosen_reading)_false_red/missed=0/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=yi_prime rule=PoolWideHighestPlusOne reading=ChosenSlotPerDisk histories=512 collision=0 reading_missed_a_slot=120 newest_slot_overwrite=248 final_q7_one(chosen_reading)_false_red/missed=0/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=yi_prime rule=PoolWideHighestPlusOne reading=SingleSlotByGeneration histories=512 collision=0 reading_missed_a_slot=120 newest_slot_overwrite=248 final_q7_one(chosen_reading)_false_red/missed=0/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=yi_prime rule=PoolWideHighestPlusOne reading=FirstDiskChosen histories=512 collision=0 reading_missed_a_slot=120 newest_slot_overwrite=248 final_q7_one(chosen_reading)_false_red/missed=0/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=yi_prime rule=PerDiskPlusOne reading=EverySelfVerifiedSlot histories=512 collision=0 reading_missed_a_slot=0 newest_slot_overwrite=0 final_q7_one(chosen_reading)_false_red/missed=0/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=yi_prime rule=PerDiskPlusOne reading=ChosenSlotPerDisk histories=512 collision=0 reading_missed_a_slot=120 newest_slot_overwrite=0 final_q7_one(chosen_reading)_false_red/missed=0/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=yi_prime rule=PerDiskPlusOne reading=SingleSlotByGeneration histories=512 collision=0 reading_missed_a_slot=120 newest_slot_overwrite=0 final_q7_one(chosen_reading)_false_red/missed=0/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=yi_prime rule=PerDiskPlusOne reading=FirstDiskChosen histories=512 collision=0 reading_missed_a_slot=120 newest_slot_overwrite=0 final_q7_one(chosen_reading)_false_red/missed=0/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=yi_prime rule=TodayConstantTwoThenTxgPlusTwo reading=EverySelfVerifiedSlot histories=512 collision=0 reading_missed_a_slot=0 newest_slot_overwrite=305 final_q7_one(chosen_reading)_false_red/missed=210/0 final_q7_one(every_slot_reading)_false_red/missed=0/0 first_collision=None
GENERATION arm=yi_prime rule=TodayConstantTwoThenTxgPlusTwo reading=ChosenSlotPerDisk histories=512 collision=120 reading_missed_a_slot=320 newest_slot_overwrite=305 final_q7_one(chosen_reading)_false_red/missed=113/0 final_q7_one(every_slot_reading)_false_red/missed=0/97 first_collision=Some(([NothingDurable, BothThenUnits, NothingDurable], [[SlotContent { generation: 2, instance: 2 }, SlotContent { generation: 5, instance: 1 }], [SlotContent { generation: 2, instance: 2 }, SlotContent { generation: 5, instance: 1 }]]))
GENERATION arm=yi_prime rule=TodayConstantTwoThenTxgPlusTwo reading=SingleSlotByGeneration histories=512 collision=120 reading_missed_a_slot=320 newest_slot_overwrite=305 final_q7_one(chosen_reading)_false_red/missed=113/0 final_q7_one(every_slot_reading)_false_red/missed=0/97 first_collision=Some(([NothingDurable, BothThenUnits, NothingDurable], [[SlotContent { generation: 2, instance: 2 }, SlotContent { generation: 5, instance: 1 }], [SlotContent { generation: 2, instance: 2 }, SlotContent { generation: 5, instance: 1 }]]))
GENERATION arm=yi_prime rule=TodayConstantTwoThenTxgPlusTwo reading=FirstDiskChosen histories=512 collision=120 reading_missed_a_slot=400 newest_slot_overwrite=305 final_q7_one(chosen_reading)_false_red/missed=113/0 final_q7_one(every_slot_reading)_false_red/missed=0/97 first_collision=Some(([NothingDurable, BothThenUnits, NothingDurable], [[SlotContent { generation: 2, instance: 2 }, SlotContent { generation: 5, instance: 1 }], [SlotContent { generation: 2, instance: 2 }, SlotContent { generation: 5, instance: 1 }]]))
```

| 规则 · 读法 | 甲″ 729 条：撞号 / 漏看 / 覆写最新槽 | 乙′ 512 条：撞号 / 漏看 / 覆写最新槽 |
|---|---|---|
| G1 · 全部槽 | 0 / 0 / 402 | 0 / 0 / 248 |
| G1 · 择到的 | 0 / 153 / 402 | 0 / 120 / 248 |
| G2 · 全部槽 | 0 / 0 / 0 | 0 / 0 / 0 |
| G2 · 择到的 | 0 / 153 / 0 | 0 / 120 / 0 |
| 今天 · 全部槽 | 0 / 0 / 391 | 0 / 0 / 305 |
| 今天 · 择到的 | 153 / 513 / 391 | 120 / 320 / 305 |
| 今天 · 第一块盘 | 153 / 603 / 391 | 120 / 400 / 305 |

1. **G1 / G2 × 两种读法四种组合，撞号都是 0**。择到的那一份漏看的甲″ 153 条恰是「第一次或第二次挂载走了回卷」的历史（算术：1 × 9 × 9 + 8 × 1 × 9 = 81 + 72），乙′ 120 = 1 × 8 × 8 + 7 × 1 × 8。写出一条（G2，择到的那一份）：
   - 第二次挂载取号 i = 2；盘 0 写 (6, 2) 落了，盘 1 的写报错；按 D18:879「先把已经写出的那几份回卷成旧代号」，盘 0 写 (7, 1)，落了。盘 0 两槽 (6, 2)、(7, 1)，择到 (7, 1)；盘 1 两槽 (4, 1)、(5, 1)，择到 (5, 1)。
   - 第三次挂载按择到的那一份：max(1, 1) + 1 = 2，实例 2 那次取号看不见，号 2 再发一次；按全部槽 max(2, 1) + 1 = 3。
   - 盘上没有任何根、记录、单元带 2（回卷只在全或无失败时发生，那时甲″ 的屏障没过、单元没发），再发一次无害。G1 下同一条历史写的也是 (6, 2) 与 (7, 1)。
   - ⚠️ 回卷若晚于本实例的单元就会撞：甲″ 下它发生不了；可要是实现在屏障报错之后不回卷就继续发单元，这一格就回来。这是甲″ 的一条实现义务。
2. **会撞号的是今天实现的写法 × 单份读法**：甲″ 153 / 729、乙′ 120 / 512。第 69 行的 first_collision（[NothingDurable, BothThenUnits, NothingDurable]，两盘都是槽 0 (2, 2)、槽 1 (5, 1)）就是模型 R 在真代码上那一格：第二次挂载的取号被 (5, 1) 藏住、单元带 2 落了，第三次挂载又发 2。换 G1 或 G2 都消失；只把读法换成全部槽也消失（第 68 行 0），但 checker 若照今天择到的那一份判 ①，304 条历史末态上误报（第 68 行 `final_q7_one(chosen_reading)_false_red/missed=304/0`）；反过来取号用择到的、checker 用全部槽，133 条历史末态撞号而 ① 绿（第 69 行 `final_q7_one(every_slot_reading)_false_red/missed=0/133`）。⇒ 取号与 checker 必须用同一个读法。
3. **G1 与 G2 的差别在覆写最新槽**：G1 在甲″ 402 / 729 条历史里有写落在滞后盘择到的那一槽上（例：盘 0 择到 6、盘 1 滞后在 5 ⇒ G1 写 7 ⇒ 槽 1，正是盘 1 那份 5 所在的槽）；G2 0。按 512 字节扇区撕开只能是整份旧或整份新（481 字节住第一个扇区，第一轮模型 A 核过），崩溃下无害；可一次写失败（介质错）会让滞后盘只剩更旧的那一槽，两槽轮换在那块盘上退化成一槽。G1 还与 D22:1005「每写一次 +1」不符（滞后盘一次跳 2）。
4. **只看世代号跨盘择一份**：G2 下两盘世代号不可比（第一轮那 1 / 16）。甲″ / 乙′ 下它不撞号（第 62、66、74、78 行），甲下多撞（G2：336 对全部槽的 276，`k-model.out` 第 54、52 行）。⇒ 取号不许用跨盘按世代号择出的那一份，它只配选两盘相同的字段（几何）。
5. **发布末尾的轮换写也要跟同一条规则**（推理，模型里 txg 单调，没量）：今天的 txg + 2 在恢复退一格选了较旧的根、之后发布的 txg 回落时，会写出比取号那次更小的世代号，那次轮换写被取号写藏住，它带的 tail 看不见。tail 只是扫描起点（D23:691「tail 只是扫描起点的优化」），不伤正确性，但择到的那一份从此不是最后写的那一份。

⇒ 攻方倾向 G2 + 全部自证过的槽：G2 两槽轮换在每块盘上都成立、与 D22:1005 字面一致；取「全部槽」是为了让取号不依赖「回卷写不会晚于单元」这条靠屏障才成立的次序；checker 的 I-7.7 / ① 用同一个读法。

## 六、按跑前写死的条款怎么判（攻方的读法）

- 失败条款（Q1 的转述与 D18:879、first-txn-layout.md:392 对不上）：不触发。两处与背景材料附录逐字节相同（一），Q1 引的两句都是这两行里的原样子串。
- K1 不归这条腿。顺带：A 段在甲下用第二套独立代码重做出第一轮那个撞号，三条非首次挂载路径各 15 个、最小持久写集合 {rows_unit@disk0}（`k-model.out` 第 10、19、28 行），与第一轮模型 B 同数。
- K2：不触发反向接受条款第 2 句，甲″ 与乙′ 在这一格同形（字面 21 / 20 个状态都违反，语义 0 / 0）。让步的是那一句，不是取号（二）。
- K3：路径那一问不触发。字面那一句由模型 R 的状态触发，但不分辨臂，按「打中不分辨臂 ⇒ 把共用前提另立一笔账先修」不拿它判甲″ 出局。若照字面判甲″ 出局，乙′ 在同一格同样中，两条候选一起出局，只剩参照臂丙；而丙那份超级块提示照样是每盘两槽、同一个择槽规则，病根不动。
- K4：触发。误报镜像：字面在取号没做完的四个镜像上；② 与甲的改写在「三块盘、盘 2 从未被这次挂载打开」上；① 在不按 fsid 过滤时的重 mkfs 残留上。漏报镜像：字面、甲的改写、② 在甲的撞号镜像与模型 R 的藏号镜像上。
- K5：触发，今天的写法 × 单份读法（历史在三与五第 2 条）；G1 / G2 四种组合不触发（五第 1 条）。
- K6 不归这条腿。顺带的数：屏障道数（录制流记法，相邻屏障记一道）非首次挂载路径甲 4 / 甲″ 5 / 乙′ 6，首次挂载 4 / 4 / 5（各 `PATH` 行 `barriers=`）；首次挂载那条流上甲″ 与甲的段序列相同，都是 `[2, 2, 1, 2, 2, 1, 2]`（第 1、3 行），与第一轮「层 0 状态数不变」一致。

⇒ 攻方的倾向：甲″ 留下，同一次写死三句——世代号取 G2；「全部超级块」= 每盘两槽里全部自证过的槽，取号与 checker 同一个读法；I-7.7 按四改成 ① ②、删掉回卷例外；D23 已定项 14 那一句按二改写。实现侧：`crates/singlefs-core/src/transaction.rs` 第 49 行的常量与 176–177 行的 txg + 2 随 G2 改；`crates/singlefs-core/src/recovery.rs` 的 `choose_superblock` 只给几何，不再当取号的输入。层 0 之上要一条多次挂载的历史：这一轮的两个洞（取号被择槽规则藏住、写行之前推抬 F）都在今天只有首次挂载的层 0 外面。

## 七、复跑

```bash
S=<临时目录>; rsync -a research/prompts/c322-r2-opus-model/ "$S/c322-r2-model/"
cd "$S/c322-r2-model/k_model" && CARGO_TARGET_DIR="$S/target-k" cargo run --release -q        # 产物 k-model.out（101 行）
cd "$S/c322-r2-model/real_code" && CARGO_TARGET_DIR="$S/target-real" cargo run --release -q   # 产物 real-code.out（4 行），按绝对路径依赖仓里四个 crate
```

本机这一轮两份产物就是这两条命令写出的：`k-model.out` sha256 640303f8…，`real-code.out` 3cae0465…，与存进模型目录的那两份相同。模型 R 读的是仓里今天的代码，代码一变它的数就可能变。

偏差如实记两条：
- 两次写文件超过了 150 行：`real_code/src/main.rs` 第一段 152 行；`k_model/src/main.rs` 从 `const JIA_DOUBLE_PRIME_OUTCOMES` 起到 `report_generation_rules` 末那一段 156 行。内容没有重写，照原样留着。
- 两个模型都没有变异表（不在 `research/e7-index-bench/src/bin/` 下，门禁 33 号不管），证据强度只靠对照：模型 R 撞号 true → false（只换取号的世代号）、模型 K 撞号 15 → 0（只换臂）、「改取号」一臂 18。模型 K 的检查与谓词是按条文重写的，没有和 checker 源码对拍。

## 八、什么现象会推翻这些结论

| 结论 | 推翻它的观测 |
|---|---|
| K2：甲″ / 乙′ 下 R1 这类状态没有可观测后果 | 下一次挂载不给被跳过的号写行（D18:879 的写行范围「[所选根的实例, 新实例)」改掉）；或回退路径第一次发布之前多出一个会被择中的根 |
| K2：「改取号」撞 18 | 取号的 max 不遍历也看得见单元，例如实例表单元先单独持久、并被一个会被择中的根指到 |
| K3：模型 R 的撞号 | 取号那次的世代号改成 G1 / G2，或取号按全部自证过的槽算（模型 R 第 3、4 行、模型 K 第 68 行就是这两个观测） |
| K3 附带：写行之前推抬 F 的 67 | 条款与实现都写明写行那次发布先于任何抬 F 的空发布，并有一条会红的计数盯着 |
| K4：① 在甲″ / 乙′ 下 0 误报 | 出现一个合法状态，某个 fsid 相同的单元写序带着比各盘超级块都大的号——例如三块盘以上错过取号的盘成了唯一读得出的超级块 |
| K5：G1 / G2 四种组合 0 撞号 | 回卷写晚于本实例的单元（屏障报错之后不回卷就继续发单元），或某次超级块写的世代号不大于那块盘上已有的槽 |

## 附录：正文引到、背景材料附录没抄或需要单独对照的 kb 原文（整行抄）

命令一律是 `sed -n 'Np' 文件`。正文里「」内的 kb 引文都是这些整行（或一里列出的、与背景材料附录逐字节相同的那几行）的原样子串。

**`.claude/kb/decisions/23-journal的角色与格式.md:1206`**

```markdown
**显式例外：管理员回退（2026-09-05，随 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3）。** 回退 = 一次恢复：管理员带外从回退候选集里选一个旧根 R_old——候选集 = 根环里按实例表判仍然有效、且 txg ≥ F_生效（2026-09-13，D16（发布语义） 已定项 1）的根，(i, T) 可选 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti（被抛弃时间线的根一条都选不中）；不施加 R_old 之后的任何记录；取新实例代号；写回退行 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)（r_old 诞生代号 ≤ T_old 的单元全部已发布、之后的一个都不算，不需要事务号上限）；第一个新根的 checkpoint_txg = 根环里全部根记录 txg 的最大值 + 1；defer 队列、分配器游标、全部记账统计量的现行值从 R_old 那棵账重新载入。**被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头——分配器多查环里每一个可读根的账（影子账；2026-09-13 用户定案，C314（回退可以复用被抛弃的根引用的单元） 取 F-A，E150（回退复用被抛弃的根引用的单元） 两格零违例、多隔离 2–3 个单元到被抛弃的最新根被轮转覆写为止；隔离量怎么进准入不等式见 C318（影子账隔离的单元没进准入不等式））。** 回退与它的第一个新根同一次发布，之前没有持久效果，崩了就重做；回退深度由候选集定（txg ≥ F_生效，2026-09-13 D16（发布语义） 已定项 1）：平时是整个根环，盘紧时可缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内。E104（扫描重建的现行版本判定）：不写回退行时全规则臂复活 3，新根取 T_old + 1 时压不过被抛弃的根。
```

**`.claude/kb/decisions/23-journal的角色与格式.md:1240`**

```markdown
⚠️ **回退例外里「之前没有持久效果」这一句按今天的条文不成立**：回退那次发布先写它自己的 COW 单元（D16（发布语义） 已定项 7 的顺序），而分配器从 R_old 的账重新载入、I-7.4（近 K 代块未被复用） 不护被抛弃的根，那些单元可以落在最新那个根引用的单元上；崩在记录之前，下一次恢复挂上那个根、读到被复用的单元（第二轮反推腿 Y-R，零故障）。回退确认之后同样可以复用，再让回退实例的根全读不出就挂上被抛弃的根。落 [checks-owed.md](../checks-owed.md) C314（回退可以复用被抛弃的根引用的单元）。
```

**`.claude/kb/decisions/16-发布语义.md:377`**

```markdown
| 准入 | 可分配 = min(可再分配 + 活元数据 − 保留池, `df`)；准入不够时先推空发布抬 F，一次准入最多 B = 4 + 2 k_tol 次发布（k_tol = 2 ⇒ 8），做满仍不够才报 ENOSPC |
```

**`.claude/rules/fs-design.md:23`**

```markdown
| **运行时决策路径**（分配、ENOSPC 准入、defer 窗口、生命周期判定） | **不许**，且代价不许随盘容量增长 | 不是慢，是**在被问到的那一刻没有答案**。准入控制要「进门前先算最坏情况」，而「释放空间这个操作本身不需要申请空间」也压在同一个数上 |
```
