import * from std;
import * from std.dispatch;
import {mstore} from std.opcodes;

// Exercise both storage encodings and the final partial word when overwriting
// longer values. Dirty input padding must not survive the round trip.
contract StorageBytes {
    content: bytes;

    function set(value: memory<bytes>) public {
        // Poison the bytes immediately after the logical payload, including
        // padding in its final memory word, before the storage helper runs.
        mstore(Typedef.rep(value) + 32 + MemorySize.len(value), ~0);
        content = value;
    }

    function get() public returns (memory<bytes>) {
        return content;
    }
}
