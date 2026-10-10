import sys
p=sys.argv[1]; s=open(p).read()
def rep(old,new,count=1):
    global s
    n=s.count(old)
    assert n==count, (old[:80], n)
    s=s.replace(old,new)
rep("""    err: bool,
    persist: bool,
}

impl FaultPlan {""","""    err: bool,
    persist: bool,
    /// 第 10 批攻方加：同一次挂载里另几段读坏（各自的窗口、选槽法、读法），与主段取并。
    extra: Vec<FaultPlan>,
}

impl FaultPlan {""")
rep("""        FaultPlan { filters: [SlotFilter::Clean, SlotFilter::Clean], from: 0, to: u64::MAX, err: false, persist: false }""",
    """        FaultPlan { filters: [SlotFilter::Clean, SlotFilter::Clean], from: 0, to: u64::MAX, err: false, persist: false, extra: Vec::new() }""")
rep("""        format!("{window}:{}/{}:{}{}", self.filters[0].short(), self.filters[1].short(), if self.err { "e" } else { "z" }, if self.persist { ":p" } else { "" })""",
    """        let mut text = format!("{window}:{}/{}:{}{}", self.filters[0].short(), self.filters[1].short(), if self.err { "e" } else { "z" }, if self.persist { ":p" } else { "" });
        for x in &self.extra { text.push('+'); text.push_str(&x.name()); }
        text""")
rep("""        FaultPlan { filters: [SlotFilter::parse(f[0]), SlotFilter::parse(f[1])], from, to, err: parts[2] == "e", persist: parts.get(3) == Some(&"p") }""",
    """        FaultPlan { filters: [SlotFilter::parse(f[0]), SlotFilter::parse(f[1])], from, to, err: parts[2] == "e", persist: parts.get(3) == Some(&"p"), extra: Vec::new() }""")
rep("""    selected: bool,
}""","""    selected: bool,
    extra_selected: Vec<bool>,
}""")
old_start = s.index("        let (fault, err) = {\n            let mut s = self.shared.lock()")
old_end = s.index("        Ok(())\n    }\n    fn write_at", old_start)
s = s[:old_start] + """        let (fault, err, zero_stage) = {
            let mut s = self.shared.lock().expect("锁");
            let index = s.counter;
            s.counter += 1;
            let starts_a_pass = s.log.last().is_none_or(|last| hits[0].ordinal <= last.1);
            s.log.push((self.device, hits[0].ordinal));
            if starts_a_pass && std::env::var("FX_TRACE").is_ok_and(|v| v == "1") {
                let trace = format!("{}", std::backtrace::Backtrace::force_capture());
                let frames: Vec<String> = trace.lines().filter(|l| l.contains("singlefs_core::") && !l.contains("block_device")).map(|l| l.trim().split("::").skip(1).collect::<Vec<_>>().join("::")).take(6).collect();
                println!("FXTRACE pass_start={index} dev={} {}", self.device, frames.join(" <- "));
            }
            let base = s.active && index >= s.plan.from && index < s.plan.to && hits.iter().any(|e| e.selected);
            let extra_hit: Option<(usize, bool)> = if base { None } else {
                s.plan.extra.iter().enumerate().find(|(k, x)| s.active && index >= x.from && index < x.to && hits.iter().any(|e| e.extra_selected.get(*k).copied().unwrap_or(false))).map(|(k, x)| (k, x.err))
            };
            let fault = base || extra_hit.is_some();
            if fault {
                s.faulted_reads += 1;
            }
            let err = if base { s.plan.err } else { extra_hit.is_some_and(|(_, e)| e) };
            (fault, err, extra_hit.map(|(k, _)| k))
        };
        if fault && err {
            return Err(singlefs_core::block_device::BlockDeviceError::InputOutput(std::io::Error::other("注入的读错")));
        }
        self.inner.read_at(offset, buffer)?;
        if fault {
            for e in hits.iter().filter(|e| match zero_stage { None => e.selected, Some(k) => e.extra_selected.get(k).copied().unwrap_or(false) }) {
                let a = e.offset.saturating_sub(offset.0) as usize;
                let b = ((e.offset + e.len - offset.0).min(length)) as usize;
                buffer[a..b].fill(0);
            }
        }
""" + s[old_end:]
rep("""                e.selected = false;
            }""","""                e.selected = false;
                e.extra_selected.iter_mut().for_each(|x| *x = false);
            }""",2)
rep("""ordinal, txg, selected: false });""","""ordinal, txg, selected: false, extra_selected: Vec::new() });""")
rep("""            let entries = ring.get(&id.0).map(|e| select(e, plan.filters[id.0 as usize])).unwrap_or_default();""",
    """            let entries = ring.get(&id.0).map(|e| {
                let mut base = select(e, plan.filters[id.0 as usize]);
                for x in &plan.extra {
                    let sel = select(e, x.filters[id.0 as usize]);
                    for (b, s2) in base.iter_mut().zip(sel.iter()) { b.extra_selected.push(s2.selected); }
                }
                base
            }).unwrap_or_default();""")
rep("""        for name in which.split(';').filter(|x| !x.is_empty()) {
            // 「P<i>:<选槽>:<读法>」：第 i 遍整遍读（按这一刀不注错那一次挂载的边界）。
            if let Some(rest) = name.strip_prefix('P') {
                let (i, tail) = rest.split_once(':').expect("P<i>:…");
                let i: usize = i.parse().expect("遍号");
                if i + 1 < boundaries.len() {
                    plans.push(FaultPlan::parse(&format!("r{}-{}:{tail}", boundaries[i], boundaries[i + 1])));
                }
                continue;
            }
            plans.push(FaultPlan::parse(name));
        }""","""        for whole in which.split(';').filter(|x| !x.is_empty()) {
            // 「P<i>:<选槽>:<读法>」：第 i 遍整遍读（按这一刀不注错那一次挂载的边界）。几段用「+」连起来（第 10 批攻方加）。
            let mut stages: Vec<FaultPlan> = Vec::new();
            let mut skip = false;
            for name in whole.split('+') {
                if let Some(rest) = name.strip_prefix('P') {
                    let (i, tail) = rest.split_once(':').expect("P<i>:…");
                    let i: usize = i.parse().expect("遍号");
                    if i + 1 < boundaries.len() {
                        stages.push(FaultPlan::parse(&format!("r{}-{}:{tail}", boundaries[i], boundaries[i + 1])));
                    } else {
                        skip = true;
                    }
                    continue;
                }
                stages.push(FaultPlan::parse(name));
            }
            if skip || stages.is_empty() { continue; }
            let mut first = stages.remove(0);
            first.extra = stages;
            plans.push(first);
        }""")
s = s.replace("from: 0, to: u64::MAX, err, persist: false });", "from: 0, to: u64::MAX, err, persist: false, extra: Vec::new() });")
s = s.replace("from: 0, to: u64::MAX, err, persist: true });", "from: 0, to: u64::MAX, err, persist: true, extra: Vec::new() });")
s = s.replace("from: boundaries[a], to: boundaries[b], err, persist: false });", "from: boundaries[a], to: boundaries[b], err, persist: false, extra: Vec::new() });")
open(p+'.new','w').write(s)
print("patched")
