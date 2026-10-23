function main(flag: bool) returns (word) {
    let seed: word = 7;
    match (flag) {
        case (true) {
            seed = 9;
        }
        case (false) {}
    }
    return seed;
}
