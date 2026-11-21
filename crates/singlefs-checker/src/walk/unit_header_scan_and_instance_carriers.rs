//! 扫描方向读单元头：认领规则下跳过的槽、码 2 节点的树 ID 与写序实例，盘上带实例代号的根、记录与单元，
//! 判 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 扫描方向那一道与 I-7.7（系统配置实例代号不低于根环）。

use super::*;

/// I-7.8 扫描方向的第二道：一个码 2 节点是不是「写序实例 i 崩在发布之前写出的孤儿」——最新根指着的实例表里有 i 的一行
/// (i, Ti, ·)、Ti > 0，而节点的诞生代号 > Ti（改法 D，`research/prompts/m2-wave3-code-r1-main-verification.md`
/// 第三节 Y1 与第四节第 1 条，被攻过零轮）。
///
/// 为什么排除：行 (i, Ti, ·) 说实例 i 被施加到 Ti 为止（D23（journal 的角色与格式） 已定项 14，写行时取恢复之后的有效根），
/// 诞生代号 > Ti 的实例 i 节点就没有发布过。只数诞生代号 ≤ 根环最大 txg 那一道挡不住它：下一个实例第一次发布取的 txg 可以
/// 等于孤儿的诞生代号（D23（journal 的角色与格式） 已定项 14 注 3 的「≥ max(…) + 1」取等号），孤儿就被数进「出现过的」、在合法状态上判红。
/// 不排除 Ti = 0 的行（恢复写的中间实例行 (i, 0, 0) 与没发布过根的实例行，行里说不出它发布到哪一代）。
/// 节点头 fsid 不是本池的读不出写序实例（`unit_write_order_instance` 交 None），照旧算。
fn written_after_its_instance_was_last_applied(
    node: &[u8],
    birth_txg: u64,
    instance_table_rows: &[InstanceTableRow],
    filesystem_identifier_low: u64,
) -> bool {
    unit_write_order_instance(node, filesystem_identifier_low).is_some_and(|write_order_instance| {
        instance_table_rows.iter().any(|row| {
            row.instance == write_order_instance
                && row.published_checkpoint_txg > 0
                && birth_txg > row.published_checkpoint_txg
        })
    })
}

/// 扫描方向认领一个单元之后跳过几槽（D18（块里携带什么信息） 已定项 7 认领规则）：
/// 起点槽上解得出一个单元头——magic 对、类标签是 1 / 2 / 3、头校验和过（码 1 明文头末尾 105、码 2 86 + 2k、码 3 107，校验和都在 10）——
/// 就认领它：码 1 数据单元、码 3 打包记录单元两槽，码 2 节点一槽；被认领的单元占着的后几槽是它的载荷，那里的探针不再参与认领。
/// 不看 fsid：别的池的单元同样占着它那几槽（它是不是本池的、发没发布，归 I-1.4 / I-1.2）。解不出头 ⇒ None，这一槽不认领、下一槽照扫。
fn slots_claimed_by_a_unit_header(header: &[u8]) -> Option<u64> {
    if header.len() < 52 || &header[..4] != b"SFSU" {
        return None;
    }
    let (plain_header_end, slots_of_the_unit) = match header[6] {
        1 => (105, DATA_UNIT_BYTES / SLOT_BYTES),
        2 => (86 + 2 * usize::from(header[51]), NODE_BYTES / SLOT_BYTES),
        3 => (107, DATA_UNIT_BYTES / SLOT_BYTES),
        _outside_the_three_unit_classes => return None,
    };
    (plain_header_end <= header.len()
        && checksum_field_holds(header, plain_header_end, UNIT_HEADER_CHECKSUM_OFFSET))
    .then_some(slots_of_the_unit)
}

