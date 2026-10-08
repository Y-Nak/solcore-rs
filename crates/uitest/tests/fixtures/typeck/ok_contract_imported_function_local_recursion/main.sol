function setValue(x: word) {}

contract C {
    function setValue(x: word) {
        match (x) {
            case 0 {}
            case _ { setValue(0); }
        }
    }

    function main() {}
}

contract D {
    function setValue(x: word) {}

    function main() {}
}
