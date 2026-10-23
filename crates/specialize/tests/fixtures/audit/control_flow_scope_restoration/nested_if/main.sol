function requireStatic(comptime x: word) returns (word) {
    return x;
}
function main(flag: bool) returns (word) {
    let seed: comptime<word> = 7;
    {
        if (flag) {
            if (flag) {
                let seed: word = 1;
            } else {}
        } else {}
    }
    return requireStatic(seed);
}
