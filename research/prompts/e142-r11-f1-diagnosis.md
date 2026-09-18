# E142 第十一次跑量 5（F1 停机）只读诊断

写于 2026-09-18（本机 UTC 02:56–03:10）。仓里文件一个字节没改，没跑 git 写命令；改动、构建与转储都在副本 `/tmp/claude-1000/e142-f1-diag/repo` 里做。

## 零、结论

11 个不等区域全部出自同一个根因：**两边喂给第一个事务的文件内容不同**。

- 装置：`research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:3717` 写 `(0..3000u32).map(|index| u8::try_from((index * 7 + 3) % 251)…)`
- 实装一侧的驱动：`crates/singlefs-harness/src/scenario.rs:29-33` 的 `first_file_content()` 写 `(0..FIRST_FILE_BYTES).map(|index| u8::try_from(index % 251)…)`，由 `scenario.rs:99` 交给 `run_first_transaction`

长度同为 3000，字节不同。数据单元载荷因此不同，此后每一个罩到它的 CRC 沿指针一路改到根：数据单元载荷 CRC 与头校验和 → 指向它的位置条目 CRC → extent 根、映射根的载荷 CRC 与头校验和 → 树表 → 根记录与 journal 记录。
装置与 `singlefs-core` 写路径在字节层逐字相同：把任一边的文件内容换成另一边的，21 个区域 21 个全等（第六节）。两边的每个校验和字段也都按 `.claude/kb/layout/01-first-txn.md` 的覆盖口径算对了，一个独立的按位 CRC32C 核对器两边各核 92 格，全过。

所以问 4 给的四类（装置错 / 实装错 / 条款两读 / 查不清）一格都不落；11 格都是另一类：**比对的输入不同**。量 5 的实装臂没满足跑前登记第一节第 2 条的「同一条路（同参数）」。偏离这条约定的是 `crates/singlefs-harness` 的驱动输入，不是 `singlefs-core` 的写路径（归属证据在第五节）。

## 一、做了什么

| 步 | 命令 / 做法 | 结果 |
|---|---|---|
| 快照 | 02:56:10 UTC 拷 `crates/`、`Cargo.*`、`research/Cargo.*`、`research/e7-index-bench/` 到副本；关键文件 sha256 存 `snapshot.sha256`；03:02 与 03:05 两次对原仓 `sha256sum -c` | 两次都是 8 个文件全部 OK（诊断期间没有人改这几份文件） |
| 副本加转储 | 实装：`first_transaction_region_bytes.rs` 在打结果行之前按 `FIRST_TRANSACTION_REGIONS` 把 21 个区域整段写成 `.bin`；装置：量 5 循环里 `let device_sha256 = …` 那一行之后把 `device_bytes` 写成 `.bin`（都由环境变量 `DIAG_DUMP_DIR` 打开） | 只多写了文件，结果行不变 |
| 构建 | `nice -n 19 cargo build --offline --release`，目标目录 `target-crates/`、`target-research/`（都在诊断目录里） | 都是 rc=0 |
| 复现 | 实装快照 `impl-snapshot.out`；装置 `e142-first-txn-dry-run impl-snapshot.out > device-run.out` | 产物第 1–284 行只差 10 行（见第七节 1），第 285–309 行（实装快照）逐字相同；`impl_bytes_equal_summary regions=21 equal=10 unequal=11` 与产物第 233 行一致 |
| 全量逐字节比 | `cmp.py`：两份转储 21 对逐字节相减，列出每一段连续差异 | 见第二节 |
| 独立核校验和 | `verify.py`：按位 CRC32C（不查表，与两份 Rust 都不共用代码，自检 `CRC32C("123456789") = 0xE3069283`），按 layout 的覆盖口径重算每个校验和字段与每条位置条目 CRC | 装置 92 格全过、实装 92 格全过（`verify-device.txt`、`verify-impl.txt` 末行 `all_ok True`） |
| 核对器自证会红 | ① 把实装的数据单元塞进装置那一套转储（`dump-mixed`）；② 在装置转储里翻根记录 400、journal(盘 1) 4000、树表 20 各一位（`dump-flip`） | ① 8 格判 False（extent 根 247/261、映射根 206/220、两份 journal 317/331）；② 14 格判 False（树表头校验和、根记录自证、journal 头校验和，外加下游的指针 CRC）。两次都是 `all_ok False` |
| 反向核对 | 副本 `scenario.rs` 改成装置的内容再跑实装；另把副本装置 `:3717` 改成实装的内容，与原实装快照比 | 两次都是 `impl_bytes_equal_summary regions=21 equal=21 unequal=0`；两次的 21 对 `.bin` 都是 `cmp` 全等 |

