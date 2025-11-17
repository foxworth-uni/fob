import { EditorState } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { defaultKeymap } from "@codemirror/commands";
import { javascript } from "@codemirror/lang-javascript";

const startState = EditorState.create({
  doc: "console.log('Hello, CodeMirror!')\n",
  extensions: [
    keymap.of(defaultKeymap),
    javascript(),
  ],
});

const view = new EditorView({
  state: startState,
  parent: document.body,
});

console.log("CodeMirror editor initialized!");
