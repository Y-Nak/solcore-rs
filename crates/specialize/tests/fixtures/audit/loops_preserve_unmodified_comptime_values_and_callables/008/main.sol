function main(flag: bool) returns (word) {
    let get: function () returns (word) = lam () -> word {
        return 7;
    };
    for (; flag;) {
        assembly {
            mstore(0, 99)
        }
        break;
    }
    return get();
}
