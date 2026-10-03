//! H2 来源一（根槽读坏，同一实例）：mkfs → 实例 1 写到 txg `last` → 把列出的那几个 txg 的根槽各翻一个字节（自证不过）→ 每组开关各判一次。
//! 坏写者由环境变量 HOLE_BAD_WRITER 在副本 core 里开（见 scripts/apply-walk-patch.py）。
//! 用法：slot_bad <S> <journal 环字节> <last> <txg,txg,…|none>
use hole_attack_proto::{build_plain, corrupt_root_slot, parameters, pool_of, show_all};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let s: u64 = a[1].parse().expect("S");
    let journal: u64 = a[2].parse().expect("journal");
    let last: u64 = a[3].parse().expect("last");
    let parameters = parameters(s, journal);
    let (devices, _, current) = build_plain(&parameters, last);
    let mut image = pool_of(&devices);
    println!("BUILT newest_txg={} bad_writer={:?}", current.root.checkpoint_txg.0, std::env::var("HOLE_BAD_WRITER").ok());
    show_all(&image, "clean");
    if a[4] != "none" {
        for txg in a[4].split(',') {
            corrupt_root_slot(&mut image, &parameters, txg.parse().expect("txg"));
        }
        show_all(&image, &format!("bad{}", a[4]));
    }
}
