//! Apple Calendar queries via EventKit.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone, Utc};
use objc2::rc::Retained;
use objc2_event_kit::{
    EKAuthorizationStatus, EKCalendar, EKEntityType, EKEvent, EKEventStore, EKParticipant,
    EKParticipantRole, EKParticipantStatus,
};
use objc2_foundation::{NSArray, NSDate, NSString};
use serde::{Deserialize, Serialize};

/// Calendar information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarInfo {
    pub id: String,
    pub title: String,
    pub color: Option<String>,
    pub source: Option<String>,
    pub calendar_type: String,
    pub allows_content_modifications: bool,
    pub is_subscribed: bool,
}

/// Event information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    pub id: String,
    pub title: String,
    pub start: String,
    pub end: String,
    pub all_day: bool,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub url: Option<String>,
    pub calendar_id: Option<String>,
    pub calendar_title: Option<String>,
    pub organizer: Option<String>,
    pub attendees: Vec<AttendeeInfo>,
    pub has_alarms: bool,
    pub is_recurring: bool,
}

/// Attendee information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendeeInfo {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: String,
    pub status: String,
}

/// Event store wrapper.
/// Note: EKEventStore is not Send/Sync, so this must be created per-request.
pub struct CalendarStore {
    store: Retained<EKEventStore>,
}

impl CalendarStore {
    /// Create a new calendar store.
    pub fn new() -> Result<Self> {
        let store = unsafe { EKEventStore::new() };
        Ok(Self { store })
    }

    /// Check authorization status.
    pub fn authorization_status() -> EKAuthorizationStatus {
        unsafe { EKEventStore::authorizationStatusForEntityType(EKEntityType::Event) }
    }

    /// Request full access to events (async via callback).
    /// Note: This is typically called once on first use.
    pub fn request_access(&self) -> Result<()> {
        let status = Self::authorization_status();

        match status {
            EKAuthorizationStatus::FullAccess => {
                // Fully authorized
                Ok(())
            }
            EKAuthorizationStatus::NotDetermined => {
                // Need to request access - this will prompt the user
                Err(anyhow!(
                    "Calendar access not determined. Please grant access in System Settings > Privacy > Calendars"
                ))
            }
            EKAuthorizationStatus::Denied => {
                Err(anyhow!(
                    "Calendar access denied. Please enable in System Settings > Privacy > Calendars"
                ))
            }
            EKAuthorizationStatus::Restricted => {
                Err(anyhow!("Calendar access is restricted by system policy"))
            }
            EKAuthorizationStatus::WriteOnly => {
                Err(anyhow!(
                    "Only write access granted. Please grant full access in System Settings > Privacy > Calendars"
                ))
            }
            _ => Err(anyhow!("Unknown authorization status")),
        }
    }

    /// Get all calendars.
    pub fn calendars(&self) -> Result<Vec<CalendarInfo>> {
        self.request_access()?;

        let calendars = unsafe { self.store.calendarsForEntityType(EKEntityType::Event) };

        let mut result = Vec::new();
        let count = calendars.count();
        for i in 0..count {
            let cal = calendars.objectAtIndex(i);
            result.push(calendar_to_info(&cal));
        }

        Ok(result)
    }

    /// Get a specific calendar by ID.
    pub fn calendar_by_id(&self, id: &str) -> Result<Option<CalendarInfo>> {
        self.request_access()?;

        let ns_id = NSString::from_str(id);
        let cal = unsafe { self.store.calendarWithIdentifier(&ns_id) };

        Ok(cal.map(|c| calendar_to_info(&c)))
    }

