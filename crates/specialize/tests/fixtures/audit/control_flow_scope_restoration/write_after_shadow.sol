function requireStatic(comptime x: word) returns (word) {
    return x;
}
function main(flag: bool) returns (word) {
    let seed: comptime<word> = 7;
    {
        if (flag) {
            let seed: word = 1;
            seed = 2;
        } else {
            seed = 3;
        }
    }
    return requireStatic(seed);
}
