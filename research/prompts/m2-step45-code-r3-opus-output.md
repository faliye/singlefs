# m2-step45-code-r3 云端攻方腿（Opus）：X1 / X2 / X3 / X8

轮名 m2-step45-code-r3。攻的是第二轮与 alloc-basis 第二轮打中之后落地的三处改法（T1 影子账 G5、T2 读不出的被抛弃根跳过并计数、T3 抬 F 回收的槽扣住到生效），分到的格是 X1、X2、X3、X8；X4、X5 没碰。

## 复跑

模型只用 `singlefs-core` 的公开 API，不改被判代码。副本在 `/tmp/claude-1000/m2-step45-code-r3-opus/copy/`（`rsync -a --exclude target --exclude .git`），副本上的数都注明是副本。

```bash
rsync -a --exclude target --exclude .git /home/fy5090/code/singlefs/ /tmp/claude-1000/m2-step45-code-r3-opus/copy/
mkdir -p /tmp/claude-1000/m2-step45-code-r3-opus/copy/crates/singlefs-core/tests
cp /home/fy5090/code/singlefs/research/prompts/m2-step45-code-r3-opus-model/x8_hold_and_isolate_bits.rs \
   /tmp/claude-1000/m2-step45-code-r3-opus/copy/crates/singlefs-core/tests/
cd /tmp/claude-1000/m2-step45-code-r3-opus/copy && nice -n 19 cargo test -p singlefs-core --test x8_hold_and_isolate_bits
```

| 文件 | 行数 | sha256 |
|---|---|---|
| `research/prompts/m2-step45-code-r3-opus-model/x8_hold_and_isolate_bits.rs` | 218 | `8a155f8eb45dfe649ae85791637c137318dfca359771d29f76515425349a6c7a` |

副本上这一次的原样输出（`cargo test` 末 8 行）：

```text
running 4 tests
test a_two_slot_record_of_an_abandoned_root_is_skipped_whole_when_its_start_slot_is_exempt_so_the_second_slot_stays_free ... ok
test the_bump_path_hands_out_a_slot_that_is_free_map_says_is_not_free_when_it_is_isolated_after_the_segment_was_opened ... ok
test once_the_holds_are_left_set_every_later_allocation_keeps_failing_until_release_reclaim_holds_is_called ... ok
test holding_the_reclaimed_segment_makes_the_next_commit_generated_allocation_fail_while_the_free_count_says_there_is_room ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| X8 T3 扣住 | **打中 2 条**（X8-A 可达并量过；X8-B 机制量过、可达性没构造出来） | X8-A：抬 F 的发布循环里一旦 `?` 返回，`release_reclaim_holds()`（`crates/singlefs-core/src/mount.rs` 第 501 行）就走不到，而记账已经先动过 —— 这个进程的池从此发不出任何固定点，而它的空闲计数还报着那些槽。X8-B：扣住位与隔离位对提交内生块的 bump 路零约束，`allocate_commit_generated` 与 `PoolAllocator::record` 一个字都不看它们 |
| X1 T1 的隔离集 | **打中 1 条（形状级，不分辨臂）** | 豁免集与隔离集都按分配记录的**起点槽号**做 key，而 `DeviceFreeMap::isolate` 按整条记录的**跨度**铺位：一条跨度 2 的被抛弃记录只要起点槽号被豁免，整条跳过，第二槽没有任何东西罩着 |
| X2 T1 的时机 | **没打中** | 崩在两次带新 F 的空发布之间、重开之后隔离集确实与崩之前不同（少隔离），但差的那几个槽在新账里是「已释放、未回收」，`is_free` 为假，发不出去；checker 在我能到的每一个盘上状态都与记账说同一件事 |
| X3 T2 跳过的根 | **打中 1 条（T2 只落实了一半）** | `raise_rollback_floor`（`crates/singlefs-core/src/mount.rs` 第 456-463 行）那一处调用把 `isolate_slots_referenced_only_by_abandoned_roots` 的返回值直接丢掉：抬 F 时读不出账的被抛弃根被跳过，而计数一个字都没留 |

引用的 kb 行号都去 kb 文件里现查过，不是从背景材料里数的。

## X8 T3 扣住：两条打中

### X8-A（可达、量过）：抬 F 的发布一旦失败，扣住位没有任何人清，而记账已经先动过

**机制。** `crates/singlefs-core/src/mount.rs` 第 465-468 行先回收并扣住：

```rust
    let reclaimed = allocator.reclaim_released_up_to(
        reclaim_floor(new_floor, oldest_valid_root),
        ReclaimedReuse::HeldUntilFloorTakesEffect,
    );
