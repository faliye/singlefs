# m3-c651-r1 云端攻方腿（Opus）报告：全池没有可用的系统配置槽 0 时槽 1 在哪

日期 2026-10-02。攻击面 V1–V3，重点 V1。下面的数都是攻方副本上量的：冻结副本 `/tmp/claude-1000/m3-c651-r1/tree/` 拷出来，打上候选开关（`patch_copy.py`），不是入库装置上的数。重型测试一条没跑；原型是自己的独立 crate `c651proto`，调 `singlefs-checker-tier` 的库函数 `judge_layer0_state_range`（层 0 枚举域，每段都展开，原地覆写取三态），没跑 checker 档的测试二进制。

## 复跑

模型目录 `research/prompts/m3-c651-r1-opus-model/`：

| 文件 | sha256 |
|---|---|
| `patch_copy.py`（副本补丁：core `recovery.rs`、checker `image.rs` 各装一个候选开关 `C651_CANDIDATE`，mkfs 装零轮收严开关 `C651_MKFS_CLEARS`） | `be4d9adc5c9499a649d91b100dcb0a10fa53389c78133d4f8ebd7047ef127f2e` |
| `copy-vs-frozen.diff`（副本相对冻结副本的全部改动，`diff -ru` 原样，170 行） | `9e5a252e55ede090864781ca4c6f64485b73ac53c5e4a7c20258a3917ead89c2` |
| `c651proto/Cargo.toml` | `9be5e987b556c57b99c1e62efeaf86b2a68109eb58d8ad1eec1c95381dbe8551` |
| `c651proto/src/main.rs`（造历史、录尾巴写流、逐态判） | `d94d90d168223f52ee915864d36bf5294060bd800551fa0fb307f1d835e6db16` |
| `run_batch.sh`（九个候选各跑一遍，退出码各写 `.rc`） | `f6eafb1dcbe30248fde61b545c0add9af556eb6cadba4e8ff0684d946fd83bdf` |
| `tabulate.py` / `pivot.py` / `summary.py`（汇表；`tabulate.py` 的第一个参数给 `runs/` 也行） | `e7c2d20c6a162515512e2a3bc295c2ba6cb03628655fc59e3f7d8c9d4339a3ec` / `ee238f712dd411438f2433d48a4a0d993f0fd647e69fdcc655a92cbc0fc81b75` / `c0fe9e69ae2092a4a50f0f5a1df05816d0efbff6fe8daaa9cedb70f489ec71e6` |
| `runs/`：批 2 九份日志 `b2-*.log`、退出码 `b2-*.rc`、`batch2.log`、`tab2.txt`、`pivot2.md`（189 段历史 × 9 候选并排）、`summary2.txt` | 见文件 |

```
D=<草稿目录>; M=<仓>/research/prompts/m3-c651-r1-opus-model
mkdir -p $D/repo && rsync -a /tmp/claude-1000/m3-c651-r1/tree/ $D/repo/ && cp -r <仓>/.cargo $D/repo/
python3 $M/patch_copy.py $D/repo && rsync -a $M/c651proto/ $D/repo/c651proto/
# 副本的 Cargo.toml 工作区成员里加一行 "c651proto"
cd $D/repo && CARGO_TARGET_DIR=$D/target nice -n 19 bash <仓>/research/scripts/run-with-memory-cap.sh auto \
  bash <仓>/research/scripts/capped.sh 8 cargo build --offline --release -p c651proto
cd $D && nice -n 19 bash <仓>/research/scripts/run-with-memory-cap.sh 3G bash <仓>/research/scripts/capped.sh 8 bash $M/run_batch.sh $D 2
python3 $M/tabulate.py $D 2 | python3 $M/pivot.py      # 并排表
python3 $M/tabulate.py $D 2 | python3 $M/summary.py    # 每个候选按结局计态数
```

