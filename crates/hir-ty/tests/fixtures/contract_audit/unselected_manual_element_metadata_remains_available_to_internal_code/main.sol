import * from std;
import * from std.dispatch;
impl ABIElementAttribs<address> {
    function headSize(p: Proxy<address>) returns (word) {
        return 64;
    }
}
impl ABITypeString<address> {
    function typeStr(p: Proxy<address>) returns (string) {
        return "bytes32";
    }
}
function privateLayout() returns (word) {
    return ABIElementAttribs.headSize(@address);
}
function privateSignature() returns (string) {
    return ABITypeString.typeStr(@address);
}
contract C {
    function take(values: calldata<array<uint256>>) public {}
}
