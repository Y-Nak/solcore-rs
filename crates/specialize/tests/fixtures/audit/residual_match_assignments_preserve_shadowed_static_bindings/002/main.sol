import * from std;
enum Box {
    Box(bool)
}
function main(flag: bool, input: word) returns (word) {
    let seed: comptime<word> = 7;
    match (input) {
        case (seed) {
            assembly {
                seed := 1
            }
        }
    }
    return seed;
}