历史与状态是确定的（内存稀疏盘、固定 fsid、固定内容），重跑应逐行相同。批 2 九件 `rc=0`、`rc_files=9 expected=9`；每件 `name=done … histories=192 states=3240`，挂钟 13.7–18.5 秒。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| V1-常量（checker 今天的 ③） | **打中**（量过） | io_min 8192 / 65536 的新池，只靠一次崩溃（同一步轮换把两块盘的槽 0 都撕了）就到「全池槽 0 不可用」：常量读法读 4096（槽距空档，全 0），整池拒，而两块盘的槽 1 在 8192 / 65536 处完好。今天的组合（实现逐档、checker 常量）在同一批态上 checker 报 I-8.5 红、实现照常挂上 |
| V1-拒 | **打中**（量过） | 同一种单次崩溃在 io_min 512（槽距 4096，所有现役测试的几何）上也整池拒：C 组 12 态全拒，S 组 240 态全拒 |
| V1-逐档（实现今天的 ③） | **打中**（量过） | 盘上留着更早一次 mkfs、槽距不同的槽 1（用过的盘换了 io_min 重做 mkfs），再来一次同样的崩溃：择到旧槽。fsid 不同时整池拒（`NoValidRoot` / 读失败 / `SystemConfigurationsDisagree`）；fsid 相同时**挂上旧池、读出旧池的文件**（`FileReadWhereThisRootHasNone`） |
| V1-探测（按挂载时探得的 io_min 算槽距，代码注释提过的第四种） | **打中**（量过） | D2 已定项 19 的挂载闸 (a) 许可实际 io_min 小于记录的槽距；探得 512 时它与常量同结局，探得 8192 时在槽距 4096 的池上整池拒 |
| V1-首个（E170 欠一句表 G03 的暂代读法：升序第一个） | **打中**（量过） | 旧槽在更小偏移时择旧槽，结局同逐档 |
| V1-唯一（我的零轮收严：一盘只收得到一档才收） | 不择错，但整池拒（量过） | 两块盘都是旧盘时 120 态全拒 |
| 逐档 + mkfs 把 [0, 根环基址) 整段清零（我的零轮收严） | 量过的 252 态全对 | 实现 252 态全择到本池槽 1；checker 另有 10 态报 I-7.7 红，其他每个候选在这 10 态上也报，和槽 1 读法无关（见「没打中的形状」第 4 条） |
| V2 | 条款没有这一句（推的 + 量过一格） | 世代号相同取偏移小，只在「本池槽 1 对旧池残留槽」时才用得上，且只在新槽距更小时选对 |
| V3 | 给出装置与档位（推的） | harness 档两张镜像加一条 33 态的层 0 流；checker 档测试里找不到用 4096 之外槽距的流（grep 0 命中，没逐个追 `parameters()` 的出处） |

## 怎么造的历史

- 盘：两块 `SparseBlockDevice`（内存稀疏盘），每块 (1408 + 384) × 16384 字节，journal 环 6 MiB（`crates/singlefs-harness/tests/common_admission_pool/mod.rs` 那一档小盘）；物理块 512，io_min 取 512 / 8192 / 65536，槽距按 `SystemImmutableSizes::slot_spacing_for` 算（4096 / 8192 / 65536）。
- 前缀直接在内存盘上做，不进写流；尾巴录成写流，进层 0 枚举：
  - 只做过 mkfs 的池（n=0）：尾巴是第一次可写挂载（取号 + 暖机两次），段形 `[2, 2, 1, 2, 2, 1, 2]`，33 态，其中 2 态两块盘的槽 0 同时撕裂（取号写世代 2、暖机第二次轮换写世代 4，都落槽 0）。
  - 用过的池（n=2：取号、暖机、第一个文件，世代到 5；n=3：再覆盖写一次，世代 6；n=4：再一次，世代 7）：尾巴只录下一次可写挂载的第一次写，取号，段形 `[2]`，9 态；n=2、n=4 各有 1 态两块盘槽 0 同时撕裂，n=3 那一次取号写槽 1、没有这种态（对照）。
