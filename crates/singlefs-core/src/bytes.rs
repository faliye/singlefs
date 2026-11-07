//! 定长字段的小端读写（D22（单元原子性怎么合成） 已定项 13：盘上定长字段一律小端）。
//!
//! 读写两侧连同小端编码的规格都在 `verus!` 里，由 Verus 直接验（里程碑四出口 5「core 直验」编解码与记录格式这一簇，被验区域登记在
//! `.claude/gate.d/verus-pins.tsv`，必红世界在 `crates/singlefs-core/verus-must-red-worlds.tsv`）。写的一侧的契约是「写到的那一截
//! 接上这一段」：[`ByteWriter`] 只往游标处写、游标只往前走，`new` 写的是 0，所以游标之后恒是 0，写完交出的整份就是写到的那一截
//! 接上还没写到的零（[`ByteWriter::spec_finished_bytes`]）。读的一侧的契约是「读出来的值是那几个字节的小端值」。两侧用同一对规格
//! [`little_endian_bytes`] 与 [`little_endian_value`]，两条引理（[`lemma_little_endian_value_of_bytes`]、
//! [`lemma_little_endian_bytes_of_value`]）证它们互逆：各记录的往返契约都落在这两条上。
//!
//! `requires` 写的是原来那几处 panic 的反面（写过缓冲末尾、读过切片末尾、6 字节字段装不下、字段表偏移与写到的位置不符）：
//! 经 Verus 验过的调用方证得出它们，没经 Verus 验的调用方违反时照旧在原来那一处 panic，产品行为与没有 `verus!` 时相同。

use vstd::prelude::*;

