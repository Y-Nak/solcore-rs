function compute() returns (word) {
    let x: word = 1;
    let y: word = 2;
    match ((x, y)) {
        case ((y, x)) {
            return x;
        }
    }
    return x;
}
function main() returns (word) {
    return compute();
}
