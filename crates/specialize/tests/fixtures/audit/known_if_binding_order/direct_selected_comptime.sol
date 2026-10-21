function main() returns (word) {
    if (true) {
        let x: comptime<word> = 7;
    } else {
        let unused: word = 1;
    }
    return x;
}
