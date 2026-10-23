function main(flag: bool) returns (word) {
    let seed: comptime<word> = 7;
    for (; flag;) {
        match (0) {
            case seed {
                assembly {
                    seed := 1
                }
            }
        }
        break;
    }
    return seed;
}
