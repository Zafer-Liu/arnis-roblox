import Cocoa
import ScreenCaptureKit

@main
struct CaptureApp {
    static func main() async {
        // Activate Roblox Studio
        let ws = NSWorkspace.shared
        for app in ws.runningApplications {
            if app.localizedName?.contains("Roblox") == true {
                app.activate()
                try? await Task.sleep(nanoseconds: 2_000_000_000)
                break
            }
        }
        
        do {
            let content = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true)
            guard let display = content.displays.first else {
                print("No display found")
                return
            }
            
            let filter = SCContentFilter(display: display, excludingWindows: [])
            let config = SCStreamConfiguration()
            config.width = display.width * 2
            config.height = display.height * 2
            config.showsCursor = false
            
            let image = try await SCScreenshotManager.captureImage(contentFilter: filter, configuration: config)
            let bitmap = NSBitmapImageRep(cgImage: image)
            if let data = bitmap.representation(using: .png, properties: [:]) {
                try data.write(to: URL(fileURLWithPath: "/tmp/studio_sck.png"))
                print("Captured \(image.width)x\(image.height)")
            }
        } catch {
            print("Error: \(error)")
        }
    }
}
