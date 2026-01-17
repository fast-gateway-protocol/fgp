import Foundation

struct FgpRequest {
    let id: String
    let version: Int
    let method: String
    let params: [String: Any]

    init(method: String, params: [String: Any] = [:]) {
        self.id = UUID().uuidString
        self.version = 1
        self.method = method
        self.params = params
    }

    func ndjsonLine() throws -> String {
        let payload: [String: Any] = [
            "id": id,
            "v": version,
            "method": method,
            "params": params
        ]
        let data = try JSONSerialization.data(withJSONObject: payload, options: [])
        guard let json = String(data: data, encoding: .utf8) else {
            throw CocoaError(.fileWriteInapplicableStringEncoding)
        }
        return json + "\n"
    }
}

struct FgpResponse {
    let id: String
    let ok: Bool
    let result: [String: Any]?
    let error: [String: Any]?
    let meta: [String: Any]?

    init(json: [String: Any]) throws {
        guard let id = json["id"] as? String,
              let ok = json["ok"] as? Bool else {
            throw CocoaError(.propertyListReadCorrupt)
        }
        self.id = id
        self.ok = ok
        self.result = json["result"] as? [String: Any]
        self.error = json["error"] as? [String: Any]
        self.meta = json["meta"] as? [String: Any]
    }
}

struct FgpClient {
    let socketPath: String
    let timeout: TimeInterval

    init(socketPath: String, timeout: TimeInterval = 30) {
        self.socketPath = socketPath
        self.timeout = timeout
    }

    func call(method: String, params: [String: Any] = [:]) throws -> FgpResponse {
        let request = FgpRequest(method: method, params: params)
        let line = try request.ndjsonLine()
        let client = UnixSocketClient(socketPath: socketPath, timeout: timeout)
        let responseLine = try client.sendLine(line)
        guard let data = responseLine.data(using: .utf8) else {
            throw CocoaError(.fileReadCorruptFile)
        }
        let json = try JSONSerialization.jsonObject(with: data, options: [])
        guard let object = json as? [String: Any] else {
            throw CocoaError(.propertyListReadCorrupt)
        }
        return try FgpResponse(json: object)
    }

    func health() throws -> FgpResponse {
        try call(method: "health")
    }

    func stop() throws -> FgpResponse {
        try call(method: "stop")
    }
}
