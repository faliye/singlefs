//! 实例表单元里的记录（D18（块里携带什么信息） 已定项 11）：`kind` 0 行与末尾的链指针记录。
//! 恢复（回退行的 W）、可写挂载与回退（写行）都从这里解；checker 是独立解析器，不共用这份。

use crate::address::{CheckpointTxg, InstanceGeneration};
use crate::bytes::{ByteReader, ByteWriter};
use crate::unit::parse_packed_unit;
use singlefs_format::INSTANCE_ROW_BYTES;

/// 实例表 `kind` 0 行记录（D18（块里携带什么信息） 已定项 11）：
/// `kind 1 | 实例代号 4 | 所选根的 checkpoint_txg 8 | 属于该实例的最大已施加事务号 W 8 | flags 1（bit0 = 回退行）| 预留 66`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceRow {
    pub instance: InstanceGeneration,
    pub selected_root_txg: CheckpointTxg,
    pub applied_transaction_high_water: u64,
    pub is_rollback: bool,
}

const INSTANCE_ROW_KIND_ROW: u8 = 0;
const INSTANCE_ROW_KIND_CHAIN: u8 = 1;
const INSTANCE_ROW_FLAG_ROLLBACK: u8 = 0b0000_0001;

impl InstanceRow {
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize::try_from(INSTANCE_ROW_BYTES).expect("88"));
        writer.put_u8(INSTANCE_ROW_KIND_ROW);
        writer.put_u32(self.instance.0);
        writer.put_u64(self.selected_root_txg.0);
        writer.put_u64(self.applied_transaction_high_water);
        writer.put_u8(if self.is_rollback {
            INSTANCE_ROW_FLAG_ROLLBACK
        } else {
            0
        });
        writer.skip(66);
        writer.assert_position(INSTANCE_ROW_BYTES, "实例表行记录");
        writer.into_bytes()
    }

    /// `kind` 0 才是行；链指针记录（`kind` 1）与别的 `kind` 返回 `None`。flags 只认 bit0，别的位非 0 拒收。
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != usize::try_from(INSTANCE_ROW_BYTES).expect("88")
            || bytes[0] != INSTANCE_ROW_KIND_ROW
        {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 1);
        let instance = InstanceGeneration(reader.get_u32());
        let selected_root_txg = CheckpointTxg(reader.get_u64());
        let applied_transaction_high_water = reader.get_u64();
        let flags = reader.get_u8();
        if flags & !INSTANCE_ROW_FLAG_ROLLBACK != 0 {
            return None;
        }
        Some(Self {
            instance,
            selected_root_txg,
            applied_transaction_high_water,
            is_rollback: flags & INSTANCE_ROW_FLAG_ROLLBACK != 0,
        })
    }
}

/// 一片实例表单元里的记录：行在前、链指针记录最末（D18（块里携带什么信息） 已定项 11）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceTableRecords {
    pub rows: Vec<InstanceRow>,
    pub chain_record: Vec<u8>,
}

impl InstanceTableRecords {
    /// 从一片实例表单元解出来；行与链指针之外的 `kind`、链指针不在最末、没有链指针，都是 `None`。
    #[must_use]
    pub fn parse(instance_table_unit: &[u8]) -> Option<Self> {
        let unit = parse_packed_unit(instance_table_unit).ok()?;
        let (last, rows_bytes) = unit.records.split_last()?;
        if last.first().copied() != Some(INSTANCE_ROW_KIND_CHAIN) {
            return None;
        }
        let mut rows = Vec::with_capacity(rows_bytes.len());
        for bytes in rows_bytes {
            rows.push(InstanceRow::parse(bytes)?);
        }
        Some(Self {
            rows,
            chain_record: last.clone(),
        })
    }

    #[must_use]
    pub fn to_records(&self) -> Vec<Vec<u8>> {
        let mut records: Vec<Vec<u8>> = self.rows.iter().map(InstanceRow::to_bytes).collect();
        records.push(self.chain_record.clone());
        records
    }
}
