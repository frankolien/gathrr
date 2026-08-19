struct OnboardingCopy: Identifiable, Sendable, Equatable {
    let id: Int
    let headline: String
    let subhead: String

    static let all: [OnboardingCopy] = [
        OnboardingCopy(
            id: 0,
            headline: "Create beautiful invitations",
            subhead: "Design an invite in minutes with thoughtful templates for any occasion."
        ),
        OnboardingCopy(
            id: 1,
            headline: "Invite anyone, anywhere",
            subhead: "Share via contacts, a link, or a QR code. RSVPs update in real time."
        ),
        OnboardingCopy(
            id: 2,
            headline: "Every event, one place",
            subhead: "Track guests, sync your calendar, and get gentle reminders as the day nears."
        ),
    ]
}
