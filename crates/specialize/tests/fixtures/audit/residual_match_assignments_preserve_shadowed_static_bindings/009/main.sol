import * from std;
enum Box {
    Box(bool)
}
function main(flag: bool, input: word) returns (word) {
    let seed: comptime<word> = 7;
    match ((flag, flag)) {
        case ((_, seed)) {
            seed = true;
        }
    }
    return seed;
}