verus! {

// ===== 小端编码的规格 =====

/// `width` 个字节装得下的取值个数，即 256 的 `width` 次方。
pub open spec fn little_endian_capacity(width: nat) -> nat
    decreases width,
{
    if width == 0 {
        1
    } else {
        256 * little_endian_capacity((width - 1) as nat)
    }
}

/// `value` 写成 `width` 个字节的小端：低字节在前。`value` 装不下时只留低 `width` 个字节（调用方要的都装得下，见各处 `requires`）。
pub open spec fn little_endian_bytes(value: nat, width: nat) -> Seq<u8>
    decreases width,
{
    if width == 0 {
        Seq::empty()
    } else {
        seq![(value % 256) as u8] + little_endian_bytes(value / 256, (width - 1) as nat)
    }
}

/// 一段字节按小端读成的值。
pub open spec fn little_endian_value(bytes: Seq<u8>) -> nat
    decreases bytes.len(),
{
    if bytes.len() == 0 {
        0
    } else {
        bytes[0] as nat + 256 * little_endian_value(bytes.drop_first())
    }
}

/// `width` 个 0 字节（留位、补齐、预留）。
pub open spec fn zero_bytes(width: nat) -> Seq<u8> {
    Seq::new(width, |_position: int| 0u8)
}

pub proof fn lemma_little_endian_bytes_length(value: nat, width: nat)
    ensures
        little_endian_bytes(value, width).len() == width,
    decreases width,
{
    if width > 0 {
        lemma_little_endian_bytes_length(value / 256, (width - 1) as nat);
    }
}

/// 1、2、4、6、8 字节各装得下多少个取值（盘上字段只有这几种宽）。
pub proof fn lemma_little_endian_capacities()
    ensures
        little_endian_capacity(1) == 0x100,
        little_endian_capacity(2) == 0x1_0000,
        little_endian_capacity(4) == 0x1_0000_0000,
        little_endian_capacity(6) == 0x1_0000_0000_0000,
        little_endian_capacity(7) == 0x100_0000_0000_0000,
        little_endian_capacity(8) == 0x1_0000_0000_0000_0000,
{
    reveal_with_fuel(little_endian_capacity, 9);
}

/// 写成小端再读回来是原值（装得下的前提下）：写的一侧到读的一侧的往返。
pub proof fn lemma_little_endian_value_of_bytes(value: nat, width: nat)
    requires
        value < little_endian_capacity(width),
    ensures
        little_endian_value(little_endian_bytes(value, width)) == value,
    decreases width,
{
    if width > 0 {
        let narrower_capacity = little_endian_capacity((width - 1) as nat);
        assert(value / 256 < narrower_capacity) by (nonlinear_arith)
            requires
                value < 256 * narrower_capacity,
        ;
        lemma_little_endian_value_of_bytes(value / 256, (width - 1) as nat);
        let all_bytes = little_endian_bytes(value, width);
        assert(all_bytes.drop_first() =~= little_endian_bytes(value / 256, (width - 1) as nat));
    }
}

/// 读成小端值再写回同样宽是原来那段字节：读的一侧到写的一侧的往返。
pub proof fn lemma_little_endian_bytes_of_value(bytes: Seq<u8>)
    ensures
        little_endian_bytes(little_endian_value(bytes), bytes.len()) == bytes,
    decreases bytes.len(),
{
    if bytes.len() > 0 {
        let rest = bytes.drop_first();
        lemma_little_endian_bytes_of_value(rest);
        let value = little_endian_value(bytes);
        let rest_value = little_endian_value(rest);
        assert(value % 256 == bytes[0] as nat && value / 256 == rest_value) by (nonlinear_arith)
            requires
                value == bytes[0] as nat + 256 * rest_value,
                bytes[0] < 256,
        ;
        assert(little_endian_bytes(value, bytes.len()) =~= bytes);
    }
}

/// 一段字节的小端值装得进它那么宽。
pub proof fn lemma_little_endian_value_fits(bytes: Seq<u8>)
    ensures
        little_endian_value(bytes) < little_endian_capacity(bytes.len()),
    decreases bytes.len(),
{
    if bytes.len() > 0 {
        let rest = bytes.drop_first();
        lemma_little_endian_value_fits(rest);
        let rest_value = little_endian_value(rest);
        let rest_capacity = little_endian_capacity(rest.len());
        assert(bytes[0] as nat + 256 * rest_value < 256 * rest_capacity) by (nonlinear_arith)
            requires
                rest_value < rest_capacity,
                bytes[0] < 256,
        ;
    }
}

/// 同宽的两个值写成的小端相同，值就相同（装得下的前提下）：各记录的写法是单射时要它。
pub proof fn lemma_little_endian_bytes_are_injective(this_value: nat, other_value: nat, width: nat)
    requires
        this_value < little_endian_capacity(width),
        other_value < little_endian_capacity(width),
        little_endian_bytes(this_value, width) == little_endian_bytes(other_value, width),
    ensures
        this_value == other_value,
{
    lemma_little_endian_value_of_bytes(this_value, width);
    lemma_little_endian_value_of_bytes(other_value, width);
}

/// 写成的小端有 `width` 个字节（broadcast：各记录的往返引理靠它算各段的偏移）。
pub broadcast proof fn lemma_little_endian_bytes_have_the_width(value: nat, width: nat)
    ensures
        #[trigger] little_endian_bytes(value, width).len() == width,
{
    lemma_little_endian_bytes_length(value, width);
}

/// 装得下的值写成小端再读回是原值（broadcast 形的 [`lemma_little_endian_value_of_bytes`]）。
pub broadcast proof fn lemma_little_endian_bytes_read_back(value: nat, width: nat)
    requires
        value < little_endian_capacity(width),
    ensures
        #[trigger] little_endian_value(little_endian_bytes(value, width)) == value,
{
    lemma_little_endian_value_of_bytes(value, width);
}

/// 各记录的往返引理 `broadcast use` 这一组。
pub broadcast group group_little_endian_round_trips {
    lemma_little_endian_bytes_have_the_width,
    lemma_little_endian_bytes_read_back,
}

/// 一段字节里 `[position, position + 前段长 + 后段长)` 是两段接起来的，就各自落在前后两截上：各记录的往返引理拿它
/// 把整条记录的写法一段一段剥到各个字段上（接起来是左结合的，从最后一段往前剥）。
pub proof fn lemma_split_a_concatenated_range(source: Seq<u8>, position: int, front: Seq<u8>, back: Seq<u8>)
    requires
        0 <= position,
        position + front.len() + back.len() <= source.len(),
        source.subrange(position, position + front.len() + back.len()) == front + back,
    ensures
        source.subrange(position, position + front.len()) == front,
        source.subrange(position + front.len(), position + front.len() + back.len()) == back,
{
    let whole = source.subrange(position, position + front.len() + back.len());
    assert forall|index: int| 0 <= index < front.len() implies source.subrange(
        position,
        position + front.len(),
    )[index] == front[index] by {
        assert(whole[index] == (front + back)[index]);
    }
    assert(source.subrange(position, position + front.len()) =~= front);
    assert forall|index: int| 0 <= index < back.len() implies source.subrange(
        position + front.len(),
        position + front.len() + back.len(),
    )[index] == back[index] by {
        assert(whole[front.len() + index] == (front + back)[front.len() + index]);
    }
    assert(source.subrange(position + front.len(), position + front.len() + back.len()) =~= back);
}

/// 一段字节是不是全 0（留位、补齐、预留位的读者判法：非 0 一律判那个结构损坏）。
#[must_use]
pub fn every_byte_is_zero(bytes: &[u8]) -> (all_zero: bool)
    ensures
        all_zero == (bytes@ == zero_bytes(bytes@.len())),
{
    let mut checked_count: usize = 0;
    while checked_count < bytes.len()
        invariant
            checked_count <= bytes@.len(),
            forall|position: int| 0 <= position < checked_count ==> #[trigger] bytes@[position] == 0u8,
        decreases bytes@.len() - checked_count,
    {
        if bytes[checked_count] != 0 {
            assert(bytes@[checked_count as int] != zero_bytes(bytes@.len())[checked_count as int]);
            return false;
        }
        checked_count += 1;
    }
    assert(bytes@ =~= zero_bytes(bytes@.len()));
    true
}

/// 把 `source` 原样拷进 `destination` 的 `[offset, offset + source.len())`，别处不动。
///
/// 只有这一个函数体不经 Verus 验（`external_body`，登记在信任清单 `.claude/gate.d/verus-trust-list.tsv`）：它就是一句
/// `copy_from_slice`，vstd 没有它的规格；逐字节写的循环验得过但成块拷贝慢，这里要成块拷贝。差分测试
/// [`tests::copying_bytes_into_a_buffer_changes_exactly_the_named_range`] 拿它与逐字节的写法在一批长度与偏移上逐字节比。
#[verifier::external_body]
#[allow(
    clippy::ptr_arg,
    reason = "Verus 对 &mut 参数的规格按 Vec 的视图写（final(destination)@），换成 &mut [u8] 要另经切片可变引用的规格，这里只拷一段、不改长度"
)]
pub fn copy_bytes_into(destination: &mut Vec<u8>, offset: usize, source: &[u8])
    requires
        offset + source@.len() <= old(destination)@.len(),
    ensures
        final(destination)@ == old(destination)@.subrange(0, offset as int) + source@ + old(
            destination,
        )@.subrange(offset + source@.len(), old(destination)@.len() as int),
{
    destination[offset..offset + source.len()].copy_from_slice(source);
}

