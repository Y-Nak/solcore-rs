import { allB } from b;
import { allA } from a;
function main(x: word, y: word) returns (word) {
    return primAddWord(allB(x, y), allA(x, y));
}
