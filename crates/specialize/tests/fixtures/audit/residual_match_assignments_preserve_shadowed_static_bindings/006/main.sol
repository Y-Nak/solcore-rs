import * from std;
enum Box {
    Box(bool)
}
function main(flag: bool, input: word) returns (word) {
    let seed: comptime<word> = 7;
    match (flag) {
        case (seed) {
            for (; seed;) {
                seed = false;
            }
        }
    }
    return seed;
}
