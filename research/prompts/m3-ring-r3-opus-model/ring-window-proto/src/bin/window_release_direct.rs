//! m3-ring-r2 攻方原型 G2 放槽量：K1 那一类后洞（新实例写行那次崩，h = 3S + 4）之后挂载 k，再覆盖写 K 次；
//! 每一步量最新根那一版账里「已释放、还扣着」的槽数（盘 0），与池级 checker（开不开窗口由 SINGLEFS_CHECKER_WINDOW 定）。
//! 可补的后洞（同一实例覆盖写崩、今天不补）作对照：历史 fillable。
//!
//! 用法：window_release_direct <S> <k1|fillable> <K>
use ring_window_proto::{build_prefix, content, parameters, pool_of, violated, WRITE_TIME, devices_of};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{allocation_records_under_root, choose_system_configuration, readable_roots_with_ring_slots};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::MemoryPool;

fn withheld(image: &MemoryPool) -> (u64, u64, u64) {
    let configuration = choose_system_configuration(image).expect("系统配置");
    let im = &configuration.immutable;
    let roots: Vec<_> = readable_roots_with_ring_slots(image, &im.region_devices, &im.sizes, &im.filesystem_identifier).into_iter().map(|(_, r)| r).collect();
    let newest = *roots.iter().max_by_key(|r| (r.checkpoint_txg, r.instance)).expect("根");
    let records = allocation_records_under_root(image, &newest).expect("账");
    let released: u64 = records.iter().filter(|r| r.device == DeviceIdentity(0) && r.is_released).map(|r| u64::from(r.span_slots)).sum();
    let allocated: u64 = records.iter().filter(|r| r.device == DeviceIdentity(0)).map(|r| u64::from(r.span_slots)).sum();
    (newest.checkpoint_txg.0, released, allocated)
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let history = a[2].as_str();
    let k: u64 = a[3].parse().expect("K");
    let parameters = parameters(s);
    let hole = 3 * s + 4;
    if history == "k1" {
        std::env::set_var("CRASH_THE_ROW_PUBLISH", "1");
    }
    let prefix = build_prefix(s, hole);
    std::env::remove_var("CRASH_THE_ROW_PUBLISH");
    let mut devices = devices_of(&prefix.image);
    let mounted = mount_writable(&parameters, &mut devices).expect("挂载");
    let instance = mounted.output.instance;
    let mut allocator = mounted.allocator.clone();
    let mut current = mounted.current.clone().into_file_version().expect("带文件");
    let anchor = format!("{:?}", singlefs_core::mount::window_anchor());
    let cw = std::env::var("SINGLEFS_CHECKER_WINDOW").unwrap_or_default();
    let (n, r, al) = withheld(&pool_of(&devices));
    let mut peak = r;
    let mut red_steps = 0u64;
    let red = violated(&pool_of(&devices));
    if !red.is_empty() { red_steps += 1; }
    println!("DIRECT s={s} history={history} anchor={anchor} checker_window={cw} step=mounted newest={n} released_withheld_dev0={r} allocated_dev0={al} red={red:?}");
    for step in 0..k {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(&mut pool, &mut allocator, &current, FirstFile { content: &content(300 + step), write_time_seconds: WRITE_TIME + 300 + step }, instance).expect("覆盖写");
        let image = pool_of(&devices);
        let (n, r, al) = withheld(&image);
        peak = peak.max(r);
        let red = violated(&image);
        if !red.is_empty() { red_steps += 1; }
        println!("DIRECT s={s} history={history} anchor={anchor} checker_window={cw} step={step} newest={n} released_withheld_dev0={r} allocated_dev0={al} red={red:?}");
    }
    println!("DIRECT_SUMMARY s={s} history={history} anchor={anchor} checker_window={cw} hole={hole} peak_released_withheld_dev0={peak} red_steps={red_steps} steps={}", k + 1);
}
