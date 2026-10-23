function requireStatic(comptime x: word) returns (word) {
    return x;
}
function main(flag: bool) returns (word) {
    let seed: comptime<word> = 7;
    match (flag) {
        case true {
            {
                if (flag) {
                    let seed: word = 1;
                } else {}
            }
            let checked: comptime<word> = requireStatic(seed);
        }
        case false {}
    }
    return requireStatic(seed);
}
