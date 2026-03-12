import SwiftUI

struct WorkoutOverviewSheet: View {
    @Bindable var vm: WorkoutViewModel
    @Environment(\.dismiss) private var dismiss
    @State private var expandedIdx: Int? = nil
    @State private var editingSet: EditingSet? = nil

    struct EditingSet: Equatable {
        let exerciseIdx: Int
        let setIdx: Int
        var weight: Double
        var reps: Int
    }

    var body: some View {
        NavigationStack {
            ZStack {
                Theme.bgPrimary.ignoresSafeArea()

                ScrollView {
                    VStack(spacing: 0) {
                        ForEach(Array(vm.exercises.enumerated()), id: \.offset) { index, ex in
                            let isDone = ex.setsCompleted.count >= ex.exercise.sets
                            let isCurrent = index == vm.currentIdx
                            let isStarted = !ex.setsCompleted.isEmpty
                            let isExpanded = expandedIdx == index

                            VStack(spacing: 0) {
                                // Exercise header
                                HStack(spacing: 10) {
                                    // Status icon — tap to jump
                                    Button {
                                        vm.jumpToExercise(idx: index)
                                        dismiss()
                                    } label: {
                                        if isDone {
                                            Image(systemName: "checkmark.circle.fill")
                                                .foregroundStyle(Theme.accentA)
                                                .font(.system(size: 14))
                                        } else if isCurrent {
                                            Image(systemName: "play.circle.fill")
                                                .foregroundStyle(Theme.accentA)
                                                .font(.system(size: 14))
                                        } else if isStarted {
                                            Image(systemName: "circle.lefthalf.filled")
                                                .foregroundStyle(Theme.accentA.opacity(0.5))
                                                .font(.system(size: 14))
                                        } else {
                                            Image(systemName: "circle")
                                                .foregroundStyle(Theme.fgMuted)
                                                .font(.system(size: 14))
                                        }
                                    }

                                    // Name + count — tap to expand
                                    Button {
                                        if isStarted {
                                            withAnimation(.easeInOut(duration: 0.15)) {
                                                expandedIdx = isExpanded ? nil : index
                                            }
                                            if isExpanded { editingSet = nil }
                                        } else {
                                            vm.jumpToExercise(idx: index)
                                            dismiss()
                                        }
                                    } label: {
                                        HStack {
                                            Text(ex.exercise.name.uppercased())
                                                .font(.mono(size: 13, weight: isCurrent ? .bold : .medium))
                                                .foregroundStyle(isCurrent ? Theme.fgPrimary : Theme.fgSecondary)
                                                .tracking(1)

                                            Spacer()

                                            if isStarted {
                                                Image(systemName: "pencil")
                                                    .font(.system(size: 11))
                                                    .foregroundStyle(Theme.fgMuted)
                                            }

                                            Text("\(ex.setsCompleted.count)/\(ex.exercise.sets)")
                                                .font(.mono(size: 12))
                                                .foregroundStyle(Theme.fgMuted)
                                        }
                                    }
                                }
                                .padding(.horizontal, 16)
                                .padding(.vertical, 12)

                                // Expandable logged sets
                                if isExpanded && !ex.setsCompleted.isEmpty {
                                    VStack(spacing: 0) {
                                        ForEach(Array(ex.setsCompleted.enumerated()), id: \.offset) { setIdx, set in
                                            let isEditing = editingSet?.exerciseIdx == index && editingSet?.setIdx == setIdx

                                            if isEditing, let editing = editingSet {
                                                SetEditRow(
                                                    setNum: setIdx + 1,
                                                    weight: editing.weight,
                                                    reps: editing.reps,
                                                    isBodyweight: ex.exercise.isBodyweight,
                                                    onWeightChange: { delta in
                                                        editingSet?.weight = max(0, (editingSet?.weight ?? 0) + delta)
                                                    },
                                                    onRepsChange: { delta in
                                                        editingSet?.reps = max(1, (editingSet?.reps ?? 1) + delta)
                                                    },
                                                    onSave: {
                                                        if let e = editingSet {
                                                            vm.updateSet(exerciseIdx: e.exerciseIdx, setIdx: e.setIdx, weight: e.weight, reps: e.reps)
                                                        }
                                                        editingSet = nil
                                                    },
                                                    onDelete: {
                                                        vm.deleteSet(exerciseIdx: index, setIdx: setIdx)
                                                        editingSet = nil
                                                    }
                                                )
                                            } else {
                                                Button {
                                                    editingSet = EditingSet(
                                                        exerciseIdx: index,
                                                        setIdx: setIdx,
                                                        weight: set.weight,
                                                        reps: set.reps
                                                    )
                                                } label: {
                                                    SetRow(
                                                        setNum: setIdx + 1,
                                                        weight: set.weight,
                                                        reps: set.reps,
                                                        isBodyweight: ex.exercise.isBodyweight
                                                    )
                                                }
                                            }
                                        }
                                        // Add set button
                                        Button {
                                            vm.addSet(exerciseIdx: index)
                                        } label: {
                                            HStack(spacing: 4) {
                                                Image(systemName: "plus")
                                                    .font(.system(size: 10))
                                                Text("SET")
                                                    .font(.mono(size: 11, weight: .medium))
                                                    .tracking(1)
                                            }
                                            .foregroundStyle(Theme.fgMuted)
                                            .padding(.vertical, 6)
                                            .padding(.horizontal, 12)
                                        }
                                        .padding(.leading, 40)
                                    }
                                    .padding(.bottom, 8)
                                }

                                Rectangle()
                                    .fill(Theme.border)
                                    .frame(height: 1)
                            }
                        }
                    }
                }
            }
            .navigationTitle("ÖVERSIKT")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Stäng") { dismiss() }
                        .font(.mono(size: 14, weight: .medium))
                        .foregroundStyle(Theme.accentA)
                }
            }
        }
    }
}

