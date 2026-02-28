// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "Html2MarkdownTests",
    targets: [
        .systemLibrary(
            name: "html2markdown_uniffiFFI",
            path: "Sources/html2markdown_uniffiFFI"
        ),
        .target(
            name: "Html2Markdown",
            dependencies: ["html2markdown_uniffiFFI"],
            path: "Sources/Html2Markdown"
        ),
        .testTarget(
            name: "Html2MarkdownTests",
            dependencies: ["Html2Markdown"],
            path: "Tests/Html2MarkdownTests"
        ),
    ]
)