⚠️ 先说清产物里 `mismatch_bytes` 的口径（问 2 的前提与它有关）：16 个单元区域在两边都是 `hexadecimal_extent=head_and_tail`，`compare_region` 只拿头 32 字节与尾 32 字节比（`e142_first_transaction_dry_run.rs:544-562`）。所以产物第 192/196/216/220 行的 `mismatch_bytes=3` / `4` **只数了头 32 字节那一窗**，不是整个区域。真实的差异数见第二节：数据单元 2995、extent 根 16、映射根 24、树表 16。根记录与 journal 记录是 `whole_region`，产物里的 20 / 56 是整段的真数。

## 二、问 1：每一类不等的是哪几个字段

字段名照 layout 第二节（单元头，第 171 行头校验和、第 180 行载荷 CRC）、第三节第 201 行（指针里的位置条目：设备 4 + 槽号 6 + 整单元 CRC32C 4）、第三·二节（映射条目 55 = key 27 + 两条位置条目）、第六节第 333/337/338/344 行（journal 头校验和、载荷校验和、新根段、点名项）、第七节第 360 行（根记录自证校验和）。偏移是区域内偏移，「两边值」是小端读出的 u32；两盘的同名区域逐字相同（产物里两盘 sha256 相同），表里只列盘 0。

| 区域 | 差异段（偏移, 长度） | 字段 | 装置 | 实装 |
|---|---|---|---|---|
| data_unit | (11, 3) | 头校验和（32 字节字段从 10 起，CRC32C 在前 4 字节 10–13；首字节两边都是 `da`，所以产物报 11） | 0x49f685da | 0x8935cdda |
| data_unit | (101, 4) | 载荷 CRC，罩 [105, 32768) | 0x42e54bef | 0x73e0742b |
| data_unit | 134 起 13 段，合计 2988 字节 | 用户数据本身（声明长度 3000 之内）；3000 个字节里有 12 个两边恰好同值，下标 i ≡ 125 (mod 251)，区域偏移 259、510、… | 首 4 字节 `03 0a 11 18`（(7i+3) mod 251） | `00 01 02 03`（i mod 251） |
| extent_root | (10, 4) | 头校验和，罩 [0, 134)（k = 24） | 0xbf6a906a | 0xd1bc08c2 |
| extent_root | (124, 4) | 载荷 CRC（76 + 2k），罩 [134, 16384) | 0xf1554896 | 0x7c6d67fa |
| extent_root | (247, 4)、(261, 4) | 唯一一条 extent 叶记录（条目区 163 起，key 24 之后是 88 字节的数据指针）里两条位置条目的整单元 CRC → 数据单元 | 0x0aff390c | 0xd1f534ad |
| mapping_root | (10, 4) | 头校验和，罩 [0, 140)（k = 27） | 0xc459e316 | 0x37e5993b |
| mapping_root | (130, 4) | 载荷 CRC，罩 [140, 16384) | 0x1157b3d3 | 0x5137e0d7 |
| mapping_root | (206, 4)、(220, 4) | 条目 0（码 1，出生树 11）value 里两条位置条目的 CRC → 数据单元 | 0x0aff390c | 0xd1f534ad |
| mapping_root | (261, 4)、(275, 4) | 条目 1（码 2，出生树 11）两条位置条目的 CRC → extent 根 | 0x59b5855e | 0xd3ba8bec |
| tree_table | (10, 4) | 头校验和，罩 [0, 102)（k = 8） | 0x43542296 | 0x82b024c1 |
| tree_table | (92, 4) | 载荷 CRC，罩 [102, 16384) | 0xa53baf34 | 0xd08aedb8 |
| tree_table | (205, 4)、(219, 4) | 条目 0（树 ID 11，extent 树）根指针（条目内偏移 14）里两条位置条目的 CRC → extent 根 | 0x59b5855e | 0xd3ba8bec |
| root_record | 5 段 × 4 = 20 字节 | 见第四节 | | |
| journal_record | 14 段 × 4 = 56 字节 | 见第四节 | | |

