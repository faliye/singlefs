# m4-partial-write-r1 正推腿报告（Sonnet，2026-10-26）

背景材料 `research/prompts/_m4-partial-write-r1-background.md`，冻结副本 `~/.local/share/singlefs/work/m4-close-rounds-frozen/`（下称冻结副本）。立场：从 D4 已定项 3、4、C385 用户定案、截断两轮判决与今天的实现推 R2–R7 每一格的候选与它要满足的条件，每格给推荐与依赖的前提。没读攻方腿这一轮的任何产出。

引用约定：kb 条款写 kb 文件名加那份文件自己的行号；代码写仓里路径加行号，**代码行号一律取冻结副本的**。我工作期间别的会话改了现工作区的 `crates/singlefs-checker/src/rebuild.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker/src/lib.rs` 与 `crates/singlefs-core/src/mounted_session.rs`（`cmp -s` 与冻结副本比出不同），这四份在现工作区的行号已与下面不同，以冻结副本为准；其余引到的 `.rs`（`transaction.rs`、`recovery.rs`、`mounted_read.rs`、`unit.rs`、`extent_tree.rs`、`truncate.rs`、`records.rs`、`image.rs`、`fault_injection.rs` 与两份测试辅助 `mod.rs`）冻结副本与现工作区逐字节相同。

## 模型与复跑

