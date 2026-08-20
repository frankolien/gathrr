import Foundation

extension CoverTemplateCatalog {
    static let suggestedSet: [CoverTemplate] = [
        .entry("sky-signature", .suggested, 0x00_89_FF, 0x4E_BC_FF, .waves(4)),
        .entry("sky-orbit", .suggested, 0x0F_3A_8A, 0x4E_BC_FF, .orbs(4)),
        .entry("sky-grid", .suggested, 0x14_3A_6B, 0x2E_8C_FF, .grid(12)),
        .entry("ink-signature", .suggested, 0x16_16_1A, 0x36_36_3E, .grid(14)),
        .entry("dusk-rays", .suggested, 0x3A_1B_5C, 0x8A_5A_E8, .rays(20)),
        .entry("mint-arcs", .suggested, 0x0E_7A_6B, 0x4E_E8_C6, .arcs(6)),
    ]

    static let summerSet: [CoverTemplate] = [
        .entry("summer-beach", .summer, 0x1F_8A_C6, 0xFF_F0_C2, .waves(5), "Beach & Friends", .script),
        .entry("summer-sun", .summer, 0xFF_9F_2E, 0xFF_D9_5C, .rays(20), "Hello Summer", .stamp),
        .entry("summer-pool", .summer, 0x1C_C4_D6, 0xE8_FB_FF, .dots(6), "Pool Party"),
        .entry("summer-palms", .summer, 0x0E_9E_86, 0xF5_E6_C8, .stripes(9), "Salt & Sand", .stamp),
        .entry("summer-sunset", .summer, 0xE9_6B_3F, 0xFF_C1_7A, .waves(4), "Golden Hour", .script),
        .entry("summer-picnic", .summer, 0x7A_B8_4F, 0xF2_EC_D9, .checks(8), "Picnic", .stamp),
        .entry("summer-lemonade", .summer, 0x2E_9B_E0, 0xFF_D6_4A, .orbs(4), "Cold Drinks"),
        .entry("summer-roadtrip", .summer, 0xD9_4A_3A, 0xFF_C6_8A, .stripes(12), "Road Trip", .stamp),
    ]

    static let invitesSet: [CoverTemplate] = [
        .entry("invite-guestlist", .invites, 0xD1_2A_2A, 0xF0_5A_5A, .grid(14), "You Are On The Guest List"),
        .entry("invite-guestlist-ink", .invites, 0x14_14_16, 0x2E_2E_33, .grid(14), "You Are On The Guest List"),
        .entry("invite-orange", .invites, 0xE8_6A_2C, 0xF2_B0_5E, .stripes(14), "You Are Invited", .stamp),
        .entry(
            "invite-cream", .invites, 0xE8_D9_A8, 0xF7_F0_DC, .stripes(14), "You Are Invited", .stamp,
            ink: 0x2A_23_16
        ),
        .entry("invite-plum", .invites, 0x8A_6A_C6, 0xE8_9E_46, .checks(9), "You Are Invited", .stamp),
        .entry("invite-plaid", .invites, 0x1F_5C_3D, 0xE9_E2_C8, .grid(9), "You Are Invited", .stamp),
        .entry("invite-blue", .invites, 0x2A_6A_C6, 0xEC_E4_CE, .stripes(14), "You Are Invited", .stamp),
        .entry("invite-press", .invites, 0xE8_5A_9E, 0xFF_B8_D6, .arcs(5), "Press To Join"),
        .entry("invite-save-date", .invites, 0x1B_3A_6B, 0xC6_A8_58, .arcs(7), "Save The Date", .script),
    ]

    static let partySet: [CoverTemplate] = [
        .entry("party-balloons", .party, 0x59_B5_F0, 0xE8_F7_FF, .orbs(5), "Let's Have A Party"),
        .entry("party-confetti", .party, 0x1B_1B_2E, 0xFF_5E_8A, .confetti(34), "Party"),
        .entry("party-neon", .party, 0x5B_2E_C6, 0x2E_E8_C6, .rays(24), "Big Night"),
        .entry("party-afterhours", .party, 0x2A_1B_45, 0xE8_B8_4A, .dots(7), "After Hours", .stamp),
        .entry("party-birthday", .party, 0xE8_4A_7A, 0xFF_D9_5C, .confetti(28), "Happy Birthday", .script),
        .entry("party-house", .party, 0x0F_3A_6B, 0x4E_BC_FF, .waves(4), "House Party"),
        .entry("party-dinner", .party, 0x6B_1B_2A, 0xE8_B8_8A, .arcs(6), "Dinner & Drinks", .script),
    ]
}
