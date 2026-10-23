contract MemoryContract {
    function main() returns (word) {
        let result: word;
        assembly {
            mstore(0, 42)
            result := mload(0)
        }
        return result;
    }
}
