# E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 第 3 次重跑登记（事后补登）：第一段产物与 A31 核对产物换名，比对对象改成 10-30 那两份

写于 2026-11-08，**事后登记**：下面两件事在写这一份之前已经发生并已核过，这一份不是跑前登记，不对任何还没出的数立判据；它只给门禁 doc-experiments 的 replay-comparison-units 格一张固定形态的比对对象表。

<!-- doc-lint:not-numbers H4 H5 A31 -->

**这一份是什么**：差分重跑登记（`.claude/agents/experiment-designer.md`「做什么」第 4 步的写法）。臂、判据、岔路都不变；只写三样——变了哪些输入、哪几行判据或岔路要重判、其余各节写「同上一次登记」。不占号；r1（第 1 次重跑）、r2（第二段 H2 续跑）原样不动。

- 上一次登记（第一段、A31 的判据出处）：`research/prompts/e183-r1-prereg.md`，`sha256sum` 现量见第十三节。
- 准入：这一次不是因输入变化要新跑实验，E183 在准入登记里没有实验键，不接准入判。

**装置写在哪（写死）：不适用，这一份没有新装置、没有新跑。** 第一段产物是 `scripts/experiment-apparatus/replay.sh` 复跑一遍已入库装置得到的；A31 核对脚本是 `scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/small_disk_anchor_check.py`，纯 Python，同上一次登记。

**起跑前提**：不适用（没有要开的跑）。

## 一、问题

英文名：capacity_read_cache_reserve_and_sweep_costs

主 agent 给的问题（逐字）：给已跑出、已核对的产物补比对对象登记（产物换名与脚本补收尾行）；比对对象是 10-30 的两份，重跑产物是 11-08 的两份；判决不变。

读法写死：「比对对象」是 r1 登记比对对象表里的重跑产物（2026-10-30）；「重跑产物」是 `replay.sh` 现在复跑表第 4 列的产物（2026-11-08）。比的是整份字节（exact），不分行、不抹字段。

变了哪些输入（两件）：

1. 第一行产物 `…-h4-2026-11-08.out` 与比对对象只差一个字段名：第 6 行 `name=constant_compare` 里 `constant=sweep_threshold_placeholder_rows` 改名为 `constant=instance_table_row_count_threshold_default_rows`，值（`local=365 implementation=365 equal=true`）不变；其余 458 行逐字节相同。详情见 `/home/fy5090/.local/share/singlefs/work/replay-rows-relabel/result.md` 第 23 行、第 40 行、第 53 行。
2. 第二行 A31 核对脚本 `small_disk_anchor_check.py` 原来不打 `name=done` 收尾行，`replay.sh` 闸 2 判「跑不了」；补了一行 `print("E7RESULT name=done emitted=%d" % (len(devices) + 3))`。产物因此与比对对象差两处：第 1 行 `product_sha256=` 的值（读的产物换了），和末尾多一行 `name=done emitted=59`；56 份读数仍是 `mismatches=0`。

岔路单（逐行抄）：**重跑**——岔路单（`research/prompts/` 下无单独文件）只有一个问题：这两份产物换名与补收尾行之后，比对对象是哪两份、判决有没有变。

| 行 | 要回答的判断 | 翻面取值 |
|---|---|---|
| 1 | 第一行产物与 10-30 比，是否只差第 6 行那一个字段名 | `diff` 出现第 6 行以外的任何一行，或第 6 行的值不是 `local=365 implementation=365 equal=true` |
| 2 | 第二行 A31 输出与 10-30 比，是否只差 `product_sha256=` 的值与末尾收尾行 | `diff` 出现这两处以外的差，或 `mismatches` 不是 0 |

## 二、被测条款与它引的定义

实验页 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md` 的「复跑」一节（第 37 至 52 行）与「第 1 次重跑」一节，整段抄自 `scripts/kb/quote-kb.py`，出口文件 `/home/fy5090/.local/share/singlefs/work/e183-r3-prereg/quote.md`，回读逐字节一致：

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:81-126`（整段抄，未转述）**

````markdown
### 第 1 次重跑（清扫阈值的本地常量 369 改 365，只重跑第一段）

