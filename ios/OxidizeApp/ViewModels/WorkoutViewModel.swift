import Foundation
import Combine

@Observable
final class WorkoutViewModel {
    // State
    var exercises: [ExerciseWorkoutState] = []
    var currentIdx: Int = 0
    var startTime: Int64 = 0
    var elapsed: Int64 = 0
    var lastSetTime: Int64 = 0
    var restElapsed: Int64 = 0
    var isResting = false
    var isFinished = false
    var showOverview = false
    var showCancelConfirm = false
    var isSaving = false
    var showSyncWarning = false
    var selectedRPE: Int? = nil

    // Timer (for timed exercises)
    var timerRunning = false
    var timerSelectedDuration: Int = 30
    var timerRemaining: Int = 0
    var showTimerFlash = false

    // Data
    var routineName: String = ""
    var bodyweight: Double = 80.0
    private var timer: Timer?

    var totalExercises: Int { exercises.count }

    var currentExercise: ExerciseWorkoutState? {
        guard currentIdx < exercises.count else { return nil }
        return exercises[currentIdx]
    }

    var currentSetNum: Int {
        (currentExercise?.setsCompleted.count ?? 0) + 1
    }

    var totalSets: Int {
        currentExercise?.exercise.sets ?? 0
    }

    var currentWeight: Double {
        currentExercise?.currentWeight ?? 0
    }

    // MARK: - Setup

    func setup(data: WorkoutData, resumedFrom: Int = 0, startElapsed: Int64 = 0) {
        routineName = data.routine.name
        exercises = data.exercises
        currentIdx = resumedFrom
        startTime = currentTimestamp() - startElapsed
        elapsed = startElapsed

        let db = StorageService.shared.loadData()
        bodyweight = db.bodyweight ?? 80.0

        startTimer()
    }

