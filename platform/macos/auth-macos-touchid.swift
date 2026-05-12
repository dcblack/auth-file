import Foundation
import LocalAuthentication

let reason = CommandLine.arguments.dropFirst().joined(separator: " ").isEmpty
    ? "Authorize auth database change"
    : CommandLine.arguments.dropFirst().joined(separator: " ")

let context = LAContext()
context.localizedCancelTitle = "Cancel"

var error: NSError?
let policy = LAPolicy.deviceOwnerAuthentication

guard context.canEvaluatePolicy(policy, error: &error) else {
    if let error = error { fputs("Authorization unavailable: \(error.localizedDescription)\n", stderr) }
    exit(2)
}

let semaphore = DispatchSemaphore(value: 0)
var ok = false
var message = "authorization failed"

context.evaluatePolicy(policy, localizedReason: reason) { success, authError in
    ok = success
    if let authError = authError {
        message = authError.localizedDescription
    }
    semaphore.signal()
}

semaphore.wait()
if ok {
    exit(0)
} else {
    fputs("Authorization denied: \(message)\n", stderr)
    exit(1)
}
