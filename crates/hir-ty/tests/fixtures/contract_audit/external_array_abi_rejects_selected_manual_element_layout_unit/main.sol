import * from std;
import * from std.dispatch;
impl ABIElementAttribs<()> {
    function headSize(p: Proxy<()>) returns (word) {
        return 64;
    }
}
contract C {
    function take(values: calldata<array<()>>) public returns (uint256) {
        return uint256(0);
    }
}
