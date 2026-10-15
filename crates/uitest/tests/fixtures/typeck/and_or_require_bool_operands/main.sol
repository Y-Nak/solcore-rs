function and(x: word, y: word) returns (bool) {
    return true;
}

function or(x: word, y: word) returns (bool) {
    return true;
}

contract C {
    function main() public returns (bool) {
        return 1 && 2;
    }

    function other() public returns (bool) {
        return 1 || 2;
    }
}