```

`DeviceFreeMap::mark_reclaimed`（`crates/singlefs-core/src/allocator.rs` 第 242-261 行）在这一步就把 `allocated_slots -= span`、`deferred_slots -= span`、`free_slots += span` 全做完了；`hold_until_floor_takes_effect`（同一文件第 221-232 行）只另置一个位。清这个位的只有 `PoolAllocator::release_reclaim_holds`（同一文件第 607-611 行），它在 `mount.rs` 里只有第 501 行一处调用点，而第 493 行的 `?` 排在它前面：

```rust
        )?;
        let device = device_of_txg(next.root.checkpoint_txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
        publishes.push(next.clone());
        *current = next;
    }
    allocator.release_reclaim_holds();
```

发布失败时 `publish_version` 会把分配器退回**这一次发布之前**（`crates/singlefs-core/src/transaction.rs` 第 955-958 行的 `allocator_before_this_publish` 与 `*allocator = allocator_before_this_publish`），退回的那一版里回收与扣住都已经在了 —— 退回帮不上忙。于是 `raise_rollback_floor` 返回 Err，调用方手里那个 `&mut PoolAllocator` 从此带着一批「记账算空闲、分配器不发」的槽。

**为什么发布会失败，而且正好在这一格。** 带新 F 的空发布自己要分配固定点，走 `allocate_commit_generated`（`allocator.rs` 第 534-561 行），开放段满了就 `self.devices[0].lowest_empty_segment()?`，而 `lowest_empty_segment`（同一文件第 329-344 行）把 `held_per_segment[*segment] == 0` 写进了 find 的条件。刚回收空的那一段正是被扣住的那一段：**回收腾出来的空间，抬 F 自己用不上**。池里没有别的全空段时 `allocate_commit_generated` 给 `None`，`publish_version` 变成 `PublishError::NoSpaceFor`（`transaction.rs` 第 1004 行），第 493 行的 `?` 走人。

**可达性。** 抬 F 的正常触发就是盘紧：`.claude/kb/decisions/16-发布语义.md` 第 377 行逐字写着

```text
| 准入 | 可分配 = min(可再分配 + 活元数据 − 保留池, `df`)；准入不够时先推空发布抬 F（写行那次发布之前不推：它是新实例的第一次发布，元数据走切换预留，同一次发布里的用户数据重做照走准入、不够返回 ENOSPC；2026-09-14 用户定案，C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方），一次准入最多 B = 4 + 2 k_tol 次发布（k_tol = 2 ⇒ 8），做满仍不够才报 ENOSPC |
```

也就是说，「池里已经没有别的全空段」不是我为了打中而挑的边角，**它是抬 F 这条路存在的理由**。`crates/` 今天没有接准入（C318（影子账隔离的单元没进准入不等式）），强制入口 `raise_rollback_floor` 是测试用的（`mount.rs` 第 392 行的函数级文档自陈），所以我在池级模型上量这一格。

**量到的（副本，`x8_hold_and_isolate_bits.rs` 第 59-98 行与第 191-218 行）。** 每盘单元区 128 槽 = 2 个聚簇段，段 0 全部仍分配、段 1 全部已释放（释放代 5）：

| 步 | 扣住（`HeldUntilFloorTakesEffect`） | 不扣住（`Immediately`） |
|---|---|---|
| 回收之前 `free_slots()` | 0 | 0 |
| 回收 64 槽之后 `free_slots()` | 64 | 64 |
| `lowest_empty_segment()` | `None` | 段 1 |
| `allocate_commit_generated(OneSlot)` | `None` | `Some(50240)` |
| 之后再试 3 次 | 3 次全是 `None`，`free_slots()` 一直报 64 | — |
| 手工 `release_reclaim_holds()` 之后 | `Some(50240)` | — |

两条路只差 `ReclaimedReuse` 一个参数，别的一模一样 —— 拦住它的就是扣住位。

**后果比「这一次抬 F 没做成」重**：扣住位留在分配器里，之后**每一次**发布都分配不到固定点，而记账（`free_slots`、`allocated_slots`、`deferred_slots`）已经按「回收成功」写好了。T3 之前的同一次失败是可恢复的：回收腾出的段立刻可用，下一次发布照样成功；T3 之后它变成这个进程的池锁死，而且记账与分配器对同一批槽说两句话。

**四句**：

| 问 | 答 |
|---|---|
| 分不分辨臂 | **分辨**。本地辩方要辩的那条替代臂「把回收整个推迟到生效之后」在这一格上不中：回收没发生，失败时什么都没动，扣住位也不存在。T3（记账先动、槽扣住）中 |
| 被判的系统当时看不看得到判别它的东西 | 看得到，而且是同一个对象的两个字段：`free_slots()` 与 `is_free()` 在同一张 `DeviceFreeMap` 上，同一个进程、同一刻，互相矛盾 |
| 满足的是判据字面的哪一个分句 | 背景材料第三节 X8 那一行的第二个分句：「或让抬 F 在扣住之后发不出固定点」。第一个分句（扣住的槽在生效之前被发出去）由 X8-B 部分满足 |
| 跑前条款给的每个改法在打中的那几格上还中不中 | 见下面「改法」表 |

**改法（只在我的模型上量过、被攻过零轮）**：

| 改法 | 修哪一格 | 在 X8-A 上还中不中 |
|---|---|---|
| α：失败路径上补 `release_reclaim_holds()`（把第 501 行挪成两处，或者用一个出作用域就放开的守卫） | 修「锁死」 | 不中了；但**记账那一半还中**：失败之后 `free_slots` 仍多算 64，而那些槽的记录还写着已释放、下一次挂载会重新算回已分配 |
| β：失败时把整个分配器退回抬 F 之前（`publish_version` 第 955-958 行已经有这一手：clone + 失败换回） | 修两半 | 不中。代价是多一次 450 KiB 的位图拷贝，`transaction.rs` 第 954 行的注释已经认过这个代价 |
| γ：把回收整个推迟到生效之后（辩方那条臂） | 修两半 | 不中；但它与 `mount.rs` 第 433-436 行写下的理由正面冲突（「一条根带的 F 与它的记账行要说同一件事」），要连 alloc-basis 那一轮的口径一起判 |

α 与 β 都不碰 X8-B。

### X8-B（机制量过、可达性没构造出来）：扣住位与隔离位对 bump 路零约束

**机制。** 三条发路径对这两个位的态度不一样：

| 发路径 | 看扣住位吗 | 看隔离位吗 | 在哪 |
|---|---|---|---|
| `lowest_user_data_slot`（用户数据） | 看（走 `is_free`） | 看 | `allocator.rs` 第 309-325 行调 `is_free`，`is_free` 在第 160-167 行 |
| `lowest_empty_segment`（开新段那一刻） | 看 | 看 | `allocator.rs` 第 329-344 行 |
| `allocate_commit_generated` 的开放段内 bump，与它调的 `PoolAllocator::record` | **不看** | **不看** | `allocator.rs` 第 534-561 行、第 463-490 行：bump 只比 `start + footprint.slots() > segment_end`，`record` 只调 `mark_allocated`，而 `mark_allocated`（第 285-305 行）只断言「跨度里有已分配的槽」，对隔离位与扣住位一个字都不说 |

也就是说，开放段里的槽由**开段那一刻**的三个条件（`used == 0 ∧ isolated == 0 ∧ held == 0`）一次性担保，开段之后再置的位对游标前方的槽没有任何效力。

**量到的（副本，`x8_hold_and_isolate_bits.rs` 第 100-145 行）。** 先发一个单槽单元开段（拿到 50176），再 `isolate_abandoned(dev, 50180, 1)`（两盘都隔离，两盘的 `is_free(50180)` 都变假），再连发 4 个单槽单元：拿到 `[50177, 50178, 50179, 50180]` —— **被隔离的 50180 照样发了出去**，`mark_allocated` 一声没吭。

**为什么这一条挂在这一轮的改法上。** T1 与 T3 之前，隔离只在 `rebuilt_allocator` 里做，那时分配器刚 `PoolAllocator::new` 出来、`open_segment` 恒 `None`（`allocator.rs` 第 379-390 行），开段一定在置位之后，上面那条担保是结构性的。T1 的「抬 F 之后按新 F 在回收之前再算一次」（`mount.rs` 第 449-464 行）与 T3 的扣住（第 465-468 行）**第一次把这两个位写进一个已经有开放段的活分配器**，担保从结构性变成了要靠一串推理才成立。

**可达性：我没构造出来，而且我给出的是我尝试推翻的那条推理。** 抬 F 那一刻新隔离的槽，是在这次挂载时被一条 txg 低于新 F 的候选根豁免过的槽。这样的槽在当前账里不可能是空闲的：

- 若它在当前账里已释放且**已回收**，那它不可能被任何候选根豁免 —— 引用它的根 txg < 释放代 ≤ 回收门槛 `max(F_生效, 环里最旧有效根)`，而候选根按定义 txg ≥ 环里最旧有效根，两条合起来矛盾（`mount.rs` 第 178-183 行的 `reclaim_floor` 与第 300-304 行的 `oldest_valid_root`）。
- 若它在当前账里没有记录，那引用它的根既不是当前版本的祖先也不是后代 —— 回退之后非被抛弃的根恰好只有 R_old 的祖先与新实例的后代，够不着这一格。
- 剩下的两种（仍分配、已释放未回收）都让那一段的 `used_per_segment > 0`，开段那一刻就被挡住了。

扣住位那一侧更紧：`mark_reclaimed`（第 242-261 行）断言整个跨度此前都已分配，而开放段里游标前方的槽按定义没分配过，所以被扣住的槽只可能落在游标后方。

⇒ 我把 X8-B 报成**打中了机制、没打中历史**。它今天不由代码挡着，由上面三条推理挡着；三条里任何一条被以后的改动碰掉（比如再加一处「发布中途隔离」的调用点、或者让 `lowest_empty_segment` 也认已回收未扣住的段），它就当场变成可达。判决要不要按打中办由主 agent 定。

**四句**：分不分辨臂 —— **不分辨**：G5、用户原措辞、保守读法三种读法在这一格上一样中，它是三条臂共用的前提（分配器对隔离位的射程），按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」那一节的第一种形态，该另立一笔账先修，不拿它判臂。被判的系统看不看得到判别它的东西 —— 看得到：`is_free(50180)` 为假与 `allocate_commit_generated` 返回 50180 在同一个 `PoolAllocator` 上同时成立。满足判据的哪一个分句 —— X8 那一行的第一个分句「一段历史让扣住的槽在生效之前被发出去」的**隔离位版本**，扣住位版本我没构造出来。跑前条款的每个改法 —— α / β / γ 三个都不碰它；能修它的是「bump 也查 `is_free`」或者「置位时若落在开放段内就当场关掉开放段」，两个我都只在脑子里过了一遍，没量。

## X1 T1 的隔离集：打中 1 条，形状级，不分辨臂

**打中的是豁免与隔离两侧的 key 口径不一致。** `crates/singlefs-core/src/mount.rs` 第 221-225 行与第 231-237 行建豁免集时，key 是 `(record.device.0, record.slot.0)` —— 只有**起点槽号**；第 246-255 行的隔离循环也按同一个 key 查表，查中就 `continue` 跳过**整条记录**；而真正铺位的 `DeviceFreeMap::isolate`（`crates/singlefs-core/src/allocator.rs` 第 200-212 行）是按 `span` 一格一格铺的：

```rust
        for record in records {
            let key = (record.device.0, record.slot.0);
            if record.is_released
                || referenced_by_candidates.contains(&key)
                || !isolated.insert(key)
            {
                continue;
            }
            allocator.isolate_abandoned(record.device, record.slot, u64::from(record.span_slots));
        }
