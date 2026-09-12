import re, sys
p = sys.argv[1]
s = open(p, encoding='utf-8').read()
def sub1(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:90])
    s = s.replace(old, new)

# 1) Simulation 加锚字段
sub1("""    /// 管理员回退之前盘上那一份分配状态，被抛弃时间线的根引用的是它（E135 的 abandoned_view 原样）。
    abandoned_view: Option<(Rc<PoolState>, u64)>,
}""",
"""    /// 管理员回退之前盘上那一份分配状态，被抛弃时间线的根引用的是它（E135 的 abandoned_view 原样）。
    abandoned_view: Option<(Rc<PoolState>, u64)>,
    /// G3A 探针：钉住的锚根（住在环之外的专用槽里，不参与轮转）。空 ⇒ 与 E139 原样一致。
    anchor_roots: Vec<RootRecord>,
    anchor_mode: bool,
    /// 更正 #3 的开关：false ⇒ 活着的元数据不记回保留池（登记里的原始口径）。
    credits_live_metadata: bool,
    /// 探针：按状态数读的上限在「非空有效根不足 4 个」那条分支上被取到几次。
    nonempty_fallback_hits: u64,
}""")

sub1("""            metrics: Metrics::fresh(),
            abandoned_view: None,
        }
    }""",
"""            metrics: Metrics::fresh(),
            abandoned_view: None,
            anchor_roots: Vec::new(),
            anchor_mode: false,
            credits_live_metadata: true,
            nonempty_fallback_hits: 0,
        }
    }""")

# 2) fallback 计数探针（不改行为）
sub1("""        if distinct_newest_first.len() >= MINIMUM_RETAINED_ROOTS {
            distinct_newest_first[MINIMUM_RETAINED_ROOTS - 1]
        } else {
            self.oldest_persisted_valid_txg()
        }
    }""",
"""        if distinct_newest_first.len() >= MINIMUM_RETAINED_ROOTS {
            distinct_newest_first[MINIMUM_RETAINED_ROOTS - 1]
        } else {
            self.oldest_persisted_valid_txg()
        }
    }

    /// 探针：上一句走没走「非空有效根不足 4 个」那条分支（第一次运行之后改的就是这一条）。
    fn is_nonempty_fallback_taken(&self) -> bool {
        self.valid_roots().iter().map(|root| root.txg).filter(|txg| self.nonempty_txgs.contains(txg))
            .collect::<BTreeSet<u64>>().len() < MINIMUM_RETAINED_ROOTS
    }

    /// 最新的非空有效根（G3A 的上限用它：锚给 3 个状态，环上只需再留 1 个）。
    fn newest_nonempty_valid_txg(&self) -> u64 {
        self.valid_roots().iter().map(|root| root.txg).filter(|txg| self.nonempty_txgs.contains(txg)).max().unwrap_or(0)
    }

    /// 有没有一个锚根落在这块的生命区间 [分配代, 释放代) 里。
    fn anchor_covers(&self, allocated_at: u64, freed_at: u64) -> bool {
        self.anchor_roots.iter().any(|root| root.txg >= allocated_at && root.txg < freed_at)
    }

    fn anchor_pinned_count(&self) -> u64 {
        if !self.anchor_mode { return 0; }
        let bound = self.reuse_bound();
        self.pool.blocks.iter().filter(|state| match state {
            BlockState::Unused | BlockState::Live { .. } => false,
            BlockState::Freed { allocated_at, freed_at } => *freed_at <= bound && self.anchor_covers(*allocated_at, *freed_at),
        }).count() as u64
    }""")

# 3) floor_upper_limit：anchor_mode 走 min(每盘最新, 最新非空)
sub1("""    fn floor_upper_limit(&self) -> u64 {
        match self.arm {""",
"""    fn floor_upper_limit(&self) -> u64 {
        if self.anchor_mode {
            return self.newest_nonempty_valid_txg().min(self.newest_valid_txg_on_every_surviving_disk());
        }
        match self.arm {""")

# 4) disposal_target：anchor_mode 不按第 4 新非空根封顶
sub1("""    fn disposal_target(&self, goal_txg: u64) -> u64 {
        if self.arm.counts_states() {""",
"""    fn disposal_target(&self, goal_txg: u64) -> u64 {
        if self.anchor_mode {
            return goal_txg;
        }
        if self.arm.counts_states() {""")

# 5) lag_blocks：anchor_mode 为 0
sub1("""    fn lag_blocks(&self) -> u64 {
        if self.arm.counts_states() {""",
"""    fn lag_blocks(&self) -> u64 {
        if self.anchor_mode {
            return 0;
        }
        if self.arm.counts_states() {""")

# 6) 可再分配：加锚区间条件（三处：reusable_count / allocate / pinned_count）
sub1("""    fn reusable_count(&self) -> u64 {
        let bound = self.reuse_bound();
        self.pool.blocks.iter().filter(|state| match state {
            BlockState::Unused => true,
            BlockState::Live { .. } => false,
            BlockState::Freed { freed_at, .. } => *freed_at <= bound,
        }).count() as u64
    }""",
"""    fn reusable_count(&self) -> u64 {
        let bound = self.reuse_bound();
        self.pool.blocks.iter().filter(|state| match state {
            BlockState::Unused => true,
            BlockState::Live { .. } => false,
            BlockState::Freed { allocated_at, freed_at } => *freed_at <= bound && !self.anchor_covers(*allocated_at, *freed_at),
        }).count() as u64
    }""")

