//! H1 行证据误认「取号跳过」落到 I-3.5：mkfs → 实例 1 写到 txg `last` → 实例 2 可写挂载（写行 txg last + 1）→ 管理员回退（一次向前发布）到 txg `target`
//! → 用户动作序列 `ops`（逗号分隔，每个取 over | seq | inodes | empty；`-` 表示不做）→ 把 `bad` 列出的那几个 txg 的根槽各翻一个字节 → 每组开关各判一次。
//! 回退之后的用户动作由用户决定，扫描脚本放开扫；这里只执行给定的那一串。
//! 用法：rollback_hole <S> <journal 环字节> <last> <target> <ops> <bad txg,…|none>
use hole_attack_proto::{build_plain, content, corrupt_root_slot, devices_of, overwrite, parameters, pool_of, publish_empty, show_all, WRITE_TIME};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::transaction::{publish_new_inodes, publish_sequential_write, FirstFile, PoolWriter};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let journal: u64 = a[2].parse().expect("journal");
    let last: u64 = a[3].parse().expect("last");
    let target: u64 = a[4].parse().expect("target");
    let parameters = parameters(s, journal);
    let (devices, _, _) = build_plain(&parameters, last);
    let mut devices = devices_of(&pool_of(&devices));
    let mounted = mount_writable(&parameters, &mut devices).expect("实例 2");
    let row_txg = mounted.output.row_publish.root().checkpoint_txg.0;
    let mut allocator = mounted.allocator.clone();
    let mut current = mounted.current.clone().into_file_version().expect("有文件");
    let before_rollback = current.root.checkpoint_txg.0;
    let rolled = roll_back_by_a_forward_publish(
        &parameters,
        &mut devices,
        &mut allocator,
        &mut current,
        RollbackTarget { instance: singlefs_core::address::InstanceGeneration(1), checkpoint_txg: singlefs_core::address::CheckpointTxg(target) },
    );
    println!("ROLLBACK row_txg={row_txg} before={before_rollback} target={target} ok={} now={}", rolled.is_ok(), current.root.checkpoint_txg.0);
    if let Err(error) = &rolled {
        println!("ROLLBACK_ERROR {error:?}");
        return;
    }
    let instance = current.root.instance;
    for op in a[5].split(',').filter(|op| *op != "-") {
        let salt = current.root.checkpoint_txg.0 + 1;
        current = match op {
            "over" => overwrite(&parameters, &mut devices, &mut allocator, &current),
            "seq" => {
                let mut longer = content(target);
                longer.extend(content(salt));
                let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
                publish_sequential_write(&mut pool, &mut allocator, &current, FirstFile { content: &longer, write_time_seconds: WRITE_TIME + salt }, instance).expect("顺序写")
            }
            "inodes" => {
                let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
                publish_new_inodes(&mut pool, &mut allocator, &current, 1, WRITE_TIME + salt, instance).expect("建 inode")
            }
            "empty" => {
                let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
                publish_empty(&mut pool, &mut allocator, &current, instance).expect("空发布")
            }
            other => panic!("不认识的动作 {other}"),
        };
    }
    let mut image = pool_of(&devices);
    println!("BUILT newest_txg={} ops={}", current.root.checkpoint_txg.0, a[5]);
    show_all(&image, "clean");
    if a[6] != "none" {
        // `upto:R`：把 row_txg..R 之前那几条根（实例 2 在 R 之前的全部根）的根槽都翻坏，R 成了洞右边第一条读得出的根。
        let bad: Vec<u64> = match a[6].strip_prefix("upto:") {
            Some(first_readable) => (row_txg..first_readable.parse::<u64>().expect("R")).collect(),
            None => a[6].split(',').map(|txg| txg.parse().expect("txg")).collect(),
        };
        println!("BAD_SLOTS {bad:?}");
        for txg in &bad {
            corrupt_root_slot(&mut image, &parameters, *txg);
        }
        show_all(&image, &format!("bad{}", a[6]));
    }
}
