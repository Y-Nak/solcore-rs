function main() returns (word) {
    let result: word;
    assembly {
        let n := and(calldatasize(), 31)
        mstore(0, 42)
        result := keccak256(0, n)
    }
    return result;
}
