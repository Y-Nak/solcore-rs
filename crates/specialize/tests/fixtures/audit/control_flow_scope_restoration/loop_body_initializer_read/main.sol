function main(flag: bool) returns (word) {
    let seed: comptime<word> = 7;
    let result: word = 0;
    {
        for (; flag;) {
            let seed: word = seed;
            result = seed;
            break;
        }
    }
    return result;
}
