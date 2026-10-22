contract C {
    function main() returns (word) {
        let f = lam (x) {
            return 0;
        };
        return f(f);
    }
}
