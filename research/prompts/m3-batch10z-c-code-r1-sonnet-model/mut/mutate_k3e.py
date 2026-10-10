import sys
sys.path.insert(0, '.')
import mutate_k3d as b
b.M.clear()
m = b.m
m("M29 块队：any_left 取反", "!self.blocks_not_taken_locked().is_empty()", "self.blocks_not_taken_locked().is_empty()")
m("M30 卡线程那一圈：判不了（Refused）也当没事接着领",
  "BlockJudgedOnACard::Refused { reason } => return CardLeftTheBlocks::Refused { reason },",
  "BlockJudgedOnACard::Refused { reason: _ } => {}")
m("M31 卡线程那一圈：写库那头走了也接着领",
  "BlockHandedToTheStore::TheStoreIsGone => return CardLeftTheBlocks::TheStoreIsGone,",
  "BlockHandedToTheStore::TheStoreIsGone => {}")
m("M32 几轮：还有块没写时才算判完（<= 1）",
  "if blocks_left_unstored == 0 {\n        return AfterARoundOfTheCards::EveryBlockStored;", "if blocks_left_unstored <= 1 {\n        return AfterARoundOfTheCards::EveryBlockStored;")
m("M33 按块序写库：写一块算两块",
  "Ok(()) => self.blocks_stored += 1,", "Ok(()) => self.blocks_stored += 2,")
m("M34 报到：报多了不 panic（checked_sub 换成 saturating）",
  ".checked_sub(1)\n            .expect(\"每一份报到只报一次：领一次加一，报一次减一\")", ".saturating_sub(1)")
m("M35 卡线程那一圈：交回的块还给队里之后仍把这张卡的 OOM 当没事（返回 NothingLeftOrStopped）",
  "                blocks.give_back(pending_index);\n                return CardLeftTheBlocks::OutOfMemory { reason };",
  "                blocks.give_back(pending_index);\n                let _ = reason;\n                return CardLeftTheBlocks::NothingLeftOrStopped;")
if __name__ == "__main__":
    a = [int(x) for x in sys.argv[1:]] or list(range(len(b.M)))
    for i in a:
        print(b.run(i), flush=True)
