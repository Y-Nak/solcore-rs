function compute() returns (word) {
    if (false) {
        let x: word = 42;
    } else {}
    return x;
}
function main() returns (word) {
    return compute();
}