    func startTimer() {
        timer?.invalidate()
        timer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            guard let self else { return }
            let now = currentTimestamp()
            self.elapsed = now - self.startTime

            if self.isResting && self.lastSetTime > 0 {
                self.restElapsed = now - self.lastSetTime
            }

            if self.timerRunning {
                self.timerRemaining -= 1
                if self.timerRemaining <= 0 {
                    self.timerRemaining = 0
                    self.timerRunning = false
                    self.showTimerFlash = true
                    HapticService.timerDone()

                    DispatchQueue.main.asyncAfter(deadline: .now() + 0.8) { [weak self] in
                        self?.showTimerFlash = false
                        self?.completeTimedSet()
                    }
                }
            }
        }
    }

    func cleanup() {
        timer?.invalidate()
        timer = nil
    }

    // MARK: - Actions

    func completeSet(reps: Int) {
        let now = currentTimestamp()
        let rest: Int64? = lastSetTime > 0 ? now - lastSetTime : nil
        let idx = currentIdx

        let setsDone = exercises[idx].setsCompleted.count
        let setsTarget = exercises[idx].exercise.sets
        let isSuperset = exercises[idx].exercise.isSuperset

        exercises[idx].setsCompleted.append(SetRecord(
            weight: exercises[idx].currentWeight,
            reps: reps,
            timestamp: now,
            restBeforeSecs: rest
        ))

        HapticService.setCompleted()
        lastSetTime = now
        restElapsed = 0

        let justFinished = setsDone + 1 >= setsTarget

        if justFinished {
            if isSuperset, let partnerIdx = findPartnerIdx(for: idx) {
                let partner = exercises[partnerIdx]
                if partner.setsCompleted.count < partner.exercise.sets {
                    currentIdx = partnerIdx
                    isResting = true
                    return
                }
            }
            advanceToNext(from: idx)
        } else if isSuperset, let partnerIdx = findPartnerIdx(for: idx) {
            let partner = exercises[partnerIdx]
            if partner.setsCompleted.count < partner.exercise.sets {
                currentIdx = partnerIdx
            }
            isResting = true
        } else {
            isResting = true
        }
    }

    func completeTimedSet() {
        completeSet(reps: timerSelectedDuration)
    }

    func startExerciseTimer() {
        timerRemaining = timerSelectedDuration
        timerRunning = true
    }

    func stopExerciseTimer() {
        timerRunning = false
        timerRemaining = 0
    }

    func continueWorkout() {
        isResting = false
    }

    func skipExercise() {
        if currentIdx + 1 >= exercises.count {
            isFinished = true
            HapticService.workoutFinished()
        } else {
            currentIdx += 1
            isResting = false
        }
    }

    func adjustWeight(delta: Double) {
        guard currentIdx < exercises.count else { return }
        exercises[currentIdx].currentWeight = max(0, exercises[currentIdx].currentWeight + delta)
    }

    func jumpToExercise(idx: Int) {
        currentIdx = idx
        isResting = false
        showOverview = false
    }

    // MARK: - Pause / Cancel / Save

    func pauseAndExit() {
        let paused = PausedWorkout(
            routineName: routineName,
            exercises: exercises,
            currentExerciseIdx: currentIdx,
            startTimestamp: startTime,
            elapsedSecs: elapsed
        )
        StorageService.shared.savePausedWorkout(paused)
        cleanup()
    }

    func cancelWorkout() {
        StorageService.shared.clearPausedWorkout()
        cleanup()
    }

    func saveWorkout() {
        isSaving = true
        StorageService.shared.clearSyncFailed()

        let records: [ExerciseRecord] = exercises
            .filter { !$0.setsCompleted.isEmpty }
            .map { ExerciseRecord(name: $0.exercise.name, sets: $0.setsCompleted) }

        StorageService.shared.saveSession(
            routineName: routineName,
            exercises: records,
            durationSecs: elapsed
        )

        // Poll for sync failure
        var checkCount = 0
        Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { [weak self] timer in
            checkCount += 1

            if StorageService.shared.getSyncFailedSession() != nil {
                self?.isSaving = false
                self?.showSyncWarning = true
                timer.invalidate()
                return
            }

            if checkCount >= 10 {
                self?.isSaving = false
                timer.invalidate()
            }
        }

        // Save to HealthKit
        let startDate = Date(timeIntervalSince1970: TimeInterval(startTime))
        let endDate = Date()
        let cal = Double(finishStats.calories)
        let rpe = selectedRPE
        let name = routineName
        Task {
            await HealthKitService.shared.saveWorkout(
                name: name,
                start: startDate,
                end: endDate,
                calories: cal,
                rpe: rpe
            )
        }

        cleanup()
    }

    // MARK: - Computed

    var finishStats: (volume: Double, calories: Int, durationMins: Int) {
        let durationMins = max(1, Int((elapsed + 30) / 60))
        let totalVolume = exercises.flatMap(\.setsCompleted).reduce(0.0) { $0 + $1.weight * Double($1.reps) }

        let efficiency = durationMins > 0 ? totalVolume / Double(durationMins) : 0
        let efficiencyBonus = min(efficiency / 200.0, 1.0) * 1.5
        let met = 5.0 + efficiencyBonus
        let hours = Double(durationMins) / 60.0
        let calories = Int((hours * bodyweight * met).rounded())

        return (totalVolume, calories, durationMins)
    }

    // MARK: - Private

    private func findPartnerIdx(for idx: Int) -> Int? {
        guard exercises[idx].exercise.isSuperset,
              let partnerName = exercises[idx].exercise.supersetWith
        else { return nil }
        return exercises.firstIndex { $0.exercise.name == partnerName }
    }

    private func advanceToNext(from idx: Int) {
        var nextIdx = idx + 1
        while nextIdx < exercises.count {
            let next = exercises[nextIdx]
            if next.setsCompleted.count < next.exercise.sets {
                break
            }
            nextIdx += 1
        }

        if nextIdx < exercises.count {
            currentIdx = nextIdx
            isResting = true
        } else {
            isFinished = true
            HapticService.workoutFinished()
        }
    }
}
