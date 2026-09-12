## E143 一事务一单元下的 journal 账 —— 已跑（2026-09-13，纯算术，6 单测 / 10 条变异全抓）

问的是 C310（事务切分纪律与记录数口径打架） 那笔账在用户 2026-09-13 定案（切分纪律是一事务一单元，D16（发布语义） 已定项 5 末段照旧）之下要付多少：
按 D25（目标负载优先级） 的六个负载点，每次 fsync 几条记录、journal 占用户数据几成、T_dirty 满窗要多大的环。
**六条判据全过，跨装置闸成立**：丙臂按 E91（已定格式下的环账与准入账） 的口径算出 1847 条 / 7388 KiB，逐字相等。
**主数**：甲（今天条款）在 seq 批 1 上每次 fsync 8 条记录、journal 占用户数据 12.5%；T_dirty 满窗 65 536 条记录、256 MiB，是预想 64 MiB 环的 4 倍；
乙（一条记录装多个事务，对照）1 条、1.56%、满窗 4232 KiB。只报数，不判条款该不该改。

**复跑命令**（`exact` 模式，与留存产物逐字节比对）：

```bash
cd research && bash scripts/replay.sh E143
# 直接跑装置：
cd research && cargo run --release --bin e143-one-unit-per-txn-journal
```

代码 `research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs`，原始输出 `research/results/e143-one-unit-per-txn-journal-2026-09-13.out`（34 行，末行 `emitted=34`），
跑前登记 `research/prompts/e143-preregistration.md`，变异表 `research/mutations/e143_one_unit_per_transaction_journal.tsv`。

### 这是纯算术，不是测量

没有 I/O、没有随机源 ⇒ 跑 N 遍逐字节一致。证据强度来自 6 个单测里钉死的绝对值 + 10 条变异全抓：
记录头改回 78、乙的项宽改 56、甲把祖先另开记录、seq 批 1 改 12 叶、盘数改 1、丙的满窗按单元数算、F 改 2、环改 128 MiB、丙每单元一个事务、T_dirty 改 1 GiB，每条都有测试红。

### 被测对象与臂

| 臂 | 事务怎么切 | 一条记录装什么 | 每条装几项 |
|---|---|---|---|
| 甲 今天条款 | 每个数据单元一个事务（D16（发布语义） 已定项 5 末段） | 一个事务（D23（journal 的角色与格式） 已定项 7：一条记录一个事务号）；祖先并进这次 fsync 的末条 | (4096 − 95) ÷ 56 = 71 |
| 乙 对照 | 每个数据单元一个事务 | 多个事务：点名项各带事务号 8，记录头去掉事务号与提交标记、加首末事务号 16 | (4096 − 102) ÷ 64 = 62 |
| 丙 对照（用户已否） | 一次 fsync 一个事务（E75（记录尺寸与环几何） 的旧读法） | 一个事务 | 71 |

负载点照 D25（目标负载优先级）：rand 批 1（1 叶 + 4 祖先）、multistream 批 1（1 + 4）、seq 批 1（8 + 4）、metaheavy 批 1（2 + 5.937，祖先按项数取整 6）、seq 批 10（80 + 4）、multistream 批 10（10 + 31）；
「叶」按 E75（记录尺寸与环几何） 的读法当数据单元。

### 判决（按跑前写死的判据，整行抄自产物）

