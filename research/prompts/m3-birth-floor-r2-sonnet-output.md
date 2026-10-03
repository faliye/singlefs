# m3-birth-floor-r2 辩方腿（Sonnet）报告：复核第一轮判决；J2、J4、J5（2026-10-02）

<!-- doc-lint:not-numbers V1 V2 V3 V2s V3s F1 F2 F3 J1 J2 J3 J4 J5 CS1 CS2 M2 -->

批次：records/2026-09-30-里程碑三开工第一轮.md#5。立场：辩方，复核 `research/prompts/m3-birth-floor-r1-main-verification.md`。读冻结副本 `/tmp/claude-1000/m3-batch5-freeze/tree/`（sha256 与 `research/prompts/m3-birth-floor-r2-snapshot/tree-sha256.txt` 对过，`sha256sum -c` 不是 OK 的 0 行）；kb 行号都是冻结副本里的。没读禁读清单里的任何文件。

模型都在 `research/prompts/m3-birth-floor-r2-sonnet-model/`：`birth_floor_v2.py` 是第一轮正推腿的抽象模型原样拷来（sha256 与第一轮相同，2bae0b6e…），新写的是 `r2_variants.py`、`r2_lineage.py` 与几个 `run_r2_*.py`。纯 Python、不读 `crates/`；模型是抽象的，零红只对它的操作集说话。没编译 Rust，没跑 checker 档任何东西。

## 先说结论

| # | 判决条目 | 辩方判定 | 一句话证据 |
|---|---|---|---|
| 1 | 第 2 行「V1 出局」的依据：模型上 6 张 | **站得住（跑前判据字面打中）；但「V1 系救不了」不成立** | 6 张复现，是 3 种形状各 2 张；放宽到深度 8、含任意多次丢根槽，V1 系（攻方的 V3s 写法）红 41 810 / 12 618 652 与 12 973 / 16 307 544 张；同一批上给 V1 系补一条「回退与本根之间有被丢根槽的 txg 就整对报不适用」后 0 红。出局靠字面判据，不靠 V1 内核错 |
| 2 | 同一格「机理对得上代码」 | **对得上，但判决引的行号指错一处** | 读不出的根不进候选集靠 `image.rs` 的 `valid_roots`（`image.rs:994`），不是 `walk.rs:5977` 那一行（那一行判抛弃与 F；在这份冻结副本里是 `walk.rs:6766`）。建 inode 不动 extent 树、回退只换两条根指针、读不出的根槽不会被 journal 记录补回（没有同实例的实例表行）三处都对得上 |
| 3 | 第 4 行「取按树继承下界、不取 V2s / V3s」 | **取法站得住，写下的理由要改** | 「带 V1 的条件，那一格照样红」从推的变成模型上量过的（上一行的数）；真正的取舍是：V1 系多抓「只改一条链」（extent 链单改 k = 2、3：V1 系 7 240 / 7 240 抓到，按树继承下界 0 / 7 240；inode 链单改两边都 0 / 8 482，V1 的条件只看 inode 链），代价是丢槽时得报不适用、槽读坏另要判。按树继承下界不要这一补丁 |
| 4 | 第 5 行 / 第三节射程：谱系记录版 (b) 的措辞 | **J5：字面「T_rb = R.txg」不够，会红合法镜像；改成「R⁻.txg < T_rb ≤ R.txg」** | 字面版在丢了回退根的槽、之后建 inode 的镜像上红 41 810 张（与 V1 系同一批），改后 0 红；改后的写法在坏镜像层把整条链盖小（k = 1、2、3，36 306 张）全抓，按树继承下界在其中 k ≥ 2 的 24 204 张上 0 抓到（前提：谱系记录本身可信） |
| 5 | 第 9 行 α / β：β 多判的「再次出现」那一半 | **不值（推的，加模型上的数）；α 站得住** | 合法镜像上 α、β 同样 0 红（28 926 196 张）；非回退根再次引用旧单元的坏镜像，带已释放记录时 I-3.9 已红（`walk.rs:3519`），没有只红它的镜像（第一轮正推自己写了）。注意：判决写下的措辞本身就是 β 的写法（相邻差集），选 α 要把措辞改回「第一次出现」 |
| 6 | J2 非回退根再次引用 | **合法历史里不出现**（结构上 + 模型上） | 仓里把旧树装进新根的只有回退这一处（`mount.rs:5148`、`mount.rs:5163`）；模型含任意丢根槽、抛弃最新根、可写挂载，28 926 196 张镜像 0 红 |
| 7 | J2 洞两侧会不会误判再次出现 | **不会（模型）；一个「合成的」反例查清是模型的人造物** | 候选集不滤被抛弃的根时 β 红 13 176 张，全是实例表单元 `it`（13 176 / 13 176），是模型里「抛弃之后不经挂载写行就接着写」造成的，不当证据 |
| 8 | J4 措辞三处 + 代价 | **三处各给了写法（推的）；代价改了一个方向** | 同 txg 两条合法候选根造不出（`recovery.rs:1289`）；`deleted_inodes` 加进用户可见树、非全零才算；读不出头的单元不计入「压得住」。代价：判决里「去掉 depth 条件、每条候选根多走一遍」是贵的那条路；挂在主走读上（`walk.rs:6784` 已经对每条候选根走到叶）可能更便宜，没量 |
| 9 | 固定住的前提 | 「合法」的边界要写明 | 6 张里的故障是「单个根槽读不出」（根环 R = 3 个失败域设计就为它，D22）；形状还要求回退后建 inode、再抬 F 越过回退目标、且丢的恰是回退根——这是合法但窄的一类 |

## A　复核第 2 行：V1 出局的依据

