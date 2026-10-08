contract Before {
    function setValue(x: word) returns (word) {
        return setValue(x);
    }
}

import {setValue} from util;

contract After {
    function setValue(x: word) returns (word) {
        return setValue(x);
    }
}