```

⇒ 一条被抛弃根的记录 (槽 X, 跨度 2)，只要有哪条候选根（或当前账）在槽 X 上有一条**跨度 1** 的记录，整条就被跳过，**槽 X+1 一个字都没人罩**：候选根那条单槽记录盖不到 X+1，被抛弃根的双槽单元占着它。副本上量过这段集合算术（`x8_hold_and_isolate_bits.rs` 第 147-189 行，用与 `mount.rs` 同一套 key 口径重写一遍）：隔离集为空，而 `50191` 不在候选根盖到的槽里。

压着的条款，`.claude/kb/decisions/23-journal的角色与格式.md` 第 1209 行（那一行整行是回退那一整段，还包含候选集定义与回退发布语义；下面抄的是其中加粗的主句开头）：

```text
**被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头——分配器多查环里每一个可读根的账、只隔离其中
```

X+1 被发出去就是「重新分配」，主句在这一格为假。

**可达性：没构造出来。** 要让同一个起点槽号在两版账里跨度不同，得有一次「同槽换跨度的复用」跨过抛弃边界（`PoolAllocator::record` 第 463-478 行的复用分支会改写 `span_slots`），而复用要先回收、回收要 F 抬过释放代。我在 60 分钟里没把这段历史搭出来，所以只报形状。**同一条口径不一致还有一个反方向**（候选根的跨度 2 记录只贡献起点 X，被抛弃根在 X+1 上的单槽记录照样被隔离）——那一侧是多隔离，今天不咬人。

**四句**：分不分辨臂 —— **不分辨**，G5 / 用户原措辞 / 保守读法三种读法用的是同一段 key 算术，一起中；按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「打中不分辨臂」那一格，该另立一笔账先修，不拿它判 T1 选哪条读法。被判的系统看不看得到 —— 看得到：`span_slots` 就在同一条记录里，第 254 行已经在用它。满足判据的哪一个分句 —— X1 那一行的第一个分句「让一个只被被抛弃根引用的槽被 `is_free` 判空闲（豁免集算宽了）」。跑前条款的每个改法 —— 反向接受条款写的是「X1 打中 ⇒ 改代码、补一条会红的用例与一条变异行、再攻一轮」，改法是把豁免集与隔离集都按**跨度展开**成槽集合（`record.slot.0 .. record.slot.0 + span`）；只在我的模型上量过、被攻过零轮。

## X2 T1 的时机：没打中

试过的形状与它们各自倒在哪一步：

| 形状 | 结论 |
|---|---|
| 崩在两次带新 F 的空发布之间、重开，隔离集与崩之前差在哪 | 确实不同：重开时 `effective_floor` 回落到旧 F（`crates/singlefs-core/src/recovery.rs` 第 351-372 行按每盘 F 的最大值再取各盘最小值），候选集变大 ⇒ 抬 F 那一刻补隔离的那几个槽这次被豁免、不隔离。**但差的那几个槽发不出去**：重开从最新根那一版记录重建（`mount.rs` 第 278-279 行），它们的记录仍写着已释放、释放代 > 旧 F，`reclaim_released_up_to` 按旧 F 不回收它们（`allocator.rs` 第 572-577 行的 filter），于是回到「已分配 + defer」，`is_free` 为假 |
| 抬 F 时在内存里做的回收，崩了之后还算不算数 | 不算数，而且方向是安全的那一边：回收只动 `DeviceFreeMap` 与 `PoolAllocator::reclaimed`（都只住内存），盘上那条记录一个字没改；重开之后那些槽重新是「已释放、未回收」 |
| 回收过的槽在新账里会不会被 checker 判红 | 不会。checker 的 I-3.1（已分配统计对得上） 走的是「最新根 + 按最新根的实例表有效 + txg ≥ 最新根自己带的 F」这三条（`crates/singlefs-checker/src/walk.rs` 第 786-798 行取 `newest_rollback_floor`、第 831-843 行判 I-3.1）。崩在第一条带新 F 的根之后那一刻，最新根带的正是新 F，被回收的槽的引用者 txg < 释放代 ≤ 新 F，全部落在候选集外，记账说空闲、遍历也数不到它们 —— 两边说的是同一件事 |
| `raise_rollback_floor` 的 `oldest_valid_root` 按 `current` 的表判、重建按最新根的表判 | 第二轮本地攻方 1a 已经问过，这一轮不重复；我只核了一句：两处之间没有别的发布，`current` 的实例表就是最新根指着的那一份 |
| 再抬一次 F 会不会重复回收 | 不会：`PoolAllocator::reclaimed`（`allocator.rs` 第 372 行、第 580 行的 `if !self.reclaimed.insert(key) { continue; }`）挡着；但它只住内存，跨进程不挡 —— 跨进程那一侧由「记录仍写着已释放」兜住，`mark_reclaimed` 第 245-248 行的断言会挡住二次回收 |

顺带留一条给判决用的正面结论（我反复用到它，也想不出反例）：**一个已被回收的槽不可能被任何候选根豁免**，因为引用它的根 txg < 释放代 ≤ `max(F_生效, 环里最旧有效根)`，而候选根按 `readable_roots` 取最小值的定义 txg ≥ 环里最旧有效根。T1 的豁免集与 T3 的回收门槛因此不会打架。

## X3 T2 跳过的根：打中 1 条（改法只落实了一半）

**打中：抬 F 那一处的跳过不计数。** T2 的形态是「跳过那条根、计数、不拒绝挂载」，而 `isolate_slots_referenced_only_by_abandoned_roots` 有两个调用点，只有一个把计数接走了：

- `rebuilt_allocator`（`crates/singlefs-core/src/mount.rs` 第 309-319 行）把返回值绑给 `abandoned_roots_unreadable`，一路带到 `MountOutput`。
- `raise_rollback_floor`（同一文件第 456-463 行）**把返回值当语句丢掉**：`isolate_slots_referenced_only_by_abandoned_roots(devices, allocator, &roots, &|root| abandoned_by_table(root, &table), new_floor, &current.allocation_records);`。返回类型 `u64` 没有 `#[must_use]`，编译器一声不吭。