不等的只有这些。inode 叶、inode 根、分配根、记账根、两份超级块槽整段全等（`cmp.py` 输出 `diff_bytes=0`）：它们罩到的字节里没有文件内容。inode 记录只带长度 3000，不带内容；这几个单元也没有指向数据单元、extent 根、映射根、树表的位置条目。

## 三、问 2：头校验和不等，原因是哪一种

**前提不成立。** 「校验和字段不等而其余字节相等」是 `head_and_tail` 只抽头尾 32 字节造成的错觉（第一节 ⚠️）。四类单元的其余字节都不等：

- 数据单元：载荷与载荷 CRC 都不等；
- extent 根、映射根、树表：载荷 CRC 与条目区里的位置条目 CRC 都不等，这几段都在 32 字节窗外。

头校验和不等是这些字节不等的**下游**：载荷 CRC 的字段（数据单元在 101，码 2 节点在 76 + 2k）落在头校验和的覆盖范围 [0, header_end) 之内，载荷 CRC 一变，头校验和跟着变。

逐条排掉题面给的另外三种解释：

| 解释 | 查的是什么 | 结论 |
|---|---|---|
| 覆盖范围（header_end）不同 | 数据单元：实装 `unit.rs:233` `DATA_UNIT_HEADER_BYTES` = 105（`singlefs-format/src/lib.rs:33`）；装置 `:956` 传 `DATA_UNIT_HEADER_BYTES` = 105（`:23`）。码 2：实装 `unit.rs:154-158` = `index_node_header_bytes(k) − 29` = 86 + 2k（`singlefs-format/src/lib.rs:50-54` 是 86 + 2k + 29）；装置 `:1047` = `index_node_header_bytes(k)` = 86 + 2k（`:94` 是 86，不含 29） | 两边相同。另一个证据：`verify.py` 按 105 / 86 + 2k 重算，两边的头校验和都对得上 |
| 封口时机不同 | 两边都是先写载荷 CRC、再封头校验和、之后不再改：实装 `unit.rs:263-265`（码 1）与 `:200-202`（码 2）；装置 `:954-956` 与 `:1076-1078` | 相同；而且各自存的头校验和都等于用最终字节重算的值（`verify.py` 两边全过），没有「封口之后又改了被罩字节」这回事 |
| 比的不是同一段字节 | 实装快照里 `impl_region_table_against_writes regions=21 write_calls=21 regions_without_a_write=none writes_outside_the_table=none matches=true`（产物第 308 行）；把内容对齐之后 21 段全等（第六节） | 同一段。区域表与装置的 `region_geometry` 落点一致 |
| **别的：被罩的字节本身不同，因为输入不同** | 第二节的全量比对；反向核对两次都是 21/21 全等 | **是这一条** |

## 四、问 3：根记录偏移 96 起 20 字节、journal 记录偏移 46 起 56 字节

值直接取自产物的整段十六进制行（装置第 223 / 225 行，实装第 302 / 303 行），按盘上字节序抄；括号里是小端 u32。两段里除表中各行之外，没有别的字节不等（逐字节扫过，表外差异 `[]`）。

### 根记录（区域 0 槽 1，512 字节；字段序照 layout 第七节，装置 `to_slot` 在 `:1280-1302`，实装在 `crates/singlefs-core/src/root_record.rs:31` 起、自证校验和在 `:56`）

「20 字节」不是连着的 20 字节，是 5 段各 4 字节：产物只报了第一处偏移 96 与总数 20。

