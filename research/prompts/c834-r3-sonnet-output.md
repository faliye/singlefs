# C834（择到的系统配置实例代号随盘序变） 第三轮正推腿（Sonnet）报告（2026-10-15）

立场：正推。从已定条款与 `crates/` 的实现推出「指针槽读法」与「映射根变体」各该写成什么条款，核第二轮报的数，列相邻没有条款的格。格：T3、T4、T5（正文「三、格」）。T1、T2 归攻方腿，这一份不判。方案按 `crates/` 今天的实现来谈，冻结副本 `~/.local/share/singlefs/work/c834-r1/repo/`。

引文规则：kb 条款写 kb 文件自己的行号，代码写 `crates/` 下路径自己的行号，行号都是这一次 `grep -nF` 现取；冻结副本与主工作区这几份文件逐字相同（`mount.rs`、`recovery.rs`、`unit.rs`、`transaction.rs`、`pointer.rs`、`singlefs-format/src/lib.rs` 的 sha256 前 12 位两边相同，见 1.6），所以代码引文写仓内相对路径、两边都对得上。数是冻结副本拷贝上的数，不是入库装置上的数（`.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做`）。「读码，推的」= 读了代码没造盘面。「零轮」= 这一轮新冒出来、没人攻过。禁读清单里的文件没读。

模型目录：`research/prompts/c834-r3-sonnet-model/`（`tools/rerun.sh`、`tools/steps.sh` 等，说明见同目录 `README.md`）。
## 一、T3：指针槽读法写成一句可以落进 D23 已定项 14 的条款

### 1.1 条款原文（拟落点：`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14 的可写挂载预隔离那一段，换掉最后一句「现算不成不隔离、不拒」）

**条款甲（指针槽读法，一句）**：

> 较新可读根（按择根的序比所选那一版新、按最新那条根的实例表有效、读得出）那次发布的记录一条都读不出、账读不出、中央映射也现算不成时，读这条根的根记录里树表指针与中央映射树根指针各自两条位置条目指着的槽（共四槽；映射根指针是全零空指针的根只读树表那两槽），每个槽从槽首读，索引节点读 16384 字节、数据单元与打包记录单元读 32768 字节、三类逐个试解，四槽里只要有一个解出自证过（头校验和与载荷校验和过）、fsid 是本池的单元头，其写序实例代号（索引节点头的实例代号字段；数据单元与打包记录单元头写序里的实例代号）严格大于这条根自己根记录里的实例代号，就不隔离、不拒，否则（读不出、读回全零、解不出自证过的单元头、不是本池的、实例代号不大于这条根的）照「记录不齐」那一支办——重读一次账与映射（这四槽不随重读再读），现行账里有罩住这条根树表槽与映射根槽的凭据就不拒，没有就在取号之前拒可写挂载，拒因与 C393 那一拒分开报——不读系统配置里的实例代号。

**条款乙（映射根变体，一句）**：

> 同条款甲，只读中央映射树根指针的两条位置条目指着的两个槽、只按索引节点（16384 字节）解，映射根指针是全零空指针的根没有槽可读，直接照「记录不齐」那一支办。

「读哪几个槽、按什么长度读、比哪一个实例代号、读不出怎么判」四件事在甲里各占一处：四槽 / 16384 与 32768 / 单元头写序实例代号对根记录实例代号 / 否则走记录不齐那一支。实现对应：`research/prompts/c834-r2-opus-model/patch/apply_candidates.py` 的 `pointer_per_kind`，`research/prompts/c834-r2-sonnet-model/patch/apply_candidates.py` 的 `pointer_slots_fixed`；乙对应后者的 `mapping_root_only`。

**条款甲有三处写得比两腿原型明确**（都是零轮，没量）：① 「fsid 是本池的」，原型没做：两个补丁里 `filesystem_identifier` 只出现在系统配置读盘的辅助函数里，没有一处比单元头的 fsid（`grep -n 'filesystem_identifier\|fsid'` 两个补丁各 2 行，都在 `c834_instances_of_each_device`）；② 「这四槽不随重读再读」，原型就是这样（指针槽判在重读之前，重读只重做账与映射），条款把它写出来；③ 「映射根指针是全零空指针的根」，原型不判空指针，读的是设备 0 的槽 0（读码）。三处各自的理由与去向在 T5。

### 1.2 逐句核（kb 引文写那份文件自己的行号；代码引冻结副本，主工作区这六份 sha256 与冻结副本相同，`sha256sum` 对过，见 1.6）

条款甲拆成十个分句：1 适用范围；2 不读系统配置里的实例代号；3 读哪四个槽；4 按什么长度读；5 比哪一个实例代号；6 严格大于就不隔离、不拒；7 否则照记录不齐那一支办；8 fsid 是本池的；9 四槽不随重读再读；10 映射根指针是空指针的根没有映射根槽。每个分句对四处：已定项 14 第 392 行那一段（下称「392 段」）、Y2 收严（用户 2026-10-08 定「两形都修」，改法照攻方臂 2）、I-7.2、I-7.4。判：一致 / 冲突 / 规则没说。

**分句 1（适用范围）**：较新可读根、记录一条都读不出、账读不出、映射现算不成。

