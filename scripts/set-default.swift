import AppKit
import UniformTypeIdentifiers

let arguments = CommandLine.arguments
guard arguments.count == 2 else {
    FileHandle.standardError.write("usage: set-default.swift <path/to/mdview.app>\n".data(using: .utf8)!)
    exit(2)
}
let app = URL(fileURLWithPath: arguments[1])
let extensions = ["md", "markdown"]
let types = Set(extensions.flatMap { UTType.types(tag: $0, tagClass: .filenameExtension, conformingTo: nil) })
var failures = 0

for type in types.sorted(by: { $0.identifier < $1.identifier }) {
    let done = DispatchSemaphore(value: 0)
    NSWorkspace.shared.setDefaultApplication(at: app, toOpen: type) { error in
        if let error {
            print("fail  \(type.identifier): \(error.localizedDescription)")
            failures += 1
        }
        done.signal()
    }
    done.wait()
}

for ext in extensions {
    guard let type = UTType(filenameExtension: ext) else { continue }
    let handler = NSWorkspace.shared.urlForApplication(toOpen: type)?.path ?? "none"
    let ok = handler == app.path
    if !ok { failures += 1 }
    print("\(ok ? "ok  " : "FAIL") .\(ext)  \(type.identifier)  ->  \(handler)")
}
exit(failures == 0 ? 0 : 1)