- 两块盘的槽 0 同时撕裂是层 0 枚举域里的态，不是我加的故障：轮换在一个循环里给每块盘各写一槽、之间没有屏障（`crates/singlefs-core/src/transaction.rs:328`「for index in 0..self.devices.len() {」），写的槽是「(slot_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing,」（`crates/singlefs-core/src/transaction.rs:414`），两次写同段；系统配置槽写是原地覆写，取三态，其中一态是「新旧都读不出」（`crates/singlefs-checker-tier/src/crash.rs:1648`「原地覆写的写取三态（没持久 / 新旧都读不出 / 持久）」）。
- C 组（新盘，纯崩溃）：io_min 3 档 × n 4 档 = 12 段历史。
- S 组（旧盘重做 mkfs）：盘上先有一个池（io_min x，用户动作 u = 0 只 mkfs / 1 取号加暖机 / 5 再写一个文件、覆盖三次，旧槽 1 的世代号分别是 1 / 3 / 7），换 io_min y 重做 mkfs（x→y 取 8192→512、65536→512、512→8192、512→65536、65536→8192），两块盘都是旧盘或只有盘 0 是旧盘，新池 fsid 与旧池不同或相同（mkfs 的 fsid 由调用方给），之后 n = 0 / 2 / 4。5 × 3 × 2 × 2 × 3 = 180 段。
- 用户决定的几步（旧池用了多少、新池 mkfs 之后做了几次发布、哪几块盘是旧盘、fsid）都放开扫，只固定「重做 mkfs」与「两盘槽 0 同撕」这两样前缀与故障（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节）。
- 每个态：恢复（`recover`，内含实现的 `choose_system_configuration`）、多版本 oracle、池级 checker（`check_pool_image`，内含 checker 的 `system_configuration_slot_readings`）都跑；另单独调一次 `choose_system_configuration` 记它择到哪一槽（fsid、槽距、世代号）。
- 状态数：批 2 九件各 192 段、3 240 态（`closed_form_state_count` 两态口径每段不超过 18，三态口径不超过 33），远在一次 10⁶、合计 10⁷ 之内。批 2 全量 9 × 3 240 = 29 160 态；批 1 被停之前八件各跑完 16 段（每件 197 151 态：C 组 9 段 135 486 态、S 组 7 段 61 665 态，按日志里 `states=` 加起来），合计约 1.6 × 10⁶ 态，在 10⁷ 之内。
- 缩过一次历史长度：批 1 的 n=2 / n=3 尾巴是整次可写挂载（段形 `[2, 14, 2, 1, 2, 12, 2, 1, 2]`，20 511 态；`[…, 12, 2, 1, 2]`，24 618 态）。批 1 九件并行、30 分钟里每件只跑完 16 段历史，被后台时限停了（日志原样在 `runs/batch1-stopped/`，只作参考）。批 2 把用过的池的尾巴缩成「只录取号」（9 态）。被缩掉的是挂载取号之后的写，与「全池槽 0 不可用」无关：那几段里两盘槽 0 同撕的态，批 1 的 n=2 已经量到，结局与批 2 同一行（批 1 `today` 日志 `C:y=8192:n=2 … all_slot0_unusable=2 … checker_red=2`）。估时：批 1 每段 2 万态约 2–4 分钟；批 2 先单跑 `C:y=8192` 四段，1.1 秒，按 192 段估约 1 分钟，实测 13.7–18.5 秒。

## V1：三个候选（加我认出的三种）各在哪种态上出错

下面引的都是 `runs/` 里日志的原样行（一行一个签名，`count` 是态数）。

### 打中 1：V1-常量在单次崩溃上把能挂的池拒掉（io_min ≠ 4096）

C 组 `C:y=8192:n=2`，取号那一步两块盘的槽 0 同撕，两块盘的槽 1（世代 5）在 8192 处完好：

```
name=sig candidate=const h=C:y=8192:n=2 count=1 all_slot0_unusable=true core=Err:NoValidSystemConfiguration outcome=Failed:NoValidSystemConfiguration oracle=NoRootChosen checker_slot1=notverified|notverified checker_red=["I-8.5"]
name=sig candidate=search h=C:y=8192:n=2 count=1 all_slot0_unusable=true core=new@8192g5d0 outcome=FileRead(i1,t3) oracle=- checker_slot1=new@8192g5|new@8192g5 checker_red=[]
name=sig candidate=today h=C:y=8192:n=2 count=1 all_slot0_unusable=true core=new@8192g5d0 outcome=FileRead(i1,t3) oracle=- checker_slot1=notverified|notverified checker_red=["I-8.5"]
```

第三行是今天的组合：实现读出了文件，checker 在同一态上报 I-8.5「全池皆废」。也就是说，只要层 0 有一条流用 io_min 8192 的盘，今天就会在这一态上判红。层 0 今天多半没有这样的流（推的，依据只是 V3 一节那几条 grep）：`crates/singlefs-checker-tier/tests/` 里没有一处调 `slot_spacing_for`，也没有一处 `fixed_structure_slot_spacing` 写成 4096 之外的字面值（V3 一节的命令）。io_min 65536（`C:y=65536:*`）结局相同。介质损坏只抹槽 0（`zeroed=4096`）时结局也一样；按 GPT 的 34 个扇区抹 [0, 17408) 时，8192 处的槽 1 也被抹掉，只有 65536 那一档的槽 1 还在，逐档挂得上、常量挂不上。

四句：
1. 分辨臂：分。逐档、首个、唯一、逐档加清零在这些态上都择到本池槽 1；常量、拒、探测（探得 512）都整池拒。
2. 判别的东西被判系统看不看得到：看得到。8192 处的槽自证得过，记的槽距 8192 等于它所在的偏移；读者只要去读就能看到。
3. 满足的分句：派发给的攻击面「在一个真会出现的状态（…崩溃在系统配置写中途、io_min 不是 4096 的设备）上…把能挂的池拒掉」。对条款来说，D22 已定项 16 写的是「槽 i 的设备内偏移 = i × 固定结构槽距」（`.claude/kb/decisions/22-单元原子性怎么合成.md:331`），常量读的 4096 不是这个池的槽 1；I-8.5 写「池里至少有一块盘的两个系统配置槽里有一个槽自证过」（`.claude/kb/invariants.md:94`），这里两块盘的槽 1 都自证过，常量读法下 checker 却报红。所以常量与 I-8.5 的字面相冲，前提是槽距不是 4096 的池存在；D2 已定项 19 正是让这种池存在的条款（`.claude/kb/decisions/02-RAID条带策略.md:313`「槽距 = 4096 向上取整到 mkfs 时探测的 `io_min` 的整数倍」）。
4. 改法在这几格上中不中：跑前条款没给改法。改法逐个量过的见「改法表」：逐档、首个、唯一、逐档加清零都不中；探测只在探得的 io_min 等于 mkfs 时那个时不中。

### 打中 2：V1-拒在槽距 4096 的池上也把单次崩溃变成整池拒

```
name=sig candidate=refuse h=C:y=512:n=2 count=1 all_slot0_unusable=true core=Err:NoValidSystemConfiguration outcome=Failed:NoValidSystemConfiguration oracle=NoRootChosen checker_slot1=notverified|notverified checker_red=["I-8.5"]
```

`summary2.txt`：`summary label=refuse group=C states=12 ok=0 checker_only=0 stale_mounted=0 refused=12`。

四句：
1. 分辨臂：分。逐档、首个、唯一、常量（槽距 4096 时）在这一态上都挂得上。
2. 看得到：看得到，同打中 1。
3. 满足的分句：「把能挂的池拒掉」。对条款：D23 已定项 3 改 jbd2 的第 1 处写「改 ≥2 槽轮换 + 代号 + 先逐个验证全部候选再择新」，理由是「一次撕裂会抹掉唯一见证者」（`.claude/kb/decisions/23-journal的角色与格式.md:122`）。V1-拒让同一次轮换里两块盘槽 0 的一次撕裂抹掉全池的见证者，两槽轮换要防的正是这个；它也不「逐个验证全部候选」：槽 1 一个都不读。D22 已定项 8 第 1 条「系统配置每盘放一份。」（`.claude/kb/decisions/22-单元原子性怎么合成.md:174`）买的冗余，在这一态上也一份都用不上。checker 在拒读法下报 I-8.5 红，与 I-8.5 字面「两个系统配置槽里有一个槽自证过」（`.claude/kb/invariants.md:94`）对不上：槽 1 自证过，只是没去读。
4. 改法：同打中 1，拒读法本身就是这一格的结局，没有在它之内修的改法。

### 打中 3：V1-逐档在重做过 mkfs 的盘上择到旧池的槽

旧池在盘上留下的槽 1，自证得过，记的槽距也等于它所在的偏移。mkfs 只清根环与 journal 环（`crates/singlefs-core/src/make_filesystem.rs:551`「for region_to_clear in 0..ROOT_RING_REGIONS {」），然后写本次的两槽。所以旧池的槽 1 只要不落在 0 或新槽距上，就一直留着。实现自己的注释已经写了这个风险（`crates/singlefs-core/src/recovery.rs:953`「风险：同一块盘上更早一次 mkfs 用别的槽距写下的槽 1，这一次 mkfs 不清」），harness 用例是把字节手摆进去的。这里换成真走一遍旧池、重做 mkfs、再崩溃，结局是：

```
name=sig candidate=search h=S:x=65536:y=512:u=5:reuse=both:fsid=same:n=0 count=2 all_slot0_unusable=true core=new@65536g7d0 outcome=FileRead(i1,t6) oracle=FileReadWhereThisRootHasNone checker_slot1=new@65536g7|new@65536g7 checker_red=[]
name=sig candidate=search h=S:x=8192:y=512:u=5:reuse=disk0:fsid=diff:n=2 count=1 all_slot0_unusable=true core=Err:SystemConfigurationsDisagree outcome=Failed:SystemConfigurationsDisagree oracle=NoRootChosen checker_slot1=old@8192g7|new@4096g5 checker_red=["I-1.4", "I-7.1"]
name=sig candidate=const h=S:x=8192:y=512:u=5:reuse=disk0:fsid=diff:n=2 count=1 all_slot0_unusable=true core=new@4096g5d0 outcome=FileRead(i1,t3) oracle=- checker_slot1=new@4096g5|new@4096g5 checker_red=[]
name=sig candidate=search h=S:x=8192:y=512:u=5:reuse=both:fsid=same:n=2 count=1 all_slot0_unusable=true core=new@8192g7d0 outcome=FileRead(i1,t2) oracle=- checker_slot1=new@8192g7|new@8192g7 checker_red=[]
```

- 第一行：新池（槽距 4096）mkfs 之后一次文件都没写，恢复却读出一个文件，根是实例 1 第 6 代。那是旧池（槽距 65536，写过一个文件、覆盖三次）的最后一版。也就是说，**挂上的是旧池**。fsid 相同时，各盘择到的 fsid 对得上，`SystemConfigurationsDisagree` 拦不住；旧池的根落在新 mkfs 按 4096 × S 清的那一段之外，留下来了（第 6 代落区域内槽位 (6 div 3) mod 8 = 2，偏移 2 × 65536，超出新 mkfs 清的 8 × 4096；按区域内槽位公式推的，没逐字节看）。
- 第二、三行：只有盘 0 是旧盘、fsid 不同。逐档的盘 0 择旧槽、盘 1 择新槽，整池拒；常量在这一格（旧槽距 8192 大于新槽距 4096）择对。
- 第四行：逐档择到旧槽（槽距 8192、世代 7），oracle 却过了：按旧槽距 8192 读根环，新池的根落在 4096 的偶数倍上照样读得到，journal 又重建出第 3 代。择到的几何是错的，oracle 没看见（推的：之后可写挂载按 8192 写槽与根会怎样，我没跑）。
- 逐档在 S 组 240 个两盘槽 0 同撕的态里：`summary label=search group=S states=240 ok=136 checker_only=0 stale_mounted=52 refused=52`（`stale_mounted` 是实现择到旧槽、恢复没报错，含上面两种 oracle 过或不过；`refused` 是择旧槽后恢复失败，或两盘择到的对不上）。

放开用户动作扫出来的结果（`pivot2.md`）：旧槽赢，条件是它的世代号比本池槽 1 大，或者两边相等而旧槽偏移更小。旧池用得越久（u=5 时旧槽 1 是世代 7），新池写得越少（n=0 时本池槽 1 是世代 1 或 3），就越会中。`S:x=65536:y=512:u=5:reuse=both:fsid=same:n=4`（本池槽 1 追到世代 7，平局取小偏移 4096）择对：

```
name=sig candidate=search h=S:x=65536:y=512:u=5:reuse=both:fsid=same:n=4 count=1 all_slot0_unusable=true core=new@4096g7d0 outcome=FileRead(i1,t5) oracle=- checker_slot1=new@4096g7|new@4096g7 checker_red=[]
```

所以「逐档不中」只在新池世代号追上之后才成立。只看 n=4 那一格会误判成不中。

四句：
1. 分辨臂：分。常量在旧槽距更大（x > y）的格上择对，在旧槽距更小（x < y，旧槽 1 就在 4096）的格上同样择旧槽：`name=sig candidate=const h=S:x=512:y=8192:u=1:reuse=both:fsid=diff:n=0 count=2 all_slot0_unusable=true core=old@4096g3d0 outcome=Failed:NoValidRoot oracle=NoRootChosen checker_slot1=old@4096g3|old@4096g3 checker_red=["I-7.1"]`。逐档加清零在这 240 态里一态都不择旧槽。
2. 看得到：**做决定那一刻，只读的读者看不到**。两槽都自证过、记的槽距都等于所在偏移；fsid 相同时，两槽里能分出新旧的只剩世代号，而世代号恰恰偏向旧池。根环也不一定分得开（第一行里旧池的根还在）。所以单靠「读法」这一句修不了这一格，这是证据纪律那张表里「判别子观测不到」那一行。要分得开，只能在写的一侧让旧槽不存在（mkfs 清零），或者在格式里加一样能分的东西（推的，没做）。
3. 满足的分句：V1 题面那一问「逐档那一种「自述等于所在偏移」会不会把一个旧的、槽距不同的残留槽当成槽 1」：会，而且后果不只是整池拒，还可能读出旧池的文件。对条款：D22 已定项 16 的「择槽取校验和过且世代号最大的」（`.claude/kb/decisions/22-单元原子性怎么合成.md:331`）照字面就择旧槽，条款里没有哪一句把别的 mkfs 留下的槽排除在外。
4. 改法：见「改法表」。逐档加清零修掉全部 240 态；首个只修 x > y 那一半；唯一把择错变成整池拒；常量只修 x > y，同时把打中 1 带回来。

### 打中 4：我认出的另三种读法

- **V1-探测**（槽距按挂载时探得的 io_min 现算；实现的注释提到过这条路：`recovery.rs` 第 1080–1081 行说要 `PoolReader` 交得出探测值，今天交不出）。D2 已定项 19 的挂载闸是「(a) 实际 `io_min` ≤ 记录的槽距」（`.claude/kb/decisions/02-RAID条带策略.md:313`），许可池挪到 io_min 更小的设备上（例：镜像从 dm-thin 拷成普通文件）。探得 512 时，它在 C 组 io_min 8192 / 65536 的池上整池拒，与常量相同：`name=sig candidate=probe h=C:y=8192:n=2 count=1 all_slot0_unusable=true core=Err:NoValidSystemConfiguration outcome=Failed:NoValidSystemConfiguration oracle=NoRootChosen checker_slot1=notverified|notverified checker_red=["I-8.5"]`（`runs/b2-7-probe512.log`）。探得 8192 时，在槽距 4096 的池上整池拒（`pivot2.md` 的 `probe8192` 一列，`C:y=512:*` 那两行）。它也躲不开旧槽：`summary label=probe8192 group=S states=240 ok=92 checker_only=4 stale_mounted=24 refused=120`。
- **V1-首个**（E170 欠一句表 G03 的暂代读法：升序第一个自证过、记的槽距等于偏移的槽）。旧槽在更小偏移时择旧槽：`name=sig candidate=first h=S:x=512:y=8192:u=1:reuse=both:fsid=diff:n=0 count=2 all_slot0_unusable=true core=old@4096g3d0 outcome=Failed:NoValidRoot oracle=NoRootChosen checker_slot1=old@4096g3|old@4096g3 checker_red=["I-7.1"]`。`summary label=first group=S states=240 ok=138 checker_only=6 stale_mounted=48 refused=48`。
- **V1-唯一**（我的零轮收严：一块盘上逐档只收到一档才收，收到两档以上当这块盘没有可择槽）。它不择错，但两块盘都是旧盘时整池拒：`name=sig candidate=unique h=S:x=8192:y=512:u=5:reuse=both:fsid=diff:n=2 count=1 all_slot0_unusable=true core=Err:NoValidSystemConfiguration outcome=Failed:NoValidSystemConfiguration oracle=NoRootChosen checker_slot1=notverified|notverified checker_red=["I-8.5"]`。`summary label=unique group=S states=240 ok=0 checker_only=120 stale_mounted=0 refused=120`；那 120 态 `checker_only` 是只有盘 0 是旧盘的格：盘 0 两档都收到、按唯一读法不收，另一块盘照常择，实现挂得上，checker 的盘 0 读成没有可用槽，不判红。

### 相容性小结（条款文字 × 候选，量过的格标出处）

| 候选 | D22 已定项 16 | D2 已定项 19 | D23 已定项 3 | I-8.5 | 在哪个可达状态上错 |
|---|---|---|---|---|---|
| V1-常量 | 槽距 ≠ 4096 时读的不是「i × 固定结构槽距」 | 它许可的槽距 > 4096 的池上错 | 相容 | checker 在槽 1 自证过时报红（打中 1） | io_min > 4096 的池单次崩溃；旧槽在 4096 时择旧槽 |
| V1-拒 | 相容（不读就不会读错） | 相容 | 两槽轮换要防的一次撕裂照样抹掉全池见证者 | 槽 1 自证过时整池拒，I-8.5 却成立（打中 2） | 任何槽距的单次崩溃 |
| V1-逐档 | 照「世代号最大」字面择旧槽 | 相容 | 相容 | 相容 | 用过的盘换 io_min 重做 mkfs，之后单次崩溃（打中 3） |
| V1-探测 | 实际 io_min ≠ mkfs 时那个时读错 | 闸 (a) 许可的情形里错 | 相容 | 同常量 | 池挪到 io_min 不同的设备 |
| 逐档 + mkfs 清 [0, 根环基址) | 相容 | 相容 | 相容 | 相容 | 量过的 252 态里没有；没被攻过 |

「两个以上相容、而在一个状态上给出不同结局」这一条（跑前判据第 2 句）：照字面看，只有逐档与条款文字都不冲突，常量、拒、探测各与至少一句条款相冲（上表）。逐档的错不在条款文字上，在条款缺了一句：别的 mkfs 留下的槽怎么排除。这一句只靠读的一侧补不出来（打中 3 第 2 问）。

## V2：几档都收得到时取哪一槽

- 「世代号最大」出自 D22 已定项 16「择槽取校验和过且世代号最大的」（`.claude/kb/decisions/22-单元原子性怎么合成.md:331`），说的是一块盘上的两槽之间，逐档时实现把它推到了 2 033 档之间。
- 「世代号相同取偏移小」没有条款。同一个池里，逐档只会收到一个槽 1（槽 0 不可用才走到这一步，本池的槽 1 只有一个），所以平局只出现在本池槽 1 对旧池残留槽之间。取小偏移只在新槽距更小时选对。量过的一格：`S:x=512:y=8192:u=0:…:n=0`，新旧槽 1 世代号都是 1，`today` 一列取到 4096 处的旧槽（`pivot2.md` 那一行 `N/SS[I-7.1]+S!noroot/SS[I-7.1]`，第二个态就是平局那一态）。
- I-7.16「两个槽都可择时世代号相差不超过 1，相等时只许都是 1」（`.claude/kb/invariants.md:70`）管同一块盘的槽 0 与槽 1，管不到逐档收到的两个「槽 1」。
- 推的：若 mkfs 把 [0, 根环基址) 清零（下面的改法表），逐档在一块盘上最多收到一个槽，V2 这一问就没有了；不清零的话，平局规则怎么写都有一半格错，错的那一半由新旧槽距哪个大决定。

## V3：checker 与实现照同一句之后怎么验

- 现查：`grep -rn 'slot_spacing_for' crates/singlefs-checker-tier/tests/ | wc -l` 输出 `0`；`grep -rn 'fixed_structure_slot_spacing: ' crates/singlefs-checker-tier/tests/ | wc -l` 输出 `0`（槽距只以 `parameters().geometry.fixed_structure_slot_spacing` 的形式出现，取自 harness 的 common 模块，那里写的是 4096；没逐个追）；背景材料要数的 `grep -rn 'io_min' crates/singlefs-harness/tests/ | wc -l` 输出 `38`。
- 坏盘镜像（harness 档，内存稀疏盘、几毫秒）：
  1. mkfs(io_min 8192) → 取号 → 暖机 → 两块盘槽 0 抹坏：断言 `choose_system_configuration` 与 `singlefs_checker::image::system_configuration_slot_readings` 都择 8192 处的槽 1，且 `check_pool_image` 的 I-8.5 成立。今天这一张会红在 checker 那一半（打中 1 第三行就是它的层 0 版）。已有的 `slot_one_is_found_by_trying_every_spacing_when_no_slot_zero_in_the_pool_verifies` 只判实现那一半。
  2. 真走一遍旧池（io_min 65536、用过）→ 换 io_min 512、同 fsid 重做 mkfs → 两块盘槽 0 抹坏：断言两边都择本池槽 1（4096 处）、恢复报没有文件。今天实现会读出旧池的文件（打中 3 第一行）。这一张替换手摆字节的 `the_search_takes_the_highest_generation_even_when_it_is_a_stale_slot_of_an_earlier_make_filesystem`：手摆的只证明逐档会择旧槽，证明不了这种盘面能真走出来。
- 层 0（checker 档，提交时跑）：mkfs(io_min 8192) → 第一次可写挂载这条流，33 态、2 态两盘槽 0 同撕（我的副本上的数）：`name=history candidate=today h=C:y=8192:n=0 segments=[2, 2, 1, 2, 2, 1, 2] states_two_state=18 states=33 all_slot0_unusable=2 oracle_red=0 checker_red=2`。量级小，可以全量进门禁 54 号那一档。
- O3 严格跑法照这一句写的那一步（推的，零轮）：「全池没有一块盘的槽 0 可择时，一块盘的槽 1 = 在 [4096, 根环基址 − 4096] 里每 512 字节一档读到的、自证过、池级字段在读者收的范围里、记的槽距等于所在偏移的那些槽里世代号最大的；mkfs 写系统配置槽之前把 [0, 根环基址) 整段清零，所以一块盘上最多收到一个」。执行器按前半句逐档，后半句给它一个可判的前提：收到两个以上就是 mkfs 违约，报出来，不择。

## 改法表（每格：中 = 这个读法在这一格上还出错；量过 = 副本上原样输出见 `runs/`；推的 = 没实现、没跑）

| 格 | 常量 | 拒 | 逐档 | 探测 | 首个 | 唯一 | 逐档 + mkfs 清零 |
|---|---|---|---|---|---|---|---|
| io_min > 4096 的新池、两盘槽 0 同撕（C 组 y=8192 / 65536） | 中，量过 | 中，量过 | 不中，量过 | 探得值等于 mkfs 时的不中，不等就中，量过 | 不中，量过 | 不中，量过 | 不中，量过 |
| 槽距 4096 的新池、两盘槽 0 同撕（C 组 y=512） | 不中，量过 | 中，量过 | 不中，量过 | 同上 | 不中，量过 | 不中，量过 | 不中，量过 |
| 旧盘重做 mkfs、旧槽距更大（S 组 x > y） | 不中，量过 | 中，量过 | 中（世代号决定），量过 | 探得值算出的槽距等于新池槽距才不中，量过 | 不中，量过 | 两盘都是旧盘时整池拒，量过 | 不中，量过 |
| 旧盘重做 mkfs、旧槽距更小（S 组 x < y） | 中，量过 | 中，量过 | 中（世代号决定），量过 | 同上，量过 | 中，量过 | 同上 | 不中，量过 |
| GPT 式抹 [0, 17408)、槽距 65536 | 中，量过 | 中，量过 | 不中，量过 | 探得 65536 不中，推的 | 不中，量过 | 不中，量过 | 不中，量过 |
| 池挪到 io_min 更小的设备上（D2 已定项 19 闸 (a) 许可） | 与设备无关，同第 1 行，推的 | 同第 1 行，推的 | 不中，推的（不读探测值） | 中，量过（探得 512 那一列） | 不中，推的 | 不中，推的 | 不中，推的 |

所有带「我的」的改法（唯一、逐档 + mkfs 清零）**只在我的模型上量过、被攻过零轮**。逐档 + mkfs 清零的代价（推的）：实现一侧，mkfs 每块盘多一次整段清零（`patch_copy.py` 的 mkfs 那一处 4 行），逐档本身今天已经在实现里；checker 一侧要补一份逐档（我副本里的 `c651_checker_slot_one_when_no_slot_zero_in_the_pool` 约 45 行）。它会改 D22 已定项 8 第 3 条写的 mkfs 步数（「每块盘 4 步、两块盘共 8 步」那一句，`.claude/kb/decisions/22-单元原子性怎么合成.md:176`），layout 八「mkfs 种根」那一行跟着改。层 0 从 mkfs 之后起枚举，基线镜像里多出的只是 0。清零之后，同一块盘上更早一次 mkfs 留下的任何槽都在 [0, 1 MiB) 里被抹掉。它挡不住的（推的）：mkfs 之后才写进那一段的外来字节（别的工具写的、不是 singlefs 的槽，自证不过，不影响）。

## 没打中的形状

1. 只有一块盘的槽 0 撕了、另一块的槽 0 可用：走借槽距那一档（②），九个候选全都择对。层 0 里全池槽 0 都可用、或只坏一块的态，除第 4 条那 70 态之外一态都不红（`tabulate.py` 的 `other_red` 行，每个候选都只有 `oracle=- red=["I-7.7"] states=70`）。
2. 新旧两次 mkfs 槽距相同：没进取样（S 组只取 x ≠ y）。按代码推，新 mkfs 写的两槽正好盖掉旧的两槽，没有残留槽（推的）。
3. 逐档撞上一个自证得过、池级字段这个读者不收的残留槽时整池拒（`recovery.rs:978` 那个 `?`）：没造出来。同一版 mkfs 写出的旧槽字段都在收的范围里；要造就得有另一版格式写过这块盘，第一版只有一版（推的，没试）。
4. 同 fsid 重做 mkfs、旧池用过（u=5）、新池 n=0 的那几格，每个候选都有 checker 报 I-7.7 红（全池槽 0 都可用的态 70 个，两盘槽 0 同撕的态里逐档 + 清零那一列 10 个）。各候选的数一样，所以分不出候选，是另一笔账：多半是旧池的根留在新 mkfs 按新槽距清的那一段之外（与 C484 同一类），根因我没查。
5. io_min 非 2 的幂：今天 mkfs 按判定宽度拒（`mkfs_refuses_geometries_whose_root_slots_do_not_align.rs`），走不到。只按 2 的幂逐档的读法会漏掉调用方直接给的槽距 4608（已有用例 `a_slot_spacing_that_is_a_multiple_of_512_bytes_only_is_found_too` 用 mkfs 造得出来）。这个读法我没量。
6. mkfs 中途崩溃（槽 1 写了、槽 0 没写）：层 0 从 mkfs 之后起枚举（`crates/singlefs-core/src/make_filesystem.rs:541`「mkfs 中途崩溃第一版没有条款」），不在攻击面里。

取样范围：C 组 12 段、S 组 180 段，每段的崩溃态按层 0 枚举域全取；io_min 只取 512 / 8192 / 65536；两块盘、物理块 512、环 6 MiB、S = 8；介质损坏只取两种（抹 [0, 4096)、抹 [0, 17408)），都在整条尾巴持久之后。

## 这条腿自己的限度

- checker 一侧的逐档、首个、唯一、探测是我在副本里照实现的语义写的（含「撞上不收的槽整池拒」那一处），没有独立路径核过两边逐态等价。日志里实现只打出它择到的那一份（多是盘 0 的），没有逐盘逐态比两边择的是不是同一槽；`pivot2.md` 里能并排看到实现择到的与 checker 每盘读到的。
- 只跑了读的路径（`recover` 与 `check_pool_image`）。择到旧槽之后可写挂载会写什么、写坏什么，没跑。打中 3 第四行那种「oracle 过、几何错」的后果是推的。
- 「用过的盘换了 io_min 重做 mkfs」有多常见，没量。E129 量过真设备上 io_min 65536 的存在（dm-thin）；盘从这类设备上拿下来重做 mkfs 这一步是推的。
- 尾巴缩过（只录取号），理由与估时在「怎么造的历史」一节；被缩掉的写里两盘槽 0 同撕的态，批 1 量到的结局与批 2 一致，但批 1 没跑完所有候选的所有段。
- 一条腿只抽一次样：上面的「没打中」都只是这一次没打中。

## 没做什么

- 没跑任何重型测试（checker 档测试二进制、门禁、Verus、变异）；原型只调 `singlefs-checker-tier` 的库函数，测试目标与二进制名都不带 layer0。
- 没改仓里的 `crates/`；没写 kb；不替主 agent 采纳；没判正推腿那几格。
- 副本上的数不算入库装置上的数，要引就照规则在入库装置上重做。
- 草稿：仓副本 `/tmp/claude-1000/m3-c651-r1-attack/repo`（13M）与编译目录 `/tmp/claude-1000/m3-c651-r1-attack/target`（519M）已删。草稿目录里剩的是日志（2.3M），其中批 2 与批 1 的日志已拷进模型目录 `runs/`。
