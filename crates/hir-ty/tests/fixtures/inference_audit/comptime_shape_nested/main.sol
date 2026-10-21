function get(comptime value: (word, (word, word))) returns (word) {
    match (value) {
        case ((_, (head, _))) {
            return head;
        }
    }
}
