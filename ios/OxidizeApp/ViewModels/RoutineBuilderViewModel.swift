import Foundation

@Observable
final class RoutineBuilderViewModel {
    var routineId: String?
    var routineName = ""
    var routineFocus = ""
    var passes: [Pass] = []
    var isLoading = false
    var isSaving = false
    var selectedPassIdx = 0
    var searchQuery = ""
    var searchResults: [WgerExercise] = []
    var isSearching = false
    var showExerciseSearch = false
    var addingToFinishers = false
    var showSupersetPicker = false
    var supersetExerciseIdx: Int?
    var showDeleteConfirm = false

    // AI Wizard
    var showAIWizard = false
    var aiStep = 1
    var aiPassCount = 3
    var aiFocus = "Styrka"
    var aiDescription = ""
    var aiAreas = ""
    var aiStyle = "Tunga lyft, få reps"
    var aiEquipment = "Fullt gym"
    var aiDuration = "Normala (45-60 min)"
    var aiSupersets = true
    var aiFinishers = true
    var aiGenerating = false
    var aiError: String?

    func loadRoutine(id: String?) async {
        guard let id else {
            // New routine — start with one empty pass
            passes = [Pass(name: "Pass A", description: "", exercises: [], finishers: [])]
            return
        }

        routineId = id
        isLoading = true

        let routines = (try? await SupabaseService.shared.fetchRoutines()) ?? []
        if let existing = routines.first(where: { $0.id == id }) {
            routineName = existing.name
            routineFocus = existing.focus
            passes = existing.passes
        }

        isLoading = false
    }

    // MARK: - Pass management

    func addPass() {
        let names = ["Pass A", "Pass B", "Pass C", "Pass D", "Pass E"]
        let name = passes.count < names.count ? names[passes.count] : "Pass \(passes.count + 1)"
        passes.append(Pass(name: name, description: "", exercises: [], finishers: []))
        selectedPassIdx = passes.count - 1
    }

    func removePass(at index: Int) {
        guard passes.count > 1 else { return }
        passes.remove(at: index)
        if selectedPassIdx >= passes.count {
            selectedPassIdx = passes.count - 1
        }
    }

    // MARK: - Exercise management

    func addExercise(from wger: WgerExercise) {
        let exercise = Exercise.fromWger(
            name: wger.name,
            sets: 3,
            reps: "8-12",
            primaryMuscles: wger.primaryMuscles,
            secondaryMuscles: wger.secondaryMuscles,
            imageUrl: wger.imageUrl,
            equipment: wger.equipment,
            wgerId: wger.id
        )

        if addingToFinishers {
            var modified = exercise
            modified.isBodyweight = true
            passes[selectedPassIdx].finishers.append(modified)
        } else {
            passes[selectedPassIdx].exercises.append(exercise)
        }
        showExerciseSearch = false
    }

    func removeExercise(passIdx: Int, exerciseIdx: Int, isFinisher: Bool) {
        if isFinisher {
            passes[passIdx].finishers.remove(at: exerciseIdx)
        } else {
            let name = passes[passIdx].exercises[exerciseIdx].name
            // Unlink superset partner if exists
            for i in passes[passIdx].exercises.indices {
                if passes[passIdx].exercises[i].supersetWith == name {
                    passes[passIdx].exercises[i].isSuperset = false
                    passes[passIdx].exercises[i].supersetWith = nil
                    passes[passIdx].exercises[i].supersetName = nil
                }
            }
            passes[passIdx].exercises.remove(at: exerciseIdx)
        }
    }

    func linkSuperset(exerciseIdx: Int, partnerIdx: Int) {
        let name1 = passes[selectedPassIdx].exercises[exerciseIdx].name
        let name2 = passes[selectedPassIdx].exercises[partnerIdx].name

        passes[selectedPassIdx].exercises[exerciseIdx].isSuperset = true
        passes[selectedPassIdx].exercises[exerciseIdx].supersetWith = name2

        passes[selectedPassIdx].exercises[partnerIdx].isSuperset = true
        passes[selectedPassIdx].exercises[partnerIdx].supersetWith = name1

        showSupersetPicker = false
    }

    func unlinkSuperset(exerciseIdx: Int) {
        let name = passes[selectedPassIdx].exercises[exerciseIdx].name
        let partnerName = passes[selectedPassIdx].exercises[exerciseIdx].supersetWith

        passes[selectedPassIdx].exercises[exerciseIdx].isSuperset = false
        passes[selectedPassIdx].exercises[exerciseIdx].supersetWith = nil
        passes[selectedPassIdx].exercises[exerciseIdx].supersetName = nil

        if let partnerName {
            if let partnerIdx = passes[selectedPassIdx].exercises.firstIndex(where: { $0.name == partnerName }) {
                passes[selectedPassIdx].exercises[partnerIdx].isSuperset = false
                passes[selectedPassIdx].exercises[partnerIdx].supersetWith = nil
                passes[selectedPassIdx].exercises[partnerIdx].supersetName = nil
            }
        }
    }

    // MARK: - Search

    func searchExercises() async {
        guard !searchQuery.isEmpty else { searchResults = []; return }
        isSearching = true
        searchResults = (try? await WgerService.searchExercises(query: searchQuery)) ?? []
        isSearching = false
    }

    // MARK: - Save / Delete

    func saveRoutine() async {
        isSaving = true

        let id = routineId ?? "routine_\(currentTimestamp())"
        let routine = SavedRoutine(
            id: id,
            userId: SupabaseService.shared.currentUserId,
            name: routineName,
            focus: routineFocus,
            passes: passes,
            isActive: true,
            createdAt: routineId != nil ? currentTimestamp() : currentTimestamp()
        )

        do {
            // Deactivate others and save this as active
            try await SupabaseService.shared.setActiveRoutine(id)
            try await SupabaseService.shared.saveRoutine(routine)
            StorageService.shared.saveActiveRoutine(routine)
        } catch {
            print("Save routine failed: \(error)")
        }

        isSaving = false
    }

    func deleteRoutine() async {
        guard let id = routineId else { return }

        do {
            try await SupabaseService.shared.deleteRoutine(id)
            StorageService.shared.clearActiveRoutine()
        } catch {
            print("Delete routine failed: \(error)")
        }
    }

    // MARK: - AI Wizard

    func generateWithAI() async {
        aiGenerating = true
        aiError = nil

        let db = StorageService.shared.loadData()
        let bw = db.bodyweight

        do {
            let (name, focus, generatedPasses) = try await GeminiService.generateRoutine(
                passCount: aiPassCount,
                goal: aiFocus,
                description: aiDescription,
                targetAreas: aiAreas,
                style: aiStyle,
                equipment: aiEquipment,
                duration: aiDuration,
                supersets: aiSupersets,
                finishers: aiFinishers,
                bodyweight: bw
            )

            routineName = name
            routineFocus = focus
            passes = generatedPasses
            showAIWizard = false
        } catch {
            print("[AI Wizard] Error: \(error)")
            if let oxError = error as? OxidizeError {
                aiError = oxError.localizedDescription
            } else {
                aiError = "Fel: \(error.localizedDescription)"
            }
        }

        aiGenerating = false
    }
}
