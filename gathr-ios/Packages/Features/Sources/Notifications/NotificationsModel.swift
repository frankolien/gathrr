import Foundation
import Models
import Networking
import Observation

public enum ActivityBucket: Sendable, Hashable, CaseIterable {
    case today
    case yesterday
    case thisWeek
    case earlier
}

public struct ActivitySection: Identifiable, Sendable, Hashable {
    public let bucket: ActivityBucket
    public let entries: [ActivityEntry]

    public var id: ActivityBucket { bucket }
}

