import * from std;
import * from std.dispatch;

enum Box { Box(uint256, uint256) }

contract PatternBindingOrder {
    // #[(1, 2) -> 2]
    // #[(7, 9) -> 9]
    function swap(x: uint256, y: uint256) public returns (uint256) {
        match (x, y) { case (y, x) { return x; } }
    }

    // #[(1, 2, 3) -> 3]
    function cycle(x: uint256, y: uint256, z: uint256) public returns (uint256) {
        match (x, y, z) { case (y, z, x) { return x; } }
    }

    // #[(1, 2, 3) -> 3]
    function nested(x: uint256, y: uint256, z: uint256) public returns (uint256) {
        match ((x, (y, z))) { case (z, (x, y)) { return y; } }
    }

    // #[(1, 2) -> 2]
    function boxed(x: uint256, y: uint256) public returns (uint256) {
        let box = Box(x, y);
        match (box, y) { case (Box(y, x), box) { return x; } }
    }
}