    /// Get events within a date range.
    pub fn events(
        &self,
        start: DateTime<Local>,
        end: DateTime<Local>,
        calendar_ids: Option<Vec<String>>,
    ) -> Result<Vec<EventInfo>> {
        self.request_access()?;

        // Convert to NSDate
        let start_date = datetime_to_nsdate(start);
        let end_date = datetime_to_nsdate(end);

        // Get calendars to search
        let calendars: Option<Retained<NSArray<EKCalendar>>> = if let Some(ids) = calendar_ids {
            let cals: Vec<Retained<EKCalendar>> = ids
                .iter()
                .filter_map(|id| {
                    let ns_id = NSString::from_str(id);
                    unsafe { self.store.calendarWithIdentifier(&ns_id) }
                })
                .collect();

            if cals.is_empty() {
                None
            } else {
                Some(NSArray::from_retained_slice(&cals))
            }
        } else {
            None
        };

        // Create predicate
        let predicate = unsafe {
            self.store.predicateForEventsWithStartDate_endDate_calendars(
                &start_date,
                &end_date,
                calendars.as_deref(),
            )
        };

        // Fetch events
        let events = unsafe { self.store.eventsMatchingPredicate(&predicate) };

        let mut result = Vec::new();
        let count = events.count();
        for i in 0..count {
            let event = events.objectAtIndex(i);
            result.push(event_to_info(&event));
        }

        // Sort by start date
        result.sort_by(|a, b| a.start.cmp(&b.start));

        Ok(result)
    }

    /// Get today's events.
    pub fn today(&self) -> Result<Vec<EventInfo>> {
        let now = Local::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap();

        self.events(
            Local.from_local_datetime(&start).unwrap(),
            Local.from_local_datetime(&end).unwrap(),
            None,
        )
    }

    /// Get upcoming events.
    pub fn upcoming(&self, days: u32) -> Result<Vec<EventInfo>> {
        let now = Local::now();
        let end = now + Duration::days(days as i64);

        self.events(now, end, None)
    }

