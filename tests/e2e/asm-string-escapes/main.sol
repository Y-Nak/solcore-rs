contract AssemblyStringEscapes {
    function main() returns (word) {
        let value: word;
        assembly {
            value := "a\n\t\"\\b"
            switch "\n"
            case "\n" { }
            default { revert(0, 0) }
            mstore(0, value)
            return(0, 32)
        }
        return value;
    }
}
