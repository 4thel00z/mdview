import AppKit

let arguments = CommandLine.arguments
guard arguments.count == 2 else {
    FileHandle.standardError.write("usage: make-icon.swift <out.iconset>\n".data(using: .utf8)!)
    exit(2)
}
let output = URL(fileURLWithPath: arguments[1])
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)

let paper = NSColor(srgbRed: 0.965, green: 0.965, blue: 0.945, alpha: 1)
let ink = NSColor(srgbRed: 0.082, green: 0.082, blue: 0.082, alpha: 1)
let gold = NSColor(srgbRed: 0.976, green: 0.663, blue: 0.0, alpha: 1)
let rule = NSColor(srgbRed: 0.867, green: 0.867, blue: 0.835, alpha: 1)

func draw(size: Int) -> Data? {
    let pixels = CGFloat(size)
    guard let rep = NSBitmapImageRep(
        bitmapDataPlanes: nil, pixelsWide: size, pixelsHigh: size, bitsPerSample: 8,
        samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB,
        bytesPerRow: 0, bitsPerPixel: 0)
    else { return nil }
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
    let unit = pixels / 1024
    let tile = NSRect(x: 100 * unit, y: 100 * unit, width: 824 * unit, height: 824 * unit)
    let shape = NSBezierPath(roundedRect: tile, xRadius: 185 * unit, yRadius: 185 * unit)
    NSGraphicsContext.saveGraphicsState()
    let shadow = NSShadow()
    shadow.shadowColor = NSColor.black.withAlphaComponent(0.28)
    shadow.shadowBlurRadius = 24 * unit
    shadow.shadowOffset = NSSize(width: 0, height: -10 * unit)
    shadow.set()
    paper.setFill()
    shape.fill()
    NSGraphicsContext.restoreGraphicsState()
    rule.setStroke()
    shape.lineWidth = max(1, 4 * unit)
    shape.stroke()

    let font = NSFont.systemFont(ofSize: 430 * unit, weight: .semibold)
    let style = NSMutableParagraphStyle()
    style.alignment = .center
    let glyph = NSAttributedString(string: "M", attributes: [
        .font: font, .foregroundColor: ink, .kern: -12 * unit, .paragraphStyle: style,
    ])
    let bounds = glyph.boundingRect(with: tile.size, options: [.usesLineFragmentOrigin])
    glyph.draw(in: NSRect(x: 100 * unit, y: 512 * unit - bounds.height / 2 + 40 * unit,
                          width: 824 * unit, height: bounds.height))

    let arrow = NSBezierPath()
    let cx = 512 * unit
    arrow.move(to: NSPoint(x: cx, y: 330 * unit))
    arrow.line(to: NSPoint(x: cx, y: 230 * unit))
    arrow.move(to: NSPoint(x: cx - 46 * unit, y: 276 * unit))
    arrow.line(to: NSPoint(x: cx, y: 228 * unit))
    arrow.line(to: NSPoint(x: cx + 46 * unit, y: 276 * unit))
    arrow.lineWidth = 30 * unit
    arrow.lineCapStyle = .round
    arrow.lineJoinStyle = .round
    gold.setStroke()
    arrow.stroke()
    NSGraphicsContext.restoreGraphicsState()
    return rep.representation(using: .png, properties: [:])
}

for base in [16, 32, 128, 256, 512] {
    for scale in [1, 2] {
        let suffix = scale == 1 ? "" : "@2x"
        guard let data = draw(size: base * scale) else { exit(1) }
        try data.write(to: output.appendingPathComponent("icon_\(base)x\(base)\(suffix).png"))
    }
}
