# 删除线第 60、61 批代码轮第一轮：攻方腿（Opus）报告（2026-10-11）

攻击面 P1–P5（正文第三节），重点 P4、P5。代码行号按冻结副本 `~/.local/share/singlefs/work/m3-batch60-61-code-r1/repo/`（下面写 `crates/…`；
这几份文件此刻与主工作区逐字节相同，sha256 前 12 位核过）。副本上量出的数只算副本上的数，不算入库装置上的数。

## 复跑

模型目录 `research/prompts/m3-batch60-61-code-r1-opus-model/`。每个阶段单独起（机器满载时一条前台命令只够跑一段）：

```
cd ~/code/singlefs
W=~/.local/share/singlefs/work/<新目录>
bash research/prompts/m3-batch60-61-code-r1-opus-model/rerun.sh $W list
OPUS_MEMORY_CAP=4G bash research/prompts/m3-batch60-61-code-r1-opus-model/rerun.sh $W base-build
OPUS_MEMORY_CAP=4G bash research/prompts/m3-batch60-61-code-r1-opus-model/rerun.sh $W base-batch base-h1 base-h2 base-h3 base-h4
OPUS_MEMORY_CAP=4G bash research/prompts/m3-batch60-61-code-r1-opus-model/rerun.sh $W mut-mv mut-mi-pipe mut-mi-given mut-mp
OPUS_MEMORY_CAP=4G bash research/prompts/m3-batch60-61-code-r1-opus-model/rerun.sh $W p2-base p3-base mut-tsv3957 mut-tsv3958 mut-tsv857 mut-tsv2755
OPUS_MEMORY_CAP=4G bash research/prompts/m3-batch60-61-code-r1-opus-model/rerun.sh $W mut-row-65 mut-row-396 mut-row-657 mut-row-2232 compare
OPUS_MEMORY_CAP=4G OPUS_COST_SIZES=0,3000,30000,120000 bash …/rerun.sh $W cost-build      # 没建完重起，接着建
OPUS_MEMORY_CAP=4G OPUS_COST_SIZES=<一档> OPUS_WHOLE_LIMIT=3000 bash …/rerun.sh $W cost-measure
OPUS_MEMORY_CAP=4G OPUS_RESUME_STORE=/dev/shm/<目录> OPUS_RESUME_FRACTIONS=0.05,0.2,…,1.5 bash …/rerun.sh $W resume
```

compare 只比 `expected/` 里确定的那几份；`measured/` 下是计时与杀进程时刻定的（P5 读次数、P4 断了重跑），只落盘、不比。
种子：本腿把实现员编译目录里已编好的 `librocksdb-sys` 三个单元拷进自己的编译目录当种子（`root-output` 让 cargo 认新位置），不拷也行，只是冷编一遍 C++。

模型文件 sha256（交回前算的）：

```
caa8294fa130624eb09fbdb7a1c63d2cc8a0f2cb7a28cab71c6071010b3f27be  compare.py
55155d4f1e478785d828cfad022fa60e4ce874d2f1c770fada3fc8e929091e72  extract.py
6c7eeaf43d651b23feaae75a7c1931b360b01c29b94840b294b12ca2bbc27e27  mutants.py
a47ae7664b5060944bf1d2b97f49be4b37765da7246c89a6ac24649d0b8ecde7  probe-crate/Cargo.toml
2dd4558beab2623618c12ee104dbccec2e33ecfe0d63cf677832b59ee7da85ef  probe-crate/src/lib.rs
e9c0decebd57546ea8cb6581729c235cbf283315a6b48bba430e936b26064f1e  probe-crate/tests/planned_import_store_probe.rs
0ef457c28389d5a1b2e0ae90f7f175bca99bc8b1df99e93c06e995baede3b709  probes/pipeline_import_append.rs
a5e2bf3d3d2254f2829903c4af71c513e766fd0f81c847d2bb43b700bc409c90  measured/cost-measure.txt
56633de9e4f054f170a148e6ac9d8935b7bc8735a093c19c65c453877f82065d  measured/resume-a.txt
94e499abcafc0ff54fc29b9122c6ad461619ff03ad9d6ace671e46077807b856  measured/resume-b.txt
（rerun.sh 与 expected/ 下 19 份的 sha256 见文末「交回前的 sha256」一节，rerun.sh 在写报告时还改过注释）
```

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| P1 精确红集与 k = 2 变异 | 没打中只读码 | CPU 红集只由 CPU 位攒（GPU 内核变异碰不到它），三行 k = 2 变异只能红在逐位不一致那一条；上一轮攻方模型（旧代码）量过 [2,1,2,2,1] 三条流上别的不变量一个状态都不红，这一轮没重跑 |
| P2 统计量 7 与分配记录判序 | 没打中且量过 | 两份目标在副本上 13 + 43 条全绿；N3 两行变异各只红自己那一条（8 条判序用例里各红 1 条） |
| P3 轻型核心行的修补 | 没打中且量过（checker 档三行只读码） | 两条新取样点各被点名变异红在点名的断言上；6 行 harness 档的替换行在副本上都红在点名用例；根槽清零与「挂载那一刻读不出」在根环表里同一类 |
| P4 按计划导入的正确性 | **打中（改坏照绿）**；读回正确性、断了重跑、给定状态表那一路没打中且量过 | 流水线导入那一段排键时一行核对行都不点（mv），第 60、61 批新增与改过的用例全绿；同一个变异在副本上开了核对的历史里让导入那一趟 `blocks_without_verifier` 从 0 变 1、读回与整库导入差一行。按计划导入与整库导入在 8 段历史上逐项相同；杀进程 16 个时刻之后重跑都补齐 |
| P5 代价与钉值 | 代价：没打中且量过；钉值：**打中（改坏照绿，小）** | 对方库累计 0→120000 块别的流，按计划导入的 Get 次数 193、seek 48 不动；`imported_blocks` 只数写下的（不数本库已有且相同的）两条新用例都照绿 |

