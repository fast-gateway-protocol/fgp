//! Apple Reminders queries via EventKit.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone, Utc};
use objc2::rc::Retained;
use objc2_event_kit::{
    EKAuthorizationStatus, EKCalendar, EKEntityType, EKEventStore, EKReminder,
};
use objc2_foundation::{NSArray, NSDate, NSString};
use serde::{Deserialize, Serialize};
use std::sync::mpsc;

/// Reminder list (calendar) information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderListInfo {
    pub id: String,
    pub title: String,
    pub color: Option<String>,
    pub source: Option<String>,
    pub allows_content_modifications: bool,
}

/// Reminder information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderInfo {
    pub id: String,
    pub title: String,
    pub notes: Option<String>,
    pub priority: i64,
    pub is_completed: bool,
    pub completion_date: Option<String>,
    pub due_date: Option<String>,
    pub creation_date: Option<String>,
    pub last_modified_date: Option<String>,
    pub list_id: Option<String>,
    pub list_title: Option<String>,
    pub has_alarms: bool,
    pub is_recurring: bool,
}

/// Reminder store wrapper.
/// Note: EKEventStore is not Send/Sync, so this must be created per-request.
pub struct ReminderStore {
    store: Retained<EKEventStore>,
}

impl ReminderStore {
    /// Create a new reminder store.
    pub fn new() -> Result<Self> {
        let store = unsafe { EKEventStore::new() };
        Ok(Self { store })
    }

    /// Check authorization status for reminders.
    pub fn authorization_status() -> EKAuthorizationStatus {
        unsafe { EKEventStore::authorizationStatusForEntityType(EKEntityType::Reminder) }
    }

    /// Check if access is granted.
    pub fn check_access(&self) -> Result<()> {
        let status = Self::authorization_status();

        match status {
            EKAuthorizationStatus::FullAccess => Ok(()),
            EKAuthorizationStatus::NotDetermined => Err(anyhow!(
                "Reminders access not determined. Please grant access in System Settings > Privacy > Reminders"
            )),
            EKAuthorizationStatus::Denied => Err(anyhow!(
                "Reminders access denied. Please enable in System Settings > Privacy > Reminders"
            )),
            EKAuthorizationStatus::Restricted => {
                Err(anyhow!("Reminders access is restricted by system policy"))
            }
            EKAuthorizationStatus::WriteOnly => Err(anyhow!(
                "Only write access granted. Please grant full access in System Settings > Privacy > Reminders"
            )),
            _ => Err(anyhow!("Unknown authorization status")),
        }
    }

    /// Get all reminder lists (calendars).
    pub fn lists(&self) -> Result<Vec<ReminderListInfo>> {
        self.check_access()?;

        let calendars = unsafe { self.store.calendarsForEntityType(EKEntityType::Reminder) };

        let mut result = Vec::new();
        let count = calendars.count();
        for i in 0..count {
            let cal = calendars.objectAtIndex(i);
            result.push(calendar_to_list_info(&cal));
        }

        Ok(result)
    }

    /// Get all reminders (both complete and incomplete).
    pub fn all_reminders(&self, list_ids: Option<Vec<String>>) -> Result<Vec<ReminderInfo>> {
        self.check_access()?;

        // Get calendars to search
        let calendars = self.get_calendars(list_ids)?;

        // Create predicate for all reminders
        let predicate =
            unsafe { self.store.predicateForRemindersInCalendars(calendars.as_deref()) };

        // Fetch reminders synchronously using a channel
        self.fetch_reminders(&predicate)
    }

    /// Get incomplete reminders.
    pub fn incomplete_reminders(
        &self,
        list_ids: Option<Vec<String>>,
        due_start: Option<DateTime<Local>>,
        due_end: Option<DateTime<Local>>,
    ) -> Result<Vec<ReminderInfo>> {
        self.check_access()?;

        let calendars = self.get_calendars(list_ids)?;

        let start_date = due_start.map(datetime_to_nsdate);
        let end_date = due_end.map(datetime_to_nsdate);

        let predicate = unsafe {
            self.store
                .predicateForIncompleteRemindersWithDueDateStarting_ending_calendars(
                    start_date.as_deref(),
                    end_date.as_deref(),
                    calendars.as_deref(),
                )
        };

        self.fetch_reminders(&predicate)
    }

