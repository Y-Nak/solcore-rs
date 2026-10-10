function writeSlot() returns (word) {
    assembly { sstore(0, add(sload(0), 1)) }
    return 0;
}
function second(first: word, value: word) returns (word) { return value; }
contract IgnoredCallEffects {
    function main() returns (word) {
        // Even an unused argument must execute exactly once.
        let result: word = second(writeSlot(), 7);
        assembly {
            mstore(0, add(mul(result, 10), sload(0)))
            return(0, 32)
        }
        return result;
    }
}
