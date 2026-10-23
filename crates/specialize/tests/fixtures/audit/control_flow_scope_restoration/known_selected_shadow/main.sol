function requireStatic(comptime x: word) returns (word) {
    return x;
}
function main() returns (word) {
    let seed: word = 7;
    {
        if (false) {
            seed = 9;
        } else {
            let seed: word = 1;
        }
    }
    return requireStatic(seed);
}
