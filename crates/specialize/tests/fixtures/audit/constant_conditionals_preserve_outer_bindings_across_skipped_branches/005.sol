function main() returns (word) {
    let x: word = 1;
    if (true) {
        let x: word = 42;
    } else {
        let x: word = 7;
    }
    return x;
}
