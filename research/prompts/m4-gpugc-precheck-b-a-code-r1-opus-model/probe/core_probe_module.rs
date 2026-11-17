
/// 乙甲代码轮第一轮攻方腿的探针（只拼进冻结副本的一份拷贝，不进仓）：
/// 按线程开关把抬 F 之前先算整个关掉（放不下的盘那一判与推串之前合取 7 那一判都不判出，被拒就推，O 那一臂），
/// 记下推串之前合取 7 那一判每一次算出来判没判出（开关关着时照算照记、只是不交），
/// 记下每一串推成的抬 F 前后每块盘的全空聚簇段数（合取 7 读的那一个，`DeviceFreeMap::empty_segments`）。
pub mod refusal_precheck_oracle_probe {
    use crate::allocator::PoolAllocator;
    use std::cell::{Cell, RefCell};

    /// 一串推成的抬 F：推之前、推之后每块盘的全空聚簇段数（按分配器里盘的次序）。
    pub type EmptyClusterSegmentsAroundOneSeries = (Vec<u64>, Vec<u64>);

    thread_local! {
        static PRECHECK_SWITCHED_OFF: Cell<bool> = const { Cell::new(false) };
        static CONJUNCT_BEFORE_ANY_SERIES: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
        static SERIES: RefCell<Vec<EmptyClusterSegmentsAroundOneSeries>> = const { RefCell::new(Vec::new()) };
    }

    pub fn switch_off_the_precheck(switched_off: bool) {
        PRECHECK_SWITCHED_OFF.with(|cell| cell.set(switched_off));
    }

    pub fn precheck_is_switched_off() -> bool {
        PRECHECK_SWITCHED_OFF.with(Cell::get)
    }

    pub fn note_conjunct_before_any_series(found_below_the_floor: bool) {
        CONJUNCT_BEFORE_ANY_SERIES.with(|seen| seen.borrow_mut().push(found_below_the_floor));
    }

    pub fn take_conjunct_before_any_series() -> Vec<bool> {
        CONJUNCT_BEFORE_ANY_SERIES.with(|seen| std::mem::take(&mut *seen.borrow_mut()))
    }

    pub fn empty_cluster_segments_of(allocator: &PoolAllocator) -> Vec<u64> {
        allocator
            .devices
            .iter()
            .map(|device_map| device_map.empty_segments())
            .collect()
    }

    pub fn note_series(before: &[u64], allocator: &PoolAllocator) {
        let after = empty_cluster_segments_of(allocator);
        SERIES.with(|seen| seen.borrow_mut().push((before.to_vec(), after)));
    }

    pub fn take_series() -> Vec<EmptyClusterSegmentsAroundOneSeries> {
        SERIES.with(|seen| std::mem::take(&mut *seen.borrow_mut()))
    }
}
