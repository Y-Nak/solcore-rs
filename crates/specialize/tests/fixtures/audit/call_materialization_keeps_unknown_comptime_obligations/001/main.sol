function main(input: word) returns (word) {
    let seed: comptime<word> = input;
    return primAddWord(input, seed);
}
