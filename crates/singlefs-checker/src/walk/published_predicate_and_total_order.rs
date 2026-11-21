//! 已发布谓词（I-1.2（块头写序已发布））、扫描方向的码 1 与码 3 单元、I-1.8（归并后版本全序） 与 I-7.3（环健康性）。

use super::*;

/// 已发布谓词的输入，按单元类的两种取法分成两个成员（`crates/singlefs-checker/invariants/01-块自描述.md` I-1.2（块头写序已发布） 那一行逐字：
/// 「有行 (i, T_pub, W) ⇒ 码 1 要 b ≤ T_pub ∨ n ≤ W、码 2 / 3 要 b ≤ T_pub」）：码 1 那一支带事务号 n，
/// 码 2 / 码 3 那一支不带——它们的取法不读 n，分成两个成员就不必给一个用不上的 n 编个值。
/// 码 2 与码 3 在谓词上是同一支，而它们在 I-1.8（归并后版本全序） 上不是（码 2 不进 I-1.8），
/// 所以这个枚举与 [`ContentUnitClass`] 是两件事，不合并。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WriteOrderUnderThePublishedPredicate {
    /// 码 1：诞生代号 b 与写序 (i, n)。
    DataUnit {
        birth_txg: u64,
        instance: u32,
        transaction: u64,
    },
    /// 码 2 / 码 3：诞生代号 b 与写序的实例代号 i。
    IndexOrPackedRecordUnit { birth_txg: u64, instance: u32 },
}

impl WriteOrderUnderThePublishedPredicate {
    pub(super) fn instance(self) -> u32 {
        match self {
            WriteOrderUnderThePublishedPredicate::DataUnit { instance, .. }
            | WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit { instance, .. } => {
                instance
            }
        }
    }

    pub(super) fn birth_txg(self) -> u64 {
        match self {
            WriteOrderUnderThePublishedPredicate::DataUnit { birth_txg, .. }
            | WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit { birth_txg, .. } => {
                birth_txg
            }
        }
    }
}

/// 扫描式方向自己的那一份已发布谓词（C410（checker 两个方向共享什么没有检查））：
/// 条款同 [`PublishedPredicate`]（I-1.2（块头写序已发布） 的谓词，C113（扫描重建时多版单元的现行版本判定无输入） P2），
/// 照条款另写，不调遍历式那一份与它的输入类型（D13（验证路线） 已定项 12「两者对同一条不变量独立判断，共享的只有格式解析」）。
/// 输入照旧：挂载根（最新那条根）的实例代号与 txg，加上遍历式读出的那一版实例表行——实例表是扫描重建的输入（C113 P2），
/// 传值不是调用。两份判得一样由测试模块里的对拍单测钉住。
pub(super) struct PublishedPredicateOfTheScanDirection<'rows> {
    pub(super) mount_root_instance: u32,
    pub(super) mount_root_checkpoint_txg: u64,
    pub(super) instance_table_rows: &'rows [InstanceTableRow],
}

impl PublishedPredicateOfTheScanDirection<'_> {
    /// 扫描到的一个码 1 / 码 3 单元头（诞生代号、写序实例、写序事务号）判没判成已发布：
    /// - 实例比挂载根大：在飞垃圾（上一次挂载取了号、写出单元、一个根都没发布就崩了），挡在「已发布」之外、不判损坏
    ///   （理由同 [`PublishedPredicate::holds_for`] 文档那一段）；
    /// - 实例等于挂载根：诞生代号不大于挂载根的 txg；
    /// - 实例比挂载根小：实例表里没有这一实例的行算已发布（那一实例的单元早于实例表的记载）；有行时诞生代号不大于那一行的已发布 txg，
    ///   码 1 另可凭事务号不大于那一行的已应用事务号水位。
    fn admits_the_header(
        &self,
        unit_class: ContentUnitClass,
        birth_txg: u64,
        instance: u32,
        transaction: u64,
    ) -> bool {
        if instance > self.mount_root_instance {
            return false;
        }
        if instance == self.mount_root_instance {
            return birth_txg <= self.mount_root_checkpoint_txg;
        }
        let mut row_of_the_instance: Option<&InstanceTableRow> = None;
        for row in self.instance_table_rows {
            if row.instance == instance {
                row_of_the_instance = Some(row);
                break;
            }
        }
        let Some(row) = row_of_the_instance else {
            return true;
        };
        let birth_is_published = birth_txg <= row.published_checkpoint_txg;
        match unit_class {
            ContentUnitClass::Data => {
                birth_is_published || transaction <= row.applied_transaction_high_water_mark
            }
            ContentUnitClass::PackedRecord => birth_is_published,
        }
    }
}

