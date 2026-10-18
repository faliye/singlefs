# m3-b44-code-r2 攻方腿第二次抽样（opus-s2）报告（2026-10-17）

## 复跑

模型目录 `research/prompts/m3-b44-code-r2-opus-s2-model/`。复跑（整条经内存包装，工作目录随意；编译占大头，前台 4 分钟放不下，交主 agent 后台跑）：

```
bash scripts/hosts/run-with-memory-cap.sh 6G bash research/prompts/m3-b44-code-r2-opus-s2-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-b44-code-r2/repo \
  --work <新的草稿目录>
```

它拷冻结副本、打上 `prototype/mount-measurement-hook.diff`（只加一个原子计数标出「只进内存那一趟」的读，不改行为）、放进 `prototype/attack_opus_s2_rehearsal.rs`，编，跑窗口 2 的小样、窗口 1 的同一段与 J1 那一形，最后与 `out/window-2-from-30-step-1.tsv` 逐行比。rerun.sh 本身没有端到端跑过（里面每一步都在草稿目录里单独跑过，见各节）。

| 文件 | sha256 |
|---|---|
| out/coarse.tsv | dfd2a06d442212d283ae9e844318d9ec46724bac0f8b835d913163bbd8650c55 |
| out/j1-extent.log | 0c3ddd3f8d13edb2adb70232c0f2ba5b9f619480847ef251e109815d76b8a3ed |
| out/reads-of-the-refused-mount.tsv | 04b69590b4d48f1f3eabfab57498fa05d5432c3fcb0c521f43296443d774ac7f |
| out/window-1.tsv | da8771229621b8ae23d4551a4e6cbaf0bade428341965ab5cc07a9582d15e777 |
| out/window-2-from-30-step-1.tsv | a7b125488b5392a1c3a61a26c440790727f2548cad5795fd343706d96015c03d |
| out/window-2.tsv | b08f5483fd44626f3edca333dd02a5f2b2957e7f9f6977dc917951c487db70a4 |
| prototype/attack_opus_s2_rehearsal.rs | 2577662a988a67b6c66efe2fee18206ddcb41c7039ecc61e265a762d8ebba941 |
| prototype/mount-measurement-hook.diff | 8d631e83ac1f275ea1770a96114fabf660beca55b3f5b2f55cd16d45c968310b |
| tools/rerun.sh | 51a2bc814d1f25602db955f6b985139113474e466224bf707dc0532d2b11a3c9 |
| tools/sweep-batch.sh | c923b9f87bd9412d945816ccb7442065a8349c5bd06179b31900e42da567c644 |
| tools/window-batch.sh | f6419007f13d18869f18c6a9fe9896953d70dd3d71ae70aaf402ac7a75f21e99 |

`tools/sweep-batch.sh`、`tools/window-batch.sh` 是草稿目录里分批并行跑扫描用的驱动，路径写死在草稿目录，只作记录。所有数都是在冻结副本的拷贝（草稿目录 `repo/`）上量的，是副本上的数。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| J3 | 打中 | 窄盘 C841 镜像上，只进内存那一趟（推抬 F 那一格的预演）里连续两次读错，预演交 `None`、取号之前那一判放过；真做那一趟读得通，取号、写行、暖机、推抬 F、续做 15 步落盘之后才交 C841，盘变了。764 个窗口起点里 62 个打中；单次读错 764 个一个不中 |
| J2 | 打中（同一段，经 J3 的路径） | 「预演放过而真做在取号之后交 C841（盘变了）」那一形，就是上一行那 62 格；不带故障的形（分配器拷贝没放开扣住位、取号前那一判排在逐盘核之前）只推了、没造出判错 |
| J3 读数与内存 | 量到读数，内存没量 | 那一份镜像上挂一次 1070 次读，其中 764 次落在只进内存那一趟，取号之前那一判之前的 306 次之外多出 2.5 倍；只进内存的盘的内存占用没量（推的上界：被写过的不同物理块数，最多到盘上被写到的那几段的大小） |
| J1 | 没打中 | 试了一形（extent 上段叶里给 2 号 inode 追加一条逆序指针的条目），重建在 I-1.1 上先拒，没走到 I-2.5 那一判与写者断言 |
| J4 | 没打中 | 查了 8 处按 `MountError` 分支的调用方，没有一处靠这个成员断定「写过」；但同一成员同一原因（C841）现在既可能写账全空、也可能写了 19 次，调用方今天只看原因不看写账 |
| J5 | 没做 | 余力不够 |

