import * from a;
import * from b;
function main() returns(word) {
  return primAddWord(primAddWord(directA(),directB()),
    primAddWord(primAddWord(genericA(),genericB()),primAddWord(explicitA(),explicitB())));
}
