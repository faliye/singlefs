import sys
p=sys.argv[1]; s=open(p).read()
old="""    // 丢一块盘之后的结局只看剩下那块盘："""
assert s.count(old)==1
new="""    // 第 10 批攻方加（K3）：FX_I79=1 时把 I-7.9 判红的状态的 checker 原文打出前三条。从最后一个状态往前查（抬 F 的根落盘的状态在后面），
    // 14 线程分段查，凑满三条就停；最多查 FX_I79_MAX 个（默认 4000）状态，查完都没有就报一行 FXI79NONE。
    if std::env::var("FX_I79").is_ok_and(|v| v == "1") {
        let limit = env_u64("FX_I79_MAX", 4000) as usize;
        let order: Vec<usize> = (0..persisted_of_each.len()).rev().take(limit).collect();
        let found = std::sync::atomic::AtomicUsize::new(0);
        let messages: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
        let threads = env_u64("JUDGE_THREADS", 14).max(1) as usize;
        let chunk = order.len().div_ceil(threads).max(1);
        std::thread::scope(|scope| {
            for part in order.chunks(chunk) {
                let (found, messages, persisted_of_each) = (&found, &messages, &persisted_of_each);
                scope.spawn(move || {
                    for state_index in part {
                        if found.load(std::sync::atomic::Ordering::Relaxed) >= 3 { return; }
                        let persisted = &persisted_of_each[*state_index];
                        let state = materialize(crashed, w2, persisted);
                        for (name, verdict) in singlefs_checker::walk::check_pool_image(&state) {
                            if name == "I-7.9" {
                                if let singlefs_checker::image::InvariantVerdict::Violated(text) = verdict {
                                    let persisted_count = persisted.iter().filter(|p| **p).count();
                                    found.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    messages.lock().expect("锁").push(format!("state={state_index} persisted_writes={persisted_count} {text}"));
                                }
                            }
                        }
                    }
                });
            }
        });
        let messages = messages.into_inner().expect("锁");
        if messages.is_empty() {
            println!("FXI79NONE {label} n={n} plan={plan_name} checked_states={}", order.len());
        }
        for m in messages.iter().take(3) {
            println!("FXI79 {label} n={n} plan={plan_name} {m}");
        }
    }
    // 丢一块盘之后的结局只看剩下那块盘："""
s=s.replace(old,new)
open(p+'.new','w').write(s)
print('ok')
