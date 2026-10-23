enum Box<a> {
    Box(a)
}
function main(flag: bool) returns (word) {
    let seed: comptime<word> = 3;
    match (Box(seed), flag) {
        case (Box(inner), selected) {
            inner = primAddWord(inner, ((selected) ? 1 : 2));
            return inner;
        }
    }
}
