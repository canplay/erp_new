//! Notification Preferences Module
//!
//! Manages per-user notification preferences including:
//! - Channel preferences (email, SMS, push, in-app)
//! - Quiet hours configuration
//! - Per-type opt-in/opt-out

use chrono::{NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;

/// Notification preference errors
#[derive(Error, Debug)]
pub enum PreferenceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Invalid preference: {0}")]
    Invalid(String),

    #[error("Preference not found for user: {0}")]
    NotFound(i64),
}

/// Result type for preference operations
pub type PreferenceResult<T> = Result<T, PreferenceError>;

/// Notification channel types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NotificationChannel {
    /// Email notifications
    Email,
    /// SMS notifications
    Sms,
    /// Push notifications (mobile/web)
    Push,
    /// In-app notifications
    #[default]
    InApp,
}

impl std::fmt::Display for NotificationChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Email => write!(f, "email"),
            Self::Sms => write!(f, "sms"),
            Self::Push => write!(f, "push"),
            Self::InApp => write!(f, "in_app"),
        }
    }
}

impl std::str::FromStr for NotificationChannel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "email" => Ok(Self::Email),
            "sms" => Ok(Self::Sms),
            "push" => Ok(Self::Push),
            "in_app" | "inapp" => Ok(Self::InApp),
            _ => Err(format!("Unknown notification channel: {s}")),
        }
    }
}

/// Notification type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NotificationType {
    /// System notifications (always on)
    #[default]
    System,
    /// Workflow/task notifications
    Workflow,
    /// Security alerts
    Security,
    /// Marketing/promotional
    Marketing,
    /// Social/interaction
    Social,
    /// Billing/payment
    Billing,
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::Workflow => write!(f, "workflow"),
            Self::Security => write!(f, "security"),
            Self::Marketing => write!(f, "marketing"),
            Self::Social => write!(f, "social"),
            Self::Billing => write!(f, "billing"),
        }
    }
}

impl std::str::FromStr for NotificationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "system" => Ok(Self::System),
            "workflow" => Ok(Self::Workflow),
            "security" => Ok(Self::Security),
            "marketing" => Ok(Self::Marketing),
            "social" => Ok(Self::Social),
            "billing" => Ok(Self::Billing),
            _ => Err(format!("Unknown notification type: {s}")),
        }
    }
}

/// Quiet hours configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    /// Whether quiet hours are enabled
    pub enabled: bool,
    /// Start time (local time)
    pub start_time: NaiveTime,
    /// End time (local time)
    pub end_time: NaiveTime,
    /// Timezone (e.g., "Asia/Shanghai")
    pub timezone: String,
    /// Whether to allow urgent notifications during quiet hours
    pub allow_urgent: bool,
}

impl Default for QuietHours {
    fn default() -> Self {
        Self {
            enabled: false,
            start_time: NaiveTime::from_hms_opt(22, 0, 0).unwrap_or_default(),
            end_time: NaiveTime::from_hms_opt(8, 0, 0).unwrap_or_default(),
            timezone: "UTC".to_string(),
            allow_urgent: true,
        }
    }
}

impl QuietHours {
    /// Check if the given time falls within quiet hours
    pub fn is_quiet_time(&self, time: NaiveTime) -> bool {
        if !self.enabled {
            return false;
        }

        if self.start_time <= self.end_time {
            // Normal range (e.g., 22:00 - 08:00 doesn't apply here)
            time >= self.start_time && time <= self.end_time
        } else {
            // Overnight range (e.g., 22:00 - 08:00)
            time >= self.start_time || time <= self.end_time
        }
    }
}

/// Per-channel preference for a specific notification type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypePreference {
    pub notification_type: NotificationType,
    pub email_enabled: bool,
    pub sms_enabled: bool,
    pub push_enabled: bool,
    pub in_app_enabled: bool,
}

impl Default for TypePreference {
    fn default() -> Self {
        Self {
            notification_type: NotificationType::System,
            email_enabled: true,
            sms_enabled: false,
            push_enabled: true,
            in_app_enabled: true,
        }
    }
}

