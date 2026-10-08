import * from std;
import * from std.dispatch;
import {setValue} from lib;
contract C {
    function setValue(x: uint256) public {
        setValue(x);
    }
}
