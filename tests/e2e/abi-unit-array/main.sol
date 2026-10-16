import * from std;
import * from std.dispatch;

type UnitAlias = ();
type Units = calldata<array<UnitAlias>>;

contract UnitArrays {
  seed: uint256;

  constructor(value: uint256) { seed = value; }

  function units(values: Units) public returns (uint256) {
    return Length.length(values);
  }

  function at(values: Units, i: uint256) public returns (uint256) {
    let value: UnitAlias = values[i];
    return Length.length(values);
  }

  function nested(values: calldata<array<Units>>, i: uint256) public returns (uint256) {
    let inner: Units = values[i];
    return Length.length(inner);
  }

  function combined(prefix: uint256, values: Units) public returns (uint256) {
    return prefix + Length.length(values) + seed;
  }

  function wrapped(values: calldata<array<(Units, uint256)>>) public returns (uint256) {
    match (values[0]) {
      case (units, number) { return Length.length(units) + number; }
    }
  }

  // Empty argument/return products must keep their existing dispatch identity.
  function empty() public returns (UnitAlias) { return; }
}
