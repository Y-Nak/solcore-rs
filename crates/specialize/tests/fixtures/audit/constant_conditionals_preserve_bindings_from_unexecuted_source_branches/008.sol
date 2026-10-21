function compute() returns (word) {
    if (true) {} else {
        if (true) {
            let x: word = 7;
        } else {}
    }
    return x;
}
function main() returns (word) {
    return compute();
}
