# m2-wave2-code-r1 云端攻方腿（Opus）报告：Z1 上界准入 / Z2 失败账 / Z3 聚簇段登记

立场：攻方，只攻 Z1、Z2、Z3（Z4 归本地攻方与正推腿，Z5、Z6 归正推腿，本报告不判）。时刻 2026-09-18 02:30–03:10 UTC。
行号全部以开工快照那一版为准（`/tmp/claude-1000/m2-wave2-code-r1/start-snapshot.sha256` 17 个文件，两份副本拷贝后 `sha256sum -c` 都是 17/17 OK）。
⚠️ 下面每个数都是**攻方副本上的数**，不进 kb；要引先在入库装置上重做（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」）。

## 零、复跑命令与文件指纹

两份副本（都是 `rsync -a --exclude target --exclude .git` 拷的整仓）：

- **未插桩副本**：`crates/` 一个字节都没改，只在 `crates/singlefs-harness/tests/` 加测试文件。Z1-a、Z1-d、Z2 的数都在这一份上量。
- **插桩副本**：`crates/singlefs-core/src/` 三个文件打了模型目录里的三份 diff——只读插桩（`opus_probe_reclaimed_count`、`opus_probe_exact_record_counts`、`OPUS_PROBE_LOG`：取号之前那一刻记下分配器的条数、已回收条数，并在分配器的**拷贝**上按同样的角色次序真分配一遍）外加一个改法开关 `OPUS_ADMISSION_MODE`（缺省 0 = 行为不变）。Z1-b、Z1-c、Z3 的数在这一份上量。

```
M=research/prompts/m2-wave2-code-r1-opus-model
# 未插桩副本 <P>
cp $M/opus_probe_pristine.rs $M/opus_probe_common_pristine.rs $M/opus_probe_z2.rs <P>/crates/singlefs-harness/tests/
cd <P> && nice -n 19 cargo test --release -p singlefs-harness --test opus_probe_pristine -- --nocapture --test-threads=1
cd <P> && nice -n 19 cargo test --release -p singlefs-harness --test opus_probe_z2 -- --nocapture --test-threads=1
# 插桩副本 <Q>
for f in allocator transaction mount; do patch <Q>/crates/singlefs-core/src/$f.rs $M/instr-$f.diff; done
cp $M/opus_probe_common.rs $M/opus_probe_z1.rs $M/opus_probe_z1_fixes.rs $M/opus_probe_z3.rs $M/opus_probe_overlap.rs <Q>/crates/singlefs-harness/tests/
cd <Q>
for n in 0 1 4 8; do OPUS_Z1A_PER_SESSION=$n nice -n 19 cargo test --release -p singlefs-harness --test opus_probe_z1 -- z1a --nocapture --test-threads=1; done
nice -n 19 cargo test --release -p singlefs-harness --test opus_probe_z1 -- z1b_calibration --nocapture --test-threads=1
OPUS_Z1B_MOUNTS=30 nice -n 19 cargo test --release -p singlefs-harness --test opus_probe_z1 -- z1b_sweep --nocapture --test-threads=1
OPUS_Z1B_INITIAL=0,10 OPUS_Z1B_EMPTY=0 OPUS_Z1B_PER_SESSION=30,40,48,60,100 OPUS_Z1B_MOUNTS=5 \
  nice -n 19 cargo test --release -p singlefs-harness --test opus_probe_z1 -- z1b_sweep --nocapture --test-threads=1
nice -n 19 cargo test --release -p singlefs-harness --test opus_probe_z1_fixes -- --nocapture --test-threads=1
nice -n 19 cargo test --release -p singlefs-harness --test opus_probe_overlap -- --nocapture --test-threads=1
nice -n 19 cargo test --release -p singlefs-harness --test opus_probe_z3 -- --nocapture --test-threads=1
```

三份 diff 在未插桩副本的三个文件上 `patch` 之后与插桩副本逐字节相同（`cmp` 三个 ok）。编译与跑动时 `ps` 看到别的会话在跑 `cargo test ... second_transaction_step_zero_layer0` 与 `cargo test --bin e142-first-txn-dry-run`，没有性能测量进程；我的全在副本自己的 `target/` 里，没等锁。

| 文件（`research/prompts/m2-wave2-code-r1-opus-model/`） | sha256 |
|---|---|
| `instr-allocator.diff` | `65f10672085b82ac3b39aa756fba305716f7f5fd977c5e196e3d40e9d2501731` |
| `instr-mount.diff` | `e0866f771f0b165dcdb8e5cb9c030b0c145763138c4fc6043e47b7baeee2bc28` |
| `instr-transaction.diff` | `b4a80789a5338f4ab8306f4897f03c491a8891c551c04ab971431ab66af7318e` |
| `opus_probe_common.rs` | `c5a9ed3a512e7d71a0934294b2746e659210b0161e7075f0504b3f633ca6b381` |
| `opus_probe_common_pristine.rs` | `24845633904e505c05d9276bc9d38d391007b6229c3321988895c01cd64f7f58` |
| `opus_probe_overlap.rs` | `f637f17a38682e8fba1adb5061d699024d524d61acc5ec70956405f939b75347` |
| `opus_probe_pristine.rs` | `cdebade096c95e725e23d5c2bd1aebf83b4ab0f0e8d0f30c6346485c5f7485e8` |
| `opus_probe_z1.rs` | `af7e577fe6ff94c51668b9d06c28ea28d610a1744821fcbbe76f85919eb16c0d` |
| `opus_probe_z1_fixes.rs` | `78d23b5c62b2276a830dc98c1b472200bb28942b0585b1933320fdf540dad1dd` |
| `opus_probe_z2.rs` | `04b8315b33449561e2ce4e0fb31bf07b3b04c6b9092cff902d2d1d46746501e1` |
| `opus_probe_z3.rs` | `64bf2745816d33cfea0d380893db7afdf72a404e1d930cb068382673f8bebc2f` |
| `pristine-run.log` | `61da886d11b0a5c306da6251314249569070965d777055dc62dcd2775789c1b6` |
| `z2.log` | `488800fd96bc55cad4f2edb054e880705a544195d4f3d9d0227e35561a0b31df` |
| `z2c.log` | `d5aec5600c7ed57f79eaf819f6bdf9395d3f01cfeca28ed525ed0a186c8d81f1` |
| `z1a-per0.log` | `a949bd42dd64c5ac9cc17208e68957d5ce46c2ab90f7466840310685daf560c7` |
| `z1a-per1.log` | `9dac86b9e675839aedadd143697905417ec3ca598435c2fa7709fd4b8e8d9917` |
| `z1a-per4.log` | `cb02fe09ef77a15d8b0f808d0f64effaf31436a8069cccc7b1eed8799abfda75` |
| `z1a-per8.log` | `5ffff2453c6f04ec67a8f447d8641b9094a0e17362841263a93209156a1da62b` |
| `z1b-sweep.log` | `8fd668966615bb7ee5f9c3decb2e616a6309f9ecefa84bad35a3185fae62659b` |
| `z1b-heavy.log` | `07e52a30681052ea5644c3707d33599b1a6060700ac94b5a9dbe4bd9ba7f48e1` |
| `z1-fixes.log` | `ef5ba044c5f5a2a7836592da6277071dbdcd50595c31265a45f7716f718774f2` |
| `z1-fixes-natural.log` | `5c4f9cb6e79ac8c7ccfcfb310074bd3a627d3f5206acf16581b8c71a57bc3e10` |
| `overlap.log` | `ad7bf085560a0e888a523218dc3dd3f80b8da1b22b4528daa9797fb84e1443ef` |
| `overlap-pinpoint.log` | `3b77a55955361f5dbb522f829c08e9fcf2530b2e9556f094d5eed6afcd0ae1c8` |
| `z3-final.log` | `361cfa5a59a58a3d900d4a8c75f022516c18b480925bd1ec08447ec26e19b741` |
| `z3-first-run.log`（`opus_probe_z3.rs` 第一版的输出，只用来引小盘那一行） | `20d33eb7745054f81cf5e7940c3aab1fe11dfb034d5cf9d03474f10c2e7f95ab` |
| `z1-fixes-30-ps0.log` | `15e105233ce25ba34429b87209955e4a13dda568576094ecd0af9079d425c16c` |
| `z1-fixes-30-ps1.log` | `ec3f8fa8cd57cb584263d3e23ea3335ed85501cfa62037230f8635504424dac1` |

