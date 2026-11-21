//! 几张卡一起判的那一队块：每张卡领块、Out of Memory 时交回，叫停之后只领判完的块等着的那些，按块序写库，
//! 几轮卡接着算到判完或停下。

use super::*;

/// 卡线程这一刻要不要停（每领一块之前问一次，[`judge_blocks_on_one_card`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopOfTheCards {
    /// 照常领。
    NotRequested,
    /// 调用方叫停：不领新的块，只领判完的块在等的那几块（比判完的块号小、交回队里的），判完的块照块序都写得进库、库里不留空洞；
    /// 没判的留给下一趟（收尾再起的那一轮里叫停，上一轮判完的块在等交回的那一块）。
    RequestedByTheCaller,
    /// 写库失败或有卡拒判（这一趟判红）：一块都不再领。
    TheRunFailed,
}

/// 几张卡一起判的那一队块（崩溃放量产品路径第 ① 段，`singlefs_verification::crash_amplification`；块号是要判的块在那一趟 `pending` 里的次序）：
/// 每张卡领序号最小的、还没领走的那一块；一张卡报了 Out of Memory 把手里那一块交回，交回的序号比没领过的都小，下一张来领的卡先领到它
/// （P2：一张卡 Out of Memory 不停整趟）。记着判完的块里块号最大的（几轮接着算）：调用方叫停之后
/// 只领比它小的（[`StopOfTheCards::RequestedByTheCaller`]）。不碰卡：单测不要卡就跑（文件头）。
pub struct BlocksForTheCards {
    queue: Mutex<QueueOfTheBlocks>,
}

struct QueueOfTheBlocks {
    blocks_not_taken: BTreeSet<usize>,
    /// 判完的块里块号最大的（[`judge_blocks_on_one_card`] 判完一块就记）：队里比它小的，判完的块在等。
    largest_judged: Option<usize>,
}

impl QueueOfTheBlocks {
    /// 队头那一块 `first` 此刻照 `stop` 领不领得。
    fn may_take(&self, first: usize, stop: StopOfTheCards) -> bool {
        match stop {
            StopOfTheCards::NotRequested => true,
            StopOfTheCards::RequestedByTheCaller => {
                self.largest_judged.is_some_and(|largest| first < largest)
            }
            StopOfTheCards::TheRunFailed => false,
        }
    }
}

impl BlocksForTheCards {
    /// 一队块：`pending_indices` 里的每一块，还没有判完的。
    #[must_use]
    pub fn new(pending_indices: impl IntoIterator<Item = usize>) -> Self {
        Self {
            queue: Mutex::new(QueueOfTheBlocks {
                blocks_not_taken: pending_indices.into_iter().collect(),
                largest_judged: None,
            }),
        }
    }

    fn queue_locked(&self) -> MutexGuard<'_, QueueOfTheBlocks> {
        // 持锁只为取、放一个序号，没有会 panic 到一半的写：别的线程 panic 留下的毒照用
        self.queue.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// 照 `stop` 领一块：没叫停领序号最小的、还没领走的（交回的也算）；调用方叫停了，那一块比判完的块号小（判完的块在等它）才领；
    /// 判红了不领。领不得或都领走了是 `None`。
    #[must_use]
    pub fn take_a_block(&self, stop: StopOfTheCards) -> Option<usize> {
        let mut queue = self.queue_locked();
        let first = queue.blocks_not_taken.first().copied()?;
        if queue.may_take(first, stop) {
            queue.blocks_not_taken.pop_first()
        } else {
            None
        }
    }

    /// 照 `stop` 此刻还有没有领得的块（起下一张卡之前问：没有就不起）。
    #[must_use]
    pub fn any_left_to_take(&self, stop: StopOfTheCards) -> bool {
        let queue = self.queue_locked();
        queue
            .blocks_not_taken
            .first()
            .is_some_and(|first| queue.may_take(*first, stop))
    }

    /// 判完了一块（交去写库之前记）。
    fn record_a_judged_block(&self, pending_index: usize) {
        let mut queue = self.queue_locked();
        queue.largest_judged = Some(
            queue
                .largest_judged
                .map_or(pending_index, |largest| largest.max(pending_index)),
        );
    }

    /// 领了没判完的一块交回队里（报了 Out of Memory 的卡交）。
    ///
    /// # Panics
    /// 这一块此刻就在队里（交了两次，或没领过）。
    pub fn give_back(&self, pending_index: usize) {
        let is_newly_given_back = self.queue_locked().blocks_not_taken.insert(pending_index);
        assert!(
            is_newly_given_back,
            "块 {pending_index} 交回时已经在队里：一块领一次只交回一次"
        );
    }

    /// 还有没领走的块没有（没领过的或交回的）。
    #[must_use]
    pub fn any_left(&self) -> bool {
        !self.queue_locked().blocks_not_taken.is_empty()
    }

    /// 此刻还没领走的块，按序号（几张卡都收工之后，就是这一轮没判的那几块）。
    #[must_use]
    pub fn blocks_not_taken(&self) -> Vec<usize> {
        self.queue_locked()
            .blocks_not_taken
            .iter()
            .copied()
            .collect()
    }

    /// 一轮卡都收工之后，下一轮的那一队：这一轮没领走的，连同判完的块里块号最大的（下一轮叫停了照样领判完的块在等的那几块）。
    #[must_use]
    pub fn left_for_another_round(&self) -> Self {
        let queue = self.queue_locked();
        Self {
            queue: Mutex::new(QueueOfTheBlocks {
                blocks_not_taken: queue.blocks_not_taken.clone(),
                largest_judged: queue.largest_judged,
            }),
        }
    }
}

/// 一张卡判一块的结局（[`judge_blocks_on_one_card`] 的 `judge_a_block` 交）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockJudgedOnACard<Judged> {
    Judged(Judged),
    /// 卡上 wgpu 报了 Out of Memory（`reason` 是本次未跑那一行的原因）：这一块交回，这张卡收工。
    OutOfMemory {
        reason: String,
    },
    /// 这张卡判不了这一块里的某个状态（表越过了内核的容量这类，`reason` 说哪一个）：这一块不交回，调用方叫所有卡停下。
    Refused {
        reason: String,
    },
}

/// 判完的一块交没交到写库那一头（[`judge_blocks_on_one_card`] 的 `hand_to_the_store` 交）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockHandedToTheStore {
    Handed,
    /// 写库那一头已经不收了（写库那条线程走了）。
    TheStoreIsGone,
}

/// 一张卡怎么收的工（[`judge_blocks_on_one_card`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CardLeftTheBlocks {
    /// 队里没有领得的了（领完了，或照叫停领不得）。
    NothingLeftOrStopped,
    /// 判一块时报了 Out of Memory：那一块已经交回队里。
    OutOfMemory { reason: String },
    /// 判不了一块：那一块没交回。
    Refused { reason: String },
    /// 判完的一块交不到写库那一头。
    TheStoreIsGone,
}

/// 一张卡的线程判块的那一圈（崩溃放量产品路径第 ① 段的卡线程，判器建好之后）：每领一块之前问 `stop_of_the_cards`、照它领一块
/// （[`BlocksForTheCards::take_a_block`]，叫停了只领判完的块在等的），交 `judge_a_block` 判；判完记下、交 `hand_to_the_store`
/// （写库按块序，[`BlocksStoredInOrder`]）；报了 Out of Memory 就把这一块交回队里、这张卡收工，不再领（只停这一张卡）；
/// 判不了就收工、这一块不交回。循环的次数不超过队里的块数加交回的次数（每次领走一块，每张卡至多交回一次）。
pub fn judge_blocks_on_one_card<Judged>(
    blocks: &BlocksForTheCards,
    stop_of_the_cards: &dyn Fn() -> StopOfTheCards,
    judge_a_block: &mut dyn FnMut(usize) -> BlockJudgedOnACard<Judged>,
    hand_to_the_store: &mut dyn FnMut(usize, Judged) -> BlockHandedToTheStore,
) -> CardLeftTheBlocks {
    while let Some(pending_index) = blocks.take_a_block(stop_of_the_cards()) {
        match judge_a_block(pending_index) {
            BlockJudgedOnACard::Judged(judged) => {
                blocks.record_a_judged_block(pending_index);
                match hand_to_the_store(pending_index, judged) {
                    BlockHandedToTheStore::Handed => {}
                    BlockHandedToTheStore::TheStoreIsGone => {
                        return CardLeftTheBlocks::TheStoreIsGone
                    }
                }
            }
            BlockJudgedOnACard::OutOfMemory { reason } => {
                blocks.give_back(pending_index);
                return CardLeftTheBlocks::OutOfMemory { reason };
            }
            BlockJudgedOnACard::Refused { reason } => return CardLeftTheBlocks::Refused { reason },
        }
    }
    CardLeftTheBlocks::NothingLeftOrStopped
}

