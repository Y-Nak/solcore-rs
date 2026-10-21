function main(flag: bool, n: word) returns (word) {
    for (; flag;) {
        let value: word = n;
        break;
    }
    let get: function () returns (word) = lam () -> word {
        assembly {
            value := add(value, 1)
        }
        return value;
    };
    return get();
}
