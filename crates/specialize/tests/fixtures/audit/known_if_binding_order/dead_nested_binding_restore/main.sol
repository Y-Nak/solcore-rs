function pick() returns (word) {
    let x: word = 7;
    {
        if (false) {
            if (true) {
                let x: word = 9;
            }
        } else {}
    }
    return x;
}
function main() returns (word) {
    let value: comptime<word> = pick();
    return value;
}
