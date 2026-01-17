import Foundation
import ServiceManagement

struct AutostartService {
    func isEnabled() -> Bool {
        SMAppService.mainApp.status == .enabled
    }

    func enable() throws {
        try SMAppService.mainApp.register()
    }

    func disable() throws {
        try SMAppService.mainApp.unregister()
    }
}
