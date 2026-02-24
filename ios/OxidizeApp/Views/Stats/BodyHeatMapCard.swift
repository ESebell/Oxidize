import SwiftUI

struct BodyHeatMapCard: View {
    let vm: StatsViewModel
    @State private var showBack = false

    private let outlineColor = Color(hex: "#1e90ff")

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("VECKOVOLYM, MUSKELKARTA")
                .font(.mono(size: 12, weight: .bold))
                .foregroundStyle(Theme.fgPrimary)
                .tracking(1)

            // Front / Back toggle
            HStack(spacing: 0) {
                toggleBtn("FRAMSIDA", active: !showBack) {
                    withAnimation(.easeInOut(duration: 0.2)) { showBack = false }
                }
                toggleBtn("BAKSIDA", active: showBack) {
                    withAnimation(.easeInOut(duration: 0.2)) { showBack = true }
                }
            }
            .clipShape(RoundedRectangle(cornerRadius: 4))
            .overlay(RoundedRectangle(cornerRadius: 4).stroke(Theme.border, lineWidth: 1))

            // Body figure
            Canvas { context, size in
                let regions = showBack ? MusclePaths.parsedBack : MusclePaths.parsedFront
                let scale = size.width / MusclePaths.viewWidth
                let offsetX: CGFloat = showBack ? -MusclePaths.backOffsetX : 0
                let transform = CGAffineTransform(translationX: offsetX, y: 0)
                    .concatenating(.init(scaleX: scale, y: scale))

                let detail = vm.summary.weeklyMuscleDetail

                for region in regions {
                    let sets = region.muscles.isEmpty ? 0 :
                        region.muscles.map { detail[$0, default: 0] }.max() ?? 0

                    for path in region.paths {
                        let scaled = path.applying(transform)

                        if !region.muscles.isEmpty && sets > 0 {
                            let color = heatColor(for: sets)
                            let glowR = glowRadius(for: sets) * scale

                            // Glow
                            context.drawLayer { layer in
                                layer.addFilter(.blur(radius: glowR))
                                layer.fill(scaled, with: .color(color.opacity(0.35)))
                            }
                            // Fill
                            context.fill(scaled, with: .color(color.opacity(0.7)))
                            // Edge
                            context.stroke(scaled, with: .color(color.opacity(0.9)),
                                           style: StrokeStyle(lineWidth: 0.5 * scale))
                        } else {
                            // Dim fill + outline for untrained muscles
                            context.fill(scaled, with: .color(outlineColor.opacity(0.045)))
                            context.stroke(scaled, with: .color(outlineColor.opacity(0.36)),
                                           style: StrokeStyle(lineWidth: 0.65 * scale))
                        }
                    }
                }
            }
            .aspectRatio(MusclePaths.viewWidth / MusclePaths.viewHeight, contentMode: .fit)
            .frame(maxHeight: 420)
            .frame(maxWidth: .infinity)

            // Legend (matches WeeklyVolumeCard zones)
            HStack(spacing: 6) {
                legendItem(sets: 0, label: "0")
                legendItem(sets: 5, label: "1-9")
                legendItem(sets: 15, label: "10-20")
                legendItem(sets: 25, label: "20+")
            }
            .frame(maxWidth: .infinity)
        }
        .padding()
        .background(Theme.bgCard)
        .overlay(RoundedRectangle(cornerRadius: 4).stroke(Theme.border, lineWidth: 1))
        .clipShape(RoundedRectangle(cornerRadius: 4))
    }

    // MARK: - Heat colors

    private func heatColor(for sets: Int) -> Color {
        switch sets {
        case 0: return outlineColor.opacity(0.2)
        case 1...9: return Theme.volLow        // Cyan
        case 10...20: return Theme.volOptimal   // Green
        default: return Theme.volHigh           // Red
        }
    }

    private func glowRadius(for sets: Int) -> CGFloat {
        switch sets {
        case 0: return 0
        case 1...9: return 5
        case 10...20: return 9
        default: return 12
        }
    }

    // MARK: - UI helpers

    @ViewBuilder
    private func toggleBtn(_ label: String, active: Bool, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Text(label)
                .font(.mono(size: 10, weight: .medium))
                .tracking(1)
                .padding(.horizontal, 16)
                .padding(.vertical, 8)
                .frame(maxWidth: .infinity)
        }
        .foregroundStyle(active ? Theme.bgPrimary : Theme.fgMuted)
        .background(active ? Theme.accentA : Color.clear)
    }

    @ViewBuilder
    private func legendItem(sets: Int, label: String) -> some View {
        HStack(spacing: 3) {
            Circle()
                .fill(sets == 0 ? outlineColor.opacity(0.3) : heatColor(for: sets))
                .frame(width: 6, height: 6)
            Text(label)
                .font(.mono(size: 9))
                .foregroundStyle(Theme.fgMuted)
        }
    }
}