    /// Get completed reminders.
    pub fn completed_reminders(
        &self,
        list_ids: Option<Vec<String>>,
        completion_start: Option<DateTime<Local>>,
        completion_end: Option<DateTime<Local>>,
    ) -> Result<Vec<ReminderInfo>> {
        self.check_access()?;

        let calendars = self.get_calendars(list_ids)?;

        let start_date = completion_start.map(datetime_to_nsdate);
        let end_date = completion_end.map(datetime_to_nsdate);

        let predicate = unsafe {
            self.store
                .predicateForCompletedRemindersWithCompletionDateStarting_ending_calendars(
                    start_date.as_deref(),
                    end_date.as_deref(),
                    calendars.as_deref(),
                )
        };

        self.fetch_reminders(&predicate)
    }

    /// Get reminders due today.
    pub fn due_today(&self, list_ids: Option<Vec<String>>) -> Result<Vec<ReminderInfo>> {
        let now = Local::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap();

        self.incomplete_reminders(
            list_ids,
            Some(Local.from_local_datetime(&start).unwrap()),
            Some(Local.from_local_datetime(&end).unwrap()),
        )
    }

    /// Get overdue reminders (past due and incomplete).
    pub fn overdue(&self, list_ids: Option<Vec<String>>) -> Result<Vec<ReminderInfo>> {
        let now = Local::now();
        // Get reminders due before now
        self.incomplete_reminders(list_ids, None, Some(now))
            .map(|reminders| {
                // Filter to only those actually overdue (due date in past)
                reminders
                    .into_iter()
                    .filter(|r| {
                        r.due_date.as_ref().map_or(false, |d| {
                            DateTime::parse_from_rfc3339(d)
                                .map(|dt| dt < now)
                                .unwrap_or(false)
                        })
                    })
                    .collect()
            })
    }

    /// Get upcoming reminders (due within N days).
    pub fn upcoming(&self, days: u32, list_ids: Option<Vec<String>>) -> Result<Vec<ReminderInfo>> {
        let now = Local::now();
        let end = now + Duration::days(days as i64);

        self.incomplete_reminders(list_ids, Some(now), Some(end))
    }

