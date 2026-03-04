import 'dart:convert';
import 'dart:io';

import 'package:test/test.dart';
import 'package:html2markdown_dart_test/html2markdown_uniffi.dart';

void main() {
  setUpAll(() {
    // Load the native library from the build output.
    final projectRoot = Directory.current.path.replaceAll(
      RegExp(r'/tests/bindings/dart$'),
      '',
    );
    final libDir = '$projectRoot/uniffi/target/release';
    final String libPath;
    if (Platform.isMacOS) {
      libPath = '$libDir/libhtml2markdown_uniffi.dylib';
    } else if (Platform.isLinux) {
      libPath = '$libDir/libhtml2markdown_uniffi.so';
    } else {
      throw UnsupportedError('Unsupported platform: ${Platform.operatingSystem}');
    }
    configureDefaultBindings(libraryPath: libPath);
  });

  group('convert', () {
    test('heading', () {
      expect(convert('<h1>Hello</h1>'), equals('# Hello\n'));
    });

    test('empty string', () {
      expect(convert(''), equals(''));
    });

    test('paragraph', () {
      expect(convert('<p>Hello</p>'), equals('Hello\n'));
    });

    test('emphasis', () {
      expect(convert('<em>Hello World.</em>'), equals('*Hello World.*\n'));
    });

    test('strong', () {
      expect(convert('<strong>Hello World.</strong>'), equals('**Hello World.**\n'));
    });

    test('link', () {
      final html = '<a href="http://example.com" title="example">example</a>';
      expect(convert(html), equals('[example](http://example.com "example")\n'));
    });

    test('image', () {
      final html = '<img src="http://example.com" alt="example">';
      expect(convert(html), equals('![example](http://example.com)\n'));
    });

    test('code', () {
      expect(convert('<code>toString()</code>'), equals('`toString()`\n'));
    });

    test('blockquote', () {
      final html = '<blockquote><p>This is a blockquote.</p></blockquote>';
      expect(convert(html), equals('> This is a blockquote.\n'));
    });

    test('unordered list', () {
      final html = '<ul><li>Alpha</li><li>Bravo</li><li>Charlie</li></ul>';
      expect(convert(html), equals('* Alpha\n* Bravo\n* Charlie\n'));
    });

    test('ordered list', () {
      final html = '<ol><li>Alpha</li><li>Bravo</li><li>Charlie</li></ol>';
      expect(convert(html), equals('1. Alpha\n2. Bravo\n3. Charlie\n'));
    });
  });

  group('convertWith', () {
    test('default options matches convert', () {
      final html = '<h1>Hello</h1>';
      expect(convertWith(html, defaultOptions()), equals(convert(html)));
    });
  });

  group('defaultOptions', () {
    test('stringify options', () {
      final opts = defaultStringifyOptions();
      expect(opts.headingStyle, equals(HeadingStyle.atx));
      expect(opts.bullet, equals('*'));
      expect(opts.bulletOrdered, equals('.'));
      expect(opts.emphasis, equals('*'));
      expect(opts.strong, equals('*'));
      expect(opts.fence, equals('`'));
      expect(opts.rule, equals('*'));
      expect(opts.ruleRepetition, equals(3));
      expect(opts.ruleSpaces, isFalse);
      expect(opts.closeAtx, isFalse);
      expect(opts.listItemIndent, equals(ListItemIndent.one));
      expect(opts.incrementListMarker, isTrue);
      expect(opts.quote, equals('"'));
      expect(opts.fences, isTrue);
      expect(opts.resourceLink, isFalse);
    });

    test('conversion options', () {
      final opts = defaultOptions();
      expect(opts.newlines, isFalse);
      expect(opts.checked, isNull);
      expect(opts.unchecked, isNull);
      expect(opts.quotes, equals(['"']));
    });
  });

  group('error handling', () {
    test('invalid bullet throws OptionsErrorException', () {
      final opts = defaultOptions().copyWith(
        stringify: defaultStringifyOptions().copyWith(bullet: 'x'),
      );
      expect(
        () => convertWith('<p>hi</p>', opts),
        throwsA(
          isA<OptionsErrorExceptionInvalidOption>()
            .having((e) => e.field, 'field', equals('bullet'))
            .having((e) => e.value, 'value', equals('x')),
        ),
      );
    });

    test('empty bullet throws OptionsErrorException', () {
      final opts = defaultOptions().copyWith(
        stringify: defaultStringifyOptions().copyWith(bullet: ''),
      );
      expect(
        () => convertWith('<p>hi</p>', opts),
        throwsA(isA<OptionsErrorExceptionInvalidOption>()),
      );
    });
  });

  group('fixtures', () {
    final projectRoot = Directory.current.path.replaceAll(
      RegExp(r'/tests/bindings/dart$'),
      '',
    );
    final fixturesDir = Directory('$projectRoot/test-fixtures');

    const fixtureNames = [
      'a', 'blockquote', 'br', 'code', 'em', 'heading',
      'img', 'ol', 'paragraph', 'strong', 'table', 'ul',
    ];

    for (final name in fixtureNames) {
      test('fixture: $name', () {
        final dir = Directory('${fixturesDir.path}/$name');
        expect(dir.existsSync(), isTrue, reason: 'Fixture dir not found: $dir');

        final html = File('${dir.path}/index.html').readAsStringSync();
        final expectedMd = File('${dir.path}/index.md').readAsStringSync();
        final configJson = jsonDecode(
          File('${dir.path}/index.json').readAsStringSync(),
        ) as Map<String, dynamic>;

        if (configJson['fragment'] != true) {
          // Skip non-fragment fixtures
          return;
        }

        expect(convert(html), equals(expectedMd),
            reason: "Fixture '$name' mismatch");
      });
    }
  });
}
