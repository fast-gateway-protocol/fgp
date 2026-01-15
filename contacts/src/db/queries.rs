//! AddressBook database queries.
//!
//! Core Data timestamps are seconds since 2001-01-01 00:00:00 UTC.
//!
//! CHANGELOG:
//! - 01/15/2026 - Initial implementation (Claude)

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::Serialize;

/// Core Data epoch: 2001-01-01 00:00:00 UTC
const CORE_DATA_EPOCH: i64 = 978307200;

/// Convert Core Data timestamp to ISO 8601 string.
pub fn core_data_to_iso(cd_ts: f64) -> String {
    let unix_ts = (cd_ts as i64) + CORE_DATA_EPOCH;
    chrono::DateTime::from_timestamp(unix_ts, 0)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Get Core Data timestamp for N days ago.
pub fn days_ago_core_data(days: u32) -> f64 {
    let now = chrono::Utc::now().timestamp();
    let cutoff = now - (days as i64 * 24 * 3600);
    (cutoff - CORE_DATA_EPOCH) as f64
}

// ============================================================================
// Contact Query Types
// ============================================================================

#[derive(Debug, Serialize, Clone)]
pub struct Contact {
    pub id: i64,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub organization: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub nickname: Option<String>,
    pub emails: Vec<EmailAddress>,
    pub phones: Vec<PhoneNumber>,
    pub modification_date: Option<String>,
}

impl Contact {
    /// Get display name (first + last, or organization, or nickname).
    pub fn display_name(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => format!("{} {}", first, last),
            (Some(first), None) => first.clone(),
            (None, Some(last)) => last.clone(),
            (None, None) => self
                .organization
                .clone()
                .or_else(|| self.nickname.clone())
                .unwrap_or_else(|| "(No Name)".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct EmailAddress {
    pub address: String,
    pub label: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct PhoneNumber {
    pub number: String,
    pub label: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug, Serialize)]
pub struct ContactGroup {
    pub id: i64,
    pub name: String,
    pub member_count: i64,
}

#[derive(Debug, Serialize)]
pub struct ContactStats {
    pub total_contacts: i64,
    pub with_email: i64,
    pub with_phone: i64,
    pub with_organization: i64,
    pub total_groups: i64,
}

// ============================================================================
// Contact Queries
// ============================================================================

/// Get all contacts with basic info.
pub fn query_contacts_list(conn: &Connection, limit: u32) -> Result<Vec<Contact>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            Z_PK,
            ZFIRSTNAME,
            ZLASTNAME,
            ZORGANIZATION,
            ZJOBTITLE,
            ZDEPARTMENT,
            ZNICKNAME,
            ZMODIFICATIONDATE
        FROM ZABCDRECORD
        WHERE ZFIRSTNAME IS NOT NULL
           OR ZLASTNAME IS NOT NULL
           OR ZORGANIZATION IS NOT NULL
        ORDER BY ZSORTINGLASTNAME, ZSORTINGFIRSTNAME
        LIMIT ?
        "#,
    )?;

    let contact_rows: Vec<(i64, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<f64>)> = stmt
        .query_map(params![limit], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut contacts = Vec::new();
    for (id, first, last, org, job, dept, nick, mod_date) in contact_rows {
        let emails = query_emails_for_contact(conn, id)?;
        let phones = query_phones_for_contact(conn, id)?;

        contacts.push(Contact {
            id,
            first_name: first,
            last_name: last,
            organization: org,
            job_title: job,
            department: dept,
            nickname: nick,
            emails,
            phones,
            modification_date: mod_date.map(core_data_to_iso),
        });
    }

    Ok(contacts)
}

/// Search contacts by name (fuzzy).
pub fn query_search_contacts(conn: &Connection, query: &str, limit: u32) -> Result<Vec<Contact>> {
    let search_pattern = format!("%{}%", query.to_lowercase());

    let mut stmt = conn.prepare(
        r#"
        SELECT
            Z_PK,
            ZFIRSTNAME,
            ZLASTNAME,
            ZORGANIZATION,
            ZJOBTITLE,
            ZDEPARTMENT,
            ZNICKNAME,
            ZMODIFICATIONDATE
        FROM ZABCDRECORD
        WHERE LOWER(ZFIRSTNAME) LIKE ?
           OR LOWER(ZLASTNAME) LIKE ?
           OR LOWER(ZORGANIZATION) LIKE ?
           OR LOWER(ZNICKNAME) LIKE ?
        ORDER BY ZSORTINGLASTNAME, ZSORTINGFIRSTNAME
        LIMIT ?
        "#,
    )?;

    let contact_rows: Vec<(i64, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<f64>)> = stmt
        .query_map(params![&search_pattern, &search_pattern, &search_pattern, &search_pattern, limit], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut contacts = Vec::new();
    for (id, first, last, org, job, dept, nick, mod_date) in contact_rows {
        let emails = query_emails_for_contact(conn, id)?;
        let phones = query_phones_for_contact(conn, id)?;

        contacts.push(Contact {
            id,
            first_name: first,
            last_name: last,
            organization: org,
            job_title: job,
            department: dept,
            nickname: nick,
            emails,
            phones,
            modification_date: mod_date.map(core_data_to_iso),
        });
    }

    Ok(contacts)
}

/// Find contact by email address.
pub fn query_contact_by_email(conn: &Connection, email: &str) -> Result<Option<Contact>> {
    let email_lower = email.to_lowercase();

    let mut stmt = conn.prepare(
        r#"
        SELECT
            r.Z_PK,
            r.ZFIRSTNAME,
            r.ZLASTNAME,
            r.ZORGANIZATION,
            r.ZJOBTITLE,
            r.ZDEPARTMENT,
            r.ZNICKNAME,
            r.ZMODIFICATIONDATE
        FROM ZABCDRECORD r
        JOIN ZABCDEMAILADDRESS e ON e.ZOWNER = r.Z_PK
        WHERE LOWER(e.ZADDRESS) = ?
        LIMIT 1
        "#,
    )?;

    let result = stmt.query_row(params![&email_lower], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, Option<f64>>(7)?,
        ))
    });

    match result {
        Ok((id, first, last, org, job, dept, nick, mod_date)) => {
            let emails = query_emails_for_contact(conn, id)?;
            let phones = query_phones_for_contact(conn, id)?;

            Ok(Some(Contact {
                id,
                first_name: first,
                last_name: last,
                organization: org,
                job_title: job,
                department: dept,
                nickname: nick,
                emails,
                phones,
                modification_date: mod_date.map(core_data_to_iso),
            }))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Find contact by phone number (matches last 4 digits or full number).
pub fn query_contact_by_phone(conn: &Connection, phone: &str) -> Result<Option<Contact>> {
    // Extract digits only for matching
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    let last_four = if digits.len() >= 4 {
        &digits[digits.len() - 4..]
    } else {
        &digits
    };

    let mut stmt = conn.prepare(
        r#"
        SELECT
            r.Z_PK,
            r.ZFIRSTNAME,
            r.ZLASTNAME,
            r.ZORGANIZATION,
            r.ZJOBTITLE,
            r.ZDEPARTMENT,
            r.ZNICKNAME,
            r.ZMODIFICATIONDATE
        FROM ZABCDRECORD r
        JOIN ZABCDPHONENUMBER p ON p.ZOWNER = r.Z_PK
        WHERE p.ZLASTFOURDIGITS = ? OR p.ZFULLNUMBER LIKE ?
        LIMIT 1
        "#,
    )?;

    let phone_pattern = format!("%{}%", digits);
    let result = stmt.query_row(params![last_four, &phone_pattern], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, Option<f64>>(7)?,
        ))
    });

    match result {
        Ok((id, first, last, org, job, dept, nick, mod_date)) => {
            let emails = query_emails_for_contact(conn, id)?;
            let phones = query_phones_for_contact(conn, id)?;

            Ok(Some(Contact {
                id,
                first_name: first,
                last_name: last,
                organization: org,
                job_title: job,
                department: dept,
                nickname: nick,
                emails,
                phones,
                modification_date: mod_date.map(core_data_to_iso),
            }))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Get recently modified contacts.
pub fn query_recent_contacts(conn: &Connection, days: u32, limit: u32) -> Result<Vec<Contact>> {
    let cutoff = days_ago_core_data(days);

    let mut stmt = conn.prepare(
        r#"
        SELECT
            Z_PK,
            ZFIRSTNAME,
            ZLASTNAME,
            ZORGANIZATION,
            ZJOBTITLE,
            ZDEPARTMENT,
            ZNICKNAME,
            ZMODIFICATIONDATE
        FROM ZABCDRECORD
        WHERE ZMODIFICATIONDATE > ?
          AND (ZFIRSTNAME IS NOT NULL OR ZLASTNAME IS NOT NULL OR ZORGANIZATION IS NOT NULL)
        ORDER BY ZMODIFICATIONDATE DESC
        LIMIT ?
        "#,
    )?;

    let contact_rows: Vec<(i64, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<f64>)> = stmt
        .query_map(params![cutoff, limit], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut contacts = Vec::new();
    for (id, first, last, org, job, dept, nick, mod_date) in contact_rows {
        let emails = query_emails_for_contact(conn, id)?;
        let phones = query_phones_for_contact(conn, id)?;

        contacts.push(Contact {
            id,
            first_name: first,
            last_name: last,
            organization: org,
            job_title: job,
            department: dept,
            nickname: nick,
            emails,
            phones,
            modification_date: mod_date.map(core_data_to_iso),
        });
    }

    Ok(contacts)
}

/// Get contact statistics.
pub fn query_contact_stats(conn: &Connection) -> Result<ContactStats> {
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ZABCDRECORD WHERE ZFIRSTNAME IS NOT NULL OR ZLASTNAME IS NOT NULL OR ZORGANIZATION IS NOT NULL",
        [],
        |row| row.get(0),
    )?;

    let with_email: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT ZOWNER) FROM ZABCDEMAILADDRESS WHERE ZADDRESS IS NOT NULL",
        [],
        |row| row.get(0),
    )?;

    let with_phone: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT ZOWNER) FROM ZABCDPHONENUMBER WHERE ZFULLNUMBER IS NOT NULL",
        [],
        |row| row.get(0),
    )?;

    let with_org: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ZABCDRECORD WHERE ZORGANIZATION IS NOT NULL AND ZORGANIZATION != ''",
        [],
        |row| row.get(0),
    )?;

    // Groups are also in ZABCDRECORD but with different Z_ENT value
    let groups: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM ZABCDRECORD WHERE ZNAME IS NOT NULL AND ZFIRSTNAME IS NULL AND ZLASTNAME IS NULL",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(ContactStats {
        total_contacts: total,
        with_email,
        with_phone,
        with_organization: with_org,
        total_groups: groups,
    })
}

