
// ---- m3-batch27-code-r1 攻方（Opus）原型：只追加进仓副本里的 checker_known_bad_images.rs，不进入库代码 ----
#[cfg(test)]
mod opus_b27_attack {
    use super::*;

    fn verdict_lists(verdicts: &[(&'static str, InvariantVerdict)]) -> (Vec<&'static str>, Vec<&'static str>) {
        let red = verdicts
            .iter()
            .filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_)))
            .map(|(n, _)| *n)
            .collect();
        let partial = verdicts
            .iter()
            .filter(|(_, v)| matches!(v, InvariantVerdict::PartiallyJudged(_)))
            .map(|(n, _)| *n)
            .collect();
        (red, partial)
    }

    fn kind_of(verdicts: &[(&'static str, InvariantVerdict)], name: &str) -> &'static str {
        match &verdicts.iter().find(|(n, _)| *n == name).expect("清单里有").1 {
            InvariantVerdict::Holds => "H",
            InvariantVerdict::Violated(_) => "V",
            InvariantVerdict::PartiallyJudged(_) => "P",
            InvariantVerdict::NotApplicable(_) => "N",
        }
    }

    // ---------------- J3：inode 树根条目类型段 × 孩子故障 × 条目里指针故障 × 根层级 ----------------

    #[derive(Clone, Copy, Debug)]
    enum ChildFault {
        None,
        ContentBoth,
        ContentDev0,
        ContentDev1,
        Class9Carried,
        Class2Carried,
        Class1Carried,
        StaleNodeDev0,
        StaleNodeDev1,
    }
    const CHILD_FAULTS: [ChildFault; 9] = [
        ChildFault::None,
        ChildFault::ContentBoth,
        ChildFault::ContentDev0,
        ChildFault::ContentDev1,
        ChildFault::Class9Carried,
        ChildFault::Class2Carried,
        ChildFault::Class1Carried,
        ChildFault::StaleNodeDev0,
        ChildFault::StaleNodeDev1,
    ];

    #[derive(Clone, Copy, Debug)]
    enum EntryFault {
        None,
        FlipSum0,
        FlipSum1,
        FlipBoth,
        RedirectToHole,
        RedirectToNodeWithItsSum,
        RedirectToNodeKeepSum,
    }
    const ENTRY_FAULTS: [EntryFault; 7] = [
        EntryFault::None,
        EntryFault::FlipSum0,
        EntryFault::FlipSum1,
        EntryFault::FlipBoth,
        EntryFault::RedirectToHole,
        EntryFault::RedirectToNodeWithItsSum,
        EntryFault::RedirectToNodeKeepSum,
    ];
    const ENTRY_TYPES: [u16; 5] = [2, 0, 7, 1, 0xffff];
    const ROOT_LEVELS: [Option<u8>; 2] = [None, Some(2)];
    /// 条目里子指针两条位置条目：(盘 4、槽 6、整单元校验和 4) 在 120 字节条目里的起点。
    const ENTRY_LOCATION_STARTS: [usize; 2] = [34 + 50, 34 + 64];
    /// 两块 4 GiB 盘（262 144 槽）上流里没写过的一截。
    const HOLE_SLOT: u64 = 250_000;

    fn bases() -> Vec<(&'static str, MemoryPool)> {
        vec![
            ("B1-new-pool-file", build_pool("opus-b27-b1").memory_pool()),
            (
                "B2-rolled-back-to-first-root",
                image_after_rolling_back_to_the_first_root("opus-b27-b2").image,
            ),
            (
                "B3-sequential-write-3",
                common_pointer_head_corruption::image_after_a_sequential_write_of("opus-b27-b3", 3),
            ),
            (
                "B4-abandoned-third-root",
                image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth(
                    "opus-b27-b4",
                ),
            ),
        ]
    }

    fn apply_child_fault(image: &mut MemoryPool, fault: ChildFault) {
        let child = child_pointer_of_an_entry_of_the_root_of_kind(image, TREE_KIND_INODE, 0);
        let slot = common_pointer_head_corruption::the_slot_of(&child);
        let length = 32768usize;
        let flip = |image: &mut MemoryPool, device: u32| {
            let mut bytes = read(image, device, slot.0 * SLOT, length);
            bytes[length - 100] ^= 0xff;
            write(image, device, slot.0 * SLOT, &bytes);
        };
        let carried = |image: &mut MemoryPool, class: u8| {
            let mut bytes = read(image, 0, slot.0 * SLOT, length);
            assert_eq!(bytes[6], 3, "孩子是码 3 叶容器");
            bytes[6] = class;
            common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum(
                image, slot, &bytes,
            );
        };
        let stale = |image: &mut MemoryPool, device: u32| {
            let node = root_node_bytes_of_the_tree_of_kind(image, TREE_KIND_INODE);
            write(image, device, slot.0 * SLOT, &node);
        };
        match fault {
            ChildFault::None => {}
            ChildFault::ContentBoth => {
                flip(image, 0);
                flip(image, 1);
            }
            ChildFault::ContentDev0 => flip(image, 0),
            ChildFault::ContentDev1 => flip(image, 1),
            ChildFault::Class9Carried => carried(image, 9),
            ChildFault::Class2Carried => carried(image, 2),
            ChildFault::Class1Carried => carried(image, 1),
            ChildFault::StaleNodeDev0 => stale(image, 0),
            ChildFault::StaleNodeDev1 => stale(image, 1),
        }
    }

    fn change_the_inode_root(
        image: &mut MemoryPool,
        level: Option<u8>,
        change: impl FnOnce(&mut [u8]),
    ) {
        let root_pointer =
            common_pointer_head_corruption::tree_table_entry_of_kind(image, TREE_KIND_INODE).root;
        let root_slot = common_pointer_head_corruption::the_slot_of(&root_pointer);
        common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
            image,
            root_slot,
            common_pointer_head_corruption::node_bytes(),
            |root| {
                if let Some(level) = level {
                    root[INDEX_NODE_LEVEL_OFFSET] = level;
                }
                let start = 115 + 2 * usize::from(root[INDEX_NODE_KEY_WIDTH_OFFSET]);
                change(&mut root[start..start + INTERNAL_ENTRY_BYTES]);
            },
        );
    }

    fn apply_entry_change(
        image: &mut MemoryPool,
        entry_type: u16,
        fault: EntryFault,
        level: Option<u8>,
    ) {
        // 改指的码 2 节点取分配记录树的根：它的内容里没有 inode 树根的整单元 CRC（取树表或映射树的节点时，补校验和沿引用链补成一个环、补不完）。
        let node_target = {
            let root = common_pointer_head_corruption::tree_table_entry_of_kind(image, TREE_KIND_ALLOCATION).root;
            let slot = common_pointer_head_corruption::the_slot_of(&root);
            let bytes = common_pointer_head_corruption::unit_bytes_at(
                image,
                slot,
                common_pointer_head_corruption::node_bytes(),
            );
            (slot.0, crc32_castagnoli(&bytes))
        };
        change_the_inode_root(image, level, |entry| {
            set_u16(entry, CONTAINER_TREE_ENTRY_TYPE_OFFSET, entry_type);
            let set_slot = |entry: &mut [u8], slot: u64| {
                for start in ENTRY_LOCATION_STARTS {
                    entry[start + 4..start + 10].copy_from_slice(&slot.to_le_bytes()[..6]);
                }
            };
            match fault {
                EntryFault::None => {}
                EntryFault::FlipSum0 => entry[ENTRY_LOCATION_STARTS[0] + 10] ^= 0x5a,
                EntryFault::FlipSum1 => entry[ENTRY_LOCATION_STARTS[1] + 10] ^= 0x5a,
                EntryFault::FlipBoth => {
                    entry[ENTRY_LOCATION_STARTS[0] + 10] ^= 0x5a;
                    entry[ENTRY_LOCATION_STARTS[1] + 10] ^= 0x5a;
                }
                EntryFault::RedirectToHole => set_slot(entry, HOLE_SLOT),
                EntryFault::RedirectToNodeWithItsSum => {
                    set_slot(entry, node_target.0);
                    for start in ENTRY_LOCATION_STARTS {
                        entry[start + 10..start + 14].copy_from_slice(&node_target.1.to_le_bytes());
                    }
                }
                EntryFault::RedirectToNodeKeepSum => set_slot(entry, node_target.0),
            }
        });
    }

    /// I-2.1 字面（推的、只看这一条指针）：每条位置条目读头里的类标签；登记过的类按那一类的长度读回、整单元 CRC 与位置条目对不上 ⇒ V；
    /// 类标签不登记（长度认不出）时，16 KiB 与 32 KiB 两种长度都对不上 ⇒ V（按哪种长度都违反），有一种对得上 ⇒ ?；全都对得上 ⇒ H。
    /// 类标签改成 2、整单元 CRC 按 32 KiB 补的那一形（Class2Carried）按头里的类读 16 KiB、记 V：哪一个长度算「内容」有两读，报告里单列、不当打中。
    fn literal_i21_of_the_inode_child(image: &MemoryPool) -> &'static str {
        let child = child_pointer_of_an_entry_of_the_root_of_kind(image, TREE_KIND_INODE, 0);
        let mut unknown = false;
        for location in child.locations {
            let header = read(image, location.device.0, location.slot.0 * SLOT, 16384);
            let length = match header[6] {
                1 | 3 => 32768,
                2 => 16384,
                _ => {
                    let matches_at_some_length = [16384usize, 32768].iter().any(|length| {
                        crc32_castagnoli(&read(image, location.device.0, location.slot.0 * SLOT, *length))
                            == location.unit_checksum
                    });
                    if matches_at_some_length {
                        unknown = true;
                        continue;
                    }
                    return "V";
                }
            };
            let content = read(image, location.device.0, location.slot.0 * SLOT, length);
            if crc32_castagnoli(&content) != location.unit_checksum {
                return "V";
            }
        }
        if unknown {
            "?"
        } else {
            "H"
        }
    }

    #[test]
    fn opus_b27_j3_matrix() {
        // 取样：OPUS_B27_J3_FROM..OPUS_B27_J3_TO（形的下标，左闭右开）；类型段默认取 2、0、7，OPUS_B27_J3_TYPES 可另给（1、65535 那一批另跑，见报告）。
        let from: usize = std::env::var("OPUS_B27_J3_FROM").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        let to: usize = std::env::var("OPUS_B27_J3_TO").ok().and_then(|v| v.parse().ok()).unwrap_or(usize::MAX);
        let types: Vec<u16> = std::env::var("OPUS_B27_J3_TYPES")
            .ok()
            .map(|v| v.split(",").map(|t| t.parse().expect("类型段")).collect())
            .unwrap_or_else(|| vec![2, 0, 7]);
        let base_names = [
            "B1-new-pool-file",
            "B2-rolled-back-to-first-root",
            "B3-sequential-write-3",
            "B4-abandoned-third-root",
        ];
        let mut shapes = Vec::new();
        for base_index in 0..base_names.len() {
            for child_fault in CHILD_FAULTS {
                for entry_fault in ENTRY_FAULTS {
                    for entry_type in types.iter().copied() {
                        for level in ROOT_LEVELS {
                            shapes.push((base_index, child_fault, entry_fault, entry_type, level));
                        }
                    }
                }
            }
        }
        let to = to.min(shapes.len());
        println!("J3-PLAN total_shapes={} from={from} to={to}", shapes.len());
        let started = std::time::Instant::now();
        // 每个起点状态在这个进程里只建一次池，之后每一形从内存里那一份拷。
        let mut built: Vec<Option<MemoryPool>> = vec![None, None, None, None];
        for index in from..to {
            let base_index = shapes[index].0;
            if built[base_index].is_none() {
                built[base_index] = Some(match base_index {
                    0 => build_pool("opus-b27-b1").memory_pool(),
                    1 => image_after_rolling_back_to_the_first_root("opus-b27-b2").image,
                    2 => common_pointer_head_corruption::image_after_a_sequential_write_of("opus-b27-b3", 3),
                    _ => image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth(
                        "opus-b27-b4",
                    ),
                });
            }
        }
        println!("J3-BASES-BUILT seconds={:.1}", started.elapsed().as_secs_f64());
        let threads: usize = std::env::var("OPUS_B27_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(1).max(1);
        let lines: std::sync::Mutex<Vec<(usize, String)>> = std::sync::Mutex::new(Vec::new());
        std::thread::scope(|scope| {
            for thread in 0..threads {
                let shapes = &shapes;
                let built = &built;
                let lines = &lines;
                scope.spawn(move || {
                    for index in (from..to).filter(|index| index % threads == thread) {
            let (base_index, child_fault, entry_fault, entry_type, level) = shapes[index];
            let base_name = base_names[base_index];
            let mut image = built[base_index].clone().expect("建过");
            apply_child_fault(&mut image, child_fault);
            apply_entry_change(&mut image, entry_type, entry_fault, level);
            let literal = literal_i21_of_the_inode_child(&image);
            let verdicts = check_pool_image(&image);
            let (red, partial) = verdict_lists(&verdicts);
            let line = format!(
                "J3\tb={base_name}\tc={child_fault:?}\te={entry_fault:?}\tt={entry_type}\tl={}\tlit21={literal}\ti21={}\ti48={}\ti72={}\ti74={}\ti31={}\ti51={}\tred={}\tpartial={}",
                level.map_or(1, |l| l),
                kind_of(&verdicts, "I-2.1"),
                kind_of(&verdicts, "I-4.8"),
                kind_of(&verdicts, "I-7.2"),
                kind_of(&verdicts, "I-7.4"),
                kind_of(&verdicts, "I-3.1"),
                kind_of(&verdicts, "I-5.1"),
                red.join(","),
                partial.join(",")
            );
            let line = if std::env::var("OPUS_B27_J3_DETAIL").is_ok() {
                let details: Vec<String> = verdicts
                    .iter()
                    .filter_map(|(name, v)| match v {
                        InvariantVerdict::Violated(detail) => Some(format!("{name}: {detail}")),
                        InvariantVerdict::PartiallyJudged(detail) => Some(format!("{name}（部分没判）: {detail}")),
                        _ => None,
                    })
                    .collect();
                format!("{line}\n  DETAIL {}", details.join("\n  DETAIL "))
            } else {
                line
            };
            lines.lock().expect("锁").push((index, line));
                    }
                });
            }
        });
        let mut lines = lines.into_inner().expect("锁");
        lines.sort();
        for (_, line) in &lines {
            println!("{line}");
        }
        println!(
            "J3-DONE shapes={} seconds={:.1}",
            to.saturating_sub(from),
            started.elapsed().as_secs_f64()
        );
    }

    // ---------------- J1：最新根的实例表链（CPU 实跑 + GPU 走读循环的逐行转写） ----------------

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum PageState {
        Intact,
        /// 两份各翻一个载荷字节、不重封（指着它的链指针记录按翻之前的整单元 CRC）。
        Unreadable,
        /// 容器号写成别的片号（身份四元组不对）。
        ContainerNumberWrong,
        /// 链指针记录「有无下一片」写 2（不合法），这一片就是链上最后一片。
        ChainRecordMalformed,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ChainEnd {
        /// 末片的链指针记录说没有下一片。
        LastPage,
        /// 末片的链指针记录指回第 j 片的槽（整单元 CRC 写成 j 片的 CRC 加一：环上 CRC 自洽造不出来）。
        BackTo(usize),
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum RowsOfTheAbandonedInstance {
        /// 实例 2 那一行去掉：门开就红 I-9.29，门关报部分没判。
        Dropped,
        /// 实例 2 那一行放在末片：门开滤得掉、I-9.29 成立。
        OnTheLastPage,
    }

    const FIRST_SLOT_OF_THE_PAGES_AFTER_THE_FIRST: u64 = 200_000;
    const ROOT_INSTANCE_TABLE_POINTER: usize = 170;

    fn newest_root_record_bytes(image: &MemoryPool) -> Vec<u8> {
        root_slots()
            .into_iter()
            .map(|(device, offset)| read(image, device, offset, 512))
            .filter(|root| &root[..4] == b"SFSR")
            .max_by_key(|root| {
                (
                    get_u64(root, ROOT_RECORD_CHECKPOINT_TXG_OFFSET),
                    get_u32(root, ROOT_RECORD_INSTANCE_OFFSET),
                )
            })
            .expect("根环里有根")
    }

    fn records_of_a_page(bytes: &[u8]) -> Vec<Vec<u8>> {
        let count = usize::from(u16::from_le_bytes([
            bytes[PACKED_UNIT_RECORD_COUNT_OFFSET],
            bytes[PACKED_UNIT_RECORD_COUNT_OFFSET + 1],
        ]));
        (0..count)
            .map(|i| {
                let start = INSTANCE_TABLE_FIRST_ROW_START + i * INSTANCE_TABLE_RECORD_BYTES;
                bytes[start..start + INSTANCE_TABLE_RECORD_BYTES].to_vec()
            })
            .collect()
    }

    fn replace_records(bytes: &mut [u8], records: &[Vec<u8>]) {
        bytes[INSTANCE_TABLE_FIRST_ROW_START..].fill(0);
        for (i, record) in records.iter().enumerate() {
            let start = INSTANCE_TABLE_FIRST_ROW_START + i * INSTANCE_TABLE_RECORD_BYTES;
            bytes[start..start + INSTANCE_TABLE_RECORD_BYTES].copy_from_slice(record);
        }
        set_u16(
            bytes,
            PACKED_UNIT_RECORD_COUNT_OFFSET,
            u16::try_from(records.len()).expect("装得下"),
        );
        set_u16(
            bytes,
            PACKED_UNIT_DECLARED_LENGTH_OFFSET,
            u16::try_from(records.len() * INSTANCE_TABLE_RECORD_BYTES).expect("装得下"),
        );
    }

    fn chain_record(next: Option<(&[u8], u64, u32)>, malformed: bool) -> Vec<u8> {
        let mut record = vec![0u8; INSTANCE_TABLE_RECORD_BYTES];
        record[0] = INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD;
        if let Some((template, slot, checksum)) = next {
            let mut pointer = template.to_vec();
            for location in [50usize, 64] {
                pointer[location + 4..location + 10].copy_from_slice(&slot.to_le_bytes()[..6]);
                pointer[location + 10..location + 14].copy_from_slice(&checksum.to_le_bytes());
            }
            record[CHAIN_RECORD_HAS_NEXT_PAGE_OFFSET] = 1;
            record[2..2 + 86].copy_from_slice(&pointer);
        }
        if malformed {
            record[CHAIN_RECORD_HAS_NEXT_PAGE_OFFSET] = 2;
        }
        record
    }

    fn slot_of_page(index: usize, first_page_slot: u64) -> u64 {
        if index == 0 {
            first_page_slot
        } else {
            FIRST_SLOT_OF_THE_PAGES_AFTER_THE_FIRST + 2 * u64::try_from(index).expect("片号")
        }
    }

    /// 在 `base`（被崩溃恢复抛弃的 C 与现行线同号不同出生代的那一份）上把最新根的实例表改成 `states.len()` 片的链。
    fn image_with_a_chain(
        base: &MemoryPool,
        states: &[PageState],
        end: ChainEnd,
        rows: RowsOfTheAbandonedInstance,
    ) -> MemoryPool {
        let mut image = base.clone();
        let first_page_slot = instance_table_slot_of_the_newest_root(&image);
        let root = newest_root_record_bytes(&image);
        let template = root[ROOT_INSTANCE_TABLE_POINTER..ROOT_INSTANCE_TABLE_POINTER + 86].to_vec();
        let first_page = read(&image, 0, first_page_slot * SLOT, INSTANCE_TABLE_UNIT_BYTES);
        let mut records = records_of_a_page(&first_page);
        let chain = records.pop().expect("链指针记录");
        assert_eq!(chain, chain_record(None, false), "单片、末片");
        let (abandoned, others): (Vec<Vec<u8>>, Vec<Vec<u8>>) = records
            .into_iter()
            .partition(|row| get_u32(row, 1) == 2);
        assert_eq!(abandoned.len(), 1, "实例 2 那一行恰好一行");
        let pages = states.len();
        let mut rows_of_each_page: Vec<Vec<Vec<u8>>> = vec![Vec::new(); pages];
        rows_of_each_page[0] = others;
        if rows == RowsOfTheAbandonedInstance::OnTheLastPage {
            rows_of_each_page[pages - 1].extend(abandoned);
        }
        // 先算各片（末片往前），环回的那条按目标片的 CRC 加一。
        let mut built: Vec<Option<Vec<u8>>> = vec![None; pages];
        let mut crc_of: Vec<u32> = vec![0; pages];
        let mut next: Option<(u64, u32)> = None;
        for index in (0..pages).rev() {
            let mut records = rows_of_each_page[index].clone();
            let malformed = states[index] == PageState::ChainRecordMalformed;
            let next_pointer = if index == pages - 1 {
                match end {
                    ChainEnd::LastPage => None,
                    ChainEnd::BackTo(j) => Some((slot_of_page(j, first_page_slot), 0xdead_beef_u32)),
                }
            } else if malformed {
                None
            } else {
                next
            };
            records.push(chain_record(
                next_pointer.map(|(slot, sum)| (template.as_slice(), slot, sum)),
                malformed,
            ));
            if index == 0 {
                // 第 0 片就地改、重封、补到根（mutate_unit_of 做）
                built[0] = Some(records.concat());
                let _ = &mut crc_of;
                break;
            }
            let mut page = first_page.clone();
            replace_records(&mut page, &records);
            let container = if states[index] == PageState::ContainerNumberWrong {
                u64::try_from(index).expect("片号") + 1000
            } else {
                u64::try_from(index).expect("片号")
            };
            set_u64(&mut page, PACKED_UNIT_CONTAINER_NUMBER_OFFSET, container);
            reseal_unit(&mut page);
            let crc = crc32_castagnoli(&page);
            crc_of[index] = crc;
            if states[index] == PageState::Unreadable {
                page[INSTANCE_TABLE_UNIT_BYTES - 100] ^= 0xff;
            }
            let slot = slot_of_page(index, first_page_slot);
            for device in DEVICES {
                write(&mut image, device, slot * SLOT, &page);
            }
            next = Some((slot, crc));
            built[index] = Some(page);
        }
        let first_records: Vec<Vec<u8>> = {
            let flat = built[0].take().expect("第 0 片");
            flat.chunks(INSTANCE_TABLE_RECORD_BYTES).map(<[u8]>::to_vec).collect()
        };
        let first_state = states[0];
        let units = units_found_on_device_zero(&image);
        mutate_unit_of(
            &mut image,
            &units,
            first_page_slot,
            |bytes| {
                replace_records(bytes, &first_records);
                if first_state == PageState::ContainerNumberWrong {
                    set_u64(bytes, PACKED_UNIT_CONTAINER_NUMBER_OFFSET, 1000);
                }
            },
            true,
        );
        if first_state == PageState::Unreadable {
            break_both_copies_of_the_unit(&mut image, first_page_slot, INSTANCE_TABLE_UNIT_BYTES);
        }
        image
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ChainOutcome {
        /// 走到末片（门开）。
        ReachedTheLastPage,
        /// 提前出口（门关），带停下时的片号。
        StoppedAt(usize),
        /// GPU：走满上限链还没完，记判不了（站点 61）。
        UndecidableAfterThePagesWalked,
    }

    /// `crash_judge_checker.wgsl` 的 w_walk_instance_table_chain 循环逐行转写到这里造的链上（`cap` 为 None 时是 walk.rs 的 walk_instance_table_chain：不设上限）。
    /// 读不出 → break；走过的单元（这里只会是环回的那一片）读不出先于 visited 判（CRC 写错）；身份不对 → break；链指针记录 0 → 末片、break；
    /// 2 → break；1 → 下一片。
    fn transcribed_walk(states: &[PageState], end: ChainEnd, cap: Option<usize>) -> ChainOutcome {
        let mut page = 0usize;
        let mut current = 0usize;
        let mut visited: Vec<usize> = Vec::new();
        let mut back_pointer_followed = false;
        loop {
            if let Some(cap) = cap {
                if page >= cap {
                    return ChainOutcome::UndecidableAfterThePagesWalked;
                }
            }
            if back_pointer_followed {
                // 环回那条链指针的整单元 CRC 是错的：读不出，先于 visited 判
                return ChainOutcome::StoppedAt(page);
            }
            match states[current] {
                PageState::Unreadable | PageState::ContainerNumberWrong => {
                    return ChainOutcome::StoppedAt(page)
                }
                PageState::ChainRecordMalformed => return ChainOutcome::StoppedAt(page),
                PageState::Intact => {}
            }
            assert!(!visited.contains(&current));
            visited.push(current);
            if current == states.len() - 1 {
                match end {
                    ChainEnd::LastPage => return ChainOutcome::ReachedTheLastPage,
                    ChainEnd::BackTo(j) => {
                        current = j;
                        back_pointer_followed = true;
                    }
                }
            } else {
                current += 1;
            }
            page += 1;
        }
    }

    #[test]
    fn opus_b27_j1_chains() {
        let base =
            image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth(
                "opus-b27-j1",
            );
        println!("J1-PLAN");
        let started = std::time::Instant::now();
        let mut shapes: Vec<(String, Vec<PageState>, ChainEnd, RowsOfTheAbandonedInstance)> = Vec::new();
        for rows in [RowsOfTheAbandonedInstance::Dropped, RowsOfTheAbandonedInstance::OnTheLastPage] {
            for n in [1usize, 2, 255, 256, 257, 258, 300] {
                shapes.push((format!("len{n}"), vec![PageState::Intact; n], ChainEnd::LastPage, rows));
            }
            for (n, k) in [(256usize, 255usize), (257, 255), (257, 256), (257, 1), (258, 256), (2, 1)] {
                for state in [
                    PageState::Unreadable,
                    PageState::ContainerNumberWrong,
                    PageState::ChainRecordMalformed,
                ] {
                    let mut states = vec![PageState::Intact; n];
                    states[k] = state;
                    if state == PageState::ChainRecordMalformed {
                        // 不合法的链指针记录之后的片不再挂在链上：链就到 k 为止
                        states.truncate(k + 1);
                    }
                    shapes.push((format!("len{n}-{state:?}@{k}"), states, ChainEnd::LastPage, rows));
                }
            }
            for (n, j) in [(255usize, 0usize), (256, 0), (256, 1), (257, 0), (257, 200), (3, 1)] {
                shapes.push((
                    format!("len{n}-back-to-{j}"),
                    vec![PageState::Intact; n],
                    ChainEnd::BackTo(j),
                    rows,
                ));
            }
        }
        for (name, states, end, rows) in &shapes {
            let image = image_with_a_chain(&base, states, *end, *rows);
            let verdicts = check_pool_image(&image);
            let (red, partial) = verdict_lists(&verdicts);
            let cpu_transcribed = transcribed_walk(states, *end, None);
            let gpu_transcribed = transcribed_walk(states, *end, Some(256));
            let i929 = kind_of(&verdicts, "I-9.29");
            let cpu_door_open = i929 != "P";
            let transcription_matches_the_cpu =
                cpu_door_open == (cpu_transcribed == ChainOutcome::ReachedTheLastPage);
            let gpu_prediction = match gpu_transcribed {
                ChainOutcome::UndecidableAfterThePagesWalked => "undecidable(61)->cpu".to_string(),
                ChainOutcome::ReachedTheLastPage => format!("door-open i929={}", if cpu_door_open { i929 } else { "?" }),
                ChainOutcome::StoppedAt(_) => "door-closed i929-bit=0".to_string(),
            };
            let divergence = match gpu_transcribed {
                ChainOutcome::UndecidableAfterThePagesWalked => false,
                other => other != cpu_transcribed,
            };
            println!(
                "J1\tshape={name}\trows={rows:?}\tpages={}\tcpu_i929={i929}\tcpu_transcribed={cpu_transcribed:?}\ttranscription_matches_cpu={transcription_matches_the_cpu}\tgpu={gpu_prediction}\tgpu_cpu_divergence={divergence}\tred={}\tpartial={}",
                states.len(),
                red.join(","),
                partial.join(",")
            );
        }
        println!(
            "J1-DONE shapes={} seconds={:.1}",
            shapes.len(),
            started.elapsed().as_secs_f64()
        );
    }
}