几份日志与源文件的先后：`z1a-per0/1.log` 跑完之后 `opus_probe_z1.rs` 的 `run_history` 加过「取号之前就 panic」的分支（`z1a` 函数没动）；`overlap.log`、`z2.log`、`z1-fixes.log` 跑完之后同一个源文件里追加过 `overlap_pinpoint`、`z2c`、自然历史那几条测试，已有的测试函数没改。按上面的命令复跑出同样的行。

## 一、各格判定一览

| 格 | 判定 | 满足的是判据字面的哪一句 | 证据（量过 / 推的） |
|---|---|---|---|
| Z1-a 实例表只增不减，第 371 号实例的写行发布在取号之后 panic | **打中（漏判）** | Z1 触发列第二句「按上界放行而真跑起来撞断言」：准入放行（记录 280–590 条），取号写出，写行发布在 `bytes.rs:18` 切片越界 panic——是 panic，不是 `assert!` 宏 | 量过，未插桩副本：第 370 次挂载 panic，再试两次各烧一个号（371 → 372 → 373）；每次挂载之间覆盖写 0 / 1 / 4 / 8 次四种都在第 370 次挂载中 |
| Z1-b 按上界拒掉真发得起来的池（实现员那一例之外） | **打中** | Z1 触发列第一句「按上界被拒而真跑得完」 | 量过，插桩副本：实现员那两个池（796、788，用例文件里已写明是按上界拒）之外，另 3 个单实例的态，其中 2 个拒在**写行**那一次；另 2 条跨挂载的历史，最自然的一条「挂载 → 一路写到被拒 → 重开」在写行那次按 816 被拒，拷贝上真分配 808 / 808；吸收态 |
| Z1-c 抵扣可复用的已释放记录之后还拒不拒错 | **打中（换一种错）** | Z1 问列第二句 | 量过，插桩副本上实现的三个改法臂：全额抵扣不再假拒但漏（2 个态写行 panic、自然历史里会话中第 15 次覆盖写 panic）；按拷贝真分配只改取号之前那一处，2 个态 + 自然历史在取号之后被发布路径拒、烧号；两处都改则 5 个假拒态全挂上、真拒态照拒，但被放行的池 1–2 次挂载后撞 Z1-d |
| Z1-d 复用已回收的记录时跨度变了，旧记录留着，下一次挂载在重建分配器时 panic | **打中，但归属另议** | 字面上是 Z1 触发列第二句（放行而撞断言，`allocator.rs:375` 的 `assert!`），病根不在上界、在步 5 的复用（打中归错了判据） | 量过，未插桩副本：每次挂载之后覆盖写 2 次，第 8 次挂载后第 1 次覆盖写造出重叠记录，第 9、10 次挂载都 panic；扫描 288 段历史里 208 段以它结束 |
| Z1-e 记录条数上的漏判（上界放行、真实条数越过 812） | 没打中 | — | 推的（一个角色一个落点、一个落点每盘至多加一条）+ 量过（拷贝上真分配与真挂载逐次相等，2134 次挂载） |
| Z1-f 会话里同一个上界算式拒掉装得下的覆盖写 | 打中（问列，不在取号那一格） | Z1 问列「上界算术会不会拒掉真发得起来的」；不涉取号 | 量过：扫描里 13 次（798 + 16 = 814 被拒，拷贝上真分配 800） |
| Z2-a 按种类的合计与设备一层 | 没打中 | — | 量过：原样重试、失败后换一条路、连败两次加一次落盘前失败，三种走法合计都等于录制器 |
| Z2-b 失败账被下一次成功发布吞进去 | 没打中 | — | 推的（快照在每次发布自己的落盘开头取）+ 同上三组数 |
| Z2-c 最后一步（超级块槽）失败时根已 FUA，分配器照样退回，换一条路发布之后断电，池挂不上 | **打中问列的走法，不满足触发列** | 问列「失败之后不重试、直接换一条路」「失败发生在最后一步超级块槽写」；账相等，病在失败路径的分配器退回 | 量过，未插桩副本：checker 5 条红，重开报 `Recovery(UnitUnreadable { slot: 50257 })`；对照（原样重试）重开正常 |
| Z2-d 「份数就是失败过几次发布」与代码不符 | 打中（注释与代码），不在触发列 | — | 量过：失败 3 次、失败账 2 份（落盘之前就失败的那次不记） |
| Z2-e 调用方不取时账去哪了 | 已知一族（收口表第 20b 行 ⚠️），补两个细节 | 问列最后一句 | 量过：挂载在第二次暖机失败，设备一层 31 次 / 386 048 字节，`MountError` 里一份账都没有（成功的写行与第一次暖机的账也丢）；`raise_rollback_floor` 是第 20b 行没点名的另一处 |
| Z3-a 重开之后用户数据落进上一次挂载开过、仍装着环里的根引用的提交内生块的段 | **打中（可达）** | Z3 触发列第一句「用户数据落进装着提交内生块的段」（字面比本意宽，见第四节） | 量过，插桩副本（路径不经插桩）：88 种「每次挂载之后覆盖写几次」的模式里 18 种 40 次挂载内中 |
| Z3-b 登记表把整个单元区登记成聚簇段、用户数据无处可落 | 没打中（第一版几何下不可达） | — | 推的（记录 ≤ 812 ⇒ 每盘非空段 ≤ 406 < 2288）+ 量过（一次挂载里写到被拒，最多登记 7 段） |
| Z3-c 同一次挂载里回落两次、回落之后又开新段 | 没打中（回落本身不可达） | — | 推的，同上界 |

## 二、Z1 上界准入

### Z1-a（漏判）实例表只增不减：第 371 号实例的写行发布在取号之后 panic

**历史**（未插桩副本，`opus_probe_pristine.rs::pristine_instance_table_rows_overflow_after_acquisition`）：mkfs → 取号 1 → 暖机 → 第一个事务；之后只做可写挂载，一次接一次，挂载之间不写（插桩副本上另跑了挂载之间覆盖写 1 / 4 / 8 次，结果同形）。每一步许可它的那一句：

1. 每次可写挂载把上一版那张表原样接上这次要写的行，没有删行、没有第二片：`crates/singlefs-core/src/mount.rs:712` 原文 `    table_after.rows.extend_from_slice(rows_written);`。行数于是 = 新实例号 − 1（第 k 次挂载之后 rows = k，日志里 `rows_before` 与挂载次数逐次相差 1）。
2. 一片装 370 条（含链指针）：`crates/singlefs-format/src/lib.rs:101` 原文 `pub const INSTANCE_TABLE_PAGE_RECORDS: u64 = 370;`。`build_packed_unit` 没有容量判断，逐条 `writer.put(record)`（`crates/singlefs-core/src/unit.rs:111`），第 371 条写过单元末尾时在 `crates/singlefs-core/src/bytes.rs:18` 原文 `        self.bytes[self.cursor..self.cursor + slice.len()].copy_from_slice(slice);` 越界。
3. 取号之前的准入只算两棵记录树：`crates/singlefs-core/src/mount.rs:838` 原文 `/// 都要装得下，前面几次要新增的分配记录算进后面几次的基数；算不过就在任何写之前返回——取号写出去的实例代号一去不回，`（它上一行 837 写的是「分配记录树与记账树在每一次之后」），条数算式 `crates/singlefs-core/src/transaction.rs:1319` 原文 `        records_before_this_publish + rewritten.len() * allocator.devices.len();`。稳态下记录条数不再涨（每次都复用已回收的），准入恒放行。
4. 放行之后先取号再写行：`mount.rs:927` 原文 `    refuse_publishes_before_acquisition_that_do_not_pass_admission(`，`mount.rs:933` 原文 `    let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(`。

