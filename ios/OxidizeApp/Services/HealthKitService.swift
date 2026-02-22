import Foundation
import HealthKit

final class HealthKitService: Sendable {
    static let shared = HealthKitService()

    private let store = HKHealthStore()

    private let bodyMassType = HKQuantityType(.bodyMass)
    private let activeEnergyType = HKQuantityType(.activeEnergyBurned)
    private let workoutType = HKWorkoutType.workoutType()

    var isAvailable: Bool {
        HKHealthStore.isHealthDataAvailable()
    }

    // MARK: - Authorization

    func requestAuthorization() async {
        guard isAvailable else { return }

        let readTypes: Set<HKObjectType> = [bodyMassType]
        let writeTypes: Set<HKSampleType> = [bodyMassType, activeEnergyType, workoutType]

        do {
            try await store.requestAuthorization(toShare: writeTypes, read: readTypes)
        } catch {
            print("[HealthKit] Authorization failed: \(error)")
        }
    }

    // MARK: - Status

    /// Returns .notDetermined, .sharingDenied, or .sharingAuthorized for write access
    func authorizationStatus() -> HKAuthorizationStatus {
        guard isAvailable else { return .notDetermined }
        return store.authorizationStatus(for: bodyMassType)
    }

    // MARK: - Bodyweight

    func fetchLatestBodyweight() async -> (weight: Double, date: Date)? {
        guard isAvailable else { return nil }

        let sortDescriptor = NSSortDescriptor(key: HKSampleSortIdentifierStartDate, ascending: false)
        let query = HKSampleQuery(
            sampleType: bodyMassType,
            predicate: nil,
            limit: 1,
            sortDescriptors: [sortDescriptor]
        ) { _, _, _ in }

        return await withCheckedContinuation { continuation in
            let query = HKSampleQuery(
                sampleType: bodyMassType,
                predicate: nil,
                limit: 1,
                sortDescriptors: [sortDescriptor]
            ) { _, results, error in
                guard let sample = results?.first as? HKQuantitySample, error == nil else {
                    continuation.resume(returning: nil)
                    return
                }
                let kg = sample.quantity.doubleValue(for: .gramUnit(with: .kilo))
                continuation.resume(returning: (kg, sample.startDate))
            }
            store.execute(query)
        }
    }

    func saveBodyweight(_ kg: Double) async {
        guard isAvailable else { return }

        let quantity = HKQuantity(unit: .gramUnit(with: .kilo), doubleValue: kg)
        let sample = HKQuantitySample(
            type: bodyMassType,
            quantity: quantity,
            start: Date(),
            end: Date()
        )

        do {
            try await store.save(sample)
        } catch {
            print("[HealthKit] Save bodyweight failed: \(error)")
        }
    }

    // MARK: - Workout

    func saveWorkout(
        name: String,
        start: Date,
        end: Date,
        calories: Double,
        rpe: Int?
    ) async {
        guard isAvailable else { return }

        let energyBurned = HKQuantity(unit: .kilocalorie(), doubleValue: calories)

        var metadata: [String: Any] = [
            HKMetadataKeyWorkoutBrandName: "Oxidize"
        ]
        if let rpe {
            // Apple's effort score: integer value
            metadata["HKMetadataKeyWorkoutEffortScore"] = rpe
        }

        let workout = HKWorkout(
            activityType: .traditionalStrengthTraining,
            start: start,
            end: end,
            duration: end.timeIntervalSince(start),
            totalEnergyBurned: energyBurned,
            totalDistance: nil,
            metadata: metadata
        )

        do {
            try await store.save(workout)
        } catch {
            print("[HealthKit] Save workout failed: \(error)")
        }
    }
}
