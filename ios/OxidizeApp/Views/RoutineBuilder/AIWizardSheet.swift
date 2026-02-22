import SwiftUI

struct AIWizardSheet: View {
    @Bindable var vm: RoutineBuilderViewModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ZStack {
                Theme.bgPrimary.ignoresSafeArea()

                VStack(spacing: 20) {
                    // Step indicator
                    HStack(spacing: 6) {
                        ForEach(1...5, id: \.self) { step in
                            Circle()
                                .fill(step <= vm.aiStep ? Theme.accentA : Theme.fgMuted.opacity(0.3))
                                .frame(width: 8, height: 8)
                                .shadow(color: step == vm.aiStep ? Theme.accentA.opacity(0.6) : .clear, radius: 3)
                        }
                    }
                    .padding(.top)

                    ScrollView {
                        VStack(spacing: 16) {
                            switch vm.aiStep {
                            case 1:
                                stepOne
                            case 2:
                                stepTwo
                            case 3:
                                stepThree
                            case 4:
                                stepFour
                            case 5:
                                stepFive
                            default:
                                EmptyView()
                            }
                        }
                        .padding()
                    }

                    // Navigation
                    HStack {
                        if vm.aiStep > 1 {
                            Button {
                                vm.aiStep -= 1
                            } label: {
                                Text("TILLBAKA")
                                    .font(.mono(size: 12, weight: .medium))
                                    .tracking(1)
                                    .foregroundStyle(Theme.fgSecondary)
                                    .padding(.horizontal, 16)
                                    .padding(.vertical, 10)
                                    .overlay(
                                        RoundedRectangle(cornerRadius: 4)
                                            .stroke(Theme.border, lineWidth: 1)
                                    )
                            }
                        }

                        Spacer()

                        if vm.aiStep < 5 {
                            Button {
                                vm.aiStep += 1
                            } label: {
                                Text("NÄSTA")
                                    .font(.mono(size: 14, weight: .bold))
                                    .tracking(2)
                                    .foregroundStyle(Theme.bgPrimary)
                                    .padding(.horizontal, 24)
                                    .padding(.vertical, 10)
                                    .background(Theme.accentA)
                                    .clipShape(RoundedRectangle(cornerRadius: 4))
                            }
                        } else {
                            Button {
                                Task { await vm.generateWithAI() }
                            } label: {
                                if vm.aiGenerating {
                                    ProgressView()
                                        .tint(Theme.bgPrimary)
                                } else {
                                    Text("GENERERA")
                                        .font(.mono(size: 14, weight: .bold))
                                        .tracking(2)
                                }
                            }
                            .foregroundStyle(Theme.bgPrimary)
                            .padding(.horizontal, 24)
                            .padding(.vertical, 10)
                            .background(Theme.accentA)
                            .clipShape(RoundedRectangle(cornerRadius: 4))
                            .disabled(vm.aiGenerating)
                            .opacity(vm.aiGenerating ? 0.5 : 1)
                        }
                    }
                    .padding()
                }
            }
            .navigationTitle("AI-RUTIN")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Avbryt") { dismiss() }
                        .foregroundStyle(Theme.accentA)
                }
            }
            .alert("Fel", isPresented: .init(
                get: { vm.aiError != nil },
                set: { if !$0 { vm.aiError = nil } }
            )) {
                Button("OK") { vm.aiError = nil }
            } message: {
                Text(vm.aiError ?? "")
            }
        }
    }

    private var stepOne: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("HUR MÅNGA PASS?")
                .font(.mono(size: 14, weight: .bold))
                .foregroundStyle(Theme.fgPrimary)
                .tracking(1)

            Picker("Pass", selection: $vm.aiPassCount) {
                ForEach(1...5, id: \.self) { n in
                    Text("\(n) pass").tag(n)
                }
            }
            .pickerStyle(.segmented)
        }
    }

    private var stepTwo: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("MÅL")
                .font(.mono(size: 14, weight: .bold))
                .foregroundStyle(Theme.fgPrimary)
                .tracking(1)

            Picker("Mål", selection: $vm.aiFocus) {
                Text("Styrka").tag("Styrka")
                Text("Volym").tag("Volym")
                Text("Funktionell").tag("Funktionell")
            }
            .pickerStyle(.segmented)

            TextField("Beskriv (valfritt)", text: $vm.aiDescription, axis: .vertical)
                .lineLimit(3)
                .darkInputStyle()
        }
    }

    private var stepThree: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("MÅLOMRÅDEN & STIL")
                .font(.mono(size: 14, weight: .bold))
                .foregroundStyle(Theme.fgPrimary)
                .tracking(1)

            TextField("Målområden (t.ex. överkropp, ben)", text: $vm.aiAreas)
                .darkInputStyle()

            Picker("Stil", selection: $vm.aiStyle) {
                Text("Tunga lyft").tag("Tunga lyft, få reps")
                Text("Moderat").tag("Moderata vikter, medel reps")
                Text("Cardio-mix").tag("Cardio/HIIT-blandat")
            }
            .pickerStyle(.segmented)
        }
    }

    private var stepFour: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("UTRUSTNING & TID")
                .font(.mono(size: 14, weight: .bold))
                .foregroundStyle(Theme.fgPrimary)
                .tracking(1)

            Picker("Utrustning", selection: $vm.aiEquipment) {
                Text("Fullt gym").tag("Fullt gym")
                Text("Hemmagym").tag("Hemmagym med hantlar och bänk")
                Text("Kroppsvikt").tag("Bara kroppsvikt")
            }
            .pickerStyle(.segmented)

            Picker("Passlängd", selection: $vm.aiDuration) {
                Text("Kort (30 min)").tag("Korta (30 min)")
                Text("Normal (45-60)").tag("Normala (45-60 min)")
                Text("Lång (75+)").tag("Långa (75+ min)")
            }
            .pickerStyle(.segmented)
        }
    }

    private var stepFive: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("EXTRA")
                .font(.mono(size: 14, weight: .bold))
                .foregroundStyle(Theme.fgPrimary)
                .tracking(1)

            Toggle("Supersets", isOn: $vm.aiSupersets)
                .font(.mono(size: 14))
                .tint(Theme.accentA)
            Toggle("Finishers", isOn: $vm.aiFinishers)
                .font(.mono(size: 14))
                .tint(Theme.accentA)

            VStack(alignment: .leading, spacing: 4) {
                Text("SAMMANFATTNING")
                    .font(.mono(size: 10, weight: .medium))
                    .foregroundStyle(Theme.fgMuted)
                    .tracking(2)
                Text("\(vm.aiPassCount) pass, \(vm.aiFocus), \(vm.aiStyle)")
                    .font(.mono(size: 12))
                    .foregroundStyle(Theme.fgSecondary)
                Text("\(vm.aiEquipment), \(vm.aiDuration)")
                    .font(.mono(size: 12))
                    .foregroundStyle(Theme.fgSecondary)
            }
            .padding()
            .background(Theme.bgCard)
            .overlay(
                RoundedRectangle(cornerRadius: 4)
                    .stroke(Theme.border, lineWidth: 1)
            )
            .clipShape(RoundedRectangle(cornerRadius: 4))
        }
    }
}
