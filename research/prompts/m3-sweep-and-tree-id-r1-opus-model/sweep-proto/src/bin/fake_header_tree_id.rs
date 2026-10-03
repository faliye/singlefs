//! S2 / I-7.8：用户文件内容里恰在槽边界上带一个码 2 节点头（头校验和自洽），池级 checker 的扫描方向把它当成「盘上出现过的树 ID」。
//! 合法历史：mkfs → 可写挂载 → 第一个文件（内容由用户给，任意字节）。用法：fake_header_tree_id <臂>
//!   control：同长内容、不带伪头；high：伪头树 ID = 2^40、写序实例 = 本实例、诞生代号 1；
//!   foreign：同 high，fsid 改成别的池的；same65536：树 ID 照抄 extent 树 65536、诞生代号 1（同一树 ID 跨发号点的伪证）；
//!   inode：伪头不在槽边界（内容偏移 0），对照「只有落在槽边界上才被看见」。
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolWriter};
use singlefs_format::{DATA_UNIT_PAYLOAD_OFFSET, SLOT_BYTES};
use sweep_proto::*;

fn main() {
    let arm = std::env::args().nth(1).expect("臂");
    let parameters = parameters(4);
    let mut devices = fresh_devices();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let Mounted { output, mut allocator, current } = mount_writable(&parameters, &mut devices).expect("挂载");
    let instance = output.instance;
    // 模板：盘上一个真码 2 节点的头。
    let pool = pool_of(&devices);
    let ((device, slot), template) = every_unit_header(&pool).into_iter().find(|(_, h)| h.class == 2).expect("盘上有码 2 节点");
    let head = read_slot_head(&pool, singlefs_core::address::DeviceIdentity(device), slot, 4096);
    let header_end = header_end_of(&head).expect("头");
    let k = 2 * usize::from(head[51]);
    let mut fake = head[..header_end].to_vec();
    match arm.as_str() {
        "high" | "foreign" | "inode" => fake[42..50].copy_from_slice(&(1u64 << 40).to_le_bytes()),
        "same65536" => fake[42..50].copy_from_slice(&65536u64.to_le_bytes()),
        _ => {}
    }
    fake[52 + k..60 + k].copy_from_slice(&1u64.to_le_bytes());
    fake[68 + k..72 + k].copy_from_slice(&instance.0.to_le_bytes());
    if arm == "foreign" {
        fake[60 + k..68 + k].copy_from_slice(&0x0123_4567_89ab_cdefu64.to_le_bytes());
    }
    reseal_header(&mut fake, header_end);
    let mut file = content(1, 20_000);
    // 数据单元第二个槽的起点 = 单元偏移 SLOT_BYTES = 内容偏移 SLOT_BYTES − 载荷偏移。
    let at = if arm == "inode" { 0 } else { usize::try_from(SLOT_BYTES - DATA_UNIT_PAYLOAD_OFFSET).expect("小") };
    if arm != "control" {
        file[at..at + header_end].copy_from_slice(&fake);
    }
    let published = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(&mut pool, &mut allocator, current.root(), FirstFile { content: &file, write_time_seconds: WRITE_TIME }, instance, current.record_bytes()).expect("第一个文件")
    };
    let image = pool_of(&devices);
    println!(
        "ARM {arm} template=(dev {device}, slot {slot}, class {}, tree {}) fake_at_content_offset={at} header_end={header_end} root=({},{}) watermark={}",
        template.class, template.tree, published.root.instance.0, published.root.checkpoint_txg.0, published.root.tree_identifier_watermark
    );
    println!("CHECKER red={:?}", violated(&image));
    println!("READ_ONLY_MOUNT ok={}", mount_read_only(&image).is_ok());
    let mut again = devices_of(&image);
    match mount_writable(&parameters, &mut again) {
        Ok(m) => println!("REMOUNT ok instance={} red_after={:?}", m.output.instance.0, violated(&pool_of(&again))),
        Err(e) => println!("REMOUNT err={}", format!("{e:?}").chars().take(200).collect::<String>()),
    }
}