## J3（连同 J2）：只进内存那一趟吃掉读错，取号之前那一判放过，真做在写之后交 C841

### 历史

起点是冻结副本自己的用例 `a_writable_mount_whose_deletion_continuation_raises_the_floor_before_its_final_batch_opens_a_retirement_container_is_refused_before_acquisition` 用的那一份镜像（不补环窗口的窄盘、单元区 304 槽，第一个文件 16 个数据单元、删除队列 {1}，退役树唯一那片容器号改成 `u64::MAX`；造法原样拷进原型，`narrow_pool_with_one_leaf_retired_and_the_sixteen_unit_first_file_queued_for_deletion`）。批大小一个单元、退役容器一片 1 条的删除开关做可写挂载。不注入时它在取号之前拒、写账全空、两块盘不变（下面 `# baseline` 那一行）。

故障：两块盘外面各包一层带读序号的盘（原型里的 `LoggingDevice`，两块盘共用一个序号），让序号 n 与 n + 1 两次读报 `BlockDeviceError::InputOutput`，别的读原样转下去。n 扫过「只进内存那一趟」里的每一次读（764 个），每个 n 用同一份镜像新开两块盘挂一次。打中的那几格里报错的两次读都落在那一趟里（`all_inside_walk=true`，由测量钩子判），真做那一趟一次读错都没碰到。

原样输出（`out/window-2-from-30-step-1.tsv`，窗口 2，只进内存那一趟里第 30 起 8 个起点）：

```
# baseline	Publish cause=NextContainerNumberPastTheTopOfItsRange c841=true persisted=0 failed=0 changed=false	reads=1070	walk=764
337	Publish cause=NextContainerNumberPastTheTopOfItsRange c841=true persisted=0 failed=0 changed=false	fired=2	all_inside_walk=true	read=Some((337, true, 1, 0, 4096))
338	Publish cause=NextContainerNumberPastTheTopOfItsRange c841=true persisted=0 failed=0 changed=false	fired=2	all_inside_walk=true	read=Some((338, true, 1, 4096, 4096))
339	Publish cause=NextContainerNumberPastTheTopOfItsRange c841=true persisted=19 failed=0 changed=true	fired=2	all_inside_walk=true	read=Some((339, true, 0, 0, 4096))
340	Publish cause=NextContainerNumberPastTheTopOfItsRange c841=true persisted=19 failed=0 changed=true	fired=2	all_inside_walk=true	read=Some((340, true, 0, 4096, 4096))
341	Publish cause=NextContainerNumberPastTheTopOfItsRange c841=true persisted=19 failed=0 changed=true	fired=2	all_inside_walk=true	read=Some((341, true, 1, 0, 4096))
342	Publish cause=NextContainerNumberPastTheTopOfItsRange c841=true persisted=0 failed=0 changed=false	fired=2	all_inside_walk=true	read=Some((342, true, 1, 4096, 4096))
343	Publish cause=NextContainerNumberPastTheTopOfItsRange c841=true persisted=19 failed=0 changed=true	fired=2	all_inside_walk=true	read=Some((343, true, 0, 0, 4096))
344	Publish cause=NextContainerNumberPastTheTopOfItsRange c841=true persisted=19 failed=0 changed=true	fired=2	all_inside_walk=true	read=Some((344, true, 0, 4096, 4096))
```

`read=` 那一列是（序号, 在不在那一趟, 盘, 偏移, 长度）。339 那一格：盘 0 的系统配置两槽（偏移 0 与 4096）接连读错。`persisted=19` 是 `PublishSequenceFailed::writes_of_persisted_publishes` 的条数，`changed=true` 是挂完之后两块盘与挂之前逐字节比不等。

全量（764 个起点，8 件并行各取步长 8 的一份，`out/window-2.tsv`；窗口 1 同样 764 个，`out/window-1.tsv`），命令数的：

```
$ grep -c 'persisted=19' window-2.tsv
62
$ grep 'persisted=19' window-2.tsv | cut -f1 | awk '…连续段…'
339-341
343-345
953-976
985-1016
$ grep -c 'persisted=0 failed=0 changed=false' window-1.tsv
764
```

