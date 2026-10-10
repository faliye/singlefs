//! 草稿探针（K5.2）：384 槽小盘上，第一个文件顺序写到贴边（扫写入长度），再 unlink 两个空 inode（第二个让左片吸收右片、开出退役树），
//! 再 unlink 第一个文件；每一步记结局。目的：删除类操作在贴边盘面上有没有因空间被拒；去掉退役那几项（变异）后有没有变。
mod common;
mod common_admission;

use common_admission::start_plain;
use singlefs_core::address::InodeNumber;
use singlefs_core::mounted_session::UserChange;
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::history::HistoryDeviceWidth;

fn short(result: &Result<singlefs_core::mounted_session::UserChangePublished, singlefs_core::mounted_session::UserChangeRefused>) -> String {
    match result {
        Ok(p) => format!("ok(raises={},versions={})", p.floor_raises.len(), p.versions_published.len()),
        Err(e) => {
            let text = format!("{e:?}");
            format!("ERR:{}", text.chars().take(60).collect::<String>())
        }
    }
}

/// 一次试验：返回 (写入结局, unlink2, unlink3, unlink1, 之后再写一次的结局)
fn trial(du: usize) -> (String, String, String, String, String) {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    let created = pool.publish(UserChange::NewInodes { count: 234, write_time_seconds: 1_788_000_100 }, None);
    if created.is_err() {
        return (format!("create:{}", short(&created)), String::new(), String::new(), String::new(), String::new());
    }
    let length = du * usize::try_from(data_unit_payload_capacity()).unwrap();
    let w = pool.sequential_write(length);
    let ws = short(&w);
    if w.is_err() {
        return (ws, String::new(), String::new(), String::new(), String::new());
    }
    let mut filled = 0usize;
    let mut fill_stop = String::from("cap");
    for _ in 0..600 {
        let c = pool.create_one_inode();
        if c.is_err() { fill_stop = short(&c); break; }
        filled += 1;
    }
    let ws = format!("{ws} then +{filled} inodes (stop:{})", fill_stop.chars().take(40).collect::<String>());
    let u2 = pool.publish(UserChange::Unlink { inode: InodeNumber(2), write_time_seconds: 1_788_000_200 }, None);
    let u3 = pool.publish(UserChange::Unlink { inode: InodeNumber(3), write_time_seconds: 1_788_000_201 }, None);
    let u1 = pool.publish(UserChange::Unlink { inode: InodeNumber(1), write_time_seconds: 1_788_000_202 }, None);
    let again = pool.sequential_write(length.min(3000));
    (ws, short(&u2), short(&u3), short(&u1), short(&again))
}

#[test]
fn sweep() {
    let lo: usize = std::env::var("K52_LO").ok().and_then(|s| s.parse().ok()).unwrap_or(60);
    let hi: usize = std::env::var("K52_HI").ok().and_then(|s| s.parse().ok()).unwrap_or(130);
    let mut max_ok = 0usize;
    let step: usize = std::env::var("K52_STEP").ok().and_then(|s| s.parse().ok()).unwrap_or(1);
    for du in (lo..=hi).step_by(step) {
        let r = trial(du);
        if r.0.starts_with("ok") { max_ok = du; }
        println!("K52 du={du} write={} | unlink2={} unlink3={} unlink1={} | write_after={}", r.0, r.1, r.2, r.3, r.4);
    }
    println!("K52 max_ok_write_du={max_ok}");
}
