import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {runInNewContext} from 'node:vm';

const source = readFileSync(new URL('app.js', import.meta.url), 'utf8');
for (const clipboardAvailable of [true, false]) {
  let click;
  let copied;
  let selected;
  const snippet = {textContent: 'first command\nsecond command'};
  const status = {textContent: ''};
  const button = {
    dataset: {copy: 'snippet'},
    textContent: 'Copy',
    addEventListener: (_, handler) => { click = handler; },
  };
  runInNewContext(source, {
    document: {
      querySelectorAll: () => [button],
      getElementById: (id) => id === 'snippet' ? snippet : status,
      createRange: () => ({selectNodeContents: (node) => { selected = node; }}),
    },
    navigator: {clipboard: {writeText: async (text) => {
      if (!clipboardAvailable) throw new Error('Clipboard denied');
      copied = text;
    }}},
    window: {getSelection: () => ({removeAllRanges() {}, addRange() {}})},
    setTimeout() {},
  });
  await click();
  if (clipboardAvailable) {
    assert.equal(copied, snippet.textContent);
    assert.equal(button.textContent, 'Copied');
    assert.match(status.textContent, /copied/);
  } else {
    assert.equal(selected, snippet);
    assert.equal(button.textContent, 'Select text');
    assert.match(status.textContent, /manual copying/);
  }
}
console.log('Site copy controls: clipboard success and denial checks passed.');
