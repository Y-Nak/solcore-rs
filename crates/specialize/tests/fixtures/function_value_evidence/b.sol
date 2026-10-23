import * from base;
import base;
export { localB, qualifiedB, argumentB, nestedB, genericB };
impl Phantom<word, bool> {
    function pick(x: word) returns (word) {
        return 4;
    }
}
function localB() returns (word) {
    let f = run;
    return f(0);
}
function qualifiedB() returns (word) {
    let f = base.run;
    return f(0);
}
function argumentB() returns (word) {
    return apply(run, 0);
}
function nestedB() returns (word) {
    return relay(run, 0);
}
function genericB() returns (word) {
    return genericValue(0);
}
