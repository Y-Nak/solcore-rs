function c() returns (comptime<word>) {
    if (true) {
        return 7;
    } else {
        let unused: word = 1;
        return 5;
    }
    return 0;
}
function main() returns (word) {
    return c();
}