重跑登记 `research/prompts/e183-r1-prereg.md`（sha256 `9417c63368fbe1b6f6d0e04844214d07a98059ca1a2cd6bc409f608b5076fd59`；第十二节「修订」一段没有写）；问题单 `research/prompts/m3-close-b19-e183-threshold-365-questions.md` 两行。起因：C690（清扫触发阈值的默认值没定） 那一行写的默认值定为 365（用户 2026-10-29 定），`crates/singlefs-core/src/instance_table_sweep.rs` 第 74 行已是 `Self(365)`，装置里的本地常量 `SWEEP_THRESHOLD_PLACEHOLDER_ROWS` 还是 369，两条快档用例在 2026-10-30 红（登记第四节读数：回比用例报「本地 369 实现 365」，逐字节用例同时红）。只改了装置两行（常量值 369 改 365，文档注释改抄新的登记位；常量名与产物里的键名 `sweep_threshold_placeholder_rows` 不改），然后重出第一段 `all` 产物；第二段、第三段、第四段这一次不碰。

跑法与前提（都是这一趟现查的）：

- 起跑前提四样成立：`instance_table_sweep.rs:74` 是 `Self(365)`；装置 `grep -n SWEEP_THRESHOLD_PLACEHOLDER_ROWS` 命中第 78、1879 两行；两份 2026-10-23 旧产物的 `sha256sum` 与登记相同（`b4ff64f1…`、`da09f826…`）；`anchors_e183.py` 与 `small_disk_anchor_check.py` 的 `sha256sum` 与登记相同。
- 装置由 `scripts/hosts/compile-then-swap.py --release` 编过再整份换进主工作区（退 0）；二进制 sha256 `4130ccac6b3f66d0817e2f318362f558e65ac658de28fe3abc685e898078d190`；源文件在产物那一刻 sha256 `56a6b495d92699d42eb06cccd8843f54557c9b06ba893be452000bc23a168810`，与草稿副本逐字节相同，比换之前主工作区那一份（`b3b37608…`）只多那两行。`find crates -name '*.rs' -newer <二进制>` 在 `singlefs-core`、`singlefs-format`、`singlefs-harness` 下零命中（只列出 验证包里另几个文件）。
- 被测树：`git diff HEAD -- crates/ | sha256sum` 为 `e5f1218c991320c5456abbd300932d2aa1937a8790b7848fb2712de5deff0f0f`，`--stat` 末行「193 files changed, 54733 insertions(+), 5377 deletions(-)」，与登记第三节写的 `97a92443…`（182 个文件）不同，原因是别的会话在同一棵树上继续改；`all` 用到的 `singlefs-core` 没有比二进制新的源文件。
- `all` 跑两遍（并行，各经内存包装 8G、线程上限 31，装置单线程），都退 0，`cmp` 逐字节相同，sha256 `7c5add8502106e9822bdeea4ec806c5e3079c50aaa916bcd16d5b619227a7870`；两遍一起 226 秒。落成产物 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out`，459 行、152434 字节、`grep -vc '^E7RESULT '` 为 0、末字节是换行、末行 `E7RESULT name=done emitted=459`。
- A31 核对：`python3 -B -I …/small_disk_anchor_check.py …/anchors_e183.py <新产物>` 退 0，输出落成 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30-anchor-check.out`（58 行，sha256 `eaecf66cbfa6057b56bdf71e470d55998061388c6f5ad52c7fd307cc7b144c00`），末行 `E7RESULT name=verdict item=anchor_check readings_checked=56 mismatches=0`。

逐行差（问题单第 1 行；`diff 2026-10-23 产物 2026-10-30 产物` 的原样输出，1 个 hunk）：

```text
6c6
< E7RESULT name=constant_compare constant=sweep_threshold_placeholder_rows local=369 implementation=369 equal=true
---
> E7RESULT name=constant_compare constant=sweep_threshold_placeholder_rows local=365 implementation=365 equal=true
```

判定（比较脚本 `compare_e183_first_rerun.py` 的输出，脚本与输出在执行员草稿目录，不入库；纯 Python，十二条自检用例全过）：

| 量 | 判定 |
|---|---|
| 逐行差 | 只变第 6 行：恰好一个 hunk，头 `6c6`，旧行逐字等于登记引的旧第 6 行，新行逐字等于登记钉的新行 |
| 第 6 行新值 | 对：`local=365 implementation=365 equal=true` |
| 行数与字节数 | 459 行、152434 字节，与旧产物相同；非 `E7RESULT` 起头的行 0 条 |
| 回比判决行 | 对：`name=verdict item=constants constants_compared=21 constants_mismatched=0` |
| 判决行（下面「判决」一节那 11 行） | 11 行逐字段比，全部没变：H4 的判决行没变 |
| 两遍确定性 | 两遍逐字节相同 |
| A31 核对 | 对得上：退 0，56 份读数 0 处不符 |
| A31 输出差 | 只变产物 sha 那一行：1 个 hunk `1c1`，只差 `product_sha256=` 的值，新值等于新产物的 `sha256sum` |
| 两边是否一致 | 一致：产物只变第 6 行、核对输出只变头一行，没有互相对不上的地方 |
| 旧产物未被覆盖 | 对：两份 2026-10-23 产物的 `sha256sum` 跑完之后仍是登记里记的 |