/// Complete notification preferences for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub user_id: i64,
    /// Global channel toggles
    pub email_enabled: bool,
    pub sms_enabled: bool,
    pub push_enabled: bool,
    pub in_app_enabled: bool,
    /// Quiet hours configuration
    pub quiet_hours: QuietHours,
    /// Per-type preferences
    pub type_preferences: Vec<TypePreference>,
    /// Updated timestamp
    pub updated_at: chrono::DateTime<Utc>,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            user_id: 0,
            email_enabled: true,
            sms_enabled: false,
            push_enabled: true,
            in_app_enabled: true,
            quiet_hours: QuietHours::default(),
            type_preferences: vec![
                TypePreference {
                    notification_type: NotificationType::System,
                    email_enabled: true,
                    sms_enabled: true,
                    push_enabled: true,
                    in_app_enabled: true,
                },
                TypePreference {
                    notification_type: NotificationType::Workflow,
                    email_enabled: true,
                    sms_enabled: false,
                    push_enabled: true,
                    in_app_enabled: true,
                },
                TypePreference {
                    notification_type: NotificationType::Security,
                    email_enabled: true,
                    sms_enabled: true,
                    push_enabled: true,
                    in_app_enabled: true,
                },
                TypePreference {
                    notification_type: NotificationType::Marketing,
                    email_enabled: false,
                    sms_enabled: false,
                    push_enabled: false,
                    in_app_enabled: true,
                },
                TypePreference {
                    notification_type: NotificationType::Social,
                    email_enabled: true,
                    sms_enabled: false,
                    push_enabled: true,
                    in_app_enabled: true,
                },
                TypePreference {
                    notification_type: NotificationType::Billing,
                    email_enabled: true,
                    sms_enabled: true,
                    push_enabled: true,
                    in_app_enabled: true,
                },
            ],
            updated_at: Utc::now(),
        }
    }
}

impl NotificationPreferences {
    /// Check if a notification should be sent via the given channel
    pub fn should_send(&self, channel: NotificationChannel, notification_type: NotificationType) -> bool {
        // Check global channel toggle
        let global_enabled = match channel {
            NotificationChannel::Email => self.email_enabled,
            NotificationChannel::Sms => self.sms_enabled,
            NotificationChannel::Push => self.push_enabled,
            NotificationChannel::InApp => self.in_app_enabled,
        };

        if !global_enabled {
            return false;
        }

        // System notifications are always sent
        if notification_type == NotificationType::System {
            return true;
        }

        // Check per-type preference
        if let Some(pref) = self.type_preferences.iter().find(|p| p.notification_type == notification_type) {
            match channel {
                NotificationChannel::Email => pref.email_enabled,
                NotificationChannel::Sms => pref.sms_enabled,
                NotificationChannel::Push => pref.push_enabled,
                NotificationChannel::InApp => pref.in_app_enabled,
            }
        } else {
            // Default: allow if global is enabled
            true
        }
    }

    /// Check if currently in quiet hours
    pub fn is_quiet_hours(&self) -> bool {
        let now = Utc::now().naive_utc().time();
        self.quiet_hours.is_quiet_time(now)
    }
}

/// Notification preferences repository
pub struct NotificationPreferencesRepository {
    pool: PgPool,
}

impl NotificationPreferencesRepository {
    /// Create a new repository instance
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get preferences for a user
    pub async fn get_preferences(&self, user_id: i64) -> PreferenceResult<NotificationPreferences> {
        let row = sqlx::query_as::<_, NotificationPreferencesRow>(
            r#"
            SELECT email_enabled, sms_enabled, push_enabled, in_app_enabled,
                   quiet_hours_enabled, quiet_hours_start, quiet_hours_end, quiet_hours_timezone,
                   quiet_hours_allow_urgent, updated_at
            FROM notification_preferences
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => {
                let type_prefs = self.get_type_preferences(user_id).await?;
                Ok(NotificationPreferences {
                    user_id,
                    email_enabled: r.email_enabled,
                    sms_enabled: r.sms_enabled,
                    push_enabled: r.push_enabled,
                    in_app_enabled: r.in_app_enabled,
                    quiet_hours: QuietHours {
                        enabled: r.quiet_hours_enabled,
                        start_time: r.quiet_hours_start,
                        end_time: r.quiet_hours_end,
                        timezone: r.quiet_hours_timezone,
                        allow_urgent: r.quiet_hours_allow_urgent,
                    },
                    type_preferences: type_prefs,
                    updated_at: r.updated_at,
                })
            }
            None => {
                // Return default preferences
                Ok(NotificationPreferences {
                    user_id,
                    ..Default::default()
                })
            }
        }
    }