/// 扫描方向：候选槽上头校验和过的码 2 节点，取它头里的树 ID。槽按从小到大扫、照认领规则按整单元跳
/// （[`slots_claimed_by_a_unit_header`]）：用户文件内容里恰好对齐到槽边界、头校验和自洽的码 2 伪头落在它那个数据单元的第二槽，
/// 不认领就被数成「出现过的」树 ID，合法镜像上 I-7.8 误红
/// 只数诞生代号不超过 `newest_published_txg` 的节点：
/// 根环里没有一条根发布过的那一代，单元还不算「出现过」（I-7.8 的这一读法；字面读法会把
/// 崩在发布之前的那个事务写出的孤儿节点也算进去，见 records 十一·七）。再排除最新根的实例表判得出没发布过的孤儿
/// （[`written_after_its_instance_was_last_applied`]）。
///
/// 认领规则留的缝（判决同一行「F4 只修一半」，C89（单元头收口的遗留重算） 第 ② 笔）：释放了的两槽数据单元，第一槽被一槽的码 2 节点拿走、
/// 第二槽的旧载荷还在时，那个节点只认领一槽，旧载荷里的伪头照样被数进来。
///
/// 循环：每块盘按槽号升序走一遍候选槽（轮数 = 候选槽数），跨轮携带「下一个没被认领的槽」，只增不减；没有提前出口。
pub(super) fn scanned_tree_identifiers(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    newest_published_txg: u64,
    instance_table_rows: &[InstanceTableRow],
    filesystem_identifier_low: u64,
) -> BTreeSet<u64> {
    let mut identifiers = BTreeSet::new();
    for device in reader.devices() {
        // 读者给的候选排一遍；给不出候选时按区间迭代（本来就有序），不把几千万个槽号先收进 Vec
        let slots: Box<dyn Iterator<Item = u64>> = match reader.candidate_unit_slots(device) {
            Some(mut candidates) => {
                candidates.sort_unstable();
                candidates.dedup();
                Box::new(candidates.into_iter())
            }
            None => {
                let end = reader.device_bytes(device).unwrap_or(0) / SLOT_BYTES;
                Box::new(geometry.unit_area_start_slot..end)
            }
        };
        let mut first_slot_not_claimed = 0u64;
        for slot in slots {
            if slot < first_slot_not_claimed {
                continue;
            }
            let Some(bytes) = reader.read(device, slot * SLOT_BYTES, node_bytes()) else {
                continue;
            };
            let Some(claimed_slots) = slots_claimed_by_a_unit_header(&bytes) else {
                continue;
            };
            first_slot_not_claimed = slot + claimed_slots;
            if bytes[6] != 2 {
                continue;
            }
            let header_end = 86 + 2 * usize::from(bytes[51]);
            // 头校验和 `slots_claimed_by_a_unit_header` 刚核过（同一段、同一个字段），这里不再算一遍
            if header_end < bytes.len() {
                let birth_txg = read_u64(&bytes, 52 + 2 * usize::from(bytes[51]));
                if birth_txg <= newest_published_txg
                    && !written_after_its_instance_was_last_applied(
                        &bytes,
                        birth_txg,
                        instance_table_rows,
                        filesystem_identifier_low,
                    )
                {
                    identifiers.insert(read_u64(&bytes, 42));
                }
            }
        }
    }
    identifiers
}

/// 扫描方向读单元头读多少：码 2 头最宽 86 + 2 × 255 = 596 字节，读一块 4096 就罩得住三类。
pub(super) const UNIT_HEADER_SCAN_BYTES: usize = 4096;

/// 单元头写序里的实例代号：头校验和过、fsid 与本池相同才算（码 1：fsid 83、写序 91、头末 105；码 2：fsid 60 + 2k、
/// 写序 68 + 2k、头末 86 + 2k；码 3：fsid 81、写序 93、头末 107——D18（块里携带什么信息） 已定项 18 的偏移表）。
fn unit_write_order_instance(bytes: &[u8], filesystem_identifier_low: u64) -> Option<u32> {
    if bytes.len() < 52 || &bytes[..4] != b"SFSU" {
        return None;
    }
    let (header_end, fsid_offset, instance_offset) = match bytes[6] {
        1 => (105, 83, 91),
        2 => {
            let key_span = 2 * usize::from(bytes[51]);
            (86 + key_span, 60 + key_span, 68 + key_span)
        }
        3 => (107, 81, 93),
        _ => return None,
    };
    if header_end > bytes.len()
        || !checksum_field_holds(bytes, header_end, 10)
        || read_u64(bytes, fsid_offset) != filesystem_identifier_low
    {
        return None;
    }
    Some(read_u32(bytes, instance_offset))
}

/// 盘上带实例代号的东西：根环里自证过的根、journal 环里 fsid 相同的记录、单元区里头校验和过且 fsid 相同的单元写序。
/// 读得出的都收进 `carriers`；有一个该读的槽读不出时照样往下读，只记下 `some_slot_is_unreadable`（I-7.7（系统配置实例代号不低于根环）
/// ① 那一半读不全报不适用，② 在读得出的这些上照判）。
struct InstanceCarriersOnDisk {
    carriers: Vec<(String, u32)>,
    some_slot_is_unreadable: bool,
}

