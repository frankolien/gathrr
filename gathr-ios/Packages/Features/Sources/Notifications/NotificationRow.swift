import DesignSystem
import Models
import SwiftUI

struct NotificationRow: View {
    let entry: ActivityEntry

    var body: some View {
        HStack(spacing: Spacing.stackGap) {
            badge
            VStack(alignment: .leading, spacing: 2) {
                Text(entry.kind.headline(actor: entry.actorDisplayName))
                    .font(Typography.body)
                    .foregroundStyle(Palette.textPrimary)
                    .lineLimit(2)
                Text(entry.eventTitle)
                    .font(Typography.footnote)
                    .foregroundStyle(Palette.textSecondary)
                    .lineLimit(1)
            }
            Spacer(minLength: Spacing.unit)
            VStack(alignment: .trailing, spacing: 6) {
                Text(entry.createdAt.formatted(.relative(presentation: .numeric)))
                    .font(Typography.chip)
                    .foregroundStyle(Palette.textTertiary)
                if !entry.read {
                    Circle().fill(Palette.accent).frame(width: 8, height: 8)
                }
            }
        }
        .padding(Spacing.gutter)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Palette.surface)
        .clipShape(RoundedRectangle(cornerRadius: Radius.card, style: .continuous))
        .accessibilityElement(children: .combine)
    }

    private var badge: some View {
        let look = entry.kind.look
        return Image(systemName: look.symbol)
            .font(.system(size: 15, weight: .semibold))
            .foregroundStyle(look.tint)
            .frame(width: 38, height: 38)
            .background(look.tint.opacity(0.14), in: Circle())
    }
}
