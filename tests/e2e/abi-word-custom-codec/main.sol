import * from std;
import * from std.dispatch;
import { mload, mstore } from std.opcodes;

// Applications could supply raw-word codecs before the standard fallback
// existed. Array elements must retain those explicitly selected dictionaries.
impl ABIEncode<word> {
    function encodeInto(value: word, base: word, offset: word, tail: word) returns (word) {
        mstore(base + offset, value + 100);
        return tail;
    }
}

impl<reader> ABIDecode<ABIDecoder<word, reader>, word> where reader: WordReader {
    function decode(decoder: ABIDecoder<word, reader>, offset: word) returns (word) {
        return WordReader.read(WordReader.advance(decoder, offset)) - 100;
    }
}

contract CustomWordCodec {
    // The public interface still uses canonical uint256 codecs; only internal
    // raw-word values use the application-specific representation.
    // #[(7) -> 7]
    function scalar(value: uint256) public returns (uint256) {
        let encoded = abi_encode(Typedef.rep(value));
        assert(mload(MemoryPointer.ptr(encoded)) == Typedef.rep(value) + 100);
        let decoded: word = abi_decode(encoded, @word, @MemoryWordReader);
        return uint256(decoded);
    }

    // #[() -> 3]
    function array() public returns (uint256) {
        let values: memory<DynArray<word>> = [1, 2];
        let encoded = abi_encode(values);
        let start = MemoryPointer.ptr(encoded);
        assert(mload(start + 64) == 101);
        assert(mload(start + 96) == 102);
        let decoded: memory<DynArray<word>> =
        abi_decode(encoded, @memory<DynArray<word>>, @MemoryWordReader);
        return uint256(decoded[0] + decoded[1]);
    }
}