模型目录 `research/prompts/m4-partial-write-r1-sonnet-model/`。复跑命令（整条经内存包装，`--work` 给一个新目录）：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-partial-write-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m4-close-rounds-frozen --work <新的草稿目录> --model <仓根>/research/prompts/m4-partial-write-r1-sonnet-model
```

| 文件 | sha256 | 是什么 |
|---|---|---|
| `tools/rerun.sh` | `db9468fdc999c1184e99f7c9d0fef3c33389ac907a524cb0e34ec3adbc76eb65` | 模板拷来，只改 `rerun_steps`：两个小模型脚本、两个 Rust 探针（三条测试）、与 `out/` 逐行比 |
| `ext_and_hole_write_check.py` | `bba8082dd3f672c7a90ac5a1b58d01be23291916026edcc8bde5e4c8768b350e` | 纯 python：只留变长与洞里部分写，今天的三条规则认不认得回活单元（R5） |
| `scan_rule_model.py` | `6de3806013912effaef915d473d2b9b7b7631e894651214f2d5bba798fcc9971` | 纯 python：部分写之后扫描重建几种认领规则认不认得回活单元（R4、R5） |
| `partial_write_cost_probe.rs` | `c6adac5a635a34062e982bdd08c0ad9eaf91b79ca72ce819087bd0519ba13655` | 拷进冻结副本拷贝的 `crates/singlefs-harness/tests/`：量重写一个数据单元与整份重写各发多少角色（第一条），截断意图对 unlink 意图各发多少角色（第二条） |
| `rebuild_mirror_probe.rs.txt` | `f42aacef5ad37d7ae2ef58b53ed36ad32be7c3901b6f9d88edceceb508cef987` | 插进拷贝里 `crates/singlefs-checker/src/rebuild.rs` 的测试模块：R4 的镜像 |
| `out/scan-rule-model-depth10.txt` | `e99f9059d5181c2a20786188fb1571d4cad828fe435ee5c2bdbc779eeba12377` | 小模型输出 |
| `out/ext-and-hole-write-check.txt` | `77489d66686d3c2a41e7822e712229247e3a5fd412bc1f3ab5b161ab8f80bd21` | 补充小模型输出 |
| `out/tombstone-proxy-run.txt` | `60cd359ad1d067322414c8f019867e48b1c735d29383e5355b580ddd1724e345` | 墓碑单价代理量的输出 |
| `out/cost-probe-run.txt` | `3eccc788fdf0f64a1a6f97113a2f5013971c120a138dd00e2d4fc4aa19c5bcb3` | 成本探针输出 |
| `out/rebuild-mirror-run.txt` | `9927443af922b7cce86405fe9906a1780b76d7cd350c1728d6f21dd5b5c1c3c0` | 镜像探针输出 |

两个 Rust 探针都在冻结副本的拷贝里跑（拷贝在 `~/.local/share/singlefs/work/m4-partial-write-r1-sonnet/tree/`，删之前 `du -sh` 是 2.0G，交回前已删），只起 harness 档与 `singlefs-checker` 的测试，没碰 `singlefs-checker-tier`，没起门禁阶段。

## 结论总表（细节在各节，推翻条件在各节末）

| 格 | 一句话推荐 |
|---|---|
| R2 | 读回次数 = 触及单元里净荷没被整单元盖满的个数；有旧单元读旧单元、没有旧单元读新槽残留、残留的字节一个都不用；残留读出错记数不拒写（备选：拒写，代价写在 R2.3） |
| R3 | 读盘（提示位置、再中央映射，整单元 CRC-32C 加头对查找路径），不取内存字节；截断尾单元与变长那条路一起改；同一次发布内同一单元只读一次、后到的写并进内存缓冲 |
| R4 | 今天的读法 Z 加写序在中间单元部分写上会判死后面的活单元（镜像量到）。候选里「字段取代写序」（本腿新起的名字，被攻过零轮）在小模型上全绿且没有写放大；「写到 o 时把 o 之后一起重写」稳妥但写放大随 n−o 线性涨；区间墓碑要翻用户 2026-10-08 的定案；「有洞报不适用」把 I-1.13 在要查的地方关掉 |
| R5 | 起点越过末尾的空隙必须补整单元零（单元从 0 起连号是现行四处判器的前提）；截断字段 T、N_w 部分写不碰，新单元出生 txg 不早于 T，被认领；洞被填满时标签要换 |
| R6 | 先放开「任意 inode、size 不变的部分写」，改 size 的部分写（越过末尾、洞里）只许第一个文件，等 m4-multi-inode-size-r1 |
| R7 | 故障用例十一格（见 R7.1），崩溃流三条；K4 能照搬，但有一个陷阱：计划里带第一个文件的内容就会被当整份重写，把没改的单元一起换下 |

## R2　「首次写」包括哪几格

### R2.1 读到的事实

- D4 已定项 3 的定案句：「首次写不设例外：所有凑不满一个单元的写都走 RMW」（`.claude/kb/decisions/04-校验和位置.md:52`）；射程那句没区分两种读法：「新分配的单元没有旧内容可读」（`.claude/kb/decisions/04-校验和位置.md:56`）。D14 把「首次写不必 RMW」算成收益：「独立 extent 首次写不必 RMW」（`.claude/kb/decisions/14-双轨大小文件-持久临时.md:71`）。C385 要的用例：「断言读回次数与定案一致（0 或 1）」（`.claude/kb/checks-owed.md:271`）。用户 2026-10-26 取「读」，这一轮不再判读不读。
- 今天没有旧单元的几种文件形态：新建的 inode 长度 0、没有数据单元（`publish_new_inodes` 建的，`crates/singlefs-core/src/transaction.rs:9202`）；尾部的洞只由上段叶条目的标签 3、4、0 声明：「只有标签 3、4、0 的 inode 许单元数少于」（`research/prompts/m4-trunc-r1-main-verification.md:39`）。
- 存在的单元必须从 0 起连号，这是写者、冷走读、挂载态读者、池级 checker 共同的前提：「① 存在的数据单元从 0 起连号」（`crates/singlefs-checker/src/walk.rs:13012`），挂载态读者按位次定位：「第 i 项就是文件第 i 个数据单元（位次定位」（`crates/singlefs-core/src/mounted_read.rs:937`）。**没有「中间留洞」这种形态**，洞只在最后一个存在的单元之后。
- 新单元的落点在发布内、分配之后才知道：一次发布先分配落点再装单元，失败时「失败时分配器由调用方退回，这里不管」（`crates/singlefs-core/src/transaction.rs:13224`）；装单元在 `build_file_version_units`（`crates/singlefs-core/src/transaction.rs:12006`）。所以「读新槽的残留」只能放在分配之后、任何写之前。
- 块设备读到的字节没有参照校验和：数据单元的 CRC-32C 住在指向它的父指针的位置条目里，读法是「整单元 CRC-32C 等于条目里的校验和才算读到」（`crates/singlefs-core/src/mounted_read.rs:1074`，实现在 `read_unit_by_location_entries`，`crates/singlefs-core/src/mounted_read.rs:1216`）。新单元还没有父指针，中央映射里也没有它的 key。

### R2.2 格与规则（每格一句）

记号：cap = 32634（净荷，`mounted_read.rs` 里 `payload_capacity_in_bytes`），l = 现存单元数，S = 写之前的 size，L = ⌈S ÷ cap⌉.max(1)。「有没有旧单元」只看元数据（u < l），不看任何一次读的结果。

| 格 | 例 | 有旧单元 | 规则（一句） | 读回次数 |
|---|---|---|---|---|
| 盖满 | 写整整一个单元的净荷（含已有单元被整个盖住） | 不论 | 不是「凑不满一个单元的写」，不 RMW、不发读 | 0 |
| 已有单元 | 3 个单元的文件改中间 4 KiB | 有 | 读旧单元（提示位置，不行再中央映射），整单元 CRC-32C 对位置条目、单元头对查找路径，对不上或读不到整个写失败 | 1 |
| 新文件 | 新建 inode（l = 0、S = 0）写 4 KiB | 无 | 在这次发布给这个单元分到的槽、`locations[0]` 那块盘上发一次整单元读，字节丢掉，旧内容按零 | 1 |
| 文件末尾之外 | 单元号 u ≥ l 且 u 的净荷没被盖满 | 无 | 同「新文件」；末尾单元 l−1 因声明长度要变长而重写时，那一个按「已有单元」读 | 每个没盖满的新单元 1 |
| 洞里 | l ≤ u < L（标签 3、4、0） | 无 | 同「新文件」；读洞本身不发盘读（`crates/singlefs-core/src/mounted_read.rs:1022`「这一段答 0、不读盘」），写洞的 RMW 才发新槽那一读 | 每个没盖满的新单元 1 |
| 空隙 | 起点越过 size 或洞，u 之前的单元 | 无 | 为守「从 0 起连号」补整单元零，这是系统合成的整单元，不是用户的「凑不满」，不发读（空隙单元自己带的写序与出生 txg 同这次发布） | 0 |

读回次数 = 触及单元里净荷没被整单元盖满的个数。C385 的用例照这条定数：新文件写 4 KiB = 1；在 3 个单元的文件上跨单元边界的 4 KiB（两个单元都没盖满）= 2；整单元覆盖 = 0；在文件末尾紧接着追加 4 KiB（末尾单元已满）= 1（新单元的残留读）。

### R2.3 读了残留之后「校验不过」怎么办

D4 已定项 4 的失败条件写的是「旧单元读不回来、或读回来 MAC 验不过时，那个部分写失败并返错」（`.claude/kb/decisions/04-校验和位置.md:70`）。它的触发物是「旧单元」。残留不是旧单元：没有父指针、没有映射 key、没有任何一个参照校验和，「验」无从谈起；按单元头结构去验残留也没有意义（残留可能是别的文件放掉的合法旧单元、半截单元、零、随机字节，都与这次写无关）。**所以残留不进 D4 已定项 4 的射程，用在残留上不会让首次写必失败**（推的：没有「残留验不过 ⇒ 失败」这一条，首次写只剩「读发出了」这一个可判的事实）。

残留读的结果有三个处理候选：

- 丢弃字节、读出错也不拒写（推荐）：理由两条——完整性上没有东西可保护（字节不用）；可用性上，被读的槽是马上要被这次写盖掉的空闲槽，一个读不出的空闲扇区在写之后就是好的，拒写会让带潜在坏扇区的空闲槽挡住所有会分到它的写（推的：分配器退回后重试是否会再分到同一个槽，我没读分配器的选槽顺序）。
- 读出错拒写：与 D4 已定项 4 的「读不回就失败」字面最近，代价是上面那条可用性风险；选它要在 `publish_admitted` 里把残留读放在分配之后、写之前，出错走已有的「分配器由调用方退回」。
- 不发读：用户已否。

**残留的字节一个都不许进新单元**，这条才是首次写读回唯一有牙的性质：新单元的净荷以外、声明长度之后的补齐必须是零（I-2.3），旧内容部分必须是零。判法见 R7.2「毒化空闲槽」。

### R2.4 依赖的前提与推翻条件

- 依赖：分配在装单元之前完成（`crates/singlefs-core/src/transaction.rs:13231` 起的 `publish_admitted`）；「单元从 0 起连号」不变（否则「空隙」一格要改成洞）。
- 推翻：若用户要的「读」本意是发现点什么（比如发现目标槽已坏），残留读丢弃字节就什么也发现不了；这时要改成「读出错拒写」并接受可用性风险，或另立一条写后读回验证（不在这一轮）。若分配器退回后换了槽，可用性风险不成立，两种处理可以任选。

## R3　读回旧单元读哪里

### R3.1 读到的事实

- 截断尾单元今天读内存：「let bytes = &previous.unit(TransactionUnit::Data(rewrite.unit)).bytes;」（`crates/singlefs-core/src/transaction.rs:13189`）。入口在任何写之前先调一遍，读不出就拒：「payload_of_the_rewritten_tail_data_unit(previous, rewrite)?;」（`crates/singlefs-core/src/transaction.rs:10567`）；装单元时再取一遍（`crates/singlefs-core/src/transaction.rs:13460`），两次都不碰设备。
- 内存里的字节只有两个来源：本进程上一次发布自己装的（从没读回过），和挂载时从盘上读的：重建出的版本按数据指针逐个读单元字节（`crates/singlefs-core/src/recovery.rs:4535` 的 `.zip(data_unit_contents.first_file_in_file_order)`），读法是提示位置、再中央映射（`crates/singlefs-core/src/recovery.rs:815` 的 `read_data_unit_via_hint_then_central_mapping`）；两条都读不出的单元字节是空的：「要重写的尾单元在这一版里没有字节」（`crates/singlefs-core/src/transaction.rs:5556`），成员 `TailDataUnitBytesNotInTheVersion`（`crates/singlefs-core/src/transaction.rs:5558`）。
- 挂载态读者 `read_at`（`crates/singlefs-core/src/mounted_read.rs:978`）的读法：`dereference_data_unit`（`crates/singlefs-core/src/mounted_read.rs:1076`）先按父指针的两条位置条目逐条试（整单元 CRC-32C 对得上才算读到），都不算数再查中央映射重试一遍并记多跳；读到之后 `check_data_unit_against_the_lookup_path`（`crates/singlefs-core/src/mounted_read.rs:1130`）核单元头的五元组、出生身份、锚点偏移、声明长度。这一整条挂在挂载态（`OpenFileForRead`）上，发布路径不能直接调。
- 发布路径手里有能读盘的东西：写入口的 `devices` 是 `[(DeviceIdentity, Device)]`，它本身实现读盘口子：「impl<Device: BlockDevice> PoolReader for [(DeviceIdentity, Device)] {」（`crates/singlefs-core/src/recovery.rs:304`），`publish_first_file` 已经拿它读过树表（`crates/singlefs-core/src/transaction.rs:9003`）。上一版的中央映射条目在内存里，释放路径就是这么取位置的（`crates/singlefs-core/src/transaction.rs:4993` 的 `mapping_locations_of_a_released_unit`）。
- 释放路径已经在盘上读被换下的单元：`copies_failing_the_release_checksum_check`（`crates/singlefs-core/src/transaction.rs:5086`），「只读，在动分配器、发任何一个写之前调」（`crates/singlefs-core/src/transaction.rs:5078`）；但它的失败口径是隔离那一份、发布照成，与 D4 已定项 4 的「读不回就失败」相反，不能当 RMW 的读。
- 故障注入设备看得见的只有块设备调用：`ReadFails`（`crates/singlefs-harness/src/fault_injection.rs:148`）与 `ReadReturnsCorruptedBytes`（`crates/singlefs-harness/src/fault_injection.rs:150`）都作用在 `BlockDevice::read_at` 上。

### R3.2 两种读法逐项比

| 项 | 取内存里上一版字节 | 读盘（提示位置、中央映射、验 CRC 与头） |
|---|---|---|
| 注入的读故障看得见吗 | 看不见：这条路不发设备读。C57 的验法是「判据是那个写必须失败」（`.claude/kb/checks-owed.md:54`），同一行还要求「不注入故障时同一个写必须成功」（`.claude/kb/checks-owed.md:54`）；内存读法下前一半恒红 | 看得见 |
| 与验收句 | 验收写「坏旧单元与读故障时请求必须失败，不能零填充伪造成功，补齐与校验仍成立」（`.claude/kb/milestone/04-fourth-txn.md:200`），内存读法满足不了 | 满足 |
| 唯一能失败的时点 | 挂载那一刻（字节为空 ⇒ `TailDataUnitBytesNotInTheVersion`）；挂载之后设备再坏、写丢了，都不会报 | 每次部分写 |
| 本进程自己写的单元 | 读到的是「我打算写的」，从没读回过 | 读到设备上真有的；说谎设备吞了上一次的写时，CRC 对不上，这次写失败 |
| 开销 | 零读；但每次发布把全部单元字节克隆一遍（`carried_file_version_units`，`crates/singlefs-core/src/transaction.rs:13200` 起） | 每个没盖满的已有单元 1 读（健康时） |

### R3.3 同一次发布里对同一单元第二次部分写，各读到哪一版

- 今天：一次改动一次发布。会话每次改动先把队列走空（`crates/singlefs-core/src/mounted_session.rs:1758` 的 `publish_the_queue_until_it_is_empty`），再各发各的（`crates/singlefs-core/src/mounted_session.rs:1778` 的 `publish_the_change_once`），同一次发布里不会有第二次写（m4-trunc-r2 判决也这么记：「R7 攒批欠账」（`research/prompts/m4-trunc-r2-main-verification.md:51`），攒批今天没落地）。
- 跨发布：第二次部分写读到第一次发布的结果。内存读法读到的是第一次装的字节；读盘读法读到设备上的，两者相同当且仅当第一次的写真落了盘。
- 攒批落地之后（D16 已定项 10）：读盘读法读的是发布之前的那一版，同一次发布里先到的写在盘上还没有，直接各自读盘再改会丢更新。规则：**同一次发布里同一单元只读一次，后到的写改内存里合并后的缓冲**，不再读盘。内存读法天然合并，读盘读法要在发布计划里建「单元 → 合并缓冲」。这一条今天不可测（造不出），记为攒批那一批的前提。

### R3.4 推荐

读盘。具体：

1. 在 `publish_size_change` 已有的「任何写之前读得出」位置（`crates/singlefs-core/src/transaction.rs:10567`）加一个读盘函数，签名取「上一版 + 单元号 + 写入口的设备」，读法照挂载态读者：提示位置→中央映射，整单元 CRC-32C 对位置条目，单元头对上一版内存里的指针与记录（出生树、对象、对象出生代、fsid、出生 txg、写序、锚点偏移、声明长度）。可以抽 `read_data_unit_via_hint_then_central_mapping` 与 `check_data_unit_against_the_lookup_path` 成两边共用的函数，不另写第三份读法。
2. 字节在计划阶段读到之后随计划交给装单元那一步，不在 `crates/singlefs-core/src/transaction.rs:13460` 再取一次（省一次读，也避免两次读之间设备变了）。
3. 读在任何分配和写之前，失败返类型化错误，盘上逐字节不变（与现有 10565 行注释同口径）。
4. 释放路径读被换下单元的那几读（`crates/singlefs-core/src/transaction.rs:5086`）照旧；所以一个已有单元的部分写会在这个单元的位置上发三读：RMW 一读加释放核两份。**读回次数用例不能只数「某地址被读了几次」**，要钉发布里发出的读的完整次序（R7.2）。
5. 可选的交叉核：内存字节非空而与读盘字节不同时失败（抓说谎设备与本进程的装配 bug）。多一种失败，想要的话另记，不是 D4 已定项 4 的要求。

### R3.5 截断尾单元（与变长）那条路要不要一起改

要。理由：D4 已定项 4 的失败条件没有给截断留例外；D4 已定项 3 的射程自己写了「截断尾单元的专用读改写不能代替通用部分写」（`.claude/kb/decisions/04-校验和位置.md:57`）；留着两种读法等于同一个「读旧单元再改」有两套失败口径。要付的账：`publish_size_change` 多一个设备读依赖和 `TailDataUnitBytesNotInTheVersion` 换成读盘失败的成员；截断相关用例在健康设备上不变（读盘读到的就是内存里那份），要补坏读用例。

### R3.6 依赖与推翻条件

- 依赖：发布路径能在计划阶段拿写入口的 `devices` 当读口子（`crates/singlefs-core/src/recovery.rs:304` 的实现，`crates/singlefs-core/src/transaction.rs:9003` 已有先例）；上一版内存里的中央映射条目够回退用（释放路径已这么用）。
- 推翻：若 `devices` 在计划阶段已被别的借用占着读不了（我没在编译器里验过借用），需要把读提前到会话层；若测量发现读盘读法让一次部分写多出的读时延不可接受，则只能退成「内存字节加 CRC 交叉核」，那种读法仍看不见读故障，验收句要改。

## R4　截断判决 K6「读法 Z 加写序」的前提重开

### R4.1 读到的事实

- K6 原文给了重开条件：「前提是活单元的出生 txg 随偏移不降」（`research/prompts/m4-trunc-r1-main-verification.md:40`）、「（pwrite、punch）时这条要重开，记欠账」（`research/prompts/m4-trunc-r1-main-verification.md:40`）。m4-trunc-r2 保留了它：「R4 写序那一条保留（K6 第三条），第四条与它是合取。」（`research/prompts/m4-trunc-r2-main-verification.md:33`），并记了以后的入口：「pwrite、punch 进来时 R3 与写序那一条都要重开」（`research/prompts/m4-trunc-r2-main-verification.md:52`）。
- 代码侧：「前提是活单元的出生 txg 随偏移不降」（`crates/singlefs-checker/src/rebuild.rs:615`）。算法：每个偏移之下的最大出生 txg 表 `newest_birth_txg_below_each_data_unit`（`crates/singlefs-checker/src/rebuild.rs:580`），认领时「.is_some_and(|newest_below| *newest_below > birth_txg_of(version))」（`crates/singlefs-checker/src/rebuild.rs:800`）就不认领；第四条是「if truncation_field_txg != 0」（`crates/singlefs-checker/src/rebuild.rs:812`）。同一个偏移上几版按写序择最新：对象身份取单元头 42..75，含锚点偏移：「object_key: 42..75,」（`crates/singlefs-checker/src/lib.rs:997`）。
- I-1.13 判的是三张表逐项相同：「扫描重建与遍历方向的索引相同」（`crates/singlefs-checker/invariants/01-块自描述.md:19`）。比对在 walk.rs 末尾：重建一侧由「rebuild_index_tables(&current, &container_copies)」（`crates/singlefs-checker/src/walk.rs:16244`）算出，数据单元集合一格是「table: IndexTableOfTheScanRebuild::DataUnitsOfEachInode,」（`crates/singlefs-checker/src/walk.rs:16299`），任何一格有差就红：「let every_table_agrees = tables.iter().all(|table| table.differences.is_empty());」（`crates/singlefs-checker/src/walk.rs:16322`）。I-1.13 只在 CPU 一侧，不上 GPU：「S4 I-1.13 留在内核不置位」（`research/prompts/m4-trunc-r1-main-verification.md:86`）。
- 今天没有任何入口能造出「中间单元出生更晚」的合法镜像：`grep` 不到按偏移写的发布入口（背景材料第二节的观测，我用 `grep -n 'pub fn publish_' crates/singlefs-core/src/transaction.rs` 复核过，入口只有整份写、按 inode 号整份写、改 size 一族）。所以镜像只能在扫描重建这一层造。

### R4.2 镜像（部分写之后只走扫描重建）

做法：在冻结副本的拷贝里，往 `rebuild.rs` 的测试模块加一条（`research/prompts/m4-partial-write-r1-sonnet-model/rebuild_mirror_probe.rs.txt`，`crates/singlefs-checker/src/rebuild.rs` 里已有的 `rebuilt_from`、`data_unit_copy`、`inode_container_holding` 三个测试辅助照用），三个单元的第一个文件（size 罩三个单元，偏移 0、32634、65268），出生 txg 取 [3,5,3] 等几组，打出重建认领了哪几个偏移。命令（经内存包装，harness 档以外只起 `singlefs-checker` 的 lib 测试一条）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-checker 的 lib 测试 rebuild 里正推腿草稿的一条; 范围=1 个测试; 为什么=在冻结副本拷贝上造部分写之后只走扫描重建的镜像' nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G cargo test --offline -p singlefs-checker --lib -- probe_a_partial_write_in_the_middle_unit --nocapture
```

