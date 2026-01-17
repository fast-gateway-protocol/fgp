import SwiftUI

struct TierBadge: View {
    let tier: UInt8?

    var body: some View {
        if let tier, let qualityTier = QualityTier(rawValue: tier) {
            HStack(spacing: 4) {
                Image(systemName: qualityTier.icon)
                    .font(.caption2)
                Text(qualityTier.label)
                    .font(.caption2)
            }
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .background(Capsule().fill(tierColor(qualityTier).opacity(0.2)))
            .foregroundColor(tierColor(qualityTier))
        }
    }

    private func tierColor(_ tier: QualityTier) -> Color {
        switch tier {
        case .verified:
            return .green
        case .trusted:
            return .blue
        case .community:
            return .secondary
        case .unverified:
            return .orange
        }
    }
}

struct SpeedupBadge: View {
    let speedup: Float?

    var body: some View {
        if let speedup, speedup > 1 {
            HStack(spacing: 2) {
                Image(systemName: "bolt.fill")
                    .font(.caption2)
                Text("\(Int(speedup))x")
                    .font(.caption2)
            }
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .background(Capsule().fill(Color.yellow.opacity(0.2)))
            .foregroundColor(.orange)
        }
    }
}