### A1　模型上的 6 张

复跑第一轮正推腿的 `run_fault_split.py`（拷到草稿目录，`bash research/scripts/capped.sh 7 nice -n 19 python3 …`，41 秒）：与第一轮的 `run_fault_split.out` 只差最后两行首例里列的是哪个单元（`frozenset` 迭代序随哈希种子变），红的镜像数逐行相同（`opusI` 6、`both` 8 515、`sc_le` / `sc_lt` / `holes_lt` / `app_lt` 0）。

另写 `run_r2_six.py`（输出 `run_r2_six.out`，sha256 48cad850…）把这 6 张逐张列出（深度 6 穷举 376 484 张；原样）：

```text
FAULT 去掉 X、深度 6 穷举：376484 张镜像；opusI（第一轮攻方 V1 写法）红 6 张；v1p（V1 条件 ∧ 压得住的继承子树，攻方的 V3s）红 7 张
  opusI 红： (('W',), ('W',), ('R', ('o', 0)), ('N',), ('F', 4), ('L', 2))  候选集 txg [4, 5, 7]  丢掉的根槽 txg [6]
  opusI 红： (('W',), ('N',), ('R', ('o', 0)), ('N',), ('F', 4), ('L', 2))  候选集 txg [4, 5, 7]  丢掉的根槽 txg [6]
  opusI 红： (('W',), ('R', ('o', 0)), ('N',), ('W',), ('F', 4), ('L', 1))  候选集 txg [4, 6, 7]  丢掉的根槽 txg [5]
  opusI 红： (('W',), ('R', ('o', 0)), ('N',), ('N',), ('F', 4), ('L', 1))  候选集 txg [4, 6, 7]  丢掉的根槽 txg [5]
  opusI 红： (('W',), ('R', ('n', 1)), ('N',), ('W',), ('F', 4), ('L', 1))  候选集 txg [4, 6, 7]  丢掉的根槽 txg [5]
  opusI 红： (('W',), ('R', ('n', 1)), ('N',), ('N',), ('F', 4), ('L', 1))  候选集 txg [4, 6, 7]  丢掉的根槽 txg [5]
```

（每行后面还有「各写法红的单元数」一列：`plain` 3、`sc` 0、`scp` 0、`v1p` 3、`v1p_skip` 0，全 6 行相同，原样见 `run_r2_six.out`。）

读法：6 张是同一件事的变种。回退之后紧接一次建 inode（N），再抬 F 越过回退目标，丢的恰是那条回退根的根槽；N 成了 extent 树继承子树的第一次出现，它的 inode 树是新的（出生 txg = N.txg），V1 的条件（inode 树根指针是继承来的）不成立，落回原措辞判，红。F 取 4 合模型里的上限：4 条不同状态的非空根。表头的 7 是 `v1p` 或 `v1p_all` 红的并集，比 `opusI` 多的那 1 张是 `N W R(o,0) N 抬 F4 丢槽 2`：只在含再次出现的那一半（`v1p_all`）红，同一形状（我单跑核过，草稿脚本不入库）。

### A2　机理对不对得上冻结副本的代码