窗口 1（单次读错）一格不中：只进内存那一趟里读系统配置、读根环槽、读释放核验都各有重读或另一槽兜着。953–1016 两段是那一趟里推抬 F 读根环槽与码 2 节点的读（`out/reads-of-the-refused-mount.tsv` 里这几个序号的偏移在 1052672、4194304、7340032 附近与单元区 23–25 MiB）。

### 每一步许可它的那一句

1. 分配器拷贝上那一趟被拒是空间不够时，换在只进内存的盘上走，那里交 `None` 就不拒：「Some(refusal_of_the_continuation) => refusal_of_the_continuation,」（`crates/singlefs-core/src/mount.rs:5285`），「None => return Ok(()),」（`crates/singlefs-core/src/mount.rs:5286`）。
2. 那一趟的读穿到底下那块盘：「预演用的盘：读穿到底下那块盘（这一刻盘上的字节）」（`crates/singlefs-core/src/mounted_session.rs:356`），「(*identity, DeviceKeepingWritesInMemory::over_the_disk(disk))」（`crates/singlefs-core/src/mount.rs:6963`）。底下那块盘报的读错原样交给那一趟。
3. 那一趟在续做之前报任何错都折成 `None`：「.ok()?;」（`crates/singlefs-core/src/mount.rs:6980`）。文档写的理由：「或取号之后那一串在续做之前就报错，交回 `None`：那些照旧留给真发，真发时报它自己的。」（`crates/singlefs-core/src/mount.rs:6945`）——这句默认真做会撞上同一个错；读错是瞬时的时候真做撞不上。
4. 续做里推抬 F 自己报错（`MountError::FloorRaiseFailedAfterTheMountsPublishes`）不是 `MountError::Publish`，同样折成 `None`：「if let Err(MountError::Publish(failed)) = continuation {」（`crates/singlefs-core/src/mount.rs:6992`）。953–1016 那两段走的是这一条（推的：按偏移与这一句推，没在那一趟里面打印错的成员）。
5. 放过之后真做照常取号与写：调用处的注释「有一步要开的容器号到顶（C841）就在取号之前拒，盘上逐字节不变」（`crates/singlefs-core/src/mount.rs:6319`）在这几格上不成立；文档「同一份计划、同一个推抬 F 的判法与同一段抬 F，续做报的错就是真发时会报的那一个。」（`crates/singlefs-core/src/mount.rs:6941`）只在两趟读到同样结局时成立。

### 打中之后的四句

- 分不分辨臂：分。被判的是「取号之前那一判按只进内存那一趟的结局拒不拒」这一种做法；把那一趟自己没走完（续做之前报错、推抬 F 报错）当成「判不出」另行处置的改法（下面甲、乙）在这几格上不中。没有不带故障也打中的形，打中靠的是那一趟比真做多出的读。
- 系统当时看不看得到判别它的东西：看得到。第 3、4 步折成 `None` 之前，手里就是那一趟报的错（`MountError`），里面分得出「读错」与「续做走到空 / 推满仍放不下」；是代码把它丢了。
- 满足的是判据字面的哪一个分句：J3 验收「带读故障注入的设备上预演多出的读让真做那一趟看到不同的读结局（故障被预演吃掉）」与 J2 验收「预演放过而真做在取号之后交 C841（盘变了）」两句同时满足。⚠️ 故障是我原型里的两次连发读错，不是 harness `FaultSchedule` 的单点那一形（`FaultOccurrence::TheNthMatchingCall` 一次只注一个，「只在第 n 次命中的调用上注入一次，之后放行（随机注入那一路用这一种：一段历史一个注入点）。」（`crates/singlefs-harness/src/fault_injection.rs:459`））；单点那一形我全扫了，一格不中（上面窗口 1）。所以这一格落在「条款对连发瞬时读错没有一句话」的那一侧，按跑前条款第三行可以交用户定。
- 跑前条款给的改法在打中的格上还中不中：跑前条款只写「那一处要改」，没给具体改法；下一节的甲在 62 格上推的不中，乙只缩小窗口、窗口 3 起推的照样中。

### 改法（我自己提的，只在我的模型上量过打中、改法本身一个都没实现，被攻过零轮）

| 改法 | 做什么 | 62 格上 | 单点读错（764 格） | 不带故障的现有三条 C841 挂载用例 |
|---|---|---|---|---|
| 甲 | 只进内存那一趟交三种结局：走到判定（今天的 `Some` / `None`）、没走完（续做之前报错、推抬 F 报错、读错），没走完时在取号之前拒、交一个新的类型化错误（写账空），不当成「放过」 | 推的：不中（这几格那一趟都没走完） | 推的：不变（都走完了） | 推的：不变 |
| 乙 | 那一趟没走完时整趟重走一次 | 推的：窗口 2 的 62 格多半不中；窗口 3 起、或读错持续两趟的仍中 | 推的：不变 | 推的：不变 |
| 丙 | 推抬 F 要读的（根环、实例表、被抛弃根的账）在取号之前读一次，预演与真做共用这一份读 | 推的：不中（两趟读同一份） | 推的：不变 | 推的：不变，但改动面大，抬 F 的读者要换入口 |

甲的代价：只进内存那一趟遇到瞬时读错时可写挂载被拒（用户重挂），而今天这次挂载会照常写下去。这一取舍条款没写（见上面四句第三句）。

## J3：只进内存的盘的其它几样

- 读数（量的，`out/reads-of-the-refused-mount.tsv`，第二列是「在不在那一趟」）：同一份镜像挂一次 1070 次读，764 次在那一趟里、306 次在它之外。命令：`awk -F'\t' '$2=="true"' reads-of-the-refused-mount.tsv | wc -l` 出 764，`$2=="false"` 出 306。306 次是不推抬 F 预演时这次挂载取号之前本来就有的读（推的：那一判拒在取号之前，306 次里没有真做的读）。
- 单点读故障（harness `FaultSchedule::the_nth_call_across_the_pool` 那一形，量的）：整池第 n 次读注入 `ReadFails`，n 取 1、11、21……1061 共 107 个（粗扫，`out/coarse.tsv`），107 格全是取号之前拒、盘不变；窗口 1 在那一趟里逐个扫 764 格，同样全不变。单点那一形吃不出差别。
- 写零、读没写过的块、设备报错：没造用例。只进内存的盘不转发 `journal_record_offsets_hint`（`DeviceKeepingWritesInMemory` 没有覆盖这个方法，落到默认 `None`），推的后果是那一趟里扫 journal 环时按全环逐槽扫、比真做多读；不改结局（那个方法的文档要求提示集合罩住全环扫描会读到的每一条）。没量。
- 内存上界：没量。推的：那一趟把写过的每个物理块存进 `BTreeMap`，占用等于取号、写行、暖机、推的抬 F 与续做每一步写过的不同块数之和，上界是这几次写落到的那几段（单元区、journal 环、根环、系统配置）的大小，与续做步数线性涨、到单元区被写遍为止。这一份镜像上那一趟发了 19 次发布（与打中那几格真做落盘的条数相同），字节数没数。

## J2：不带故障的几形（都没造出判错）

1. 分配器拷贝没放开扣住位：真做在续做之前放开重建时扣住的那一截（「allocator.release_reclaim_holds();」（`crates/singlefs-core/src/mount.rs:6864`）），分配器拷贝那一趟接的是没放开的那一份（「allocator_after_the_last_publish: copy,」（`crates/singlefs-core/src/mount.rs:5224`））。两边落点可以不同；但 C841 只看容器树的内容（下一个号 = 最右那片的号 + 1，与落点、txg 无关，`crates/singlefs-core/src/container_tree.rs` 的 `append_records_to_the_open_container`），拷贝更紧只会更早走到只进内存那一趟。推的、没造。
2. 取号之前那一判排在逐盘核之前（调用在 `crates/singlefs-core/src/mount.rs:6320`，逐盘核在 `crates/singlefs-core/src/mount.rs:6344`）：一组盘既有一块不带所选那一版、续做又会撞 C841 时，今天报 `Publish`（C841），W8 之前报 `WritableMountRefusedByDevicesWithoutTheSelectedVersion`；只进内存那一趟也会在这样的盘上走。没造这一组盘，不知道那一趟会不会撞断言。
3. 拷贝那一趟停在一个真做不会撞的「别的错」上、真做接着走到 C841：候选只有依赖分配器状态的几种（固定点不收敛、多层码 2 树算不出形状），没找到造得出的形。
4. 误拒：释放核验隔离了一份（真做那一槽留在已分配、拷贝照常释放）又恰好卡在空间边上，真做推满仍放不下、挂载照样做成可写而队列留着，拷贝那一趟却走到 C841 拒。推的、没造。

## J1：没打中

试的形：第一个文件三个数据单元那一版（`pool_with_a_three_unit_file`），在 extent 上段根（层级 0 的叶）里照抄第一个文件那一条、inode 号改成 2、指针两条位置条目的设备身份对调（逆序），改头里条目数与声明长度，重封、校验和补到根槽；挂可写，挂成了就 unlink 第一个文件走到底（最后一批重写那片叶，会照抄 2 号那一条，转写在 `crates/singlefs-core/src/transaction.rs:7311` 那一段：「同一片叶里别的 inode 的条目照抄上一版那片叶的（第一版只有第一个文件有内容，今天这一串是空的）。」（`crates/singlefs-core/src/transaction.rs:7311`））。

- 头里最大 key 跟着改成 2 号那一条：挂载拒，`Recovery(InvariantViolated { invariant: "I-1.1", detail: "extent 树节点头里的 key 区间不是它的位置规定罩的那一段" })`（这一次的输出没留文件，原样抄自这次会话的终端输出）。
- 头里 key 区间不动：挂载拒，原样（`out/j1-extent.log`）：`J1 extent leaf: mount=refused Recovery(InvariantViolated { invariant: "I-1.1", detail: "同一个 extent 树节点被两条父条目引用" })`。照抄来的那条是指下段根的那一种，与 1 号那一条指同一个节点。

没试的：2 号那一条换成内联数据单元指针、指一个新的单元；换成指一个新造的下段节点。按重建报的第二个错推，重建会顺着别的 inode 的条目往下走，那些指针多半随走读进了 I-2.5 那一判的清单，但我没核到它们落进哪个字段。写者一侧我列了 `NodePointer::write_to` / `DataPointer::write_to` 的调用处（`grep -n` 出 22 行，见「没打中的形状」），照抄盘上指针写回的只找到上面那一段与已在清单里的几类。

## J4：没打中

查的调用方（冻结副本与主工作区这几份逐字节相同，`cmp` 现核）：

| 调用方 | 按 `MountError::Publish` 做什么 | 在取号之前那一拒上成不成立 |
|---|---|---|
| `crates/singlefs-core/src/mount.rs` 的 `floor_raise_refused_for_space` | 归「不是空间不够」 | 成立 |
| `crates/singlefs-harness/src/model_comparison.rs` 的 `refusal_reason_of_mount_error` | 按 `cause` 取理由：「MountError::Publish(failed)」（`crates/singlefs-harness/src/model_comparison.rs:815`），C841 映射成没理由：「PublishError::NextContainerNumberPastTheTopOfItsRange(_)」（`crates/singlefs-harness/src/model_comparison.rs:725`） | 成立；随机历史走不到 C841（「墓碑树、谱系树或退役树最右那片的容器号已到顶（C841）：容器号从 1 起逐片加一，健康的内存盘上走不到，坏镜像才有。」（`crates/singlefs-harness/src/model_comparison.rs:724`）） |
| 同一文件 `root_ring_slot_still_bad_after_one_reread_of_mount_error`、`reported_ceiling_of_mount_error` | 交 `None` | 成立 |
| `crates/singlefs-harness/src/history.rs` 的 `mount_error_member` | 只取成员名与原因名 | 成立 |
| `crates/singlefs-harness/src/history.rs` 可写挂载那一步的 `Err(error)` 分支 | 记成「拒」、模型按拒之前一个字节都不写判 | 取号之前那一拒一个字节都不写，成立；注释写的是「挂载半路报错（写行之后的发布失败）」，与这一拒的时机不符，只是注释 |
| `crates/singlefs-harness/src/mount_transparency.rs` 的 `refusal_reason_on_the_d23_list` | 不在 D23 单子上，交 `None` | 成立 |
| `crates/singlefs-harness/src/scenario.rs` | 只接新池新建文件那一串的写账，不碰挂载 | 不涉及 |
| `crates/singlefs-core/src/mount.rs` 只进内存那一趟自己 | 只认 `MountError::Publish` 当续做的判定（`crates/singlefs-core/src/mount.rs:6992`） | 见 J3：别的成员一律当放过 |

没找到把这一拒当成「写了一半」的调用方。要记的一点：同一个成员、同一个原因（C841）今天有两种写账——取号之前拒时全空（「续做里有一步要开的容器号到顶（C841）在取号之前预演出来时同样交这个成员，那时一个写都没发，三份写账都是空的」（`crates/singlefs-core/src/mount.rs:118`）），J3 那 62 格里 `writes_of_persisted_publishes` 是 19 条；调用方要分清「盘变没变」只能看写账，今天没有一处看。

## 没打中的形状

| 形 | 取样 | 结局 |
|---|---|---|
| 单点读错（harness 那一形），整个挂载 | 1070 次读里步长 10 取 107 个 | 107 格全是取号之前拒、盘不变 |
| 单点读错，只进内存那一趟 | 764 个读序号逐个 | 764 格全是取号之前拒、盘不变 |
| 连发两次读错，只进内存那一趟 | 764 个起点逐个 | 702 格不中、62 格中（上文） |
| 读回改坏的字节 | 原型里写了扫描（`attack_s2_read_returns_corrupted_bytes_swept_over_every_read_of_the_mount`），没跑 | 没判 |
| J1 extent 上段叶别的 inode 那一条 | 两种头部写法 | 都被 I-1.1 先拒 |
| J1 写者调用处 | `grep -n 'pointer\.write_to\|pointer.write_to\|\.root\.write_to\|child\.write_to\|_pointer\.write_to\|\.write_to(&mut' *.rs \| grep -v '^pointer.rs' \| wc -l` 出 22 | 照抄盘上指针写回、又不在清单里的，只找到 extent 上段叶别的 inode 那一段，没造通 |
| J2 不带故障的四形 | 只推、没造 | 见 J2 一节 |
| J4 调用方 | 8 处 | 见上表 |

只扫了一份镜像（窄盘、16 个单元、续做里推一串抬 F 的那一份），没换几何、没换续做长度；窗口只扫了 1 与 2。

## 要主 agent 跑的命令

1. 复跑模型（核 62 格的签名）：工作目录随意，环境变量不用另设，内存上限 6G，线程 31（脚本里经 `scripts/hosts/capped.sh 31`）：
   `bash scripts/hosts/run-with-memory-cap.sh 6G bash research/prompts/m3-b44-code-r2-opus-s2-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-b44-code-r2/repo --work <新的草稿目录>`
   看末尾 `name=compare_window_2` 之后是不是 `same`；`name=window_2_slice` 下 339、340、341、343、344 五行带 `persisted=19 … changed=true`。编译从零起要几分钟，放后台跑。
2. 没有重型测试要跑；没起门禁阶段。

## 这条腿自己的限度

- 故障形是我自己写的两次连发读错，harness 现成的单点一形打不中；它在条款里算不算该防的故障，我判不了，交用户。
- 测量钩子（`prototype/mount-measurement-hook.diff`）改了副本里的 `mount.rs`：只在只进内存那一趟进出时加减一个原子计数，读序号与不加钩子时相同（推的：钩子不读不写盘）；`all_inside_walk` 一列靠它。
- 953–1016 两段我按偏移与第 4 步那一句推成推抬 F 报错，没在那一趟里面把错的成员打印出来；339–345 那一段推成那一趟取号读见证时两槽读不出（同盘两槽接连读错），同样没打印。
- 改法甲、乙、丙一个都没实现、没跑。
- 一份镜像、一种删除开关、一种盘宽；崩溃状态没枚举（这一格是读故障，不是崩溃点）。
- 数全是副本上的数，没在入库装置上重做。

## 没做什么

- J5（注释说的是不是今天代码做的事）没做。
- 读回改坏字节那一扫写了没跑；窗口 3 及以上没扫；内存占用没量。
- J1 只试了 extent 上段叶一处，没造内联指针与新下段节点两形。
- J2 不带故障的四形只推、没造。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-b44-code-r2-opus-s2/` 里的仓副本 `repo/` 与编译目录 `target/` 交回前删掉（原型与补丁已拷进模型目录）；`logs/`、`out/`、`out-small/`、`rc/`、`tools/` 留着，是这一轮的原始输出与驱动。
