function main(flag: bool) returns (word) {
    let x: word = 1;
    if (flag) {
        let x: bool = false;
    } else {
        let x: word = 3;
    }
    return x;
}
