function main() returns (word) {
    let x: word = 1;
    if (true) {} else {
        if (true) {
            let x: word = 7;
        } else {}
    }
    return x;
}