原样输出（`pristine-run.log`）：

```
mount=369 instance=370 rows=369 base=280 planned=2 records_after=280
thread 'pristine_instance_table_rows_overflow_after_acquisition' (3687742) panicked at crates/singlefs-core/src/bytes.rs:18:19:
mount=370 PANICKED: range end index 32784 out of range for slice of length 32768; base=280 (upper bound with 2 warm-ups: 306 <= 812)
superblock instances before: [(0, [370, 370]), (1, [370, 370])]; after: [(0, [370, 371]), (1, [370, 371])]
thread 'pristine_instance_table_rows_overflow_after_acquisition' (3687742) panicked at crates/singlefs-core/src/bytes.rs:18:19:
retry PANICKED again: range end index 32784 out of range for slice of length 32768; superblock instances now [(0, [371, 372]), (1, [371, 372])]
```

挂载之间写 0 / 1 / 4 / 8 次（插桩副本，`z1a-per{0,1,4,8}.log`）都在第 370 次挂载中，稳态记录条数分别 280 / 342 / 486 / 590，准入恒放行：

```
z1a-per0.log:mount=370 PANICKED instance_to_acquire=371 rows_before=369 rows_to_write>=1 base=280 planned=2 exact=[Some(280), Some(280), Some(280)] upper_refusal=None
z1a-per1.log:mount=370 PANICKED instance_to_acquire=371 rows_before=369 rows_to_write>=1 base=342 planned=1 exact=[Some(342), Some(342)] upper_refusal=None
z1a-per4.log:mount=370 PANICKED instance_to_acquire=371 rows_before=369 rows_to_write>=1 base=486 planned=1 exact=[Some(486), Some(486)] upper_refusal=None
z1a-per8.log:mount=370 PANICKED instance_to_acquire=371 rows_before=369 rows_to_write>=1 base=590 planned=2 exact=[Some(590), Some(590), Some(590)] upper_refusal=None
```

调用栈（`z1a-per1.log`，`RUST_BACKTRACE=1`）：`ByteWriter::put` ← `unit::build_packed_unit` ← `transaction::publish_admitted` ← `transaction::publish_version` ← `mount::publish_rows_on_file_version` ← `mount::establish_instance` ← `mount::mount_writable`。panic 在装单元、落盘之前，盘上只多了取号那两次超级块槽写；之后每试一次可写挂载就再烧一个号、再 panic 一次——正是收口表第 20a 行要挡的「每试一次可写挂载就多烧一个代号」，只是这回连错误都不交回。

**四句**：
1. 不分辨臂。上界、全额抵扣、拷贝上真分配三种记录条数算法在这一格上都放行（记录 280–590 条，离 812 很远），它们共用「取号之前只算分配记录树与记账树」这个前提。按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」第一种，另立一笔账先修，不拿它判 20a 那几条改法谁好。
2. 看得到。取号之前 `start.previous` 里已有这一版的实例表（`PreviousVersion::WithFile { table, .. }`），要写的行数由 `instance_to_acquire` 与所选根的实例定，两个都在取号之前算好。条款也这么写：`.claude/kb/decisions/28-挂载期承诺量.md:82`（已定项 3）原文整行：
   `**式子**：一次切换的最坏量 = 副本数 × 32768 × max(1, ⌈(rows0 + N_switch) / 每片行数⌉) 字节，其中副本数 = 2（实例表单元指针的两个位置条目，D22（单元原子性怎么合成） 已定项 7；两份落两块盘，所以按设备各算一份 32768 × 片数），每片行数 = ⌊(32768 − 135) / 行宽⌋ − 1（链指针记录恒为一片的最后一条，D18（块里携带什么信息） 已定项 11；行宽 88 ⇒ 369 行，2026-09-13 C304（实例表链指针记录装不下 83 宽的指针） 收口时从 64 改宽），rows0 = 挂载时读到的行数加写行那次要写的行数（新实例 − max(所选根的实例, 1)，挂载时就知道；每次可写挂载都写行）。预留 = (N_switch + 1) × 这个最坏量（N_switch = 3，D23（journal 的角色与格式） 已定项 14；多的一份给写行那次发布的元数据——写行那次是新实例的第一次发布，之前不推抬 F 的空发布，同一次发布里的用户数据重做照走准入、不够返回 ENOSPC；2026-09-14 用户定案，C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方）——N_switch 那几份是逐次累加的上界：连续切换的前提是上一次的根没发布、重建态从同一个根来，上一次的链页在重建态里不存在，实际不累加。`
   条款按多片算，实现只有一片、也不判片满。
3. 字面：Z1 触发列第二句「按上界放行而真跑起来撞断言」。放行是真的（日志 `upper_refusal=None`），撞的是切片越界 panic，不是 `assert!`；主 agent 若只认 `assert!`，这一格改记在 Z1 问列「按上界放行、真发起来却装不下」。
4. 改法：反向接受条款只写「改代码、补会红的用例与变异」，没给候选；收口表第 20a 行那一问的候选（把可复用的已释放记录抵扣掉）碰不到这一格（推的：抵扣只动记录条数）。碰得到的有两条，都是推的、没实现、被攻过零轮：① 取号之前加一项「这一版的行数 + 这次要写的行数 + 1（链指针）≤ 370」——只把 panic 换成取号之前拒绝，第 371 次挂载起池永远只能只读，是与 Z1-b 同一种吸收态；② 实现第二片（D28 已定项 3 与 D18（块里携带什么信息） 已定项 11 已按多片写），或实现删行（`.claude/kb/decisions/18-块里携带什么信息.md:882` 那一整段里的「回收」条件，要整轮清扫，这一版没有）。

**现查它是不是已知**：收口表第 28 行（`.claude/kb/milestone/02-second-txn.md:340`）列的容量是数据单元、分配记录树 812、记账树 477、树表 81，没有实例表；第 20a 行（同文件 348 行）写的是「映射与树表也没有条数上限」，而这一版映射树恒 6 条、树表恒 7 条，会长的是实例表。`crates/` 里 `grep -rn '删行\|下一片\|多片'` 只命中 `make_filesystem.rs:95` 一处注释（链指针记录的格式）。

### Z1-b 按上界拒掉真发得起来的池：实现员两个池之外的五处

「真发得起来」怎么量：插桩副本在取号之前那一刻，把分配器**拷贝一份**，按写行加暖机的角色次序真调 `allocate_user_data` / `allocate_commit_generated`，记每一次之后的条数（`opus_probe_exact_record_counts`，只读原分配器）。释放只改写记录、不动位图，不影响落点，所以拷贝上不做释放。**自检**：凡是真挂上的，拷贝上的条数与真挂载每次发布之后的条数逐次相等（探针里是 `assert_eq!`），`z1a` 四份各 369 次 + 扫描里 658 次，共 2134 次挂载全部相等。标定：实现员那个池（48 次覆盖写 + 1 次空发布）拷贝上是 798 → 798 → 800，与用例注释写的「写行只加 2 条、两次暖机加 0 与 2 条」相同。