| 一步 | 代码（冻结副本） | 对得上？ |
|---|---|---|
| 建 inode 的发布不动 extent 树与数据单元 | 「数据单元与 extent 树不动」（`crates/singlefs-core/src/transaction.rs:4848`） | 对得上 |
| 回退只把 extent、inode 两条树表条目的根指针换成目标的，其余条目随现行 | 「if entry.kind == TREE_KIND_EXTENT || entry.kind == TREE_KIND_INODE {」（`crates/singlefs-core/src/mount.rs:5162`）；目标的根指针从「let target_root_of_the_tree_of_kind = |kind: u16| {」（`crates/singlefs-core/src/mount.rs:5148`）取；grep 这个闭包名全仓只有这一处用它；别的建根路径我没逐个读，「只有回退把旧树装进新根」是推的 | 对得上 |
| 根槽读不出或校验不过的根，不在候选集里 | 根环槽只收自证过的：「.and_then(|bytes| check_root_slot(&bytes, &geometry.filesystem_identifier).ok())」（`crates/singlefs-checker/src/image.rs:1001`，函数 `valid_roots` 在 `image.rs:994`）；候选集再按抛弃与 F 滤：「let walked = *index == newest_index || (!abandoned && !below_floor);」（`crates/singlefs-checker/src/walk.rs:6766`） | 对得上。**判决第 2 行把这一步引成 `walk.rs:5977`，那一行（第一轮冻结副本）判的是抛弃与 F，不是读不出；真正的出处是 `image.rs:994` 一带。机理不受影响，引文要改** |
| journal 记录不会把丢了槽的那一版补回候选集 | 补回要四条同时成立，其中一条是实例表里有同实例的行：「row.instance == record.instance」（`crates/singlefs-checker/src/walk.rs:5837`）、「record.checkpoint_txg <= row.published_checkpoint_txg」（`crates/singlefs-checker/src/walk.rs:5838`），另一条「&& !root_slot_is_readable」（`crates/singlefs-checker/src/walk.rs:5846`） | 对得上，**但只在「回退之后没有再经过一次可写挂载」的镜像上**：再挂载一次会给该实例写一行，此后 journal 记录若还没被环绕盖掉，丢槽的那一版会被补回、V1 在那张镜像上不红。6 张里没有挂载（模型里 K 一次都不出现），所以不受影响 |
| 丢一个根槽之后镜像不会因别的原因先红 | 根环健康判只有 I-7.3（有没有更早一代）：「上一代被直接覆盖了，下一次撕裂无路可退」（`crates/singlefs-checker/src/walk.rs:5083`），丢中间一槽不触发 | 推的（读代码，没在入库镜像上造） |
| 故障本身合不合法 | 根环三个区域放在不同失败域里就是为容忍这种丢失（D22 已定项 2「R = 3」），每条根只写一个槽（区域 = txg mod R，已定项 16），所以「单个根槽读不出」是单点故障 | 合法 |

我没在入库装置上造这 6 张的任何一张（不编、不跑 checker 档），所以 V1 在真镜像上红，和第一轮一样仍是「模型上量到、代码上读到机理」。

### A3　放宽之后：不止 6 张，而且补一条不适用就清零

`run_r2_deep.py`：在受限字母表上穷举到深度 8，丢根槽 L 可出现在任意位置、任意多次（`ROLL`：覆盖写、建 inode、四种回退目标、抬 F 到 4..9、丢第 0..3 条；`ROLLKX`：再加可写挂载 K 与抛弃最新根 X）。每个历史前缀一张镜像。写法定义在 `r2_variants.py` 文件头：`sc` / `scp` 是按树继承下界（`scp` 带「压得住」，判决措辞原样），`v1p` 是 V1 的条件加压得住的继承子树（攻方的 V3s 的我的读法），`*_skip` 是给对应写法补「(R⁻, R) 之间有被丢根槽的 txg 就整对不判」。原样（`run_r2_deep_ROLL8.out` sha256 3e7e8c5b…、`run_r2_deep_ROLLKX8.out` sha256 860286b3…；首例一列随进程到达序会变，数不变）：

```text
[深枚举] 字母表 ROLL（16 个操作）深度 8：前缀 28 个；检查的镜像（每个历史前缀一张）12618652
  plain          红的镜像 4757238
  plain_all      红的镜像 11461942
  sc             红的镜像 0
  sc_all         红的镜像 0
  scp            红的镜像 0
  scp_all        红的镜像 0
  v1p            红的镜像 41810
  v1p_all        红的镜像 86940
  scp_skip       红的镜像 0
  scp_all_skip   红的镜像 0
  v1p_skip       红的镜像 0
  v1p_all_skip   红的镜像 0
[深枚举] 字母表 ROLLKX（14 个操作）深度 8：前缀 40 个；检查的镜像（每个历史前缀一张）16307544
  plain          红的镜像 2589801
  plain_all      红的镜像 10982912
  sc             红的镜像 0
  sc_all         红的镜像 0
  scp            红的镜像 0
  scp_all        红的镜像 0
  v1p            红的镜像 12973
  v1p_all        红的镜像 18096
  scp_skip       红的镜像 0
  scp_all_skip   红的镜像 0
  v1p_skip       红的镜像 0
  v1p_all_skip   红的镜像 0
```

（上面两块是两份 `.out` 去掉「首例」「用时」列后的原样行；两份 `.out` 里 `plain` 等各行的数完整。）合计 28 926 196 张镜像：按树继承下界 0 红，V1 系 54 783 张红（`v1p`，两档相加），补不适用后 0。

### A4　给 V1 系补「丢槽不适用」：仓内有先例，但只覆盖「读不出」

- I-7.4 被抛弃根那一半已经这样做：「let some_root_slot_is_unreadable = root_slot_positions(geometry)」（`crates/singlefs-checker/src/walk.rs:5895`），读不出就整条报判不了：「根环有读不出的槽、最新根的实例表里又有行」（`crates/singlefs-checker/src/walk.rs:5901`），理由是那个槽里可能坐着一条这一半要看的根。
- I-7.7 同形：「根环、journal 环或单元区有读不出的槽」（`crates/singlefs-checker/src/walk.rs:2781`）。
- 两处判的都是 `reader.read(…).is_none()`（`crates/singlefs-checker/src/walk.rs:5897`），也就是**读失败**。读到了但自证不过（读坏、撕裂）的槽，`valid_roots` 在 `image.rs:1001` 那一行静默丢掉，仓里没有先例去区分它。要给下界补不适用，得新加「槽读到了、非全零、又不自证」这一类，或按缺的 txg 算出槽位再看（区域 = txg mod R，区域内槽位 = (txg div R) mod S，D22 已定项 16）。**这是比先例多出来的工作，模型上看不见**。
- 另有一条要写明的口径：整对报不适用会让「回退根被丢」的那一对永远不判。这等于承认丢槽的镜像上下界半边没有判别力；按树继承下界不需要这个口径，那一对在它上面是判得出来并且绿的（`sc` 在丢槽那 6 张上 0 红，见 A1）。

### A5　辩方对第 1、2 行的结论

- 「V1 在任一合法镜像上红 ⇒ 出局」这条跑前判据字面打中，**站得住**。
- 打中的是「V1 系的条件只看 inode 树根指针」这一个设计点，不是「继承豁免」这个想法。给它补一条丢槽不适用，在 28 926 196 张模型镜像上 0 红。所以「V1 系救不了」不成立；成立的是「V1 系要多一条先例只覆盖一半的不适用，才不红」。
- 判决第 2 行引的行号要改：读不出的根不进候选集那一步，出处是 `image.rs:994` 一带，不是候选集那一行。机理其余各步对得上。

## B　复核第 4 行：取按树继承下界、不取 V2s / V3s

### B1　判决写的理由现在有数了

判决第 4 行写：V2s、V3s 要求 V1 的条件，「在第 2 行那一格上会照原措辞判、红（推的）」。A3 里的 `v1p` 就是这条的实现（V1 条件成立才免，免只到压得住的继承子树）：红 41 810 与 12 973 张，补不适用后 0。**从推的变成模型上量过的**（`v1p` 是我对 V3s 的读法，攻方原型里实现的 V2、V3 不带 V1 条件，两者不是同一份代码）。

### B2　V1 系比按树继承下界多抓什么

`run_r2_bad.py`（输出 `run_r2_bad.out`，sha256 ca91e195…）：1500 段随机合法历史（API 字母表，无故障，种子 5），在每条非回退候选根 R 的新写单元上把「树根指针出生 txg + 树里新写单元的头」盖小 k：两条链都盖（M2both）、只盖 extent 链（M2x）、只盖 inode 链（M2i）。「天花板外」一列全 0（盖到的数都比 R⁻.txg 小）。原样：

```text
  模式                k     镜像数     天花板外    plain       sc      scp  scp_all      v1p  v1p_all
  M2both            1    4241        0     4241     4241     4241     4241     4241     4241
  M2both            2    4241        0     4241        0        0        0        0        0
  M2both            3    4241        0     4241        0        0        0        0        0
  M2i(只inode链)      1    4241        0     4241     4241     4241     4241     4241     4241
  M2i(只inode链)      2    4241        0     4241        0        0        0        0        0
  M2i(只inode链)      3    4241        0     4241        0        0        0        0        0
  M2x(只extent链)     1    3620        0     3620     3620     3620     3620     3620     3620
  M2x(只extent链)     2    3620        0     3620        0        0        0     3620     3620
  M2x(只extent链)     3    3620        0     3620        0        0        0     3620     3620
```

- k = 1 全部写法都抓（严格小于的功劳，第一轮已有）。
- k ≥ 2：两条链一起盖，所有写法都漏（今天判不出，等谱系记录）；只盖 inode 链，所有写法都漏（V1 的条件只看 inode 链，它盖小了反倒打开豁免）；**只盖 extent 链，V1 系抓、按树继承下界漏**（7 240 / 7 240 对 0 / 7 240）。
- 所以 V1 系的真实长处是「只有 extent 链出错」这一类，而且是对称缺半边的（inode 链单出错它也漏）。一个「两条树都得是继承来的才免」的写法能补上 inode 那半边，但那就是第一轮的 `both`，在建 inode 之后的合法镜像上本来就红（第一轮 API 档 645 张），要同样的丢槽不适用才能用。

### B3　结论

取按树继承下界、不取 V2s / V3s：**站得住**。理由改写成三条：①V1 系要多一条丢槽不适用才不红合法镜像，那条在仓里只有「读不出」的先例（A4）；②V1 系多抓的只是「整条用户可见链改小」那一大类里的「只有 extent 链」一角，这一大类判决已经交给谱系记录（J5 里的改写把它整类收了）；③按树继承下界不需要补丁，丢槽的那一对照常判。

交主 agent 看的一件事：如果更看重「写路径里只有一棵树的出生代号打戳出错」这一类缺陷在谱系记录落地之前就能抓，V1 系加丢槽不适用是一条站得住的备选；这一轮没有任何一条腿攻过它（零轮）。

## C　J5：谱系记录之后的 (b)

### C1　判决措辞不够：「T_rb = R.txg」会红合法镜像

判决第 5 行与第三节写的收法：「R 里有一条 T_rb = R.txg 的谱系记录、树根指针的出生 txg 不大于它的 T_old」（`research/prompts/m3-birth-floor-r1-main-verification.md:32`）。问题在 R 不是回退根自己、而是回退根之后的写：回退根的槽丢了（A1 那 6 张的形状），R = N，N 的谱系树里有那条记录，但记录的 T_rb 是回退根的 txg，不是 N.txg，字面要求 `T_rb = R.txg` 不成立，N 继承来的 extent 树整棵红。

D18 的字段够判这件事：「T_rb 是回退那次发布的 txg，T_old 是 R_old 的 txg」（`.claude/kb/decisions/18-块里携带什么信息.md:569`），谱系树「随现行那一版、只增不删」（同一行），D23 写了回退根的谱系树不取 R_old、随现行：「墓碑树、谱系树随现行那一版，不取 R_old，回退那次发布写一条谱系记录 (T_rb, T_old)」（`.claude/kb/decisions/23-journal的角色与格式.md:377`），所以 N 看得到那条记录。

改成（辩方写法，零轮）：(b) 收成「R 的谱系树里有一条记录 (T_rb, T_old)，R⁻.txg < T_rb ≤ R.txg，且那棵用户可见树的根指针出生 txg ≤ T_old；谱系树读不全时这一对报不适用」。不沿父链走，所以不依赖 C616（谱系父链靠 txg 连号没有断言，`.claude/kb/checks-owed.md:485`）那一步。

### C2　模型上量

`r2_lineage.py`：每条回退根记成 (T_rb = 它的 txg, T_old = 模型里树完全相同的更早非被抛弃根里 txg 最大的那条)；记录随现行那一版往后带。`lin_v` 是判决字面，`lin_fix` 是上一节的改写。深度 8 的 `ROLL` 字母表，12 618 652 张镜像（`run_r2_lineage_ROLL8.out`，sha256 272c38e8…；原样，去掉首例列）：

```text
[J5 谱系记录写法·合法镜像] 字母表 ROLL 深度 8：镜像 12618652
  lin_v          红的镜像 41810
  lin_v_all      红的镜像 86940
  lin_fix        红的镜像 0
  lin_fix_all    红的镜像 0
```

`lin_v` 的 41 810 / 86 940 与 A3 里 `v1p` 的数逐位相同，同一批镜像：字面版在「回退根丢了、N 在后」上的红，就是 V1 出局的那个形状。坏镜像层（`run_r2_lineage.py bad 1500 14 5`，`run_r2_lineage_bad.out`，sha256 eb71b0f8…，同 B2 的盖法）：M2both、M2i、M2x 各 k = 1、2、3，`lin_v`、`lin_fix`、`lin_fix_all` 都是「抓到 = 镜像数」（4 241、4 241、3 620 各三个 k），合计 36 306 / 36 306；按树继承下界在其中 k ≥ 2 的 24 204 张上 0（B2 表）。

**前提写明**：这里的记录是可信的。模型没造「同时伪造一条谱系记录」的坏镜像；谱系树的结构由 I-9.17 判（`.claude/kb/invariants.md:303`，状态「未实现（三·3 实现）」），记录是否与 journal、回退目标对得上还没有一条不变量判。所以谱系版把整条链改小那一洞收掉，是「谱系记录可信」之下的收；要伪造得连记录一起伪造，而伪造的记录没有人核。

### C3　写进 I-3.5 行射程的话（建议）

I-3.5 行今天的状态列「未实现 ⚠️ ② 的下界半出局……」（`.claude/kb/invariants.md:146`）。谱系记录落地之前，射程里写：

> 整条用户可见链（树根指针出生 txg 与树里单元的诞生代号）一起改小的坏镜像判不出：盘上与一次合法回退逐项同形，判别子只有回退那次发布写的谱系记录（D18 已定项 24），今天 `crates/` 里没有（`grep -rnE 'T_rb|lineage_record|谱系记录' --include=*.rs` 0 行）。落地之后 (b) 改成：R 的谱系树里有一条记录 (T_rb, T_old)，R⁻.txg < T_rb ≤ R.txg，且该树根指针的出生 txg ≤ T_old；谱系树读不全时那一对报不适用。

「今天 `crates/` 里没有」我在冻结副本现跑：`grep -rnE 'T_rb|lineage_record|谱系记录' --include=*.rs crates | wc -l` 见末节。

### C4　J5 没答到、也不开第三轮的

- 谱系记录落地后 T_old 是不是总等于被回退到那一版的 txg：D18 写「T_old 是 R_old 的 txg」；模型里我取「树相同的最大 txg 的根」近似，真实的回退目标是其中之一，继承来的树根指针出生 txg 对其中任何一个都 ≤。多次回退叠在一起（第二次回退的目标本身就是回退根）模型里在字母表里有，没单独计数。
- 记录读不全报不适用的口径、与 I-9.18 的「在谱系上」（`.claude/kb/invariants.md:304`）怎么共用同一个读谱系树的函数：属于三·3 的实现，不在这里定。
- `deleted_inodes` 是谱系版 (b) 里第三棵被继承的树，它自己没有判别问题（见 F2）。

## D　替 α（删掉 ①）辩护：β 多判的「再次出现」那一半值不值

### D1　先对齐一件事：判决写下的措辞就是 β 的写法

判决的按树继承下界措辞写「R 引用而 R⁻ 不引用的单元 b 要满足」（`research/prompts/m3-birth-floor-r1-main-verification.md:41`），这是相邻差集，第一次出现与再次出现都落在里面；第二轮正文也这么说：β 出现规则「它就是上面那条措辞本身」（`research/prompts/_m3-birth-floor-r2-body.md:16`）。所以：

- 选 α 不只是在 I-3.5 行里删 ①。② 的下界得写回「第一次出现」的形：I-3.5 ② 原文就是 R_first / R_prev 的形（「设 R_first 为最早者」，`.claude/kb/invariants.md:146`），α = 原 ② + 继承豁免 (b) + 删 ① + 补「回退之后候选集里的 Reach 允许有洞」；β = 把 ①② 重写成相邻差集一句。α 对 I-3.5 行的改动更小。
- 实现形态：β 只要前一条根的引用集合；α 要「更早的候选根引用过的并集」（`seen`）。都是一两行，不构成取舍理由。

### D2　再次出现在合法镜像上不红，所以 β 不比 α 多红

`sc` / `scp`（只判第一次出现，α 的下界半）与 `sc_all` / `scp_all`（相邻差集，β）在 A3 那 28 926 196 张上都是 0 红。第一轮正推腿「出现规则 = ② 的 `sc_lt` ∪ ① 的 `holes_lt`，50 834 张 0 张不同」那一条没被打破。两者没有「合法镜像上一个红一个不红」的差别。

### D3　β 那一半抓什么、别处是不是已经抓了

β 多抓的：非回退根 R 再次引用一个更早候选根引用过、R⁻ 已不引用的单元（诞生代号 ≤ R⁻.txg，又不在继承来的树里）。这类缺陷在仓里先撞上 I-3.9：

- 「若每一条有效根都还引用它，它不许带已释放标志」（`.claude/kb/invariants.md:151`，I-3.9）：旧单元被前面某次发布换下时分配记录已改成已释放，释放代落在 (最后一条仍引用它的根, 最早不再引用它的根] 里。代码按「还引用它的最大 txg」取下界：「let Some(last_referencing_txg) = scanned」（`crates/singlefs-checker/src/walk.rs:3496`），再判「let in_the_witnessed_interval = record.generation > last_referencing_txg」（`crates/singlefs-checker/src/walk.rs:3519`）。非回退根 R 再引用它时，最后一条引用它的根成了 R，记录里的释放代（早于 R）不大于 R.txg，判红。
- 回退不会触发这一条：回退「改回已分配，分配代写回」（D23 已定项 14，`.claude/kb/decisions/23-journal的角色与格式.md:383`），不带已释放标志。
- 单元被回收复用之后再被引用：I-2.1 / I-3.1 / I-5.x 这一族先红（旧单元的头与指针对不上、已分配和对不上）。

所以 β 那一半**只在「复活的单元的分配记录也被一并改回已分配、账也对得上」时才独自判得出**。这是一次手工做平的复活（实质是回退那条路径被非回退发布调用），第一轮正推腿自己写了没有便宜的造法：「① 半边的「只红它」镜像我没有便宜的造法」（`research/prompts/m3-birth-floor-r1-sonnet-output.md:258`）。第二轮正文给下界定的标准是「按树继承下界写不出一份只红它的坏镜像（在入库的池级 checker 上） ⇒ 下界不立」（`research/prompts/_m3-birth-floor-r2-body.md:54`）；β 的这一半按同一标准过不了。

### D4　一个我先以为能支持 α、查清后撤掉的差别

候选集不滤被抛弃的根（例如最新根的实例表读不出，`walk.rs` 注释写「最新根的实例表读不出时一行都没有、不滤」）时，β 红而 α 不红：`run_r2_nonlinear.py`（深度 8，ROLLKX，16 307 544 张，输出 `run_r2_nonlinear_8.out` sha256 9c2118a8…）`sc_all` 与 `scp_all` 红 13 176 张，`sc` 与 `scp` 0 张。再拆单元种类（`run_r2_nonlinear_kinds.py 8`，`run_r2_nonlinear_kinds_8.out`）：

```text
sc_all 红的镜像按单元种类：{'仅 it': 13176}；红的单元按种类 {'it': 13176}（深度 8）
```

13 176 张全是实例表单元，是模型里「抛弃最新根之后不经过可写挂载写行就接着写」才有的情形（真实恢复后新实例第一次发布必写行、重写实例表）。**用户可见单元一个也没红**（继承豁免已罩住）。这个差别是模型的人造物，不用它支持 α，也不当 β 的缺陷。

### D5　辩方对第 9 行的判定

- α **站得住**；β 多判的那一半**不值**（推的：与 I-3.9 重叠，剩下的部分没有只红它的镜像；合法镜像上没有额外代价所以也不是危险）。
- 若主 agent 仍取 β（用户已定 β，第二轮「一并攻」），这一半写进射程时要同时写「它与 I-3.9 重叠，独自能判的只有账也对得上的复活」，不要把它当作 ① 的替代。
- 会推翻「不值」的：一份非回退根再次引用旧单元、而在入库池级 checker 上其余不变量全绿的镜像（只红 β 这一半）。

## E　J2：β 的再次出现在合法镜像上

### E1　结构上

把旧版本的用户可见树装进新根的只有回退（`mount.rs:5148` 闭包的唯一用处在 `mount.rs:5163`）；恢复重基（新实例的第一次发布接在所选的旧根之后）也是「新根的树取自一条更早的根」，但那一对里被抛弃的根按实例表滤掉（`walk.rs:6766`），相邻对是 (所选根, 新实例根)，新根的树就是所选根的树，差集为空。其余发布（覆盖写、建 inode、空发布、挂载写行、暖机）的树都是现行版本的树加本次新写，所以非回退根不可能再引用一个现行版本已经丢下的单元。这一条是读代码推的，没逐个读完所有建根路径。

### E2　模型上

`sc_all`、`scp_all` 在 A3 的 28 926 196 张上 0 红，其中字母表含：任意位置任意多次丢根槽（K 次叠加）、抛弃最新根 X、可写挂载 K、回退四种目标、抬 F 到 4..9。第一轮没取的那三种形状里，被抛弃的根与回退同在（X、R 同在字母表）、根槽读坏的中间根（L）都在里面；journal 环绕盖掉施加版那一种在模型里等价于「候选集里少一条根」，也就是 L 覆盖的形状（A2 第 4 行）。

洞两侧的单元会不会被误判成再次出现：丢掉的根 X 夹在 R⁻ 与 R 之间时，R 引用而 R⁻ 不引用的单元要么是 X 或 R 新写的（诞生代号 > R⁻.txg，(a) 罩住），要么是回退继承来的（(b) 罩住）。模型上这一类 0 红。

### E3　没覆盖到

模型是一个环、一个实例序列；两块盘各自的根环、最新根自己的槽丢了（模型里最新根不可丢，丢它等于历史少一步）、设备失而复得、实例切换重发（`crates/` 里没有）都没取。

## F　J4：措辞写全与代价

### F1　同 txg 并成一条时引用集合怎么并

合法镜像上造不出两条同 txg 的候选根（推的）：新实例的第一次发布取「max(它, 环里全部自证通过的记录的 checkpoint_txg) + 1」（`crates/singlefs-core/src/recovery.rs:1289`），根槽写失败重发时 checkpoint_txg 推进一格再发（D23，`.claude/kb/decisions/23-journal的角色与格式.md:398`）；被抛弃的那一条与新实例的根即使同 txg，前者按实例表滤掉（`walk.rs:6766`）。要撞，得记录与根槽都读不出的双故障。择最新根时有 (txg, 实例) 的二级键（`crates/singlefs-checker/src/walk.rs:6667` 一带），那是择根，不是候选集里的并。

仍要写一句时（零轮，推的）：R 侧逐条各判一次（每条各自对 R⁻），R⁻ 侧取并集；不把 R 侧并起来，免得一条坏根的单元被同 txg 的另一条好根顶上。

### F2　`deleted_inodes` 有根之后，删后回退的镜像怎么判

推的（`crates/` 里今天零根，没有镜像可验）：

- 用户可见树取 D23 已定项 14 写的三棵：「新根的用户可见树（inode 树、extent 树）与 `deleted_inodes` 的根指针取 R_old 那一版的」（`.claude/kb/decisions/23-journal的角色与格式.md:377`）。措辞里的集合写成「树表里种类 1、2、9 三条里根指针非全零的」。
- 回退到删除之前：R_old 的 `deleted_inodes` 根是零（全零指针不算继承树，条件「非全零」罩住），复活的单元走 extent、inode 两棵；回退到删除之后：`deleted_inodes` 根取 R_old 的，出生 txg ≤ R_old.txg < R⁻.txg，走 (b)。
- 墓碑树、谱系树随现行版本（同一行），不在集合里：它们的单元要么在 R⁻ 里（R⁻ 是回退之前的现行版本），要么是这次新写的（诞生代号 > R⁻.txg）；回退根之前的现行根槽丢了、R⁻ 更早时，这两棵树在 R⁻ 与 R 之间新写的单元诞生代号 > R⁻.txg，(a) 罩住。
- 今天回退只换两条：「if entry.kind == TREE_KIND_EXTENT || entry.kind == TREE_KIND_INODE {」（`crates/singlefs-core/src/mount.rs:5162`），与 D23 写的三条不一致；三·删除落地时回退代码要加种类 9。措辞里把 9 放进集合，在回退代码没加 9 的版本上也不红（9 号树随现行，差集为空），所以先写进措辞无害。
- 与 C617（删后回退留下的墓碑没有预付罩着）、C614 同在 D18 已定项 24 的「欠」里（`.claude/kb/decisions/18-块里携带什么信息.md:587`），不是下界的问题。

### F3　「压得住」读不出的单元头算不算压得住

建议：**不计入**。压得住是对继承子树里「每个单元头的诞生代号 ≤ 树根指针出生 txg」的全称量词，头读不出或头校验和不过的单元没有诞生代号可比，该格没有对象——与 I-3.10 的射程 ③ 同一个口径（第一轮正推腿措辞里也是这么写的：「头读不出或头校验和不过的这一格没有对象」，`research/prompts/m3-birth-floor-r1-sonnet-output.md:143`）。理由：①那一格自己会在 I-2.1 一类判定上红，再让整棵继承树失免会把一个坏头放大成整棵树上每个单元一条红；②按「不压得住」算也不会让伪造多一条出路：伪造要的是把诞生代号盖小，读不出头的单元没有被盖小的诞生代号。这一条没有镜像可验（模型里单元头不会读坏），算推的。

### F4　代价

今天 `check_pool_image` 对候选集里每条根都走一遍主走读：「walk.walk_root(&root.record_bytes, false);」（`crates/singlefs-checker/src/walk.rs:6784`），每个被引用单元的诞生代号已经在走读里读出来存进表里：「birth_txg_of_walked_units: BTreeMap<(u32, u64), u64>,」（`crates/singlefs-checker/src/walk.rs:341`）；每一版自己的落点集合也有：「placements_referenced_in_this_version: BTreeMap<(u32, u64), String>,」（`crates/singlefs-checker/src/walk.rs:304`），每版开头清空（`walk.rs:703`）。I-3.9 / I-9.14 另走一遍 `references_of_root`，只在最新根的账里有已释放记录时才走到叶：「let depth = if newest」（`crates/singlefs-checker/src/walk.rs:3641`）。

- 第一轮判决第三节列的路径（`RootReferences` 按树分开落点、去掉那个深度条件让每条候选根都多走一遍）是**贵的那一条**：它在最新根的账里没有已释放记录的状态上，给每条候选根额外加一整趟走到叶。
- 可能便宜的路径（推的，没读到那两张表在版本结束处被谁消费，没实现）：不动 `references_of_root`，在主走读 `walk_root` 的每一版结束时把这一版的落点集合留下（按树表条目标一个来源字段，用来分「落在哪棵用户可见树之下」），相邻差集用留下的前一版集合做；(a) 要的诞生代号查 `birth_txg_of_walked_units`；(b) 与「压得住」的子树成员判定用 `placements_referenced_below_a_walked_unit`（`walk.rs:310`），共享子树被先走过时那张表的并入逻辑已有（`walk.rs:739` 起的 `merge_placements_below_a_unit_walked_by_an_earlier_version`）。这样增量是每个相邻对一次集合差，加回退对上一次读继承子树各单元已在表里的诞生代号，没有第二趟树走读。
- 没量：层 0 每个崩溃状态实际多多少时间。量法交主 agent（checker 档，我不跑）：在层 0 一个流上数「`depth` 落在只读树表」的状态占几成，再对比有无新判定的单状态耗时。**判不了「便宜路径」是否真便宜**，只能说今天的数据结构上看不出必须多走一趟。
- 内存：β 的相邻差集只留前一版的集合；α 的「第一次出现」要累积并集，每版多并一次。同量级。

## G　复跑、文件、推翻条件、没做什么

### G1　复跑

都在仓根跑，纯 Python，线程上限 7（`bash research/scripts/capped.sh 7 nice -n 19 python3 research/prompts/m3-birth-floor-r2-sonnet-model/<脚本> <参数>`）。计数逐次相同；`run_r2_deep.py` 与 `run_r2_nonlinear.py` 的「首例」列随进程到达序会变（计数不变），所以这几份 `.out` 的 sha256 复跑会不同，比数不比 sha。

| 脚本与参数 | 输出 | 用时（本机） |
|---|---|---|
| `run_r2_six.py` | `run_r2_six.out` | 没单独计时（376 484 张，数十秒量级） |
| `run_r2_deep.py ROLL 8 7` | `run_r2_deep_ROLL8.out` | 93 秒 |
| `run_r2_deep.py ROLLKX 8 7` | `run_r2_deep_ROLLKX8.out` | 74 秒 |
| `run_r2_bad.py 1500 14 5` | `run_r2_bad.out` | 没单独计时（远小于一分钟） |
| `run_r2_lineage.py ROLL 8 7` | `run_r2_lineage_ROLL8.out` | 46 秒 |
| `run_r2_lineage.py bad 1500 14 5` | `run_r2_lineage_bad.out` | 没单独计时（远小于一分钟） |
| `run_r2_nonlinear.py 8 7` | `run_r2_nonlinear_8.out` | 87 秒 |
| `run_r2_nonlinear_kinds.py 7` / `8` | `run_r2_nonlinear_kinds_7.out` / `_8.out` | 没单独计时 |

```text
2bae0b6eb443…  birth_floor_v2.py（第一轮正推腿原样拷来）
f5455a54ab88…  r2_variants.py
d313fcb3f92d…  r2_lineage.py
2ec02f2f78f4…  run_r2_bad.py
1a453c37d4a1…  run_r2_deep.py
003ac95a630d…  run_r2_lineage.py
e95046fce502…  run_r2_nonlinear_kinds.py
247bb550a318…  run_r2_nonlinear.py
a558cf873ae6…  run_r2_six.py
ca91e195aed2…  run_r2_bad.out
3e7e8c5bc25f…  run_r2_deep_ROLL8.out
860286b314fe…  run_r2_deep_ROLLKX8.out
eb71b0f8c283…  run_r2_lineage_bad.out
272c38e84fb2…  run_r2_lineage_ROLL8.out
9c2118a8e9e6…  run_r2_nonlinear_8.out
6b95ee8af036…  run_r2_nonlinear_kinds_7.out
f778a2b42963…  run_r2_nonlinear_kinds_8.out
48cad85008ee…  run_r2_six.out
```

深度 9 的 `ROLLKX`（`run_r2_deep.py ROLLKX 9 7`）撞了 `timeout 235` 被杀，没有输出，不引。

冻结副本里「回退不写谱系记录」现跑（原样）：

```text
$ cd /tmp/claude-1000/m3-batch5-freeze/tree && grep -rnE 'T_rb|lineage_record|谱系记录' --include=*.rs crates | wc -l
0
```

冻结副本 `sha256sum -c research/prompts/m3-birth-floor-r2-snapshot/tree-sha256.txt`：OK 3 054 行，不是 OK 的 0 行。

### G2　什么现象会推翻这些结论

- 第 1 行「V1 出局站得住」：6 张里任何一张在入库的池级 checker 上造不出（例如那条回退根的 journal 记录被补回、或别的不变量先红到使「合法」不成立）。第 1 行「V1 系补不适用后 0 红」：深度 8 模型之外找到一张补了不适用还红的合法镜像。
- 第 2 行引文要改：不会被推翻，是 `grep -nF` 可复核的。
- 第 3 行「按树继承下界多出的取舍是 V1 系只多抓 extent 链单改」：找到一类缺陷，V1 系抓、按树继承下界漏，且它不是「整条链改小」的子类。
- 第 4 行 J5：用谱系记录写法在合法镜像上红一张（`lin_fix` 在 `ROLL` 深度 8 的 12 618 652 张 0 红是模型上的）；或谱系记录落地后写出的 T_old 不是 R_old 的 txg。
- 第 5 行 α / β：一份只红 β 那一半、入库池级 checker 上其余全绿的镜像；或合法镜像上 `sc` 绿而 `sc_all` 红的用户可见单元（D4 里那 13 176 张只有实例表单元，不算）。
- 第 6、7 行 J2：一张合法镜像上非回退根再次引用旧单元而没有继承树罩住；模型没取的几种（两块盘各自的根环、最新根自己的槽丢了、设备失而复得）里任一出现。
- 第 8 行 J4：同 txg 两条合法候选根造出来；读不出头的单元造出一份只靠它逃过「压得住」的坏镜像；代价那一条被层 0 上的实测翻掉（便宜路径其实也要多走一趟）。

### G3　没做什么

- 没编译 Rust，没跑 checker 档任何东西，没在入库装置上造任何镜像：所有「V1 红」「按树继承下界 0 红」都是抽象模型上的数；机理对代码是读代码核的（A2）。
- 没判 J1、J3（攻方的格）：没有攻按树继承下界的新形状，没用攻方的原型；没读攻方这一轮的任何产出。
- 没量代价（F4 的数是推的）；没造 I-3.5 谱系版的伪造记录镜像；没验 `deleted_inodes` 那几条（今天零根）。
- 没替主 agent 撤销或维持判决，交的是复核结果。第 3 行末那件「V1 系加丢槽不适用作备选」不是推荐，是交主 agent 看的一个选项。
- 草稿目录 `/tmp/claude-1000/m3-birth-floor-r2-sonnet/` 里只有第一轮模型拷贝、一份输出与一个单跑脚本（共 40K），没有编译目录、工作树或仓副本，没有要删的。

### G4　引文核对

kb 条款与代码行号都是冻结副本里的；报告里引的 `research/prompts/` 下的文件在主仓。末行：

```text
$ python3 research/scripts/cite-check.py research/prompts/m3-birth-floor-r2-sonnet-output.md --root /tmp/claude-1000/m3-batch5-freeze/tree --root /home/fy5090/code/singlefs --background research/prompts/_m3-birth-floor-r2-background.md
  ✓ 核了 31 处引文，对上 31 处，没判 0 处（逐处列在上面）
$ python3 research/scripts/cite-check.py research/prompts/m3-birth-floor-r2-sonnet-output.md
  ✓ 核了 31 处引文，对上 31 处，没判 0 处（逐处列在上面）
```