产物里每一类结果行的条数与旧产物逐字相同（见「复跑」一节那条命令）；新产物里 `name=verdict` 共 11 条，整行与「判决」一节抄的那 11 行逐字相同，行号相同（第 23、26、49、120、197、289、369、395、421、447、458 行）。11 条里取值为 `false` 的字段是：第 120、197、289、369 行的 `positive_control_admission_holds=false` 与 `reading_checks_hold=false`，第 395、421、447 行的 `reading_checks_hold=false`，第 458 行的 `dominant_item_stable_across_default_disk_points=false`；原因与旧产物一样（空池一半 mkfs 先败、K32 空池 `before_a_publish` 入口按 G8 记、默认盘主导项跨 S 不稳定，见本页第一节「现状」），都是登记预期内的；没有 `not_run`，违例、不匹配、歧义、失败类的整数计数都是 0（`panicked_trials=0`、`anchor_mismatches=0`、`constants_mismatched=0`、`invalid_device_conditions=0`）。

答问题单第 1 行：常量改 365 之后第一段 `all` 产物只变第 6 行那一行比对，别的行不变，H4 的 11 条判决行不变。登记 P6 推的（只变第 6 行）与量出的一致。

换钉值与复跑登记：判决行没变、第 6 行对、A31 对得上，登记放行，所以复跑登记两行和 `driver_e183_anchor_check` 读的产物改指新文件（`scripts/experiment-apparatus/replay.sh` 第 1811、1812、3275 行），比对对象登记两行进 `.claude/gate.d/replay-comparison-units.tsv`；装置里钉的第一段 SHA-256 与行数（`SEGMENT_ONE_PINNED_SHA256` 改 `7c5add85…`，行数 459 不变）在草稿副本里改好，由主 agent 经 `compile-then-swap.py`（整条经 `run-with-memory-cap.sh 24G`）换进主工作区，退 0（换上之前主工作区那一份钉的是 `b4ff64f1…`，而装置的输出已是 `7c5add85…`）。旧的两份产物留着，不改名不删。

问题单第 2 行：A31 对得上（上表）。两条用例（`local_constants_equal_the_implementation_constants`、`segment_one_all_output_equals_the_registered_product_byte_for_byte`）是验证包的测试，执行员不跑，钉值换进主工作区之后由主 agent 在主工作区跑，结果「2 passed; 0 failed」（调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md` 2026-10-30「E183 U3 先编后换」那一条）；改前的红是上面登记第四节读数的那两条。

变异：这一次不新加变异行。装置里变异行 54 条，改完之后每一行的原文在装置里仍恰好命中一次（草稿副本与主工作区两份各数一遍，不是 1 次的行 0 条）；变异表里没有一行的原文含常量那一行或钉值。

````

**出处 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md:37-52`（整段抄，未转述）**

````markdown
### 复跑

```bash
# 仓根；入库装置，经 scripts/experiment-apparatus/replay.sh 的两行登记（driver_e183 编装置并跑 all，driver_e183_anchor_check 跑 A31 核对）；
# 验证包要编；这两行在门禁 research-build-and-replay 的 experiment-replay 格里跑
bash scripts/experiment-apparatus/replay.sh E183

# 只看装置本身（同一份源码，不经复跑登记）
cargo run -q --release -p singlefs-verification --bin e183_capacity_read_cache_reserve_and_sweep_costs -- all

# A31 独立核对，纯 Python
python3 -B -I scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/small_disk_anchor_check.py scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/anchors_e183.py research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out
```

登记行在 `scripts/experiment-apparatus/replay.sh` 里 E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 名下两行：驱动 `@driver_e183`、产物 `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-11-08.out`、判法 exact；驱动 `@driver_e183_anchor_check`、产物 `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30-anchor-check.out`、判法 exact。装置入口：`--list-items`、`--item <项>`、`all`；项名十个：`constants`、`positive-control-heap`、`anchors`、`h4-small-disk-slots-per-region-4`、`h4-small-disk-slots-per-region-8`、`h4-small-disk-slots-per-region-16`、`h4-small-disk-slots-per-region-8-journal-ring-4-mebibytes`、`h4-default-disk-slots-per-region-4`、`h4-default-disk-slots-per-region-8`、`h4-default-disk-slots-per-region-16`。每项耗时与退出码打到标准错误，不进产物。

