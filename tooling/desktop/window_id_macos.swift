// Resolve the exact native test window for window-buffer capture.
import CoreGraphics
import Foundation

let arguments = CommandLine.arguments
guard arguments.count == 6,
      let pid = Int(arguments[1]),
      let x = Double(arguments[2]),
      let y = Double(arguments[3]),
      let width = Double(arguments[4]),
      let height = Double(arguments[5]),
      let windows = CGWindowListCopyWindowInfo(
          [.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID
      ) as? [[String: Any]] else {
    fatalError("Invalid native capture window request")
}

let matches = windows.filter { window in
    guard window[kCGWindowOwnerPID as String] as? Int == pid,
          window[kCGWindowLayer as String] as? Int == 0,
          let bounds = window[kCGWindowBounds as String] as? [String: Any],
          let rectangle = CGRect(dictionaryRepresentation: bounds as CFDictionary) else {
        return false
    }

    return abs(rectangle.minX - x) < 1 && abs(rectangle.minY - y) < 1
        && abs(rectangle.width - width) < 1 && abs(rectangle.height - height) < 1
}

guard matches.count == 1, let id = matches[0][kCGWindowNumber as String] as? Int else {
    fatalError("Expected exactly one native test window; found \(matches.count)")
}

print(id)
