function pick() returns (word) {
    let x: word = 7;
    if (true) {
        return x;
    } else {
        let x: word = 9;
        return x;
    }
}
function main() returns (word) {
    let value: comptime<word> = pick();
    return value;
}
