import {setValue as importedSetValue} from util;

contract C {
    function setValue(x: word) {
        importedSetValue(x);
    }

    function main() {}
}
