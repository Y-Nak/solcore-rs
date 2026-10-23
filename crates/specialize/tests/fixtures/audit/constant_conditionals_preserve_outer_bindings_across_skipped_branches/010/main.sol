function compute() returns (word) {
    let x: word = 1;
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
