import Foundation

public enum CoverMotif: Hashable, Sendable {
    case wash
    case stripes(Int)
    case checks(Int)
    case dots(Int)
    case grid(Int)
    case rays(Int)
    case waves(Int)
    case arcs(Int)
    case confetti(Int)
    case orbs(Int)
}

struct CoverRandom {
    private var state: UInt64

    init(seed: String) {
        var hash: UInt64 = 0xCBF2_9CE4_8422_2325
        for byte in seed.utf8 {
            hash ^= UInt64(byte)
            hash &*= 0x0000_0100_0000_01B3
        }
        state = hash | 1
    }

    mutating func unit() -> Double {
        state ^= state << 13
        state ^= state >> 7
        state ^= state << 17
        return Double(state % 100_000) / 100_000
    }

    mutating func between(_ lower: Double, _ upper: Double) -> Double {
        lower + unit() * (upper - lower)
    }
}