fn instance_carriers(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    filesystem_identifier_low: u64,
) -> InstanceCarriersOnDisk {
    let mut some_slot_is_unreadable = false;
    let root_slot_bytes = usize::try_from(geometry.physical_block_size).expect("根槽宽");
    for (_, _, device, offset) in root_slot_positions(geometry) {
        if reader.read(device, offset, root_slot_bytes).is_none() {
            some_slot_is_unreadable = true;
        }
    }
    let mut carriers: Vec<(String, u32)> = roots
        .iter()
        .map(|(region, slot, root)| (format!("根环区域 {region} 槽 {slot} 的根"), root.instance))
        .collect();
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    for device in reader.devices() {
        // 读者给不出候选时逐槽扫整环：按区间迭代，不把几十万个槽号先收进一个 Vec
        let journal_slots: Box<dyn Iterator<Item = u64>> =
            match reader.candidate_journal_slots(device) {
                Some(candidates) => Box::new(candidates.into_iter()),
                None => Box::new(0..geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES),
            };
        for slot in journal_slots {
            let offset =
                geometry.journal_ring_start_slot * SLOT_BYTES + slot * JOURNAL_RECORD_BYTES;
            let Some(bytes) = reader.read(device, offset, record_bytes) else {
                some_slot_is_unreadable = true;
                continue;
            };
            if let Ok(record) = check_journal_record(&bytes, filesystem_identifier_low) {
                carriers.push((
                    format!("盘 {device} journal 环槽 {slot} 的记录"),
                    record.instance,
                ));
            }
        }
        // 读者给不出候选时逐槽扫整个单元区：按区间迭代，1 TiB 的盘有几千万个槽，收进 Vec 要几百 MiB
        let unit_slots: Box<dyn Iterator<Item = u64>> = match reader.candidate_unit_slots(device) {
            Some(candidates) => Box::new(candidates.into_iter()),
            None => {
                let end = reader.device_bytes(device).unwrap_or(0) / SLOT_BYTES;
                Box::new(geometry.unit_area_start_slot..end)
            }
        };
        for slot in unit_slots {
            let Some(bytes) = reader.read(device, slot * SLOT_BYTES, UNIT_HEADER_SCAN_BYTES) else {
                some_slot_is_unreadable = true;
                continue;
            };
            if let Some(instance) = unit_write_order_instance(&bytes, filesystem_identifier_low) {
                carriers.push((format!("盘 {device} 槽 {slot} 的单元写序"), instance));
            }
        }
    }
    InstanceCarriersOnDisk {
        carriers,
        some_slot_is_unreadable,
    }
}

/// I-7.7（系统配置实例代号不低于根环） 的 ① ②（C322（取号那一步的屏障怎么放没有条款））：
/// 每盘的实例代号取它两槽里全部自证过（fsid 与本池相同）的槽中最大的；① 本池的每个根记录、journal 记录、单元写序的实例代号
/// 不超过各盘的最大者，② 各盘不等时较大者不出现在它们里——两句合起来就是「每一个带号的东西都不超过各盘最大号里最小的那个」。
pub(super) fn judge_instance_carriers(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    judgements: &mut Judgements,
) {
    let highest_by_device: Vec<Option<u32>> = verified_system_configuration_slots(reader)
        .into_iter()
        .map(|(_, slots)| {
            slots
                .iter()
                .filter(|(view, _)| view.filesystem_identifier == geometry.filesystem_identifier)
                .map(|(view, _)| view.journal_instance)
                .max()
        })
        .collect();
    let Some(highest_by_device) = highest_by_device.into_iter().collect::<Option<Vec<u32>>>()
    else {
        judgements.not_applicable(
            "I-7.7",
            "有一块盘没有本池 fsid 的系统配置槽：那块盘归不归这个池由 I-1.4 判",
        );
        return;
    };
    let filesystem_identifier_low = u64::from_le_bytes(
        geometry.filesystem_identifier[..8]
            .try_into()
            .expect("fsid 前 8 字节"),
    );
    let InstanceCarriersOnDisk {
        carriers,
        some_slot_is_unreadable,
    } = instance_carriers(reader, geometry, roots, filesystem_identifier_low);
    let pool_highest = highest_by_device.iter().copied().max().unwrap_or(0);
    let pool_lowest = highest_by_device.iter().copied().min().unwrap_or(0);
    // ① 任一根环区域、journal 记录槽、单元头读不出时这一半不判，报部分没判、不报成立（C701（别的判法静默跳过读不出的部分））：
    // 载体在镜像上、在射程里，只是读不出（条文见 `02-根环.md` I-7.7）。
    // ② 的不适用条件是另一件事（镜像记不下这次挂载独占打开的集合），读不全时照样在读得出的载体上判：找到较大者就红；
    // 找不到时读不出的那几个槽里可能就坐着它，这一半也判不了（代码审阅 6c 第 9 条：原来一个槽读不出就 ① ② 一起报不适用）。
    if some_slot_is_unreadable {
        judgements.part_of_the_range_left_unjudged(
            "I-7.7",
            "根环、journal 环或单元区有读不出的槽：① 那一半读不出的载体没判，② 在读得出的载体上没找到较大者，读不出的那几个也没判".to_string(),
        );
    } else {
        let above_every_disk = carriers
            .iter()
            .find(|(_, instance)| *instance > pool_highest);
        judgements.judge("I-7.7", above_every_disk.is_none(), || {
            let (what, instance) = above_every_disk.expect("判红时有");
            format!("{what}带实例代号 {instance}，高于各盘系统配置的最大者 {pool_highest}（①）")
        });
    }
    // ② 只在各盘的号不等时有对象（「各盘的实例代号……不等时，较大者不出现在……」）；各盘相等时高于它的载体是 ① 那一半的事。
    let disks_disagree = pool_lowest < pool_highest;
    let witnessed_by_one_disk = carriers
        .iter()
        .find(|(_, instance)| disks_disagree && *instance > pool_lowest);
    if witnessed_by_one_disk.is_some() || !some_slot_is_unreadable {
        judgements.judge("I-7.7", witnessed_by_one_disk.is_none(), || {
            let (what, instance) = witnessed_by_one_disk.expect("判红时有");
            format!("各盘实例代号 {highest_by_device:?} 不等，而{what}带着较大的 {instance}（②）")
        });
    }
}
