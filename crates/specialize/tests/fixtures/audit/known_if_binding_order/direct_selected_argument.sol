function consume(comptime x: word) returns (word) {
    return x;
}
function main() returns (word) {
    let x: word = 0;
    if (true) {
        x = 7;
    } else {
        let unused: word = 1;
    }
    return consume(x);
}