// ============================================================================
// Helper Queries
// ============================================================================

/// Get email addresses for a contact.
fn query_emails_for_contact(conn: &Connection, contact_id: i64) -> Result<Vec<EmailAddress>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT ZADDRESS, ZLABEL, ZISPRIMARY
        FROM ZABCDEMAILADDRESS
        WHERE ZOWNER = ?
        ORDER BY ZORDERINGINDEX
        "#,
    )?;

    let emails = stmt
        .query_map(params![contact_id], |row| {
            Ok(EmailAddress {
                address: row.get(0)?,
                label: row.get(1)?,
                is_primary: row.get::<_, Option<i64>>(2)?.unwrap_or(0) == 1,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(emails)
}

/// Get phone numbers for a contact.
fn query_phones_for_contact(conn: &Connection, contact_id: i64) -> Result<Vec<PhoneNumber>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT ZFULLNUMBER, ZLABEL, ZISPRIMARY
        FROM ZABCDPHONENUMBER
        WHERE ZOWNER = ?
        ORDER BY ZORDERINGINDEX
        "#,
    )?;

    let phones = stmt
        .query_map(params![contact_id], |row| {
            Ok(PhoneNumber {
                number: row.get(0)?,
                label: row.get(1)?,
                is_primary: row.get::<_, Option<i64>>(2)?.unwrap_or(0) == 1,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(phones)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_days_ago() {
        let cutoff = days_ago_core_data(7);
        assert!(cutoff > 0.0);
    }
}
