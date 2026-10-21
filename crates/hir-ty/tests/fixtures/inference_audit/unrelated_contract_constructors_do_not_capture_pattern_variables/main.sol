contract Other {
    enum Hidden {
        value
    }
}
function main(x: word) returns (word) {
    match (x) {
        case value {
            return value;
        }
    }
}