扫描（`z1b-sweep.log`）：初始在实例 1 里覆盖写 {0, 20, 40, 44, 45, 46, 47, 48, 49} 次、空发布 {0, 1, 2, 3} 次，之后「挂载 → 会话里覆盖写 k 次」循环，k ∈ {0, 1, 2, 3, 5, 8, 12, 20}，最多 30 次挂载，共 288 段历史、714 次挂载判定。按上界被拒 56 次，其中拷贝上装得下的 40 次（5 个不同的态，每个态被 8 个 k 值各碰一次，因为都拒在第 1 次挂载、k 还没起作用）：

| 初始（覆盖写, 空发布） | 分配器 | 上界拒在 | 拷贝上真分配 | 与实现员那两个池的关系 |
|---|---|---|---|---|
| (48, 1) | 796 条，已回收 418，暖机 2 次 | 暖机第 1 次，814 | 798, 798, 800 | 实现员那一例 |
| (47, 2) | 788，418，2 次 | 暖机第 2 次，814 | 790, 790, 792 | 用例文件第二个池 |
| (47, 3) | 796，434，1 次 | 暖机第 1 次，814 | 798, 798 | 新 |
| (49, 0) | 804，418，2 次 | **写行**，814 | 806, 806, 808 | 新：`RowPublishAdmissionRefusedBeforeAcquisition` 假拒 |
| (48, 2) | 804，434，1 次 | **写行**，814 | 806, 806 | 新：同上 |

跨挂载、最像真用户的一条（`z1b-heavy.log`，另扫 k ∈ {30, 40, 48, 60, 100}、初始 {0, 10} 次覆盖写）：mkfs → 第一个文件 → 可写挂载（实例 2）→ 会话里一路覆盖写到发布路径拒（第 49 次，真拒：拷贝上 822）→ 重开：

```
h=(0,0,60) mount=1 in-session overwrite refused: before=806 upper=822 exact=[Some(822)] in_session_true_refusal
h=(0,0,60) mount=2 base=806 reclaimed=420 planned=1 exact=[Some(808), Some(808)] upper=Some((0, 816)) deduct=None exact_refusal=None path=Some((0, 816)) class=FALSE_REFUSAL PATH_REFUSES_AFTER_ACQ_IF_PREACQ_EXACT PATH_REFUSES_AFTER_ACQ_IF_PREACQ_DEDUCT status=refused_row
h=(10,0,40) mount=2 base=798 reclaimed=412 planned=1 exact=[Some(800), Some(800)] upper=Some((1, 816)) deduct=None exact_refusal=None path=None class=FALSE_REFUSAL status=refused_warm_up_1
```

k = 48 / 60 / 100 三个值都落到同一个态（806 条、写行 816 被拒）；初始 10 次、k = 40 / 48 / 60 / 100 四个值都落到 798 条、暖机第 1 次 816 被拒。这个池此后盘上一个字节都不变，每一次可写挂载都按同一个数被拒，只能只读——吸收态；而改法臂 3（下一小节）在同一段历史上挂上了（808, 808）。每一步的许可：会话里不回收（回收只在重建分配器与抬 F 时，`mount.rs:396` 那一处与 `raise_rollback_floor`），所以会话里写到上界就真满；重开时 `rebuilt_allocator` 回收释放代 ≤ 门槛的，于是写行与暖机大多改写已回收的记录（`allocator.rs:601` 原文 `            if self.reclaimed.remove(&key) {`）；而取号之前按 `transaction.rs:1297` 原文 `        records_before_this_publish += rewritten.len() * allocator.devices.len();` 一律按追加算。

**四句**：① 分辨臂：上界拒、拷贝上真分配放行，正是 20a 那一问的两边。② 看得到：取号之前内存里的分配器就是后面真发时用的那一个（取号不碰它），已回收集合、位图、开放段都在；拷贝上真分配与真跑逐次相等（2134 次）。③ 字面：Z1 触发列第一句「一段可达历史按上界被拒而真跑得完」；「真跑得完」是拷贝上量的条数 ≤ 812，并由改法臂 3 在同一个池上真挂上坐实（`z1-fixes.log`、`z1-fixes-natural.log`）。④ 改法：见 Z1-c。

### Z1-e 记录条数上没有漏判（没打中）

推的：`PoolAllocator::record`（`allocator.rs:598` 起）对每块盘要么改写一条已回收的（601 行那个分支）、要么追加一条（`allocator.rs:614` 原文 `            self.records.push(AllocationRecord {`），每块盘至多加一条；`publish_admitted` 每个重写的角色恰好取一个落点（`transaction.rs:1593` 原文 `        let placement = placement.ok_or(PublishError::NoSpaceFor { unit: *identity })?;`）。所以一次发布真实增加 ≤ 角色数 × 盘数，取号之前那一串的每一项 ≥ 发布路径那一遍（发布路径按真实的「这一次之前」加这一次的上界），也 ≥ 真实。量过：2134 次挂载里没有一次真实条数大于上界。**什么会推翻它**：某个角色一次取两个落点（分裂、多片），或 `record` 在一块盘上追加两条。

### Z1-f 会话里同一个上界算式拒掉装得下的覆盖写

扫描里 13 次：重开之后会话里第 n 次覆盖写，发布路径按 798 + 16 = 814 拒，拷贝上真分配 800（例：`h=(46,3,8) mount=1 in-session overwrite refused: before=798 upper=814 exact=[Some(800)] IN_SESSION_FALSE_REFUSAL`）。它不在取号那一格（取号早已做完），拒的是用户的一次写；算式是同一个 `admission_of_one_publish`（`transaction.rs:1319`）。只答 Z1 问列，列在这里给主 agent 判要不要一起改。

### Z1-c 抵扣可复用的已释放记录之后：三个改法臂在打中的格上

改法臂在插桩副本里用 `OPUS_ADMISSION_MODE` 切（diff 在 `instr-transaction.diff`）：**1** 全额抵扣——取号之前那一串与发布路径那一遍都按「每盘每个角色先用掉一条已回收的记录」扣（最宽的抵扣）；**2** 拷贝上真分配，只换取号之前那一串，发布路径照旧按上界；**3** 两处都换成拷贝上真分配。三个臂都**只在我的副本上量过、被攻过零轮**。表里每一格都是「量过」，原样输出在 `z1-fixes.log`、`z1-fixes-natural.log`、`z1-fixes-30-ps{0,1}.log`（后两份 sha256 `15e105233ce25ba34429b87209955e4a13dda568576094ecd0af9079d425c16c`、`ec3f8fa8cd57cb584263d3e23ea3335ed85501cfa62037230f8635504424dac1`，命令同 `z1_fix_arms_on_the_swept_cells`，加 `OPUS_FIX_MOUNTS=30 OPUS_FIX_PER_SESSION=0` 或 `1`）。