- 392 段：一致。「较新可读根那次发布的记录一条都读不出、账也读不出时，映射现算得出就隔离，现算不成不隔离、不拒（主 agent 2026-10-08 定）」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）；较新可读根的范围同段前文「再并上根环里比所选那一版新（按择根的序）、按最新那条根的实例表有效、读得出的根自己的账里的落点」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）。
- Y2：一致。改法落在同一个分支：`crates/singlefs-core/src/mount.rs:2078` 「None if journal_instance_in_the_system_configuration」，上一支 `crates/singlefs-core/src/mount.rs:2072` 「match placements_referenced_by_root_computed_from_the_mapping(」是映射现算。
- I-7.2、I-7.4：不涉及（只划范围）。

**分句 2（不读系统配置里的实例代号）**：条款甲、乙都不读。

- Y2：**冲突**。两边整行并排：
  - Y2 判决原文：`research/prompts/m3-batch37-43-code-r1-main-verification.md:25` 「只在系统配置里取过号的实例代号不比 R 新时」；
  - 实现：`crates/singlefs-core/src/mount.rs:2078` 「None if journal_instance_in_the_system_configuration」接 `crates/singlefs-core/src/mount.rs:2079` 「<= root.instance =>」；
  - 条款甲：「不读系统配置里的实例代号」。
  这是换判别子，不是 Y2 的补充：Y2 的拒与放由系统配置里的号定，C834 说的正是这个号随盘序变。用户 2026-10-08 定「两形都修」时改法是攻方臂 2（`research/prompts/m3-batch37-43-code-r1-opus-output.md:77` 「同臂 1，只在系统配置里取过号的实例代号不比 R 新时拒」），换成条款甲要用户另定，不能当成 Y2 已覆盖。
- 392 段：规则没说（392 段通篇没提系统配置里的号）；Y2 收严在 kb 里零命中：`grep -c 'Y2' .claude/kb/decisions/23-journal的角色与格式.md` 输出 0。
- I-7.2、I-7.4：不涉及。

**分句 3（读哪四个槽）**：树表指针、映射根指针各两条位置条目。

- I-7.2：一致。走读从根记录的树根指针出发（`crates/singlefs-checker/invariants/02-根环.md:9` 「S 中代号最大者，其 checkpoint 树根指针出发按 D4（校验和位置） 递归校验，能走完全部权威态」），树表与映射根是这条走读的头两步；这两处任一被别的单元占了，走读就走不完。
- I-7.4：一致。这两个槽是这条根「所引用的块」（`crates/singlefs-checker/invariants/02-根环.md:11` 「所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头」）。
- 392 段：规则没说为什么只读这两条而不读根记录里另外两条（实例表指针、分配记录树根指针）。根记录四条指针都有（`.claude/kb/decisions/22-单元原子性怎么合成.md:145` 「中央映射树的根住根记录、不进树表，与树表单元指针同形（自举豁免）；mkfs 与暖机写全 0」，实例表指针见同文件前一行，分配记录树根指针 `.claude/kb/decisions/22-单元原子性怎么合成.md:146` 「树表 0 条的一版上写行那次发布写的那一片分配记录节点」）。读码的理由：带文件的版本的发布每次都给树表与映射根发新指针（`crates/singlefs-core/src/transaction.rs:9129` 「tree_table: tree_table_pointer,」与 `crates/singlefs-core/src/transaction.rs:9133` 「mapping_root: mapping_pointer,」），实例表指针可以抄上一版（`crates/singlefs-core/src/transaction.rs:8272` 「InstanceTablePlan::Carry(pointer) => *pointer,」；攻方上一个代码轮的盘面里 R 就没重写实例表，`research/prompts/m3-batch37-43-code-r1-opus-output.md:91` 「ow 与 sw3 的 R 都没重写实例表」）；树表 0 条的版本树表与映射根两条都从上一版抄来（见 T5）。
- Y2：规则没说（Y2 不读槽）。

**分句 4（按什么长度读）**：索引节点 16384，数据单元与打包记录单元 32768，三类逐个试。

- 一致于单元格式：`crates/singlefs-core/src/unit.rs:391` 「if bytes.len() != usize::try_from(NODE_BYTES).expect("16384") {」，数据单元与打包记录单元的两处在 `crates/singlefs-core/src/unit.rs:464` 「if bytes.len() != usize::try_from(DATA_UNIT_BYTES).expect("32768") {」与 `crates/singlefs-core/src/unit.rs:530` 「if bytes.len() != usize::try_from(DATA_UNIT_BYTES).expect("32768") {」，都是 `DATA_UNIT_BYTES`（32768）；D18 已定项 7 的认领规则同口径：`.claude/kb/decisions/18-块里携带什么信息.md:186` 「就按整单元跳（码 1、码 3 两槽，码 2 一槽）」。
- 这一句是第一轮「丁上全拒」的病根：第一轮攻方原型把 32768 字节缓冲交给 `parse_index_node`，永远 `TooShort`（`crates/singlefs-core/src/unit.rs:391` 「if bytes.len() != usize::try_from(NODE_BYTES).expect("16384") {」），所以条款里长度按类别写，不写「读 32 KiB」。
- 392 段、Y2、I-7.2、I-7.4：规则没说长度（都不涉及读槽）。

**分句 5（比哪一个实例代号）**：槽上自证单元头的写序实例代号，对这条根自己根记录里的实例代号。

- 取的量：数据单元与打包记录单元头的写序是「写出它的那个事务的 (实例代号, 事务号)」（`.claude/kb/decisions/18-块里携带什么信息.md:178` 「写出它的那个事务的 (实例代号, 事务号)」），索引节点头只留实例代号（`.claude/kb/decisions/18-块里携带什么信息.md:183` 「写序 4（只实例代号）」）；比的对象是根记录的实例代号字段，它与 journal 实例代号是同一个计数（`.claude/kb/decisions/22-单元原子性怎么合成.md:138` 「择新在 checkpoint_txg 平局时按它高者赢」那一行）。
- 与 G3 的四种读法都不同（`research/prompts/c834-r1-main-verification.md:21` 「至少四个量：取号（全部盘全部槽最大）、checker（每盘最大里取最小）、本判（盘表第一块世代号最大那一槽）、取号前逐盘核（每盘世代号最大那一槽）」），是第五个量，**不读系统配置**。
- I-7.7 ①：一致，且给了它一个上界：`crates/singlefs-checker/invariants/02-根环.md:14` 「fsid 与本池相同的每个根记录、journal 记录、单元写序的实例代号 ≤ 各盘系统配置实例代号的最大者」——槽上单元的实例代号至多等于系统配置里取过的最大号，所以「严格大于这条根的实例代号」意味着有一次取过号**并且写过单元**的挂载在这条根之后；取号不写（烧号态，G1 的第二形）不会出现在单元头里。这正是 Y2 用系统配置里的号判不出来、条款甲判得出来的那一位。
- 392 段：一致，但**这是第三处比法**。392 段所在的已定项 14 注 2 已有两处：`.claude/kb/decisions/23-journal的角色与格式.md:370` 「与 jsn 同序；择根照旧 txg 为主、实例代号破平局」，句末写「两处比的东西不同，各守各的序」。条款甲再加一处（单元头实例代号对根实例代号，只比实例代号、不比 txg），落进去时要在注 2 补一句，否则读者以为只有两处。不比 txg 的理由：实例代号随取号单调、一次挂载取号之后才写单元，所以实例代号更大必然更晚；`checkpoint_txg` 在恢复落旧根后新实例的第一次发布也取「根环最大 + 1」以上（已定项 14 注 3，`.claude/kb/decisions/23-journal的角色与格式.md:371` 「checkpoint_txg 也一样」），所以没有「实例大而 txg 小」的新单元；一次挂载取号之后才写单元：`.claude/kb/decisions/23-journal的角色与格式.md:478` 「那道屏障完成之前本实例的全部写者都不发单元、记录、根」。
- Y2：一致于「比 R 新」的方向，不同在被比的数（分句 2）。I-7.2、I-7.4：不涉及。

**分句 6（严格大于就不隔离、不拒）**与 **分句 7（否则照记录不齐那一支办）**：

- 392 段：分句 6 与旧的一句**冲突**，整行并排——旧：`.claude/kb/decisions/23-journal的角色与格式.md:392` 「较新可读根那次发布的记录一条都读不出、账也读不出时，映射现算得出就隔离，现算不成不隔离、不拒（主 agent 2026-10-08 定）」；新：「现算不成」之后分两支，四槽里有更晚的单元才不隔离、不拒，没有的照记录不齐办（可以拒）。这是条款甲要改写的那一句，冲突是设计，不是遗漏。分句 7 与同段前一支一致：`.claude/kb/decisions/23-journal的角色与格式.md:392` 「不齐（缺序号或缺末条）就照被抛弃根的账读不出那一份办，从中央映射现算、只在内存里隔离，现算不成重读一次，仍不成而现行账里没有凭据就在取号之前拒可写」；凭据的定义没动：`.claude/kb/decisions/23-journal的角色与格式.md:390` 「凭据是内存里的现行账有一条代大于那条根的 txg、罩住它树表槽（与映射根槽）的记录，只看账、不读那一槽的内容，有它就不拒，没有它照拒」。
- Y2：分句 7 一致（臂 1 与臂 2 都落到「记录不齐」那一支，`research/prompts/m3-batch37-43-code-r1-opus-output.md:76` 「重读、凭据、仍不成拒 C_MAPPING」）；分句 6 **同结局、不同触发**：Y2 在系统配置里的号比 R 新就不交也不拒（实现 `crates/singlefs-core/src/mount.rs:2089` 「None => continue,」），条款甲在四槽里有更晚的单元才不交也不拒。两者的差就是 C834：号比 R 新不等于有一次挂载写过单元（取号两盘落完、一字未写，第一轮 G1 第二形），也随盘序变；单元头里的号比 R 新才是写过。
- I-7.2：一致，且条款甲的放过条件蕴含 I-7.2 在到站的镜像上已经红。论证（推的，由下面「蕴含」的量佐证）：放过要求树表或映射根的某个槽里是一个自证过、实例代号大于这条根的单元；这条根的走读要从这两个指针读单元、并按指针里记的校验和验（`crates/singlefs-core/src/recovery.rs:399` 「if crc32_castagnoli(&bytes) == location.unit_checksum {」），槽里换成了别的单元，这条走读就失配。这条根若是根环里代号最大的（没有更新的可读根），I-7.2 红在它身上；不是最大的，这一格由下面的 I-7.4 担，I-7.2 管的是最大那条。放过没有把一个绿的镜像放红，只是不再去护一条走不完的根。
- I-7.4：一致，同一个蕴含。这个槽是这条根「所引用的块」，被别的单元占了就是被重新分配（`crates/singlefs-checker/invariants/02-根环.md:11` 「所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头」）；放过时 I-7.4 在到站的镜像上已经红。
- 这个蕴含靠一条归纳（推的）：写了那一槽的挂载要么看不见这条根（根槽读不出；这是已登记的 C747，`.claude/kb/checks-owed.md:510` 「判别子观测不到，判决定这一形不修」），要么按 392 段看得见就隔离或拒（所以不会写它引用的块），要么走的是条款甲的放过那一支（它放过要求更早的挂载已经写过）。第一个写它的只能是第一种。

**分句 8（fsid 是本池的）**、**分句 9（四槽不随重读再读）**、**分句 10（映射根指针是空指针的根没有映射根槽）**：这三句是条款甲比两腿原型多写的，原型各自怎么做见 1.1，依据与挡不挡见 T5 第 7、3、5 条。对照：fsid 一句对 I-1.4 一致（`crates/singlefs-checker/invariants/01-块自描述.md:9` 「任一单元，其头记录的 fsid == 系统配置 fsid」）；另两分句 392 段、Y2、I-7.2、I-7.4 都没说，规则没说。

**条款乙逐句**：同甲的分句 1、2、6、7、8、9 逐句同判；分句 3 只剩映射根两槽（I-7.2 一致：映射根是走读第二步；I-7.4 一致：这是这条根引用的块）；分句 4 只剩索引节点 16384 一档（映射根是码 2 节点，`crates/singlefs-core/src/unit.rs:391` 的长度判）；分句 5 同；分句 10 变成「没有槽可读 ⇒ 永远走记录不齐那一支」。乙是甲的一半：凡乙放过的甲都放过（映射根槽里的证据是甲四槽里的两个），所以乙不会比甲多放过一个、可能多拒。**乙比甲多拒的形**（推的，读码；两腿都没造出来，见 T2）：树表槽里是更晚的单元、映射根槽里不是（读回全零、或还是 R 自己的、或被数据单元的后半截占了）。这样一个状态要求映射根槽被读故障藏住而树表槽不被藏——第二轮的读故障集只有 `none`、`record_only`、`y2`（树表+映射根+记录一起）、`y2_m0_record`（y2 再加三次挂载里 M0 那一条记录）、`tree_and_mapping`（树表+映射根一起），没有「只藏映射根与记录」这一档。

**两个变体在已量历史上的配对**：第二轮攻方用每个状态上记下的两个凭据（`mapping_root_newer` 与 `pointer_per_kind`）推出逐行同判；这一轮复跑用同一份记录重数一遍，并让辩方腿的原型对 `mapping_root_only` 与 `pointer_slots_fixed` 逐状态配对，数在 T4。

### 1.3 读盘开销（读码，推的）

两个变体都只在「记录一条都读不出、账读不出、映射现算不成」这一支里读槽，平时的挂载不多读一个字节。这一支里：条款甲每个槽先读 16384 字节试索引节点，解不开再读 32768 字节试数据单元与打包记录单元（`research/prompts/c834-r2-opus-model/patch/apply_candidates.py` 的 `c834_unit_at`），四槽至多 4 × (16384 + 32768) = 196608 字节；条款乙两个槽各 16384 字节，共 32768 字节。原型另读的诊断字段不算。

### 1.4 条款甲、乙与实现的对照点（落进 kb 之后实现要改的地方）

实现今天的那一判是 `crates/singlefs-core/src/mount.rs:2078` 「None if journal_instance_in_the_system_configuration」接 `crates/singlefs-core/src/mount.rs:2079` 「<= root.instance =>」；`crates/` 里没有指针槽读法也没有映射根变体：这一轮在仓根与冻结副本的 `crates/` 各跑一遍 `grep -rn 'pointer_slot\|mapping_root_only' crates/`，两处都是零输出、退出码 1。改的地方：这一支的守卫换成「四槽里没有更晚的单元」；函数参数 `crates/singlefs-core/src/mount.rs:2040` 「journal_instance_in_the_system_configuration: InstanceGeneration,」不再被这一支读，调用处 `crates/singlefs-core/src/mount.rs:3124` 「system_configuration.quantities.journal_instance,」的实参随之去掉；读槽用 `read_unit_via_locations` 那样的读盘入口，但**不核指针里的整单元校验和**（核了就读不到别的单元）。

### 1.5 推翻 T3 各结论的现象

- 「条款甲的放过条件蕴含到站镜像已红」：任何一个（状态，盘序），条款甲放过、而被判那次挂载之前池级 checker 在这个镜像上不红（`state_red=-`）。2.6 给出这一轮的量。
- 「第三处比法不会造出实例大而 txg 小的新单元」：一段合法历史里，某个实例取号之后写出的单元，所在发布的 `checkpoint_txg` 小于更早实例的某条根（已定项 14 注 3 的下界被破）。
- 「Y2 的触发与条款甲的触发不同」：某个状态上两者的拒放一致且系统配置里的号不随盘序变——此时 C834 不存在；第一轮 G1 已经量到号随盘序变。
- 「分句 3 为什么只读两条指针」：带文件的版本发布出来的根，树表或映射根指针是抄来的而不是新发的（`crates/singlefs-core/src/transaction.rs` 的 `publish_admitted` 里 `tree_table_pointer`、`mapping_pointer` 两处不是新发）。

### 1.6 主工作区与冻结副本的这几份文件相同

```
crates/singlefs-core/src/mount.rs main=975e77aff09e frozen=975e77aff09e same
crates/singlefs-core/src/recovery.rs main=006b1970dc81 frozen=006b1970dc81 same
crates/singlefs-core/src/unit.rs main=a1d760b12d07 frozen=a1d760b12d07 same
crates/singlefs-core/src/transaction.rs main=9793b8362d6b frozen=9793b8362d6b same
crates/singlefs-core/src/pointer.rs main=3756ec554fe2 frozen=3756ec554fe2 same
crates/singlefs-format/src/lib.rs main=358e68d0dc24 frozen=358e68d0dc24 same
```

放过那一支的量（条款甲的放过条件蕴含「到站镜像已红」，推的，下面是这一轮在两腿原型上数出来的）见 2.6。

## 二、T4：第二轮报的数能不能核

### 2.1 怎么复跑

- 冻结副本 `~/.local/share/singlefs/work/c834-r1/repo/`：在冻结副本里跑 `sha256sum -c <仓根>/research/prompts/c834-r3-snapshot/crates-sha256.txt 2>&1 | awk '{print $NF}' | sort | uniq -c`，原样输出一行 `95 OK`（不是 OK 的 0 份）。
- 两腿的补丁（`research/prompts/c834-r2-opus-model/patch/apply_candidates.py`、`research/prompts/c834-r2-sonnet-model/patch/apply_candidates.py`）各装进冻结副本的一份拷贝（`~/.local/share/singlefs/work/c834-r3-sonnet/opus/tree-candidates/` 与 `…/sonnet/tree-candidates/`），模型原型各拷一份，共用一个编译目录 `~/.local/share/singlefs/work/c834-r3-sonnet/target`；攻方原型编译 10 分 42 秒（含 librocksdb-sys），辩方原型 1 分 48 秒。每条重命令都是 `nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 24G bash scripts/hosts/capped.sh 31 <命令>`，并行 31 件一批，每件至多 600 个状态；前台 4 分钟的限被工具挪进后台的几次，等完成通知，没写轮询。
- 步骤本体在 `research/prompts/c834-r3-sonnet-model/tools/steps.sh`，`tools/rerun.sh` 读它；这一轮是分步跑的同一份代码（`RERUN_STEPS` 点步名），`rerun.sh` 整条没有从头跑过（见「没做什么」）。
- 对数的办法：这一轮重汇总的行与第二轮入库的汇总同一批历史的行逐行比（攻方汇总的例子序号取最小，确定，不去字段；辩方汇总的例子序号先到先得，两边都去掉）。`compare.py` 报「same」就是行数相同、逐行相同。

### 2.2 攻方腿模型：丙、三次挂载、丁、丁只藏记录、丁加 y2

重跑的 6 格共 168 件、退出码 168 个 0（`cat runs-opus/batch-*.log | grep -c 'rc=0'` 得 168，`grep -c 'rc=[1-9]'` 得 0）。汇总与第二轮同一批历史的行比：

```
name=r3_compare opus same rows_mine=574 rows_ref=574
```

覆盖行（状态数与「盘序 × 状态」行是否齐；去掉行首的 `name=c834_coverage`）：

```
history=root_units_and_record faults=none state_count=20516 state_order_rows=41032 complete=True
history=root_units_and_record faults=record_only state_count=20516 state_order_rows=41032 complete=True
history=root_units_and_record faults=y2 state_count=20516 state_order_rows=41032 complete=True
history=three_mount_tail faults=y2 state_count=8227 state_order_rows=16454 complete=True
history=three_mount_tail faults=y2_m0_record state_count=8227 state_order_rows=16454 complete=True
history=tree_table_and_record faults=tree_and_mapping state_count=20516 state_order_rows=41032 complete=True
```

条款甲读法（`pointer_per_kind`）在这 6 格走到那一判的结局，`cells.py` 压成一格一行（`not_reached` = 那一判之前就已经定了拒或做成，与候选无关；先写判:结局，再写状态数；两种盘序结局相同时只数 forward，不同的会另标，这里没有）：

```
root_units_and_record/none state_count=20516 complete=True | not_reached:mounted_r_units_kept:red=- 20512 ; pass:mounted_r_units_kept:red=- 4
root_units_and_record/record_only state_count=20516 complete=True | not_reached:mounted_r_units_kept:red=- 13608 ; not_reached:refused:PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion 4100 ; pass:mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4 2796 ; pass:mounted_r_units_kept:red=- 12
root_units_and_record/y2 state_count=20516 complete=True | not_reached:mounted_r_units_kept:red=- 16 ; not_reached:refused:PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion 4100 ; refuse:refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords 16400
three_mount_tail/y2 state_count=8227 complete=True | not_reached:refused:AccountOfAnAbandonedRoot 8223 ; refuse:refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords 4
three_mount_tail/y2_m0_record state_count=8227 complete=True | not_reached:refused:AccountOfAnAbandonedRoot 8223 ; refuse:refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords 4
tree_table_and_record/tree_and_mapping state_count=20516 complete=True | not_reached:mounted_r_units_kept:red=- 16396 ; not_reached:refused:AccountOfAnAbandonedRoot 4116 ; refuse:refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords 4
```

同一批状态上 K6（第二轮出局的那个）：

```
root_units_and_record/none state_count=20516 complete=True | not_reached:mounted_r_units_kept:red=- 20512 ; pass:mounted_r_units_kept:red=- 4
root_units_and_record/record_only state_count=20516 complete=True | not_reached:mounted_r_units_kept:red=- 13608 ; not_reached:refused:PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion 4100 ; pass:mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4 2796 ; pass:mounted_r_units_kept:red=- 12
root_units_and_record/y2 state_count=20516 complete=True | not_reached:mounted_r_units_kept:red=- 16 ; not_reached:refused:PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion 4100 ; refuse:refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords 16400
three_mount_tail/y2 state_count=8227 complete=True | not_reached:refused:AccountOfAnAbandonedRoot 8223 ; pass:mounted_r_units_kept:red=- 4
three_mount_tail/y2_m0_record state_count=8227 complete=True | not_reached:refused:AccountOfAnAbandonedRoot 8223 ; pass:mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4 3 ; refuse:refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords 1
tree_table_and_record/tree_and_mapping state_count=20516 complete=True | not_reached:mounted_r_units_kept:red=- 16396 ; not_reached:refused:AccountOfAnAbandonedRoot 4116 ; pass:mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4 3 ; refuse:refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords 1
```

逐格对第二轮报的数（攻方报告第六节那张表，第 341 至 349 行；每格的原文在下表第三列）：

| 格 | 状态数 | 第二轮报的 | 这一轮 | 对上 / 对不上 |
|---|---|---|---|---|
| 丙 `tree_table_and_record/tree_and_mapping` | 20516 | `research/prompts/c834-r2-opus-output.md:341` 「| `tree_table_and_record/tree_and_mapping`（丙） | 20516 | 打中（3 个） | 没打中 |」 | K6 放过 3 个并写掉 14 份（`pass:mounted_overwrote_r_units` 3），指针槽读法拒 4（那 3 个加 16395） | **对上** |
| 三次挂载 `three_mount_tail/y2` | 8227 | `research/prompts/c834-r2-opus-output.md:348` 「| `three_mount_tail/y2`（新） | 8227 | 没打中 | 没打中 |」 | K6 放过 4 个、一份没写掉；指针槽读法拒 4 | **对上** |
| 三次挂载 `three_mount_tail/y2_m0_record` | 8227 | `research/prompts/c834-r2-opus-output.md:349` 「| `three_mount_tail/y2_m0_record`（新） | 8227 | 打中（3 个） | 没打中 |」 | K6 放过 3 个并写掉 14 份、拒 1；指针槽读法拒 4 | **对上** |
| 丁 `root_units_and_record/none` | 20516 | `research/prompts/c834-r2-opus-output.md:342` 「| `root_units_and_record/none`（丁） | 20516 | 没打中 | 没打中（第一轮原型原样全拒 4 个，缺陷） |」 | 指针槽读法放过 4 个，一份没再写掉（`pass:mounted_r_units_kept` 4） | **对上** |
| 丁加 y2 `root_units_and_record/y2` | 20516 | `research/prompts/c834-r2-opus-output.md:346` 「| `root_units_and_record/y2`（形三，新） | 20516 | 没打中（拒 16388 个已坏的，可用性代价） | 同左 |」 | 拒 16400 个：其中盘上已红（R 已被写掉）的 16388（`tally.txt` 里 pointer_per_kind 拒·红 32776 行 ÷ 2），一份不少的 12 | **对上**（16388 对上；多出的 12 个是 R 的单元一份不少、拒的，第二轮表里记在「一份不少 24 行」） |
| 丁只藏记录 `root_units_and_record/record_only` | 20516 | `research/prompts/c834-r2-opus-output.md:347` 「| `root_units_and_record/record_only`（形二，新） | 20516 | 放过多写掉已坏 R 的剩余单元 2796 个，交主 agent | 同左 |」 | 放过 2808 个：写掉剩余单元 2796、没再写掉 12 | **对上**（2808 = 2796 + 12，第二轮主判决 `research/prompts/c834-r2-main-verification.md:21` 「2808 个状态 K6 与指针槽读法都放过，其中 2796 个把 R 剩下的单元也写掉」） |

九段（`nine`，固定盘面，攻方模型自带）：`diff research/prompts/c834-r2-opus-model/out/nine.txt <这一轮 nine.txt>` 退出码 0，两份都是 109 行、sha256 同为 `fc53e91b29a377309274755720f883743ecb26f6929b28b9882bcb9519f8eee2`（命令原样见 2.5）。

### 2.3 辩方腿模型：丙、丁、乙，指针槽读法与映射根变体

重跑的 280 件（映射根变体 140 件、指针槽读法 140 件：四个前缀 × 序号 0 到 count 全量），退出码：`cat runs-sonnet/batch-*.log | grep -c 'rc=0'` 得 280、`grep -c 'rc=[1-9]'` 得 0（第 280 件在第 10 批；第一次只跑了 9 批，汇总时 `compare-sonnet` 报 differs（少一件），补跑那一件后转 same，那次 differs 的输出没留，第二次的在草稿目录 `step-summarize.log`）。

指针槽读法（`pointer_slots_fixed`）的汇总与第二轮 `aggregate-all.txt` 里同一批前缀、同一候选的行逐行比（例子序号两边去掉）：

```
name=r3_compare sonnet same rows_mine=60 rows_ref=60
```

这一轮两个候选在四个前缀上走到那一判的结局，`cells.py` 压成一格一行（`not_reached` = 那一判之前已定；先写判:结局，再写状态数）：

```
== pointer_slots_fixed
clean/y2 state_count=20511 complete=True | not_reached:mounted_r_units_kept:red=- 4116 ; refuse:refused 16395
root_units_and_record/none state_count=20516 complete=True | not_reached:mounted_overwrote_r_units:red=- 4116 ; not_reached:mounted_overwrote_r_units:red=I-2.1+I-3.1+I-3.11+I-4.8+I-7.2+I-7.4 288 ; not_reached:mounted_overwrote_r_units:red=I-2.1+I-3.1+I-3.11+I-4.8+I-7.2+I-7.4+I-9.1 720 ; not_reached:mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4 8012 ; not_reached:mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4+I-9.1 4452 ; not_reached:mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.4 1940 ; not_reached:mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.4+I-9.1 972 ; not_reached:mounted_r_units_kept:red=- 12 ; pass:mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4 4
tree_table_and_record/tree_and_mapping state_count=20516 complete=True | not_reached:mounted_r_units_kept:red=- 16396 ; not_reached:refused 4116 ; refuse:refused 4
tree_table_and_record/y2 state_count=20516 complete=True | not_reached:refused 4116 ; refuse:refused 16400
== mapping_root_only
clean/y2 state_count=20511 complete=True | not_reached:mounted_r_units_kept:red=- 4116 ; refuse:refused 16395
root_units_and_record/none state_count=20516 complete=True | not_reached:mounted_overwrote_r_units:red=- 4116 ; not_reached:mounted_overwrote_r_units:red=I-2.1+I-3.1+I-3.11+I-4.8+I-7.2+I-7.4 288 ; not_reached:mounted_overwrote_r_units:red=I-2.1+I-3.1+I-3.11+I-4.8+I-7.2+I-7.4+I-9.1 720 ; not_reached:mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4 8012 ; not_reached:mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4+I-9.1 4452 ; not_reached:mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.4 1940 ; not_reached:mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.4+I-9.1 972 ; not_reached:mounted_r_units_kept:red=- 12 ; pass:mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4 4
tree_table_and_record/tree_and_mapping state_count=20516 complete=True | not_reached:mounted_r_units_kept:red=- 16396 ; not_reached:refused 4116 ; refuse:refused 4
tree_table_and_record/y2 state_count=20516 complete=True | not_reached:refused 4116 ; refuse:refused 16400
```

（`root_units_and_record/none` 那一行很长：丁的 20516 个状态按池级 checker 此刻红的不变量组合分成几类，每类一段 `not_reached:…`。）

逐格对第二轮报的数：

| 格 | 状态数 | 第二轮报的 | 这一轮 | 对上 / 对不上 |
|---|---|---|---|---|
| 丙 `tree_table_and_record/tree_and_mapping` | 20516 | `research/prompts/c834-r2-sonnet-output.md:129` 「丙拒（对）：`out/aggregate-all.txt:149`、`:152`，3 个状态 judge=refuse outcome=refused」 | 指针槽读法拒 4（其中 `number_taken_records_written_r_units_intact` 3、`number_taken_no_record_r_units_intact` 1）；映射根变体同 | **对上**（`compare-sonnet` same，60 行） |
| 丁 `root_units_and_record/none` | 20516 | `research/prompts/c834-r2-sonnet-output.md:129` 「丁 4 个状态放过（对）」 | 指针槽读法放过 4，映射根变体放过 4，结局都是 `mounted_overwrote_r_units`（辩方的口径是「写行根落盘前 R 的单元被写掉的份数」，含进来之前就已被写掉的 14 份；攻方口径是这次挂载新写掉的份数，所以同一 4 个状态攻方记 `kept`、辩方记 `overwrote`，是口径不同不是结局不同） | **对上**（数）；两腿的结局名不同，见括注 |
| 乙 `clean/y2` | 20511 | 第二轮 `aggregate-all.txt` 的 `number_taken_no_record_r_units_intact` 16384 个全拒（`research/prompts/c834-r2-sonnet-output.md:129` 「甲、乙拒」同一行） | 该类 16384 个全拒，两个候选一样；整个前缀走到那一判的 16395 个全拒，没有一个放过 | **对上** |
| 乙 `tree_table_and_record/y2` | 20516 | 同上，该类 16387 个 | 该类 16387 个全拒；整个前缀走到那一判的 16400 个全拒 | **对上** |
| 映射根变体头尾两段（序号 0–19、16000 起）| 四个前缀 | `research/prompts/c834-r2-sonnet-output.md:85` 「头尾两段（序号 0–20 与 16000 起，覆盖甲、乙的头、丙、丁、不覆盖乙的中间约 16000 个状态，那一类里 K6 与 pointer_slots_fixed 全量已是全拒）加回卷 17 个状态加九段：」，丙 `…:86` 「- 丙：`out/aggregate-mapping-tail.txt:43`（forward）、`:46`（reversed）：3 个状态 judge=refuse outcome=refused，对。」 | 从这一轮全量输出里挑出这两段，用第二轮的汇总脚本重汇总，与 `aggregate-mapping-tail.txt` 逐行比 | **对上**（`compare-tail` same，56 行；输出见 2.4） |

### 2.4 `mapping_root_only` 在乙那一类中间约 16000 个状态上补跑全量

第二轮只量了序号 0–19 与 16000 起（`research/prompts/c834-r2-sonnet-output.md:177` 「乙那一类里 `mapping_root_only` 只量了头尾两段（序号 0–20、16000 起），中间约 16000 个状态没跑」）。这一轮四个前缀都跑了序号 0 到 count 全量；中间一段（序号 20 到 15999，每个前缀 15980 个状态）单独汇总，原样（`middle.py`）：

```
candidate=mapping_root_only prefix=clean/y2 state_count=20511 state_order_rows=31960 complete=False
candidate=mapping_root_only prefix=tree_table_and_record/y2 state_count=20516 state_order_rows=31960 complete=False
prefix=clean/y2 class=number_taken_no_record_r_units_intact candidate=mapping_root_only order=forward judge=refuse outcome=refused states=15980
prefix=clean/y2 class=number_taken_no_record_r_units_intact candidate=mapping_root_only order=reversed judge=refuse outcome=refused states=15980
prefix=tree_table_and_record/y2 class=number_taken_no_record_r_units_intact candidate=mapping_root_only order=forward judge=refuse outcome=refused states=15980
prefix=tree_table_and_record/y2 class=number_taken_no_record_r_units_intact candidate=mapping_root_only order=reversed judge=refuse outcome=refused states=15980
```

读法：乙的两个前缀中间 15980 个状态（每种盘序）全部是 `number_taken_no_record_r_units_intact`（号取了、记录没写、R 的单元一份不少），映射根变体**全部拒**，一个放过都没有。这是该拒的类，拒对。映射根变体与指针槽读法在整个序号范围的逐状态配对（每个状态那一行同时记着两个凭据，`flags.py`）：

```
pair prefix=clean/y2 mapping_newer_index_node=false pointer_fixed=false rows=32790
pair prefix=root_units_and_record/none mapping_newer_index_node=true pointer_fixed=true rows=8
pair prefix=tree_table_and_record/tree_and_mapping mapping_newer_index_node=false pointer_fixed=false rows=8
pair prefix=tree_table_and_record/y2 mapping_newer_index_node=false pointer_fixed=false rows=32800
pair_total rows=65606 differ=0
```

走到那一判的 65606 个（状态，盘序）行里，两个凭据逐行相同，不同 0 行。并且由指针槽凭据推出的拒放，与第二轮 `aggregate-all.txt` 里 `pointer_slots_fixed` 的同一格逐格相同（不是只比总数，按前缀 × 盘序 × 放过或拒分别比；头尾加中间覆盖了整个序号范围的前缀才比，四个前缀都比了）：

```
pointer_vs_reference prefix=clean/y2 order=forward judge=pass mine=0 reference=0 same
pointer_vs_reference prefix=clean/y2 order=forward judge=refuse mine=16395 reference=16395 same
pointer_vs_reference prefix=clean/y2 order=reversed judge=pass mine=0 reference=0 same
pointer_vs_reference prefix=clean/y2 order=reversed judge=refuse mine=16395 reference=16395 same
pointer_vs_reference prefix=root_units_and_record/none order=forward judge=pass mine=4 reference=4 same
pointer_vs_reference prefix=root_units_and_record/none order=forward judge=refuse mine=0 reference=0 same
pointer_vs_reference prefix=root_units_and_record/none order=reversed judge=pass mine=4 reference=4 same
pointer_vs_reference prefix=root_units_and_record/none order=reversed judge=refuse mine=0 reference=0 same
pointer_vs_reference prefix=tree_table_and_record/tree_and_mapping order=forward judge=pass mine=0 reference=0 same
pointer_vs_reference prefix=tree_table_and_record/tree_and_mapping order=forward judge=refuse mine=4 reference=4 same
pointer_vs_reference prefix=tree_table_and_record/tree_and_mapping order=reversed judge=pass mine=0 reference=0 same
pointer_vs_reference prefix=tree_table_and_record/tree_and_mapping order=reversed judge=refuse mine=4 reference=4 same
pointer_vs_reference prefix=tree_table_and_record/y2 order=forward judge=pass mine=0 reference=0 same
pointer_vs_reference prefix=tree_table_and_record/y2 order=forward judge=refuse mine=16400 reference=16400 same
pointer_vs_reference prefix=tree_table_and_record/y2 order=reversed judge=pass mine=0 reference=0 same
pointer_vs_reference prefix=tree_table_and_record/y2 order=reversed judge=refuse mine=16400 reference=16400 same
```

**乙那一类中间的补跑判：对上，且不是两个变体的分辨点。** 映射根变体在乙中间 15980 × 2 个前缀 × 2 种盘序全拒，与指针槽读法逐状态同判，第二轮「乙那一类中间没跑」这一缺口补上。

### 2.5 命令与原样输出

```
$ cat runs-opus/batch-*.log | grep -c 'rc=0'; cat runs-opus/batch-*.log | grep -c 'rc=[1-9]'
168
0
$ cat runs-sonnet/batch-*.log | grep -c 'rc=0'; cat runs-sonnet/batch-*.log | grep -c 'rc=[1-9]'
280
0
$ diff research/prompts/c834-r2-opus-model/out/nine.txt runs-opus/nine.txt; echo "diff rc=$?"
diff rc=0
$ sha256sum research/prompts/c834-r2-opus-model/out/nine.txt runs-opus/nine.txt
fc53e91b29a377309274755720f883743ecb26f6929b28b9882bcb9519f8eee2  research/prompts/c834-r2-opus-model/out/nine.txt
fc53e91b29a377309274755720f883743ecb26f6929b28b9882bcb9519f8eee2  runs-opus/nine.txt
$ grep Finished build-opus-2.log build-sonnet.log
build-sonnet.log:    Finished `release` profile [optimized] target(s) in 1m 48s
build-opus-2.log:    Finished `release` profile [optimized] target(s) in 10m 42s
```

（`runs-opus/`、`runs-sonnet/` 是草稿目录 `~/.local/share/singlefs/work/c834-r3-sonnet/` 下这一轮逐状态原样输出的目录，约 90M 与 173M，太大不入库；`out/` 里的汇总入库在 `research/prompts/c834-r3-sonnet-model/out/`。）

跑前 `ps -o pid,args -u "$(id -u)"` 看负载：没有 `qemu-system`、`vm-bench`、`e152-file-system-benchmark`、`fio`（`ps … | grep -E 'qemu-system|vm-bench|e152-file-system|fio ' | grep -v grep | wc -l` 得 0）；别的会话在跑 `cargo test` 与一条 `c794_attack_probe`，负载均值开工时 40、跑攻方批的时候 94 到 137，辩方批的时候 37 到 65；所有重活 `nice -n 19`，没等锁。

### 2.6 放过的状态在被判那次挂载之前是不是已经红（`tally.py`）

条款甲放过的每个（状态，盘序）行，被判那次挂载之前池级 checker 在这个镜像上红不红（`state_red`）。攻方原型（指针槽读法 `pointer_per_kind`）：

```
opus credentials mapping_root_newer=false pointer_per_kind=false rows=32824
opus credentials mapping_root_newer=true pointer_per_kind=true rows=5624
opus candidate=pointer_per_kind judge=pass state_red=red rows=5624
opus candidate=pointer_per_kind judge=refuse state_red=green rows=48
opus candidate=pointer_per_kind judge=refuse state_red=red rows=32776
```

辩方原型（映射根变体与指针槽读法）：

```
sonnet candidate=mapping_root_only judge=pass state_red=red rows=8
sonnet candidate=mapping_root_only judge=refuse state_red=green rows=65598
sonnet candidate=pointer_slots_fixed judge=pass state_red=red rows=8
sonnet candidate=pointer_slots_fixed judge=refuse state_red=green rows=65598
```

读法：两个原型里放过的行，被判那次挂载之前**全部**已经红（`pass … state_red=red`；`state_red=green` 的放过行：条款甲 0 行）。这是 1.2 里「蕴含」的量：丁 4 个状态 + 丁只藏记录 2808 个状态 = 2812 个状态 = 5624 行（攻方）；辩方只跑了丁的 4 个状态，8 行。拒的行里有 `green`（R 的单元一份不少）：该拒的类（甲乙戊与取号只落一块盘）和 2.3 里列的几类可用性代价都在里面，拒在安全一侧。攻方汇总里两个凭据的四种组合：`mapping_root_newer=true pointer_per_kind=true` 5624 行、`false false` 32824 行，**没有一行一真一假**——这一轮量的历史上两个变体逐行同判（辩方 65606 行同样，见 2.4）。

## 三、T5：条款甲写定之后，相邻还没有条款的格

每条写：没条款的是什么 / 挡不挡 C834 / 依据。「挡」的标准：不写它，C834 登记行的验收句（断言结局：不写掉还在用的单元、该做成的做成）在条款甲写定后仍可能被破。**挡的 0 条**；下面 12 条都落在拒的一侧（安全，代价是可用性）或不碰指针槽读法的读数。

1. **回卷之后的号**：不挡。条款甲不读系统配置里的号，回卷、烧号、取号只落一块盘都不进判。已有条款管着号本身：回卷写发生在任何单元之前，所以被回卷的号不会出现在单元头里（`.claude/kb/decisions/23-journal的角色与格式.md:478` 「那道屏障完成之前本实例的全部写者都不发单元、记录、根」；回卷写的次序在 `.claude/kb/decisions/18-块里携带什么信息.md` 已定项 11「可写挂载的顺序」那一条，第 319 行）。没条款的是**择到的那一份系统配置本身**：`crates/singlefs-core/src/recovery.rs:1092` 「fn choose_system_configuration_naming_devices_without_a_mountable_slot」取盘表第一块交得出的那一份，D22 已定项 8 没写；第一轮判决已记「择系统配置取盘表第一块那一份没有条款，留下的两个候选都不读择到的那一份」（`research/prompts/c834-r1-main-verification.md:38`）。条款甲写定、`mount.rs` 里那一判不再收 `journal_instance_in_the_system_configuration` 之后，这个函数的读者里 C834 点名的那一个就没了，但函数本身仍随盘序变，别的读它的地方（F、几何）没逐个核 ⇒ 记欠账，不挡。
2. **映射根与树表落点不在同一块盘上**：不挡。第一版的布局是一个单元两盘各一份、两条位置条目同槽（`crates/singlefs-core/src/pointer.rs:43` 「是第一版的布局约定（一个单元整个落在一列上、两盘各一份）」），池第一版只有两盘（`crates/singlefs-checker/invariants/02-根环.md:14` 「第一版两盘就是全部盘」）。条款甲按每条位置条目自己的 (盘, 槽) 读，不要求四槽同盘或同槽号；两条位置条目槽号不等的指针按现行口径是「解不开」（`crates/singlefs-core/src/pointer.rs` 的 `slot_shared_by_both_location_entries`），条款甲对这类指针没写：两腿原型照读，读到什么算什么。三盘及以上时哪几块盘算「读得出」没有条款。
3. **可写挂载重读一次的范围**：不挡。「记录不齐」那一支的重读只重做账与映射（`crates/singlefs-core/src/mount.rs:1815` 「before_the_one_reread.before_rereading();」之后接着读账、读映射），读阶段的读缓存「只收读成且不是全零的落点」（`.claude/kb/decisions/23-journal的角色与格式.md:394` 「在这次挂载内有效的读缓存（只收读成且不是全零的落点）上把读阶段重做一遍」）。指针槽判在重读**之前**、不随重读再读：第一遍这四槽被瞬时读故障藏住、重读时露出更新的单元，原型照拒（安全一侧）。条款甲把「不随重读再读」写出来；要不要让它随重读是个没定的选择（放过更多，代价是重读时序又多一处语义），没挡。
4. **指针槽上落着数据单元或打包记录单元（整个或后半截）**：不挡。两腿没造出：`research/prompts/c834-r2-main-verification.md:21` 「多层映射树、映射根被数据单元覆盖没造出来（384 槽小盘只有一层映射树）」。读码：用户数据与提交内生块按 32768 对齐占 2 槽（`crates/singlefs-core/src/transaction.rs:2471` 「用户数据与 32768 对齐的提交内生块 2 槽，16 KiB 节点 1 槽」），数据单元只能从偶数槽起头；R 的指针槽是奇数槽而被数据单元盖住时，单元头在前一槽，槽首读不出头 ⇒ 没有证据 ⇒ 拒。攻方第二轮已写同一个读码判断：`research/prompts/c834-r2-opus-output.md:297` 「落在那一槽的是一个数据单元的后半截时两者都认不出，拒（可用性代价）」。条款甲只读槽首，不读前一槽；映射根变体连整个数据单元起头的那一支也不认（只认索引节点）。这是两个变体的一处读法差，归 T2。
5. **树表 0 条的版本（没写过文件的一版）**：不挡。零单元发布与树表 0 条版本上的写行发布，根记录的树表指针和映射根指针都从上一版抄来（`crates/singlefs-core/src/transaction.rs:1489` 「tree_table: previous_root.tree_table,」、`crates/singlefs-core/src/transaction.rs:1493` 「mapping_root: previous_root.mapping_root,」，写行那一支同样在 `crates/singlefs-core/src/transaction.rs:2010` 「tree_table: previous_root.tree_table,」与 `crates/singlefs-core/src/transaction.rs:2014` 「mapping_root: previous_root.mapping_root,」），映射根在 mkfs 与暖机根里是全零空指针（`.claude/kb/decisions/22-单元原子性怎么合成.md:145` 「中央映射树的根住根记录、不进树表，与树表单元指针同形（自举豁免）；mkfs 与暖机写全 0」）。这类根的四槽里放着上一版（或 mkfs）的单元，实例代号只会小于这条根，**条款甲对它们永远读不到证据**；这类根自己新写的是实例表那一片与分配记录树根那一片（`.claude/kb/decisions/22-单元原子性怎么合成.md:146` 「树表 0 条的一版上写行那次发布写的那一片分配记录节点」），条款甲不读。拒在安全一侧；走到这一支还要树表单元读不出——而树表单元与所选那一版共用，所选那一版读得出（推的，读码，没造盘面）。映射根变体对映射根指针是空指针的根一个槽都没有。
6. **读回全零之外的坏字节（位翻转、撕裂写落在指针槽上）**：不挡。本仓读故障模型只有读回全零，没造（`research/prompts/c834-r2-opus-output.md:358` 「读故障之外的坏字节（位翻转、撕裂写落在指针槽上）」）。读码：单元头自证（头校验和、载荷校验和），坏字节解不出就是「没有证据」，拒；翻出一个自证过、实例代号更大的单元要同时碰对两处校验和（推的）。
7. **fsid 与自备份镜像**：不挡，但条款甲和原型有一处不同（见 1.1 ①）。I-1.4 要求任一单元头的 fsid 等于系统配置 fsid（`crates/singlefs-checker/invariants/01-块自描述.md:9` 「任一单元，其头记录的 fsid == 系统配置 fsid」），D18 已定项 7 认领规则承认同卷自备份镜像 fsid 相同挡不住（`.claude/kb/decisions/18-块里携带什么信息.md:186` 「就按整单元跳（码 1、码 3 两槽，码 2 一槽）」）。条款甲只读槽首，槽首只会是外层单元自己的头，镜像里的单元头落在载荷内部、不在槽首（推的，读码，没造）；写进「fsid 是本池的」几乎没有代价，两腿原型没实现，零轮。
8. **被比的是根记录的实例代号还是指针自带的实例代号**：不挡。指针也带一个实例代号字段（`crates/singlefs-core/src/pointer.rs:284` 「pub instance: InstanceGeneration,」），位置条目还带整单元校验和（`crates/singlefs-core/src/recovery.rs:399` 「if crc32_castagnoli(&bytes) == location.unit_checksum {」）。条款甲取根记录的实例代号（两腿原型就是这样）；抄来的指针（上一条）里指针自带的实例代号小于根的，两种取法在那一类根上结局可以不同，带文件的版本新发的指针上两者相等。「槽上是自证过的单元而整单元校验和对不上指针里记的」这个证据更直接，但它分不开「被更晚的挂载盖掉」与「这条根的写没写全」，也不进两腿原型，没量。
9. **R 有多条记录、记录读得出一部分**：不挡。那是「记录不齐」那一支的范围，392 段早已有条款（`.claude/kb/decisions/23-journal的角色与格式.md:392` 「不齐（缺序号或缺末条）就照被抛弃根的账读不出那一份办，从中央映射现算、只在内存里隔离，现算不成重读一次，仍不成而现行账里没有凭据就在取号之前拒可写」）；条款甲只管一条都读不出的那一支。归 T1。
10. **多层中央映射树**：不挡。映射根读得出而下层节点读不出时映射现算不成，但映射根槽里还是 R 自己的索引节点（实例代号等于 R 的），条款甲读到「不大于」，拒，不会多放过（攻方第二轮同判断，`research/prompts/c834-r2-opus-output.md` 形四；推的）。映射根变体同。归 T1。
11. **Y2、Y4 与 C1 复算判据的写回**：不挡条款甲，但同一次写回要一起做。三样的判决都写明要写进 D23 已定项 14：`research/prompts/m3-batch37-43-code-r1-main-verification.md:46` 「D23 已定项 14 写进两条收严，以及 C1 的镜像复算判据（顺树表指针找账根，只判必要条件）」），今天都不在 D23 里：Y2 零命中（`grep -c 'Y2' .claude/kb/decisions/23-journal的角色与格式.md` 输出 0）；Y4 的改法 `F2_NO_SKIP` 零命中（同一文件 `grep -c 'F2_NO_SKIP'` 输出 0），实现里有：`crates/singlefs-core/src/recovery.rs:5655` 「臂 OPUS_ARM_F2_NO_SKIP=1，用户 2026-10-08 定」；C1 的复算判据在 `crates/singlefs-harness/src/mount_transparency.rs:2718`：「这一格只判必要条件：实现拒之前还要「从中央映射现算不成」，那一半崩溃镜像上不复算」，D23 没有对应一句。第一轮 G5 只点了 Y2。条款甲落地后挂载透明那条复算判据仍是必要条件（条款甲只让拒的集合变成「没有更晚的单元」的子集），不用改，但 D23 要有它。
12. **挂载透明对放过的一支不判**：不挡。挂载透明只复算拒合不合法（`C_MAPPING` 那一格从 `crates/singlefs-harness/src/mount_transparency.rs:2707` 起），不复算放过合不合法；条款甲的放过条件（四槽里有更晚的单元）没有对应的镜像判官（读码，推的：这个文件里 `C_MAPPING` 只出现在拒因复算里，没有放过侧的函数）。放过写掉还在用的单元由池级 checker 判（I-7.2、I-7.4），所以不是漏判，只是没有一条「放过要有证据」的断言；记欠账。

## 四、两个变体的可用性代价与读盘开销（本轮量过的状态上；出口要求的数）

两个变体在这一轮量过的全部状态上逐行同判，所以代价是同一份。拒了而放过本来不丢单元、或盘本来就红的形：

| 形 | 状态数 | 量法 / 出处 |
|---|---|---|
| `later_mount_records_landed_r_units_intact`（取号那次挂载择了 R、记录落了根没落，之后 R 的树表、映射根、记录读回全零） | 九段里 1 段，两序 | 攻方 `nine.txt` 该段 `judge=refuse`，沿用第二轮的口径（`research/prompts/c834-r2-main-verification.md:31` 「`later_mount_records_landed_r_units_intact` 那一形与 clean 前缀里 M1 建在 R 上、R 一份不少的 3 个状态拒（放过也不丢单元）」） |
| 前缀里 M1 建在 R 上、R 的单元一份不少，被判那次挂载拒 | `clean/y2` 3 个 + `tree_table_and_record/y2` 5 个 | 辩方汇总 `number_taken_records_written_r_units_intact` 两个候选都拒（2.3 的原样输出）；「放过也不丢单元」这半句引第二轮的 K3 列，这一轮没跑 K3 |
| 丁加 y2：R 的单元已被写掉，读故障又藏住指针槽，一律拒 | 16388 个（盘本来就红）+ 12 个（R 的单元一份不少） | 攻方 2.2 表第 5 行 |
| 读故障消失之前可写挂载都拒，盘不变 | 同上各形 | `research/prompts/c834-r2-main-verification.md:31` 「读故障消失之前可写挂载都拒，盘不变」，第二轮口径沿用 |

读盘开销：见 1.3，只在三样都读不出的那一支里读，条款甲至多 196608 字节、条款乙 32768 字节（读码，推的）。

## 五、新冒出来的零轮形态（第三轮之后不再开轮，写进判决的交用户表）

1. 零轮：条款甲比原型多的 fsid 一句（T5 第 7 条）。
2. 零轮：条款甲把「四槽不随重读再读」写成条款（T5 第 3 条）。
3. 零轮：树表 0 条的版本（零单元发布与树表 0 条版本上的写行发布）的树表与映射根指针是从上一版抄来的，条款甲对它们永远读不到证据，映射根变体对映射根全零空指针的根一个槽都没有（T5 第 5 条；读码，没造盘面）。
4. 零轮：条款甲是实例代号的第三处比法，已定项 14 注 2 要补一句（1.2 分句 5）。
5. 零轮：数据单元从偶数槽起头，R 的指针槽是奇数槽而被数据单元盖住时槽首读不出头（T5 第 4 条）。
6. 零轮（推的，归 T2）：能把两个变体分开的形需要「树表槽里是更晚的单元、映射根槽不是」。第二轮的读故障集（`none`、`record_only`、`y2`、`y2_m0_record`、`tree_and_mapping`）里没有「只藏映射根与记录、不藏树表」这一档，这一轮量到的 65606 + 38448 个行上两个凭据一行也没有分开（2.4、2.6）；这一形没造。
7. 零轮：槽上自证单元整单元校验和与指针里记的对不上，作为比实例代号更直接的证据（T5 第 8 条）。
8. 零轮：放过一支没有镜像判官（T5 第 12 条）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T3 条款甲（指针槽读法） | 写得出，一句 | 读哪四槽、按类别取 16384 或 32768、比单元头写序实例代号对根记录实例代号、读不出照记录不齐办，四件事各占一处（1.1） |
| T3 条款乙（映射根变体） | 写得出，一句 | 甲的一半；凡乙放过甲都放过，乙可能多拒（1.2 条款乙逐句） |
| T3 × 392 段 | 冲突（设计） | 旧句「现算不成不隔离、不拒」要被条款甲改写（1.2 分句 6），其余分句一致 |
| T3 × Y2 收严 | 冲突（换判别子） | Y2 读系统配置里的号（`crates/singlefs-core/src/mount.rs:2078` 「None if journal_instance_in_the_system_configuration」），条款甲不读；用户 2026-10-08 定的是臂 2，换条款甲要用户另定（1.2 分句 2） |
| T3 × I-7.2、I-7.4 | 一致 | 放过条件蕴含到站镜像已红：攻方原型 5624 行、辩方原型 8 行放过行全红，绿的 0 行（2.6） |
| T3 与两腿原型的出入 | 三处 | fsid、不随重读再读、映射根空指针（1.1），都零轮 |
| T4 攻方腿 6 格 | 对上 | 丙、三次挂载两段、丁、丁只藏记录、丁加 y2，574 行逐行同，九段 sha256 同（2.2、2.5） |
| T4 辩方腿 丙、丁、乙 | 对上 | 指针槽读法 60 行同；映射根变体头尾两段 56 行同（2.3、2.4） |
| T4 `mapping_root_only` 乙中间补跑 | 对上，全拒 | 15980 × 2 个前缀 × 2 种盘序全拒，与指针槽读法逐状态同判，65606 行不同 0（2.4） |
| T5 相邻没有条款的格 | 12 条 | 挡 C834 的 0 条；落在拒的一侧或不碰指针槽读法（三 T5） |
| 两个变体可分性 | 这一轮量过的状态上分不开 | 没造出分得开的历史（归 T2）；预言的分点见五第 6 条 |

## 没做什么

- 不判别，不替主 agent 采纳或出判决；T1、T2（攻方格）没做，两个变体站得住或站不住不在这一份里判。
- 没在入库装置上跑：数都是冻结副本拷贝上的数；条款落 D23 之前这些数不进 kb（`.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做`）。
- `tools/rerun.sh` 整条没有从头跑过一遍：第二轮两腿的补丁、原型、驱动脚本是按步分批跑的同一份代码（`steps.sh`），`rerun.sh` 只验了 `prepare` 步和用法拒绝（`--frozen` 写成不存在的目录时退 2），`summarize` 步用入库的模型目录跑过一遍、逐项 same。整条的命令在下面「要主 agent 跑的命令」。
- 没复跑的：辩方腿九段（`cargo test … c834_order_of_devices_decides_the_newer_root_judgement` 七个候选）、入库 13 份 harness 用例、攻方腿的 `acquisition_rolled_back/y2`（戊 17 个状态）、攻方原型在 `clean/y2` 与 `tree_table_and_record/y2` 两个前缀上的甲乙格；这些不在 T4 点名的格里。
- 攻方腿九段 `nine` 复跑了，K3 与 K6 的列也在同一份输出里，但 K3 单独的「放过也不丢单元」这一半没有逐格核。
- T5 里标「读码，推的」的几条没造盘面：树表 0 条版本、奇数槽被数据单元盖住、位翻转落在指针槽、fsid 与镜像、同一实例连续发布；造它们归攻方腿（T1）。
- 没起 GPU，没跑 checker 档测试和门禁阶段，没跑 `--selftest`（`rerun.sh` 的 `--selftest` 是模板自带的，我的 `rerun_steps` 读 `tools/steps.sh`，模板的自证把脚本拷进空目录里跑、找不到 `steps.sh`，会红；自证归提交前）。
- 引文里没有背景材料的行号；没读禁读清单里的文件。

## 要主 agent 跑的命令

整条复跑（本机忙时两三个小时，空时约一个小时；一条 cargo，24G 内存包装，线程 31）：

```
bash scripts/hosts/run-with-memory-cap.sh 24G bash research/prompts/c834-r3-sonnet-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/c834-r1/repo \
  --work <新的草稿目录> --model research/prompts/c834-r3-sonnet-model
```

工作目录：仓根（命令里的路径是相对仓根）。环境变量：`RERUN_THREADS`（默认 31）、`RERUN_TARGET_DIR`（编译目录，默认 `<草稿目录>/target`）。跑完看末几行的 `name=rerun_compare … same`：`compare-opus`、`compare-sonnet`、`compare-tail`、`compare-nine`、`sonnet-aggregate`、`flags`、`tally`、`cells-opus-pointer`、`cells-sonnet-mapping`、`cells-sonnet-pointer`、`mapping-middle` 十一项都应是 same，退出码 0。前台命令超过 4 分钟的话，它得放进主 agent 的后台跑（子 agent 不放后台）。

## 草稿与产物去向

- 入库：本报告与 `research/prompts/c834-r3-sonnet-model/`（`README.md`、`tools/`、`out/`）。
- 草稿目录 `~/.local/share/singlefs/work/c834-r3-sonnet/`：`runs-opus/`（90M）、`runs-sonnet/`（173M）是这一轮逐状态原样输出，太大不入库，留着供主 agent 核汇总（`summarize` 步可以直接在它上面重汇总）；`runs-sonnet-first90/`（62M）是第一遍只起 90 件核流程的输出，已被 280 件全量取代，不入库；`report/` 是报告的分段草稿。
- 交回前删掉的：两份冻结副本拷贝 `opus/tree-candidates/`、`sonnet/tree-candidates/` 与编译目录 `target/`（见交回正文）。
