function compute() returns (word) {
    let x: word = 7;
    let y: bool = true;
    match (y) {
        case (x) {}
    }
    assembly {
        x := add(x, 1)
    }
    return x;
}
function main() returns (word) {
    return compute();
}
