function main() returns (word) {
    let x: word = 1;
    if (true) {} else {
        for (; false;) {
            let x: word = 7;
        }
    }
    return x;
}