原样输出（`research/prompts/m4-partial-write-r1-sonnet-model/out/rebuild-mirror-run.txt` 里 `name=` 行与结果行）：

```
name=partial_write_middle_only births=[3,5,3] claimed=[0, 32634]
name=partial_write_first_only births=[5,3,3] claimed=[0]
name=partial_write_last_only births=[3,3,5] claimed=[0, 32634, 65268]
name=partial_write_middle_and_tail_rewritten births=[3,5,5] claimed=[0, 32634, 65268]
name=partial_write_first_and_all_after_rewritten births=[5,5,5] claimed=[0, 32634, 65268]
name=unit_offsets k=32634
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 111 filtered out; finished in 0.01s
```

读法：births 三个数是单元 0、1、2 的出生 txg，`claimed` 是重建认领的偏移。

- `[3,5,3]`：在中间的单元 1 上做了一次部分写（txg 5），单元 2 没动（txg 3）。重建只认领 0 与 32634，**偏移 65268 的单元 2 是活的却没被认领**。遍历方向从 extent 树读出的集合里有它，I-1.13 的「数据单元集合」一格有差 ⇒ 红（红的是这两个事实的合取；我没有整镜像，因为没有合法入口能造它，这一步标「推的」）。
- `[5,3,3]`：单元 0 上做部分写，后面两个活单元全被判死。
- `[3,3,5]`：只改最后一个单元，全认领——这正是今天截断尾单元、变长重写尾单元的形状，所以今天的前提没被打破。
- `[3,5,5]`、`[5,5,5]`：把 o 之后的单元一起重写（同一个出生 txg），全认领，写序那一条不动就够。

### R4.3 小模型（候选能不能成立的判别）

`research/prompts/m4-partial-write-r1-sonnet-model/scan_rule_model.py`：把文件抽象成「现存单元数 l、size（半个单元为步长，奇数在单元中间、偶数在边界）、每个位置最新拷贝的出生 txg、截断字段 (T, N_w)」，入口取现行的整份写 / 截断 / 变长，加本腿要加的部分写与洞里部分写（补零单元一起写）。放掉的单元拷贝留在盘上（最坏）。扫描规则取 rebuild.rs 的三条（size、写序、截断字段）加候选。每次都与真值（位置 0..l−1 为活）比，广度优先到深度 10、按状态去重，全部可达状态逐个比。

