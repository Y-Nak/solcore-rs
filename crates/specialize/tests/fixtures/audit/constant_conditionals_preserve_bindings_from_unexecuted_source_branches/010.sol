function compute() returns (word) {
    if (true) {} else {
        for (; false;) {
            let x: word = 7;
        }
    }
    return x;
}
function main() returns (word) {
    return compute();
}
