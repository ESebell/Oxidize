import SwiftUI

struct ExerciseRowView: View {
    @Binding var exercise: Exercise
    var isFinisher: Bool = false
    var onRemove: (() -> Void)?
    var onSuperset: (() -> Void)?
    var onUnlinkSuperset: (() -> Void)?

    @State private var setsText: String = ""
    @State private var repsText: String = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            // Row 1: Name + remove button
            HStack {
                Text(exercise.name)
                    .font(.mono(size: 14, weight: .bold))
                    .foregroundStyle(Theme.fgPrimary)
                    .lineLimit(1)

                Spacer()

                if let onRemove {
                    Button { onRemove() } label: {
                        Image(systemName: "xmark")
                            .font(.system(size: 11, weight: .medium))
                            .foregroundStyle(Theme.fgMuted)
                    }
                }
            }

            // Row 2: Sets × Reps editing + superset/timer controls
            HStack(spacing: 6) {
                // Sets input
                TextField("3", text: $setsText)
                    .font(.mono(size: 14, weight: .bold))
                    .foregroundStyle(Theme.fgPrimary)
                    .keyboardType(.numberPad)
                    .frame(width: 32)
                    .multilineTextAlignment(.center)
                    .padding(.vertical, 4)
                    .background(Theme.bgSecondary)
                    .overlay(Rectangle().stroke(Theme.border, lineWidth: 1))
                    .onChange(of: setsText) {
                        exercise.sets = Int(setsText) ?? exercise.sets
                    }

                Text("×")
                    .font(.mono(size: 14))
                    .foregroundStyle(Theme.fgMuted)

                // Reps input
                TextField("8-12", text: $repsText)
                    .font(.mono(size: 14, weight: .bold))
                    .foregroundStyle(Theme.fgPrimary)
                    .frame(width: 56)
                    .multilineTextAlignment(.center)
                    .padding(.vertical, 4)
                    .background(Theme.bgSecondary)
                    .overlay(Rectangle().stroke(Theme.border, lineWidth: 1))
                    .onChange(of: repsText) {
                        if isFinisher && exercise.durationSecs != nil {
                            let num = Int(repsText.filter(\.isNumber)) ?? 30
                            exercise.durationSecs = num
                            exercise.repsTarget = "\(num)s"
                        } else {
                            exercise.repsTarget = repsText
                        }
                    }

                // Timer toggle (finishers only)
                if isFinisher {
                    Button {
                        if exercise.durationSecs != nil {
                            exercise.durationSecs = nil
                            exercise.repsTarget = "10-15"
                            repsText = "10-15"
                        } else {
                            exercise.durationSecs = 30
                            exercise.repsTarget = "30s"
                            repsText = "30s"
                        }
                    } label: {
                        Text(exercise.durationSecs != nil ? "⏱" : "#")
                            .font(.mono(size: 14, weight: .bold))
                            .foregroundStyle(exercise.durationSecs != nil ? Theme.accentB : Theme.fgMuted)
                            .frame(width: 28, height: 28)
                            .background(exercise.durationSecs != nil ? Theme.accentB.opacity(0.15) : Color.clear)
                            .overlay(Rectangle().stroke(Theme.border, lineWidth: 1))
                    }
                }

                Spacer()

                // Superset controls (exercises only, not finishers)
                if !isFinisher {
                    if exercise.isSuperset {
                        // Show partner badge + unlink button
                        if let partner = exercise.supersetWith {
                            HStack(spacing: 4) {
                                Text("⟷ \(partner)")
                                    .font(.mono(size: 9, weight: .bold))
                                    .foregroundStyle(Theme.accentB)
                                    .lineLimit(1)

                                if let onUnlinkSuperset {
                                    Button { onUnlinkSuperset() } label: {
                                        Text("✂")
                                            .font(.system(size: 12))
                                            .foregroundStyle(Theme.fgMuted)
                                            .frame(width: 22, height: 22)
                                            .overlay(Rectangle().stroke(Theme.border, lineWidth: 1))
                                    }
                                }
                            }
                            .padding(.horizontal, 6)
                            .padding(.vertical, 3)
                            .background(Theme.accentB.opacity(0.08))
                        }
                    } else {
                        if let onSuperset {
                            Button { onSuperset() } label: {
                                Text("⟷")
                                    .font(.mono(size: 14, weight: .bold))
                                    .foregroundStyle(Theme.accentB)
                                    .frame(width: 28, height: 28)
                                    .overlay(Rectangle().stroke(Theme.accentB.opacity(0.3), lineWidth: 1))
                            }
                        }
                    }
                }
            }
        }
        .padding(.vertical, 6)
        .onAppear {
            setsText = "\(exercise.sets)"
            repsText = exercise.repsTarget
        }
    }
}
