//! B5（C572）：回退目标引用一个隔离单元。V1（第一个文件）的数据单元 u1 在盘 1 那一份在 V1 → V2 那次覆盖写释放前读盘核时对不上：
//! 两份都留在已分配（D19 已定项 5 硬规则 1）。之后挂着时回退到 V1。三种造法：
//! - `persist`：盘 1 那一份一直坏（翻一个字节，不改回）。
//! - `transient`：释放核那一刻坏（两次读都对不上），V2 发布之后那一份又读对了（字节翻回去）：隔离照旧，单元其实完好。
//! - `control`：不隔离：V2 正常释放 u1，之后才把盘 1 那一份翻坏（复活集那一支）。
//! 回退的臂由核心副本里的 OPUS_C572_ARM 定（空 / verify_quarantined / verify_all_shared / refuse_quarantined）。
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::transaction::PoolVersion;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::SharedStream;

use crate::pool::{content_of, OVERWRITE_BYTES};
use crate::util::*;

fn flip(pool: &mut crate::pool::PoolUnderTest<Rec>, device: DeviceIdentity, offset: u64) {
    let index = pool.devices.iter().position(|(id, _)| *id == device).expect("有这块盘");
    let image = &mut pool.devices[index].1.wrapped_device_mut().image;
    let sector = offset - offset % 512;
    let mut bytes = image.read(singlefs_core::address::DeviceOffsetInBytes(sector), 512);
    bytes[(offset % 512) as usize] ^= 0xff;
    image.write(singlefs_core::address::DeviceOffsetInBytes(sector), &bytes);
}

pub fn main_b5() {
    for form in env_list("FORMS", "persist,transient,control") {
        let stream = SharedStream::retaining_contents();
        let mut pool = start_pool(&stream);
        let v1 = pool.current_file_version().clone();
        let v1_content = pool.content_of_the_current_version.clone();
        let u1 = v1.data_pointers[0].locations;
        let bad = u1.iter().find(|l| l.device == DeviceIdentity(1)).copied().expect("盘 1 有一份");
        let bad_offset = bad.slot.0 * SLOT_BYTES + 100;
        if form != "control" {
            flip(&mut pool, bad.device, bad_offset);
        }
        let v2 = pool.overwrite(OVERWRITE_BYTES).expect("V2");
        let v2_output = pool.current_file_version().clone();
        let quarantined = v2_output.quarantined_after_release_checksum_mismatch.len();
        let still_allocated = pool
            .session()
            .allocator
            .records()
            .iter()
            .filter(|r| r.slot == bad.slot && !r.is_released)
            .count();
        match form.as_str() {
            "transient" => flip(&mut pool, bad.device, bad_offset),
            "control" => flip(&mut pool, bad.device, bad_offset),
            _ => {}
        }
        let _ = v2;
        // 再发一版 V3，让 V1 不是紧挨着现行那一版（回退目标在环里、按实例表有效、带文件）。
        pool.overwrite(OVERWRITE_BYTES).expect("V3");
        for _ in 0..env_u64("EXTRA_OVERWRITES", 0) {
            pool.overwrite(OVERWRITE_BYTES).expect("V3 之后再写");
        }
        let checker_before: Vec<String> = check_pool_image(&pool.image())
            .into_iter()
            .filter_map(|(n, v)| matches!(v, InvariantVerdict::Violated(_)).then(|| n.to_string()))
            .collect();
        let mut session = pool.session.take().expect("有会话");
        let mut current = session.current.file_version().expect("带文件").clone();
        let target = RollbackTarget { instance: v1.root.instance, checkpoint_txg: v1.root.checkpoint_txg };
        let outcome = roll_back_by_a_forward_publish(&pool.parameters, &mut pool.devices, &mut session.allocator, &mut current, target);
        let outcome_text = match &outcome {
            Ok(rolled) => format!(
                "ok resurrected_copies={} released_units={} quarantined_copies_at_rollback={}",
                rolled.resurrected_user_visible_copies.len(),
                rolled.released_user_visible_units.len(),
                rolled.quarantined_user_visible_copies.len()
            ),
            Err(e) => format!("refused {}", crate::b1::counter_name(&format!("{e:?}"))),
        };
        let new_references_u1 = outcome.is_ok()
            && current.data_pointers.iter().any(|p| p.locations.iter().any(|l| l.slot == bad.slot));
        session.current = PoolVersion::WithFile(current);
        pool.session = Some(session);
        let image = pool.image();
        let checker: Vec<String> = check_pool_image(&image)
            .into_iter()
            .filter_map(|(n, v)| matches!(v, InvariantVerdict::Violated(_)).then(|| n.to_string()))
            .collect();
        let mut reads = Vec::new();
        for (tag, lost) in [("both", None), ("lost0", Some(0)), ("lost1", Some(1))] {
            let (root, result) = read_only(&image_without(&image, lost));
            let verdict = match &result {
                Ok(c) if *c == v1_content => "reads_v1".to_string(),
                Ok(c) if *c == content_of(OVERWRITE_BYTES, 0) => "reads_other".to_string(),
                Ok(_) => "reads_some_other_version".to_string(),
                Err(e) => format!("failed_{}", crate::b1::counter_name(e)),
            };
            reads.push(format!("{tag}:{:?}:{verdict}", root.map(|r| r.1 .0)));
        }
        println!(
            "OPUS-B5 form={form} arm={:?} extra={} rollback_txg={} checker_before_rollback={checker_before:?} quarantined_copies_at_v2={quarantined} u1_records_still_allocated={still_allocated} rollback={outcome_text} new_root_references_u1={new_references_u1} checker_violated={checker:?} ro=[{}]",
            std::env::var("OPUS_C572_ARM").unwrap_or_default(),
            env_u64("EXTRA_OVERWRITES", 0),
            pool.session().current.root().checkpoint_txg.0,
            reads.join(" ")
        );
    }
}
