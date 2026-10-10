import json
F='crates/singlefs-harness/src/gpu_memory_ledger.rs'
C="-p singlefs-harness --lib -- gpu_memory_ledger::tests"
rows=[]
rows.append(("G1 列出来的借据不看它等的是哪一台就销",F,
"            if *listed_on_the_device == device\n                && self\n                    .leases_kept_after_a_failed_wait\n                    .remove(listed_lease)\n                    .is_some()",
"            if { let _ = (listed_on_the_device, device); true }\n                && self\n                    .leases_kept_after_a_failed_wait\n                    .remove(listed_lease)\n                    .is_some()", C))
rows.append(("G2 换下来的旧设备在 started 锁里丢（改回拆锁之前）",F,
"        let replaced = self\n            .started\n            .lock()\n            .unwrap_or_else(PoisonError::into_inner)\n            .replace(device.clone());\n        drop(replaced);",
"        *self.started.lock().unwrap_or_else(PoisonError::into_inner) = Some(device.clone());", C))
rows.append(("G3 started_now 起着时交 None",F,
"    pub fn started_now(&self) -> Option<Device> {\n        self.clone_of_the_device_kept()",
"    pub fn started_now(&self) -> Option<Device> {\n        if self.starting.try_lock().is_err() { return None; }\n        self.clone_of_the_device_kept()", C))
rows.append(("G4 take_in_that 不核卡位置",F,
"        assert_eq!(\n            listed.card_position, self.card_position,",
"        assert_eq!(\n            self.card_position, self.card_position,", C))
rows.append(("G5 列卡带出的是此刻全部借据而不是列卡那一刻（克隆换成引用同一份：列卡时多列一个哨兵借据）",F,
"                    leases_kept_on_each_device: books_of_the_card\n                        .leases_kept_after_a_failed_wait\n                        .clone(),",
"                    leases_kept_on_each_device: BTreeMap::new(),", C))
json.dump(rows,open('extra-gpu.json','w'),ensure_ascii=False)
