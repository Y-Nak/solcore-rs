contract Before {
    function setValue(x: word) {
        setValue(x);
    }
}

import {original as setValue} from util;

contract After {
    function setValue(x: word) {
        setValue(x);
    }
}