/// 第 `position` 张卡备好了（或建判器时报了 Out of Memory、不判）之后起哪一张卡的线程（崩溃放量产品路径第 ① 段，卡一张一张备、不同时编内核）：
/// 后面第一张装了状态的卡（`states_placed` 按这一轮卡的次序，0 是没装），而且队里照此刻的叫停还有领得的块
/// （[`BlocksForTheCards::any_left_to_take`]：叫停了只剩判完的块不在等的，不起）；否则不起（
/// 这一步抽到这里，起下一张卡时看不看叫停不要卡就测得到）。
/// 叫停只在后面有装了状态的卡时才问 `stop_of_the_cards_now`、问一次，没有下一张卡一次都不问（
/// 抽出来那一版不管有没有下一张卡都先问，每起一张卡多问一次；调用方的叫停可以按「问第几次」变，多问一次卡上就少领一块）。
#[must_use]
pub fn next_card_to_start(
    states_placed: &[u64],
    position: usize,
    blocks: &BlocksForTheCards,
    stop_of_the_cards_now: impl FnOnce() -> StopOfTheCards,
) -> Option<usize> {
    let next_position =
        (position + 1..states_placed.len()).find(|candidate| states_placed[*candidate] > 0)?;
    if blocks.any_left_to_take(stop_of_the_cards_now()) {
        Some(next_position)
    } else {
        None
    }
}

/// 判完的块按块序写库（判定向量的编号按块序登记，与用了几张卡无关）：块号从 0 起连着，后面的块先到就等着，前面那一块到了一起写；
/// 写库失败之后到的不再写。几轮卡（[`after_a_round_of_the_cards`]）共用一份：上一轮判完、等着前面那一块的，下一轮那一块判完照块序一起写，
/// 不再判一遍。
pub struct BlocksStoredInOrder<Block> {
    blocks_stored: usize,
    waiting: BTreeMap<usize, Block>,
    storing_error: Option<String>,
}

/// 一块都没写、没有等着的就是起点，没有别的不变量。
impl<Block> Default for BlocksStoredInOrder<Block> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Block> BlocksStoredInOrder<Block> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            blocks_stored: 0,
            waiting: BTreeMap::new(),
            storing_error: None,
        }
    }

    /// 块号 `pending_index` 的一块判完到了：照块序交 `store_block` 写（它和前面连着的、等着的一起）；写失败记下那一句。
    pub fn arrive(
        &mut self,
        pending_index: usize,
        block: Block,
        store_block: &mut dyn FnMut(Block) -> Result<(), String>,
    ) {
        self.waiting.insert(pending_index, block);
        while let Some(next_block) = self.waiting.remove(&self.blocks_stored) {
            if self.storing_error.is_some() {
                continue;
            }
            match store_block(next_block) {
                Ok(()) => self.blocks_stored += 1,
                Err(error) => self.storing_error = Some(error),
            }
        }
    }

    /// 照块序写进库了几块（块号 `0..` 这几块）。
    #[must_use]
    pub fn blocks_stored(&self) -> usize {
        self.blocks_stored
    }

    /// 判完了、等着前面那一块的有几块。
    #[must_use]
    pub fn blocks_waiting(&self) -> usize {
        self.waiting.len()
    }

    /// 写库失败的那一句（没失败是 `None`）。
    #[must_use]
    pub fn storing_error(&self) -> Option<&str> {
        self.storing_error.as_deref()
    }
}

/// 调用方叫没叫停（一轮卡都收工之后读的那一次，[`RoundsOfTheCards::after_the_round`]；卡线程每领一块之前问的是 [`StopOfTheCards`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopOfTheRun {
    Requested,
    NotRequested,
}

/// 一趟里调用方叫没叫停，见过一次就记住（叫停是一个文件在不在，外层循环、卡线程、一轮收尾、
/// 整趟收尾各自现读时文件中途被删，四处读到的不一样，轮末断言 panic、或照叫停收尾而整趟没记成提早收尾）。这一趟里问叫停的每一处都问它；
/// 头一次读到叫停之后不再读 `read_the_request_now`，之后恒答叫停。几条线程一起问（卡线程与收写库那一条）。
pub struct StopRequestRememberedInThisRun<'reader> {
    read_the_request_now: &'reader (dyn Fn() -> bool + Sync),
    has_seen_the_request: AtomicBool,
}

impl<'reader> StopRequestRememberedInThisRun<'reader> {
    /// 这一趟开头：还没见过叫停，要问时照 `read_the_request_now` 现读（崩溃放量产品路径是叫停文件在不在）。
    #[must_use]
    pub fn new(read_the_request_now: &'reader (dyn Fn() -> bool + Sync)) -> Self {
        Self {
            read_the_request_now,
            has_seen_the_request: AtomicBool::new(false),
        }
    }

    /// 这一刻叫没叫停：这一趟里见过就是叫了；没见过就现读一次，读到叫停记下。
    #[must_use]
    pub fn is_requested(&self) -> bool {
        if self.has_seen_the_request.load(Ordering::SeqCst) {
            return true;
        }
        if (self.read_the_request_now)() {
            self.has_seen_the_request.store(true, Ordering::SeqCst);
            return true;
        }
        false
    }

    /// 同 [`Self::is_requested`]，写成一轮收尾交 [`RoundsOfTheCards::after_the_round`] 的那一种。
    #[must_use]
    pub fn stop_of_the_run(&self) -> StopOfTheRun {
        if self.is_requested() {
            StopOfTheRun::Requested
        } else {
            StopOfTheRun::NotRequested
        }
    }
}

/// 一轮卡都收工之后这一趟怎么走（[`after_a_round_of_the_cards`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AfterARoundOfTheCards {
    /// 要判的块都写进了库。
    EveryBlockStored,
    /// 被叫停，判完的块都写进了库（没有在等的），没判的留给下一趟；这一轮有卡报了 Out of Memory 也一样（
    /// 交回的块是卡上领走之前报的还是之后报的，结局不随先后变）。
    StoppedOnRequest,
    /// 这一轮有卡报了 Out of Memory、还有块没写、没被叫停，而还有没报过的卡：收尾这一步再起一轮，只用没报过的卡判剩下的块
    /// （第 5 条：交回的块在别的卡都收工之后也要有人判，状态全装在第一张卡时也要起第二张）。
    AnotherRoundWithoutTheCardsShortOfMemory,
    /// 被叫停，这一轮有卡报了 Out of Memory、交回的块前面判完的块在等它，而还有没报过的卡：再起一轮，只用没报过的卡判判完的块在等的那几块
    /// （叫停照旧：卡只领那几块，[`StopOfTheCards::RequestedByTheCaller`]），判完的都入库再照叫停收尾。
    AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor,
    /// 有卡报了 Out of Memory、还有块没写，而报过的卡之外没有卡了（被叫停的，是判完的块在等的那几块没有卡能判）：这一趟报本次未跑，
    /// 写进库的块在库里。
    NotRunTheGpuMemoryWasShort,
}

/// 一轮卡都收工之后：`blocks_left_unstored` 块还没写进库（共要判的减照块序写进的，[`BlocksStoredInOrder::blocks_stored`]），其中
/// `blocks_waiting` 块判完了在等前面的块，这一轮 `cards_short_of_memory_this_round` 张卡报了 Out of Memory，到这一轮为止没报过的卡还有
/// `cards_not_short_of_memory` 张。每再起一轮至少少一张卡，轮数不超过卡数。
#[must_use]
pub fn after_a_round_of_the_cards(
    blocks_left_unstored: usize,
    blocks_waiting: usize,
    cards_short_of_memory_this_round: usize,
    stop: StopOfTheRun,
    cards_not_short_of_memory: usize,
) -> AfterARoundOfTheCards {
    if blocks_left_unstored == 0 {
        return AfterARoundOfTheCards::EveryBlockStored;
    }
    if cards_short_of_memory_this_round == 0 {
        return AfterARoundOfTheCards::StoppedOnRequest;
    }
    match stop {
        StopOfTheRun::Requested if blocks_waiting == 0 => AfterARoundOfTheCards::StoppedOnRequest,
        StopOfTheRun::Requested if cards_not_short_of_memory == 0 => {
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort
        }
        StopOfTheRun::Requested => {
            AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor
        }
        StopOfTheRun::NotRequested if cards_not_short_of_memory == 0 => {
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort
        }
        StopOfTheRun::NotRequested => {
            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory
        }
    }
}

