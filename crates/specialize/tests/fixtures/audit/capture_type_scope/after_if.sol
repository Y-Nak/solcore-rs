function main(flag: bool, n: word) returns (word) {
    if (flag) {
        let value: word = n;
    } else {}
    let get: function () returns (word) = lam () -> word {
        assembly {
            value := add(value, 1)
        }
        return value;
    };
    return get();
}
