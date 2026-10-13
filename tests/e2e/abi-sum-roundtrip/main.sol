import * from std;
import * from std.dispatch;
import * from std.Generic;
import * from std.ABIGeneric;

// ABI wire roundtrips use one keccak256("Constructor(field signatures)") tag.
// Dynamic ADTs have a single offset to [tag][fields]; three variants do not
// introduce a second discriminant or offset. Static ADTs reserve the largest
// payload plus exactly one tag word. Unknown variant tags must revert.
enum D2 { L(uint256), R(memory<bytes>) }                // dynamic (shallow)
enum D3 { X(uint256), Y(uint256), Z(memory<bytes>) }   // dynamic, three variants
enum S2 { P(uint256), Q(uint256) }                      // static

enum S3 { A, B(uint256), C(uint256, uint256, uint256) }

contract SumRoundtrip {
  constructor() {}

  // dynamic, shallow: decode a sum(uint256, bytes) then re-encode it.
  function rtD2(x: D2) public returns (D2) {
    return x;
  }

  // Three variants still use one wire tag and one dynamic offset.
  function rtD3(x: D3) public returns (D3) {
    return x;
  }

  function rtS3(x: S3) public returns (S3) { return x; }

  // static control: inline layout must round-trip unchanged.
  function rtS2(x: S2) public returns (S2) {
    return x;
  }
}