⇒ 抬 F 是**候选集缩小、隔离集变大**的那一刻，正是最需要知道「有几条被抛弃根罩不到」的时候；今天这一刻读不出账的根被静默跳过，进程里没有任何地方留痕。`MountOutput::abandoned_roots_unreadable` 又只有用例在读（背景材料第一节末尾自陈），所以这一格现在是「一个只在一半调用点上写、且没有人读」的计数器。

**同族的第三类，条款的措辞罩不到。** 一条被抛弃根的**根槽**自己读不出时，它根本不进 `readable_roots`（`crates/singlefs-core/src/recovery.rs` 第 331-347 行只收自证通过的），既不隔离也不计数 —— 它落在「读不出账的被抛弃根」这个说法之外。今天不咬人（`choose_root` 与回退目标都只认 `readable_roots` 里的根），但这个数以后要进准入或不变量时，它的定义域得先写清楚是「账读不出」还是「罩不到」。

**核过、没打中的三问：**

| 问 | 结论 |
|---|---|
| 被跳过的那条根后来成了最新根，`choose_root` 会不会选中它 | 会。`choose_root`（`recovery.rs` 第 287-303 行）在全部自证通过的根里取 `(checkpoint_txg, 实例代号)` 最大的，树表坏不坏它不看；`recover`（同一文件第 1228-1253 行）**没有回退到次新根这一步**，`walk_to_file` 在那条根上失败就整轮失败。这是交用户那张表第 4 行已经写下的代价（「跳过 ⇒ 那条根引用的槽可能被复用，退到它时走读失败」），不是新的一格 |
| 候选集按实例表判它被抛弃、退回实例 1 旧根时它的记录会不会被接上 | 第二轮 X1-A 已判过这一格（C340（回退之后记录链从哪条之后接没有定义） 的 P1 / P2），这一轮不重复 |
| 候选根的账读不出「当它什么都不豁免」方向对不对 | 方向对，但在今天可达的历史里它是**空操作**：候选根引用的槽在当前账里要么仍分配、要么已释放未回收（理由见 X2 那一节末尾那条正面结论），两种 `is_free` 都已经是假，多标一个隔离位不改变任何一次分配。⚠️ 它改变的是 `isolated_slots()` 报出来的数 —— 那个数是 D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」的替身（`.claude/kb/decisions/28-挂载期承诺量.md` 第 30 行定义为「被抛弃根的已分配统计量 − R_old 的已分配统计量」）。候选根的账一读不出，它就会超过那个定义算出来的数。这一条挨着 X4，按分工不由我判，只记在这里 |

