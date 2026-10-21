function compute() returns (word) {
    let x: word = 1;
    {
        x = 3;
        let x: word = 4;
        x = 5;
    }
    return x;
}
function main() returns (word) {
    return compute();
}
