import * from std;

function main() returns (uint256) {
    let xs: storage<array<uint256>> = storage(0x100);
    return xs[uint256(0)];
}
