import Foundation

extension CoverTemplateCatalog {
    static let sportsSet: [CoverTemplate] = [
        .entry("sports-pitch", .sports, 0x1B_7A_3E, 0x2E_A0_55, .stripes(12), "Game Day"),
        .entry("sports-court", .sports, 0xD9_6A_2A, 0xF2_A8_5C, .arcs(6), "Run It Back"),
        .entry("sports-track", .sports, 0x14_14_18, 0xE8_4A_3A, .stripes(9), "Race Day", .stamp),
        .entry("sports-tennis", .sports, 0x1F_5C_9E, 0xD9_E8_4A, .grid(10), "Match Point", .stamp),
        .entry("sports-gym", .sports, 0x2A_2A_30, 0x4E_BC_FF, .waves(5), "Training"),
        .entry("sports-ride", .sports, 0x0E_6A_7A, 0xE8_E2_C8, .dots(6), "Sunday Ride", .script),
    ]

    static let techSet: [CoverTemplate] = [
        .entry("tech-launch", .tech, 0x14_16_2E, 0x4E_8C_FF, .grid(12), "Launch"),
        .entry("tech-hackathon", .tech, 0x0E_1A_16, 0x2E_E8_8A, .confetti(24), "Hackathon"),
        .entry("tech-demo", .tech, 0x2E_2A_6B, 0x8A_6A_FF, .rays(18), "Demo Night"),
        .entry("tech-meetup", .tech, 0x1A_1A_1E, 0x6C_6C_78, .dots(8), "Meetup", .stamp),
        .entry("tech-buildweek", .tech, 0x0A_2E_45, 0x2E_C6_E8, .arcs(7), "Build Week"),
        .entry("tech-office-hours", .tech, 0x24_2A_3A, 0x7A_8A_C6, .stripes(10), "Office Hours", .stamp),
    ]

    static let businessSet: [CoverTemplate] = [
        .entry(
            "business-teamwork", .business, 0xEC_E7_DA, 0xFF_FF_FF, .grid(12), "Team Work", .block,
            ink: 0x1B_1B_1F
        ),
        .entry("business-growth", .business, 0x1B_4A_2E, 0x3A_8A_55, .waves(4), "Growth"),
        .entry("business-standup", .business, 0x2E_3A_6B, 0x6A_7A_C6, .stripes(10), "Standup", .stamp),
        .entry("business-offsite", .business, 0xC6_8A_2E, 0xE8_C6_6A, .orbs(4), "Offsite", .script),
        .entry("business-review", .business, 0x1B_1B_22, 0x4E_4E_5A, .checks(10), "Quarter Review", .stamp),
        .entry("business-mixer", .business, 0x3A_2A_4A, 0xC6_9E_E8, .arcs(6), "Founder Mixer", .script),
    ]

    static let schoolSet: [CoverTemplate] = [
        .entry("school-orientation", .school, 0x2E_5C_C6, 0xE8_E2_C8, .checks(10), "Orientation", .stamp),
        .entry("school-study", .school, 0x8A_4A_C6, 0xE0_C6_FF, .grid(11), "Study Group"),
        .entry("school-graduation", .school, 0x14_2E_5C, 0xE8_C6_6A, .rays(20), "Class Of", .script),
        .entry("school-clubday", .school, 0xE8_7A_2E, 0xFF_C6_8A, .confetti(26), "Club Day"),
        .entry("school-formal", .school, 0x3A_1B_45, 0xC6_A8_E8, .arcs(6), "Formal", .script),
        .entry("school-fresher", .school, 0x0E_6A_5C, 0xC6_E8_58, .dots(7), "Freshers Week", .stamp),
    ]
}