````

## 三、实现今天的样子

- `scripts/experiment-apparatus/replay.sh` 第 2337 行：`E183|@driver_e183||e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-11-08.out|exact`；第 2338 行：`E183|@driver_e183_anchor_check||e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-11-08-anchor-check.out|exact`。
- `scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/small_disk_anchor_check.py` 第 61 行：收尾行（`git diff HEAD` 里这个文件只有这一行加，现量）。
- `.claude/gate.d/replay-comparison-units.tsv` 第 65、66 行现在还指 `e183-r1-prereg.md` 与 10-30 两个产物名，等主 agent 改成第六节末给的整行。

## 四、跑之前已经存在的数

事后登记，这一节的数都是已发生的（现量）：

| # | 数 | 来源 | 对判据的影响 |
|---|---|---|---|
| P1 | 比对对象 1：459 行、152434 字节、sha256 `7c5add8502106e9822bdeea4ec806c5e3079c50aaa916bcd16d5b619227a7870` | `sha256sum` / `wc` | 与 A31 读它时打的 `product_sha256` 相同 |
| P2 | 重跑产物 1：459 行、152449 字节、sha256 `04050c83f8fa8563df195fab5745c59cc66233e6d0bf6d00e1edfc72d794fd88` | 同上 | 与入库装置钉的 `SEGMENT_ONE_PINNED_SHA256` 相同（relabel 报告量的） |
| P3 | `diff` 比对对象 1 与重跑产物 1：只有第 6 行一行不同（字段名），多 15 字节 | `diff` 现量 | 岔路单第 1 行：未翻面 |
| P4 | 比对对象 2：58 行、sha256 `eaecf66cbfa6057b56bdf71e470d55998061388c6f5ad52c7fd307cc7b144c00`，已不在工作区，取 `git show 84b4ae23f^:research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30-anchor-check.out` | 现取现量 | |
| P5 | 重跑产物 2：59 行、19130 字节、sha256 `2335b00f63e73b1da2b3f3ba2b5f67268d1db57373ea6c227bf4a56c1ecc7f62`，末行 `E7RESULT name=done emitted=59` | 现量 | |
| P6 | `diff` 比对对象 2 与重跑产物 2：第 1 行 `product_sha256=` 的值（`7c5add85…` 换成 `04050c83…`）与第 59 行多出的收尾行，别无他差；倒数第二行 `readings_checked=56 mismatches=0` | `diff` 现量 | 岔路单第 2 行：未翻面 |

## 五、臂、阳性对照、真实基线

不适用：没有臂（没有新跑，只登记两份已有产物与其比对对象的关系）。阳性对照：不适用，理由同上；产物判得住坏的证据在 r1 登记的阳性对照与变异里，同上一次登记，这一份不改。

## 六、报哪些量与各自的判据

| 量 | 怎么算 | 门槛 | 对应岔路单行 |
|---|---|---|---|
| 第一行逐行差 | `diff` 比对对象 1 与重跑产物 1 | 恰一行不同，且是第 6 行，字段名之外 `local=365 implementation=365 equal=true` 不变 | 1 |
| 第一行行数 | `wc -l` | 459 | 1 |
| 第二行逐行差 | `diff` 比对对象 2 与重跑产物 2 | 恰两处：第 1 行 `product_sha256=` 的值、多一行 `name=done` | 2 |
| 第二行 `mismatches` | 重跑产物 2 倒数第二行 | 0，且 `readings_checked=56` | 2 |
| 收尾行 | 重跑产物 2 末行 | `name=done emitted=59`，59 等于产物行数 | 2 |

门禁侧的验收：`.claude/gate.d/replay-comparison-units.tsv` 第 65、66 行改成下面两整行（制表符分隔，六列）之后，replay-comparison-units 格对本份的比对对象表两行都对上：

```
E183	research/prompts/e183-r3-prereg.md	all	crates/singlefs-verification/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs	e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-11-08.out	driver_e183
E183	research/prompts/e183-r3-prereg.md	all	crates/singlefs-verification/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs	e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-11-08-anchor-check.out	driver_e183_anchor_check
```

## 七、钉绝对值的断言

出自被测条款本身的：第一行重跑产物 459 行、sha256 `04050c83…fd88`（条款：入库装置 `SEGMENT_ONE_PINNED_SHA256`、`SEGMENT_ONE_PINNED_LINE_COUNT`，relabel 报告第 53 行量过相同）；不符走第十节第 1 条。

独立算出并用命令核过的：P1 至 P6 的 sha256、行数，命令在第十三节；不符作废。