/// 一段字节原样拷成定长数组（盘上定长的 fsid、key 这类）。
///
/// # Panics
/// 长度不是 `WIDTH`（`requires` 排除的那一种）。
#[must_use]
pub fn copied_into_an_array<const WIDTH: usize>(slice: &[u8]) -> (array: [u8; WIDTH])
    requires
        slice@.len() == WIDTH,
    ensures
        array@ == slice@,
{
    assert!(slice.len() == WIDTH, "拷成定长数组的那一段字节恰好是数组那么宽");
    let mut array = [0u8; WIDTH];
    let mut copied_count: usize = 0;
    while copied_count < WIDTH
        invariant
            copied_count <= WIDTH,
            slice@.len() == WIDTH,
            array@.len() == WIDTH,
            forall|position: int| 0 <= position < copied_count ==> #[trigger] array@[position] == slice@[position],
        decreases WIDTH - copied_count,
    {
        array[copied_count] = slice[copied_count];
        copied_count += 1;
    }
    assert(array@ =~= slice@);
    array
}

/// 一段字节原样拷成一个新的 `Vec`（`to_vec` 没有 Verus 规格，这里经 [`ByteWriter`] 写一遍，同一个结果）。
#[must_use]
pub fn copied_into_a_vector(slice: &[u8]) -> (copy: Vec<u8>)
    ensures
        copy@ == slice@,
{
    let mut writer = ByteWriter::new(slice.len());
    writer.put(slice);
    let copy = writer.into_bytes();
    assert(copy@ =~= slice@);
    copy
}