必红自证（模型会不会红）：今天的入口、今天的规则 0 红；去掉截断字段那一条红 924 个状态，例 `whole 3 → trunc 2 → ext 3`，正是 m4-trunc-r2 的题面；去掉写序那一条红 1796 个状态，例 `whole 3 → whole 0 → ext 3`，正是 m4-trunc-r2 R4 说「整份写变短再变长只有它罩得住」的那一形。模型复现了两轮判决各自的反例，说明它分得出对错。

原样输出（`research/prompts/m4-partial-write-r1-sonnet-model/out/scan-rule-model-depth10.txt`，`variant=` 与 `selfcheck=` 行）：

```
selfcheck=today_ops_today_rule(should_be_0) depth=10 states=10482 mismatches=0
selfcheck=today_ops_without_field_clause(should_be_gt0) depth=10 states=10482 mismatches=924
selfcheck=today_ops_without_order_clause(should_be_gt0) depth=10 states=10482 mismatches=1796
variant=today_rule_pw_only depth=10 states=26109 mismatches=3123
variant=today_rule_pw_tail_rewrite depth=10 states=20244 mismatches=0
variant=field_only_whole_always depth=10 states=28393 mismatches=0
variant=field_only_whole_if_rel depth=10 states=25227 mismatches=0
variant=field_only_whole_clear depth=10 states=26109 mismatches=2722
variant=field_only_whole_clear_unless_released depth=10 states=25852 mismatches=1400
variant=range_tombstone depth=10 states=26109 mismatches=0
```

变体名：`today_rule_pw_only` = 今天的三条规则、部分写只重写那一个单元；`today_rule_pw_tail_rewrite` = 同规则、部分写把 o 之后的单元一起重写；`field_only_whole_always` = 去掉写序那一条，整份写总写字段 (t, m)；`field_only_whole_if_rel` = 去掉写序，整份写放了单元才写字段 (t, m)、没放就照抄；`field_only_whole_clear` = 去掉写序、整份写清零（今天的写法）；`field_only_whole_clear_unless_released` = 去掉写序，整份写放了单元就写、没放就清零；`range_tombstone` = 每次放单元写一条区间墓碑 (t, 起点)，规则只看墓碑。

得到的：(1) 今天的规则加「只重写一个单元」的部分写：3123 个状态红，最短例子两步（`whole 3`、`pw 0`）。(2) 「尾重写」0 红。(3) 去掉写序并让整份写写字段：`always` 与 `if_released` 0 红；`clear` 与 `clear_unless_released` 红——也就是去掉写序的前提是**整份写不能清零**。(4) 区间墓碑 0 红。

模型没覆盖的（所以只是线索）：快照与回退（谱系过滤）、多个实例、同一 txg 装几次改动、崩溃留下的未发布单元、两个 inode 交错；单元数上限 4。

### R4.4 候选各要改哪几处、写放大多少

记号：n = 文件现存单元数，o = 部分写落在的单元号（0 起），底数 = 「重写一个数据单元」今天的角色数（见下）。

**成本探针（量的）**：`research/prompts/m4-partial-write-r1-sonnet-model/partial_write_cost_probe.rs` 在冻结副本拷贝上量一次发布写出多少角色。「尾单元重写」用 `publish_size_change` 变长（只重写最后一个单元，别的照抄），「整份重写」用 `publish_sequential_write`。原样输出（`name=` 行，`rewritten` 是这次发布写的角色数，`records` 是 journal 记录数，`released` 是换下的落点数）：

```
name=whole_write_first units=1 rewritten=12 records=1 released=12 roles=datax1,t*x11
name=tail_unit_rewrite units=1 rewritten=12 records=1 released=12 roles=datax1,t*x11
name=whole_rewrite units=1 rewritten=12 records=1 released=12 roles=datax1,t*x11
name=whole_write_first units=3 rewritten=15 records=3 released=12 roles=datax3,extent_lowerx1,t*x11
name=tail_unit_rewrite units=3 rewritten=13 records=1 released=13 roles=datax1,extent_lowerx1,t*x11
name=whole_rewrite units=3 rewritten=15 records=3 released=15 roles=datax3,extent_lowerx1,t*x11
name=whole_write_first units=144 rewritten=158 records=144 released=12 roles=datax144,extent_lowerx1,t*x13
name=tail_unit_rewrite units=144 rewritten=15 records=1 released=15 roles=datax1,extent_lowerx1,t*x13
name=whole_rewrite units=144 rewritten=158 records=144 released=158 roles=datax144,extent_lowerx1,t*x13
name=whole_write_first units=145 rewritten=161 records=145 released=12 roles=datax145,extent_lowerx3,t*x13
name=tail_unit_rewrite units=145 rewritten=16 records=1 released=16 roles=datax1,extent_lowerx2,t*x13
name=whole_rewrite units=145 rewritten=161 records=145 released=161 roles=datax145,extent_lowerx3,t*x13
name=whole_write_first units=288 rewritten=306 records=288 released=12 roles=datax288,extent_lowerx3,t*x15
name=tail_unit_rewrite units=288 rewritten=17 records=1 released=17 roles=datax1,extent_lowerx2,t*x14
name=whole_rewrite units=288 rewritten=308 records=288 released=306 roles=datax288,extent_lowerx3,t*x17
```

读法：n = 1、3、144、145、288 时，重写一个数据单元（`tail_unit_rewrite`）写 12、13、15、16、17 个角色、1 条记录；整份重写 n 个单元（`whole_rewrite`）写 12、15、158、161、308 个角色、n 条记录（一个数据单元一个事务一条记录：「切分纪律切出的每个一单元事务装一个数据单元」（`crates/singlefs-core/src/transaction.rs:12000`））。角色里「data」是数据单元，「extent_lower」是下段节点（一片叶罩 144 个单元），「t*」是固定点那几样（inode 叶容器、根、记账与映射树节点、树表等，随池长大从 11 涨到 15–17）。所以**底数 = 1 个数据单元 + 1 到 2 个下段节点 + 11 到 15 个固定角色**，与 n 几乎无关。

#### 候选甲：保留读法 Z 加写序，写到 o 时把 o 之后的单元一起重写

- 改动点：(1) 新发布入口（`transaction.rs` 里 `publish_overwrite_of_an_inode` 一族旁边，`crates/singlefs-core/src/transaction.rs:9270`），计划里数据角色 = o..l−1；(2) 读 o..l−1 每个单元的旧字节（R3 的读盘，n−o 读）；(3) 释放路径的「整份重写」判断必须改，见 R7.3 的陷阱；(4) 扫描、checker、GPU 判器、D18 条文都不动。
- 写放大：数据单元 n−o 个、journal 记录 n−o 条、下段叶约 (n−o)/144 片，加固定角色；读放大 n−o 读。端点已量：o = n−1 时 = 尾单元重写（13–17 个角色），o = 0 时 = 整份重写（n = 288 为 308 个角色）；中间点是这两个端点按线性补的（推的：我只量了端点）。均匀分布的 o 期望重写 (n+1)/2 个单元。n = 33 000（约 1 GiB）、o = 0 时一次 1 字节的写要写 33 000 × 32 768 B = 1 081 344 000 B。
- 怎么量：把成本探针里 `whole_rewrite` 换成 o..n−1 的重写（需要新入口；入口出来之前用公式 `(n−o) + ⌈n/144⌉ − ⌊o/144⌋ + 内部节点 + 固定角色` 对端点），取样点取 n ∈ {1, 3, 144, 145, 288, 1000}、o ∈ {0, n/2, n−1}——两条臂在 o = 0 与 o = n−1 上差别最大。
- 判：不要求改格式、被攻过零轮的读法 Z 原样用，稳；但通用部分写的意义（只付改的那一点）没了，写放大不低于今天的整份写。

#### 候选乙：区间墓碑

- 改动点：(1) 翻用户 2026-10-08 的定案「truncate / punch 这类部分删除不写区间墓碑（区间墓碑不做）」（`.claude/kb/decisions/18-块里携带什么信息.md:237`）；(2) 每个放掉单元的入口（截断意图、整份写变短）写一条区间记录——今天截断意图那次发布墓碑树整棵照抄（`TombstoneTreeAfterThisPublish::carried`，`crates/singlefs-core/src/transaction.rs:8114`），要改成追加；(3) `rebuild.rs` 里墓碑只认整对象：`TombstoneRangeOtherThanTheWholeObjectOfTheFirstVersion`（`crates/singlefs-checker/src/rebuild.rs:145`）的交回要换成按区间判，删掉读法 Z 的写序与截断字段两条（`rebuild.rs:795` 到 `:820` 一段）；(4) 池级 checker 的墓碑几判（`crates/singlefs-checker/src/walk.rs:99` 起的 `TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED`）与 I-4.10、I-9.39、截断字段（m4-trunc-r2 R1）整套要重新对账；GPU 恢复内核里墓碑怎么读我没读（未核）。
- 写放大：部分写 = 底数（不放单元就不写墓碑）；每个放单元的入口多一条 40 字节记录、墓碑树永久涨（墓碑第一版不回收，依据 `crates/singlefs-core/src/transaction.rs:4020` 注释）。每条记录多出的角色数用代理量了一次，见下。
- 怎么量与量到的：代理量——同一个三单元的池里 `publish_size_change` 截断意图那次（不碰墓碑）的角色数对 `publish_unlink_intent`（追加墓碑）那次的；脚手架 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `publish_the_intent` 与 `crates/singlefs-harness/tests/common_truncation/mod.rs:62` 的 `change_the_size_of_the_first_file_and_advance`。原样输出（`research/prompts/m4-partial-write-r1-sonnet-model/out/tombstone-proxy-run.txt`）：