    /// Search reminders by title/notes.
    pub fn search(
        &self,
        query: &str,
        include_completed: bool,
        list_ids: Option<Vec<String>>,
    ) -> Result<Vec<ReminderInfo>> {
        let reminders = if include_completed {
            self.all_reminders(list_ids)?
        } else {
            self.incomplete_reminders(list_ids, None, None)?
        };

        let query_lower = query.to_lowercase();

        Ok(reminders
            .into_iter()
            .filter(|r| {
                r.title.to_lowercase().contains(&query_lower)
                    || r.notes
                        .as_ref()
                        .map(|n| n.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .collect())
    }

    /// Get reminders for a specific date.
    pub fn on_date(&self, date: NaiveDate, list_ids: Option<Vec<String>>) -> Result<Vec<ReminderInfo>> {
        let start = date.and_hms_opt(0, 0, 0).unwrap();
        let end = date.and_hms_opt(23, 59, 59).unwrap();

        self.incomplete_reminders(
            list_ids,
            Some(Local.from_local_datetime(&start).unwrap()),
            Some(Local.from_local_datetime(&end).unwrap()),
        )
    }

    // Helper to get calendars from IDs
    fn get_calendars(
        &self,
        list_ids: Option<Vec<String>>,
    ) -> Result<Option<Retained<NSArray<EKCalendar>>>> {
        match list_ids {
            Some(ids) => {
                let cals: Vec<Retained<EKCalendar>> = ids
                    .iter()
                    .filter_map(|id| {
                        let ns_id = NSString::from_str(id);
                        unsafe { self.store.calendarWithIdentifier(&ns_id) }
                    })
                    .collect();

                if cals.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(NSArray::from_retained_slice(&cals)))
                }
            }
            None => Ok(None),
        }
    }

    // Fetch reminders using predicate (synchronous wrapper)
    fn fetch_reminders(
        &self,
        predicate: &objc2_foundation::NSPredicate,
    ) -> Result<Vec<ReminderInfo>> {
        // Use a channel to synchronize the async callback
        let (tx, rx) = mpsc::channel();

        // Create the completion block
        let block = block2::RcBlock::new(move |reminders: *mut NSArray<EKReminder>| {
            let result = if reminders.is_null() {
                Vec::new()
            } else {
                let reminders = unsafe { &*reminders };
                let mut result = Vec::new();
                let count = reminders.count();
                for i in 0..count {
                    let reminder = reminders.objectAtIndex(i);
                    result.push(reminder_to_info(&reminder));
                }
                result
            };
            let _ = tx.send(result);
        });

        // Start the fetch
        unsafe {
            self.store
                .fetchRemindersMatchingPredicate_completion(predicate, &block);
        }

        // Wait for the result (with timeout)
        rx.recv_timeout(std::time::Duration::from_secs(30))
            .map_err(|_| anyhow!("Timeout fetching reminders"))
    }
}

impl Default for ReminderStore {
    fn default() -> Self {
        Self::new().expect("Failed to create ReminderStore")
    }
}

/// Helper to convert NSString to Rust String.
fn nsstring_to_string(ns: &NSString) -> String {
    ns.to_string()
}

/// Convert EKCalendar to ReminderListInfo.
fn calendar_to_list_info(cal: &EKCalendar) -> ReminderListInfo {
    let id = nsstring_to_string(unsafe { &cal.calendarIdentifier() });
    let title = nsstring_to_string(unsafe { &cal.title() });

    // Get color (simplified)
    let color = unsafe { cal.CGColor().map(|_| "#reminder".to_string()) };

    // Get source name
    let source = unsafe { cal.source().map(|s| nsstring_to_string(&s.title())) };

    let allows_modifications = unsafe { cal.allowsContentModifications() };

    ReminderListInfo {
        id,
        title,
        color,
        source,
        allows_content_modifications: allows_modifications,
    }
}

/// Convert EKReminder to ReminderInfo.
fn reminder_to_info(reminder: &EKReminder) -> ReminderInfo {
    // calendarItemIdentifier returns Retained<NSString> directly
    let id = unsafe { nsstring_to_string(&reminder.calendarItemIdentifier()) };

    // title returns Retained<NSString> directly
    let title = unsafe { nsstring_to_string(&reminder.title()) };

    // notes returns Option<Retained<NSString>>
    let notes = unsafe { reminder.notes().map(|s| nsstring_to_string(&s)) };

    // priority (0 = none, 1-4 = high, 5 = medium, 6-9 = low)
    let priority = unsafe { reminder.priority() as i64 };

    // completion status
    let is_completed = unsafe { reminder.isCompleted() };

    // completionDate returns Option
    let completion_date = unsafe { reminder.completionDate().map(|d| nsdate_to_string(&d)) };

    // dueDateComponents -> convert to date string if available
    let due_date = unsafe {
        reminder.dueDateComponents().and_then(|components| {
            // Try to convert NSDateComponents to a date
            components_to_date_string(&components)
        })
    };

    // creationDate returns Option
    let creation_date = unsafe { reminder.creationDate().map(|d| nsdate_to_string(&d)) };

    // lastModifiedDate returns Option
    let last_modified_date = unsafe { reminder.lastModifiedDate().map(|d| nsdate_to_string(&d)) };

    // calendar (list) info - returns Option<Retained<EKCalendar>>
    let (list_id, list_title) = unsafe {
        reminder
            .calendar()
            .map(|cal| {
                (
                    Some(nsstring_to_string(&cal.calendarIdentifier())),
                    Some(nsstring_to_string(&cal.title())),
                )
            })
            .unwrap_or((None, None))
    };

    // has alarms
    let has_alarms = unsafe { reminder.hasAlarms() };

    // is recurring
    let is_recurring = unsafe { reminder.hasRecurrenceRules() };

    ReminderInfo {
        id,
        title,
        notes,
        priority,
        is_completed,
        completion_date,
        due_date,
        creation_date,
        last_modified_date,
        list_id,
        list_title,
        has_alarms,
        is_recurring,
    }
}

/// Convert NSDateComponents to date string.
fn components_to_date_string(
    components: &objc2_foundation::NSDateComponents,
) -> Option<String> {
    // Get the date from components using NSCalendar
    let calendar = objc2_foundation::NSCalendar::currentCalendar();
    calendar.dateFromComponents(components).map(|date| nsdate_to_string(&date))
}

/// Convert DateTime to NSDate.
fn datetime_to_nsdate(dt: DateTime<Local>) -> Retained<NSDate> {
    let timestamp = dt.timestamp() as f64;
    NSDate::dateWithTimeIntervalSince1970(timestamp)
}

/// Convert NSDate to ISO string.
fn nsdate_to_string(date: &NSDate) -> String {
    let timestamp = date.timeIntervalSince1970();
    let dt = DateTime::from_timestamp(timestamp as i64, 0).unwrap_or_else(|| Utc::now());
    dt.to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[test]
    fn test_authorization_status() {
        let status = ReminderStore::authorization_status();
        println!("Reminders authorization status: {:?}", status);
    }

    #[test]
    fn test_datetime_nsdate_roundtrip() {
        let local = Local.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
        let nsdate = datetime_to_nsdate(local);
        let iso = nsdate_to_string(&nsdate);
        let parsed = DateTime::parse_from_rfc3339(&iso).expect("parse iso");

        assert_eq!(parsed.timestamp(), local.timestamp());
    }
}
