function pick() returns (function () returns (word)) {
    let x: word;
    return lam () -> word {
        if (true) {
            return 7;
        } else {
            return x;
        }
    };
}
function main() returns (word) {
    let f = pick();
    return f();
}