## 八、轨迹与几何敏感性

不适用：没有轮次，没有被谓词消费的量，也没有几何或旋钮取样点（只比两份已有产物）。

## 九、变异

不适用：没有给执行员的新变异。判别力的证据：往重跑产物 1 中间一行加一个字节，relabel 报告的 `--inject` 把逐字节比对翻成 `bytes_equal_after=False`（relabel 报告第 40 行附近量的）；这一份不另立。

## 十、失败条款

1. 第一行 `diff` 出现第 6 行以外的差，或第 6 行值变了：触发观测是 `diff` 输出多于一个 hunk。则「只差改名」不成立，两份产物不是同一结果换名，这一份作废、交回主 agent。
2. 第二行 `diff` 出现第 1 行与收尾行以外的差，或 `mismatches` 不为 0：则 A31 核对不再是「对得上」，同上。

## 十一、作废条款与停机条款

作废：P1 至 P6 任一个 sha256 或行数现量与上表不符；比对对象 2 取不到（`git show 84b4ae23f^:…` 退非 0）。

停机：`replay.sh` 第 2337、2338 行的产物名与第六节给的整行不一致，则不改 tsv，交回。

够判停机：岔路单两行都够判（第一行逐行差、第二行逐行差与 `mismatches` 都已量），没有没跑的量。

### 比对对象（这一次要与之比的上一次产物；一行一个正式跑）

| # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |
|---|---|---|---|---|---|---|
| 1 | `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out` | `7c5add8502106e9822bdeea4ec806c5e3079c50aaa916bcd16d5b619227a7870` | 459 | `replay.sh` 第 2337 行，驱动 `@driver_e183`，判法 exact；跑法装置 `-- all` | `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-11-08.out` | 只差第 6 行字段名；事后登记，产物已在树里 |
| 2 | `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30-anchor-check.out` | `eaecf66cbfa6057b56bdf71e470d55998061388c6f5ad52c7fd307cc7b144c00` | 58 | `replay.sh` 第 2338 行，驱动 `@driver_e183_anchor_check`，判法 exact；跑法 A31 核对脚本读重跑产物 1 | `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-11-08-anchor-check.out` | 比对对象已不在树里，取 `git show 84b4ae23f^:research/results/…-10-30-anchor-check.out`；差第 1 行产物 sha 与多一行收尾 |

## 十二、修订

留空。

## 十三、读过的文件与跑过的命令

读过：`.claude/gate.d/replay-comparison-units.tsv` 第 1 至 12 行、第 65 至 66 行；`.claude/gate.d/doc-experiments.sh` 第 3799 至 4140 行；`research/prompts/e183-r1-prereg.md` 第 1 至 27、245 至 268 行；`research/prompts/e183-r2-prereg.md` 第 1 至 30、345 至 353、484 至 500 行；`/home/fy5090/.local/share/singlefs/work/replay-rows-relabel/result.md` 全文；`.claude/kb/checks-owed.md` 里 C880、C690、C951 三行（其余是 grep 命中行）；`scripts/experiment-apparatus/replay.sh` 第 2335 至 2339 行；`.claude/kb/experiments/183-…md` 的节标题与 grep 命中行（`A31`、`11-08`）。

命令（仓根）：

- `sha256sum research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-11-08.out research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-11-08-anchor-check.out`
- `git show 84b4ae23f^:research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30-anchor-check.out`（落草稿目录 `old-anchor.out`）后 `sha256sum`、`wc -l`、`diff`
- `diff research/results/…-h4-2026-10-30.out research/results/…-h4-2026-11-08.out`
- `git diff HEAD -- scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/small_disk_anchor_check.py`
- `sha256sum research/prompts/e183-r1-prereg.md`，读数 `9417c63368fbe1b6f6d0e04844214d07a98059ca1a2cd6bc409f608b5076fd59`（r1 登记比对对象表 #1、#2 是 10-23 到 10-30，不动）。

## 十四、并行与挂钟

独立单位：不适用的跑，没有新跑；两份产物的 `diff` 与 `sha256sum` 互相独立，各一条命令。
线程：1 条线程即可，登记值 `.claude/heavy-thread-default` 是 31，这里不起重活，不套这个数。
串行：无，两个 `diff` 与两个 `sha256sum` 互不依赖，都能并行；它们都是毫秒级。
预计挂钟：不到 1 秒，四条命令都是读 150 KiB 以内的文件（推的：459 行、152449 字节）。
预计在用核数：1 个核，门禁 replay-comparison-units 格另算（它是 python3 单进程读表）。