| 格（初始覆盖写, 空发布） | 0 原样 | 1 全额抵扣（两处） | 2 真分配（只取号之前） | 3 真分配（两处） |
|---|---|---|---|---|
| (48,1) 796 | 取号前拒（暖机 1，814） | 挂上 798/798/800 | 挂上 | 挂上 |
| (47,2) 788 | 取号前拒（暖机 2，814） | 挂上 | 挂上 | 挂上 |
| (47,3) 796 | 取号前拒（暖机 1，814） | 挂上 | 挂上 | 挂上 |
| (49,0) 804 | 取号前拒（写行，814） | 挂上 806/806/808 | **取号之后**被发布路径拒（814），超级块 1 → 2，烧号 | 挂上 |
| (48,2) 804 | 取号前拒（写行，814） | 挂上 | **取号之后**被拒，烧号 | 挂上 |
| (49,1) 812，真拒 | 取号前拒（822） | **取号之后 panic**：`unit.rs:160` 的 `assert!`「条目装不进一个节点」，烧号 | 取号前拒（真分配 814） | 取号前拒（814） |
| (48,3) 812，真拒 | 取号前拒（822） | **取号之后 panic**，烧号 | 取号前拒 | 取号前拒 |
| 自然历史：挂载 → 写到被拒 → 重开 | 取号前拒（写行，816） | 挂上 808/808；会话里第 15 次覆盖写 **panic**（`unit.rs:160`，写之前 808 条、已回收 180）；再挂载 panic（Z1-d） | **取号之后**被拒（816），烧号 | 挂上 808/808；写 14 次后正常拒（真分配 814）；第 3 次挂载 panic（Z1-d） |
| 被放行的 5 个态再挂 30 次（挂载之间写 0 或 1 次） | — | — | — | 10 段历史全部在第 2–3 次挂载撞 Z1-d 的 panic |

原样（`z1-fixes.log`）：

```
mode=1 h=(49,1,0) mount=1 PANICKED 条目装不进一个节点：写者要先按 index_node_entry_capacity 判、报错，不许走到这里 superblocks [(0, [1, 1]), (1, [1, 1])] -> [(0, [1, 2]), (1, [1, 2])] INSTANCE_BURNED
mode=2 h=(49,0,0) mount=1 refused Publish(AllocationRecordsExceedOneNode { records: 814, capacity: 812 }) superblocks [(0, [1, 1]), (1, [1, 1])] -> [(0, [1, 2]), (1, [1, 2])] INSTANCE_BURNED
mode=3 h=(49,0,0) mount=1 mounted instance=2 records_after_each=[806, 806, 808] superblocks [(0, [1, 1]), (1, [1, 1])] -> [(0, [2, 2]), (1, [2, 2])]
```

读法：
- **全额抵扣是漏的**：812 条、已回收 434 条的池，拷贝上写行仍然新增 2 条（814）。已回收的记录只在「被选中的那个槽正好有一条已回收记录」时才被改写（`allocator.rs:601`）；重开后开的是最低的**全空**段，段内新布局与旧布局对不齐（推的，没逐槽打印：旧的两槽单元第二个槽没有记录，落到那里的单槽单元就要新追加），「已回收多少条」不等于「会复用多少条」。
- **只改取号之前那一处，20a 要挡的烧号回来**：取号之前按真分配放行，发布路径仍按「这一次之前的真实条数 + 这一次的上界」判，804 + 10 = 814 在取号之后被拒。扫描里 16 次判定落在这一格（`PATH_REFUSES_AFTER_ACQ_IF_PREACQ_EXACT`）。
- **两处都换成真分配**：五个假拒态全放行、两个真拒态照拒在取号之前、自然历史挂上——在这几格上是唯一不假拒也不漏的臂；但它碰不到 Z1-a，而且被它放行的池马上撞 Z1-d（上界把这些近满的池挡在门外，恰好也把它们挡在 Z1-d 之前）。

**四句**（对「抵扣之后还拒不拒错」这一问）：① 分辨臂：三个臂在 (49,0)/(48,2)/(49,1)/(48,3)/自然历史上结局各不相同。② 看得到：已回收集合与位图都在内存（`reclaimed` 是私有字段，臂 1 要加访问器，副本里加了 `opus_probe_reclaimed_count`）。③ 字面：臂 1 的 panic 满足 Z1 触发列第二句（放行而撞断言，这回是真 `assert!`）；臂 2 的取号之后拒不是断言，是第 20a 行那一格本身。④ 改法：20a 的候选「抵扣」按「抵多少」分成上面三种，只有臂 3 在这些格上不中；它对 Z1-a、Z1-d 不起作用，交出去要写明。

### Z1-d 复用已回收的记录时跨度变了：旧记录留着，下一次可写挂载在重建分配器时 panic

**历史**（未插桩副本，`opus_probe_pristine.rs::pristine_reuse_with_a_different_span_leaves_an_overlapping_record_and_the_next_mount_panics`）：mkfs → 第一个文件 → 「可写挂载 → 覆盖写 2 次」× 7 → 第 8 次挂载 → 第 1 次覆盖写（txg 38）。每一步的许可：
1. 重开时回收：`mount.rs:396` 那一次 `reclaim_released_up_to(..., ReclaimedReuse::Immediately)`；旧段整段回收空之后，`lowest_empty_segment`（`allocator.rs:419`）重新开它。
2. 新布局的两槽容器（inode 树叶）落在 50320：50320 上有一条已回收的单槽记录，按 `allocator.rs:601` 那个分支改写，跨度改成 2（`allocator.rs:609` 原文 `                existing.span_slots = u16::try_from(placement.span).expect("跨度 2 字节");`）；50321 上那条旧的单槽已释放记录（释放代 7）没人动，仍在记录表里。两条记录罩住同一个槽 50321，随 txg 38 的分配记录树落盘。
3. 下一次可写挂载 `rebuilt_allocator` → `PoolAllocator::rebuild_from_records` 逐条 `mark_allocated`（`allocator.rs:544` 原文 `            device_map.mark_allocated(record.slot, span);`），第二条撞 `allocator.rs:375` 的 `assert!`，消息是 377 行原文 `            "跨度里有已分配的槽"`。

原样（`pristine-run.log`）：

```
mount=8 overwrite=1 txg=38 overlapping records on device 0: [(AllocationRecord { device: DeviceIdentity(0), slot: SlotNumber(50320), span_slots: 2, generation: CheckpointTxg(38), is_released: false }, AllocationRecord { device: DeviceIdentity(0), slot: SlotNumber(50321), span_slots: 1, generation: CheckpointTxg(7), is_released: true })]
thread 'pristine_reuse_with_a_different_span_leaves_an_overlapping_record_and_the_next_mount_panics' (3688017) panicked at crates/singlefs-core/src/allocator.rs:375:9:
mount=9 PANICKED: 跨度里有已分配的槽; superblock instances [(0, [9, 9]), (1, [9, 9])]
thread 'pristine_reuse_with_a_different_span_leaves_an_overlapping_record_and_the_next_mount_panics' (3688017) panicked at crates/singlefs-core/src/allocator.rs:375:9:
mount=10 PANICKED again: 跨度里有已分配的槽
```

panic 在取号之前（超级块 9 → 9 不变），池从此每次可写挂载都 panic。插桩副本上同形（`overlap.log`、`overlap-pinpoint.log`）：挂载之间覆盖写 k 次，k ∈ {2, 3, 5, 6, 10, 12, 16, 20} 依次在第 9、8、7、7、5、4、4、4 次挂载 panic，k ∈ {0, 1, 4, 8} 60 次挂载不中（这几个 k 下每段开的段布局每次相同）。Z1-b 的扫描里 288 段历史有 208 段以它结束（`grep -c 'BEFORE_ADMISSION status=panicked 跨度里有已分配的槽' z1b-sweep.log` = 208），133 段在第 2 次挂载；k = 0 也有 23 段（初始填得多的池）。

checker 在造出重叠的那一版上（`overlap-pinpoint.log`，每次发布前后各跑一遍 `check_pool_image`）：28 条里只有 I-3.1（已分配统计对得上）红，而它从第 6 次挂载起、重叠出现之前就已经红（「盘 0：记账的已分配 Some(3031040)，遍历全部有效根得到 2850816」，疑似收口表第 ② 行那一族「一次挂载转过一整圈根环时 checker 在合法状态上判 I-3.1 红」，没核）。重叠本身没有一条不变量专门抓：这一格对 checker 是盲的（推的：按 28 条的判定列表，没逐条读 walk.rs）。

