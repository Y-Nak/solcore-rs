function main(flag: bool) returns (word) {
    let seed: comptime<word> = 7;
    for (; flag;) {
        {
            let seed: word = 8;
            seed = 9;
        }
        break;
    }
    return seed;
}
