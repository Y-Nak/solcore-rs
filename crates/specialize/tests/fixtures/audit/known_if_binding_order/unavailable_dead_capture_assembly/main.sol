function pick() returns (function () returns (word)) {
    let x: word;
    return lam () -> word {
        if (true) {
            return 7;
        } else {
            let result: word;
            assembly {
                result := x
            }
            return result;
        }
    };
}
function main() returns (word) {
    let f = pick();
    return f();
}
