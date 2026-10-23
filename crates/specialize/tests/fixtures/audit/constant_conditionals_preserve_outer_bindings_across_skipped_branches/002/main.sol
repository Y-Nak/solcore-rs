function compute() returns (word) {
    let x: word = 1;
    if (false) {
        let x: word = 42;
    } else {}
    return x;
}
function main() returns (word) {
    return compute();
}