    /// Search events by title.
    pub fn search(&self, query: &str, days: u32) -> Result<Vec<EventInfo>> {
        let events = self.upcoming(days)?;
        let query_lower = query.to_lowercase();

        Ok(events
            .into_iter()
            .filter(|e| {
                e.title.to_lowercase().contains(&query_lower)
                    || e.location
                        .as_ref()
                        .map(|l| l.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || e.notes
                        .as_ref()
                        .map(|n| n.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .collect())
    }

    /// Get events for a specific date.
    pub fn events_on_date(&self, date: NaiveDate) -> Result<Vec<EventInfo>> {
        let start = date.and_hms_opt(0, 0, 0).unwrap();
        let end = date.and_hms_opt(23, 59, 59).unwrap();

        self.events(
            Local.from_local_datetime(&start).unwrap(),
            Local.from_local_datetime(&end).unwrap(),
            None,
        )
    }
}

impl Default for CalendarStore {
    fn default() -> Self {
        Self::new().expect("Failed to create CalendarStore")
    }
}

/// Helper to convert NSString to Rust String.
fn nsstring_to_string(ns: &NSString) -> String {
    ns.to_string()
}

/// Convert EKCalendar to CalendarInfo.
fn calendar_to_info(cal: &EKCalendar) -> CalendarInfo {
    let id = nsstring_to_string(unsafe { &cal.calendarIdentifier() });
    let title = nsstring_to_string(unsafe { &cal.title() });

    // Get color as hex string (simplified - just indicate it exists)
    let color = unsafe { cal.CGColor().map(|_| "#calendar".to_string()) };

    // Get source name
    let source = unsafe { cal.source().map(|s| nsstring_to_string(&s.title())) };

    // Get calendar type
    let calendar_type = unsafe {
        match cal.r#type() {
            objc2_event_kit::EKCalendarType::Local => "local",
            objc2_event_kit::EKCalendarType::CalDAV => "caldav",
            objc2_event_kit::EKCalendarType::Exchange => "exchange",
            objc2_event_kit::EKCalendarType::Subscription => "subscription",
            objc2_event_kit::EKCalendarType::Birthday => "birthday",
            _ => "unknown",
        }
        .to_string()
    };

    let allows_modifications = unsafe { cal.allowsContentModifications() };
    let is_subscribed = unsafe { cal.isSubscribed() };

    CalendarInfo {
        id,
        title,
        color,
        source,
        calendar_type,
        allows_content_modifications: allows_modifications,
        is_subscribed,
    }
}

/// Convert EKEvent to EventInfo.
fn event_to_info(event: &EKEvent) -> EventInfo {
    // eventIdentifier returns Option<Retained<NSString>> - can be nil for unsaved events
    let id = unsafe {
        event
            .eventIdentifier()
            .map(|s| nsstring_to_string(&s))
            .unwrap_or_default()
    };

    // title() returns Retained<NSString> directly (non-nil)
    let title = unsafe { nsstring_to_string(&event.title()) };

    // startDate/endDate return Retained<NSDate> directly (non-nil for events)
    let start = unsafe { nsdate_to_string(&event.startDate()) };
    let end = unsafe { nsdate_to_string(&event.endDate()) };

    let all_day = unsafe { event.isAllDay() };

    // Optional fields - these return Option<Retained<T>>
    let location = unsafe { event.location().map(|s| nsstring_to_string(&s)) };
    let notes = unsafe { event.notes().map(|s| nsstring_to_string(&s)) };

    // URL returns Option<Retained<NSURL>>
    let url = unsafe {
        event.URL().and_then(|u| {
            u.absoluteString().map(|s| nsstring_to_string(&s))
        })
    };

    // calendar() returns Option<Retained<EKCalendar>>
    let (calendar_id, calendar_title) = unsafe {
        event
            .calendar()
            .map(|cal| {
                (
                    Some(nsstring_to_string(&cal.calendarIdentifier())),
                    Some(nsstring_to_string(&cal.title())),
                )
            })
            .unwrap_or((None, None))
    };

    // Organizer returns Option
    let organizer = unsafe {
        event
            .organizer()
            .and_then(|p| p.name().map(|s| nsstring_to_string(&s)))
    };

    // Attendees returns Option<Retained<NSArray<EKParticipant>>>
    let attendees = unsafe {
        event
            .attendees()
            .map(|arr| {
                let mut result = Vec::new();
                let count = arr.count();
                for i in 0..count {
                    let attendee = arr.objectAtIndex(i);
                    result.push(participant_to_info(&attendee));
                }
                result
            })
            .unwrap_or_default()
    };

    // Has alarms
    let has_alarms = unsafe { event.hasAlarms() };

    // Is recurring
    let is_recurring = unsafe { event.hasRecurrenceRules() };

    EventInfo {
        id,
        title,
        start,
        end,
        all_day,
        location,
        notes,
        url,
        calendar_id,
        calendar_title,
        organizer,
        attendees,
        has_alarms,
        is_recurring,
    }
}

/// Convert EKParticipant to AttendeeInfo.
fn participant_to_info(participant: &EKParticipant) -> AttendeeInfo {
    // name() returns Option<Retained<NSString>>
    let name = unsafe { participant.name().map(|s| nsstring_to_string(&s)) };

    // URL() returns Retained<NSURL> directly
    // absoluteString() returns Option<Retained<NSString>>
    let email = unsafe {
        let url = participant.URL();
        url.absoluteString().and_then(|s| {
            let url_str = nsstring_to_string(&s);
            if url_str.starts_with("mailto:") {
                Some(url_str.trim_start_matches("mailto:").to_string())
            } else {
                None
            }
        })
    };

    let role = unsafe {
        match participant.participantRole() {
            EKParticipantRole::Unknown => "unknown",
            EKParticipantRole::Required => "required",
            EKParticipantRole::Optional => "optional",
            EKParticipantRole::Chair => "chair",
            EKParticipantRole::NonParticipant => "non_participant",
            _ => "unknown",
        }
        .to_string()
    };

    let status = unsafe {
        match participant.participantStatus() {
            EKParticipantStatus::Unknown => "unknown",
            EKParticipantStatus::Pending => "pending",
            EKParticipantStatus::Accepted => "accepted",
            EKParticipantStatus::Declined => "declined",
            EKParticipantStatus::Tentative => "tentative",
            EKParticipantStatus::Delegated => "delegated",
            EKParticipantStatus::Completed => "completed",
            EKParticipantStatus::InProcess => "in_process",
            _ => "unknown",
        }
        .to_string()
    };

    AttendeeInfo {
        name,
        email,
        role,
        status,
    }
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
        let status = CalendarStore::authorization_status();
        println!("Authorization status: {:?}", status);
    }

    #[test]
    fn test_nsdate_roundtrip() {
        let local = Local.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
        let nsdate = datetime_to_nsdate(local);
        let iso = nsdate_to_string(&nsdate);
        let parsed = DateTime::parse_from_rfc3339(&iso).expect("parse iso");

        assert_eq!(parsed.timestamp(), local.timestamp());
    }
}