```
name=truncation_intent_into_unit_zero units=3 rewritten=14 records=1 released=13 roles=datax1,extent_lowerx1,t*x11,tlo*x1
name=unlink_intent_appends_a_tombstone units=3 rewritten=13 records=1 released=10 roles=t*x10,td*x1,tt*x2
```

  读法：unlink 意图写的角色里 `tt` 两个是墓碑树的叶容器与根（`tag()` 映射：「TransactionUnit::TombstoneLeafContainer(index) => format!("tt@{}", index.0),」（`crates/singlefs-core/src/transaction.rs:2779`）是墓碑叶容器，`tt` 是墓碑根），`td` 是 deleted_inodes 根（「TransactionUnit::DeletedInodesRoot => "td".to_string(),」（`crates/singlefs-core/src/transaction.rs:2778`））——所以**往墓碑树追加一条记录在小池上多写 2 个角色（墓碑叶容器加墓碑根）**，约占一次意图发布 13–14 个角色的七分之一；墓碑树长到多层后会多（没量），这是小池上的代理，不是乙真实现的数。
- 判：小模型 0 红，理论上最完整；它是 m4-trunc-r2 R8 点名的、pwrite 与 punch 都来之后唯一站得住的形态。代价是翻一条用户定案加整套墓碑判器重做。

#### 候选丙：有洞的盘报不适用

- 改动点：扫描一侧要知道「这个 inode 做过部分写」。扫描方向读不到上段叶条目的标签：「扫描方向不读 extent 树，手里没有上段叶的标签，判不出单元 0 已死」（`.claude/kb/checks-owed.md:594`）。所以要在 inode 记录里加一个黏住的记号（偏移 108–119 与 136–139 今天恒 0，I-9.7 判；占用就要再动一次格式与 incompat 位），`rebuild.rs` 对带记号的 inode 交回 `NotApplicable`（`crates/singlefs-checker/src/walk.rs:16202` 已有这条出口）。
- 写放大：= 底数。
- 判：不推荐。I-1.13 恰好在被部分写过的 inode 上关掉，而这一轮的验收（补齐与校验仍成立）和以后的 punch 都靠它；还得多一个格式位。

#### 候选丁：先建计数实验

- 不是改动，是选甲、乙、戊（下）之前的闸。形态照 E109、E110：纯算术、钉绝对值、变异表、进 `replay.sh`。要两样输入：甲与底数——上面探针已量；乙每条记录多出的角色数——上面代理量，没量。两条臂真不同的取样点：o = 0 与 o = n−1（甲差 n 倍）、截断频率高的负载（乙差在墓碑条数）。
- 本腿的结论：甲的账已经够清（线性、可算），不需要为它再建实验；要建实验是因为乙的单价没数。

#### 候选戊（本腿新起的名字）：字段取代写序

- 内容：读法 Z 去掉第三条（写序），只留 size、对象出生代、截断字段 (T, N_w) 三条；整份写（顺序写、覆盖写）**不再清零**，改成「总写 (这次 txg, N_w = 整份写后的 size)」（模型里 `always`，小模型里的 m 是 ⌈N_w ÷ 净荷⌉.max(1)）或「放了单元才写、没放照抄」（`if_released`）；部分写、洞里部分写不碰字段（与变长同口径：`crates/singlefs-core/src/transaction.rs:10511` 的变长一支照抄 `record.truncation_field`）。
- 改动点：(1) `crates/singlefs-core/src/transaction.rs:8964` 整份写造记录的地方（`inode_record_of_file_version`，第一个文件与别的文件的整份写共用这一处；`:9233` 是建 inode 的，不改），`NOT_WRITTEN` 换成按上面规则算；(2) `rebuild.rs` 删 `newest_birth_txg_below_each_data_unit`（`:580`）与 `:795` 到 `:802` 那一段，注释 `:615` 重写，单测 `a_data_unit_born_before_a_data_unit_at_a_smaller_offset_is_not_claimed`（`:1688`）翻成「中间单元部分写后面的活单元仍被认领」；(3) D18 已定项 10 读法 Z 三条改成两条加字段、I-1.13 条文（`01-块自描述.md:19`）跟着改；(4) 判决 m4-trunc-r2 第六节用户定案里「整份写清零」那一句要用户重定：「第五节第 1 条选「放了单元才写、整个替换」…整份写清零（R2 的整份写那一句取清零）」（`research/prompts/m4-trunc-r2-main-verification.md:69`）——**这是用户亲定的一点**；(5) 池级 checker 与 GPU 判器不用改（I-1.13 只在 CPU 一侧；I-9.39 判「T ≠ 0 时字段之外的活单元不早于 T」，整份写之后字段之外没有活单元，仍成立）。
- 写放大：= 底数，没有额外账。
- 判：模型里 `always`、`if_released` 在深度 10、每个可达状态 0 红；`clear` 红（见 R4.3）。**被攻过零轮，只在本腿自己的小模型上量过**（证据纪律：这类「攻方腿自己提的收严」算没被攻过，交用户时要标）。

### R4.5 推荐、依赖、推翻条件

推荐：**候选戊（字段取代写序）**为首选，**候选甲（尾重写）**为不碰格式的退路，乙留给 punch（里程碑十一，C778）。排序理由：

1. 甲、戊、乙在小模型里都成立；差别在代价。甲随 n−o 线性涨，且读放大同阶；戊与乙的部分写 = 底数（乙每个放单元的入口再多约 2 个角色，代理量）。
2. 戊比乙小：不翻 D18 已定项 10「区间墓碑不做」，不动墓碑树与墓碑判器，不涨永久的墓碑树；它只翻一句用户亲定的「整份写清零」（m4-trunc-r2 第六节），并删读法 Z 的写序一条。
3. 戊的代价是**读法 Z 的第三条是被攻过的东西，换掉它等于让这一块重新零轮**；乙同样是零轮。甲没有这个代价，但换来的是失去通用部分写的意义。
4. 以后 punch 来了：m4-trunc-r2 R8 说成立的只有乙（`research/prompts/m4-trunc-r2-main-verification.md:52`）。戊不为 punch 铺路——punch 要在文件中间放掉单元并留下洞，而「存在的数据单元从 0 起连号」（`crates/singlefs-checker/src/walk.rs:13012`）这个前提本身就不允许中间的洞。所以戊与 punch 是两件事，选戊不排斥以后换乙。

依赖的前提：(a) 单元从 0 起连号、洞只在尾部（R5 的补零空隙）；(b) 同一 txg 里只有一次改动（攒批没落地，`research/prompts/m4-trunc-r2-main-verification.md:51` 的 R7 欠账）；(c) 小模型的入口集合是今天入口加部分写，没有 punch。

什么现象会推翻：

- 戊：只在小模型上量过；在整镜像上（快照下部分写、回退到部分写之前或之中、两个实例）找到任一个合法镜像上重建漏认或多认。攻方在冻结副本拷贝上用本腿 `scan_rule_model.py` 加快照与回退两个入口就能试。
- 「甲写放大随 n−o 线性涨」被推翻：中间点实测不线性（比如写一个单元的 journal 记录数不随 n 涨）；我只量了端点 o = 0 与 o = n−1。
- 「镜像里重建漏认」被推翻：`[3,5,3]` 那行在整镜像（真发布出来的）上 I-1.13 绿——整镜像造不出，因为没有入口，所以这条要等入口落地后用真镜像复核。

## R5　写跨文件末尾、写进截断留下的旧区

### R5.1 读到的事实

