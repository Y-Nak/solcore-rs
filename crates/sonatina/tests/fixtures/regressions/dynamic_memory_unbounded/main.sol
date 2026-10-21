function main() returns (word) {
    let result: word;
    assembly {
        let n := calldatasize()
        mstore(0, 42)
        result := keccak256(0, n)
    }
    return result;
}
