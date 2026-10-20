import * from std;
import * from std.signed;
import * from std.unsigned;

contract IntEcho {
    function echo(a: int8, b: int256, c: uint64) public returns (int8, int256, uint64) {
        return (a, b, c);
    }
}
