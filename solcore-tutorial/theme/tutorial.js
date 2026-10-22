// Reuse one working tab across lessons, keeping the book open and avoiding
// multiple playground tabs competing for its one saved workspace.
window.name = "solcore-tutorial";
for (const link of document.querySelectorAll('a[href*="#/examples/"]')) {
  link.target = "solcore-tutorial-playground";
  link.dataset.playgroundLink = "true";
  link.title = "Open this example in the playground tab";
}