sub1("""    fn pinned_count(&self) -> u64 {
        self.count_freed_after(self.reuse_bound())
    }""",
"""    fn pinned_count(&self) -> u64 {
        let bound = self.reuse_bound();
        self.pool.blocks.iter().filter(|state| match state {
            BlockState::Unused | BlockState::Live { .. } => false,
            BlockState::Freed { allocated_at, freed_at } => *freed_at > bound || self.anchor_covers(*allocated_at, *freed_at),
        }).count() as u64
    }""")

sub1("""        let mut chosen: Vec<usize> = self.pool.blocks.iter().enumerate()
            .filter(|(_, state)| matches!(state, BlockState::Freed { freed_at, .. } if *freed_at <= bound))
            .map(|(index, _)| index).take(wanted).collect();""",
"""        let anchor_txgs: Vec<u64> = self.anchor_roots.iter().map(|root| root.txg).collect();
        let mut chosen: Vec<usize> = self.pool.blocks.iter().enumerate()
            .filter(|(_, state)| matches!(state, BlockState::Freed { allocated_at, freed_at }
                if *freed_at <= bound && !anchor_txgs.iter().any(|a| *a >= *allocated_at && *a < *freed_at)))
            .map(|(index, _)| index).take(wanted).collect();""")

# 7) 候选集与状态数：把锚算进去
sub1("""    fn count_fake_candidates(&self) -> (u64, u64, u64, u64) {
        let floor = self.candidacy_floor();
        let candidates: Vec<RootRecord> = self.valid_roots().into_iter().filter(|root| root.txg >= floor).collect();""",
"""    fn count_fake_candidates(&self) -> (u64, u64, u64, u64) {
        let floor = self.candidacy_floor();
        let mut candidates: Vec<RootRecord> = self.valid_roots().into_iter().filter(|root| root.txg >= floor).collect();
        for anchor in &self.anchor_roots {
            if !candidates.iter().any(|root| root.txg == anchor.txg) {
                candidates.push(*anchor);
            }
        }""")

# 8) live_metadata_credit 受开关控制
sub1("""    fn live_metadata_credit(&self) -> u64 {
        if self.arm.is_tightened() { self.pool.metadata.len() as u64 } else { 0 }
    }""",
"""    fn live_metadata_credit(&self) -> u64 {
        if self.arm.is_tightened() && self.credits_live_metadata { self.pool.metadata.len() as u64 } else { 0 }
    }""")

# 9) record_check 顺带记 fallback 命中
sub1("""        if self.is_retention_violated() {
            self.metrics.retention_violations += 1;
        }
    }""",
"""        if self.is_retention_violated() {
            self.metrics.retention_violations += 1;
        }
        if self.arm.counts_states() && !self.anchor_mode && self.is_nonempty_fallback_taken() {
            self.nonempty_fallback_hits += 1;
        }
    }""")

# 10) 锚版的预填：先跑三个建对象的窗口（锚 = txg 1/2/3 三个不同状态），再填到 fill，块的分配代都晚于锚
sub1("""    fn save_snapshot(&mut self) {""",
"""    /// G3A 的预填：先造三个不同的早期用户状态当锚（环外的专用槽），再把盘填到 fill。
    /// 填进去的块分配代都 > 最大锚 txg ⇒ 锚一块也不钉它们。
    fn prefill_anchored(&mut self, fill: f64) {
        for _ in 0..3 {
            self.run_user_window(false, false);
        }
        let anchor_txgs: Vec<u64> = vec![1, 2, 3];
        self.anchor_roots = self.ring.iter().flatten().filter(|root| anchor_txgs.contains(&root.txg)).copied().collect();
        assert_eq!(self.anchor_roots.len(), 3, "三个锚根都要在环里找得到");
        self.anchor_mode = true;
        let txg = self.next_txg;
        let target_live = (CAPACITY_BLOCKS as f64 * fill) as usize;
        let mut live_now = self.pool.blocks.iter().filter(|state| matches!(state, BlockState::Live { .. })).count();
        loop {
            let free_slots: Vec<usize> = self.pool.blocks.iter().enumerate()
                .filter(|(_, state)| matches!(state, BlockState::Unused)).map(|(index, _)| index)
                .take(USER_BLOCKS_PER_OBJECT).collect();
            if live_now + USER_BLOCKS_PER_OBJECT > target_live || free_slots.len() < USER_BLOCKS_PER_OBJECT {
                break;
            }
            let mut object = [0usize; USER_BLOCKS_PER_OBJECT];
            object.copy_from_slice(&free_slots);
            for &index in &object {
                self.pool.blocks[index] = BlockState::Live { allocated_at: txg };
            }
            self.pool.objects.push(object);
            live_now += USER_BLOCKS_PER_OBJECT;
        }
        let old_metadata = std::mem::take(&mut self.pool.metadata);
        self.free_blocks(&old_metadata, txg);
        let metadata = self.allocate(METADATA_BLOCKS_PER_PUBLICATION, txg).expect("预填之后写得下元数据");
        self.pool.metadata = metadata;
        self.current_window_changes_user_state = true;
        self.publish(txg);
    }

    fn save_snapshot(&mut self) {""")

open(p, 'w', encoding='utf-8').write(s)
print("ok")
