function main(flag: bool) returns (word) {
    let x: word = 1;
    if (flag) {} else {
        let x: word = 7;
    }
    return x;
}