**四句**：① 不分辨 Z1 的臂：三个准入臂都放行它，拷贝上真分配也走同一个 `record`，照样造出重叠——共用前提是「复用 = 改写那一条记录」。按「打中不分辨臂」另立一笔账先修；Z1-c 表的最后一行说明它会让臂 3 放行的池马上死掉。② 看得到：`record` 改写时手里有旧记录的跨度与这块盘上所有记录。③ 字面：Z1 触发列第二句「放行而真跑起来撞断言」（`assert!`，下一次挂载），但病根不在上界——**打中归错了判据**，归步 5 的复用（里程碑 02 步 5 现状「回收过的落点被再分配时那条记录改写」）；它也不是 Z3。④ 改法：跑前条款没给；推的方向有两条（改写时按新跨度把被罩住的已回收记录一并删掉或改写、旧跨度大于新跨度时把剩下那一槽的记录拆出来；或复用只挑跨度相同的已回收记录），都没实现、被攻过零轮。

**现查它是不是已知**：`m2-step45-code-r3-main-verification.md` 第四节 X1 碰过「同槽换跨度的复用」，那一格讲的是影子账的豁免与隔离按起点槽号做 key，判「今天不可达」；这里是复用本身留下重叠记录，可达、量过。kb 与收口表里 `grep` 「跨度里有已分配的槽」「重叠」无命中。

## 三、Z2 失败账

### Z2-a / Z2-b 合计对得上、没被吞（没打中）

未插桩副本，设备栈同 `second_transaction_supplement_one_write_accounting.rs`（录制器 → 按开关报错的一层 → 稀疏内存盘，报错那次写既不进录制流也不进按种类的账）。三种走法（`z2.log`）：

```
control: first try err=true failure entries=1 sum=WriteCallsAndBytes { write_calls: 41, written_bytes: 685056 } device=WriteCallsAndBytes { write_calls: 41, written_bytes: 685056 }
failure accounts: 2 entries, sum=WriteCallsAndBytes { write_calls: 28, written_bytes: 471552 }; device level in window=WriteCallsAndBytes { write_calls: 28, written_bytes: 471552 }; equal=true
sum(failure accounts)+success=WriteCallsAndBytes { write_calls: 42, written_bytes: 803328 }; device level=WriteCallsAndBytes { write_calls: 42, written_bytes: 803328 }
```

依次是：C 在最后一步失败后原样重试；C 在最后一步失败后改发空发布 E、E 也失败；同一个写入口连败两次（盘 1 第 3 次写、盘 0 第 9 次写）加一次落盘之前就失败（内容 40 000 字节）再成功。每一份失败账的快照都在那次发布自己的落盘开头取（`transaction.rs:2096` 原文 `    let writes_before_this_publish = pool.writes_by_structure_kind.clone();`，零单元发布在 541 行），失败在 `transaction.rs:2121` 记账，下一次成功发布重新取快照，所以吞不进去。**什么会推翻它**：录制器装在报错层里面（报错的写也进设备计数，真设备部分写入属于这一类）；或有写在两个快照之外发生（取号那两次超级块槽写本来就在任何一次发布之外，用例已单列）。

### Z2-c 最后一步失败时根已 FUA，分配器照样退回；换一条路发布、断电之后池挂不上

**历史**（`opus_probe_z2.rs::z2a_failure_after_the_root_is_durable_then_a_different_publish`）：第一个事务（txg 3）→ 覆盖写 B（txg 4）→ 覆盖写 C（txg 5，根落区域 2 = 盘 0）：盘 1 放行 9 次写（8 个单元 + 1 条记录），第 10 次——盘 1 的超级块槽——报错。此时 C 的单元、记录已落两盘，**C 的根已 FUA 落盘**，盘 0 的超级块槽也写了。
1. 许可：持久顺序先根后超级块槽（`transaction.rs:2111` 原文 `        writer.perform(CommitStep::WriteRootRecordForceUnitAccess {`，之后 2115 行 `RotateSuperblockSlots`）；任一步失败整次发布报错、分配器退回（`transaction.rs:1221` 原文 `/// 按 D16（发布语义） 已定项 7 的持久顺序落盘。准入之后任何一步失败，分配器退回到进来时的样子——这次发布没有成立，`，1258 行原文 `        *allocator = allocator_before_this_publish;`）。C 的失败账：20 次写，其中根槽 1 次、超级块槽 1 次、记录 2 次。
2. 调用方不重试 C，同一个写入口、同一个（已退回的）分配器，从 B 发一次空发布 E（txg 又是 5）。分配器退回到 C 之前，E 的四个固定点单元按同一个游标落到 C 刚用过的那几个槽上（推的，没打印 E 的落点；下面 checker 与重开都点名 C 的 extent 树根所在的 50257）。
3. E 在盘 0 的记录那一步失败（放行 4 次 = 四个单元），代替「E 的单元写完、E 的记录与根还没写时断电」。
4. 故障全清，可写挂载。

原样（`z2.log`）：

```
C failure account: total=WriteCallsAndBytes { write_calls: 20, written_bytes: 340480 } root_slot=WriteCallsAndBytes { write_calls: 1, written_bytes: 512 } superblock=WriteCallsAndBytes { write_calls: 1, written_bytes: 4096 } journal=WriteCallsAndBytes { write_calls: 2, written_bytes: 8192 }
checker on the image now: ["I-2.1: Violated(\"树 11（种类 1）的根 在盘 0 槽 50257 的那一份与位置条目里的校验和对不上\")", "I-3.1: Violated(\"盘 0：记账的已分配 Some(540672)，遍历全部有效根得到 475136\")", "I-4.8: Violated(\"最新根（txg 5）出发的遍历有单元对不上或读不出\")", "I-7.2: Violated(\"最新的根走不完：树 11（种类 1）的根 两份都读不到对得上的；树 12（种类 2）的根 两份都读不到对得上的；映射条目指的单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的\")", "I-7.4: Violated(\"最新根（txg 5）引用的单元已被复用或抹头（校验和对不上或头用不了）\")"]
remount: REFUSED Recovery(UnitUnreadable { slot: SlotNumber(50257) })
```

对照（同一个故障，C 失败后原样重试同内容）：`control remount: mounted, chosen root txg=5`。

**四句**：① 不分辨 Z2 的臂：账在两种走法下都相等，病在失败路径把「根已 FUA 的发布」当成没发生——所有账的做法共用这一个前提。② 看得到：落盘闭包知道自己停在哪一步，`WriteRootRecordForceUnitAccess` 返回成功之后再失败，这次发布在盘上已经成立。③ 字面：不满足 Z2 触发列（合计相等、没被吞）；它答的是 Z2 问列点名的两种走法，实质是正确性（checker 五条红、池挂不上）——**打中归错了判据**，归 `publish_version` 的失败路径（步 2 第二轮攻方腿那次改的「失败就整个分配器换回去」）。账这边的连带：C 那 20 次写记在失败账里，而断电重开之后如果没有 E，恢复选中的正是 C 的根（对照组就是这样挂上的）——失败账里装着一次事后成立的发布。④ 改法：跑前条款没给；推的方向：根 FUA 成功之后的失败不退回分配器（这次发布算成立，只是超级块槽没转完），或退回之后禁止复用这次发布占过的槽，直到下一个根落盘。没实现、被攻过零轮。

**故障模型**：一次瞬时写错（超级块槽）+ 一次断电（在下一次进程内发布的单元与根之间）。第一种是 20b 那条用例用的注入方式；第二种层 0 按崩溃点枚举。换一条路之后 E 成功写完根，这一格就关上（E 的根盖掉同一个根槽）。