/// 往一段定长缓冲里顺序写字段；每一段的偏移用 [`ByteWriter::assert_position`] 钉在字段表上。
pub struct ByteWriter {
    bytes: Vec<u8>,
    cursor: usize,
}

impl ByteWriter {
    /// 写到的那一截：从缓冲起点到游标。
    pub closed spec fn written(&self) -> Seq<u8> {
        self.bytes@.subrange(0, self.cursor as int)
    }

    /// 缓冲的总宽（`new` 时给的那个数）。
    pub closed spec fn total_bytes(&self) -> nat {
        self.bytes@.len()
    }

    /// 游标不过缓冲末尾，游标之后的字节都还是 0（`new` 写 0、只往游标处写、游标只往前走）。
    pub closed spec fn is_well_formed(&self) -> bool {
        &&& self.cursor <= self.bytes@.len()
        &&& forall|position: int|
            self.cursor <= position < self.bytes@.len() ==> #[trigger] self.bytes@[position] == 0u8
    }

    /// [`Self::into_bytes`] 交出的整份：写到的那一截接上还没写到的零。
    pub open spec fn spec_finished_bytes(&self) -> Seq<u8> {
        self.written() + zero_bytes((self.total_bytes() - self.written().len()) as nat)
    }

    /// 一个写得进 `byte_count` 个字节的、合乎规矩的写者：各个写的方法的前提。
    pub open spec fn has_room_for(&self, byte_count: nat) -> bool {
        &&& self.is_well_formed()
        &&& self.written().len() + byte_count <= self.total_bytes()
    }

    proof fn lemma_written_is_within_the_buffer(&self)
        requires
            self.is_well_formed(),
        ensures
            self.written().len() == self.cursor,
            self.cursor <= self.total_bytes(),
            self.total_bytes() <= usize::MAX,
    {
        assert(self.bytes@.len() == self.bytes.len());
    }

    #[must_use]
    pub fn new(total_bytes: usize) -> (writer: Self)
        ensures
            writer.is_well_formed(),
            writer.written() == Seq::<u8>::empty(),
            writer.total_bytes() == total_bytes,
    {
        let writer = Self { bytes: vec![0u8; total_bytes], cursor: 0 };
        assert(writer.written() =~= Seq::<u8>::empty());
        writer
    }

    /// # Panics
    /// 写过缓冲末尾（`requires` 排除的那一种）。
    pub fn put(&mut self, slice: &[u8])
        requires
            old(self).has_room_for(slice@.len()),
        ensures
            final(self).is_well_formed(),
            final(self).written() == old(self).written() + slice@,
            final(self).total_bytes() == old(self).total_bytes(),
    {
        proof {
            self.lemma_written_is_within_the_buffer();
        }
        let start = self.cursor;
        copy_bytes_into(&mut self.bytes, start, slice);
        self.cursor = start + slice.len();
        assert(self.written() =~= old(self).written() + slice@);
        assert forall|position: int|
            self.cursor <= position < self.bytes@.len() implies #[trigger] self.bytes@[position]
            == 0u8 by {
            assert(self.bytes@[position] == old(self).bytes@[position]);
        }
    }