/// 已发布谓词（I-1.2（块头写序已发布） 的谓词，C113（扫描重建时多版单元的现行版本判定无输入） P2 /
/// D18（块里携带什么信息） 已定项 1）的输入：挂载根（池级 checker 里就是最新的那条根）的实例代号与 txg，
/// 加上它指着的那一版实例表里的行。这一份是遍历式方向的（I-4.2（无被引用未提交块）、I-9.6 墓碑那一半）；I-1.8（归并后版本全序） 的射程逐字是
/// 「可读**已发布**单元」，扫描式方向拿同一条条款另写的那一份过滤（[`PublishedPredicateOfTheScanDirection`]）。
pub(super) struct PublishedPredicate<'rows> {
    pub(super) mount_root_instance: u32,
    pub(super) mount_root_checkpoint_txg: u64,
    pub(super) instance_table_rows: &'rows [InstanceTableRow],
}

impl PublishedPredicate<'_> {
    /// 写序 (i, n) 与诞生代号 b 判没判成已发布。
    ///
    /// **i > 挂载根实例这一支在这里判成「不是已发布」，不判损坏**：I-1.2（块头写序已发布） 那一行逐字是
    /// 「i > 挂载根实例 ⇒ 判损坏」，而那句话站在恢复刚发布完新根的那一刻——那时不存在比挂载根更高的实例。
    /// 池级 checker 对着的是任意一份崩溃镜像：上一次挂载取了号、写出单元、一个根都没发布就崩了，盘上就有
    /// i > 最新根实例的单元，而它是合法的在飞垃圾。所以扫描方向这一支判不了（射程见 `crates/singlefs-checker/invariants/01-块自描述.md`
    /// I-1.2 的状态列），这里只把它挡在「已发布」之外：I-1.8（归并后版本全序） 的射程要的正是这个。
    /// 被引用的单元走的是另一条路——遍历方向由 I-4.2（无被引用未提交块） 判，见
    /// [`Walk::judge_birth_identity_of_a_referenced_unit`]。
    pub(super) fn holds_for(&self, write_order: WriteOrderUnderThePublishedPredicate) -> bool {
        match write_order.instance().cmp(&self.mount_root_instance) {
            Ordering::Greater => false,
            Ordering::Equal => write_order.birth_txg() <= self.mount_root_checkpoint_txg,
            Ordering::Less => match self
                .instance_table_rows
                .iter()
                .find(|row| row.instance == write_order.instance())
            {
                None => true,
                Some(row) => match write_order {
                    WriteOrderUnderThePublishedPredicate::DataUnit {
                        birth_txg,
                        transaction,
                        ..
                    } => {
                        birth_txg <= row.published_checkpoint_txg
                            || transaction <= row.applied_transaction_high_water_mark
                    }
                    WriteOrderUnderThePublishedPredicate::IndexOrPackedRecordUnit {
                        birth_txg,
                        ..
                    } => birth_txg <= row.published_checkpoint_txg,
                },
            },
        }
    }
}

/// I-1.8 ② 的那个全序键：码 1 与码 3 同形，(诞生代号, 实例代号, 事务号) 三元，诞生代号打头（`crates/singlefs-checker/invariants/01-块自描述.md`
/// I-1.8 那一行写死；码 1 那一半是 C464（I-1.8 对码 1 的全序键写窄了）的「① 加诞生代号打头」）。
/// 字段次序就是比较次序（derive 的 `Ord` 按字段先后比）。码 1 也要诞生代号打头：一个事务跨两个 checkpoint 时两次发布写出的同一个 key
/// 写序逐字节相同（写序取事务号低 48 位），只有诞生代号分得开它们——与码 3「同一实例连着两个 checkpoint 都不分配新事务号」是同一个问题。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct TotalOrderKey {
    pub(super) birth_txg: u64,
    pub(super) instance: u32,
    pub(super) transaction: u64,
}

/// 扫描方向读到的一个码 1 / 码 3 已发布单元（「可读」的后半截由 `payload_checksum_holds_on_disk` 现算）。
pub(super) struct ScannedContentUnit {
    pub(super) device: u32,
    pub(super) slot: u64,
    pub(super) unit_class: ContentUnitClass,
    /// 「类身份段全部字段含写序」去掉载荷校验和那一段：同一份内容的两个副本在这上面逐字节相同。
    merge_key: Vec<u8>,
    /// 类身份段里与版本无关的那一段：I-1.8 里「同 key」的那个 key。
    object_key: Vec<u8>,
    total_order_key: TotalOrderKey,
    pub(super) payload_checksum: u32,
    /// 明文头原样（`[0, 明文头末尾)`）：扫描重建（`rebuild` 模块）从它现算中央映射 key 与对象身份，这一份只交过去、这里不再解。
    pub(super) plain_header: Vec<u8>,
}