## 没打中的形状

| 试过的形状 | 取样范围 | 倒在哪 |
|---|---|---|
| 抬 F 的发布循环因为 `ROOT_RING_REGIONS` 上限退出而没落满每块盘，`release_reclaim_holds()` 照样放开 | 逐个 txg 起点算过 3 连发的区域号 | 不可达。`ROOT_RING_REGIONS = 3`（`crates/singlefs-format/src/lib.rs` 第 185 行）、`ROOT_RING_REGION_DEVICES = [0, 1, 0]`（同一文件第 192 行）、`target_for_publish` 取 `txg % 3`（`crates/singlefs-core/src/root_ring.rs` 第 32-37 行）：任何 3 个连续 txg 都覆盖区域 0 / 1 / 2，也就一定覆盖两块盘，而 `mount.rs` 第 472-476 行的循环正好允许 3 次。上限与需要的次数**恰好相等**，没有余量 —— 区域数或区域到盘的映射一改，这一格当场可达 |
| 扣住的槽落在开放段游标**前方**被 bump 发出去 | 逐条走 `mark_reclaimed` 第 245-248 行的断言 | 不可达：被扣住的槽此前必须是已分配的，而开放段里游标前方的槽按开段条件没分配过 |
| 抬 F 的空发布把刚回收的槽当固定点发出去 | 副本上跑过 `ReclaimedReuse::Immediately` 的对照 | 这是 alloc-basis 第二轮已经打中、T3 正是它的修法；对照组确认扣住位确实挡住了它（X8-A 那张表第 4 行） |
| 崩在两次带新 F 的空发布之间让被回收的槽被发出去 | 见 X2 那张表 | 不可达，记录仍写着已释放、按旧 F 不回收 |
| 最新根的实例表读不出 ⇒ `newest_table = None` ⇒ 一条根都不算被抛弃 ⇒ 影子账一格不隔离 | 读了 `mount.rs` 第 292-299 行与第 657-658 行、第 732-733 行 | 不可达成「静默」：`mount_writable` 在第 657-658 行拿同一份字节 `InstanceTableRecords::parse` 失败会报 `InstanceTableMalformed`，`mount_rollback` 在第 732-733 行更早就报；两条路都不会带着空的被抛弃集合往下走 |
| 豁免集漏掉当前账里**已释放**的记录（第 223 行与第 234 行的 `!record.is_released` 过滤） | 逐条对 D16（发布语义） 已定项 1 第 371 行的可再分配谓词 | 不是洞：已释放的落点仍占着槽（`mark_released` 不动位图），本来就发不出去 |
| 同一个槽被两条被抛弃根用不同跨度引用，`isolated.insert(key)` 去重之后只按第一条的跨度铺位 | `mount.rs` 第 250 行 | 与 X1 那条同源（key 不带跨度），并进 X1 一起报，不另算一条 |