| 区域内偏移 | 字段 | 装置 | 实装 | 罩的是什么 |
|---|---|---|---|---|
| 96 | 树表单元指针（36 起，86）· 位置条目 0（盘 0）的整单元 CRC | `0d2cca05`（0x05ca2c0d） | `24acb30e`（0x0eb3ac24） | 各自的树表单元 50248 整 16384 字节 |
| 110 | 同一指针 · 位置条目 1（盘 1）的 CRC | 同上 | 同上 | 同上 |
| 138 | 自证校验和（CRC32C 4 + 28 零），罩整槽 [0, 512) | `f826eb07`（0x07eb26f8） | `afc75d3e`（0x3e5dc7af） | 本槽，其中含上下两行 |
| 316 | 中央映射树根指针（256 起）· 位置条目 0 的 CRC | `014e4098`（0x98404e01） | `bedfaead`（0xadaedfbe） | 各自的映射根 50247 |
| 330 | 同一指针 · 位置条目 1 的 CRC | 同上 | 同上 | 同上 |

magic、fsid、flags、实例代号 1、checkpoint_txg 3、两个指针的其余 78 字节（槽号 50248 / 50247、出生树、出生 txg、实例代号、出生序号）、树 ID 水位、F、实例表指针、尾部留位，两边逐字节相同。

### journal 记录（jsn (1, 3)，两盘各一份，4096 字节；字段序照 layout 第六节「记录头字段表」与「点名项字段表」，装置 `to_bytes` 在 `:1567-1608`，实装在 `crates/singlefs-core/src/journal.rs:150-167`）

「56 字节」同样是 14 段各 4 字节：

| 区域内偏移 | 字段 | 装置 | 实装 | 罩的是什么 |
|---|---|---|---|---|
| 46 | `header_csum`（CRC32C 4 + 28 零），罩 [0, 4096) | `8407351e`（0x1e350784） | `c25d7918`（0x18795dc2） | 整条记录，含下面各行 |
| 91 | 载荷校验和，罩点名项数组 [307, 307 + 8 × 56 = 755) | `7a627f47`（0x477f627a） | `60b9cb62`（0x62cbb960） | 8 个点名项，含下面 8 行 |
| 155、169 | 新根段 · 树表指针（95 起）两条位置条目的 CRC | `0d2cca05` | `24acb30e` | 树表单元 |
| 241、255 | 新根段 · 映射根指针（181 起）两条位置条目的 CRC | `014e4098` | `bedfaead` | 映射根 |
| 317、331 | 点名项 0（t1，码 1）两条位置条目的 CRC | `0c39ff0a`（0x0aff390c） | `ad34f5d1`（0xd1f534ad） | 数据单元 |
| 373、387 | 点名项 1（t2，码 2）两条位置条目的 CRC | `5e85b559`（0x59b5855e） | `ec8bbad3`（0xd3ba8bec） | extent 根 |
| 653、667 | 点名项 6（t7，码 2）两条位置条目的 CRC | `014e4098` | `bedfaead` | 映射根 |
| 709、723 | 点名项 7（t8，码 2）两条位置条目的 CRC | `0d2cca05` | `24acb30e` | 树表单元 |

点名项 2–5（inode 叶、inode 根、分配根、记账根）、事务号、提交标记、反向链（偏移 87，两边一样；它罩的是上一条暖机记录的头，与文件内容无关）、水位 19、F 0、fsid、MAC，两边逐字节相同。

### 哪一边与 kb 条款一致

**两边都一致。** 上面每一格我都用 `verify.py` 按条款口径独立重算过：根记录自证校验和罩整槽 512 含补齐、自身按 0 参与（layout 第 360 行）；journal `header_csum` 罩 [0, 4096)、自身按 0 参与（第 333 行）；载荷校验和罩点名项数组（第 337 行）；每条位置条目的 4 字节是被指单元的整单元 CRC32C（第 201 行、第 344 行）。

两边的每一格都等于用它自己镜像里被罩字节算出来的值（两边各 92 格 `ok=True`）。值不同只是因为被罩的数据单元字节不同。拿一边的字节去核另一边的 CRC 必然对不上：`dump-mixed` 就是这么造出来、判出 8 格 False 的。

kb 里没有任何一格规定文件内容本身：layout 第二节「单元净荷」那一行只写「文件内容 + 0」，`.claude/kb/experiments/142-第一个事务的干跑.md` 与 `.claude/kb/milestone/01-first-txn.md` 只写「3000 字节」。我在 `.claude/kb/` 里 grep 过 `% 251`、`* 7 + 3`，零命中。