impl ScannedContentUnit {
    pub(super) fn describe(&self) -> String {
        format!(
            "盘 {} 槽 {} 的{}单元",
            self.device,
            self.slot,
            self.unit_class.name()
        )
    }
}

/// 一个候选槽上的头解成扫描项：magic、类标签在这一条的射程内、头校验和过、fsid 与本池相同、按已发布谓词判成已发布。
/// 任一步不成立就不是这一条判的对象（各自另有不变量说话：I-1.6 / I-2.4 / I-1.4 / I-1.2）。
fn content_unit_of_header(
    header: &[u8],
    device: u32,
    slot: u64,
    filesystem_identifier_low: u64,
    published: &PublishedPredicateOfTheScanDirection<'_>,
) -> Option<ScannedContentUnit> {
    if header.len() < UNIT_HEADER_SCAN_BYTES || &header[..4] != b"SFSU" {
        return None;
    }
    let unit_class = ContentUnitClass::of(header[6])?;
    let offsets = unit_class.header_offsets();
    if !checksum_field_holds(
        header,
        offsets.plain_header_end,
        UNIT_HEADER_CHECKSUM_OFFSET,
    ) || read_u64(header, offsets.filesystem_identifier) != filesystem_identifier_low
    {
        return None;
    }
    let birth_txg = read_u64(header, offsets.birth_txg);
    let instance = read_u32(header, offsets.write_order);
    let transaction =
        read_six_byte_unsigned(header, offsets.write_order + WRITE_ORDER_TRANSACTION_OFFSET);
    if !published.admits_the_header(unit_class, birth_txg, instance, transaction) {
        return None;
    }
    let total_order_key = TotalOrderKey {
        birth_txg,
        instance,
        transaction,
    };
    Some(ScannedContentUnit {
        device,
        slot,
        unit_class,
        merge_key: [
            &header[offsets.merge_key_before_payload_checksum],
            &header[offsets.merge_key_after_payload_checksum],
        ]
        .concat(),
        object_key: header[offsets.object_key].to_vec(),
        total_order_key,
        payload_checksum: read_u32(header, offsets.payload_checksum),
        plain_header: header[..offsets.plain_header_end].to_vec(),
    })
}

/// 扫描方向：候选槽上的码 1 / 码 3 已发布单元。32768 的单元占两个槽，第二个槽上没有 magic，扫描自然跳过它。
pub(super) fn scanned_content_units(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    filesystem_identifier_low: u64,
    published: &PublishedPredicateOfTheScanDirection<'_>,
) -> Vec<ScannedContentUnit> {
    let mut units = Vec::new();
    for device in reader.devices() {
        let slots: Box<dyn Iterator<Item = u64>> = match reader.candidate_unit_slots(device) {
            Some(candidates) => Box::new(candidates.into_iter()),
            None => {
                let end = reader.device_bytes(device).unwrap_or(0) / SLOT_BYTES;
                Box::new(geometry.unit_area_start_slot..end)
            }
        };
        for slot in slots {
            let Some(header) = reader.read(device, slot * SLOT_BYTES, UNIT_HEADER_SCAN_BYTES)
            else {
                continue;
            };
            if let Some(unit) =
                content_unit_of_header(&header, device, slot, filesystem_identifier_low, published)
            {
                units.push(unit);
            }
        }
    }
    units
}

/// 这一份在盘上读得出来、而且它的载荷与它自己头里那个载荷校验和对得上——「可读」的后半截。
/// 头校验和过而载荷对不上的那一份不可读（撕裂、被盖），不进 I-1.8 的归并组。
fn payload_checksum_holds_on_disk(reader: &dyn ImageReader, unit: &ScannedContentUnit) -> bool {
    copy_payload_checksum_holds_on_disk(reader, unit.unit_class, unit.device, unit.slot)
}

