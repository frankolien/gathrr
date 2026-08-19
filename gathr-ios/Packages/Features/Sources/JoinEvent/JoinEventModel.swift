import Foundation
import Models
import Networking
import Observation

public enum InviteCodeInput {
    public static let length = 10
    private static let alphabet = Set("0123456789ABCDEFGHJKMNPQRSTVWXYZ")

    public static func normalize(_ raw: String) -> String {
        String(
            raw
                .uppercased()
                .compactMap { character -> Character? in
                    switch character {
                    case "I", "L": "1"
                    case "O": "0"
                    case "-", " ", "_": nil
                    default: alphabet.contains(character) ? character : nil
                    }
                }
                .prefix(length)
        )
    }

    public static func isComplete(_ normalized: String) -> Bool {
        normalized.count == length
    }
}