// MARK: - Set display row

private struct SetRow: View {
    let setNum: Int
    let weight: Double
    let reps: Int
    let isBodyweight: Bool

    var body: some View {
        HStack {
            Text("\(setNum)")
                .font(.mono(size: 12))
                .foregroundStyle(Theme.fgMuted)
                .frame(width: 24, alignment: .center)

            if isBodyweight {
                Text("\(reps) reps")
                    .font(.mono(size: 13, weight: .medium))
                    .foregroundStyle(Theme.fgSecondary)
            } else {
                Text("\(formatWeight(weight)) kg")
                    .font(.mono(size: 13, weight: .medium))
                    .foregroundStyle(Theme.fgSecondary)

                Text("×")
                    .font(.mono(size: 12))
                    .foregroundStyle(Theme.fgMuted)

                Text("\(reps)")
                    .font(.mono(size: 13, weight: .medium))
                    .foregroundStyle(Theme.fgSecondary)
            }

            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.leading, 24)
        .padding(.vertical, 6)
    }
}

// MARK: - Set edit row

private struct SetEditRow: View {
    let setNum: Int
    let weight: Double
    let reps: Int
    let isBodyweight: Bool
    let onWeightChange: (Double) -> Void
    let onRepsChange: (Int) -> Void
    let onSave: () -> Void
    let onDelete: () -> Void

    var body: some View {
        VStack(spacing: 10) {
            HStack(spacing: 0) {
                Text("SET \(setNum)")
                    .font(.mono(size: 11, weight: .bold))
                    .foregroundStyle(Theme.accentA)
                    .tracking(1)

                Spacer()

                Button(action: onDelete) {
                    Image(systemName: "trash")
                        .font(.system(size: 12))
                        .foregroundStyle(Theme.danger)
                }
            }

            if !isBodyweight {
                HStack {
                    Text("VIKT")
                        .font(.mono(size: 10, weight: .medium))
                        .foregroundStyle(Theme.fgMuted)
                        .tracking(1)
                        .frame(width: 36, alignment: .leading)

                    Button { onWeightChange(-2.5) } label: {
                        Text("-2.5")
                            .font(.mono(size: 12, weight: .bold))
                            .foregroundStyle(Theme.fgSecondary)
                            .frame(width: 44, height: 32)
                            .overlay(Rectangle().stroke(Theme.border, lineWidth: 1))
                    }

                    Text("\(formatWeight(weight)) kg")
                        .font(.mono(size: 14, weight: .bold))
                        .foregroundStyle(Theme.fgPrimary)
                        .frame(minWidth: 70)

                    Button { onWeightChange(2.5) } label: {
                        Text("+2.5")
                            .font(.mono(size: 12, weight: .bold))
                            .foregroundStyle(Theme.fgSecondary)
                            .frame(width: 44, height: 32)
                            .overlay(Rectangle().stroke(Theme.border, lineWidth: 1))
                    }
                }
            }

            HStack {
                Text("REPS")
                    .font(.mono(size: 10, weight: .medium))
                    .foregroundStyle(Theme.fgMuted)
                    .tracking(1)
                    .frame(width: 36, alignment: .leading)

                Button { onRepsChange(-1) } label: {
                    Text("-1")
                        .font(.mono(size: 12, weight: .bold))
                        .foregroundStyle(Theme.fgSecondary)
                        .frame(width: 44, height: 32)
                        .overlay(Rectangle().stroke(Theme.border, lineWidth: 1))
                }

                Text("\(reps)")
                    .font(.mono(size: 14, weight: .bold))
                    .foregroundStyle(Theme.fgPrimary)
                    .frame(minWidth: 70)

                Button { onRepsChange(1) } label: {
                    Text("+1")
                        .font(.mono(size: 12, weight: .bold))
                        .foregroundStyle(Theme.fgSecondary)
                        .frame(width: 44, height: 32)
                        .overlay(Rectangle().stroke(Theme.border, lineWidth: 1))
                }
            }

            Button(action: onSave) {
                Text("OK")
                    .font(.mono(size: 12, weight: .bold))
                    .tracking(1)
                    .foregroundStyle(Theme.bgPrimary)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 8)
                    .background(Theme.accentA)
                    .clipShape(RoundedRectangle(cornerRadius: 4))
            }
        }
        .padding(12)
        .padding(.leading, 24)
        .padding(.horizontal, 16)
        .background(Theme.bgCard)
        .overlay(
            Rectangle()
                .stroke(Theme.accentA.opacity(0.3), lineWidth: 1)
        )
    }
}