    /// 在游标处写 `value` 的低 `width` 个字节、低字节在前，游标前移 `width`：各个定宽字段共用这一份写法。
    fn put_little_endian(&mut self, value: u64, width: usize)
        requires
            old(self).has_room_for(width as nat),
            width <= 8,
            value < little_endian_capacity(width as nat),
        ensures
            final(self).is_well_formed(),
            final(self).written() == old(self).written() + little_endian_bytes(
                value as nat,
                width as nat,
            ),
            final(self).total_bytes() == old(self).total_bytes(),
    {
        proof {
            self.lemma_written_is_within_the_buffer();
            lemma_little_endian_bytes_length(value as nat, width as nat);
        }
        let start = self.cursor;
        let mut remaining = value;
        let mut written_count: usize = 0;
        while written_count < width
            invariant
                start == old(self).cursor,
                self.cursor == start,
                start + width <= self.bytes@.len(),
                self.bytes@.len() <= usize::MAX,
                self.bytes@.len() == old(self).bytes@.len(),
                written_count <= width,
                self.bytes@.subrange(start as int, start + written_count) + little_endian_bytes(
                    remaining as nat,
                    (width - written_count) as nat,
                ) == little_endian_bytes(value as nat, width as nat),
                forall|position: int|
                    0 <= position < start ==> #[trigger] self.bytes@[position] == old(
                        self,
                    ).bytes@[position],
                forall|position: int|
                    start + written_count <= position < self.bytes@.len() ==> #[trigger] self.bytes@[position]
                        == old(self).bytes@[position],
            decreases width - written_count,
        {
            let ghost previous_bytes = self.bytes@;
            let ghost previous_remaining = remaining;
            self.bytes[start + written_count] = u8::try_from(remaining % 256)
                .expect("对 256 取余装得进一个字节");
            remaining /= 256;
            proof {
                let rest_width = (width - written_count - 1) as nat;
                assert(little_endian_bytes(previous_remaining as nat, (width - written_count) as nat)
                    =~= seq![(previous_remaining % 256) as u8] + little_endian_bytes(
                    remaining as nat,
                    rest_width,
                ));
                assert(self.bytes@.subrange(start as int, start + written_count + 1)
                    =~= previous_bytes.subrange(start as int, start + written_count) + seq![
                    (previous_remaining % 256) as u8]);
            }
            written_count += 1;
        }
        self.cursor = start + width;
        proof {
            assert(little_endian_bytes(remaining as nat, 0) =~= Seq::<u8>::empty());
            assert(self.written() =~= old(self).written() + little_endian_bytes(
                value as nat,
                width as nat,
            ));
        }
    }

    pub fn put_u8(&mut self, value: u8)
        requires
            old(self).has_room_for(1),
        ensures
            final(self).is_well_formed(),
            final(self).written() == old(self).written() + little_endian_bytes(value as nat, 1),
            final(self).total_bytes() == old(self).total_bytes(),
    {
        proof {
            lemma_little_endian_capacities();
        }
        self.put_little_endian(u64::from(value), 1);
    }

    pub fn put_u16(&mut self, value: u16)
        requires
            old(self).has_room_for(2),
        ensures
            final(self).is_well_formed(),
            final(self).written() == old(self).written() + little_endian_bytes(value as nat, 2),
            final(self).total_bytes() == old(self).total_bytes(),
    {
        proof {
            lemma_little_endian_capacities();
        }
        self.put_little_endian(u64::from(value), 2);
    }

    pub fn put_u32(&mut self, value: u32)
        requires
            old(self).has_room_for(4),
        ensures
            final(self).is_well_formed(),
            final(self).written() == old(self).written() + little_endian_bytes(value as nat, 4),
            final(self).total_bytes() == old(self).total_bytes(),
    {
        proof {
            lemma_little_endian_capacities();
        }
        self.put_little_endian(u64::from(value), 4);
    }

    /// 6 字节字段（16 KiB 槽号、jsn 计数器、事务号低 48 位）。
    ///
    /// # Panics
    /// 值装不进 48 位（`requires` 排除的那一种）。
    pub fn put_six_byte_unsigned(&mut self, value: u64)
        requires
            old(self).has_room_for(6),
            value < 0x1_0000_0000_0000u64,
        ensures
            final(self).is_well_formed(),
            final(self).written() == old(self).written() + little_endian_bytes(value as nat, 6),
            final(self).total_bytes() == old(self).total_bytes(),
    {
        proof {
            lemma_little_endian_capacities();
        }
        assert(1u64 << 48 == 0x1_0000_0000_0000u64) by (bit_vector);
        assert!(value < (1u64 << 48), "48 位字段装不下 {value}");
        self.put_little_endian(value, 6);
    }

    pub fn put_u64(&mut self, value: u64)
        requires
            old(self).has_room_for(8),
        ensures
            final(self).is_well_formed(),
            final(self).written() == old(self).written() + little_endian_bytes(value as nat, 8),
            final(self).total_bytes() == old(self).total_bytes(),
    {
        proof {
            lemma_little_endian_capacities();
        }
        self.put_little_endian(value, 8);
    }

    /// 留位、补齐：跳过的字节保持 0。
    pub fn skip(&mut self, byte_count: usize)
        requires
            old(self).has_room_for(byte_count as nat),
        ensures
            final(self).is_well_formed(),
            final(self).written() == old(self).written() + zero_bytes(byte_count as nat),
            final(self).total_bytes() == old(self).total_bytes(),
    {
        proof {
            self.lemma_written_is_within_the_buffer();
        }
        self.cursor += byte_count;
        assert(self.written() =~= old(self).written() + zero_bytes(byte_count as nat));
    }

    #[must_use]
    pub fn position(&self) -> (position: usize)
        requires
            self.is_well_formed(),
        ensures
            position == self.written().len(),
    {
        self.cursor
    }

    /// 字段表上的偏移与写到的位置不符就是写错了字段表，当场断言。
    ///
    /// # Panics
    /// 写到的位置不是 `expected`（`requires` 排除的那一种：经 Verus 验过的调用方在编出来之前就证了字段表与写法对得上）。
    pub fn assert_position(&self, expected: u64, what: &str)
        requires
            self.is_well_formed(),
            self.written().len() == expected,
    {
        let cursor = u64::try_from(self.cursor).expect("偏移装得进 u64");
        // assert_eq! 展开成的 core::panicking::assert_failed 没有 Verus 规格，写成 assert! 加同一句话（多带两边的值）。
        assert!(
            cursor == expected,
            "{what} 的偏移与字段表不符：写到 {cursor}，字段表是 {expected}"
        );
    }

    #[must_use]
    pub fn into_bytes(self) -> (bytes: Vec<u8>)
        requires
            self.is_well_formed(),
        ensures
            bytes@ == self.spec_finished_bytes(),
            bytes@.len() == self.total_bytes(),
    {
        proof {
            self.lemma_written_is_within_the_buffer();
            assert(self.bytes@ =~= self.spec_finished_bytes());
        }
        self.bytes
    }
}

