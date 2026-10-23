function main(flag: bool) returns (word) {
    let seed: comptime<word> = 7;
    let result: word = 0;
    {
        if (flag) {
            let seed: word = seed;
            result = seed;
        } else {}
    }
    return result;
}
