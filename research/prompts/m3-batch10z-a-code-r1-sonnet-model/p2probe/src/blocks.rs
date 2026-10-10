// 从冻结副本 crash_amplification.rs 第 2048-2106 行逐字抄来（去掉 cfg 属性），不起卡、不依赖 checker 档。
pub struct BlocksForTheCards {
    pending_blocks: usize,
    next_pending: std::sync::atomic::AtomicUsize,
    given_back: std::sync::Mutex<Vec<usize>>,
}

impl BlocksForTheCards {
    pub fn new(pending_blocks: usize) -> Self {
        Self {
            pending_blocks,
            next_pending: std::sync::atomic::AtomicUsize::new(0),
            given_back: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// 领一块：先领交回的（序号小的先），没有就从队头领；都领完了是 `None`。
    pub fn take_a_block(&self) -> Option<usize> {
        // 持锁只为取一个序号，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
        let mut given_back = self
            .given_back
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(smallest) = given_back.iter().copied().min() {
            given_back.retain(|pending_index| *pending_index != smallest);
            return Some(smallest);
        }
        drop(given_back);
        let pending_index = self
            .next_pending
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        (pending_index < self.pending_blocks).then_some(pending_index)
    }

    /// 领了没判完的一块交回队里（报了 Out of Memory 的卡交）。
    pub fn give_back(&self, pending_index: usize) {
        self.given_back
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(pending_index);
    }

    /// 还有没领走的块没有（队里的或交回的）。
    pub fn any_left(&self) -> bool {
        self.next_pending.load(std::sync::atomic::Ordering::SeqCst) < self.pending_blocks
            || !self
                .given_back
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty()
    }

    /// 从队头领走了几块（不算交回的；序号连着、从 0 起）。
    pub fn taken_from_the_head(&self) -> usize {
        self.next_pending
            .load(std::sync::atomic::Ordering::SeqCst)
            .min(self.pending_blocks)
    }
}
