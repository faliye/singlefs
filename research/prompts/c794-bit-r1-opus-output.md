# C794 盘上隔离位：没定的六处条款，第一轮攻方腿（Opus）报告（2026-10-15）

## 复跑

整条经内存包装、在任意工作目录起（`--work` 给一个新目录；它拷冻结副本、打补丁、编、跑，最后逐份比 `out/*.txt`，末行 `name=rerun_done compared=31 differing=0` 才算复出来）：

```
bash scripts/hosts/run-with-memory-cap.sh 24G bash research/prompts/c794-bit-r1-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo --work <新的草稿目录>
```

线程数取 `RERUN_THREADS`（不设是 31）。编译含 librocksdb-sys，冷编约 3 分钟；跑完全部约 5 分钟（本机这一轮量的：探针每个镜像场景几秒，harness 用例每条 1–30 秒）。
模板自带的 `--selftest` 拿假仓跑 `rerun_steps`，换成这一腿的步骤之后它要真仓真编，自证那一格不再成立（没改模板别处）。

模型目录 `research/prompts/c794-bit-r1-opus-model/` 的文件与 sha256（`out/` 下 31 份输出与一份 `.note`逐份的 sha256 在草稿里列过，这里只列三份源文件）：

| 文件 | sha256 |
|---|---|
| `c794_attack_probe.rs`（原型，也在补丁里作新文件） | 9147f1f0375e060f200a2e4dae2bd41c829af23a5fd877b051d9dbafbbb067eb |
| `patches/attack-switches.diff`（打在冻结副本上：读法开关、原型、取样计数） | 216fb9c203295751c052c2504f8bcde5bd6ef718eb6bb1de8b79caae81cebd67 |
| `tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷，只换 `rerun_steps`） | 9f9ae9a1bcd5c7b18a0a0f0aa40ba013b5ac6391d215c5b6526dff8113fc5349 |

补丁里的读法开关（全是环境变量，不设就是冻结副本的行为）：

| 开关 | 改哪 | 读法 |
|---|---|---|
| `C794_EXEMPT_READING=per_unit` | `walk.rs` 的 `quarantined_slots_exempted_per_device` | Q5 对面：同一起点槽、同跨度、未释放无引用的几条记录里有一条带位就整组豁免 |
| `C794_Q3=newest` / `every` | `walk.rs`，豁免算完之后 | Q3 对面：隔离位置在最新根引用（判 I-3.11）/ 任一走过的版本引用（判 I-3.1）的记录上判违例，详情以 `C794-Q3` 开头 |
| `C794_ROLLBACK_KEEPS_BIT=1` | `mount.rs` 回退清位那个循环 | Q2 对面：回退不清位 |
| `C794_ISOLATED_BY_BIT=1` | `mount.rs` 的 `resurrection_against_the_current_account` | Q4 对面：每块盘上记录都带位的才认成隔离单元 |
| `C794_WRITE_BOTH_BITS=1` | `allocator.rs` 的 `to_bytes` | 造写者缺陷：已释放 + 隔离位照写两位 |
| `C794_MUTANT_NEW_RECORD_BIT=1` | `allocator.rs` 新建记录那一处 | Q8 变异：新分配的记录带隔离位 |
| （常开）`C794_QUARANTINE_EVENT` 一行 | `transaction.rs` 发布释放之前 | Q8 取样计数：这次发布隔离了几份 |
| `c794_set_quarantine_flag` | `allocator.rs` 新方法 | 造写者缺陷：直接改一块盘上那条记录的隔离位 |

盘全是内存稀疏盘（`SparseBlockDevice`），流都在原型里自己造；原型是 `singlefs-checker-tier` 的 `src/bin/` 二进制（崩溃枚举入口只认那几个目录下的设点文件），`cargo build` 编、直接执行，不是测试目标。

## 各格判定一览

| 格 | 实现员先做的读法 | 对面读法 | 量过 / 推的 |
|---|---|---|---|
| Q1 | **打中**（不一致，不是漏判）：写者缺陷写出的「已释放 + 隔离位」记录，池级 checker 全绿、I-1.1 判 Holds，冷启动走读拒成 I-1.1；I-1.1 的条文里没有这一组合 | 没打中（checker 也判红：写者经 `to_bytes` 写不出这一组合，崩溃状态里也造不出） | 量过（镜像）；对面读法没误判是推的 |
| Q2 | 没打中（清位）：清位落在回退那次发布里，在两道逐盘验之后 | **打中**（不清）：回退重新引用隔离单元、之后写者漏释放它，镜像与合法隔离逐字段同形，checker 全绿；清位读法下 I-3.11 红 | 量过（镜像）；回退那次发布的崩溃状态没跑（见「崩溃状态」） |
| Q3 | 没打中（不判） | 按「任一走过的版本引用」判：**打中**，合法的隔离之后 A 还在候选集里就判 I-3.1 红（误判合法状态）；按「最新根引用」判：没打中，合法镜像不红，能抓「回退不清位」 | 量过（镜像）；崩溃状态没跑 |
| Q4 | 没打中（按推断认隔离单元） | **打中**（看位）：不带位的漏释放单元一份坏了，回退不验就做成，新根引用一份对不上的单元，回退之后 checker 判 I-2.1、I-4.8、I-7.4 红；推断读法在任何写之前拒 | 量过（镜像）；只在写者缺陷之后够得着 |
| Q5 | 没打中（按记录） | **打中**（按单元）：写者只在一块盘上置位（缺陷），按单元豁免全绿，按记录判 I-3.11 红；写者两盘都置位时两读法同判 | 量过（镜像）；「只置了一块盘就崩」的崩溃状态没跑（16777228 个，越过约 10⁶） |
| Q8 | 没打中（不观测） | 不观测时有一个量到的盲点：变异「新分配的记录带隔离位」在两档随机历史里全绿（快档池级 checker 跑了 0 次）；随机历史里隔离一次都没发生，「回退不清位」那一类观测了也抓不到 | 量过（变异与取样计数）；加观测之后能抓到是推的，没实现 |

## Q1 已释放位与隔离位同时置着

**用例**（`c794_attack_probe q1`，输出 `out/q1.txt`）：新池新建文件 A（txg 3）；覆盖写 B 照常释放 A 的数据单元（两份都读得出）；两块盘上那条已释放记录的隔离位置上（`c794_set_quarantine_flag`），开 `C794_WRITE_BOTH_BITS=1` 让编码器照写两位，覆盖写 C 把它写进盘上的账；对 C 之后的镜像跑池级 checker 与冷启动走读。原样输出：

```
Q1 B 之后 A 数据槽 50180: 盘0(已释放=true,隔离位=false,代=4) 盘1(已释放=true,隔离位=false,代=4)
Q1 C 之后 A 数据槽 50180: 盘0(已释放=true,隔离位=true,代=4) 盘1(已释放=true,隔离位=true,代=4)
Q1 checker: violated=[] I-1.1=Holds I-3.1=Holds I-3.11=Holds
Q1 cold=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(5))), failure: InvariantViolated { invariant: "I-1.1", detail: "分配记录的已释放位与隔离位同时置着：条款没定这一组合，第一版不认" } }
```

每一步许可它的那一句：
- 写者正常路径写不出这一组合：「!(self.is_released && self.is_quarantined),」（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/allocator.rs:63`），所以要用开关绕开断言造写者缺陷；崩溃状态里也造不出（分配记录叶是整单元校验的 COW 单元，撕一半校验和不过，推的，没跑崩溃枚举）。
- core 读者拒：「if record.is_released && record.is_quarantined {」（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/allocation_record_tree.rs:747`）；GPU 恢复内核同一处：「if (((span_field >> ALLOCATION_RECORD_RELEASED_BIT) & (span_field >> ALLOCATION_RECORD_QUARANTINED_BIT) & 1u) == 1u) {」（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2335`）——GPU 那一处只读了代码，没起卡。
- 池级 checker 两位都解出来（「is_quarantined: span_field & ALLOCATION_RECORD_QUARANTINED_FLAG != 0,」，`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-checker/src/walk.rs:4505`），豁免那一判先跳过已释放的（「if record.is_released」，`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-checker/src/walk.rs:7040`），别处不看这一组合：结果 I-1.1 判 Holds。
- I-1.1 条文管分配记录的只有起点与末槽：「分配记录叶里每条记录的起点与末槽都落在叶罩的那一段里」（`crates/singlefs-checker/invariants/01-块自描述.md:6`），没有标志位组合这一样。

**四句**：
1. 分不分辨臂：分。实现员的读法下同一张镜像 checker 说 I-1.1 成立、core 说 I-1.1 违例；对面读法（checker 也把这一组合判成 I-1.1 违例）下两边一致。
2. 被判的系统当时看不看得到：看得到，checker 在 `walk.rs:4505` 已经把隔离位解出来了，判别它只差一个条件。
3. 满足判据字面的哪一句：打中的是正文 Q1 格「三处读者对同一组合的判法一致吗」——不一致；不是「漏判真泄漏」也不是「误判合法状态」。按 I-1.1 条文字面，checker 判 Holds 没判错；是 core 把拒因记在 I-1.1 名下、而 I-1.1 条文里没有这一样（`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 四形里的「打中归错了判据」那一形，落在 core 的拒因上）。
4. 改法在打中的格上还中不中：甲「checker 也判 I-1.1 红，I-1.1 条文补一句标志位组合」——这一格不再不一致；乙「core 不拒、当已释放读」——三处一致，但盘上多一种没人写的合法编码，写者缺陷（把隔离位留在已释放记录上）从此谁都不报。两样都只在我的模型上推过，被攻过零轮。

**射程与失败处置**：正文问「合不合 D22 的失败处置」，D22 那份文件里 `grep -cF '失败处置'` 命中 0 次；这一类「盘上出现写者不写的东西就在任何写之前停」的条款在 D23：「恢复遇到它就是盘坏了，在任何写之前停下」（`.claude/kb/decisions/23-journal的角色与格式.md:406`）。core 的拒与这一句同形（量到：冷启动走读停在 I-1.1、没有走到读文件）；可写挂载那一条路没在原型里跑。

## Q2 回退重新引用隔离单元时清不清位

**用例**（`c794_attack_probe q2-image`，清位读法输出 `out/q2-clear.txt`，不清读法开 `C794_ROLLBACK_KEEPS_BIT=1`、输出 `out/q2-keep.txt`）：A（txg 3）；B 覆盖写时 A 的数据单元两块盘那一份的读各先报错两次（瞬时隔离，盘上字节不动）；C；挂着时回退到 A（txg 6，重新引用那个隔离单元）；D 覆盖写换下它——这里注入写者缺陷「D 漏了这一次释放」：记录停在 D 之前的样子（仍分配、分配代 3、隔离位照 D 之前）；E 把账写盘。原样输出：

```
Q2 keeps_bit=false 回退之后 槽 50180: 盘0(已释放=false,隔离位=false,代=3) 盘1(已释放=false,隔离位=false,代=3)
Q2 回退之后 checker: violated=[] I-1.1=Holds I-3.1=Holds I-3.11=Holds
Q2 D 照常释放之后 槽 50180: 盘0(已释放=true,隔离位=false,代=7) 盘1(已释放=true,隔离位=false,代=7)
Q2 漏释放之后 槽 50180: 盘0(已释放=false,隔离位=false,代=3) 盘1(已释放=false,隔离位=false,代=3)
Q2 漏释放之后 checker: violated=["I-3.11"] I-1.1=Holds I-3.1=Holds I-3.11=违例[盘 0：记账的已分配 Some(1376256) 减 defer 待释放 Some(1032192)，不等于从最新根（txg 8）走读到的 311296（其中隔离豁免 0）]
```

```
Q2 keeps_bit=true 回退之后 槽 50180: 盘0(已释放=false,隔离位=true,代=3) 盘1(已释放=false,隔离位=true,代=3)
Q2 回退之后 checker: violated=[] I-1.1=Holds I-3.1=Holds I-3.11=Holds
Q2 D 照常释放之后 槽 50180: 盘0(已释放=true,隔离位=false,代=7) 盘1(已释放=true,隔离位=false,代=7)
Q2 漏释放之后 槽 50180: 盘0(已释放=false,隔离位=true,代=3) 盘1(已释放=false,隔离位=true,代=3)
Q2 漏释放之后 checker: violated=[] I-1.1=Holds I-3.1=Holds I-3.11=Holds
```

每一步许可它的那一句：
- B 两份都留在已分配、两盘都置位：「任一份核出对不上，这个单元在每块盘上的分配记录都留在「已分配」」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:113`），写者那一处「record.is_quarantined = true;」（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/allocator.rs:1895`）。
- 回退到 A 合法、隔离单元要逐盘验：「R_old 引用、cur 的账里仍分配而同分配代的单元里被隔离的那些」（`.claude/kb/decisions/23-journal的角色与格式.md:388`）；两份都完好，验过。
- 清位那一写：「allocator.clear_quarantine_of_the_records_of(Placement {」（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/mount.rs:8302`），排在逐盘验「every_copy_of_the_isolated_units_the_target_references_reads_back(」（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/mount.rs:8225`）与「let allocator_before_the_rollback = allocator.clone();」（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/mount.rs:8291`）之后，只改内存里的账，随回退那次发布的分配记录树落盘——在「任何写之前拒」那几判之后（量的是代码次序，不是崩溃状态）。

**四句**：
1. 分不分辨臂：分。同一段历史、同一个注入，清位读法 I-3.11 红，不清读法一条都不红。
2. 被判的系统当时看不看得到：不清读法下看不到——漏释放之后那两条记录是「仍分配、无根引用、带位、分配代 3」，与合法的瞬时隔离（`checker_quarantine_flag_exempts_only_flagged_unreferenced_records.rs` 第一条用例那张镜像）逐字段相同，checker 没有东西可分；这是读法本身的缺口，改判据补不上（「判别子观测不到」那一形，落在不清读法上）。
3. 满足判据字面的哪一句：正文「五、跑前条款」第二行「一格的某个读法被造出判错的段 ⇒ 那个读法出局」，判错的形是正文「四、分工」攻方那一行的「漏判真泄漏」。前提是一次写者缺陷（漏释放），不是写者正常路径。
4. 改法在打中的格上还中不中：清位读法（实现员的）在这一格上不中；另外量到入库用例已经钉住清位：开 `C794_ROLLBACK_KEEPS_BIT=1` 时 `rolling_back_to_the_version_referencing_an_isolated_unit_whose_failing_copy_was_written_back_intact_goes_through` 红（`out/targeted-C794_ROLLBACK_KEEPS_BIT-rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.txt`），其余两份入库用例不红。

清位读法试过的反例形：清位之后回退那一版的 checker 全绿（上面输出第二行）；清位之后照常覆盖写释放，记录改成已释放、位清零（第三行），一条不红。崩在回退那次发布中间的状态没跑，见「崩溃状态」一节；条款对这一格的说法是「崩在回退那次发布的任何一点，恢复落到的与任何一次普通发布没成时相同」（`.claude/kb/decisions/23-journal的角色与格式.md:390`）。

## Q3 隔离位置在有根引用的记录上判不判违例

对面读法有两种取「有根引用」的口径，补丁里各一个开关。用例与 Q5 同一段历史（`c794_attack_probe q5-image`）：下面只看「对照（两盘都置位）」那两行——那是写者正常写出的镜像：A；B 瞬时隔离 A 的数据单元、两盘都置位；C。A 的根（txg 3）还在候选集里（没抬 F），它引用那个单元。

`C794_Q3=every`（`out/q3-every.txt`）：

```
Q5 读法=None 槽 50180: 盘0(已释放=false,隔离位=false,代=3) 盘1(已释放=false,隔离位=true,代=3)
Q5 checker: violated=["I-3.1", "I-3.11"] I-1.1=Holds I-3.1=违例[C794-Q3-every 隔离位置在走过的版本引用的记录上：[(1, 50180)]] I-3.11=违例[盘 0：记账的已分配 Some(737280) 减 defer 待释放 Some(442368)，不等于从最新根（txg 5）走读到的 262144（其中隔离豁免 0）]
Q5 对照（两盘都置位） 槽 50180: 盘0(已释放=false,隔离位=true,代=3) 盘1(已释放=false,隔离位=true,代=3)
Q5 对照 checker: violated=["I-3.1"] I-1.1=Holds I-3.1=违例[C794-Q3-every 隔离位置在走过的版本引用的记录上：[(1, 50180), (0, 50180)]] I-3.11=Holds
```

`C794_Q3=newest`（`out/q3-newest.txt`）：

```
Q5 读法=None 槽 50180: 盘0(已释放=false,隔离位=false,代=3) 盘1(已释放=false,隔离位=true,代=3)
Q5 checker: violated=["I-3.11"] I-1.1=Holds I-3.1=Holds I-3.11=违例[盘 0：记账的已分配 Some(737280) 减 defer 待释放 Some(442368)，不等于从最新根（txg 5）走读到的 262144（其中隔离豁免 0）]
Q5 对照（两盘都置位） 槽 50180: 盘0(已释放=false,隔离位=true,代=3) 盘1(已释放=false,隔离位=true,代=3)
Q5 对照 checker: violated=[] I-1.1=Holds I-3.1=Holds I-3.11=Holds
```

- 「任一走过的版本引用」口径：**打中**，误判合法状态。写者正常写出的对照镜像判 I-3.1 红，引用它的是还在候选集里的 A（I-3.1 的「实际遍历」按全部有效根取并集，见 `crates/singlefs-checker/invariants/05-空间记账.md:8` 的 checker 读法）。四句：分辨口径（newest 那一口径同一张镜像不红）；判别子看得到（checker 知道是哪一版引用的，只是这一口径不分）；满足的是「误判合法状态」；改法就是换成 newest 口径。
- 「最新根引用」口径：没打中。合法镜像不红（上面对照那两行；清位读法下回退前后、照常覆盖写之后也不红，`out/q2-clear-q3-newest.txt`）。它抓得到的写者缺陷量到一种：回退不清位，回退那一版当场判 I-3.11 红（`out/q2-keep-q3-newest.txt` 第二行「C794-Q3-newest 隔离位置在最新根引用的记录上」）——比 Q2 那一格的漏释放早一步、不要第二次缺陷；但那之后照常覆盖写释放会把位清掉，漏释放那一行照样全绿（同一份输出末行），抓不到 Q2 那一形。另一种推的能抓：「新分配的记录带位」（Q8 那条变异），活单元一落盘就红；没在 checker 上量（随机历史快档池级 checker 跑了 0 次，见 Q8）。
- 会不会误判 Q2 那一写落盘前的崩溃状态：推的不会——落盘前的状态最新根是 C（不引用那个单元），落盘后的最新根是回退那一版（位已清），两边都不满足「最新根引用且带位」；回退那次发布 1048588 个崩溃状态没跑（见「崩溃状态」），这一句没量。
- 这条口径是我提的收严，只在我的模型上量过、被攻过零轮。

## Q4 回退判「隔离单元」那一处改不改成看位

今天的认法：「} else if !is_referenced_by_the_current_version {」（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/mount.rs:7822`），每块盘仍分配、同分配代、现行那一版不引用就当隔离单元逐盘验。看位读法开 `C794_ISOLATED_BY_BIT=1`：每块盘上那条记录都带位才算，不带位的落到「Some(_still_allocated_under_the_same_generation) => {}」（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/mount.rs:7814`），当两版共用的单元、不验。

**用例**（`c794_attack_probe q4 [corrupt]`）：A；B 覆盖写，注入写者缺陷「B 漏释放 A 的数据单元」（两盘记录停在仍分配、代 3、不带位）；C；带 `corrupt` 时把盘 1 上那一份翻坏一个扇区；回退到 A。四份原样输出：

```
Q4 corrupt=true 读法 bit=None 回退前 槽 50180: 盘0(已释放=false,隔离位=false,代=3) 盘1(已释放=false,隔离位=false,代=3)
Q4 回退前 checker: violated=["I-3.11"] I-1.1=Holds I-3.1=Holds I-3.11=违例[盘 0：记账的已分配 Some(737280) 减 defer 待释放 Some(442368)，不等于从最新根（txg 5）走读到的 262144（其中隔离豁免 0）]
Q4 回退被拒：IsolatedUnitCopyUnreadableOrMismatched { unit: Data(DataUnitIndexInFile(0)), device: DeviceIdentity(1), slot: SlotNumber(50180) }
```

```
Q4 corrupt=true 读法 bit=Some("1") 回退前 槽 50180: 盘0(已释放=false,隔离位=false,代=3) 盘1(已释放=false,隔离位=false,代=3)
Q4 回退前 checker: violated=["I-3.11"] I-1.1=Holds I-3.1=Holds I-3.11=违例[盘 0：记账的已分配 Some(737280) 减 defer 待释放 Some(442368)，不等于从最新根（txg 5）走读到的 262144（其中隔离豁免 0）]
Q4 回退做成 槽 50180: 盘0(已释放=false,隔离位=false,代=3) 盘1(已释放=false,隔离位=false,代=3)
Q4 回退之后 checker: violated=["I-2.1", "I-4.8", "I-7.4"] I-1.1=Holds I-3.1=Holds I-3.11=Holds
Q4 回退之后 cold=FileRead root=(InstanceGeneration(1), CheckpointTxg(6)) content_matches_expected=true
```

```
Q4 回退做成 槽 50180: 盘0(已释放=false,隔离位=false,代=3) 盘1(已释放=false,隔离位=false,代=3)
Q4 回退之后 checker: violated=[] I-1.1=Holds I-3.1=Holds I-3.11=Holds
Q4 回退之后 cold=FileRead root=(InstanceGeneration(1), CheckpointTxg(6)) content_matches_expected=true
```

```
Q4 回退做成 槽 50180: 盘0(已释放=false,隔离位=false,代=3) 盘1(已释放=false,隔离位=false,代=3)
Q4 回退之后 checker: violated=[] I-1.1=Holds I-3.1=Holds I-3.11=Holds
Q4 回退之后 cold=FileRead root=(InstanceGeneration(1), CheckpointTxg(6)) content_matches_expected=true
```

- 看位读法：**打中**。不带位的漏释放单元有一份坏了，回退不验就做成，新根引用一份对不上的单元，回退之后 checker 判 I-2.1、I-4.8、I-7.4 红；推断读法同一段在任何写之前拒成 `IsolatedUnitCopyUnreadableOrMismatched`（盘 1、那个槽）。
- 四句：分辨臂（同一段历史两读法结局相反）；判别子看得到（看位读法看得到记录不带位，问题是它把「不带位」当成「不用验」）；满足的是正文 Q4 格「改了之后不带位的那一类（只能是写者漏释放）的去向」——去向是「当共用单元、不验、悄悄重新引用」，后果落在 D23 已定项 14 的「其余 cur 的账里仍分配的单元分配器不会发出去，不在逐盘验之内」（`.claude/kb/decisions/23-journal的角色与格式.md:388`）那一句的前提上（那句的前提是共用单元，这一类不是）；跑前条款「那个读法出局」。
- 前提：要先有一次写者漏释放；那之后、回退之前 checker 已经判 I-3.11 红（四份输出第二行都是）。所以这一形打中的是「看位读法让一次已经被 checker 抓到的缺陷在回退时多带出一份坏副本」，不是让一次漏判通过。
- 推断读法（实现员的）：没打中。试过的形：漏释放单元两份都好时回退做成、冷启动读回 A（`out/q4-inference-clean.txt`）；一份坏时拒；合法隔离单元（Q2 用例里的回退）认得出、验过、清位。与 C572 判决的读法一致：那份判决写的是「现行那一版的槽集合恰好认出被隔离的单元」（`research/prompts/code-backlog-b2-code-r1-main-verification.md:22`），推断读法没改这一判。推断读法把漏释放单元也当隔离单元、清位（对不带位的记录是空操作）、重新引用——漏释放在回退之后被「治好」，checker 回退之后全绿（`out/q4-inference-clean.txt` 第四行），之前那一次 I-3.11 红是唯一的报告。

## Q5 豁免按记录还是按单元

**用例**（`c794_attack_probe q5-image`）：A；B 瞬时隔离、两盘都置位；注入写者缺陷「盘 0 那条记录的位没置」（`c794_set_quarantine_flag` 盘 0 置 false）；C。按记录（`out/q5-per-record.txt`）与按单元（开 `C794_EXEMPT_READING=per_unit`，`out/q5-per-unit.txt`）：

```
Q5 读法=None 槽 50180: 盘0(已释放=false,隔离位=false,代=3) 盘1(已释放=false,隔离位=true,代=3)
Q5 checker: violated=["I-3.11"] I-1.1=Holds I-3.1=Holds I-3.11=违例[盘 0：记账的已分配 Some(737280) 减 defer 待释放 Some(442368)，不等于从最新根（txg 5）走读到的 262144（其中隔离豁免 0）]
Q5 对照（两盘都置位） 槽 50180: 盘0(已释放=false,隔离位=true,代=3) 盘1(已释放=false,隔离位=true,代=3)
Q5 对照 checker: violated=[] I-1.1=Holds I-3.1=Holds I-3.11=Holds
```

```
Q5 读法=Some("per_unit") 槽 50180: 盘0(已释放=false,隔离位=false,代=3) 盘1(已释放=false,隔离位=true,代=3)
Q5 checker: violated=[] I-1.1=Holds I-3.1=Holds I-3.11=Holds
Q5 对照（两盘都置位） 槽 50180: 盘0(已释放=false,隔离位=true,代=3) 盘1(已释放=false,隔离位=true,代=3)
Q5 对照 checker: violated=[] I-1.1=Holds I-3.1=Holds I-3.11=Holds
```

- 写者写出的镜像（两盘都置位，对照那两行）：两读法同判，都不红。量过。
- 只一块盘带位（写者缺陷）：按单元**打中**，盘 0 那条仍分配、不带位、没有根引用的记录被盘 1 的位豁免掉，全绿；按记录判 I-3.11 红。四句：分辨臂；判别子看得到（每条记录自己的位在盘上）；满足「漏判真泄漏」——盘 0 那条按 D19 已定项 5 要与盘 1 对称置位，没置就是写者没走隔离那条路；改法：按记录不中。
- 「只置了一块盘就崩」的崩溃状态：没跑。B 那次发布的全量是 16777228 个状态（`out/counts.txt`，单元写那一段 24 次写），越过约 10⁶，照定义那一段不跑。推的：写者两盘一起置位（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-core/src/transaction.rs:8145` 那一行起按位置项把两份都交给 `release_leaving_the_record_allocated_on`），两块盘的两条记录在同一棵分配记录树里、随同一个根落盘，崩溃状态里恢复落到的要么是 B 之前那一版的账（两条都不带位），要么是 B 那一版（两条都带位）；按「由记录施加出来的一版」的前缀第四条（点名单元两份都验过才施加）也拼不出一半。这一句没量。

## Q8 理想模型观测不观测隔离位

今天模型只比每块盘那条记录的设备、代与已释放：「is_released: record.is_released,」（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-harness/src/model_comparison.rs:353`），结构在 `/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-harness/src/model.rs:1093`。

量了三件事（输出在 `out/rh-*.txt` 与 `out/targeted-*.txt`，每份 `rh-` 第一行是这一档里 `C794_QUARANTINE_EVENT` 出现的次数）：

1. 随机历史里隔离发生过几次：快档（96 段、模型对拍 2335 步、比过分配记录 44888 条）与偏向回退那一档（48 段、1448 步、22362 条）都是 **0 次**——历史生成器不注入读错（`/home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/crates/singlefs-harness/src/history.rs` 里 grep 不到读错注入，推的），释放之前读盘核一次都没核出对不上。
2. 快档这棵树上不跑池级 checker：`out/fast-tier-pool-checker-runs.note` 原样「checker 跑了 0 次；一个写都没发、沿用上一次结论的 1243 步；每步镜像交出去 1733 张」，镜像交给 checker 档判（重型，没跑）。harness 档随机历史里判分配记录的只有模型。
3. 两条变异在 harness 档的去向：

| 变异 | 快档 | 偏向回退那一档 | 入库三份隔离用例 |
|---|---|---|---|
| 新分配的记录带隔离位（`C794_MUTANT_NEW_RECORD_BIT=1`） | 全绿（新发现 0） | 全绿 | `dangling_unit_no_root_ever_referenced_violates_both_allocated_statistics` 红，其余绿 |
| 回退不清位（`C794_ROLLBACK_KEEPS_BIT=1`） | 全绿 | 全绿 | `rolling_back_..._written_back_intact_goes_through` 红，其余绿 |
| 不开变异 | 全绿 | 全绿 | 全绿（5 + 2 + 14 条） |

- 实现员的读法（不观测）：没打中「漏判合法状态」那一类，但有一个量到的盲点：变异「新分配的记录带位」在随机历史里活到底（harness 档没有判据看它），只靠一条入库用例（悬空单元那条）在一个手造的形上抓到。
- 对面读法（观测、判活单元的记录不带位）：推的能抓「新分配的记录带位」——模型每一版比每个角色每块盘那条记录（44888 次），活单元带位第一版就对不上；**抓不到「回退不清位」**——那一类要先有一次隔离，随机历史里 0 次（第 1 条量的），观测了也是空比。代价（推的）：`ObservedAllocationRecord` 加一个字段，构造处 `model.rs` 3 处、`model_comparison.rs` 2 处（`grep -c 'ObservedAllocationRecord {'`），判法加一个布尔比较；运行时每条记录多比一个布尔，没量。
- 要让 Q8 的观测碰得到隔离那一类，历史生成器得能注入释放之前的读错（瞬时两次）——这是另一件事，我没做，标「被攻过零轮」。

## 崩溃状态

崩溃枚举经 `enumerate_crash_states_versions`（每一段都展开），基线取要判的那次发布之前的整条流（内存里建一次池），只展开那一次发布与它后面的写。先用 `crash_state_count_with_torn_in_place_overwrites` 数（`out/counts*.txt` 原样）：

```
COUNT flow=quarantine-publish writes=29 segments=[24, 2, 1, 2] root_index=26 states=16777228
COUNT flow=rollback-publish writes=25 segments=[20, 2, 1, 2] root_index=22 states=1048588
COUNT flow=after-rollback-overwrite writes=29 segments=[24, 2, 1, 2] root_index=26 states=16777228
```

- B 那次发布（隔离、置位）与回退之后那次覆盖写各 16777228 个，一段自己就越过约 10⁶，照定义**不跑**，写进这里交主 agent。
- 回退那次发布（清位）1048588 个，在约 10⁶ 上。估时：`sample` 子命令（同一份枚举、只判开头几个状态，单线程）量了 300 个 16.0 秒、1300 个 71.9 秒，边际约 56 毫秒一个（本机 loadavg 70–117，32 核全被占着）；全量单线程约 16 小时，31 线程约 32 分钟起，前台 4 分钟放不下，**没跑**，命令写在「要主 agent 跑的命令」。四种读法合起来 4194352 个，在约 10⁷ 之内；这一条腿已经跑过的全量状态数是 0（估时只判了 1600 个）。
- 缩没缩：没缩崩溃状态；历史已经是最短的（一次隔离或一次回退各一次发布），没有再缩的余地。

## 没打中的形状（试过的、取样范围）

| 格 | 读法 | 试过的形 |
|---|---|---|
| Q1 | 对面（checker 也判红） | 找写者正常路径能写出两位同置的地方：`to_bytes` 的断言挡住，发布路径只经它编码（推的，grep 过 `ALLOCATION_RECORD_RELEASED_FLAG` 的编码处）；崩溃状态里叶是整单元校验，推的造不出 |
| Q2 | 清位 | 回退做成之后、照常覆盖写释放之后两张镜像 checker 全绿；入库三份隔离用例全绿；回退那次发布的崩溃状态没跑 |
| Q3 | 不判 | 没找到它判错的镜像；它的代价是 Q2、Q8 里那两种缺陷各晚一步或抓不到 |
| Q3 | 最新根口径 | 合法镜像四张（隔离之后、回退之后、照常覆盖写之后、漏释放之后）都不因这一口径红 |
| Q4 | 推断 | 漏释放单元两份好 / 一份坏、合法隔离单元回退；都与条款一致 |
| Q5 | 按记录 | 写者两盘都置位的镜像不红；单盘置位只来自写者缺陷 |
| Q8 | 不观测 | 两档随机历史各 96、48 段，变异与不变异都全绿，没有误判；盲点见 Q8 节 |

每个镜像场景一段历史（A 之后 3–5 次发布），没有随机取样；用户动作（回退目标、覆盖写次数）是写死的：回退目标只试了 A，没扫别的目标与「回退之后先做什么」——Q2、Q4 的打中都要一次写者缺陷，不依赖回退之后做哪一步（推的）。

## 这条腿自己的限度

- 打中的五处（Q1 实现员读法、Q2 不清、Q3「任一走过的版本」口径、Q4 看位、Q5 按单元）里，Q2、Q4、Q5 都要先有一次写者缺陷（漏释放、单盘置位），Q1 要一次绕开编码断言的写者缺陷；只有 Q3 那一口径是在写者正常写出的镜像上误判。缺陷是用 `c794_set_quarantine_flag`、`resurrect_released_record` 与编码开关在分配器里直接造的，不是从某条真实代码路径走出来的。
- 全部是副本上的数：冻结副本打了 `patches/attack-switches.diff`；读法开关是我写的，与将来真改的样子可能不同。副本上的数不进 kb，要在入库装置上重做。
- 崩溃状态一个都没判：B 那次发布与回退之后那次覆盖写各 16777228 个，越过约 10⁶ 不跑；回退那次发布 1048588 个，前台放不下，交主 agent。Q2「清位那一写落盘前的崩溃状态」、Q3「会不会误判 Q2 那一写落盘前的崩溃状态」、Q5「只置了一块盘就崩」三句都只有推的。
- GPU 那几处（恢复内核、checker 扫描内核）只读代码，没起卡。
- 可写挂载那条路没在原型里跑，Q1 只量了冷启动走读。
- 回退目标只试了 A，回退之后的用户动作只试了「覆盖写一次」。
- Q3 的「最新根」口径、Q1 的改法甲乙、Q8「历史生成器注入释放前读错」都是我提的，只在我的模型上量过或推过、被攻过零轮。
- 这一份是一次抽样；「没打中」的格要再抽一次才算（`.claude/rules/three-way-inference.md` 一条腿只抽一次样那一节）。

## 要主 agent 跑的命令

回退那次发布的崩溃枚举（每一段都展开，四种读法各 1048588 个状态、合 4194352 个；每个读法一个新的库目录，库里按块复用结论、不认环境变量，混用会把一种读法的结论当另一种的用）。准备一次：

```
W=<新的草稿目录>
mkdir -p "$W/tree" && rsync -a --exclude target --exclude .git /home/fy5090/.local/share/singlefs/work/c794-bit-r1/repo/ "$W/tree/"
cd "$W/tree" && patch -p1 < /home/fy5090/code/singlefs/research/prompts/c794-bit-r1-opus-model/patches/attack-switches.diff
cd "$W/tree" && CARGO_TARGET_DIR="$W/target" bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh 24G \
  bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 31 cargo build --release -p singlefs-checker-tier --bin c794_attack_probe
```

然后逐个读法跑（工作目录 `$W/tree`，内存上限 24G，线程 31；`CARGO` 指编它的那个 cargo，引擎记工具链要它）：

```
for spec in "baseline C794_NONE=1" "per-unit C794_EXEMPT_READING=per_unit" "q3-newest C794_Q3=newest" "keeps-bit-q3-newest C794_ROLLBACK_KEEPS_BIT=1 C794_Q3=newest"; do
  set -- $spec; label=$1; shift
  env CARGO=/home/fy5090/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo SINGLEFS_CRASH_AMPLIFICATION_LIBRARY="$W/lib-$label" "$@" \
    bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh 24G bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh 31 \
    "$W/target/release/c794_attack_probe" enumerate rollback-publish "$label"
done
```

每个读法看 `ENUM flow=rollback-publish label=<读法> states=1048588 … checker_violated={…}` 一行与随后的 `ENUM_FIRST` 行。推的预期：baseline、per-unit、q3-newest 三个 `checker_violated={}`；keeps-bit-q3-newest 在回退那一版落到的状态上 I-3.11 红（详情以 `C794-Q3-newest` 开头）。前三个里任何一个不空，就推翻 Q2「清位读法崩溃状态上没打中」、Q3「最新根口径不误判 Q2 那一写落盘前的崩溃状态」、Q5「按记录与按单元在崩溃状态上同判」里对应的那一句。挂钟估每个读法 31 线程 30 分钟以上（单线程边际 56 毫秒一个，本机满载时量的）。

B 那次发布（隔离、置位）16777228 个状态，越过约 10⁶，我没有给命令；要判「只置了一块盘就崩」得主 agent 定要不要放开上限。

## 没做什么

- 没跑任何门禁阶段、checker 档测试、GPU；没跑崩溃枚举全量（上一节）。
- 没改仓里的文件，只写了这份报告与模型目录；冻结副本没动，改动全在草稿目录的拷贝里。
- 没判正推腿那几格，没读禁读清单里的文件。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/c794-bit-r1-opus/` 里的仓拷贝 `repo/` 与编译目录 `target/` 交回前删了；留着 `runs/`（原样日志）、`build*.log`、两份产出脚本与 `model-sha256.txt`。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/c794-bit-r1-opus-output.md --root /home/fy5090/code/singlefs` 末行（追加这一节之前跑的）：

```
  ✓ 核了 16 处引文、7 处裸行号，对上 16 处，没判 7 处（其中裸行号没有引文可比 7 处；逐处列在上面）
```