/// 第 ① 段几轮卡之间带着的（崩溃放量产品路径，`singlefs_verification::crash_amplification` 的外层循环；不碰卡，单测不要卡就跑）：
/// 按块序写库的那一份（几轮共用：上一轮判完、等着前面那一块的，下一轮那一块判完照块序一起写）、下一轮的那一队块、报过 Out of Memory 的卡
/// 与头一句、收过工的卡。一轮怎么走：[`Self::take_the_blocks_of_the_round`] 交出这一轮的那一队，卡上的线程判、写库那条线程照
/// [`Self::stored_in_order`] 写，卡都收工之后交 [`Self::after_the_round`]。
pub struct RoundsOfTheCards<Block> {
    pending_blocks: usize,
    stored_in_order: BlocksStoredInOrder<Block>,
    blocks_of_the_next_round: BlocksForTheCards,
    cards_short_of_memory: BTreeSet<u32>,
    first_out_of_memory: Option<String>,
    cards_that_finished: BTreeSet<u32>,
}

impl<Block> RoundsOfTheCards<Block> {
    /// 头一轮之前：`pending_blocks` 块要判（块号 `0..pending_blocks`），一张卡都没用过。
    #[must_use]
    pub fn new(pending_blocks: usize) -> Self {
        Self {
            pending_blocks,
            stored_in_order: BlocksStoredInOrder::new(),
            blocks_of_the_next_round: BlocksForTheCards::new(0..pending_blocks),
            cards_short_of_memory: BTreeSet::new(),
            first_out_of_memory: None,
            cards_that_finished: BTreeSet::new(),
        }
    }

    /// 这张卡（`card_index` 是 `nvidia-smi` 给它的序号）这一趟报过 Out of Memory 没有：报过的不再用（每再起一轮至少少一张卡）。
    #[must_use]
    pub fn is_short_of_memory(&self, card_index: u32) -> bool {
        self.cards_short_of_memory.contains(&card_index)
    }

    /// 这一趟报过 Out of Memory 的卡（序号，进度行用）。
    #[must_use]
    pub fn cards_short_of_memory(&self) -> &BTreeSet<u32> {
        &self.cards_short_of_memory
    }

    /// 下一轮要判的块号（按序号）。
    #[must_use]
    pub fn blocks_of_the_next_round(&self) -> Vec<usize> {
        self.blocks_of_the_next_round.blocks_not_taken()
    }

    /// 交出这一轮的那一队（一轮一次；交出之后这里留一队空的，[`Self::after_the_round`] 换上这一轮剩下的）。
    #[must_use]
    pub fn take_the_blocks_of_the_round(&mut self) -> BlocksForTheCards {
        std::mem::replace(
            &mut self.blocks_of_the_next_round,
            BlocksForTheCards::new([]),
        )
    }

    /// 写库那条线程照它写（几轮共用一份）。
    pub fn stored_in_order(&mut self) -> &mut BlocksStoredInOrder<Block> {
        &mut self.stored_in_order
    }

    /// 照块序写进库了几块。
    #[must_use]
    pub fn blocks_stored(&self) -> usize {
        self.stored_in_order.blocks_stored()
    }

    /// 判完了、等着前面那一块的有几块。
    #[must_use]
    pub fn blocks_waiting(&self) -> usize {
        self.stored_in_order.blocks_waiting()
    }

    /// 这一趟头一张报 Out of Memory 的卡报的那一句（没有卡报过是 `None`）。
    #[must_use]
    pub fn first_out_of_memory(&self) -> Option<&str> {
        self.first_out_of_memory.as_deref()
    }

    /// 照叫停收尾（[`AfterARoundOfTheCards::StoppedOnRequest`]）时要打的那一行：这一趟有卡报过 Out of Memory，头一句照「本次未跑」那一行的
    /// 形态（[`gpu_not_run_line`]，`run_name` 是那一格的名字）；没有卡报过是 `None`（叫停加有卡报
    /// Out of Memory 而照叫停收尾时，这一行不打，显存不够的缘由外面读不到）。
    #[must_use]
    pub fn not_run_line_of_a_stop_after_an_out_of_memory(&self, run_name: &str) -> Option<String> {
        self.first_out_of_memory
            .as_deref()
            .map(|reason| gpu_not_run_line(run_name, reason))
    }

    /// 收过工的卡有几张：一张卡在几轮里各收工一次只算一张（`cards_used` 算两遍）。
    #[must_use]
    pub fn cards_used(&self) -> usize {
        self.cards_that_finished.len()
    }

