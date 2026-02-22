import Foundation

enum SupabaseConfig {
    static let url = "https://ytnwppbepeojvyedrbnb.supabase.co"
    static let anonKey = "sb_publishable_Oqp9Oc-Io5o3o3MUwIVD2A_Tvv_dCuS"
    static let inactivityTimeoutSecs: Int64 = 4 * 60 * 60 // 4 hours
}

enum StorageKeys {
    static let database = "oxidize_db_v2"
    static let authSession = "oxidize_auth_session"
    static let pausedWorkout = "oxidize_paused_workout"
    static let syncStatus = "oxidize_sync_status"
    static let dataVersion = "oxidize_data_version"
    static let activeRoutine = "oxidize_active_routine"
    static let displayName = "oxidize_display_name"
    static let lastActivity = "oxidize_last_activity"
    static let syncFailed = "oxidize_sync_failed"
}

let BIG_FOUR: [String] = ["Squats", "Deadlift", "Bench Press", "Shoulder Press"]

let PASS_COLORS: [String] = ["pass-a", "pass-b", "pass-c", "pass-d", "pass-e"]
