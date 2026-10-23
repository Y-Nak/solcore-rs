function get(comptime value: (word, word)) returns (word) {
    match (value) {
        case ((head, _)) {
            return head;
        }
    }
}