    /// 一轮卡都收工（写库没出错、没有卡拒判）之后：`blocks` 是这一轮的那一队，`cards_of_the_round` 是这一轮挑卡时没报过的卡（装没装状态都算），
    /// `cards_finished` 是这一轮收了工的卡，`cards_out_of_memory` 是这一轮报了 Out of Memory 的卡与那一句（按收到的次序），`stop` 是这一刻
    /// 调用方叫没叫停。记下报了的卡、收工的卡，换上下一轮的那一队（[`BlocksForTheCards::left_for_another_round`]），照
    /// [`after_a_round_of_the_cards`] 交这一趟怎么走。
    ///
    /// # Panics
    /// 这一轮没有卡报 Out of Memory，却还有判完的块在等前面的块（判完的块在等的那几块卡都该领了，叫停了也一样），或没人叫停却还有块没领。
    pub fn after_the_round(
        &mut self,
        blocks: &BlocksForTheCards,
        cards_of_the_round: &[u32],
        cards_finished: impl IntoIterator<Item = u32>,
        cards_out_of_memory: &[(u32, String)],
        stop: StopOfTheRun,
    ) -> AfterARoundOfTheCards {
        self.cards_that_finished.extend(cards_finished);
        if cards_out_of_memory.is_empty() {
            assert_eq!(
                self.stored_in_order.blocks_waiting(),
                0,
                "没有卡报 Out of Memory：判完的块每一块都写进了库（叫停了卡也领判完的块在等的那几块）"
            );
            assert!(
                !blocks.any_left() || stop == StopOfTheRun::Requested,
                "没人叫停，却有 {} 块没判",
                blocks.blocks_not_taken().len()
            );
        }
        for (index, reason) in cards_out_of_memory {
            self.cards_short_of_memory.insert(*index);
            self.first_out_of_memory
                .get_or_insert_with(|| reason.clone());
        }
        self.blocks_of_the_next_round = blocks.left_for_another_round();
        let cards_not_short_of_memory = cards_of_the_round
            .iter()
            .filter(|index| !self.cards_short_of_memory.contains(index))
            .count();
        after_a_round_of_the_cards(
            self.pending_blocks - self.stored_in_order.blocks_stored(),
            self.stored_in_order.blocks_waiting(),
            cards_out_of_memory.len(),
            stop,
            cards_not_short_of_memory,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        after_a_round_of_the_cards, judge_blocks_on_one_card, next_card_to_start,
        AfterARoundOfTheCards, BlockHandedToTheStore, BlockJudgedOnACard, BlocksForTheCards,
        BlocksStoredInOrder, CardLeftTheBlocks, RoundsOfTheCards, StopOfTheCards, StopOfTheRun,
        StopRequestRememberedInThisRun,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    /// （P2）：几张卡一起领的那一队块，报了 Out of Memory 的卡交回的那一块先给下一张来领的卡；
    /// 交回的都领走、队里也领完才算没有剩下的。
    #[test]
    fn block_given_back_by_a_card_out_of_memory_goes_to_the_next_card_that_takes_one() {
        let go_on = StopOfTheCards::NotRequested;
        let blocks = BlocksForTheCards::new(0..3);
        assert_eq!(blocks.take_a_block(go_on), Some(0));
        assert_eq!(blocks.take_a_block(go_on), Some(1));
        blocks.give_back(0);
        assert!(blocks.any_left());
        assert_eq!(blocks.take_a_block(go_on), Some(0), "交回的先领");
        assert_eq!(blocks.take_a_block(go_on), Some(2));
        assert_eq!(blocks.take_a_block(go_on), None);
        assert!(!blocks.any_left());
        blocks.give_back(1);
        assert!(blocks.any_left(), "队里领完了、还有交回的");
        assert_eq!(blocks.blocks_not_taken(), vec![1]);
        assert_eq!(blocks.take_a_block(go_on), Some(1));
        assert_eq!(blocks.take_a_block(go_on), None);
    }

    /// （K3 攻方 `stop-checked-after-taking-a-block`、正推 M18）：调用方叫停之后卡只领判完的块
    /// 在等的那几块（比判完的块号小、交回队里的），别的留在队里给下一趟；一块都没判完时一块都不领、队里原样；判红了连判完的块在等的也不领；
    /// 下一轮的那一队记着判完的块号，叫停了照样领判完的块在等的。
    #[test]
    fn card_asked_to_stop_takes_only_the_blocks_judged_blocks_wait_for_and_leaves_the_rest() {
        let blocks = BlocksForTheCards::new(0..5);
        let mut judged_while_stopped = Vec::new();
        let left_while_stopped = judge_blocks_on_one_card(
            &blocks,
            &|| StopOfTheCards::RequestedByTheCaller,
            &mut |pending_index| {
                judged_while_stopped.push(pending_index);
                BlockJudgedOnACard::Judged(pending_index)
            },
            &mut |_pending_index, _judged| BlockHandedToTheStore::Handed,
        );
        assert_eq!(left_while_stopped, CardLeftTheBlocks::NothingLeftOrStopped);
        assert!(
            judged_while_stopped.is_empty(),
            "叫停了、没有判完的块在等：一块都不领"
        );
        assert_eq!(
            blocks.blocks_not_taken(),
            vec![0, 1, 2, 3, 4],
            "叫停时没领的块原样留在队里"
        );
        // 第一张卡领了块 0；第二张判完块 1、2 之后被叫停
        assert_eq!(blocks.take_a_block(StopOfTheCards::NotRequested), Some(0));
        let judged_by_the_second_card = std::cell::RefCell::new(Vec::new());
        let left_by_the_second_card = judge_blocks_on_one_card(
            &blocks,
            &|| {
                if judged_by_the_second_card.borrow().len() >= 2 {
                    StopOfTheCards::RequestedByTheCaller
                } else {
                    StopOfTheCards::NotRequested
                }
            },
            &mut |pending_index| {
                judged_by_the_second_card.borrow_mut().push(pending_index);
                BlockJudgedOnACard::Judged(pending_index)
            },
            &mut |_pending_index, _judged| BlockHandedToTheStore::Handed,
        );
        assert_eq!(
            left_by_the_second_card,
            CardLeftTheBlocks::NothingLeftOrStopped
        );
        assert_eq!(
            judged_by_the_second_card.into_inner(),
            vec![1, 2],
            "叫停之后块 3 比判完的块 2 大：不领"
        );
        // 第一张卡在块 0 上报了 Out of Memory、交回
        blocks.give_back(0);
        assert!(
            blocks.any_left_to_take(StopOfTheCards::RequestedByTheCaller),
            "交回的块 0 比判完的块 2 小：叫停了也领得"
        );
        assert!(
            !blocks.any_left_to_take(StopOfTheCards::TheRunFailed),
            "判红了一块都不领"
        );
        assert_eq!(blocks.take_a_block(StopOfTheCards::TheRunFailed), None);
        let next_round = blocks.left_for_another_round();
        assert_eq!(next_round.blocks_not_taken(), vec![0, 3, 4]);
        assert_eq!(
            next_round.take_a_block(StopOfTheCards::RequestedByTheCaller),
            Some(0),
            "下一轮的那一队记着判完的块 2：叫停了照样领块 0"
        );
        assert_eq!(
            next_round.take_a_block(StopOfTheCards::RequestedByTheCaller),
            None,
            "块 3 留给下一趟"
        );
        assert_eq!(next_round.blocks_not_taken(), vec![3, 4]);
    }

    /// 判不了一块的卡收工、那一块不交回，不再领；判完的一块交不到写库那一头，
    /// 这张卡收工、不再领。
    #[test]
    fn card_stops_taking_blocks_after_a_refusal_and_after_the_store_is_gone() {
        let blocks_of_the_refusing_card = BlocksForTheCards::new(0..5);
        let mut taken = Vec::new();
        let left_by_the_refusing_card = judge_blocks_on_one_card(
            &blocks_of_the_refusing_card,
            &|| StopOfTheCards::NotRequested,
            &mut |pending_index| {
                taken.push(pending_index);
                if pending_index == 1 {
                    BlockJudgedOnACard::<usize>::Refused {
                        reason: "判不了块 1".to_string(),
                    }
                } else {
                    BlockJudgedOnACard::Judged(pending_index)
                }
            },
            &mut |_pending_index, _judged| BlockHandedToTheStore::Handed,
        );
        assert_eq!(
            left_by_the_refusing_card,
            CardLeftTheBlocks::Refused {
                reason: "判不了块 1".to_string()
            }
        );
        assert_eq!(taken, vec![0, 1], "判不了的那一块之后不再领");
        assert_eq!(
            blocks_of_the_refusing_card.blocks_not_taken(),
            vec![2, 3, 4],
            "判不了的那一块不交回"
        );
        let blocks_of_the_card_without_a_store = BlocksForTheCards::new(0..5);
        let mut handed = 0;
        let left_without_a_store = judge_blocks_on_one_card(
            &blocks_of_the_card_without_a_store,
            &|| StopOfTheCards::NotRequested,
            &mut |pending_index| BlockJudgedOnACard::Judged(pending_index),
            &mut |_pending_index, _judged| {
                handed += 1;
                BlockHandedToTheStore::TheStoreIsGone
            },
        );
        assert_eq!(left_without_a_store, CardLeftTheBlocks::TheStoreIsGone);
        assert_eq!(handed, 1, "写库那一头走了：不再领");
    }

    /// 判完的块按块序写库，后面的先到就等着；写库失败之后到的不再写。
    #[test]
    fn judged_blocks_are_stored_in_block_order_and_none_after_a_storing_error() {
        let mut stored_in_order: BlocksStoredInOrder<usize> = BlocksStoredInOrder::new();
        let mut stored = Vec::new();
        for (pending_index, arrived_block) in [(2, 2), (0, 0), (1, 1), (3, 3)] {
            stored_in_order.arrive(pending_index, arrived_block, &mut |block| {
                stored.push(block);
                Ok(())
            });
        }
        assert_eq!(stored, vec![0, 1, 2, 3], "按块序写");
        assert_eq!(stored_in_order.blocks_stored(), 4);
        assert_eq!(stored_in_order.blocks_waiting(), 0);
        let mut failing: BlocksStoredInOrder<usize> = BlocksStoredInOrder::new();
        for (pending_index, arrived_block) in [(1, 1), (0, 0), (2, 2)] {
            failing.arrive(pending_index, arrived_block, &mut |block| match block {
                1 => Err("写判定块失败".to_string()),
                _ => Ok(()),
            });
        }
        assert_eq!(failing.blocks_stored(), 1, "块 1 写失败：只写进了块 0");
        assert_eq!(failing.storing_error(), Some("写判定块失败"));
    }

    /// 第五块：一轮卡收工之后这一趟怎么走，逐种（参数：没写的块、其中判完在等的、这一轮报了的卡、叫没叫停、没报过的卡）。
    #[test]
    fn after_a_round_another_round_starts_only_when_a_card_ran_out_and_another_card_is_left() {
        assert_eq!(
            after_a_round_of_the_cards(0, 0, 1, StopOfTheRun::NotRequested, 1),
            AfterARoundOfTheCards::EveryBlockStored,
            "有卡报了而别的卡判完了：照常交回"
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 0, 0, StopOfTheRun::Requested, 2),
            AfterARoundOfTheCards::StoppedOnRequest
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 0, 1, StopOfTheRun::NotRequested, 1),
            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 2, 1, StopOfTheRun::NotRequested, 1),
            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory,
            "没叫停：等着的块随剩下的一起判"
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 0, 1, StopOfTheRun::NotRequested, 0),
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort,
            "报过的卡之外没有卡了"
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 0, 1, StopOfTheRun::Requested, 1),
            AfterARoundOfTheCards::StoppedOnRequest,
            "叫停了、判完的块都写进了库：照叫停收尾，不再起一轮"
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 2, 1, StopOfTheRun::Requested, 1),
            AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor,
            "叫停了、判完的块在等交回的那一块：再起一轮只判它"
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 2, 1, StopOfTheRun::Requested, 0),
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort,
            "叫停了、判完的块在等而没有卡能判了"
        );
    }

    /// （攻方乙、正推第 4 点）：几轮之间带着的那几样：报了 Out of Memory 的卡记下、下一轮不再用，
    /// 下一轮只判这一轮没领走的，头一句 Out of Memory 留着；一张卡两轮各收工一次只算一张。
    #[test]
    fn rounds_keep_the_cards_short_of_memory_out_and_count_a_card_finishing_twice_once() {
        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(2);
        assert_eq!(rounds.blocks_of_the_next_round(), vec![0, 1]);
        let blocks_of_the_first_round = rounds.take_the_blocks_of_the_round();
        // 头一轮：卡 0 领了块 0、报了 Out of Memory 交回；卡 1 判完块 1（等着块 0）收工
        assert_eq!(
            blocks_of_the_first_round.take_a_block(StopOfTheCards::NotRequested),
            Some(0)
        );
        assert_eq!(
            blocks_of_the_first_round.take_a_block(StopOfTheCards::NotRequested),
            Some(1)
        );
        blocks_of_the_first_round.record_a_judged_block(1);
        rounds.stored_in_order().arrive(1, 1, &mut |_block| Ok(()));
        blocks_of_the_first_round.give_back(0);
        assert_eq!(
            rounds.after_the_round(
                &blocks_of_the_first_round,
                &[0, 1],
                [1],
                &[(0, "显存不够：卡 0 判块 0".to_string())],
                StopOfTheRun::NotRequested,
            ),
            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory
        );
        assert!(rounds.is_short_of_memory(0), "报了的卡 0 不再用");
        assert!(!rounds.is_short_of_memory(1));
        assert_eq!(rounds.first_out_of_memory(), Some("显存不够：卡 0 判块 0"));
        assert_eq!(
            rounds.blocks_of_the_next_round(),
            vec![0],
            "下一轮只判没领走的块 0，判完的块 1 不再判"
        );
        let blocks_of_the_second_round = rounds.take_the_blocks_of_the_round();
        assert_eq!(
            blocks_of_the_second_round.take_a_block(StopOfTheCards::NotRequested),
            Some(0)
        );
        rounds.stored_in_order().arrive(0, 0, &mut |_block| Ok(()));
        assert_eq!(rounds.blocks_stored(), 2);
        assert_eq!(
            rounds.after_the_round(
                &blocks_of_the_second_round,
                &[1],
                [1],
                &[],
                StopOfTheRun::NotRequested
            ),
            AfterARoundOfTheCards::EveryBlockStored
        );
        assert_eq!(rounds.cards_used(), 1, "卡 1 两轮各收工一次只算一张");
    }

    /// P2 那几条用例的假卡按拍走的挂钟（次序照攻方模型 `research/prompts/m3-batch10z-a-code-r1-opus-model/p2_blocks_10z.rs`）：每张卡判一块占几拍；
    /// 挂钟只在没有一张卡在跑（都停在判块上、或收了工）时往前走，最先判完的那一张先往下，同一拍卡号大的先。起卡的那一方也算在跑，
    /// 卡一张一张起完它才让开，卡领第一块的次序因此照卡号。
    struct CardsOnAClockOfTicks {
        clock: Mutex<ClockOfTicks>,
        changed: std::sync::Condvar,
    }

    struct ClockOfTicks {
        now: u64,
        running: usize,
        judging_until: std::collections::BTreeSet<(u64, std::cmp::Reverse<usize>)>,
    }

    impl CardsOnAClockOfTicks {
        fn started_by_the_one_starting_the_cards() -> Self {
            Self {
                clock: Mutex::new(ClockOfTicks {
                    now: 0,
                    running: 1,
                    judging_until: std::collections::BTreeSet::new(),
                }),
                changed: std::sync::Condvar::new(),
            }
        }

        fn locked(&self) -> std::sync::MutexGuard<'_, ClockOfTicks> {
            self.clock
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        }

        fn wait<'clock>(
            &self,
            clock: std::sync::MutexGuard<'clock, ClockOfTicks>,
        ) -> std::sync::MutexGuard<'clock, ClockOfTicks> {
            self.changed
                .wait(clock)
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        }

        fn card_joins(&self) {
            self.locked().running += 1;
        }

        /// 起卡的那一方等它刚起的那张卡停在判块上或收了工。
        fn wait_until_only_the_one_starting_the_cards_runs(&self) {
            let mut clock = self.locked();
            while clock.running > 1 {
                clock = self.wait(clock);
            }
        }

        fn one_running_stops(&self) {
            let mut clock = self.locked();
            clock.running -= 1;
            self.changed.notify_all();
        }

        fn judge_for(&self, card: usize, ticks: u64) {
            let mut clock = self.locked();
            let until = (clock.now + ticks, std::cmp::Reverse(card));
            clock.judging_until.insert(until);
            clock.running -= 1;
            self.changed.notify_all();
            while !(clock.running == 0 && clock.judging_until.first() == Some(&until)) {
                clock = self.wait(clock);
            }
            clock.judging_until.pop_first();
            clock.now = until.0;
            clock.running += 1;
            self.changed.notify_all();
        }
    }

    /// 一张假卡的线程收了工（正常收或 panic 了）就不再算在跑。
    struct CardRunningOnTheClock<'clock>(&'clock CardsOnAClockOfTicks);

    impl Drop for CardRunningOnTheClock<'_> {
        fn drop(&mut self) {
            self.0.one_running_stops();
        }
    }

    /// 假卡那一趟里调用方什么时候叫停。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum CallerStopOfTheFakeRun {
        Never,
        /// 几轮合计判完第 n 块（从 1 数）的那一刻。
        WhenTheBlocksJudgedReach(usize),
        /// 第 0 张卡报 Out of Memory 的那一刻（交回那一块之前）。
        WhenCard0RunsOutOfMemory,
        /// 第 r 轮（从 0 数）起卡之前。
        BeforeRound(usize),
    }

    /// 假卡上几轮判完一队块的结局：照块序写进库的块、每块判了几遍、最后一轮之后怎么走、留给下一趟的块、用到几张卡。
    struct BlocksJudgedOnFakeCards {
        stored: Vec<usize>,
        judged_times: Vec<usize>,
        after_the_last_round: AfterARoundOfTheCards,
        blocks_left_for_the_next_run: Vec<usize>,
        cards_used: usize,
    }

    /// 几张假卡几轮判 `pending_blocks` 块（照崩溃放量产品路径第 ① 段的走法：几轮之间带着的交 [`RoundsOfTheCards`]，一轮里卡一张一张起，
    /// 从一队块里照叫停领、判、交去按块序写库；一轮收工之后照 [`RoundsOfTheCards::after_the_round`] 走）：第 `card` 张判一块占
    /// `ticks_of_each_card[card]` 拍；头一轮只起前 `cards_holding_states_in_the_first_round` 张（装了状态的），再起的一轮起没报过 Out of Memory
    /// 的第一张（收尾那一步再起一张还有空的卡）；第 0 张判它领的第 `out_of_memory_at_block_of_card_0` 块（从 1 数）到末尾时报 Out of Memory，
    /// 报了之后再给它块就 panic（报了就收工）；调用方照 `caller_stop` 叫停。
    fn blocks_judged_on_fake_cards(
        pending_blocks: usize,
        ticks_of_each_card: &[u64],
        cards_holding_states_in_the_first_round: usize,
        out_of_memory_at_block_of_card_0: usize,
        caller_stop: CallerStopOfTheFakeRun,
    ) -> BlocksJudgedOnFakeCards {
        let card_index = |card: usize| u32::try_from(card).expect("假卡没几张");
        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(pending_blocks);
        let stored = Mutex::new(Vec::new());
        let judged_times = Mutex::new(vec![0usize; pending_blocks]);
        let blocks_taken_by_card_0 = AtomicUsize::new(0);
        let blocks_judged = AtomicUsize::new(0);
        let stop_is_requested = std::sync::atomic::AtomicBool::new(false);
        let mut round = 0;
        loop {
            if caller_stop == CallerStopOfTheFakeRun::BeforeRound(round) {
                stop_is_requested.store(true, Ordering::SeqCst);
            }
            let cards_not_short_of_memory: Vec<usize> = (0..ticks_of_each_card.len())
                .filter(|card| !rounds.is_short_of_memory(card_index(*card)))
                .collect();
            let cards_of_the_round: Vec<usize> = if round == 0 {
                (0..cards_holding_states_in_the_first_round).collect()
            } else {
                cards_not_short_of_memory.iter().copied().take(1).collect()
            };
            let blocks = rounds.take_the_blocks_of_the_round();
            let clock = CardsOnAClockOfTicks::started_by_the_one_starting_the_cards();
            // 写库那一份只借给这一轮的卡：这一轮收工之后才交 `after_the_round`
            let stored_in_order = Mutex::new(rounds.stored_in_order());
            let cards_left: Vec<(usize, CardLeftTheBlocks)> = std::thread::scope(|scope| {
                let mut card_threads = Vec::new();
                for card in cards_of_the_round.iter().copied() {
                    clock.card_joins();
                    let (blocks, clock, stored_in_order, stored, judged_times) =
                        (&blocks, &clock, &stored_in_order, &stored, &judged_times);
                    let (blocks_taken_by_card_0, blocks_judged, stop_is_requested) =
                        (&blocks_taken_by_card_0, &blocks_judged, &stop_is_requested);
                    card_threads.push((
                        card,
                        scope.spawn(move || {
                            let _running = CardRunningOnTheClock(clock);
                            judge_blocks_on_one_card(
                                blocks,
                                &|| {
                                    if stop_is_requested.load(Ordering::SeqCst) {
                                        StopOfTheCards::RequestedByTheCaller
                                    } else {
                                        StopOfTheCards::NotRequested
                                    }
                                },
                                &mut |pending_index| {
                                    let taken = match card {
                                        0 => blocks_taken_by_card_0.fetch_add(1, Ordering::SeqCst) + 1,
                                        _ => 0,
                                    };
                                    assert!(
                                        card != 0 || taken <= out_of_memory_at_block_of_card_0,
                                        "第 0 张卡报了 Out of Memory 之后又领了块 {pending_index}：报了就收工"
                                    );
                                    clock.judge_for(card, ticks_of_each_card[card]);
                                    if card == 0 && taken == out_of_memory_at_block_of_card_0 {
                                        if caller_stop == CallerStopOfTheFakeRun::WhenCard0RunsOutOfMemory {
                                            stop_is_requested.store(true, Ordering::SeqCst);
                                        }
                                        return BlockJudgedOnACard::OutOfMemory {
                                            reason: format!("显存不够：第 0 张卡判块 {pending_index}"),
                                        };
                                    }
                                    judged_times.lock().expect("判块的线程不 panic")[pending_index] += 1;
                                    let judged_so_far = blocks_judged.fetch_add(1, Ordering::SeqCst) + 1;
                                    if caller_stop
                                        == CallerStopOfTheFakeRun::WhenTheBlocksJudgedReach(judged_so_far)
                                    {
                                        stop_is_requested.store(true, Ordering::SeqCst);
                                    }
                                    BlockJudgedOnACard::Judged(pending_index)
                                },
                                &mut |pending_index, judged| {
                                    stored_in_order.lock().expect("写库不 panic").arrive(
                                        pending_index,
                                        judged,
                                        &mut |block| {
                                            stored.lock().expect("写库不 panic").push(block);
                                            Ok(())
                                        },
                                    );
                                    BlockHandedToTheStore::Handed
                                },
                            )
                        }),
                    ));
                    clock.wait_until_only_the_one_starting_the_cards_runs();
                }
                clock.one_running_stops();
                card_threads
                    .into_iter()
                    .map(|(card, thread)| (card, thread.join().expect("假卡的线程不 panic")))
                    .collect()
            });
            let cards_out_of_memory: Vec<(u32, String)> = cards_left
                .iter()
                .filter_map(|(card, left)| match left {
                    CardLeftTheBlocks::OutOfMemory { reason } => {
                        Some((card_index(*card), reason.clone()))
                    }
                    CardLeftTheBlocks::NothingLeftOrStopped
                    | CardLeftTheBlocks::Refused { .. }
                    | CardLeftTheBlocks::TheStoreIsGone => None,
                })
                .collect();
            let cards_finished: Vec<u32> = cards_left
                .iter()
                .filter(|(_, left)| *left == CardLeftTheBlocks::NothingLeftOrStopped)
                .map(|(card, _)| card_index(*card))
                .collect();
            let stop = if stop_is_requested.load(Ordering::SeqCst) {
                StopOfTheRun::Requested
            } else {
                StopOfTheRun::NotRequested
            };
            let cards_of_the_round_by_index: Vec<u32> = cards_not_short_of_memory
                .iter()
                .map(|card| card_index(*card))
                .collect();
            match rounds.after_the_round(
                &blocks,
                &cards_of_the_round_by_index,
                cards_finished,
                &cards_out_of_memory,
                stop,
            ) {
                AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory
                | AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor => {
                    round += 1;
                }
                after_the_last_round @ (AfterARoundOfTheCards::EveryBlockStored
                | AfterARoundOfTheCards::StoppedOnRequest
                | AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort) => {
                    return BlocksJudgedOnFakeCards {
                        stored: stored.into_inner().expect("写库不 panic"),
                        judged_times: judged_times.into_inner().expect("判块的线程不 panic"),
                        after_the_last_round,
                        blocks_left_for_the_next_run: rounds.blocks_of_the_next_round(),
                        cards_used: rounds.cards_used(),
                    };
                }
            }
            assert!(
                round <= ticks_of_each_card.len(),
                "每再起一轮至少少一张卡：轮数不超过卡数"
            );
        }
    }

    /// （第 5 条，数照攻方模型那三组快慢：20 块，第 0 张卡判一块是第 1 张的 1、2、3 倍）：第 0 张卡在它领的
    /// 第 1 到 20 块中任一块末尾报 Out of Memory、交回那一块，别的卡都已收工时也由收尾那一步再起的一轮判掉；每一格 20 块都照块序写进库、
    /// 没有一块判两遍。判别力：交回的那一块没人判、它后面判完的 2 块也不入库时红。
    #[test]
    fn block_a_card_gives_back_is_judged_once_and_every_block_is_stored_at_any_speed_of_two_cards()
    {
        const PENDING_BLOCKS: usize = 20;
        for ticks_of_each_card in [[1u64, 1], [2, 1], [3, 1]] {
            for out_of_memory_at_block_of_card_0 in 1..=PENDING_BLOCKS {
                let judged = blocks_judged_on_fake_cards(
                    PENDING_BLOCKS,
                    &ticks_of_each_card,
                    2,
                    out_of_memory_at_block_of_card_0,
                    CallerStopOfTheFakeRun::Never,
                );
                let cell = format!(
                    "快慢 {ticks_of_each_card:?}、第 0 张卡第 {out_of_memory_at_block_of_card_0} 块报"
                );
                assert_eq!(
                    judged.after_the_last_round,
                    AfterARoundOfTheCards::EveryBlockStored,
                    "{cell}"
                );
                assert_eq!(
                    judged.stored,
                    (0..PENDING_BLOCKS).collect::<Vec<usize>>(),
                    "{cell}：每一块都照块序写进库"
                );
                assert_eq!(
                    judged.judged_times,
                    vec![1; PENDING_BLOCKS],
                    "{cell}：每一块判一遍"
                );
            }
        }
    }

    /// （第 5 条攻方另一形）：状态全装在第一张卡（第二张头一轮没起），第一张在它的第 5 块报了：
    /// 收尾那一步再起一轮、第二张判剩下的，20 块都照块序写进库、没有一块判两遍。判别力：第二张闲着、这一趟报本次未跑时红。
    #[test]
    fn with_every_state_on_the_first_card_the_blocks_it_leaves_are_judged_on_the_second_card() {
        let judged = blocks_judged_on_fake_cards(20, &[1, 1], 1, 5, CallerStopOfTheFakeRun::Never);
        assert_eq!(
            judged.after_the_last_round,
            AfterARoundOfTheCards::EveryBlockStored
        );
        assert_eq!(judged.stored, (0..20).collect::<Vec<usize>>());
        assert_eq!(judged.judged_times, vec![1; 20]);
    }

    /// 只有一张卡，它在第 5 块报了：没有卡能判了，这一趟报本次未跑，前 4 块在库里。
    #[test]
    fn with_one_card_only_a_block_it_gives_back_leaves_the_run_not_run() {
        let judged = blocks_judged_on_fake_cards(20, &[1], 1, 5, CallerStopOfTheFakeRun::Never);
        assert_eq!(
            judged.after_the_last_round,
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort
        );
        assert_eq!(judged.stored, vec![0, 1, 2, 3]);
    }

    /// （攻方 `attack_10zc_rounds.rs` 的 `stop-in-the-closing-round` 四格）：两张卡、5 块，
    /// 第 0 张领块 0、判得慢（10 拍）、判到末尾报 Out of Memory 交回；第 1 张先判完块 1–4、收工（块 1–4 等着块 0）。叫停落在四个时刻：
    /// 不叫停、头一轮第 1 张判完它最后一块那一刻、收尾再起的那一轮起卡之前、收尾那一轮判完块 0 那一刻。每一格都不 panic，判完的块照块序都入库、
    /// 没有一块判两遍，第 1 张卡两轮各收工一次只算一张。判别力：「收尾那一轮起卡之前」那一格在轮末的断言上 panic（等着的 4 块）、
    /// 「头一轮最后一块」那一格报本次未跑、4 块判完不入库时红。
    #[test]
    fn stop_around_the_closing_round_stores_every_block_judged_before_it() {
        for caller_stop in [
            CallerStopOfTheFakeRun::Never,
            CallerStopOfTheFakeRun::WhenTheBlocksJudgedReach(4),
            CallerStopOfTheFakeRun::BeforeRound(1),
            CallerStopOfTheFakeRun::WhenTheBlocksJudgedReach(5),
        ] {
            let judged = blocks_judged_on_fake_cards(5, &[10, 1], 2, 1, caller_stop);
            assert_eq!(
                judged.after_the_last_round,
                AfterARoundOfTheCards::EveryBlockStored,
                "{caller_stop:?}"
            );
            assert_eq!(
                judged.stored,
                vec![0, 1, 2, 3, 4],
                "{caller_stop:?}：判完的块都入库"
            );
            assert_eq!(
                judged.judged_times,
                vec![1; 5],
                "{caller_stop:?}：每块判一遍"
            );
            assert_eq!(judged.cards_used, 1, "{caller_stop:?}：第 1 张卡只算一张");
        }
    }

    /// （正推 `rounds_sim4.rs` 的 `STOPSIM stop_from_round=Some(2)`）：两张卡、10 块，第 0 张很慢、
    /// 判第一块时报 Out of Memory，第 1 张把别的 9 块都判完先收工；收尾再起的那一轮起卡之前叫停：那一轮照样判交回的块 0，10 块都入库、不 panic。
    #[test]
    fn stop_before_the_closing_round_still_judges_the_block_nine_judged_blocks_wait_for() {
        let judged = blocks_judged_on_fake_cards(
            10,
            &[300, 1],
            2,
            1,
            CallerStopOfTheFakeRun::BeforeRound(1),
        );
        assert_eq!(
            judged.after_the_last_round,
            AfterARoundOfTheCards::EveryBlockStored
        );
        assert_eq!(judged.stored, (0..10).collect::<Vec<usize>>());
        assert_eq!(judged.judged_times, vec![1; 10]);
    }

    /// （「没判的留给下一趟」）：20 块，第 1 张卡判完块 1–3 时被叫停，第 0 张在块 0 上报了交回：
    /// 收尾那一轮只判块 0，块 0–3 照块序入库，块 4–19 一块都不判、留给下一趟，这一趟照叫停收尾。
    #[test]
    fn stop_leaves_the_blocks_not_judged_to_the_next_run_after_storing_the_judged_ones() {
        let judged = blocks_judged_on_fake_cards(
            20,
            &[10, 1],
            2,
            1,
            CallerStopOfTheFakeRun::WhenTheBlocksJudgedReach(3),
        );
        assert_eq!(
            judged.after_the_last_round,
            AfterARoundOfTheCards::StoppedOnRequest
        );
        assert_eq!(judged.stored, vec![0, 1, 2, 3]);
        assert_eq!(
            judged.judged_times,
            [vec![1; 4], vec![0; 16]].concat(),
            "叫停之后只判判完的块在等的块 0"
        );
        assert_eq!(
            judged.blocks_left_for_the_next_run,
            (4..20).collect::<Vec<usize>>()
        );
    }

    /// 同一轮里第 0 张卡在块 0 上报了 Out of Memory、交回，那一刻叫停；第 1 张还在判块 1：
    /// 判完块 1 之后照样领交回的块 0（块 1 在等它），块 0、1 入库，这一趟照叫停收尾、不报本次未跑（与交回的块落到收尾那一轮时同一个结局）。
    #[test]
    fn stop_with_a_block_given_back_in_the_same_round_stores_the_judged_block_and_stops() {
        let judged = blocks_judged_on_fake_cards(
            4,
            &[1, 2],
            2,
            1,
            CallerStopOfTheFakeRun::WhenCard0RunsOutOfMemory,
        );
        assert_eq!(
            judged.after_the_last_round,
            AfterARoundOfTheCards::StoppedOnRequest
        );
        assert_eq!(judged.stored, vec![0, 1]);
        assert_eq!(judged.judged_times, vec![1, 1, 0, 0]);
        assert_eq!(judged.blocks_left_for_the_next_run, vec![2, 3]);
    }

    /// 一趟里见过一次叫停就记住：叫停文件头一次读到之后被删，之后每一处问都答叫停，
    /// 也不再读文件；没见过叫停时每问一次现读一次。
    #[test]
    fn stop_request_seen_once_stays_requested_after_the_request_is_gone() {
        let reads = AtomicUsize::new(0);
        let request_is_there_on_the_second_read = || reads.fetch_add(1, Ordering::SeqCst) == 1;
        let stop = StopRequestRememberedInThisRun::new(&request_is_there_on_the_second_read);
        assert!(!stop.is_requested(), "头一次读：叫停文件还没有");
        assert!(stop.is_requested(), "第二次读：叫停文件在");
        assert_eq!(
            stop.stop_of_the_run(),
            StopOfTheRun::Requested,
            "叫停文件已经删了：见过一次就记住"
        );
        assert!(stop.is_requested(), "再问照旧");
        assert_eq!(reads.load(Ordering::SeqCst), 2, "见过叫停之后不再读文件");
    }

    /// 叫停、这一轮有卡报了 Out of Memory、判完的块都入了库而照叫停收尾：头一句 Out of Memory
    /// 照「本次未跑」那一行的形态打得出来；这一趟没有卡报过的，照叫停收尾不打这一行。
    #[test]
    fn stop_after_an_out_of_memory_prints_the_first_out_of_memory_in_the_not_run_line() {
        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
        let blocks = rounds.take_the_blocks_of_the_round();
        assert_eq!(blocks.take_a_block(StopOfTheCards::NotRequested), Some(0));
        rounds.stored_in_order().arrive(0, 0, &mut |_block| Ok(()));
        assert_eq!(
            rounds.after_the_round(
                &blocks,
                &[0, 1],
                [1],
                &[(0, "序号 0 的卡判一块时报了 Out of Memory".to_string())],
                StopOfTheRun::Requested
            ),
            AfterARoundOfTheCards::StoppedOnRequest
        );
        assert_eq!(
            rounds
                .not_run_line_of_a_stop_after_an_out_of_memory(
                    "crash_amplification::run_from_environment"
                )
                .as_deref(),
            Some(
                "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=序号 0 的卡判一块时报了 Out of Memory"
            )
        );
        let mut rounds_without_out_of_memory: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
        let blocks_without_out_of_memory =
            rounds_without_out_of_memory.take_the_blocks_of_the_round();
        assert_eq!(
            rounds_without_out_of_memory.after_the_round(
                &blocks_without_out_of_memory,
                &[0],
                [0],
                &[],
                StopOfTheRun::Requested
            ),
            AfterARoundOfTheCards::StoppedOnRequest
        );
        assert_eq!(
            rounds_without_out_of_memory.not_run_line_of_a_stop_after_an_out_of_memory(
                "crash_amplification::run_from_environment"
            ),
            None,
            "没有卡报过 Out of Memory：照叫停收尾不打本次未跑那一行"
        );
    }

    /// （起下一张卡的看叫停没有用例）：后面装了状态的那一张，队里照此刻的叫停还有领得的块才起；
    /// 叫停了、判完的块都不在等，不起；判红了不起；后面没有装了状态的卡，不起。
    #[test]
    fn the_next_card_starts_only_when_it_holds_states_and_a_block_is_left_to_take_under_the_stop() {
        let blocks = BlocksForTheCards::new([0usize, 1, 2]);
        assert_eq!(
            next_card_to_start(&[1, 0, 1], 0, &blocks, || StopOfTheCards::NotRequested),
            Some(2),
            "跳过没装状态的那一张"
        );
        assert_eq!(
            next_card_to_start(&[1, 0, 1], 0, &blocks, || {
                StopOfTheCards::RequestedByTheCaller
            }),
            None,
            "叫停了、还没有判完的块：没有判完的块在等的，不起"
        );
        assert_eq!(
            next_card_to_start(&[1, 0, 1], 0, &blocks, || StopOfTheCards::TheRunFailed),
            None,
            "判红了：不起"
        );
        assert_eq!(
            next_card_to_start(&[1, 0, 1], 2, &blocks, || StopOfTheCards::NotRequested),
            None,
            "后面没有卡了"
        );
        assert_eq!(
            next_card_to_start(&[1, 0], 0, &blocks, || StopOfTheCards::NotRequested),
            None,
            "后面的卡没装状态"
        );
    }

    /// 起下一张卡时叫停只在后面有装了状态的卡时问、问一次；后面没有卡、后面的卡都没装状态，一次都不问
    /// （崩溃放量产品路径的叫停可以按「问第几次」变，多问一次卡上就少领一块）。
    #[test]
    fn the_next_card_asks_for_the_stop_once_only_when_a_later_card_holds_states() {
        let blocks = BlocksForTheCards::new([0usize, 1, 2]);
        let times_asked = std::cell::Cell::new(0usize);
        let stop_counting_the_asks = || {
            times_asked.set(times_asked.get() + 1);
            StopOfTheCards::NotRequested
        };
        assert_eq!(
            next_card_to_start(&[1, 0, 1], 2, &blocks, stop_counting_the_asks),
            None,
            "最后一张卡之后没有卡"
        );
        assert_eq!(times_asked.get(), 0, "后面没有卡：叫停一次都不问");
        assert_eq!(
            next_card_to_start(&[1, 0, 0], 0, &blocks, stop_counting_the_asks),
            None,
            "后面的卡都没装状态"
        );
        assert_eq!(times_asked.get(), 0, "后面的卡都没装状态：叫停一次都不问");
        assert_eq!(
            next_card_to_start(&[1, 0, 1], 0, &blocks, stop_counting_the_asks),
            Some(2),
            "后面有装了状态的卡、没叫停：起它"
        );
        assert_eq!(times_asked.get(), 1, "后面有装了状态的卡：叫停恰好问一次");
    }

    /// 接住一段会 panic 的活，交它 panic 了没有。
    fn panics_when_run(work: impl FnOnce()) -> bool {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).is_err()
    }

    /// （正推 `probes/killers_new.rs` M05）：三张卡各领一块 5、6、7；7 先判完、5 后判完、
    /// 6 最后报 Out of Memory 交回。叫停之后下一张卡领得到 6：判完的块里最大的是 7（在等 6），不是最后判完的 5。
    #[test]
    fn the_largest_judged_block_not_the_last_judged_lets_a_stopped_card_take_the_block_it_waits_for(
    ) {
        let blocks = BlocksForTheCards::new([5usize, 6, 7]);
        let (block_five_taken_sender, block_five_taken) = std::sync::mpsc::channel::<()>();
        let (block_six_taken_sender, block_six_taken) = std::sync::mpsc::channel::<()>();
        let (block_seven_taken_sender, block_seven_taken) = std::sync::mpsc::channel::<()>();
        let (finish_block_five_sender, finish_block_five) = std::sync::mpsc::channel::<()>();
        let (finish_block_six_sender, finish_block_six) = std::sync::mpsc::channel::<()>();
        let (finish_block_seven_sender, finish_block_seven) = std::sync::mpsc::channel::<()>();
        let (block_five_stored_sender, block_five_stored) = std::sync::mpsc::channel::<()>();
        let (block_seven_stored_sender, block_seven_stored) = std::sync::mpsc::channel::<()>();
        let enough_for_any_step = std::time::Duration::from_secs(20);
        // 每张卡只领一块：头一次照常领，之后判红（不再领）
        let one_block_then_stop = || {
            let is_the_first_ask = std::cell::Cell::new(true);
            move || {
                if is_the_first_ask.replace(false) {
                    StopOfTheCards::NotRequested
                } else {
                    StopOfTheCards::TheRunFailed
                }
            }
        };
        let card_left_with_block_six = std::thread::scope(|scope| {
            let blocks = &blocks;
            scope.spawn(move || {
                let stop_of_the_cards = one_block_then_stop();
                let _left = judge_blocks_on_one_card(
                    blocks,
                    &stop_of_the_cards,
                    &mut |block| {
                        assert_eq!(block, 5);
                        block_five_taken_sender.send(()).expect("主线程等着");
                        finish_block_five.recv().expect("主线程会放行");
                        BlockJudgedOnACard::Judged(block)
                    },
                    &mut |_block, _judged: usize| {
                        block_five_stored_sender.send(()).expect("主线程等着");
                        BlockHandedToTheStore::Handed
                    },
                );
            });
            block_five_taken
                .recv_timeout(enough_for_any_step)
                .expect("块 5 领走了");
            let card_with_block_six = scope.spawn(move || {
                let stop_of_the_cards = one_block_then_stop();
                judge_blocks_on_one_card(
                    blocks,
                    &stop_of_the_cards,
                    &mut |block| {
                        assert_eq!(block, 6);
                        block_six_taken_sender.send(()).expect("主线程等着");
                        finish_block_six.recv().expect("主线程会放行");
                        BlockJudgedOnACard::<usize>::OutOfMemory {
                            reason: "块 6 上报了 Out of Memory".to_string(),
                        }
                    },
                    &mut |_block, _judged: usize| BlockHandedToTheStore::Handed,
                )
            });
            block_six_taken
                .recv_timeout(enough_for_any_step)
                .expect("块 6 领走了");
            scope.spawn(move || {
                let stop_of_the_cards = one_block_then_stop();
                let _left = judge_blocks_on_one_card(
                    blocks,
                    &stop_of_the_cards,
                    &mut |block| {
                        assert_eq!(block, 7);
                        block_seven_taken_sender.send(()).expect("主线程等着");
                        finish_block_seven.recv().expect("主线程会放行");
                        BlockJudgedOnACard::Judged(block)
                    },
                    &mut |_block, _judged: usize| {
                        block_seven_stored_sender.send(()).expect("主线程等着");
                        BlockHandedToTheStore::Handed
                    },
                );
            });
            block_seven_taken
                .recv_timeout(enough_for_any_step)
                .expect("块 7 领走了");
            finish_block_seven_sender.send(()).expect("块 7 那张卡等着");
            block_seven_stored
                .recv_timeout(enough_for_any_step)
                .expect("块 7 判完");
            finish_block_five_sender.send(()).expect("块 5 那张卡等着");
            block_five_stored
                .recv_timeout(enough_for_any_step)
                .expect("块 5 判完");
            finish_block_six_sender.send(()).expect("块 6 那张卡等着");
            card_with_block_six.join().expect("块 6 那张卡不 panic")
        });
        assert!(matches!(
            card_left_with_block_six,
            CardLeftTheBlocks::OutOfMemory { .. }
        ));
        assert_eq!(
            blocks.take_a_block(StopOfTheCards::RequestedByTheCaller),
            Some(6),
            "7 判完了在等 6：叫停之后下一张卡领得到 6"
        );
    }

    /// （正推 `probes/killers_new.rs` M10）：叫停、一张卡报了 Out of Memory、判完的块只有 1 块在等：还有别的卡就再起一轮，
    /// 没有别的卡是本次未跑，都不是照叫停收尾；一块都不在等才照叫停收尾。
    #[test]
    fn stop_with_one_judged_block_waiting_needs_another_round_or_is_not_run() {
        assert_eq!(
            after_a_round_of_the_cards(3, 1, 1, StopOfTheRun::Requested, 1),
            AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 1, 1, StopOfTheRun::Requested, 0),
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort
        );
        assert_eq!(
            after_a_round_of_the_cards(3, 0, 1, StopOfTheRun::Requested, 1),
            AfterARoundOfTheCards::StoppedOnRequest
        );
    }

    /// （正推 `probes/killers_new.rs` M13）：没有卡报 Out of Memory 而判完的块还有在等前面那一块的：轮末断言 panic（叫停了也一样）。
    #[test]
    fn round_without_out_of_memory_leaving_a_judged_block_waiting_panics() {
        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
        let blocks = rounds.take_the_blocks_of_the_round();
        assert_eq!(blocks.take_a_block(StopOfTheCards::NotRequested), Some(0));
        rounds.stored_in_order().arrive(2, 2, &mut |_block| Ok(()));
        assert_eq!(rounds.blocks_waiting(), 1);
        assert!(panics_when_run(|| {
            let _after =
                rounds.after_the_round(&blocks, &[0], [0u32], &[], StopOfTheRun::Requested);
        }));
    }

    /// （正推 `probes/killers_new.rs` M14）：没人叫停、没有卡报 Out of Memory，队里却还有没领走的块：轮末断言 panic；叫停了就可以留。
    #[test]
    fn round_without_a_stop_leaving_a_block_untaken_panics() {
        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
        let blocks = rounds.take_the_blocks_of_the_round();
        assert!(panics_when_run(|| {
            let _after =
                rounds.after_the_round(&blocks, &[0], [0u32], &[], StopOfTheRun::NotRequested);
        }));
        let mut stopped_rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(3);
        let stopped_blocks = stopped_rounds.take_the_blocks_of_the_round();
        assert_eq!(
            stopped_rounds.after_the_round(
                &stopped_blocks,
                &[0],
                [0u32],
                &[],
                StopOfTheRun::Requested
            ),
            AfterARoundOfTheCards::StoppedOnRequest
        );
    }

    /// （正推 `probes/killers_new.rs` M17）：这一趟头一句 Out of Memory 是第一张报的那一句，后面报的不盖掉它。
    #[test]
    fn the_first_out_of_memory_of_the_run_stays_the_first_reported() {
        let mut rounds: RoundsOfTheCards<usize> = RoundsOfTheCards::new(2);
        let blocks = rounds.take_the_blocks_of_the_round();
        let _after = rounds.after_the_round(
            &blocks,
            &[0, 1],
            [],
            &[(0, "第一句".to_string()), (1, "第二句".to_string())],
            StopOfTheRun::NotRequested,
        );
        assert_eq!(rounds.first_out_of_memory(), Some("第一句"));
    }

    /// （正推 `probes/killers_new.rs` M22）：一块交回两次要 panic。
    #[test]
    fn giving_a_block_back_twice_panics() {
        let blocks = BlocksForTheCards::new([0usize, 1]);
        let first = blocks
            .take_a_block(StopOfTheCards::NotRequested)
            .expect("队里有块");
        blocks.give_back(first);
        assert!(panics_when_run(|| blocks.give_back(first)));
    }
}