    /// Get per-type preferences for a user
    async fn get_type_preferences(&self, user_id: i64) -> PreferenceResult<Vec<TypePreference>> {
        let rows = sqlx::query_as::<_, TypePreferenceRow>(
            r#"
            SELECT notification_type, email_enabled, sms_enabled, push_enabled, in_app_enabled
            FROM notification_type_preferences
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| TypePreference {
                notification_type: r.notification_type.parse().unwrap_or(NotificationType::System),
                email_enabled: r.email_enabled,
                sms_enabled: r.sms_enabled,
                push_enabled: r.push_enabled,
                in_app_enabled: r.in_app_enabled,
            })
            .collect())
    }

    /// Update preferences for a user
    pub async fn update_preferences(
        &self,
        user_id: i64,
        email_enabled: Option<bool>,
        sms_enabled: Option<bool>,
        push_enabled: Option<bool>,
        in_app_enabled: Option<bool>,
    ) -> PreferenceResult<NotificationPreferences> {
        sqlx::query(
            r#"
            INSERT INTO notification_preferences (user_id, email_enabled, sms_enabled, push_enabled, in_app_enabled, updated_at)
            VALUES ($1, COALESCE($2, true), COALESCE($3, false), COALESCE($4, true), COALESCE($5, true), NOW())
            ON CONFLICT (user_id) DO UPDATE SET
                email_enabled = COALESCE($2, notification_preferences.email_enabled),
                sms_enabled = COALESCE($3, notification_preferences.sms_enabled),
                push_enabled = COALESCE($4, notification_preferences.push_enabled),
                in_app_enabled = COALESCE($5, notification_preferences.in_app_enabled),
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(email_enabled)
        .bind(sms_enabled)
        .bind(push_enabled)
        .bind(in_app_enabled)
        .execute(&self.pool)
        .await?;

        self.get_preferences(user_id).await
    }

    /// Update quiet hours for a user
    pub async fn update_quiet_hours(
        &self,
        user_id: i64,
        enabled: Option<bool>,
        start_time: Option<NaiveTime>,
        end_time: Option<NaiveTime>,
        timezone: Option<&str>,
        allow_urgent: Option<bool>,
    ) -> PreferenceResult<NotificationPreferences> {
        sqlx::query(
            r#"
            INSERT INTO notification_preferences (user_id, quiet_hours_enabled, quiet_hours_start, quiet_hours_end, quiet_hours_timezone, quiet_hours_allow_urgent, updated_at)
            VALUES ($1, COALESCE($2, false), COALESCE($3, '22:00:00'::time), COALESCE($4, '08:00:00'::time), COALESCE($5, 'UTC'), COALESCE($6, true), NOW())
            ON CONFLICT (user_id) DO UPDATE SET
                quiet_hours_enabled = COALESCE($2, notification_preferences.quiet_hours_enabled),
                quiet_hours_start = COALESCE($3, notification_preferences.quiet_hours_start),
                quiet_hours_end = COALESCE($4, notification_preferences.quiet_hours_end),
                quiet_hours_timezone = COALESCE($5, notification_preferences.quiet_hours_timezone),
                quiet_hours_allow_urgent = COALESCE($6, notification_preferences.quiet_hours_allow_urgent),
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(enabled)
        .bind(start_time)
        .bind(end_time)
        .bind(timezone)
        .bind(allow_urgent)
        .execute(&self.pool)
        .await?;

        self.get_preferences(user_id).await
    }

    /// Update per-type preference
    pub async fn update_type_preference(
        &self,
        user_id: i64,
        notification_type: NotificationType,
        email_enabled: Option<bool>,
        sms_enabled: Option<bool>,
        push_enabled: Option<bool>,
        in_app_enabled: Option<bool>,
    ) -> PreferenceResult<TypePreference> {
        sqlx::query(
            r#"
            INSERT INTO notification_type_preferences (user_id, notification_type, email_enabled, sms_enabled, push_enabled, in_app_enabled, updated_at)
            VALUES ($1, $2, COALESCE($3, true), COALESCE($4, false), COALESCE($5, true), COALESCE($6, true), NOW())
            ON CONFLICT (user_id, notification_type) DO UPDATE SET
                email_enabled = COALESCE($3, notification_type_preferences.email_enabled),
                sms_enabled = COALESCE($4, notification_type_preferences.sms_enabled),
                push_enabled = COALESCE($5, notification_type_preferences.push_enabled),
                in_app_enabled = COALESCE($6, notification_type_preferences.in_app_enabled),
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(notification_type.to_string())
        .bind(email_enabled)
        .bind(sms_enabled)
        .bind(push_enabled)
        .bind(in_app_enabled)
        .execute(&self.pool)
        .await?;

        Ok(TypePreference {
            notification_type,
            email_enabled: email_enabled.unwrap_or(true),
            sms_enabled: sms_enabled.unwrap_or(false),
            push_enabled: push_enabled.unwrap_or(true),
            in_app_enabled: in_app_enabled.unwrap_or(true),
        })
    }

    /// Delete all preferences for a user
    pub async fn delete_preferences(&self, user_id: i64) -> PreferenceResult<bool> {
        let result = sqlx::query("DELETE FROM notification_preferences WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM notification_type_preferences WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

/// Row type for notification_preferences query
#[derive(Debug, sqlx::FromRow)]
struct NotificationPreferencesRow {
    email_enabled: bool,
    sms_enabled: bool,
    push_enabled: bool,
    in_app_enabled: bool,
    quiet_hours_enabled: bool,
    quiet_hours_start: NaiveTime,
    quiet_hours_end: NaiveTime,
    quiet_hours_timezone: String,
    quiet_hours_allow_urgent: bool,
    updated_at: chrono::DateTime<Utc>,
}

/// Row type for notification_type_preferences query
#[derive(Debug, sqlx::FromRow)]
struct TypePreferenceRow {
    notification_type: String,
    email_enabled: bool,
    sms_enabled: bool,
    push_enabled: bool,
    in_app_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_channel_display() {
        assert_eq!(NotificationChannel::Email.to_string(), "email");
        assert_eq!(NotificationChannel::Sms.to_string(), "sms");
        assert_eq!(NotificationChannel::Push.to_string(), "push");
        assert_eq!(NotificationChannel::InApp.to_string(), "in_app");
    }

    #[test]
    fn test_notification_channel_from_str() {
        assert_eq!("email".parse::<NotificationChannel>().expect("test assertion"), NotificationChannel::Email);
        assert_eq!("sms".parse::<NotificationChannel>().expect("test assertion"), NotificationChannel::Sms);
        assert_eq!("push".parse::<NotificationChannel>().expect("test assertion"), NotificationChannel::Push);
        assert_eq!("in_app".parse::<NotificationChannel>().expect("test assertion"), NotificationChannel::InApp);
    }

    #[test]
    fn test_notification_type_display() {
        assert_eq!(NotificationType::System.to_string(), "system");
        assert_eq!(NotificationType::Workflow.to_string(), "workflow");
        assert_eq!(NotificationType::Security.to_string(), "security");
    }

    #[test]
    fn test_quiet_hours_default() {
        let qh = QuietHours::default();
        assert!(!qh.enabled);
        assert_eq!(qh.start_time, NaiveTime::from_hms_opt(22, 0, 0).expect("test assertion"));
        assert_eq!(qh.end_time, NaiveTime::from_hms_opt(8, 0, 0).expect("test assertion"));
    }

    #[test]
    fn test_quiet_hours_is_quiet_time() {
        let mut qh = QuietHours::default();
        qh.enabled = true;

        // 23:00 should be quiet time (22:00 - 08:00)
        assert!(qh.is_quiet_time(NaiveTime::from_hms_opt(23, 0, 0).expect("test assertion")));

        // 03:00 should be quiet time
        assert!(qh.is_quiet_time(NaiveTime::from_hms_opt(3, 0, 0).expect("test assertion")));

        // 12:00 should NOT be quiet time
        assert!(!qh.is_quiet_time(NaiveTime::from_hms_opt(12, 0, 0).expect("test assertion")));

        // When disabled, no time is quiet
        qh.enabled = false;
        assert!(!qh.is_quiet_time(NaiveTime::from_hms_opt(23, 0, 0).expect("test assertion")));
    }

    #[test]
    fn test_notification_preferences_default() {
        let prefs = NotificationPreferences::default();
        assert!(prefs.email_enabled);
        assert!(!prefs.sms_enabled);
        assert!(prefs.push_enabled);
        assert!(prefs.in_app_enabled);
        assert_eq!(prefs.type_preferences.len(), 6);
    }

    #[test]
    fn test_should_send() {
        let prefs = NotificationPreferences::default();

        // Email should be sent for workflow
        assert!(prefs.should_send(NotificationChannel::Email, NotificationType::Workflow));

        // SMS should NOT be sent for workflow (disabled by default)
        assert!(!prefs.should_send(NotificationChannel::Sms, NotificationType::Workflow));

        // Marketing email should NOT be sent (disabled by default)
        assert!(!prefs.should_send(NotificationChannel::Email, NotificationType::Marketing));

        // System notifications always sent
        assert!(prefs.should_send(NotificationChannel::Email, NotificationType::System));
    }

    #[test]
    fn test_should_send_global_disabled() {
        let mut prefs = NotificationPreferences::default();
        prefs.email_enabled = false;

        // Email should NOT be sent even for system when globally disabled
        assert!(!prefs.should_send(NotificationChannel::Email, NotificationType::System));
    }
}