- 变长今天的口径：「变长不立意图、一次发布做完：最后一个存在的单元声明长度小于」（`research/prompts/m4-trunc-r1-main-verification.md:39`），字段照抄：「SizeChangeOfARequest::SizeUnchanged | SizeChangeOfARequest::Extension => {」（`crates/singlefs-core/src/transaction.rs:10511`）。
- 洞的标签口径：「标签 3、4 必须至少有一个洞单元」（`research/prompts/m4-trunc-r1-main-verification.md:82`），所以洞被写满那一次标签要换回 1 或 2，上段叶条目随之重写（部分写的发布本来就要重写这个文件的下段到上段叶这一路，指针变了）。
- 单元数对标签对 size 的池级判：「② 单元数对 size 与上段叶条目的标签」（`crates/singlefs-checker/src/walk.rs:13013`），连号：`crates/singlefs-checker/src/walk.rs:13012`。
- 截断字段第四条：「if truncation_field_txg != 0」（`crates/singlefs-checker/src/rebuild.rs:812`）那一段判的是「单元号 ≥ ⌈N_w ÷ 净荷⌉.max(1) 且单元头出生 txg < T」；m4-trunc-r2 R3 把它定为合取的第四条（`research/prompts/m4-trunc-r2-main-verification.md:32`）。
- 写入口都先走空承诺类队列（`crates/singlefs-core/src/mounted_session.rs:1758`），发布函数里断言它空：「previous.queue_of_committed_intents_is_empty(),」（`crates/singlefs-core/src/transaction.rs:9389`）。所以部分写永远见不到截断中间态（size 之外还挂着单元）。

### R5.2 逐格

| 格 | 例 | 规则（一句） |
|---|---|---|
| 写跨过声明长度（起点 ≤ S） | 末尾单元里追加、写出末尾单元 | 一次发布：末尾单元 l−1 合并「变长补零」与「这次的字节」重写一次（按 R2 是「已有单元」读一次），超出的部分按顺序切成新单元（每个没盖满的新单元一次残留读），size = 写的终点，截断字段照抄 |
| 起点越过 size（空隙） | S 之后隔了几个单元再写 | 空隙补整单元零（最后一个存在的单元若声明长度不足 cap，先重写到 cap），再写目标单元；原因：单元必须从 0 起连号；代价线性于空隙单元数，没有上限；备选是改格式允许中间的洞（读者位次定位 `crates/singlefs-core/src/mounted_read.rs:937`、连号判、`crates/singlefs-core/src/extent_tree.rs:11` 的「洞就是缺席」要一起放开），不推荐 |
| 洞里（l ≤ j < L） | 截断后变长出的洞里写 | 补 l..j−1 的整单元零加目标单元 j，size 不变；j+1 = L 时（洞被写满）上段叶标签 3→1、4→2 |
| 截断留下的旧区 | 截到 N、变长回去、在洞里写 | 新单元出生 txg = 这次发布的 txg，大于等于之前任何一次截断的 T，第四条的 `birth < T` 不成立 ⇒ 认领；盘上放掉的旧拷贝与新单元同一个对象身份（含锚点偏移），按写序取最新，新单元取代它；部分写不碰 (T, N_w) |

### R5.3 与 T、N_w 的认领判法合不合

合。证据两条：

- 小模型：只留今天的入口加变长和「洞里的部分写」，今天的三条规则（size、写序、截断字段）在深度 10、17 979 个状态上 0 红（`research/prompts/m4-partial-write-r1-sonnet-model/out/ext-and-hole-write-check.txt`）：

```
name=today_rule_with_ext_and_hole_writes_only depth=10 states=17979 mismatches=0
name=today_rule_without_any_partial_write depth=10 states=10482 mismatches=0
```

  所以洞里和越过末尾的写**不需要**动读法 Z；要动读法 Z 的只有「对已有（活）单元的中间部分写」，即 R4。
- 代码对得上：新单元由 `build_file_version_units`（`crates/singlefs-core/src/transaction.rs:12006`）装，出生 txg 取这次发布的 txg；字段的写不写由造记录的地方定，部分写走变长同一支（照抄）。

I-9.39（T ≠ 0 时字段之外、size 之内的活单元不早于 T，`crates/singlefs-checker/src/walk.rs:13189` 起）：洞里新写的单元出生 txg 大于 T，成立；补零的空隙单元同一个出生 txg，成立。

### R5.4 依赖与推翻条件

- 依赖：R2 的「空隙补零」；R4 选候选甲时洞里的写也要「o 之后一起重写」——洞里写的 o 之后没有现存单元，所以不多重写。
- 推翻：模型里 `phole` 与 `ext` 的语义写错（比如真代码里补零的空隙单元不是同一个出生 txg），或第一个存在的单元之后的洞在真代码里允许非尾部；用真镜像（有入口之后）上 I-4.6、I-9.39 与 I-1.13 三条判一遍即可推翻。

## R6　哪些 inode 先有部分写

### R6.1 读到的事实

- 第一个文件与别的文件今天走两套结构：第一个文件的数据单元在 `TransactionOutput.data_pointers`，别的文件在 `files_other_than_the_first`（「pub struct FileOfAnInodeOtherThanTheFirst {」，`crates/singlefs-core/src/transaction.rs:3673`），文档写明「数据单元从 0 起连号、没有洞，尾部也没有洞」（`crates/singlefs-core/src/transaction.rs:3670`）。冷恢复对别的文件有洞同样拒：`crates/singlefs-core/src/recovery.rs:5944`「extent 树里第一个文件之外的文件没有单元、单元有洞或尾部有洞」。
- 别的文件今天只有整份换下的写：`publish_sequential_write_of_an_inode`（`crates/singlefs-core/src/transaction.rs:9321`）把它上一版的每个数据单元与每个下段节点整份换下；计划在 `FilesOtherThanTheFirstPlan`（`crates/singlefs-core/src/transaction.rs:7278`）与 `plan_the_extent_tree_after_writing_a_file_other_than_the_first`（`crates/singlefs-core/src/transaction.rs:7464`）。改 size 对它拒：「SizeChangeOfAnInodeOtherThanTheFirstFileIsNotDecidedByTheClauses { inode: u64 },」（`crates/singlefs-core/src/transaction.rs:5555`）。
- 第一个文件的「别的单元照抄、只换一个单元」已有现成路径：`carried_file_version_units`（`crates/singlefs-core/src/transaction.rs:13201`）加「data_pointers[position] = DataPointer {」（`crates/singlefs-core/src/transaction.rs:13478`）。
- 整份写造 inode 记录的地方 `inode_record_of_file_version`（`crates/singlefs-core/src/transaction.rs:8954`）取 `file.content.len()` 当 size，部分写造不了：要另写一个「size 不变或变大、其余照抄」的造记录函数。
- 只认第一个文件的写死处：`grep -c FIRST_INODE_NUMBER crates/singlefs-core/src/*.rs` 命中 `unlink.rs` 11、`truncate.rs` 4、`recovery.rs` 5、`transaction.rs` 41，合计 61（与 m4-multi-inode-size-r1 正文 S7 格写的「今天 61 行」一致）。
- R4 的读法 Z 前提不分 inode：`rebuild_index_tables` 按 (inode, 对象出生代) 分组（`crates/singlefs-checker/src/rebuild.rs:580` 的表键），所以**候选的选择对第一个文件与别的文件一样**，不能靠「只给第一个文件」躲掉。

### R6.2 两种范围各要改哪几处

先把部分写拆成两类：不改 size 的部分写——写落在 [0, size) 以内，不碰洞、不碰字段、不碰标签；改 size 的部分写——越过末尾、空隙、洞里。

| 范围 | 要改的地方 |
|---|---|
| 只给第一个文件，两类都做 | (1) 新入口 `publish_partial_write`（形状同 `publish_overwrite_of_an_inode`，`crates/singlefs-core/src/transaction.rs:9270` 先分第一个文件）；(2) 造记录函数；(3) 数据角色计划：只换被触及的单元，别的照抄（`carried_file_version_units` 与 `:13478` 一族）；(4) R3 的读盘函数；(5) R7.3 的释放清单显式交；(6) 改 size 的部分写要补的零空隙与标签 3→1、4→2；(7) 会话 `UserChange` 加一个成员（`crates/singlefs-core/src/mounted_session.rs:107`）、`publish_the_change_once` 加一支（`crates/singlefs-core/src/mounted_session.rs:1778`）；(8) 模型 `ModelPublishKind` 与 RefFS `Operation` 加部分写（RefFS 只有第一个文件）；(9) R4 选定候选的改动 |
| 任意 inode，只做不改 size 的部分写 | 在上面基础上多：(a) `FilesOtherThanTheFirstPlan` 加「只换被触及的单元与它的下段叶到根，别的单元和节点照抄」一支——今天它的 `previous_roles_without_a_successor`（`crates/singlefs-core/src/transaction.rs:7287`）按整份换下算；(b) 别的文件的上一版数据指针与字节照抄的一支（今天 `carried_file_version_units` 只管第一个文件）；(c) 读旧单元的指针取自 `FileOfAnInodeOtherThanTheFirst.data_pointers`；(d) 翻译层对按 inode 号写停下的那一处（m4-multi-inode-size-r1 正文第二节）与 RefFS 多文件——RefFS 只有第一个文件，别的文件的部分写只能对拍模型；(e) 别的文件的单元角色（`DataOfAFileOtherThanTheFirst`）的释放清单 |
| 任意 inode，两类都做 | 在上面基础上多：别的文件的洞（`recovery.rs:5944` 放开、`transaction.rs:3670` 的文档前提、`SizeChange…` 拒绝成员）——这就是 m4-multi-inode-size-r1 的射程，不重复判 |

### R6.3 推荐

入口从第一天起带 inode 参数（照 `publish_overwrite_of_an_inode` 先分第一个文件的写法，`crates/singlefs-core/src/transaction.rs:9270`）。范围分两步：**第一步，任意 inode 的不改 size 的部分写 + 第一个文件的改 size 的部分写**；改 size 的部分写对别的文件返类型化拒绝（照 `crates/singlefs-core/src/transaction.rs:5555` 的形状），等 m4-multi-inode-size-r1 判完再放开。理由：不改 size 的部分写不依赖别的文件的洞，也是验收里「坏旧单元与读故障时请求必须失败」的主体；改 size 的部分写才依赖那一轮；R4 对两类 inode 一视同仁，所以早一步放开不改 size 的部分写不增加 R4 的负担。

依赖：m4-multi-inode-size-r1 只管改 size 的部分写；R4 的选择管所有 inode。推翻：若别的文件的 `FilesOtherThanTheFirstPlan` 实际改不成「只换被触及的单元」（它的下段叶与上段叶重装逻辑耦合在整份换下上），则任意 inode 的不改 size 的部分写要降为同批之外的工作，第一步只给第一个文件。

## R7　故障与崩溃的验收面

### R7.1 故障注入用例（必须覆盖的格）

故障注入设备：`FaultInjectingBlockDevice`（`crates/singlefs-harness/src/fault_injection.rs:1034`）；能注入的读故障只有两种：「ReadFails,」（`crates/singlefs-harness/src/fault_injection.rs:148`）与「ReadReturnsCorruptedBytes { flipped_bit: FlippedBit },」（`crates/singlefs-harness/src/fault_injection.rs:150`）；落点可点名：「OffsetExactly(DeviceOffsetInBytes),」（`crates/singlefs-harness/src/fault_injection.rs:433`）。**它注入不出短读**：读口子永远按请求长度填满缓冲（`crates/singlefs-core/src/recovery.rs:319` 的 `vec![0u8; length]` 加 `read_at`），所以「读回长度不对」要另造（见 F7）。

两份拷贝（`locations` 是两条位置项，同一个槽号在两块盘上）：点名一个落点、`FaultDeviceSelector::EveryDevice`（`crates/singlefs-harness/src/fault_injection.rs:206`）才是「两份都读坏」。

| 格 | 注入 | 必须的结果 |
|---|---|---|
| F1 读坏（两份） | 旧单元槽、`ReadFails`、两块盘 | 返类型化错，盘上逐字节不变，会话现行那一版不动；绝不零填充成功 |
| F2 读坏（一份） | 同上只一块盘 | 成功（读第二份），读序里先一次失败再一次成功 |
| F3 校验不过（两份） | `ReadReturnsCorruptedBytes`、两块盘 | 失败（整单元 CRC-32C 对不上） |
| F4 校验不过（一份） | 同上只一块盘 | 成功 |
| F5 提示位置读不出、中央映射能读 | GC 搬过的单元（提示指旧槽） | 成功，多跳计数加 1 |
| F6 提示与映射都读不出 | 映射里没有这个 key | 失败，不是零基底 |
| F7 读回长度不对 | (a) 读口子替身返回短 `Vec`（照 `crates/singlefs-core/src/recovery.rs:8651` 的 `PoolWhoseReadsReturnNothing` 的写法）；(b) 单元自洽、CRC 重算后写回指针但声明长度 ≠ min(size − 起点, 净荷)；(c) 补齐字节非零 | 失败：(a) 因整单元 CRC 对不上，(b) 因单元头对查找路径（`crates/singlefs-core/src/mounted_read.rs:1130`），(c) 因 I-2.3 |
| F8 无故障对照 | 同一个写、不注入 | 成功，新单元补齐为零、CRC 对、池级 checker 全绿（C57 的要求，`.claude/kb/checks-owed.md:54`） |
| F9 残留读（新槽）读坏 | 新单元槽、`ReadFails` | 按 R2.3 推荐：写成功，这次读出现在读序里；若采备选则失败 |
| F10 毒化空闲槽 | 先把全部空闲单元区写成 0xA5，再做首次写 | 新单元与「零基底」构造的单元逐字节相同（残留一个字节也不进来） |
| F11 说谎设备 | 上一次部分写被 `WriteIsSwallowed`（`crates/singlefs-harness/src/fault_injection.rs:146`）吞掉，再做第二次部分写 | 失败（盘上是旧字节，CRC 对不上新指针）；内存读法会成功，这格把两种读法分开 |

必红变异（每条改了该有一格红）：M1 读失败时用零基底继续（F1、F3、F6 红）；M2 读内存字节不读盘（F1、F3、F11 红）；M3 不验 CRC（F3 红）；M4 首次写不发残留读（R7.2 读序用例红）；M5 拿残留当基底（F10 红）；M6 部分写的释放清单走整份写（R7.3 用例红）。

### R7.2 读回次数用例怎么钉

`FaultCallCounts.reads` 只给总数（`crates/singlefs-harness/src/fault_injection.rs:611`），`FiredFault` 只记注入过的那几次（`crates/singlefs-harness/src/fault_injection.rs:629`），都不给次序。要在测试里加一个只记读的设备包装，记 (盘, 偏移, 长度) 的次序。钉**第一个写调用之前的读序列**的绝对值，而不是「某地址被读几次」，因为一个已有单元的部分写还要发释放核的两读（`crates/singlefs-core/src/transaction.rs:5086`，不在快照里时）：

- 已有单元、没盖满、健康：`[旧单元@盘0（RMW 读），旧单元@盘0、旧单元@盘1（释放核）]`，共 3。
- 新文件写 4 KiB / 追加 / 洞里：`[新槽@盘0]`，共 1，没有释放核。
- 整单元盖满已有单元：只有释放核两读，共 2，没有 RMW 读。
- F2：`[旧@盘0 失败，旧@盘1 成功]` 接释放核。

读法换成另一种（内存字节、不读残留、多读一次）读序就变，用例由绿转红，这是 C385 要的「把读法换成另一种检查必须由绿转红」。

### R7.3 崩溃流、K4

崩溃流（接在多记录流之后，登记进 `.claude/gate.d/stage-inputs.tsv` 与 `crash-point-replay-items.tsv`，照 K9：「登记进 .claude/gate.d/stage-inputs.tsv 与 crash-point-replay-items.tsv」（`research/prompts/m4-trunc-r1-main-verification.md:43`），现有同类行如 `truncate-to-zero-stream`（`.claude/gate.d/crash-point-replay-items.tsv:29`））。三条（名字本腿新起）：`partial-write-in-the-middle-unit`、`partial-write-under-a-snapshot`、`partial-write-across-the-end-into-the-hole`。切点不手选：崩溃点重放对一次发布的写序列取每个前缀当崩溃镜像。要单独断言的类：(i) 新数据单元已落、记录与根没发——扫描一侧靠已发布谓词不认这个单元（推的，没跑），I-1.13 绿，读到的是旧字节；(ii) 全部数据单元与树节点已落、记录没发；(iii) 记录已落、根没发；(iv) 根已发。每个镜像：池级 checker 全绿（含 I-2.3、I-4.6、I-1.13、I-9.39），读到的内容只能是「全旧」或「全新」，不许一部分字节新一部分旧（K8 的崩溃允许集合，只截前、截后）。

K4（被换下的旧单元进 deadlist）能照搬，**但有一个已被截断判决踩过的陷阱**：释放清单的「整份重写」判断由计划显式交：「pub enum FileContentOfThePublish {」（`crates/singlefs-core/src/transaction.rs:4536`），「截断意图那次只重写单元 0 时，那样推会把单元 1、2 一起换下」（`crates/singlefs-core/src/transaction.rs:4533`）。部分写如果借第一个文件的整份写计划（`of_the_plan` 对 `plan.file` 是 inode 1 的一律返 `RewrittenWholeFromTheFirstDataUnit`，`crates/singlefs-core/src/transaction.rs:4547`），`roles_replaced_via_mapping`（`crates/singlefs-core/src/transaction.rs:4564`）就会把「被重写的数据角色个数 .. 上一版单元数」整段当成没有后继而换下，没改的活单元被释放或进 deadlist（读码推的，没造用例跑）。所以部分写的计划必须交 `NotRewrittenWhole`（`crates/singlefs-core/src/transaction.rs:4540`，它的文档正是「只重写一个尾单元」），换下清单 = 计划点名的角色。用例：在有快照、n 个单元的文件上改中间一个单元，deadlist 里恰是被换下的那些角色、释放的落点数钉绝对值、快照读到旧字节、活头读到新字节、其余 n−1 个单元的落点一个不动；M6 变异（改成整份写的清单）必红。候选甲时清单是 o..n−1 个单元，同一个形状。

### R7.4 依赖与推翻条件

- 依赖：R3 的读盘；R2 的残留读放在分配之后；测试里的只记读包装（新写）；`FaultInjectingBlockDevice` 的落点点名对两块盘同一个偏移成立（`slot.to_device_offset()` 同一个槽号，读码推的，没跑）。
- 推翻：F2/F4 的「一份坏仍成功」若与 D4 已定项 4 的字面（读不回就失败）被用户读成冲突——这里的「读不回」指两份都读不回，推的；要用户确认。读序列第 1 条里释放核的两读若实现上并入 RMW 读（一次读复用），则序列变成 1 读，用例钉的数跟着变，这时「读法换了绿转红」仍成立。

## 冲突两句并排（整行抄，来自 `sed -n` 原样输出）

冲突一（R2，C385 的起点）：D4 已定项 3 与 D14 已定项 3。用户 2026-10-26 已定改 D14 的收益栏，下面是改之前两边各自的原话。

```
.claude/kb/decisions/04-校验和位置.md:52 | **定案**：先把旧单元整个读回来并验 MAC，改完重算整单元的 MAC，再整单元写到新位置（COW，不原地覆盖）。首次写不设例外：所有凑不满一个单元的写都走 RMW，不给「首次写」开第二条发布路径（`.claude/rules/fs-design.md`「一个事务层，所有结构共用」同向）；首次写不读回会快得多，这笔代价是知情接受的，买到的是写路径只有一条。
.claude/kb/decisions/14-双轨大小文件-持久临时.md:71 | - 换回来的：写侧快 **一个数量级**（E66（补齐之后小文件怎么落） 设备侧：独立 extent 首次写不必 RMW，25.0 μs vs 内联 247.8 μs）——这一句与 D4（校验和位置） 已定项 3「首次写不设例外」说反话，没判，欠在 C385（首次写要不要读回，两条已定项说反话）；元数据工作集不涨（内联会让每文件 64 B → 89 B、叶数 1.40×）；以及**跨阈值迁移这条路径整个消失**——阈值为 0 就没有「内联 → extent」这一步，E8（大文件与小文件是否该走不同写路径） 记的「阈值不是合格的分支变量」这条约束因此不再适用（它拦的是「追加写在操作中途跨过阈值」）。
```

冲突二（R3，验收句与今天截断尾单元的读法）：

```
.claude/kb/milestone/04-fourth-txn.md:200 | | 通用部分写 RMW | D4（校验和位置） 已定项 3、4：没有任意偏移部分写入口；先解决首次写语义分歧，再实现保留未改字节、读旧校验与失败返错 | 用户 2026-10-26 定这次做。现有截断尾部处理不代替通用入口；坏旧单元与读故障时请求必须失败，不能零填充伪造成功，补齐与校验仍成立；加密启用后的 MAC 行为随加密阶段验证 |
crates/singlefs-core/src/transaction.rs:13189 |     let bytes = &previous.unit(TransactionUnit::Data(rewrite.unit)).bytes;
```

冲突三（R4，K6 的前提与镜像量到的现象）：

```
crates/singlefs-checker/src/rebuild.rs:615 | /// 出生 txg 更大的单元（变长留洞之后、洞里放掉而还在盘上的旧单元由它判死）。前提是活单元的出生 txg 随偏移不降：今天的写入口
research/prompts/m4-partial-write-r1-sonnet-model/out/rebuild-mirror-run.txt | name=partial_write_middle_only births=[3,5,3] claimed=[0, 32634]
```

冲突四（R7，释放清单的整份重写判断与部分写）：

```
crates/singlefs-core/src/transaction.rs:4533 | /// 由发布路径显式交，不按「这次重写的数据角色里有没有单元 0」推——截断意图那次只重写单元 0 时，那样推会把单元 1、2 一起换下）。
crates/singlefs-core/src/transaction.rs:4549 |             Some(file) if file.inode.0 == FIRST_INODE_NUMBER => {
crates/singlefs-core/src/transaction.rs:4550 |                 FileContentOfThePublish::RewrittenWholeFromTheFirstDataUnit
```

## 判定一览

判定用词：一致 = 条款与实现和推出来的互相吻合；冲突 = 两句整行并排抄在上面「冲突两句并排」；规则没说 = 条款没写，本腿推一个。

| 格 | 判定 | 一句话（含推翻条件） |
|---|---|---|
| R2 新文件 | 规则没说 | 读新槽残留 1 次、字节丢弃、旧内容按零；推翻：用户要的读本意是发现点什么 |
| R2 文件末尾之外 | 规则没说 | 每个没盖满的新单元 1 次残留读；末尾单元变长重写按已有单元读；推翻：「单元从 0 起连号」改成允许中间洞 |
| R2 洞里 | 规则没说 | 同新文件；空隙补整单元零、不发读；洞本身读不发盘读 |
| R2 残留校验不过 | 规则没说 | 残留没有参照校验和，D4 已定项 4 的触发物是旧单元，不进射程；推荐读出错不拒写，备选拒写（有可用性风险，分配器退回后是否换槽没查） |
| R2 与 C385 用例 | 一致 | 读回次数 = 净荷没被整单元盖满的触及单元数，读序列钉绝对值（R7.2） |
| R3 读盘还是内存 | 冲突 | 内存读法看不见注入的读故障，满足不了验收句与 C57；推翻：计划阶段借不到设备读口子 |
| R3 截断尾单元一起改 | 冲突 | D4 已定项 4 没给截断留例外；一起改成读盘 |
| R3 同发布第二次部分写 | 规则没说 | 今天造不出；攒批后同一单元一次发布只读一次、后到的写并进内存缓冲 |
| R4 读法 Z 前提 | 冲突 | 镜像量到：`[3,5,3]` 重建只认 [0, 32634]，中间单元部分写后面的活单元被判死，I-1.13 红（红是两个事实的合取，整镜像造不出） |
| R4 候选 | 规则没说 | 戊（字段取代写序）首选、甲（尾重写）退路、乙留给 punch、丙不推荐；戊只在本腿小模型上成立，零轮；推翻：快照或回退入口上找到漏认或多认 |
| R5 越过末尾、洞里 | 一致 | 今天的三条规则加变长加洞里部分写 17 979 个状态 0 红；T、N_w 不碰，新单元出生 txg ≥ T 被认领 |
| R5 起点越过 size | 规则没说 | 单元必须从 0 起连号，空隙补整单元零，代价线性于空隙；推翻：改格式允许中间洞 |
| R6 范围 | 规则没说 | 入口带 inode；第一步任意 inode 的 size 不变部分写 + 第一个文件的改 size 部分写；别的文件改 size 的等 m4-multi-inode-size-r1 |
| R7 故障格 | 规则没说 | F1–F11 十一格，必红变异 M1–M6；读回长度不对要新造读口子替身（故障设备注入不出短读） |
| R7 崩溃流 | 一致 | 三条新流，切点取发布写序列的每个前缀，每镜像池级 checker 全绿、读到全旧或全新 |
| R7 K4 | 冲突 | 能照搬，但部分写借第一个文件的整份写计划会把没改的单元一起换下；计划必须交 `NotRewrittenWhole` |

## 要主 agent 跑的命令

整条复跑（模型目录里 `tools/rerun.sh`，命令见开头）本腿起过两次（外层上限 8G 与 4G），都排在别的会话占满的内存预算后面（`run-with-memory-cap: 排队（已等 1150 秒，至多 3600 秒）：slice 已占 43.6 GiB，这一条要 8.0 GiB`；4G 那次等了 181 秒仍是 slice 已占 39.6 GiB），我停掉了，没跑起来；`rerun.sh` 只过了 `bash -n`，各步骤是单独跑的（命令与原样输出在 R4.2、R4.4，输出文件就是 `out/` 里那几份）。主 agent 若要核整条：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=正推腿模型目录的 rerun.sh（小模型、harness 一个新测试目标、checker 一条 lib 测试）; 范围=4 步; 为什么=核正推腿报告里抄的输出与 sha256' bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-partial-write-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m4-close-rounds-frozen --work <新的草稿目录>
```

跑完看四行 `name=compare ... same=yes`。

## 没做什么

- 没判别的腿的格；没替主 agent 采纳或出判决；没读攻方腿这一轮的任何产出。
- 整条 `rerun.sh` 没跑通（排队，见上一节），所以「脚本整条复跑的输出与 `out/` 逐行相同」这一步复核不了；成本探针与镜像探针各自的跑法是 R4.2、R4.4 里贴的命令，两个小模型脚本直接 `python3 -I` 跑。
- 没起任何门禁阶段、checker 档测试、GPU、崩溃放量；`cargo` 只起了 `singlefs-harness` 的一个新测试目标和 `singlefs-checker` 的一条 lib 测试，都在冻结副本的拷贝里，经内存包装 8G。
- R4 没在整镜像上造出 I-1.13 的红：今天没有按偏移写的发布入口，造不出合法镜像；红是「重建一侧漏认」与「遍历一侧有」两个事实的合取，在重建模块层造的镜像加读比对代码得出。
- 候选乙每条墓碑记录多出的角色数只在三单元的小池上用「unlink 意图对截断意图」代理量了一次（多 2 个角色），墓碑树多层时的数没量；候选甲的中间点（o 在 0 与 n−1 之间）没量，只量了两个端点。
- 候选戊没被攻过，小模型没覆盖快照与回退、多实例、同一 txg 装几次改动、崩溃留下的未发布单元、两个 inode 交错，单元数上限 4。
- 没验：计划阶段能不能借到写入口的 `devices` 当读口子（没过编译器的借用检查）；分配器退回后重试是否换槽；GPU 恢复内核里墓碑怎么读；崩溃切点「新单元已落、根没发」扫描一侧靠已发布谓词不认它（读码推的）；F7(b) 要在上一版内存里改指针的校验和，造法没试。
- 模型目录 `tools/rerun.sh` 里模板自带的 `--selftest` 只对模板默认的 `rerun_steps` 成立，换了步骤之后它那一格本来就红（`bash research/prompts/m4-partial-write-r1-sonnet-model/tools/rerun.sh --selftest` 退 1，原因是检查默认步骤的输出标记），没当交回条件。
- 引文核对：冻结副本为根跑 `cite-check.py --root <冻结副本>` 全绿；现工作区为根、不带 `--unchanged-since` 跑，因为别的会话在我开工之后改了 `rebuild.rs`、`walk.rs`、`lib.rs`（行号偏移 2 到 106），会红在这三份文件的引文上，带开工时刻跑则这些列为没判。两条的末行见交回。