## 这条腿自己的限度

1. **X8-A 的数是在池级模型上量的，不是在装置上量的。** 每盘 128 槽、两个聚簇段是我为了让「池里只剩一个全空段」这件事发生而挑的几何；`crates/singlefs-harness` 的用例跑的是 4 GiB 镜像、每盘 3312 段，抬 F 在那上面不会失败。我没在文件镜像上端到端跑一次失败的抬 F ——那要么把镜像缩到单元区只剩两三段（`UNIT_AREA_START_SLOT = 50176`，镜像至少 787 MiB），要么把分配记录堆到一个节点装不下（`transaction.rs` 第 942-952 行的 `AllocationRecordsExceedOneNode`，但那条路会让**之后每一次**发布都失败，分不出是扣住位还是记录树满了）。副本上的数不进 kb。
2. **X8-B 与 X1 都只量了机制，没量历史。** 两条我都写明了「可达性没构造出来」，并且把我自己用来挡住它们的那串推理逐条写出来了 —— 判它们算不算打中由主 agent 定。X1 那条要的历史（同槽换跨度的复用跨过抛弃边界）我估计构造得出来，只是这一轮时间不够。
3. **我没跑 `crates` 的既有用例，也没跑门禁。** 副本上只跑了自己那个测试文件（`cargo test -p singlefs-core --test x8_hold_and_isolate_bits`），没跑 `check.sh`、没跑层 0、没跑变异表；原仓一个已有文件都没改，git 的写操作一次都没做。
4. **我没碰 X4、X5**（本地攻方的格），也没判正推腿的 X6 / X7 与 F_生效 回落那个方向。X3 那一节末尾提到 `isolated_slots()` 会超过 D28（挂载期承诺量） 已定项 1 第九项的定义 —— 那是 X4 的题面，我只记不判。
5. **前两轮攻过的角度我没重复**：读过 `m2-step45-code-r1-opus-output.md`、`m2-step45-code-r2-opus-output.md`、`alloc-basis-r2-opus-output.md` 与两轮判决，第二轮的 X1-A / X1-B、X2-A / X2-B、X3-A 到 X3-D 与 alloc-basis 第二轮 2.1 节那一格都没再攻；X3 那一节引到它们的地方只写「已判过、不重复」。
6. **一条腿的「没打中」一次不算。** X2 那一格我报的是没打中，按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」，它需要另一次独立的观测才立得住；打中的那几条是线索，真伪由主 agent 现查坐实。

## 什么现象会推翻这份报告的结论

| 结论 | 推翻它的观测 |
|---|---|
| X8-A：抬 F 的发布失败会让扣住位永远留着 | 在 `mount.rs` 第 397-507 行里找到第二处清扣住位的地方，或者证明 `publish_version` 在抬 F 的空发布上不可能返回 Err |
| X8-A：这一格只有 T3 中、辩方那条替代臂不中 | 证明「把回收整个推迟到生效之后」也会在失败路径上留下没清的状态 |
| X8-B：bump 路不看隔离位与扣住位 | 副本上那条用例转绿变红（即 `allocate_commit_generated` 不再返回 50180） |
| X1：起点槽号做 key 会漏掉跨度里的第二槽 | 找到别处有一道检查按跨度展开地核过隔离集，或者证明两版账里同一个起点槽号的跨度恒相等 |
| X2：崩在两次空发布之间不会让被回收的槽被发出去 | 一段历史让那些槽的记录在重开之后不是「已释放、释放代 > 旧 F」 |
| X3：抬 F 那一处跳过不计数 | `mount.rs` 第 456-463 行出现绑定返回值的写法 |