```text
E7RESULT name=capacity arm=jia items_per_record=71
E7RESULT name=capacity arm=yi items_per_record=62
E7RESULT name=fsync load=seq batch=1 leaves=8 ancestors=4 arm=jia transactions=8 records=8 journal_bytes_two_devices=65536 data_bytes_two_devices=524288 journal_over_data_bp=1250 records_per_second=22280 journal_kib_per_second=178240
E7RESULT name=fsync load=seq batch=1 leaves=8 ancestors=4 arm=yi transactions=8 records=1 journal_bytes_two_devices=8192 data_bytes_two_devices=524288 journal_over_data_bp=156 records_per_second=2785 journal_kib_per_second=22280
E7RESULT name=fsync load=seq batch=10 leaves=80 ancestors=4 arm=jia transactions=80 records=80 journal_bytes_two_devices=655360 data_bytes_two_devices=5242880 journal_over_data_bp=1250 records_per_second=222800 journal_kib_per_second=1782400
E7RESULT name=fsync load=multistream batch=10 leaves=10 ancestors=31 arm=jia transactions=10 records=10 journal_bytes_two_devices=81920 data_bytes_two_devices=655360 journal_over_data_bp=1250 records_per_second=27850 journal_kib_per_second=222800
E7RESULT name=window arm=jia records=65536 bytes_kib=262144 ring_kib=65536 window_over_ring_milli=4000
E7RESULT name=window arm=yi records=1058 bytes_kib=4232 ring_kib=65536 window_over_ring_milli=64
E7RESULT name=window arm=bing records=1847 bytes_kib=7388 ring_kib=65536 window_over_ring_milli=112
E7RESULT name=ring arm=jia worst_transaction_bytes=4096 lower_bound_any_transaction_kib=12 lower_bound_dirty_window_kib=786432
E7RESULT name=ring_cap arm=jia ring_kib=65536 safety_factor=3 dirty_bytes_allowed=178946048 dirty_kib_allowed=174752
E7RESULT name=keys load=seq batch=1 arm=jia data_units=8 distinct_write_orders=8
E7RESULT name=verdict jia_records_seq_batch1=8 jia_journal_over_data_bp_seq_batch1=1250 bing_window_records=1847 bing_window_kib=7388 e91_anchor_holds=true jia_window_over_ring_milli=4000
```

| # | 判据 | 结果 |
|---|---|---|
| 1 | 每次 fsync 的记录数 | 甲 1 / 1 / 8 / 2 / 80 / 10，乙 1 / 1 / 1 / 1 / 2 / 1，丙 1 / 1 / 1 / 1 / 2 / 1，与登记表逐格相等 |
| 2 | journal 占用户数据 | 甲 12.5%（六个负载点一样，每个数据单元恒一条记录），乙 seq 批 1 是 1.56% |
| 3 | 跨装置闸 | 丙按 E91（已定格式下的环账与准入账） 口径 1847 条、7388 KiB，逐字相等 |
| 4 | T_dirty 满窗 | 甲 65 536 条、262 144 KiB，是 64 MiB 环的 4.0 倍；乙 1058 条、4232 KiB |
| 5 | I-8.1（环几何够大） 的两种读法 | 按「任一事务」读，甲最坏一个事务 1 条记录、F = 3 下界 12 KiB；按「满窗装进环」读，甲要 768 MiB，或 T_dirty 压到 174 752 KiB（约 170.7 MiB） |
| 6 | 映射 key 唯一性 | 甲、乙下一次 fsync 的 8 个数据单元 8 个不同写序；丙只有 1 个（这正是 D19（块指针的结构与宽度预算） 已定项 6 押在切分纪律上的那一格） |

### 这几个数说明什么

1. **D23（journal 的角色与格式） 已定项 12 那句「目标负载 12 项事务恰占 1 条记录 ⇒ 5.9 倍余量」按的是丙的读法**。按用户定案的切分纪律，seq 批 1 是 8 条记录、seq 批 10 是 80 条；余量那一格没有了，
   每条记录只装 1 到 5 个点名项（4096 字节里用 151 到 375 字节）。
2. **环几何的约束从「任一事务」挪到「满窗」**：I-8.1（环几何够大） 逐字管的是任一事务的最坏占用，甲下那是 1 条记录、下界 12 KiB，约束等于没有；
   真正卡住的是 T_dirty 满窗 256 MiB 对 64 MiB 环——要么环 ≥ 768 MiB（F = 3），要么 T_dirty ≤ 170.7 MiB，要么记录装多个事务（乙）。
3. **乙不是候选条款**：它要改 D23（journal 的角色与格式） 已定项 7 的记录头（去掉事务号与提交标记、点名项各带事务号），那是 D16（发布语义） 已定项 3 定的独立冻结组件；它只是让「12.5% 与 4 倍」有个对照。
4. journal 写带宽在 2785 次 fsync 每秒的 seq 批 10 上是 1.7 GiB/s（两盘合计），与数据写带宽 5 GiB/s 之比仍是 12.5%。

### 它答不了的

1. 纯算术：不模拟环回绕、不量时间；祖先节点由哪条记录点名条款没写，甲取「并进末条」。
2. 乙的记录头改法是装置为对照造的，不是候选。
3. T_dirty 的「脏字节」算数据单元还是节点，D16（发布语义） 已定项 5 没写；甲、乙按数据单元，丙按 E91（已定格式下的环账与准入账） 的节点口径，三条不在同一口径上比。
4. 确定性：跑 N 遍一样。

## 历史版本

### 2026-09-13
- 新建。跑前登记写于装置之前；一次运行，判据与期望值没有改过。