## 五、问 4：逐区域分类

题面给的四类（装置错 / 实装错 / 条款两读 / 查不清）各 0 格。11 格都归另一类：**两边写路径都按条款，比对的输入不同**。

| 区域 | 盘 | 类 | 直接原因 | 依据 |
|---|---|---|---|---|
| data_unit | 0、1 | 输入不同 | 载荷字节不同 ⇒ 载荷 CRC（101）不同 ⇒ 头校验和（10）不同 | 第二节；内容对齐后全等 |
| extent_root | 0、1 | 输入不同（下游） | 叶记录里指向数据单元的两条位置条目 CRC 不同 ⇒ 载荷 CRC ⇒ 头校验和 | 同上 |
| mapping_root | 0、1 | 输入不同（下游） | 条目 0（数据单元）与条目 1（extent 根）的位置条目 CRC ⇒ 载荷 CRC ⇒ 头校验和 | 同上 |
| tree_table | 0、1 | 输入不同（下游） | 条目 0（extent 树）根指针的位置条目 CRC ⇒ 载荷 CRC ⇒ 头校验和 | 同上 |
| root_record | 0 | 输入不同（下游） | 树表指针、映射根指针的位置条目 CRC ⇒ 自证校验和 | 第四节 |
| journal_record | 0、1 | 输入不同（下游） | 新根段两个指针、点名项 0/1/6/7 的位置条目 CRC ⇒ 载荷校验和 ⇒ `header_csum` | 第四节 |

**偏离约定的是哪一边**（这件事 kb 里没有条款，下面只是事实）：

- 跑前登记第一节第 2 条（`research/prompts/e142-r11-prereg.md:21`）把实装臂定义为 `run_first_transaction`「在内存盘上跑同一条路（同参数）」；同一节第 1 条（`:18`）写「同 3000 字节内容」。
- 实装一侧的驱动自己也声明跟着装置取参数：`crates/singlefs-harness/src/scenario.rs:1` 写「参数照 E142（第一个事务的干跑） 装置取」，`:2` 写「同参数同字节」；`crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs:2` 写「与虚机档、与 E142 装置同参数：同 fsid、同写入时刻、同 3000 字节内容」。它的 `first_file_content()`（`scenario.rs:29-33`）与装置 `:3717` 不同，所以**这几句声明是假的**。
- 不符合约定的是驱动的输入，不是 `singlefs-core` 的写路径：`unit.rs`、`checksum.rs`、`journal.rs`、`root_record.rs` 在同输入下与装置逐字节相同。
- 这条登记是怎么漏过去的：登记第 99 行核「同参数」时，内容这一项只核了长度（`同 3000 字节 FIRST_FILE_BYTES:26`），说出口的是「同 3000 字节内容」。

两边的内容各从什么时候起（`git log -S`，只读）：装置的 `(index * 7 + 3) % 251` 从 fdb39ed（2026-09-12 23:46 UTC）起；`scenario.rs` 的 `index % 251` 从 53c9f38（2026-09-14 13:51 UTC）起。所以从实装驱动建出来那天起两边就不同，量 5 是第一次逐字节比到数据单元。

对齐两边时各自会牵动的读者（只列事实，改哪边由主 agent 定）：

- 改 `scenario.rs:29-33`：QEMU 日志核对器 `first_transaction_device_log_check.rs:80` 拿 `first_file_content()` 比读回内容；`tests/second_transaction_supplement_one_write_accounting.rs:291` 也用它。另有三处测试各自手写了 `index % 251`，不经过这个函数：`tests/common/mod.rs:67`、`tests/first_transaction_step_two_data_unit.rs:51`、`tests/first_transaction_step_five_publish.rs:95`。
- 改装置 `:3717`：E142 产物里每一个罩到数据单元的 CRC 都会变（`replay.sh:157` 登记的是 exact 比对）；装置单测的 `sample_file()`（`:4121`）也是同一个式子。

## 六、三次推导