/// I-1.8（归并后版本全序）：码 1 / 3 的可读已发布单元按「类身份段全部字段含写序」归并成组之后，
/// ① **同一组的成员载荷相同**（码 3 由载荷校验和判、码 1 由载荷 CRC 判），不同即判损坏、不择；
/// ② **同 key 的各组两两全序键不等**——码 1 与码 3 的键都是 (诞生代号, 实例代号, 事务号) 三元（[`TotalOrderKey`]）。
///    码 1 的归并键是对象 key、诞生代号、fsid、写序四段连着（`ContentUnitClass::header_offsets`），扫描项的 fsid 都等于本池的
///    （`content_unit_of_header` 滤过），所以同对象 key、同全序键的两份码 1 归并键必然相同、落在同一组：码 1 的 ② 在扫描项上判不红，
///    判到的次数照算（一个对象 key 上两组以上判一次，与码 3 同），I-1.8 成立还是不适用由它与 ① 一起定。
/// 射程 w = 2（每次写两盘各一份，D2（RAID 条带策略） 已定项 6）：正常镜像上一组就是同一个单元的两个副本。
/// **「可读」的后半截只在要判红的那一步才现算**：一组里载荷校验和字段全相同时 ① 已经成立、不必把载荷读回来；
/// 字段不同才逐份验载荷——整单元 CRC 是这里最贵的一步，而崩溃点重放每个崩溃状态都要跑这一遍。
/// 交回判了几次：一次都没判到时调用方报「不适用」，不报成立（`test-discipline.md`「检查本身也可能是错的」）。
pub(super) fn judge_merged_version_total_order(
    reader: &dyn ImageReader,
    units: &[ScannedContentUnit],
    judgements: &mut Judgements,
) -> u64 {
    let mut groups: BTreeMap<&Vec<u8>, Vec<&ScannedContentUnit>> = BTreeMap::new();
    for unit in units {
        groups.entry(&unit.merge_key).or_default().push(unit);
    }
    let mut judged = 0u64;
    for members in groups.values() {
        if members.len() < 2 {
            continue;
        }
        let first_payload_checksum = members[0].payload_checksum;
        let every_member_carries_the_same_payload_checksum = members
            .iter()
            .all(|member| member.payload_checksum == first_payload_checksum);
        // 校验和字段全相同时这一条已经成立，`||` 短路掉后半截：那半截要把每一份载荷整个读回来算 CRC。
        let payloads_are_the_same = every_member_carries_the_same_payload_checksum || {
            let readable_payload_checksums: BTreeSet<u32> = members
                .iter()
                .filter(|member| payload_checksum_holds_on_disk(reader, member))
                .map(|member| member.payload_checksum)
                .collect();
            readable_payload_checksums.len() <= 1
        };
        judged += 1;
        judgements.judge("I-1.8", payloads_are_the_same, || {
            let members_and_their_payloads: Vec<String> = members
                .iter()
                .map(|member| {
                    format!(
                        "{} 载荷校验和 {:#010x}",
                        member.describe(),
                        member.payload_checksum
                    )
                })
                .collect();
            format!(
                "归并成一组的成员载荷不同（{}）：同一组的成员载荷必须相同，不同即判损坏、不择",
                members_and_their_payloads.join("、")
            )
        });
    }
    // 一组里的成员共享归并键，对象 key 与全序键都是归并键的子段 ⇒ 取第一个成员当这一组的代表。
    // 码 1 与码 3 都进 ② 这一半；对象 key 首字节是类标签，一个 key 上的各组必然同类，不会把两类混进同一个桶。
    let mut groups_by_object_key: BTreeMap<&Vec<u8>, Vec<&ScannedContentUnit>> = BTreeMap::new();
    for members in groups.values() {
        groups_by_object_key
            .entry(&members[0].object_key)
            .or_default()
            .push(members[0]);
    }
    for representatives in groups_by_object_key.values() {
        if representatives.len() < 2 {
            continue;
        }
        let distinct_total_order_keys: BTreeSet<&TotalOrderKey> = representatives
            .iter()
            .map(|representative| &representative.total_order_key)
            .collect();
        judged += 1;
        judgements.judge(
            "I-1.8",
            distinct_total_order_keys.len() == representatives.len(),
            || {
                let groups_on_this_key: Vec<String> = representatives
                    .iter()
                    .map(|representative| {
                        format!(
                            "{} 全序键 {:?}",
                            representative.describe(),
                            representative.total_order_key
                        )
                    })
                    .collect();
                format!(
                    "同一个 key 上归并出 {} 组，两两全序键不等这一条不成立（{}）",
                    representatives.len(),
                    groups_on_this_key.join("、")
                )
            },
        );
    }
    judged
}

#[cfg(test)]
mod tests;
