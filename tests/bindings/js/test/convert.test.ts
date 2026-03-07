import { describe, it, expect } from 'vitest';
import {
  Html2Markdown,
  OptionsError,
  type HeadingStyle,
  type ListItemIndent,
} from '../lib/html2markdown_uniffi.js';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const { convert, convertWith, defaultOptions, defaultStringifyOptions } = Html2Markdown;

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(__dirname, '../../../..');

describe('convert', () => {
  it('heading', () => {
    expect(convert('<h1>Hello</h1>')).toBe('# Hello\n');
  });

  it('empty string', () => {
    expect(convert('')).toBe('');
  });

  it('paragraph', () => {
    expect(convert('<p>Hello</p>')).toBe('Hello\n');
  });

  it('emphasis', () => {
    expect(convert('<em>Hello World.</em>')).toBe('*Hello World.*\n');
  });

  it('strong', () => {
    expect(convert('<strong>Hello World.</strong>')).toBe('**Hello World.**\n');
  });

  it('link', () => {
    const html = '<a href="http://example.com" title="example">example</a>';
    expect(convert(html)).toBe('[example](http://example.com "example")\n');
  });

  it('image', () => {
    const html = '<img src="http://example.com" alt="example">';
    expect(convert(html)).toBe('![example](http://example.com)\n');
  });

  it('code', () => {
    expect(convert('<code>toString()</code>')).toBe('`toString()`\n');
  });

  it('blockquote', () => {
    const html = '<blockquote><p>This is a blockquote.</p></blockquote>';
    expect(convert(html)).toBe('> This is a blockquote.\n');
  });

  it('unordered list', () => {
    const html = '<ul><li>Alpha</li><li>Bravo</li><li>Charlie</li></ul>';
    expect(convert(html)).toBe('* Alpha\n* Bravo\n* Charlie\n');
  });

  it('ordered list', () => {
    const html = '<ol><li>Alpha</li><li>Bravo</li><li>Charlie</li></ol>';
    expect(convert(html)).toBe('1. Alpha\n2. Bravo\n3. Charlie\n');
  });
});

describe('convertWith', () => {
  it('default options matches convert', () => {
    const html = '<h1>Hello</h1>';
    expect(convertWith(html, defaultOptions())).toBe(convert(html));
  });
});

describe('defaultOptions', () => {
  it('stringify options', () => {
    const opts = defaultStringifyOptions();
    expect(opts.headingStyle).toBe('Atx' satisfies HeadingStyle);
    expect(opts.bullet).toBe('*');
    expect(opts.bulletOrdered).toBe('.');
    expect(opts.emphasis).toBe('*');
    expect(opts.strong).toBe('*');
    expect(opts.fence).toBe('`');
    expect(opts.rule).toBe('*');
    expect(opts.ruleRepetition).toBe(3);
    expect(opts.ruleSpaces).toBe(false);
    expect(opts.closeAtx).toBe(false);
    expect(opts.listItemIndent).toBe('One' satisfies ListItemIndent);
    expect(opts.incrementListMarker).toBe(true);
    expect(opts.quote).toBe('"');
    expect(opts.fences).toBe(true);
    expect(opts.resourceLink).toBe(false);
  });

  it('conversion options', () => {
    const opts = defaultOptions();
    expect(opts.newlines).toBe(false);
    expect(opts.checked).toBeNull();
    expect(opts.unchecked).toBeNull();
    expect(opts.quotes).toEqual(['"']);
  });
});

describe('valid option mutations', () => {
  it.each(['-', '+', '*'])('bullet "%s" changes output', (char) => {
    const opts = defaultOptions();
    opts.stringify.bullet = char;
    const result = convertWith('<ul><li>A</li></ul>', opts);
    expect(result).toContain(`${char} A`);
  });

  it.each(['*', '_'])('emphasis "%s" changes output', (char) => {
    const opts = defaultOptions();
    opts.stringify.emphasis = char;
    const result = convertWith('<em>A</em>', opts);
    expect(result).toContain(`${char}A${char}`);
  });

  it.each(['`', '~'])('fence "%s" changes output', (char) => {
    const opts = defaultOptions();
    opts.stringify.fence = char;
    opts.stringify.fences = true;
    const result = convertWith('<pre><code>x</code></pre>', opts);
    expect(result).toContain(char.repeat(3));
  });

  it('setext headings', () => {
    const opts = defaultOptions();
    opts.stringify.headingStyle = 'Setext';
    const result = convertWith('<h1>Title</h1>', opts);
    expect(result).toContain('====');
  });

  it('close ATX headings', () => {
    const opts = defaultOptions();
    opts.stringify.closeAtx = true;
    const result = convertWith('<h1>Title</h1>', opts);
    expect(result.trim()).toBe('# Title #');
  });
});

describe('error handling', () => {
  it('invalid bullet throws OptionsError', () => {
    const opts = defaultOptions();
    opts.stringify.bullet = 'x';
    expect(() => convertWith('<p>hi</p>', opts)).toThrow(OptionsError);
  });

  it('invalid bullet error has correct variant fields', () => {
    const opts = defaultOptions();
    opts.stringify.bullet = 'x';
    try {
      convertWith('<p>hi</p>', opts);
      expect.unreachable('should have thrown');
    } catch (e) {
      expect(e).toBeInstanceOf(OptionsError);
      const err = e as OptionsError;
      expect(err.variant.tag).toBe('InvalidOption');
      if (err.variant.tag === 'InvalidOption') {
        expect(err.variant.field).toBe('bullet');
        expect(err.variant.value).toBe('x');
      }
    }
  });

  it('empty bullet throws OptionsError', () => {
    const opts = defaultOptions();
    opts.stringify.bullet = '';
    expect(() => convertWith('<p>hi</p>', opts)).toThrow(OptionsError);
  });

  it('invalid bullet_ordered throws OptionsError', () => {
    const opts = defaultOptions();
    opts.stringify.bulletOrdered = 'x';
    expect(() => convertWith('<ol><li>A</li></ol>', opts)).toThrow(OptionsError);
  });

  it('invalid rule throws OptionsError', () => {
    const opts = defaultOptions();
    opts.stringify.rule = 'x';
    expect(() => convertWith('<hr>', opts)).toThrow(OptionsError);
  });

  it('invalid rule_repetition throws OptionsError', () => {
    const opts = defaultOptions();
    opts.stringify.ruleRepetition = 2;
    expect(() => convertWith('<hr>', opts)).toThrow(OptionsError);
  });
});

describe('fixtures', () => {
  const fixturesDir = resolve(projectRoot, 'test-fixtures');
  const fixtureNames = [
    'a', 'blockquote', 'br', 'code', 'em', 'heading',
    'img', 'ol', 'paragraph', 'strong', 'table', 'ul',
  ];

  for (const name of fixtureNames) {
    it(`fixture: ${name}`, () => {
      const dir = resolve(fixturesDir, name);
      const html = readFileSync(resolve(dir, 'index.html'), 'utf-8');
      const expectedMd = readFileSync(resolve(dir, 'index.md'), 'utf-8');
      const config = JSON.parse(readFileSync(resolve(dir, 'index.json'), 'utf-8'));

      if (config.fragment !== true) {
        return;
      }

      expect(convert(html)).toBe(expectedMd);
    });
  }
});