/// 从一段字节里顺序读字段。
pub struct ByteReader<'bytes> {
    bytes: &'bytes [u8],
    cursor: usize,
}

impl<'bytes> ByteReader<'bytes> {
    /// 被读的那一段字节。
    pub closed spec fn source(&self) -> Seq<u8> {
        self.bytes@
    }

    /// 游标：下一次读从这里起。
    pub closed spec fn spec_position(&self) -> nat {
        self.cursor as nat
    }

    /// 从游标起还读得出 `byte_count` 个字节：各个读的方法的前提。
    pub open spec fn has_left(&self, byte_count: nat) -> bool {
        self.spec_position() + byte_count <= self.source().len()
    }

    /// 从游标起 `byte_count` 个字节（读的方法交出或读成值的那一截）。
    pub open spec fn next_bytes(&self, byte_count: nat) -> Seq<u8> {
        self.source().subrange(self.spec_position() as int, (self.spec_position() + byte_count) as int)
    }

    /// 读了 `byte_count` 个字节之后：同一段字节、游标前移 `byte_count`。
    pub open spec fn advanced_by(&self, earlier: &Self, byte_count: nat) -> bool {
        &&& self.source() == earlier.source()
        &&& self.spec_position() == earlier.spec_position() + byte_count
    }

    #[must_use]
    pub fn at(bytes: &'bytes [u8], offset: usize) -> (reader: Self)
        ensures
            reader.source() == bytes@,
            reader.spec_position() == offset,
    {
        Self { bytes, cursor: offset }
    }

    /// # Panics
    /// 读过切片末尾（`requires` 排除的那一种）。
    pub fn take(&mut self, byte_count: usize) -> (slice: &'bytes [u8])
        requires
            old(self).has_left(byte_count as nat),
        ensures
            final(self).advanced_by(old(self), byte_count as nat),
            slice@ == old(self).next_bytes(byte_count as nat),
    {
        assert(self.bytes@.len() == self.bytes.len());
        let slice = &self.bytes[self.cursor..self.cursor + byte_count];
        self.cursor += byte_count;
        slice
    }

    /// 从游标起读 `width` 个字节、按小端拼成值，游标前移 `width`：各个定宽字段共用这一份读法。
    fn get_little_endian(&mut self, width: usize) -> (value: u64)
        requires
            old(self).has_left(width as nat),
            width <= 8,
        ensures
            final(self).advanced_by(old(self), width as nat),
            value == little_endian_value(old(self).next_bytes(width as nat)),
            little_endian_bytes(value as nat, width as nat) == old(self).next_bytes(width as nat),
    {
        assert(self.bytes@.len() == self.bytes.len());
        let start = self.cursor;
        let mut value: u64 = 0;
        let mut unread_count: usize = width;
        while unread_count > 0
            invariant
                start == old(self).cursor,
                self.bytes == old(self).bytes,
                self.cursor == start,
                start + width <= self.bytes@.len(),
                self.bytes@.len() <= usize::MAX,
                unread_count <= width,
                width <= 8,
                value == little_endian_value(
                    self.bytes@.subrange(start + unread_count, start + width),
                ),
            decreases unread_count,
        {
            let ghost tail = self.bytes@.subrange(start + unread_count, start + width);
            unread_count -= 1;
            let byte = self.bytes[start + unread_count];
            proof {
                lemma_little_endian_value_fits(tail);
                lemma_little_endian_capacities();
                lemma_little_endian_capacity_grows((width - unread_count - 1) as nat, 7);
                let extended = self.bytes@.subrange(start + unread_count, start + width);
                assert(extended.drop_first() =~= tail);
                assert(value * 256 + byte < 0x1_0000_0000_0000_0000) by (nonlinear_arith)
                    requires
                        value < little_endian_capacity((width - unread_count - 1) as nat),
                        little_endian_capacity((width - unread_count - 1) as nat)
                            <= little_endian_capacity(7),
                        little_endian_capacity(7) == 0x100_0000_0000_0000,
                        byte < 256,
                ;
            }
            value = u64::from(byte) + value * 256;
        }
        self.cursor = start + width;
        proof {
            lemma_little_endian_bytes_of_value(old(self).next_bytes(width as nat));
        }
        value
    }

    pub fn get_u8(&mut self) -> (value: u8)
        requires
            old(self).has_left(1),
        ensures
            final(self).advanced_by(old(self), 1),
            value == little_endian_value(old(self).next_bytes(1)),
            little_endian_bytes(value as nat, 1) == old(self).next_bytes(1),
    {
        let value = self.take(1)[0];
        proof {
            let bytes = old(self).next_bytes(1);
            assert(bytes.drop_first() =~= Seq::<u8>::empty());
            reveal_with_fuel(little_endian_value, 2);
            lemma_little_endian_bytes_of_value(bytes);
        }
        value
    }

    pub fn get_u16(&mut self) -> (value: u16)
        requires
            old(self).has_left(2),
        ensures
            final(self).advanced_by(old(self), 2),
            value == little_endian_value(old(self).next_bytes(2)),
            little_endian_bytes(value as nat, 2) == old(self).next_bytes(2),
    {
        proof {
            lemma_little_endian_value_fits(old(self).next_bytes(2));
            lemma_little_endian_capacities();
        }
        let wide_value = self.get_little_endian(2);
        u16::try_from(wide_value).expect("2 字节的小端值装得进 u16")
    }

    pub fn get_u32(&mut self) -> (value: u32)
        requires
            old(self).has_left(4),
        ensures
            final(self).advanced_by(old(self), 4),
            value == little_endian_value(old(self).next_bytes(4)),
            little_endian_bytes(value as nat, 4) == old(self).next_bytes(4),
    {
        proof {
            lemma_little_endian_value_fits(old(self).next_bytes(4));
            lemma_little_endian_capacities();
        }
        let wide_value = self.get_little_endian(4);
        u32::try_from(wide_value).expect("4 字节的小端值装得进 u32")
    }

    pub fn get_six_byte_unsigned(&mut self) -> (value: u64)
        requires
            old(self).has_left(6),
        ensures
            final(self).advanced_by(old(self), 6),
            value == little_endian_value(old(self).next_bytes(6)),
            little_endian_bytes(value as nat, 6) == old(self).next_bytes(6),
            value < 0x1_0000_0000_0000u64,
    {
        proof {
            lemma_little_endian_value_fits(old(self).next_bytes(6));
            lemma_little_endian_capacities();
        }
        self.get_little_endian(6)
    }

    pub fn get_u64(&mut self) -> (value: u64)
        requires
            old(self).has_left(8),
        ensures
            final(self).advanced_by(old(self), 8),
            value == little_endian_value(old(self).next_bytes(8)),
            little_endian_bytes(value as nat, 8) == old(self).next_bytes(8),
    {
        self.get_little_endian(8)
    }

    /// # Panics
    /// 不 panic；游标可以停在切片末尾之后，之后的读在它们自己的 `requires` 上被拦下。
    pub fn skip(&mut self, byte_count: usize)
        requires
            old(self).spec_position() + byte_count <= usize::MAX,
        ensures
            final(self).advanced_by(old(self), byte_count as nat),
    {
        self.cursor += byte_count;
    }

    #[must_use]
    pub fn position(&self) -> (position: usize)
        ensures
            position == self.spec_position(),
    {
        self.cursor
    }
}

/// 宽的装得下的不少于窄的：`get_little_endian` 的循环里拿它把中途的值压在 7 字节之内。
pub proof fn lemma_little_endian_capacity_grows(narrower_width: nat, wider_width: nat)
    requires
        narrower_width <= wider_width,
    ensures
        little_endian_capacity(narrower_width) <= little_endian_capacity(wider_width),
    decreases wider_width,
{
    if narrower_width < wider_width {
        lemma_little_endian_capacity_grows(narrower_width, (wider_width - 1) as nat);
    }
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_round_trip_in_little_endian_order() {
        let mut writer = ByteWriter::new(24);
        writer.put_u16(0x0102);
        writer.put_u32(0x0304_0506);
        writer.put_six_byte_unsigned(0x0708_090A_0B0C);
        writer.put_u64(0x0D0E_0F10_1112_1314);
        writer.skip(4);
        writer.assert_position(24, "测试");
        let bytes = writer.into_bytes();
        assert_eq!(&bytes[..2], &[0x02, 0x01], "小端");
        let mut reader = ByteReader::at(&bytes, 0);
        assert_eq!(reader.get_u16(), 0x0102);
        assert_eq!(reader.get_u32(), 0x0304_0506);
        assert_eq!(reader.get_six_byte_unsigned(), 0x0708_090A_0B0C);
        assert_eq!(reader.get_u64(), 0x0D0E_0F10_1112_1314);
        assert_eq!(reader.position(), 20);
    }

    /// 差分测试：信任清单里 `copy_bytes_into`（`external_body`）的规格——`[offset, offset + 源长)` 换成源、别处不动——与逐字节写的结果
    /// 在一批缓冲长度、偏移、源长上逐字节比；规格写错（例如偏移错一位、多拷或少拷一字节）这里就红。
    #[test]
    fn copying_bytes_into_a_buffer_changes_exactly_the_named_range() {
        for buffer_length in 0..24usize {
            for offset in 0..=buffer_length {
                for source_length in 0..=(buffer_length - offset) {
                    let original: Vec<u8> = (0..buffer_length)
                        .map(|position| u8::try_from(position % 251).expect("小于 251") ^ 0x5A)
                        .collect();
                    let source: Vec<u8> = (0..source_length)
                        .map(|position| u8::try_from((position * 7 + 3) % 256).expect("小于 256"))
                        .collect();
                    let mut copied = original.clone();
                    copy_bytes_into(&mut copied, offset, &source);
                    let mut expected = original.clone();
                    for (position_in_source, byte) in source.iter().enumerate() {
                        expected[offset + position_in_source] = *byte;
                    }
                    assert_eq!(
                        copied, expected,
                        "缓冲长 {buffer_length}、偏移 {offset}、源长 {source_length}：copy_bytes_into 只换掉点名的那一段"
                    );
                }
            }
        }
    }
}