| 步 | 做法 | 结果 |
|---|---|---|
| 正推 | 21 对区域整段转储、逐字节相减（`cmp.py`），逐段对 layout 字段表；再读两边生成文件内容的源码行 | 所有差异段都能归到「载荷不同」加它沿指针传上去的 CRC；两边源码的文件内容式子不同（`:3717` 对 `scenario.rs:31`） |
| 反推 | 假设根因不是输入，而是某一边的编码（header_end、封口次序、指针布局、覆盖范围）有错，那只把文件内容对齐之后，至少应该还剩一个区域不等；或者 `verify.py` 至少应该在一边判出一格 False | 两个方向对齐内容后都是 21/21 全等（`device-run-vs-devcontent.out`、`device-implcontent-vs-impl.out` 的 `impl_bytes_equal_summary … unequal=0`）；`verify.py` 两边各 92 格全过。两样都没出现 |
| 校验 | 另写一套按位 CRC32C（Python，不共用两份 Rust 的代码），按条款口径重算每个校验和字段与位置条目 CRC | 两边全过。先证明它会红：`dump-mixed` 判出 8 格 False，`dump-flip` 判出 14 格 False |

## 七、顺带看到的（不在四问里，但会影响怎么读这份产物）

1. **产物与装置源码不同步。** 产物第 200、202、204、206、208、210、212、214、230、232 行（`equal=true` 的 10 行）写 `first_diff_offset=none`；而当前源码 `:3945` 把 `None` 一律打成 `unknown_middle_of_region`，我用当前源码复跑，这 10 行就成了 `…equal=true first_diff_offset=unknown_middle_of_region…`（`device-run.out`）。时间上，产物 mtime 是 02:38:44 UTC，装置源码 mtime 是 02:41:14 UTC，源码是在产物之后改的。`replay.sh:157` 对 E142 登记的是 exact 比对，所以按今天的源码复跑，这 10 行会对不上。另外，`equal=true` 时打「差异在中段、定位不了」本身就是一句假话：全等的行根本没有差异。
2. **`head_and_tail` 行的 `mismatch_bytes` 不是区域总数。** 它只数头 32 字节那一窗（第一节 ⚠️）。数据单元真数 2995，产物报 3。F1 条款（登记第 244 行）写的是「产物里给出 `mismatch_bytes=`」，读的人会把它当成整个区域的差异字节数。
3. **数据单元的 `first_diff_offset=11` 是巧合。** 头校验和字段从 10 起，两边第一个字节恰好都是 `0xda`。

## 八、没做的与查不清的

- 没跑门禁、`replay.sh`、任何单测与变异表，没碰 QEMU 与 herd7。四问只要字节层的归因，用不上它们。
- 量 1–4、6、8 是装置内部 M_A 与 M_B 两次跑的比较，两次用的是同一份内容，这次的输入分歧碰不到它们。这是我从登记第一节第 1 条（`:18`）推的，没有复跑去核。
- `replay.sh` 的 `driver_e142`（`:373-378`）用 `cargo run -q` 跑实装快照，是 debug 构建；我用的是 release。实装快照与产物第 285–309 行逐字相同，所以构建档对字节没有影响。
- 没有查不清的格。

## 附：诊断目录里的文件

`/tmp/claude-1000/e142-f1-diag/` 下：

| 文件 / 目录 | 是什么 |
|---|---|
| `snapshot.sha256` | 开工时 8 个关键文件的 sha256 |
| `repo/` | 副本，只在转储、换内容两处动过（第一节、第六节） |
| `dump-device/`、`dump-impl/` | 原样内容下 21 个区域的整段字节 |
| `dump-impl-devcontent/`、`dump-device-implcontent/` | 反向核对两次的转储 |
| `dump-mixed/`、`dump-flip/` | 核对器的两组阴性对照 |
| `cmp.py`、`verify.py`、`verify-device.txt`、`verify-impl.txt` | 全量比对脚本、独立 CRC 核对器与它的两份输出 |
| `impl-snapshot.out`、`device-run.out`、`impl-snapshot-devcontent.out`、`device-run-vs-devcontent.out`、`device-implcontent-vs-impl.out` | 各次跑的原样输出 |
