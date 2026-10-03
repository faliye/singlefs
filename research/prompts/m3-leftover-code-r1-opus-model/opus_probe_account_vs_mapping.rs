//! 攻方腿副本探针（m3-leftover-code-r1-opus，K2 ②）：根环里每条根，账那一臂（`placements_referenced_by_root`）
//! 与从映射现算那一臂（`placements_referenced_by_root_computed_from_the_mapping`）交出的落点集逐个比。
mod common;
mod common_tree_split;

use common::parameters;
use common_tree_split::TreeSplitPool;
use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::mount::{
    mount_writable, opus_probe_abandoned_flags, opus_probe_placements_of_every_ring_root,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::transaction::{CodeTwoTreeNodeCapacities, FirstFile, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use std::collections::BTreeSet;

fn content_needing(data_units: usize, seed: usize) -> Vec<u8> {
    let capacity = data_unit_payload_capacity();
    let length = (data_units - 1) * capacity + capacity / 2;
    (0..length)
        .map(|index| u8::try_from((index * 31 + seed * 7 + 1) % 251).expect("小于 256"))
        .collect()
}

type Set = BTreeSet<(u32, u64, u64)>;

fn as_set(placements: &[(DeviceIdentity, singlefs_core::address::SlotNumber, u64)]) -> Set {
    placements
        .iter()
        .map(|(device, slot, span)| (device.0, slot.0, *span))
        .collect()
}

/// 交回这一步的差异行数。
fn compare<Device: BlockDevice>(step: &str, devices: &Vec<(DeviceIdentity, Device)>) -> usize {
    let flags = opus_probe_abandoned_flags(devices);
    let mut differing = 0;
    for (root, tree_table_empty, account, mapping) in opus_probe_placements_of_every_ring_root(devices) {
        let key = (root.instance.0, root.checkpoint_txg.0);
        let abandoned = flags
            .iter()
            .find(|(k, _)| (k.0 .0, k.1 .0) == key)
            .and_then(|(_, a)| *a);
        let line = match (&account, &mapping) {
            (Some(account), Some(mapping)) => {
                let a = as_set(account);
                let m = as_set(mapping);
                if a == m {
                    format!("same n={}", a.len())
                } else {
                    differing += 1;
                    let only_account: Vec<_> = a.difference(&m).collect();
                    let only_mapping: Vec<_> = m.difference(&a).collect();
                    format!(
                        "DIFF account={} mapping={} only_account={only_account:?} only_mapping={only_mapping:?}",
                        a.len(),
                        m.len()
                    )
                }
            }
            (Some(account), None) => format!("mapping=None account={}", account.len()),
            (None, Some(mapping)) => format!("account=None mapping={}", mapping.len()),
            (None, None) => "both=None".to_string(),
        };
        let mapping_root_empty =
            root.mapping_root == singlefs_core::pointer::NodePointer::empty_root();
        eprintln!(
            "name=opus-probe step={step} root=({},{}) tree_table_empty={tree_table_empty:?} mapping_root_empty={mapping_root_empty} abandoned={abandoned:?} {line}",
            key.0, key.1
        );
    }
    differing
}

#[test]
fn opus_probe_account_and_mapping_agree_on_every_ring_root() {
    let pool = TreeSplitPool::with_the_first_file_version_under(
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    );
    let mut devices = pool.reopened_devices();
    let mut total = compare("first-file", &devices);
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    total += compare("mount-1", &devices);
    let steps: Vec<(&str, Box<dyn Fn() -> Vec<u8>>)> = vec![
        ("seq-320", Box::new(|| content_needing(320, 1))),
        ("seq-150", Box::new(|| content_needing(150, 2))),
        ("seq-330", Box::new(|| content_needing(330, 3))),
    ];
    for (name, content) in &steps {
        let bytes = content();
        session
            .publish_user_change(
                &mut devices,
                UserChange::SequentialWrite(FirstFile {
                    content: &bytes,
                    write_time_seconds: 1_700_000_100,
                }),
            )
            .expect(name);
        total += compare(name, &devices);
    }
    drop(session);
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    total += compare("mount-2", &devices);
    session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes {
                count: 2000,
                write_time_seconds: 1_700_000_200,
            },
        )
        .expect("建 inode");
    total += compare("inodes-2000", &devices);
    let small = content_needing(1, 9);
    session
        .publish_user_change(
            &mut devices,
            UserChange::Overwrite(FirstFile {
                content: &small,
                write_time_seconds: 1_700_000_300,
            }),
        )
        .expect("覆盖写");
    total += compare("overwrite", &devices);
    let again = content_needing(310, 5);
    session
        .publish_user_change(
            &mut devices,
            UserChange::SequentialWrite(FirstFile {
                content: &again,
                write_time_seconds: 1_700_000_400,
            }),
        )
        .expect("顺序写");
    total += compare("seq-310", &devices);
    let unlinked = session.publish_user_change(
        &mut devices,
        UserChange::Unlink {
            inode: InodeNumber(FIRST_INODE_NUMBER),
        },
    );
    eprintln!("name=opus-probe unlink={:?}", unlinked.as_ref().map(|p| p.versions_published.len()).map_err(|e| format!("{e:?}")));
    total += compare("unlink", &devices);
    drop(session);
    let mounted = mount_writable(&parameters(), &mut devices).expect("重开 3");
    let (_, _session) = MountedSession::of_the_mount(mounted);
    total += compare("mount-3", &devices);
    eprintln!("name=opus-probe-total differing_roots={total}");
}

/// 实例表长到两片之后，账那一臂与现算那一臂还一样吗（K2 ②「实例表多片」）。连着 370 次取号之后崩溃，下一次可写挂载一次写满一片再开第二片。
#[test]
fn opus_probe_account_and_mapping_agree_after_the_instance_table_grows_a_second_page() {
    use singlefs_core::transaction::{acquire_instance, PoolWriter};
    let pool = TreeSplitPool::with_the_first_file_version_under(
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    );
    let mut devices = pool.reopened_devices();
    {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        for _ in 0..375 {
            acquire_instance(&mut writer).expect("取号之后崩溃");
        }
    }
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载写两片实例表");
    let pages = singlefs_core::recovery::instance_table_page_pointers_as_far_as_readable(
        &devices,
        mounted.current.root(),
    )
    .len();
    eprintln!("name=opus-probe-pages pages={pages}");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let mut total = compare("two-pages-mount", &devices);
    let small = content_needing(1, 4);
    for round in 0..3u64 {
        session
            .publish_user_change(
                &mut devices,
                UserChange::Overwrite(FirstFile {
                    content: &small,
                    write_time_seconds: 1_700_001_000 + round,
                }),
            )
            .expect("覆盖写");
        total += compare("two-pages-overwrite", &devices);
    }
    eprintln!("name=opus-probe-total-two-pages differing_roots={total}");
}
