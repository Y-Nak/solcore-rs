function main(flag: bool) returns (word) {
    if (flag) {
        let x: word = 7;
    } else {}
    let result: word;
    assembly {
        function read() -> result {
            result := x
        }
        result := read()
    }
    return result;
}
