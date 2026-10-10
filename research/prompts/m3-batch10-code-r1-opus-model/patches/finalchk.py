import sys
p=sys.argv[1]; s=open(p).read()
old="""                if wants_enum {
                    fx_enumerate(&start.label, n, &name, j, &crashed, run, &judge);
                }"""
assert s.count(old)==1
new="""                // 第 10 批攻方加（K1、K3）：FX_FINALCHK=1 时对这次挂载（与之后覆盖写）全部落盘的那一个状态跑池级 checker，报判红的不变量与 I-7.9 原文。
                if first && std::env::var("FX_FINALCHK").is_ok_and(|v| v == "1") {
                    let all: Vec<bool> = vec![true; run.writes.len()];
                    let final_image = materialize(&crashed, &run.writes, &all);
                    let mut red: Vec<String> = Vec::new();
                    let mut i79 = String::new();
                    for (inv, verdict) in singlefs_checker::walk::check_pool_image(&final_image) {
                        if let singlefs_checker::image::InvariantVerdict::Violated(text) = verdict {
                            red.push(inv.to_string());
                            if inv == "I-7.9" || inv == "I-3.1" { i79.push_str(&format!(" [{inv}] {}", text.chars().take(420).collect::<String>())); }
                        }
                    }
                    println!("FXFINAL {} n={n} j={j} plan={name} red={red:?}{i79}", start.label);
                }
                if wants_enum {
                    fx_enumerate(&start.label, n, &name, j, &crashed, run, &judge);
                }"""
s=s.replace(old,new)
open(p+'.new','w').write(s)
print('ok')