## P4　按计划导入的正确性

### P4-1（打中，改坏照绿）：流水线导入那一段排键时漏掉核对行，第 60、61 批的用例一条都不红

改坏处（`mutants.py` 的 `mv`，在冻结副本的拷贝上）：`crates/singlefs-checker-tier/src/crash_amplification.rs:7770`「for verifier_digest in &verifier_digests {」
换成「for verifier_digest in verifier_digests.iter().skip(verifier_digests.len()) {」（两种核对代码都不点）。

被判用例为什么看不见：CPU 配置的趟不核，`crates/singlefs-checker-tier/src/crash_amplification.rs:8083`「|| std::env::var("SINGLEFS_CRASH_AMPLIFICATION_VERIFY").is_ok_and(|value| value == "cpu")」，
第 61 批新加的流水线用例 `importing_the_other_share_names_its_blocks_and_reads_back_like_one_host_judging_the_whole_flow` 不设它、只比结论；
verdict_store 那条对拍自己造 `PlannedImportKeys`，碰不到流水线排键那几行；第 61 批追加的变异 10 行里没有这一行。
配了 GPU 的真趟一律核，导入之后核对行缺就在重型用例上红：`crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs:348`「if run.share.count == 1 || imported {」、
`:350`「comparison.blocks_without_verifier, 0,」。经卸载那一份用例文件（crash_amplification_fixed_script_through_unmount.rs）第 438、440 行同形。这两处只有双机驱动的真跑走得到。

副本上的原样输出（`expected/mut-mv.txt`，命令 `rerun.sh $W mut-mv`）：

```
command args=-p opus-probe-b6061 --test pipeline_import_probe -- --nocapture --test-threads 1 --exact importing_the_other_share_names_its_blocks_and_reads_back_like_one_host_judging_the_whole_flow given_states_tables_of_another_host_are_imported_by_name_and_read_back_like_the_whole_library_import rc=0
binary pipeline_import_probe
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out
OPUS_P4 history=h02 differs planned: verifier node=crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs::publish_first_file::981bc93dfbca3240dc9620468dc7a8815dd09bf4dc55609e7ce9219adb7af87e start=0 row=None facts_bytes=None
OPUS_P4 history=h02 readback_lines=10 planned_vs_whole=1 planned_vs_one_host=1 whole_vs_one_host=0
OPUS_P4 history=h02 step=planned-import-0 share=0/2 checked=0 imported=2 unjudged=0 violations=0 stopped=false compared=33 without_verifier=1 without_judge=0 disagreements=0
```

没改坏的副本上同一段历史（`expected/base-h1.txt`）：

```
OPUS_P4 history=h02 readback_lines=10 planned_vs_whole=0 planned_vs_one_host=0 whole_vs_one_host=0
OPUS_P4 history=h02 step=planned-import-0 share=0/2 checked=0 imported=2 unjudged=0 violations=0 stopped=false compared=45 without_verifier=0 without_judge=0 disagreements=0
```

h02 是第 61 批那条用例的形状（本机 0/2 先跑、第二台 1/2 另起一份库、本机带 IMPORT 再跑），只多设了 `SINGLEFS_CRASH_AMPLIFICATION_VERIFY=cpu`。
双机驱动的形状 h03（本机先判着第二台那一份、第二台从本机库的快照起步）在 `mv` 下照样逐项相同（`readback_lines=10 planned_vs_whole=0`）：
第二台那一块本机先判着时已经核过；所以要钉这一行，取 h02 那种第二台那一块只在对方库里核过的形状。

四句：
1. 分不分辨臂：分辨。整库导入逐行抄核对表，这一行漏了只有按计划导入那一臂缺（同一副本上 `whole_vs_one_host=0`、`planned_vs_whole=1`）。
2. 系统当时看得到吗：看得到。排键时手里有这一趟编进来的核对代码摘要（同一段开头的 `verifier_digests`），对方库里那一行也在（整库导入读得到）。
3. 满足的是哪一个分句：正文 P4「核对行与事实行带没带全」，加题面「改坏照绿」。不是「读回与整库导入不同」——没改坏的代码读回相同（下面 P4-2）。
4. 改法在这几格上中不中：跑前条款没给改法。我提的（只在我的模型上量过、被攻过零轮）：第 61 批流水线导入用例再设 `SINGLEFS_CRASH_AMPLIFICATION_VERIFY=cpu`，
   断言导入那一趟 `comparison.blocks_without_verifier == 0`、按块读回核对行与整库导入（或一台判完那份库）相同，变异表追加 `mv` 这一行。
   量过：h02 在 `mv` 下 `without_verifier=1`、在原样代码下 0（上面两段输出）；没在被判用例文件里实现、没跑那条改过的用例，那一步是推的。
   GPU 核对代码那一把摘要（`#[cfg(feature = "gpu")]` 那一支）漏掉时 CPU 配置的用例照样看不见，只有配了 GPU 的趟看得见：推的，没量（不起 GPU）。

### P4-2（没打中，量过）：按计划导入之后的读回与整库导入逐项相同

原型（`probes/pipeline_import_append.rs`，接在拷进原型 crate 的被判用例文件末尾）在同一条新池新建文件流上把用户定的几步放开扫，
每段历史末尾从同一份本机库拷两份：一份带 IMPORT 再跑（按计划导入），一份先 `import_from` 整库导入再不带 IMPORT 跑；
再与一台判完那份库比：结论（GPU 判器、CPU 判器、观察者，按判定向量内容读回）、CPU 核对代码的核对行与它点名的事实行、计划里每条路径。
扫的维度：份数 2 / 3、本机那一份在导入之前先跑与否、第二台从本机库快照起步与否、本机先判着第二台那一份与否、导入那一趟跑一次或两次、
发布那个点 ignore 与否、核与不核。8 段历史（h01–h08，定义见文件里的 `OpusHistory` 与 8 个用例），原样输出（`expected/base-h1.txt`…`base-h4.txt` 里的汇总行）：

```
OPUS_P4 history=h01 readback_lines=10 planned_vs_whole=0 planned_vs_one_host=0 whole_vs_one_host=0
OPUS_P4 history=h02 readback_lines=10 planned_vs_whole=0 planned_vs_one_host=0 whole_vs_one_host=0
OPUS_P4 history=h03 readback_lines=10 planned_vs_whole=0 planned_vs_one_host=0 whole_vs_one_host=0
OPUS_P4 history=h04 readback_lines=10 planned_vs_whole=0 planned_vs_one_host=0 whole_vs_one_host=0
OPUS_P4 history=h05 readback_lines=10 planned_vs_whole=0 planned_vs_one_host=0 whole_vs_one_host=0
OPUS_P4 history=h06 readback_lines=7 planned_vs_whole=0 planned_vs_one_host=0 whole_vs_one_host=0
OPUS_P4 history=h07 readback_lines=10 planned_vs_whole=0 planned_vs_one_host=0 whole_vs_one_host=0
OPUS_P4 history=h08 readback_lines=10 planned_vs_whole=0 planned_vs_one_host=0 whole_vs_one_host=0
```

每段导入那一趟都是 `unjudged=0`、`without_verifier=0`（核的那几段）。排键的全局序号：排键那一段与 `my_blocks` 都按 `plan.points` 逐点 `point.blocks()` 数
（`crates/singlefs-checker-tier/src/crash_amplification.rs:7761`「if selected.contains(&this_block) {」，核对、观察者、对照几处同一个数法），
ignore 的点（h06）也照这个次序数，读回相同。路径：本机录入时已经按计划登了同一份身份（`crates/singlefs-checker-tier/src/crash_amplification.rs:1122`「store.record_path(&point.path_number, &point.identity)?;」），
所以排键时一条路径都不点（`mutants.py` 的 `mp`）在 h02、h03 上读回照样相同（`expected/mut-mp.txt`），这一项是等价的，不算漏。

规模：这条流按被判用例的展开上限 6 只有 45 个状态（输出里 `planned_states=45`）；全量展开的闭式状态数 16777240（`closed_form_full=`），原型没跑全量。

### P4-3（没打中，量过）：成批写入中途被杀，重跑同一条命令补齐、块内原子

`probe-crate/tests/planned_import_store_probe.rs` 的 `opus_resume_after_a_kill`：对方库 40 块、每块 4096 个状态带恢复字，40 种判定向量在本机库里倒序登记（编号与对方不同），
本机库前 10 块已有且相同；子进程（同一个测试二进制）做按计划导入，父进程在开始之后不同时刻 SIGKILL，再开库逐项读、同一次导入重跑、与不断那一份比。
为了在小库上攒出好几批，副本把 `crates/singlefs-checker-tier/src/verdict_store.rs:1024`「const PLANNED_IMPORT_BATCH_BYTES: usize = 64 << 20;」改成 256 KiB（`mutants.py` 的 `rb`；批的边界与写法不动），
库放 tmpfs（杀的是进程，不是断电；盘上同步写在别的会话满载时排几十秒）。不断那一趟 18 批；整库导入与它逐项相同：

```
OPUS_P4R reference seconds=0.016 written=30 identical=10 missing=40 paths=40 verifier=40 facts=1 rows=441 synced=18
OPUS_P4R reference_lines=161 present=40 whole_vs_reference=0
```

16 个杀的时刻（`measured/resume-a.txt`、`resume-b.txt`），杀完读得到的结论块数 10、14、16、18、24、28、30、32、34、40 都出现过，每一次
`read_errors_after_kill=0 present_but_different=0`，重跑 `rerun=Ok(…)`、`rerun_vs_reference=0`。原样一行（`resume-a.txt`）：

```
OPUS_P4R fraction=0.8 child_finished_before_the_kill=false status=signal: 9 (SIGKILL) present_after_kill=28 read_errors_after_kill=0 present_but_different=0 rerun=Ok("written=12 identical=28 missing=40 paths=40 verifier=40 facts=1 rows=225 synced=9") rerun_vs_reference=0
```

读码：一块一种结论的行连同它新登记的判定向量放进同一批，攒满才写（`write_the_planned_import_batch_when_full` 只在放完一块之后判），所以块不跨批；
判定向量的内存表在那一批写下去之后才进。冻结副本原样的 64 MiB 批长那一档（`resume-frozen`，120 块 65536 个状态、约 4 批）机器满载时没跑完，见「限度」。

### P4-4（没打中；用例在副本上跑过，其余只读码）：给定状态表那一路

- 该导的表被现判：导入只对不归本机的表做（`crates/singlefs-checker-tier/src/crash_amplification.rs:9398`「if !self.libraries_to_import_from.is_empty() && !self.owns_the_table(&key) {」），
  导完库里还没有就报对账不为空、不现判（`:9417`「这张表归别的机器判，导入了那一台取回的库之后库里还是没有它的结论」）。归本机的表从不导、照本库复用或现判，
  与流水线那一侧不点本机那一份同一个分法。双机驱动给第二台的库是本机库的快照（`scripts/crash/crash-amplification-two-hosts.sh:1040`「cp -a -- "$local_library" "$library_for_peer"」），
  本机先判着、归第二台的表第二台照快照复用，不会两边各判一份、判在不同的卡上再撞「块属性不同」。
- 导入失败被吞：导入报错经 `?` 交回 `judge_the_given_states`，崩溃注入两处调用方都 panic（`crates/singlefs-checker-tier/src/crash_injection.rs:2086`「"第一截给定状态表 {} 判不了（不退回别处判）：{reason}",」、
  `:2194`「"第三截给定状态表 {} 判不了（不退回别处判）：{reason}",」），没有吞。
- 锁序：先本机库、再要导入的库（`crates/singlefs-checker-tier/src/crash_amplification.rs:8878`「先拿本机库的锁、再逐份拿要导入的库的锁（几个引擎都是这个次序）。」）；
  要导入的就是本机这一份时开跑就拒（`:8750`「if other.key == library.key {」）。同一进程里两个引擎互为对方的本机库与要导入的库才会反序，
  而 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 与 `…_IMPORT` 都是整个进程一份，没设库时各引擎的临时库名带进程号与序号，造不出这一形。推的，没量。
- 第 61 批那条给定状态表用例在副本上跑绿（`expected/base-batch.txt`：`test result: ok. 2 passed`）。

## P5　按计划导入的代价与钉值

### P5-1（没打中，量过）：导入的读次数不随对方库里累计的行数涨

`opus_cost_build` / `opus_cost_measure`：对方库 B 里放流 F 的 64 块（当前版本 CPU 结论、观察者、核对行、事实行、路径，外加 F 在一把旧版本下的 64 块），
再加 N 块别的流（结论与核对行）；本机库 A 只有 F 的偶数块。照流水线排键的样子点 F 的奇数块（两种判器、挂观察者的观察者一把、两种核对代码）与 F 的 64 条路径。
数的是这一线程上同一份 librocksdb 的 PerfContext：`gets`=GetFromMemtableCount（每次 Get 都过一次 memtable）、`seeks`=SeekOnMemtableCount、`nexts`=NextOnMemtableCount，
`block_reads`、`key_comparisons` 是每次读在 LSM 里走多深。原样（`measured/cost-measure.txt`，按 N 排）：

```
OPUS_P5 noise=0 planned=first seconds=1.0014 written=48 identical=0 missing=32 paths=0 verifier=32 facts=1 rows=433 synced=1 gets=193 seeks=48 nexts=0 block_reads=156 block_read_bytes=207226 cache_hits=281 key_comparisons=10013 keys_skipped=304 get_read_bytes=725900
OPUS_P5 noise=3000 planned=first seconds=0.9912 written=48 identical=0 missing=32 paths=0 verifier=32 facts=1 rows=433 synced=1 gets=193 seeks=48 nexts=0 block_reads=162 block_read_bytes=212623 cache_hits=676 key_comparisons=12687 keys_skipped=304 get_read_bytes=725900
OPUS_P5 noise=30000 planned=first seconds=1.0462 written=48 identical=0 missing=32 paths=0 verifier=32 facts=1 rows=433 synced=1 gets=193 seeks=48 nexts=0 block_reads=170 block_read_bytes=217732 cache_hits=1278 key_comparisons=17970 keys_skipped=304 get_read_bytes=725900
OPUS_P5 noise=120000 planned=first seconds=0.9649 written=48 identical=0 missing=32 paths=0 verifier=32 facts=1 rows=433 synced=1 gets=193 seeks=48 nexts=0 block_reads=171 block_read_bytes=214979 cache_hits=1149 key_comparisons=18191 keys_skipped=304 get_read_bytes=725900
OPUS_P5 noise=0 planned=again seconds=0.0145 written=0 identical=48 missing=32 paths=0 verifier=0 facts=0 rows=0 synced=0 gets=289 seeks=96 nexts=304 block_reads=0 block_read_bytes=0 cache_hits=437 key_comparisons=9142 keys_skipped=608 get_read_bytes=1449516
OPUS_P5 noise=120000 planned=again seconds=0.9839 written=0 identical=48 missing=32 paths=0 verifier=0 facts=0 rows=0 synced=0 gets=289 seeks=96 nexts=304 block_reads=0 block_read_bytes=0 cache_hits=1320 key_comparisons=17213 keys_skipped=608 get_read_bytes=1449516
```

N 从 0 到 120000（对方库 1.9 MB → 370 MB、SST 0 → 9 个）：Get 193 次、seek 48 次、写一批，四档逐字相同；再导一次（全是本库已有且相同）Get 289、seek 96、next 304，两档相同。
随 N 涨的只有每次读在 LSM 里的深度：`key_comparisons` 10013 → 18191、`block_reads` 156 → 171（没布隆过滤器：`crates/singlefs-checker-tier/src/verdict_store.rs` 的 `library_options` 除了建库开关只设了压缩），
与「每次 get 在 LSM 里是 log L」那句推的一致。挂钟（`seconds=`、开库的 `open_seconds=`）是在别的会话把机器压到负载 50–130、CPU 压力 60–85% 时量的，
`nice -n 19` 下不作数；开库那一步读的是 WAL（量到 16.8 MB 的那一档开了 28 秒，1.8 MB 的那一档 3.5 秒），WAL 的上限是写缓冲，不随库里累计的行数涨（推的）。
对照的整库导入只量了 N = 0 那一档（`whole seconds=30.552 imported=144 gets=416 seeks=183`），更大的档前台时限里跑不完，没量。

旧 `import_from` 留着的那一处产品调用（对照旁库导回主库，`crates/singlefs-checker-tier/src/crash_amplification.rs:9851`「let imported = main_store.import_from(&self.store)?;」）：
旁库每一趟现建（`:9818`、`:9822` 先删同名目录、`create_empty`），抄的是这一趟对照判的块，主库那一侧每行点读，不随主库累计的行涨。只读码，没量。
双机驱动拷给第二台的整库快照（上面 `:1040` 那一行）随本机库涨，是调查报告「次一步」点名、这一批没做的，不算这一格。

### P5-2（打中，小，改坏照绿）：`imported_blocks` 的新含义没有用例钉

`imported_blocks` 这一批改成写下的加本库已有且相同的：`crates/singlefs-checker-tier/src/crash_amplification.rs:7787`「tally.conclusion_blocks_written + tally.conclusion_blocks_already_identical;」，给定状态表那一路在第 8910 行同形。
两条新用例钉的 2 与 4（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1307`「importing_run.imported_blocks, 2,」、`:1487`「(4, 4, 0, 0),」）
都是在本库一块都没有的时候数的，写下的就是全部，分不出两种含义。把相加的第二项乘 0（`mutants.py` 的 `mi-pipe`、`mi-given`），两条用例照绿：

```
command args=-p opus-probe-b6061 --test pipeline_import_probe -- --nocapture --test-threads 1 --exact importing_the_other_share_names_its_blocks_and_reads_back_like_one_host_judging_the_whole_flow given_states_tables_of_another_host_are_imported_by_name_and_read_back_like_the_whole_library_import rc=0
binary pipeline_import_probe
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out
```

（`expected/mut-mi-pipe.txt` 与 `expected/mut-mi-given.txt` 第一段都是这三行。）同一个变异在导入跑两次的历史 h07 上，第二次导入的 `imported=` 从 2 变 0（`mut-mi-pipe.txt`）：

```
OPUS_P4 history=h07 step=planned-import-1 share=0/2 checked=0 imported=0 unjudged=0 violations=0 stopped=false compared=45 without_verifier=0 without_judge=0 disagreements=0
```

原样代码那一行是 `imported=2`（`expected/base-h4.txt`）。四句：不分辨臂（只管汇总行的数，读回不受影响）；看得到；满足正文 P5「改坏会不会红」；
改法（被攻过零轮、没实现）：第 61 批那两条用例各在导入之后再导一次，钉第二次的 `imported_blocks` 等于第一次（全是已有且相同），这一步推的、没量。
影响面小：入口脚本拿 `imported_blocks>0` 认导入行，双机驱动另在行尾加 `import_run=true`，重导一遍报 0 不会把那一行认成分份行（`scripts/crash/crash-amplification.sh:79`「导入行（行尾 import_run=true，或带 imported_blocks>0）不是分份行，它留的块照算。」；推的，没跑那个入口）。

另：几份要导入的库里同一块都在时，`imported_blocks` 每份各数一次（h05 三台、两份库都带本机快照，报 `imported=5`，这条流不归本机的结论块是 3 块）。旧 `import_from` 同样每份各数，不算这一批的回退，记一句。

### P5-3：21 行那一组钉值

`crates/singlefs-checker-tier/src/verdict_store.rs:4032`「assert_eq!(written_by_plan.len(), 21, "按计划导入写下 21 行");」之后紧跟逐项集合相等
（`:4034`「written_by_plan, other_beyond_a_of_flow_f,」），后者由两份库的全部键值行另算，不经导入的账；批长判据反过来那一行变异改的是 `synced_writes`，那一格也钉了。
没在副本上跑（库单测是 checker 档测试，不跑）。只读码：没找到改坏照绿的写法。

## P1　GPU 远左短片与精确红集（没打中，只读码）

- 精确红集只由 CPU 那一侧攒：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:2562`「if *cpu_checker_bits != 0 {」，
  下面 `:2565`「.cpu_checker_red_invariants」；断言在 `:14289`「std::collections::BTreeSet::from([knob_capacity_invariant]),」。GPU 扫描内核的改坏碰不到 CPU 位，
  所以 k = 2 那三行（`crates/mutations.tsv:3954`「第60批 N1 GPU 判器：旋钮那一支只判最右之前两片（墓碑树 [2, 1, 2, 2, 1]、根环转过一圈；攻方 M1-4）」及下两行）
  只能红在其后的逐位不一致那一条，红不到精确红集那一条。内核那一支是 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3374`「if (d_container_record_count(container_index) != knob_records) {」，
  外层循环到倒数第二片为止；变异加的 `container_index + 3u >= d_container_count` 只留倒数第三、第二片，[2,1,2,2,1] 上第 2 片（下标 1）漏判，[2,1,2,1] 上不漏——三条新流分得出、旧三条分不出，与正文说的一致。
- 精确红集在正确判法下会不会因造法顺带的别的违例而红：上一轮攻方模型在上一轮冻结副本上量过同形三条流全部状态，
  `research/prompts/m3-batch59-code-r1-opus-model/expected/base-m1-l2.txt:8`「OPUS_M1 judged flow=tombstone-2-1-2-2-1-ring states=609 I-9.22_violated_states=477 I-9.22_held_states=72 states_with_any_violation=477 states_with_another_violation=0 digest=909adeca291ab0a7」（谱系、退役同文件第 6、7 行，都是 `states_with_another_violation=0`）。
  这是旧代码上的数（这一轮的底合进了根环线 E9–E11），只当参考；这一轮没重跑（GPU 那份用例的造法在 checker 档测试文件里，CPU 替身要照新底重抄一遍，机器满载、排在 P4、P5 之后没做）。
- L1 三条收成造法结构：用例名不变（diff 里三条 `#[test]` 名字原样），第 59 批那三行变异点名的用例还在；那三行改坏的仍是同一个内核分支，
  精确红集断言在前、CPU 那一侧不受内核改坏影响，照旧红在逐位不一致那一条。推的，没跑（GPU 用例，不起 GPU）。

## P2　水位统计量 7 与分配记录判序（没打中，量过）

- 副本上两份目标全绿（`expected/p2-base.txt`）：`test result: ok. 13 passed` 与 `test result: ok. 43 passed`。
- 补进来的那一行（`crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs:605`「statistic: STATISTIC_EXTENSION_POINT_QUOTA_USED,」）标签登记过、两维都不带，
  照 `.claude/kb/decisions/05-快照-空间记账机制.md:124`「| 7 | 扩展点配额已用量 | 准入（D21（权威态与派生态的分界） 已定项 17 第四条；C21（扩展点指向逃出记账）） | 不带 | 不带 |」；
  它借的是已定项 8「没有对象的统计量不写行」今天不判（`.claude/kb/decisions/05-快照-空间记账机制.md:204`「规则是有值的统计量就写一行，**没有对象的统计量不写行**」），用例注释自己写明了读者将来判它就要换造法。
  前提断言（`:629`「"台阶行之外每一行的代段都是这一版的 txg（I-3.13 照过）"」）在副本上照过。没找到读者在别的理由上先拒这一行的路径（用例断言的是「水位行缺」那一个失败成员，先拒就红，不会在错的理由上绿）。
- N3 两行变异各只红自己那一条（`expected/mut-tsv3957.txt`、`mut-tsv3958.txt`，各跑判序那 8 条）：

```
  failed allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_width_by_the_recovery
test result: FAILED. 7 passed; 1 failed; 0 ignored; 0 measured; 35 filtered out
  failed allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_width_by_the_writable_mount
test result: FAILED. 7 passed; 1 failed; 0 ignored; 0 measured; 35 filtered out
```

  冷走读那一行红在「EntryNarrowerThanItsFieldTable { what: "记账条目", entry_bytes: 22, field_table_bytes: 34 }」上，正是挪到条目宽之后该报的；另 7 条（含上一批钉逐条判、数条目两个次序的）照绿，「只由这张盘分得出」成立。

## P3　轻型核心行的修补（没打中；harness 档量过，checker 档三行只读码）

- 两条新取样点：`实审A1 第19条：取号写之前不过屏障` 那一行（变异表第 857 行）在副本上只红新加的回卷之后再取号那一条（`expected/mut-tsv857.txt`：
  「left: [SecondSystemConfigurationWriteInOneSegment { operation_index: 86, device: DeviceIdentity(0) }]」，同文件另 3 条照绿）；
  `第10g批 影子账候选过滤改回一圈窗口` 那一行（第 2755 行）只红抬 F 那一条（`expected/mut-tsv2755.txt`：「left: [(DeviceIdentity(0), 1088), (DeviceIdentity(1), 1088)]」，可写挂载取样的那一条照绿，与改过的注释说的一致）。
- 根槽清零再写回是不是硬造：同一份文件造被抛弃的根本来就用「清零、挂载、写回」表示一时读不出（`crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs:376`「mount_writable(&parameters, &mut devices).expect("择根落到前一条根，照常可写挂载");」之前那一段）；
  核心层把「挂载那一刻读不出」与「自证不过」（全零槽属于后者）放进同一类（`crates/singlefs-core/src/recovery.rs:1399`「或自证不过 [`BadRootRingSlotReading::NotSelfVerified`]）的槽不在 `ring_slots_known_to_hold_a_root` 里（挂载那一刻就读不出或自证不过、」），
  产品在挂载那一刻读那一槽报错（瞬时读错）、之后读得出，走到的是同一个根环表状态。所以清零是可达历史的替身，不是只有清零才造得出的形。读码，没用读故障重造一遍。
- 8 行替换里 harness 档的 6 行在副本上都红在点名用例：第 65、396、657、2232 行（`expected/mut-row-*.txt`，各一条 `failed …`），加上面 857、2755 两行。
  checker 档的三行（第 84 行「步 6」、第 2728 行「E171R1」、第 3959 行改名的「末条标志」）是重型目标，不跑；只核了锚点在冻结副本里恰好命中一次、点名的用例与 bin 函数在
  （`crates/singlefs-core/src/transaction.rs:9155`「let place_in_publish = if transaction_of_the_next_record.is_none() {」一处，点名用例在 `crash_enumeration_multi_record_publish_stream.rs` 里一处）。
  改名那一行换成 `if true` 之后每条记录都带末条标志，崩在记录之间时恢复会把半次发布施加上去，正是点名用例断言的那件事：推的。

## 没打中的形状（试过什么、取样多大）

- 读回不同：8 段历史 × 每段 10 行读回（ignore 那段 7 行），份数 2 / 3、导入先后、快照起步、先判着第二台那一份、导入一次 / 两次、ignore、核与不核；一行不同都没有。
  只有一条新池新建文件流、展开上限 6（45 个状态）；多卡、GPU 配置、对照那一遍（旁库）没取样。
- 「本库已有且相同就跳过」比漏：比的是块、违例、计数向量、恢复字，再比块属性（`crates/singlefs-checker-tier/src/verdict_store.rs:2183`「if stored.judged_on != requested.judged_on {」），
  键里带版本，旧判法的结论住在别的键下、顶不掉新的。比得比旧的整库导入只多不少（判在两种卡上的同一块会报错，旧导法同样报），双机驱动给第二台快照、判过的块不重判，走不到这一形。读码。
- 断了重跑：16 个杀的时刻、读到的块数 10 档；都补齐。只取了进程被杀，没取断电（tmpfs 上量的，同步写的语义没量）。
- 代价随库涨：N = 0 / 3000 / 30000 / 120000 四档，读次数逐字相同；挂钟不作数。300000 那一档建到 293000 块时机器满载、同步写排队，没建完、删了。
- 改坏照绿：试了 `mv`（打中）、`mi-pipe`、`mi-given`（打中，小）、`mp`（等价，读回相同）。verdict_store 内部那几处（判定向量批内查重那一支、换编号缓存）只读码：
  一次导入里对方库的编号与内容一一对应、缓存跨批不丢，批内查重那一支走不到，改坏它不改行为（推的）。

## 这条腿自己的限度

- 所有数都在冻结副本的拷贝上、`release`、`nice -n 19`、机器被别的会话压到负载 50–130 时量的；挂钟一律不作数，只取计数与读回。
- 流水线原型把被判用例文件整份拷进原型 crate 再接一段（`rerun.sh` 的 `copy_of`），用的是它的造流与环境守卫；造流本身错了，两臂一起错，这一层分不出。
- P4-3 在批长改成 256 KiB 的副本上量（`rb`）；冻结副本原样 64 MiB 那一档（`resume-frozen`）没跑完。块不跨批是读码推的，副本上的 18 批印证的是同一套代码在更多批边界上。
- `expected/` 里流水线那几份是在原型 crate 还没有 `src/lib.rs` 时跑的（加它只为了不让 checker 档 build.rs 每次判工作区成员的 `src/` 缺、整份重编）；
  加了之后 `mut-mv` 重跑了一次，前台时限里只跑到 h02 导入那一趟（`without_verifier=1`，与 expected 那一行相同），没跑完整段。
- P1、P3 checker 档那几行没跑；P1 的数是上一轮模型在旧底上的。
- 「没打中」都只抽了这一次样（照 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算`，要再抽一次才记没打中）。

## 没做什么

- 没跑任何 checker 档测试、门禁阶段、GPU；没跑 verdict_store 的库单测（checker 档）。没改冻结副本、没改仓里 crates/。
- 没把我提的两条改法实现进被判用例文件、没跑改过的用例。
- 没用读故障（`InjectedFault::ReadFails`）把 P3 的清零重造一遍。
- 整库导入的挂钟只量了 N = 0 一档。

## 要主 agent 跑的命令

无必须的。复跑与核对照文首「复跑」；`resume-frozen` 要在机器空闲时跑（120 块、约 260 MB 同步写）：
`OPUS_MEMORY_CAP=4G bash research/prompts/m3-batch60-61-code-r1-opus-model/rerun.sh <工作目录> resume-frozen`，看 `out/resume-frozen.txt` 每行的 `read_errors_after_kill=0 present_but_different=0` 与 `rerun_vs_reference=0`。

## 草稿与清理

草稿目录 `~/.local/share/singlefs/work/m3-batch60-61-code-r1-opus/`：交回前删了副本 `base/`、`mut/`、编译目录 `target/`（约 8.6 GB）、建的库 `stores/`、`stores-rb/` 与 tmpfs 上的
`/dev/shm/m3-batch60-61-code-r1-opus-stores-rb/`；留着 `out/`（各阶段原样输出，约 200 KB）与 `tmp/`（最后几次 cargo 日志）。入库的产物都在模型目录。

## 交回前的 sha256

```
16f764811eff59a0fc8a2e0484db78e06684481ed97f810292a89d39db9a1715  rerun.sh
9a4a3cd51ee1469d6c5967fafbc641822808acf9ee0272fd7c4622c8e62fe899  expected/base-batch.txt
1472cdc962e7a9afd9f8e005333c6a30f802b2c52241d077454358ba48561a63  expected/base-h1.txt
896558207e52fa89e1077eb032a0708750cf8203b2017945377af5bee9485bc4  expected/base-h2.txt
23e3c54fc9b326649809ef67c69928fb3e7b2b2698bb8792bccd45621c9dfc84  expected/base-h3.txt
008a9aacfad8344f7f01d6ec31a08816cdd8312a373d18b77810b69dda4c13bb  expected/base-h4.txt
9a4a3cd51ee1469d6c5967fafbc641822808acf9ee0272fd7c4622c8e62fe899  expected/mut-mi-given.txt
7ae70c20dd19d9cc96b30ccc44af1790e31d4902fb62f71bb18ead2c76577ca5  expected/mut-mi-pipe.txt
5317782ff2eafd9523827f1fe7e7914d32ddf9cc6e3095be25752df4885e3a10  expected/mut-mp.txt
ce3655b395702f6cfc470c07253a85eac5674699c702c83f7f8038a52e57dc68  expected/mut-mv.txt
4f0c64fffce573dab4a0a98a79374732e1cf7e00e1c68844308b0525848f45cd  expected/mut-row-2232.txt
74b0f9958f6665985e5c170566164375276e2ab49d0d3b0ed728043d91c9cd5b  expected/mut-row-396.txt
bd86ec6c875598e777aa0bb53de197ff3934bedf39c30532d465af2360da8ffb  expected/mut-row-657.txt
c3dcb5b2981f40128838aaf3c68b3b4a889c805393e8af68ea101919b85c742d  expected/mut-row-65.txt
f98acd3797c846c73b7989aa2524cf5dff540774d99ac413340af6cbc83758bf  expected/mut-tsv2755.txt
811b0fe333946d181316c732c5d5691c1dcbe8296c0be6a555e3c29594ca8a06  expected/mut-tsv3957.txt
66a36a4656a2d6296480b379adce1a83e02bab729eebb5c496c6982a48ab58a5  expected/mut-tsv3958.txt
fd100fe8e6409dcdd12f6995223f9089e856cb72aec74564a64dda16c3cb5662  expected/mut-tsv857.txt
48ce15d1c5dcc410217b845f6ee962d0af4f10276ad23dc33cbfaf223ef1f6f2  expected/p2-base.txt
36806691db2cbdef6ad140ac51f9f007fa1c231b1913a1c6064adc3809b225da  expected/p3-base.txt
```