### Z2-d 注释说「份数就是失败过几次发布」，代码只数落盘阶段的失败

`transaction.rs:231` 原文 `    /// 一次失败记一份，哪怕一个写都没发出去就失败（那一份是空的）——份数就是这个写入口上失败过几次发布。`。而 `count_failed_publish` 只在两个落盘闭包失败时调（`transaction.rs:560`、`2121`）；准入拒绝、`NoSpaceFor`、`ContentExceedsDataUnit`、释放判定路径的错都在落盘之前返回，不记。量过（`z2.log`）：

```
results: first=Some("BlockDevice(InputOutput(Custom { kind: O") second=Some("BlockDevice(InputOutput(Custom { kind: O") before_any_write=Some("ContentExceedsDataUnit { bytes: 40000, capacity: 32634 }")
failed publishes: 3; failure account entries: 2; per entry totals: [WriteCallsAndBytes { write_calls: 5, written_bytes: 131072 }, WriteCallsAndBytes { write_calls: 16, written_bytes: 327680 }]
```

合计不受影响（那次一个写都没有），不满足 Z2 触发列；是注释与代码不符。改注释或改代码由主 agent 定。

### Z2-e 调用方不取时账去哪了（已知一族，补两个细节）

`crates/` 里测试之外的七处 `PoolWriter::new`（`mount.rs:652` 抬 F、`mount.rs:919` 建实例、`scenario.rs:101`、`110`、`bin/first_transaction_on_device.rs:458`、`725`、`928`）没有一处调 `writes_of_failed_publishes`；全仓只有 `second_transaction_supplement_one_write_accounting.rs:559` 这一条用例调它。`PoolWriter` 没有 `Drop` 也没有 `#[must_use]` 提醒，写入口一丢账就没了。收口表第 20b 行（`02-second-txn.md:349`）已写 `establish_instance` 那一处；补两点：
1. 丢的不只是失败账：同一次挂载里**已经成功落盘**的写行与第一次暖机，它们的账在 `row_publish` 与 `warm_up_publishes` 两个局部变量里，`?` 返回时一起丢。量过（`z2c.log`，盘 1 第 15 次写报错 = 第二次暖机的第一个单元）：`z2c: mount error = Publish(BlockDevice(InputOutput(Custom { kind: Other, error: "注入的设备写错" }))); device level writes in the mount window = WriteCallsAndBytes { write_calls: 31, written_bytes: 386048 }`——取号 2 次 8 192 + 写行 15 次 213 504 + 第一次暖机 13 次 147 968 + 第二次暖机盘 0 一个单元 16 384，一份都拿不到。
2. `raise_rollback_floor`（`mount.rs:652` 新建写入口，676 行 `        )?;`）同形：第 n 次带新 F 的空发布失败，前 n − 1 次成功的账与失败账都随 `MountError` 丢。它今天是只供测试的强制入口，分量轻。

不满足 Z2 触发列字面（没有一段「合计与设备不等」的比较能做——调用方手里根本没有账），答的是问列最后一句「调用方不取时那份账去哪了」：随写入口析构，无处可取。

## 四、Z3 聚簇段登记

### Z3-a 重开之后用户数据落进上一次挂载开过、仍被环里的根引用的提交内生块所在的段（打中，可达）

**判「装着提交内生块」的口径**：同一个 64 槽段里，有槽此刻在位图里占着、最后写它的是提交内生角色，并且满足下面之一：现行版本的账里仍分配（live），或已释放而「释放代 − 1 ≥ 环里最旧可读根的 txg」（环里还有根引用它；这些历史里没有回退、F = 0、txg 连续）。只按位图占着算会把环里已没有根引用、只是会话里还没回收的槽也算进去，那一口径我先跑过（`z3a_user_data_lands_in_a_segment_opened_by_an_earlier_mount`，14 个 k 值里 5 个中），不拿它当证据。

扫的是「每次挂载之后覆盖写几次」的模式：单值 k = 1..40，以及 {0, 1, 4, 8} 与 {12, 16, 20, 24, 30, 36} 两两交替（两种先后），共 88 种，每种最多 40 次挂载。**18 种中**，34 种先撞了 Z1-d 的 panic（`z3-final.log` 末行 `patterns=88 strong_hits=18`）。两个例子（原样）：

```
STRONG_HIT pattern=[24] mount=2 write_in_session=9 txg=40 data_slot=50306 segment=50304 registered_in_this_mount=false this_mount_cluster_segments={SlotNumber(50240), SlotNumber(50560)} oldest_ring_txg=17 co_located=[(50304, InstanceTable, 4, "ring-referenced(release=30 oldest_ring=17)"), (50305, InstanceTable, 4, "ring-referenced(release=30 oldest_ring=17)")]
STRONG_HIT pattern=[24, 0] mount=3 write_in_session=11 txg=45 data_slot=50250 segment=50240 registered_in_this_mount=false this_mount_cluster_segments={SlotNumber(50560), SlotNumber(50624)} oldest_ring_txg=22 co_located=[(50240, InstanceTable, 30, "ring-referenced(release=32 oldest_ring=22)"), (50241, InstanceTable, 30, "ring-referenced(release=32 oldest_ring=22)"), (50242, AllocationTree, 30, "ring-referenced(release=31 oldest_ring=22)"), (50243, AccountingTree, 30, "ring-referenced(release=31 oldest_ring=22)"), (50244, MappingTree, 30, "ring-referenced(release=31 oldest_ring=22)"), (50245, TreeTable, 30, "ring-referenced(release=31 oldest_ring=22)"), (50246, AllocationTree, 31, "ring-referenced(release=32 oldest_ring=22)"), (50247, AccountingTree, 31, "ring-referenced(release=32 oldest_ring=22)"), (50248, MappingTree, 31, "ring-referenced(release=32 oldest_ring=22)"), (50249, TreeTable, 31, "ring-referenced(release=32 oldest_ring=22)")]
```

第二条读法：第 1 次挂载之后写 24 次；第 2 次挂载不写，它的写行（txg 30）与暖机（txg 31）从最低的全空段 50240 开段，装了实例表与八个固定点节点；第 3 次挂载写行（txg 32）把它们换下；这次会话里第 11 次覆盖写（txg 45）的数据单元落在 50250，与那十个槽同一段。环里最旧的根 txg 22，txg 30、31 两条根还在环里、还引用着它们（回退候选）。这一刻本次挂载登记的是 {50560, 50624}，50240 不在里面。

每一步的许可：登记表只住内存，`allocator.rs:495` 原文 `    /// 这次挂载开过的聚簇段的起点，开过就一直算数（只在内存，重开后按 D3（空间分配） 已定项 10 ①「挂载后新开一段」重来）。`；重建从 `Self::new` 起（`allocator.rs:536` 原文 `        let mut allocator = Self::new(devices);`，`new` 里 519 行 `            cluster_segments: BTreeSet::new(),`）；用户数据只排除登记表里的段（`allocator.rs:403` 原文 `            let inside_cluster_segment = cluster_segments.iter().any(|segment| {`）；会话里 50176 那一段的偶数槽对被数据单元占满之后，最低的空偶数槽对就在上一次挂载的段里。

