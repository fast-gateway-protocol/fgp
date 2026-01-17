import Darwin
import Foundation

enum UnixSocketError: Error, LocalizedError {
    case socketCreationFailed(Int32)
    case connectFailed(String)
    case sendFailed
    case receiveFailed
    case invalidResponse

    var errorDescription: String? {
        switch self {
        case .socketCreationFailed(let code):
            return "Failed to create socket (errno: \(code))"
        case .connectFailed(let message):
            return "Failed to connect: \(message)"
        case .sendFailed:
            return "Failed to send request"
        case .receiveFailed:
            return "Failed to receive response"
        case .invalidResponse:
            return "Invalid response from daemon"
        }
    }
}

struct UnixSocketClient {
    let socketPath: String
    let timeout: TimeInterval

    init(socketPath: String, timeout: TimeInterval = 30) {
        self.socketPath = socketPath
        self.timeout = timeout
    }

    func sendLine(_ line: String) throws -> String {
        let fd = socket(AF_UNIX, SOCK_STREAM, 0)
        if fd == -1 {
            throw UnixSocketError.socketCreationFailed(errno)
        }

        var addr = sockaddr_un()
        addr.sun_family = sa_family_t(AF_UNIX)

        let maxLen = Int(MemoryLayout.size(ofValue: addr.sun_path))
        guard socketPath.utf8.count < maxLen else {
            close(fd)
            throw UnixSocketError.connectFailed("Socket path too long")
        }

        var pathBytes = Array(socketPath.utf8)
        pathBytes.append(0)

        withUnsafeMutablePointer(to: &addr.sun_path) { ptr in
            ptr.withMemoryRebound(to: UInt8.self, capacity: maxLen) { buffer in
                buffer.initialize(from: pathBytes, count: pathBytes.count)
            }
        }

        var addrCopy = addr
        let addrLen = socklen_t(MemoryLayout.size(ofValue: addrCopy))
        let connectResult = withUnsafePointer(to: &addrCopy) { ptr in
            ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) { sockAddr in
                connect(fd, sockAddr, addrLen)
            }
        }

        if connectResult != 0 {
            let message = String(cString: strerror(errno))
            close(fd)
            throw UnixSocketError.connectFailed(message)
        }

        var timeoutValue = timeval(tv_sec: Int(timeout), tv_usec: 0)
        setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &timeoutValue, socklen_t(MemoryLayout.size(ofValue: timeoutValue)))
        setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &timeoutValue, socklen_t(MemoryLayout.size(ofValue: timeoutValue)))

        guard let data = line.data(using: .utf8) else {
            close(fd)
            throw UnixSocketError.sendFailed
        }

        let writeResult = data.withUnsafeBytes { rawBuffer in
            write(fd, rawBuffer.baseAddress, data.count)
        }

        if writeResult <= 0 {
            close(fd)
            throw UnixSocketError.sendFailed
        }

        let handle = FileHandle(fileDescriptor: fd, closeOnDealloc: true)
        var buffer = Data()
        let newline = UInt8(ascii: "\n")

        while true {
            // Use modern throwing API instead of ObjC exception-based readData
            guard let chunk = try? handle.read(upToCount: 1024), !chunk.isEmpty else {
                break
            }
            buffer.append(chunk)
            if buffer.contains(newline) { break }
        }

        guard let newlineIndex = buffer.firstIndex(of: newline) else {
            throw UnixSocketError.receiveFailed
        }

        let lineData = buffer.prefix(upTo: newlineIndex)
        guard let response = String(data: lineData, encoding: .utf8) else {
            throw UnixSocketError.invalidResponse
        }

        return response
    }
}
