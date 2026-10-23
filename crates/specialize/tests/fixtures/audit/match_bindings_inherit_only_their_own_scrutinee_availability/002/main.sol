enum Box {
    Box(word)
}
function take(comptime value: bool) returns (bool) {
    return value;
}
function get(comptime x: Box, flag: bool) returns (bool) {
    match (x, flag) {
        case (Box(y), y2) {
            return take(y2);
        }
    }
}
function main(flag: bool) returns (bool) {
    return get(Box(3), flag);
}
