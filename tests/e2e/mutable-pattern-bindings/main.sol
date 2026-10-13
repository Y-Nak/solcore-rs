import * from std;
import * from std.dispatch;

contract MutablePatternBindings {
  count: uint256;

  function changeCount() returns (()) { count = uint256(42); return; }
  // #[() -> 42]
  function assemblyWrite() public returns (uint256) {
    let result: word;
    let values: (word, word) = (0, 0);
    match (values) {
      case (v0, v1) {
        assembly { v1 := 42 }
        let after: word = v1;
        assembly { result := after }
      }
    }
    return uint256(result);
  }

  // #[() -> 42]
  function ordinaryWrite() public returns (uint256) {
    let values: (word, word) = (0, 0);
    match (values) {
      case (x, y) { y = 42; return uint256(y); }
    }
  }

  // #[() -> 83]
  function outerShadow() public returns (uint256) {
    let y: word = 41;
    let result: word;
    let values: (word, word) = (0, 0);
    match (values) {
      case (x, y) { assembly { y := 42 result := y } }
    }
    assembly { result := add(result, y) }
    return uint256(result);
  }

  // #[() -> 7]
  function assemblyShadow() public returns (uint256) {
    let result: word;
    let values: (word, word) = (0, 7);
    match (values) {
      case (x, y) {
        assembly { { let y := 99 result := y } result := y }
      }
    }
    return uint256(result);
  }
  // #[() -> 42]
  function firstArm() public returns (uint256) {
    let values: (word, word) = (0, 0);
    match (values) {
      case (x, y) { assembly { y := 42 } return uint256(y); }
      default { return uint256(99); }
    }
  }

  // #[() -> 48]
  function innerShadow() public returns (uint256) {
    let y: word = 41;
    let result: word;
    let values: (word, word) = (0, 0);
    match (values) {
      case (x, y) {
        let y: word = 7;
        assembly { result := y }
      }
    }
    assembly { result := add(result, y) }
    return uint256(result);
  }

  // #[() -> 41]
  function readOnlyShadow() public returns (uint256) {
    let y: word = 41;
    let result: word;
    let values: (word, word) = (0, 0);
    match (values) { case (x, y) { result = y; } }
    assembly { result := add(result, y) }
    return uint256(result);
  }

  // #[() -> 42]
  function comptimeAfterMatch() public returns (uint256) {
    let result: word = 0;
    let input: word = 0;
    match (input) { case 0 { result = 42; } default { result = 99; } }
    let answer: comptime<word> = result;
    return uint256(answer);
  }

  // #[() -> 42]
  function comptimeAfterBlock() public returns (uint256) {
    let result: word = 0;
    { result = 42; }
    let answer: comptime<word> = result;
    return uint256(answer);
  }

  // #[() -> 42]
  function nestedScopeUpdate() public returns (uint256) {
    let result: word = 1;
    let values: (word, word) = (0, 0);
    match (values) {
      case (x, y) {
        result = 42;
        { let result: word = 99; }
      }
    }
    let answer: comptime<word> = result;
    return uint256(answer);
  }

  // #[() -> 42]
  function writeBeforeShadow() public returns (uint256) {
    let result: word = 1;
    let input: word = 0;
    match (input) {
      case 0 { result = 42; let result: word = 99; }
      default { result = 99; }
    }
    let answer: comptime<word> = result;
    return uint256(answer);
  }

  // #[() -> 42]
  function storageShadow() public returns (uint256) {
    count = uint256(1);
    let values: (word, word) = (0, 0);
    match (values) { case (x, count) { changeCount(); } }
    return count;
  }

}
