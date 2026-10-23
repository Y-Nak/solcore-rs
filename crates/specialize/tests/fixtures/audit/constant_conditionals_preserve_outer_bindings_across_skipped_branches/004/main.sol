function compute() returns (word) {
    let x: word = 1;
    if (true) {} else {
        let x: word = 7;
    }
    return x;
}
function main() returns (word) {
    return compute();
}
