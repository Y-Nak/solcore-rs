function main() returns (word) {
    let x: word = 1;
    if (true) {
        let x: bool = false;
    }
    let get = lam () -> word {
        return x;
    };
    return get();
}