压着的条款：`.claude/kb/decisions/03-空间分配.md:255`（已定项 8 第 2 条）原文整行：
`2. **聚簇段只给提交内生块**，D3（空间分配） 已定项 5「用户数据块的落点不受此约束」原样成立、不推广；提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`——C146（无空段时的回落政策全仓无定义） 第 ② 条（回落落到哪）由此有了政策，第 ① 条（整理的下一个目的地落回自己正在清空的段）不归这一项，归 D26（后台整理与放置回收） 已定项 1。`
收口表第 20c 行（`02-second-txn.md:350`）自己写了「重开之后那一格待定」「一个聚簇段什么时候不再算聚簇段全仓没有条款」——这一格今天**可达**，上面是具体历史。后果（推的）：那一段在数据单元活着的时候永远不是全空段，`lowest_empty_segment` 不再开它。

**四句**：① 分辨臂：「登记表只住内存」与「挂载时从候选根重建登记」或「登记表落盘」在这段历史上答案不同。② 看得到，但要走树：分配记录不带角色（`AllocationRecord` 只有设备、槽号、跨度、代、已释放位），挂载时要知道哪些槽是提交内生块，得从候选根的树表与映射认出元数据单元（影子账已经在逐条候选根读分配记录树，推的：可以顺带做）。③ 字面：Z3 触发列第一句。注意触发列比本意宽：字节表的第一个事务就把数据单元放在 50180，与 mkfs 的实例表（50176–50177，按落点政策是提交内生块）同一段——按字面第一个事务就「中」。我这一格只算「上一次挂载当聚簇段开过、仍装着环里的根引用的提交内生块」的段。④ 改法：跑前条款没给候选；20c 那一问（聚簇段什么时候不再算）没定，这一格由它定。

### Z3-b / Z3-c 登记表撑满单元区、回落两次（没打中：第一版几何下不可达）

推的：journal 环默认 768 MiB 且要 ≤ 设备容量 / 4（`make_filesystem.rs:136` 原文 `    if ring_bytes > smallest / 4 {`，`crates/singlefs-format/src/lib.rs:162` 原文 `pub const JOURNAL_RING_DEFAULT_BYTES: u64 = 768 * 1024 * 1024;`）⇒ 每块盘 ≥ 3 GiB ⇒ 单元区 ≥ 146 432 槽 = 2288 段；分配记录树一个节点 812 条 ⇒ 每块盘有记录的槽 ≤ 406 个（两槽单元的两个槽在同一段）⇒ 非空段 ≤ 406 ⇒ `lowest_empty_segment` 永远答得出 ⇒ 回落走不到 ⇒ 登记表只随「开全空段」长。量过（`z3-final.log` 的 `z3c`）：一次挂载里一路覆盖写到被准入拒（48 次），登记 7 段；初始 30 / 45 次覆盖写的池重开后最多 3 段。我试过把设备缩小到单元区 128 槽，mkfs 按上面那条拒（模型目录 `z3-first-run.log` 原样：`mkfs: JournalRingTooLargeForDevice { ring_bytes: 805306368, device_bytes: 824180736 }`；那是 `opus_probe_z3.rs` 的第一版，那条测试之后从源文件里删掉了，其余尺寸没跑到，都小于 3 GiB、同一条判断），这一版造不出小盘。**什么会推翻它**：非默认的环长（环是 mkfs 参数）配小盘；或回退之后影子账隔离的槽把段挡成「非全空」（推的上界没算隔离，回退没扫）。

## 五、每个打中的推翻条件

| 格 | 什么现象会推翻它 |
|---|---|
| Z1-a | 入库装置上同一段历史（mkfs → 第一个文件 → 可写挂载 370 次）第 370 次挂载不 panic；或实例表另有删行 / 第二片的路径我没看到 |
| Z1-b | 入库装置上把取号之前的准入换成「拷贝上真分配」之后，(49,0) 与自然历史那一格的写行发布真实条数 > 812 |
| Z1-c | 全额抵扣臂在 (49,1)/(48,3) 上真实写行条数 ≤ 812（即已回收的记录确实都被复用）；或只改取号之前时发布路径不再按上界判 |
| Z1-d | 入库装置上「挂载 → 覆盖写 2 次」× 8 之后第 9 次挂载不 panic；或 `rebuild_from_records` 之前另有一步合并重叠记录 |
| Z2-c | C 在超级块槽失败之后，入库装置上 E 的落点与 C 的不重叠（分配器没退回），或恢复在最新根走不完时退到次新根 |
| Z3-a | 入库装置上 [24, 0] 模式第 3 次挂载第 11 次覆盖写的数据单元不在 50240 那一段，或那一段里 txg 30、31 写的单元已不被环里的根引用 |

## 六、没打中的形状（试过什么、取样多大）

- **Z1 记录条数漏判**：一个角色一个落点、一个落点每盘至多加一条（推的）；2134 次真挂载与拷贝上真分配逐次相等（量过）。取样：初始覆盖写 0–49 次 × 空发布 0–3 次 × 会话里覆盖写 0–100 次，挂载至多 30 次。
- **Z1 其余固定容量**：映射树恒 6 条、树表恒 7 条、记账行只随盘数变（第 28 行已记 80 块盘）；只有实例表随挂载次数长（Z1-a）。journal 记录的点名项 ≤ 9 个角色，没去算容量（推的：远小于一条记录）。
- **Z2 合计与设备不等 / 被吞**：三种走法 + 挂载里失败，四组数（量过）。没试：录制器装在报错层外面、真设备的部分写入（这两种是口径问题，不是代码的走法）。
- **Z3 撑满登记表 / 回落两次**：第一版几何下不可达（推的上界 + 量过每次挂载至多 7 段）。没试：非默认环长配小盘、回退之后的隔离。
- **按用户动作放开扫**（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」2026-09-16 那条）：Z1-a 放开了会话里的写次数（0/1/4/8）；Z1-b 的五个假拒态都拒在第 1 次挂载，与之后写几次无关，自然历史那条放开了 5 个写次数；Z1-d 放开了 12 个写次数与 88 种交替模式；Z3-a 放开了 88 种模式。**没有一格是按「某一步固定写几次」才中或才不中**——唯一的例外是 Z1-d 在 k ∈ {0, 1, 4, 8} 的固定模式下 60 次挂载不中，那是「每段布局每次相同」，一换写次数就中。

## 七、这条腿自己的限度

- 「真发得起来」是我写的拷贝上真分配（插桩副本里的函数），靠 2134 次逐次相等自检坐实；它与 `publish_admitted` 的差别只在不做释放（推的：释放不动位图，不影响落点）。
- Z1-c 三个改法臂是我在副本里实现的（`OPUS_ADMISSION_MODE`），**被攻过零轮**；臂 3 在副本里每次发布多一次分配器拷贝，代价没量。
- Z3-a 的「环里还有根引用」按「释放代 − 1 ≥ 环里最旧根的 txg」判，只在没有回退、txg 连续的历史上成立；没有逐根走树核对。
- Z2-c 的「断电」是用 E 的记录那一步写错代替的，不是层 0 枚举出来的崩溃态。
- Z1-d 的 checker 盲区是按 28 条判定列表推的，没逐条读 `walk.rs`。
- 只走了 `mount_writable`；`mount_rollback`（共用 `establish_instance`）、抬 F、盘不等大都没扫。
- 所有数都在两份副本上量，不在入库装置上；副本里的数不进 kb。

## 八、没做什么

- 没碰 Z4、Z5、Z6；没读禁读清单里的文件。
- 没改仓里任何代码、kb、用例；写的只有本报告与 `research/prompts/m2-wave2-code-r1-opus-model/`（草稿在 `/tmp/claude-1000/m2-wave2-code-r1-opus/`：两份副本、`build1.log`、`z1b-sweep` 第一版被 Z1-d 打断的日志等，都没入库，要的话主 agent 从那里拷）。
- 没跑门禁、没跑 `crates/mutations.tsv`、没在入库装置上重做任何一个数。
- 没为 Z1-a、Z1-d、Z2-c、Z3-a 写会红的用例与变异行（反向接受条款里那一步归实现员）；探针文件可以当那几条用例的起点。
- Z1-a、Z1-d、Z2-c 三处的改法只写了方向（推的），没实现。
