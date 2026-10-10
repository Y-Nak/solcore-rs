contract ArrayImplicitRuntime {
    function main() returns (word) {
        // An explicit runtime main owns memory setup and raw EVM output.
        assembly { mstore(0x40, 0x80) }
        let first: word = 10;
        let second: word = 20;
        let values = [first, second];
        let result: word;
        assembly {
            let end := mload(0x40)
            let length := mload(sub(end, 96))
            let first := mload(sub(end, 64))
            let second := mload(sub(end, 32))
            result := add(mul(length, 10000), add(mul(first, 100), second))
            mstore(0, result)
            return(0, 32)
        }
        return result;
    }
}
