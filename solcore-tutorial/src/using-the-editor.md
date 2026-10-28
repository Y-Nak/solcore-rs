# Using the editor

Each lesson includes an editable example with the files it needs.
Compilation and execution happen in your browser; your source is not sent to a compilation service.

## Open an example

Click **Try here** in a chapter to open its editor.
The editor loads only when requested, so the first run can take longer while the browser loads the compiler.
**Close editor** closes it while keeping your saved code.
Opening another exercise on the same page closes the previous editor.
Use **Open in full tab** for more space or the full Playground controls.

The inline editor uses the **solc** backend by default.
The exercises are also tested with Sonatina, which you can select in the full-tab view.
The Extensions project currently needs solc to run within the browser runner's deployment gas limit.

## Compile, call, and test

**Compile** checks the program and produces output for inspection; it does not execute a function.
In the **Run** tab, select **Try calls** to choose a public function, enter its arguments, and click **Call function**.
To run the example's test comments, select **Run tests** and click **Run all tests from start**.

| Action | Use it for | State behavior |
| --- | --- | --- |
| **Call function** under **Try calls** | Exploring public functions manually | Changes persist between calls until reset or source changes |
| **Run all tests from start** | Running the file's test comments | A fresh test deployment; sends persist within it, while expectations discard state changes |
| **Run test** above a test comment | Checking one expectation or send | The runner replays preceding sends for that contract before running the selected test |

Opening an editor selects code and a view; it does **not** run the program.
Tests and manual calls use separate contract instances.
A passed test does not prepare the state of your manual calls.
See [Public functions and tests](tests.md) and [Contracts and persistent state](state.md) for examples.

## Keep your experiments

Each tutorial project saves its source files in a separate workspace in this browser.
Returning to that project restores your edits, and opening another lesson does not replace them.
Chapters using the same project share its workspace.
The inline editor and its **Open in full tab** view use the same saved source; keep only one editor open for a given project.

Choose **Reset project** to restore that project's original source.
To keep your source but clear manual contract state, use **Reset contract** instead.
Closing an editor or reloading the page also starts a new execution session; saved source is not saved contract state.

Copy code you want to keep to a local file, especially before clearing site data or using a private window.
A shared project URL identifies the bundled project; it does not contain your edits, arguments, or contract state.

## When a result surprises you

| Symptom | What to check |
| --- | --- |
| The inline editor does not appear or a project cannot load | Try **Open in full tab** and reload the page. If that also fails, report the page URL to whoever hosts the tutorial. |
| Reopening an example keeps your edits | Use **Reset project** to restore the original source, after saving anything you want to keep. |
| Opening a full tab does not restore contract state | Repeat the manual calls that set up your state; the views share saved source, not execution sessions. |
| Tests pass but a manual call sees different state | Tests use a separate deployment. Repeat setup through **Try calls** or reset the manual instance. |
| An individual stateful test passes without manual setup | The runner replays preceding sends for that contract. |
| A function is missing from **Try calls** | Check that it is `public` and compilation succeeded. |
| Source changes make old results look stale | Run again; retained history is not a result for the edited code. |
| A loop halts | Reduce the input. Execution has a gas limit, even for finite loops. |
| Extensions compiles with Sonatina but cannot deploy | Use **solc** for this project. The current Sonatina output exceeds the runner's 1,000,000 gas limit during deployment. |
| Imports or trait methods fail to resolve | Restore the lesson's imports and helper files, and check that the required trait implementations are available. |

For a reproducible issue, save the exact source and helper files, record the chosen backend, and copy the diagnostic or failing inputs.
A project URL alone identifies the bundled source, not your edited program.
